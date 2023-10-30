#![feature(let_chains)]
#![feature(arc_unwrap_or_clone)]
#![feature(panic_info_message)]
#![feature(async_closure)]
#![feature(trait_alias)]
#![feature(exit_status_error)]

pub mod about;
pub mod entities;
pub mod gtk_util;
pub mod launch;
pub mod login;
pub mod migrations;
pub mod mpsc;
pub mod rclone;
pub mod traits;
// pub mod tray;
pub mod util;

use relm4::prelude::*;
use serde_json::json;

fn main() {
    // Configure Rclone.
    let mut config = util::get_config_dir();
    config.push("rclone.conf");
    librclone::initialize();
    librclone::rpc("config/setpath", json!({ "path": config }).to_string()).unwrap();

    // Setup our CSS and run the App.
    let app = RelmApp::new(util::APP_ID);
    relm4::set_global_css(include_str!(concat!(env!("OUT_DIR"), "/style.css")));
    app.set_visible(false).run_async::<launch::LaunchModel>(());
}
