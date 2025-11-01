use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::{sync::mpsc, time::{sleep, Duration}};
use crate::db::Db;
use rusqlite::params;

#[derive(Debug, Serialize, Deserialize)]
struct AiOutput {
  class: String,
  confidence: f32,
  severity: f32,
  entities: Vec<String>
}

pub fn spawn(app: AppHandle) {
  let (tx, mut rx) = mpsc::channel::<String>(100);
  app.manage(AiTx(tx));

  tokio::spawn(async move {
    loop {
      if let Some(eid) = rx.recv().await {
        if let Err(e) = process_event(&app, &eid).await {
          tracing::warn!("ai process {}: {}", eid, e);
        }
        sleep(Duration::from_millis(200)).await;
      }
    }
  });
}

pub struct AiTx(pub mpsc::Sender<String>);

async fn process_event(app: &AppHandle, event_id: &str) -> Result<()> {
  let db: &Db = app.state::<Db>().inner();
  let already: Option<i64> = db.conn.query_row("SELECT 1 FROM ai_labels WHERE event_id=?1", params![event_id], |r| r.get(0)).optional()?;
  if already.is_some() { return Ok(()); }

  let (title, summary) : (String,String) = db.conn.query_row(
    "SELECT COALESCE(title,''), COALESCE(summary,'') FROM event WHERE id=?1",
    params![event_id], |r| Ok((r.get(0)?, r.get(1)?)))?;

  let prompt = format!(r#"You are a crisis-event classifier.
Given the text below, output strict JSON with keys: class, confidence, severity, entities.
- class ∈ [eq, volcano, wildfire, flood, storm, conflict, protest, alert, aviation, other]
- confidence ∈ [0,1]
- severity ∈ [0,1]
- entities is an array of key proper nouns or locations.

Text:
{}
{}
JSON:"# , title, summary);

  let body = serde_json::json!({
    "model":"llama3.1:8b",
    "prompt": prompt,
    "stream": false,
    "options": { "temperature": 0.1 }
  });

  let resp = reqwest::Client::new()
    .post("http://127.0.0.1:11434/api/generate")
    .json(&body).send().await?
    .error_for_status()?
    .json::<serde_json::Value>().await?;

  let raw = resp.get("response").and_then(|v| v.as_str()).unwrap_or("{}");
  let parsed: AiOutput = serde_json::from_str(raw).unwrap_or(AiOutput{ class:"other".into(), confidence:0.5, severity:0.3, entities:vec![] });

  let mut tx = db.conn.unchecked_transaction()?;
  tx.execute("INSERT OR REPLACE INTO ai_labels(event_id,labels_json,severity) VALUES (?1,?2,?3)",
    params![event_id, serde_json::to_string(&parsed)?, parsed.severity])?;
  tx.execute("UPDATE event SET severity = MAX(severity, ?2) WHERE id=?1",
    params![event_id, parsed.severity])?;
  tx.commit()?;
  app.emit("ai_label", serde_json::json!({"id": event_id, "labels": parsed}))?;
  Ok(())
}

pub fn enqueue(app: &AppHandle, event_id: &str) {
  if let Some(tx) = app.try_state::<AiTx>() {
    let _ = tx.0.try_send(event_id.to_string());
  }
}
