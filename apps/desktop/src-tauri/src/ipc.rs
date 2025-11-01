use crate::db::Db;
use tauri::State;
use rusqlite::params;

#[derive(serde::Serialize)]
pub struct UiEvent { pub id: String, pub title: String, pub class: String, pub lat: f64, pub lon: f64, pub severity: f32, pub ts: i64 }

#[tauri::command]
pub fn search_events(db: State<Db>, q: Option<String>, _since: Option<i64>, _until: Option<i64>) -> Result<Vec<UiEvent>, String> {
  let mut stmt = db.conn.prepare("SELECT id,title,class,lat,lon,severity,first_seen FROM event WHERE (?1 IS NULL OR title LIKE '%'||?1||'%') ORDER BY first_seen DESC LIMIT 1000").map_err(|e| e.to_string())?;
  let rows = stmt.query_map([q], |r| {
    Ok(UiEvent{
      id: r.get(0)?, title: r.get(1)?, class: r.get(2)?, lat: r.get(3)?, lon: r.get(4)?,
      severity: r.get::<_, f64>(5)? as f32, ts: r.get(6)?
    })
  }).map_err(|e| e.to_string())?;
  Ok(rows.filter_map(|x| x.ok()).collect())
}

#[tauri::command]
pub fn get_event(db: State<Db>, id: String) -> Result<String, String> {
  let mut stmt = db.conn.prepare(
    "SELECT json_object('id',id,'title',title,'summary',summary,'class',class,'geojson',geojson,'severity',severity,'confidence',confidence,'first_seen',first_seen,'last_seen',last_seen) FROM event WHERE id=?1"
  ).map_err(|e| e.to_string())?;
  let s: String = stmt.query_row([id], |r| r.get(0)).map_err(|e| e.to_string())?;
  Ok(s)
}

#[derive(serde::Serialize)]
pub struct UiAlert {
  pub id: String, pub headline: String, pub event: String,
  pub severity: String, pub urgency: String, pub certainty: String,
  pub onset: i64, pub expires: i64,
  pub bbox: (f64,f64,f64,f64),
  pub geojson: Option<String>
}

#[tauri::command]
pub fn query_alerts(db: State<Db>, minx: f64, miny: f64, maxx: f64, maxy: f64, now_after: i64) -> Result<Vec<UiAlert>, String> {
  let mut stmt = db.conn.prepare(
    "SELECT a.id,a.headline,a.event,a.severity,a.urgency,a.certainty,a.onset,a.expires,a.polygon_geojson,a.bbox_minx,a.bbox_miny,a.bbox_maxx,a.bbox_maxy
     FROM alert_rtree r JOIN alert a ON a.rowid = r.rowid
     WHERE r.minx <= ?3 AND r.maxx >= ?1 AND r.miny <= ?4 AND r.maxy >= ?2
       AND a.expires >= ?5
     ORDER BY a.severity DESC, a.onset DESC LIMIT 500"
  ).map_err(|e| e.to_string())?;

  let rows = stmt.query_map(params![minx, miny, maxx, maxy, now_after], |r| {
    Ok(UiAlert{
      id: r.get(0)?, headline: r.get(1)?, event: r.get(2)?,
      severity: r.get(3)?, urgency: r.get(4)?, certainty: r.get(5)?,
      onset: r.get(6)?, expires: r.get(7)?,
      geojson: r.get(8)?,
      bbox: (r.get(9)?, r.get(10)?, r.get(11)?, r.get(12)?)
    })
  }).map_err(|e| e.to_string())?;

  Ok(rows.filter_map(|x| x.ok()).collect())
}

#[tauri::command]
pub fn analytics_daily(db: State<Db>) -> Result<Vec<(String,i64)>, String> {
  let mut stmt = db.conn.prepare(
    "SELECT strftime('%Y-%m-%d', datetime(first_seen,'unixepoch')) AS d, COUNT(1)
     FROM event WHERE first_seen >= strftime('%s','now','-30 day')
     GROUP BY d ORDER BY d"
  ).map_err(|e| e.to_string())?;
  let rows = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?))).map_err(|e| e.to_string())?;
  Ok(rows.filter_map(|x| x.ok()).collect())
}

#[tauri::command]
pub fn analytics_by_class(db: State<Db>) -> Result<Vec<(String,i64)>, String> {
  let mut stmt = db.conn.prepare(
    "SELECT class, COUNT(1)
     FROM event WHERE first_seen >= strftime('%s','now','-7 day')
     GROUP BY class ORDER BY COUNT(1) DESC"
  ).map_err(|e| e.to_string())?;
  let rows = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?))).map_err(|e| e.to_string())?;
  Ok(rows.filter_map(|x| x.ok()).collect())
}
