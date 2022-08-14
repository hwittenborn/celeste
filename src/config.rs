//! Utilities to manage the config file.
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Result, ErrorKind},
};
use crate::util;

use gtk::{prelude::*, ApplicationWindow, Label};
struct Config {
    username: String,
    password: String,
}

/// Read and parse the config.
pub fn read_config() -> File {
    // Not quite sure if/when finding the config dir can fail. We'll wait until any bug reports
    // come in.
    let config_dir_path = dirs::config_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
        + "/cluff";
    let config_file_path = config_dir_path.clone() + "/config.toml";

    let file_result = match File::open(&config_file_path) {
        Ok(file) => Ok(file),
        Err(err) => {
            if let ErrorKind::NotFound = err.kind() {
                File::create(config_file_path)
            } else {
                Err(err)
            }
        }
    };

    match file_result {
        Ok(file) => file,
        Err(err) => {
            let app = util::application();
            app.connect_activate(|app| {
                util::create_text_window(app, "Error", "WOW!");
            });
            app.run();
            quit::with_code(exitcode::IOERR);
        }
    }
}
