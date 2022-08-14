mod config;
mod util;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

fn main() {
    let app = util::application();

    app.connect_activate(|app| {
        let cfg_file = config::read_config();
    });

    app.run();
}
