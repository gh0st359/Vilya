use rusqlite::params;
use crate::db::Db;

pub fn dedup_alert_into_events(db: &Db) -> anyhow::Result<usize> {
  let mut created = 0usize;
  let mut stmt = db.conn.prepare("SELECT id,event,severity,(bbox_minx+bbox_maxx)/2.0 AS cx,(bbox_miny+bbox_maxy)/2.0 AS cy,onset
                                  FROM alert WHERE severity IN ('Severe','Extreme')")?;
  let mut rows = stmt.query([])?;
  while let Some(r) = rows.next()? {
    let _id: String = r.get(0)?; let name: String = r.get(1)?;
    let _sev: String = r.get(2)?; let cx: f64 = r.get(3)?; let cy: f64 = r.get(4)?; let onset: i64 = r.get(5)?;
    let key = format!("cap:{}:{:.2}:{:.2}", name, cx, cy);
    db.conn.execute(
      "INSERT OR IGNORE INTO event(id,first_seen,last_seen,title,summary,class,severity,confidence,lat,lon,geojson,source_rank)
       VALUES (?1,?2,?2,?3,?3,'alert',?4,0.9,?5,?6,NULL,1)",
      params![key, onset, name, 0.9f64, cy, cx]
    )?;
    if db.conn.changes() > 0 { created += 1; }
  }
  Ok(created)
}
