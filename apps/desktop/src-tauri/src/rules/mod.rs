use anyhow::{Result, Context};
use serde::Deserialize;
use tauri::AppHandle;
use crate::db::Db;
use rusqlite::params;

#[derive(Debug, Deserialize)]
#[serde(rename_all="snake_case")]
pub struct RuleYaml {
  pub id: String,
  pub name: String,
  pub enabled: Option<bool>,
  pub target: String,
  #[serde(rename="where")]
  pub where_: serde_json::Value
}

pub fn load_and_compile(app: &AppHandle, db: &Db) -> Result<usize> {
  let path = app.path().app_data_dir().unwrap().join("rules.yaml");
  if !path.exists() { return Ok(0); }
  let txt = std::fs::read_to_string(path)?;
  let rules: Vec<RuleYaml> = serde_yaml::from_str(&txt).context("parse rules.yaml")?;
  let mut tx = db.conn.unchecked_transaction()?;
  for r in rules {
    let en = r.enabled.unwrap_or(true);
    let spec = serde_json::to_string(&r.where_)?;
    tx.execute("INSERT OR REPLACE INTO rule(id,name,enabled,spec_json,updated_at) VALUES (?1,?2,?3,?4,strftime('%s','now'))",
      params![r.id, r.name, if en {1} else {0}, spec])?;
  }
  tx.commit()?;
  Ok(rules.len())
}
