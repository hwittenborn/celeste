use adw::glib::{self, MainContext};

use futures::future::Future;
use std::path::PathBuf;

pub static APP_ID: &str = "com.hunterwittenborn.Celeste";

/// Get the value out of a future.
pub fn await_future<F: Future>(future: F) -> F::Output {
    futures::executor::block_on(future)
}

/// Run a closure in the background so that the UI can keep running.
pub fn run_in_background<T: Send + 'static, F: FnOnce() -> T + Send + 'static>(f: F) -> T {
    MainContext::default().block_on(blocking::unblock(f))
}

/// Format a directory with the user's home directory replaced with '~'.
pub fn fmt_home(dir: &str) -> String {
    let home_dir = glib::home_dir().into_os_string().into_string().unwrap();

    match dir.strip_prefix(&home_dir) {
        Some(string) => "~".to_string() + string,
        None => dir.to_string(),
    }
}

/// Get the user's config directory.
pub fn get_config_dir() -> PathBuf {
    let mut config_dir = glib::user_config_dir();
    config_dir.push("celeste");
    config_dir
}

/// Get the lockfile that's used to check if a Celeste instance is running.
pub fn is_running_file() -> PathBuf {
    let mut dir = glib::user_config_dir();
    dir.push("celeste");
    dir.push("running.lock");
    dir
}

/// Get the file that's used to open a running Celeste instance.
pub fn notify_open_file() -> PathBuf {
    let mut dir = glib::user_config_dir();
    dir.push("celeste");
    dir.push("notify.lock");
    dir
}

/// Strip the slashes from the beginning and end of a string.
pub fn strip_slashes(string: &str) -> String {
    let stripped_prefix = match string.strip_prefix('/') {
        Some(string) => string.to_string(),
        None => string.to_string(),
    };

    match stripped_prefix.strip_suffix('/') {
        Some(string) => string.to_string(),
        None => stripped_prefix,
    }
}

/// Macro to get the title of a window.
macro_rules! get_title {
    ($($arg:tt)*) => {
        format!($($arg)*) + " - Celeste"
    }
}
pub(crate) use get_title;
