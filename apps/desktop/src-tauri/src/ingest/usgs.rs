use anyhow::Result;
use tauri::AppHandle;
use crate::db::Db;
use tauri::Manager;

pub async fn run(app: &AppHandle) -> Result<()> {
  let db: &Db = app.state::<Db>().inner();
  let url = "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/all_hour.geojson";
  let v: serde_json::Value = reqwest::get(url).await?.json().await?;
  if let Some(arr)=v.get("features").and_then(|x| x.as_array()) {
    let mut tx = db.conn.unchecked_transaction()?;
    for f in arr {
      let id = f.get("id").and_then(|x| x.as_str()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string().leak()).to_string();
      let props = f.get("properties").unwrap_or(&serde_json::json!({}));
      let title = props.get("title").and_then(|x| x.as_str()).unwrap_or("USGS event");
      let mag = props.get("mag").and_then(|x| x.as_f64()).unwrap_or(0.0);
      let coords = f.pointer("/geometry/coordinates").and_then(|x| x.as_array()).cloned().unwrap_or_default();
      let lon = coords.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0);
      let lat = coords.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0);
      tx.execute("INSERT OR REPLACE INTO event(id,first_seen,last_seen,title,summary,class,severity,confidence,lat,lon,geojson,source_rank)
        VALUES (?1,COALESCE((SELECT first_seen FROM event WHERE id=?1),strftime('%s','now')),strftime('%s','now'),?2,?2,'eq',?3,0.95,?4,?5,?6,5)",
        rusqlite::params![id, title, (mag/10.0).clamp(0.0,1.0), lat, lon, f.to_string()])?;
    }
    tx.commit()?;
  }
  Ok(())
}
