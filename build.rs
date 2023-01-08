use std::{fs, path::Path};
static OUTPUT_FILE_NAME: &str = "src/style.css";

fn main() {
    let scss = grass::from_path("src/style.scss", &grass::Options::default()).unwrap();

    if Path::new(OUTPUT_FILE_NAME).exists() {
        fs::remove_file(OUTPUT_FILE_NAME).unwrap();
    }
    fs::write(OUTPUT_FILE_NAME, scss).unwrap();
}
