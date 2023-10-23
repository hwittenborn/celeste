use crate::{launch, util};
use ksni::{menu::StandardItem, MenuItem, Tray as KsniTray};

pub struct Tray {
    status: String,
    pub icon: String,
}

impl Tray {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            status: tr::tr!("Awaiting sync checks..."),
            icon: "com.hunterwittenborn.Celeste.CelesteTrayLoading-symbolic".to_owned(),
        }
    }

    pub fn set_msg<T: ToString>(&mut self, msg: T) {
        self.status = msg.to_string();
    }

    pub fn set_syncing(&mut self) {
        self.icon = "com.hunterwittenborn.Celeste.CelesteTraySyncing-symbolic".to_owned();
    }

    pub fn set_warning(&mut self) {
        self.icon = "com.hunterwittenborn.Celeste.CelesteTrayWarning-symbolic".to_owned();
    }

    pub fn set_done(&mut self) {
        self.icon = "com.hunterwittenborn.Celeste.CelesteTrayDone-symbolic".to_owned();
    }

    pub fn set_disconnected(&mut self) {
        self.icon = "com.hunterwittenborn.Celeste.CelesteTrayDisconnected-symbolic".to_owned();
    }
}

impl KsniTray for Tray {
    fn icon_name(&self) -> String {
        self.icon.clone()
    }

    fn title(&self) -> String {
        "Celeste".to_owned()
    }

    fn id(&self) -> String {
        util::APP_ID.to_owned()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            MenuItem::Standard(StandardItem {
                label: self.status.clone(),
                enabled: false,
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: tr::tr!("Open"),
                activate: Box::new(|_| {
                    *(*launch::OPEN_REQUEST).lock().unwrap() = true;
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: tr::tr!("Close"),
                activate: Box::new(|_| {
                    *(*launch::CLOSE_REQUEST).lock().unwrap() = true;
                }),
                ..Default::default()
            }),
        ]
    }
}
