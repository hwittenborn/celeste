pub mod traits;

use futures::future::Future;
use glib::{self, MainContext};
use std::path::PathBuf;

/// The ID of the app.
pub static APP_ID: &str = "com.hunterwittenborn.Celeste";

/// The ID of the DBus app.
/// We have to have a separate ID because our GTK application registers the DBus
/// connection for `APP_ID`. See the conversation at
/// https://matrix.to/#/!CxdTjqASmMdXwTeLsR:matrix.org/$16727498910mwIiT:hunterwittenborn.com?via=gnome.org&via=matrix.org&via=tchncs.de
/// for more info.
pub static DBUS_APP_ID: &str = "com.hunterwittenborn.CelesteApp";

/// The DBus object of the DBus app.
pub static DBUS_APP_OBJECT: &str = "/com/hunterwittenborn/CelesteApp";

/// The ID of the tray icon.
pub static TRAY_ID: &str = "com.hunterwittenborn.CelesteTray";

/// The DBus object of the tray icon.
pub static DBUS_TRAY_OBJECT: &str = "/com/hunterwittenborn/CelesteTray";

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
#[macro_export]
macro_rules! get_title {
    ($($arg:tt)*) => {
        format!($($arg)*) + " - Celeste"
    }
}
