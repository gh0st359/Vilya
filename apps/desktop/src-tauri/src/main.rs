#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod db; mod ingest; mod merge; mod normalize; mod ai; mod telemetry; mod ipc; mod rules;

use anyhow::Result;
use std::path::PathBuf;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

fn data_dir(app: &tauri::App) -> PathBuf {
  app.path().app_data_dir().expect("data dir")
}

#[tauri::command]
async fn ping() -> String { "pong".into() }

#[tokio::main]
async fn main() -> Result<()> {
  tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default().build())
    .setup(|app| {
      let db_path = data_dir(app).join("vilya.sqlite");
      let db = db::Db::open(db_path).expect("db");
      app.manage(db);
      {
        let dbr: &db::Db = app.state::<db::Db>().inner();
        let _ = rules::load_and_compile(app, dbr);
      }
      ai::spawn(app.handle());
      ingest::spawn_collectors(app.handle());
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      ping, ipc::search_events, ipc::get_event, ipc::query_alerts,
      ipc::analytics_daily, ipc::analytics_by_class
    ])
    .run(tauri::generate_context!())
    .expect("error running app");
  Ok(())
}
