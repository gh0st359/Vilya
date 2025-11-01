use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct Db {
  pub conn: Connection
}

impl Db {
  pub fn open(path: PathBuf) -> Result<Self> {
    let conn = Connection::open(path)?;
    conn.execute_batch(include_str!("migrations.sql"))?;
    Ok(Self { conn })
  }
}
