use std::path::PathBuf;

/// CSS classes we use in the app.
pub mod css {
    /// The 'error' css class.
    pub static ERROR: &str = "error";
    /// Scrollable codeblocks.
    pub static SCROLLABLE_CODEBLOCK: &str = "celeste-scrollable-codeblock";
}

/// The ID of the app.
pub static APP_ID: &str = "com.hunterwittenborn.Celeste";

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
        tr::tr!($($arg)*) + " - Celeste"
    }
}

pub use crate::get_title;
