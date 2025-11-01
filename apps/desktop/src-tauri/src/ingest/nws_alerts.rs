use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use geojson::{Feature, FeatureCollection, GeoJson, Value};
use rusqlite::params;
use tauri::{AppHandle, Manager};
use crate::db::Db;

fn to_epoch(ts: Option<&str>) -> Option<i64> {
  ts.and_then(|s| DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc).timestamp())
}

fn bbox_of(value: &Value) -> Option<(f64,f64,f64,f64)> {
  match value {
    Value::Polygon(poly) => {
      let coords = &poly[0];
      let (mut minx, mut miny, mut maxx, mut maxy) = (f64::INFINITY,f64::INFINITY,f64::NEG_INFINITY,f64::NEG_INFINITY);
      for c in coords {
        if c.len() >= 2 {
          let (x,y) = (c[0], c[1]);
          if x<minx {minx=x;} if y<miny {miny=y;} if x>maxx {maxx=x;} if y>maxy {maxy=y;}
        }
      }
      Some((minx,miny,maxx,maxy))
    }
    Value::MultiPolygon(mpoly) => {
      let mut minx=f64::INFINITY; let mut miny=f64::INFINITY; let mut maxx=f64::NEG_INFINITY; let mut maxy=f64::NEG_INFINITY;
      for poly in mpoly {
        for c in &poly[0] {
          if c.len()>=2 { let (x,y)=(c[0],c[1]); if x<minx{minx=x;} if y<miny{miny=y;} if x>maxx{maxx=x;} if y>maxy{maxy=y;} }
        }
      }
      if minx.is_finite() { Some((minx,miny,maxx,maxy)) } else { None }
    }
    _ => None
  }
}

pub async fn run(app: &AppHandle) -> Result<()> {
  let url = "https://api.weather.gov/alerts/active?limit=200";
  let body = reqwest::get(url).await?.error_for_status()?.text().await?;
  let gj: GeoJson = body.parse().context("parse nws geojson")?;
  let db: &Db = app.state::<Db>().inner();

  if let GeoJson::FeatureCollection(FeatureCollection { features, .. }) = gj {
    let now = chrono::Utc::now().timestamp();
    let mut tx = db.conn.unchecked_transaction()?;
    for f in features {
      persist_feature(&mut tx, f, now)?;
    }
    tx.commit()?;
  }
  Ok(())
}

fn persist_feature(conn: &mut rusqlite::Connection, f: Feature, now: i64) -> Result<()> {
  let id = f.id.clone().map(|v| v.to_string()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
  let props = f.properties.unwrap_or_default();

  let headline = props.get("headline").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let event = props.get("event").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let severity = props.get("severity").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
  let urgency = props.get("urgency").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
  let certainty = props.get("certainty").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
  let area = props.get("areaDesc").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let onset = to_epoch(props.get("effective").and_then(|v| v.as_str())).or_else(|| to_epoch(props.get("onset").and_then(|v| v.as_str()))).unwrap_or(now);
  let sent = to_epoch(props.get("sent").and_then(|v| v.as_str())).unwrap_or(now);
  let expires = to_epoch(props.get("expires").and_then(|v| v.as_str())).unwrap_or(now + 3600);

  let geojson_text = serde_json::to_string(&f.geometry).ok().unwrap_or("null".into());
  let (minx, miny, maxx, maxy) = f.geometry.as_ref()
    .and_then(|g| bbox_of(&g.value))
    .unwrap_or((-180.0,-90.0,180.0,90.0));

  let raw_json = serde_json::to_string(&f).unwrap_or_default();

  conn.execute(
    "INSERT OR REPLACE INTO alert(id, source, headline, event, severity, urgency, certainty, onset, sent, expires, area_desc, polygon_geojson, bbox_minx, bbox_miny, bbox_maxx, bbox_maxy, raw_json, last_seen)
     VALUES (?1,'nws',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
    params![id, headline, event, severity, urgency, certainty, onset, sent, expires, area, geojson_text, minx, miny, maxx, maxy, raw_json, now]
  )?;

  let rowid: i64 = conn.query_row("SELECT rowid FROM alert WHERE id=?1", params![id], |r| r.get(0)).unwrap();
  conn.execute("INSERT OR REPLACE INTO alert_rtree(rowid,minx,maxx,miny,maxy) VALUES (?1,?2,?3,?4,?5)",
    params![rowid, minx, maxx, miny, maxy])?;

  Ok(())
}
