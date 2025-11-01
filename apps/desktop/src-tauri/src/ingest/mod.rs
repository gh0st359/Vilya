use tauri::AppHandle;
use tokio::time::{sleep, Duration};
use tracing::warn;

pub mod gdacs; pub mod usgs; pub mod eonet; pub mod emsc_ws;
pub mod nws_alerts; pub mod cap_generic;

pub fn spawn_collectors(app: AppHandle) {
  let h = app.clone(); tokio::spawn(async move { loop { if let Err(e)=gdacs::run(&h).await { warn!("gdacs: {e}"); } sleep(Duration::from_secs(90)).await; }});
  let h = app.clone(); tokio::spawn(async move { loop { if let Err(e)=usgs::run(&h).await  { warn!("usgs: {e}"); } sleep(Duration::from_secs(60)).await; }});
  let h = app.clone(); tokio::spawn(async move { loop { if let Err(e)=eonet::run(&h).await { warn!("eonet: {e}"); } sleep(Duration::from_secs(180)).await; }});
  let h = app.clone(); tokio::spawn(async move { if let Err(e)=emsc_ws::run(&h).await { warn!("emsc_ws: {e}"); }});
  let h = app.clone(); tokio::spawn(async move { loop { if let Err(e)=nws_alerts::run(&h).await { warn!("nws: {e}"); } sleep(Duration::from_secs(75)).await; }});
  // optional CAP XML feeds after configuring URLs
  // let h = app.clone(); tokio::spawn(async move { loop { if let Err(e)=cap_generic::run(&h).await { warn!("cap: {e}"); } sleep(Duration::from_secs(180)).await; }});
}
