use anyhow::Result;
use tauri::AppHandle;
use crate::db::Db;
use tauri::Manager;

pub async fn run(app: &AppHandle) -> Result<()> {
  let db: &Db = app.state::<Db>().inner();
  let url = "https://eonet.gsfc.nasa.gov/api/v3/events?status=open&limit=50";
  let v: serde_json::Value = reqwest::get(url).await?.json().await?;
  if let Some(arr)=v.get("events").and_then(|x| x.as_array()) {
    let mut tx = db.conn.unchecked_transaction()?;
    for e in arr {
      let id = e.get("id").and_then(|x| x.as_str()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string().leak()).to_string();
      let title = e.get("title").and_then(|x| x.as_str()).unwrap_or("EONET event");
      let class = e.get("categories").and_then(|c| c.get(0)).and_then(|c| c.get("id")).and_then(|x| x.as_str()).unwrap_or("natural");
      let coords = e.pointer("/geometry/0/coordinates").and_then(|x| x.as_array()).cloned().unwrap_or_default();
      let lon = coords.get(0).and_then(|x| x.as_f64()).unwrap_or(0.0);
      let lat = coords.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0);
      tx.execute("INSERT OR REPLACE INTO event(id,first_seen,last_seen,title,summary,class,severity,confidence,lat,lon,geojson,source_rank)
        VALUES (?1,COALESCE((SELECT first_seen FROM event WHERE id=?1),strftime('%s','now')),strftime('%s','now'),?2,?2,?3,0.6,0.8,?4,?5,?6,8)",
        rusqlite::params![id, title, class, lat, lon, e.to_string()])?;
    }
    tx.commit()?;
  }
  Ok(())
}
