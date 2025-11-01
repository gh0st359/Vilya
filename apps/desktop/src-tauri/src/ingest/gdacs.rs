use anyhow::Result;
use reqwest::StatusCode;
use tauri::AppHandle;
use tracing::info;
use crate::db::Db;
use tauri::Manager;

pub async fn run(app: &AppHandle) -> Result<()> {
  let db: &Db = app.state::<Db>().inner();
  let url = "https://www.gdacs.org/gdacsapi/api/Events/geteventlist/SEARCH?pageSize=100&pageNumber=1";
  let client = reqwest::Client::new();
  let resp = client.get(url).send().await?;
  if resp.status()==StatusCode::OK {
    let v: serde_json::Value = resp.json().await?;
    let arr = v.get("features").or_else(|| v.get("events")).cloned().unwrap_or(serde_json::json!([]));
    persist_gdacs(&db, arr).await?;
    info!("gdacs: ok");
  }
  Ok(())
}

async fn persist_gdacs(db: &Db, items: serde_json::Value) -> Result<()> {
  let mut tx = db.conn.unchecked_transaction()?;
  if let Some(arr)=items.as_array() {
    for it in arr {
      let title = it.pointer("/properties/eventname").or_else(|| it.get("title")).and_then(|x| x.as_str()).unwrap_or("GDACS event");
      let lat = it.pointer("/geometry/coordinates/1").and_then(|x| x.as_f64()).unwrap_or(0.0);
      let lon = it.pointer("/geometry/coordinates/0").and_then(|x| x.as_f64()).unwrap_or(0.0);
      let id = it.pointer("/properties/eventid").and_then(|x| x.as_str()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string().leak()).to_string();
      tx.execute("INSERT OR IGNORE INTO event(id,first_seen,last_seen,title,summary,class,severity,confidence,lat,lon,geojson,source_rank)
        VALUES (?1,strftime('%s','now'),strftime('%s','now'),?2,?3,?4,0.5,0.9,?5,?6,?7,10)",
        rusqlite::params![id, title, title, "alert", lat, lon, it.to_string()])?;
    }
  }
  tx.commit()?;
  Ok(())
}
