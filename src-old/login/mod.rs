//! Functionality for logging into a server.
use crate::{entities::RemotesModel, mpsc, util};
use adw::{prelude::*, Application, ApplicationWindow};
use relm4::{
    component::{AsyncComponentParts, AsyncComponentSender, SimpleAsyncComponent},
    prelude::*,
};
use sea_orm::DatabaseConnection;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr, EnumString};
use std::str::FromStr;

#[derive(Debug, Default, EnumIter, EnumString, IntoStaticStr)]
enum Provider {
    #[default]
    Dropbox,
    #[strum(serialize = "Google Drive")]
    GoogleDrive,
    Nextcloud,
    Owncloud,
    #[strum(serialize = "pCloud")]
    PCloud,
    #[strum(serialize = "Proton Drive")]
    ProtonDrive,
    WebDav,
}

#[derive(Debug)]
pub enum LoginMsg {
    Open,
    #[doc(hidden)]
    SetProvider(Provider),
}

#[derive(Default)]
pub struct LoginModel {
    visible: bool,
    provider: Provider,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for LoginModel {
    type Input = LoginMsg;
    type Output = ();
    type Init = ();

    view! {
        #[name(window)]
        ApplicationWindow {
            set_title: Some(&util::get_title!("Log in")),
            set_default_width: 400,
            add_css_class: "celeste-global-padding",
            #[watch]
            set_visible: model.visible,
            // When hiding/showing different entry widgets, we may end up with
            // extra padding on the bottom of the window. This resets the
            // window height to our widget size on each render.
            #[watch]
            set_default_size: (window.width(), -1),

             gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar,

                gtk::ListBox {
                    set_selection_mode: gtk::SelectionMode::None,
                    add_css_class: "boxed-list",

                    adw::ComboRow {
                        set_title: &tr::tr!("Server Type"),

                        #[wrap(Some)]
                        set_model = &gtk::StringList {
                            #[iterate]
                            append: Provider::iter().map(|provider| provider.into())
                        },

                        connect_selected_item_notify[sender] => move |row| {
                            let string_list: gtk::StringList = row.model().unwrap().downcast().unwrap();
                            let selected = string_list.string(row.selected()).unwrap().to_string();
                            let provider = Provider::from_str(&selected).unwrap();
                            sender.input(LoginMsg::SetProvider(provider));
                        }
                    },

                    adw::EntryRow {
                        set_title: &tr::tr!("Name")
                    },

                    adw::EntryRow {
                        set_title: &tr::tr!("Server URL"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::WebDav)
                    },

                    adw::EntryRow {
                        set_title: &tr::tr!("Username"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::ProtonDrive | Provider::WebDav)
                    },

                    adw::PasswordEntryRow {
                        set_title: &tr::tr!("Password"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::ProtonDrive | Provider::WebDav)
                    },

                    #[name(totp_entry)]
                    adw::EntryRow {
                        set_title: &tr::tr!("2FA Code"),
                        set_editable: false,
                        #[watch]
                        set_visible: matches!(model.provider, Provider::ProtonDrive),

                        add_prefix = &gtk::CheckButton {
                            connect_toggled[totp_entry] => move |check| {
                                let active = check.is_active();
                                totp_entry.set_editable(active);

                                if !active {
                                    totp_entry.set_text("");
                                }
                            }
                        }
                    }
                }
             }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self::default();
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        match message {
            LoginMsg::Open => self.visible = true,
            LoginMsg::SetProvider(provider) => self.provider = provider,
        }
    }
}
