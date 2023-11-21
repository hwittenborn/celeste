#![feature(let_chains)]
#![feature(arc_unwrap_or_clone)]
#![feature(panic_info_message)]
#![feature(async_closure)]
#![feature(trait_alias)]
#![feature(exit_status_error)]
#![feature(lazy_cell)]

pub mod gtk_util;
pub mod launch;
pub mod login;
pub mod rclone;
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
    relm4_icons::initialize_icons();
    relm4::set_global_css(include_str!(concat!(env!("OUT_DIR"), "/style.css")));
    app.visible_on_activate(false)
        .run_async::<launch::LaunchModel>(());
}
