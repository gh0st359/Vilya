use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tauri::AppHandle;
use tokio_tungstenite::connect_async;
use crate::db::Db;
use tauri::Manager;

pub async fn run(app: &AppHandle) -> Result<()> {
  let db: &Db = app.state::<Db>().inner();
  let (mut ws, _) = connect_async("wss://www.seismicportal.eu/standing_order/websocket").await?;
  ws.send(tokio_tungstenite::tungstenite::Message::Text(r#"{"subscribe":"quakes"}"#.into())).await?;
  while let Some(msg) = ws.next().await {
    let txt = msg?.into_text()?;
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
      if let Some(features)=v.get("features").and_then(|x| x.as_array()) {
        let mut tx = db.conn.unchecked_transaction()?;
        for f in features {
          let id = f.get("id").and_then(|x| x.as_str()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string().leak()).to_string();
          let props = f.get("properties").unwrap_or(&serde_json::json!({}));
          let mag = props.get("mag").and_then(|x| x.as_f64()).unwrap_or(0.0);
          let title = format!("EMSC M{:.1} {}", mag, props.get("flynn_region").and_then(|x| x.as_str()).unwrap_or(""));
          let coords = f.pointer("/geometry/coordinates").and_then(|x| x.as_array()).cloned().unwrap_or_default();
          let lon = coords.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0);
          let lat = coords.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0);
          tx.execute("INSERT OR REPLACE INTO event(id,first_seen,last_seen,title,summary,class,severity,confidence,lat,lon,geojson,source_rank)
            VALUES (?1,COALESCE((SELECT first_seen FROM event WHERE id=?1),strftime('%s','now')),strftime('%s','now'),?2,?2,'eq',?3,0.95,?4,?5,?6,4)",
            rusqlite::params![id, title, (mag/10.0).clamp(0.0,1.0), lat, lon, f.to_string()])?;
        }
        tx.commit()?;
      }
    }
  }
  Ok(())
}
