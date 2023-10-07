use adw::{gtk::License, prelude::*, AboutWindow, Application};

static METADATA: &str = include_str!(concat!(env!("OUT_DIR"), "/release.xml"));

pub fn about_window(app: &Application) {
    let window = AboutWindow::builder()
        .application(app)
        .application_icon("com.hunterwittenborn.Celeste")
        .application_name("Celeste")
        .copyright("© 2022-2023 Hunter Wittenborn")
        .developer_name("Hunter Wittenborn")
        .developers(vec![
            "Hunter Wittenborn https://hunterwittenborn.com".to_string()
        ])
        .issue_url("https://github.com/hwittenborn/celeste")
        .license_type(License::Gpl30)
        .release_notes(METADATA)
        .support_url("https://github.com/hwittenborn/celeste/issues")
        .build();
    window.add_credit_section(Some(&tr::tr!("App icons by")), &["Adrien Facélina"]);

    window.show();
}
