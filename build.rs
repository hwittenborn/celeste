use nipper::Document;
use std::{env, fs, path::Path};

static METAINFO: &str = include_str!("assets/com.hunterwittenborn.Celeste.metainfo.xml");

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    // Compile the CSS file.
    let css_path = Path::new(&out_dir).join("style.css");
    let scss = grass::from_path("src/style.scss", &grass::Options::default()).unwrap();
    fs::write(css_path, scss).unwrap();

    // Get the release notes for the current version.
    let release_path = Path::new(&out_dir).join("release.xml");
    let metadata = Document::from(METAINFO);
    let release_notes: String = metadata
        .select("release")
        .iter()
        .find(|node| node.attr("version").unwrap() == env!("CARGO_PKG_VERSION").into())
        .unwrap()
        .children()
        .iter()
        .map(|node| node.html().to_string())
        .collect();
    fs::write(release_path, release_notes).unwrap();
}
