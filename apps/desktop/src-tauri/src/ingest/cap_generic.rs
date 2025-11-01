use anyhow::Result;
use quick_xml::events::Event as XEvent;
use quick_xml::Reader;
use tauri::{AppHandle, Manager};
use crate::db::Db;
use rusqlite::params;

const CAP_URLS: &[&str] = &[];

pub async fn run(app: &AppHandle) -> Result<()> {
  if CAP_URLS.is_empty() { return Ok(()); }
  let db: &Db = app.state::<Db>().inner();
  let now = chrono::Utc::now().timestamp();
  let client = reqwest::Client::new();

  for url in CAP_URLS {
    let text = client.get(*url).send().await?.error_for_status()?.text().await?;
    let mut reader = Reader::from_str(&text);
    reader.trim_text(true);
    let mut buf = Vec::new();

    let mut identifier = String::new();
    let mut headline = String::new();
    let mut event = String::new();
    let mut severity = String::new();
    let mut urgency = String::new();
    let mut certainty = String::new();
    let mut area_desc = String::new();
    let mut sent = String::new();
    let mut effective = String::new();
    let mut expires = String::new();
    let mut polygon_texts: Vec<String> = Vec::new();
    let mut in_elem = String::new();

    loop {
      match reader.read_event_into(&mut buf) {
        Ok(XEvent::Start(e)) => { in_elem = String::from_utf8_lossy(e.name().as_ref()).to_string(); }
        Ok(XEvent::Text(t)) => {
          let v = t.unescape().unwrap_or_default().to_string();
          match in_elem.as_str() {
            "identifier" => identifier = v,
            "headline" => headline = v,
            "event" => event = v,
            "severity" => severity = v,
            "urgency" => urgency = v,
            "certainty" => certainty = v,
            "areaDesc" => area_desc = v,
            "sent" => sent = v,
            "effective" => effective = v,
            "expires" => expires = v,
            "polygon" => polygon_texts.push(v),
            _ => {}
          }
        }
        Ok(XEvent::Eof) => break,
        _ => {}
      }
      buf.clear();
    }

    let id = if identifier.is_empty() { uuid::Uuid::new_v4().to_string() } else { identifier };
    let onset = parse_ts(&effective).unwrap_or(now);
    let sent_e = parse_ts(&sent).unwrap_or(now);
    let exp = parse_ts(&expires).unwrap_or(now + 3600);

    let (geojson_text, minx, miny, maxx, maxy) = cap_polygons_to_geojson_bbox(&polygon_texts);

    let mut tx = db.conn.unchecked_transaction()?;
    tx.execute(
      "INSERT OR REPLACE INTO alert(id, source, headline, event, severity, urgency, certainty, onset, sent, expires, area_desc, polygon_geojson, bbox_minx, bbox_miny, bbox_maxx, bbox_maxy, raw_json, last_seen)
       VALUES (?1,'cap',?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
      params![id, headline, event, severity, urgency, certainty, onset, sent_e, exp, area_desc, geojson_text, minx, miny, maxx, maxy, text, now]
    )?;
    let rowid: i64 = tx.query_row("SELECT rowid FROM alert WHERE id=?1", params![id], |r| r.get(0))?;
    tx.execute("INSERT OR REPLACE INTO alert_rtree(rowid,minx,maxx,miny,maxy) VALUES (?1,?2,?3,?4,?5)",
      params![rowid, minx, maxx, miny, maxy])?;
    tx.commit()?;
  }
  Ok(())
}

fn parse_ts(s: &str) -> Option<i64> {
  chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&chrono::Utc).timestamp())
}

fn cap_polygons_to_geojson_bbox(polys: &[String]) -> (String,f64,f64,f64,f64) {
  use geojson::{Geometry, Value, Feature, FeatureCollection};
  let mut features = Vec::new();
  let mut minx=f64::INFINITY; let mut miny=f64::INFINITY; let mut maxx=f64::NEG_INFINITY; let mut maxy=f64::NEG_INFINITY;

  for p in polys {
    let ring: Vec<Vec<f64>> = p
      .split_whitespace()
      .filter_map(|pair| {
        let mut it = pair.split(',');
        let lat = it.next()?.parse::<f64>().ok()?;
        let lon = it.next()?.parse::<f64>().ok()?;
        Some(vec![lon, lat])
      })
      .collect();

    if ring.len() >= 3 {
      for pt in &ring { let (x,y)=(pt[0],pt[1]); if x<minx{minx=x;} if y<miny{miny=y;} if x>maxx{maxx=x;} if y>maxy{maxy=y;} }
      let mut closed = ring.clone();
      if closed.first() != closed.last() { closed.push(closed[0].clone()); }
      let geom = Geometry::new(Value::Polygon(vec![closed]));
      features.push(Feature{ geometry: Some(geom), properties: None, bbox: None, id: None, foreign_members: None });
    }
  }

  let fc = FeatureCollection{ features, bbox: None, foreign_members: None };
  let txt = serde_json::to_string(&geojson::GeoJson::from(fc)).unwrap_or("null".into());
  if !minx.is_finite() { return ("null".into(), -180.0,-90.0,180.0,90.0); }
  (txt, minx, miny, maxx, maxy)
}
