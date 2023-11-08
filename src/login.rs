//! Functionality for logging into a server.
use crate::{rclone, util};
use adw::{prelude::*, Application, ApplicationWindow};
use regex::Regex;
use relm4::{
    component::{AsyncComponentParts, AsyncComponentSender, SimpleAsyncComponent},
    prelude::*,
};
use relm4_components::alert::{Alert, AlertSettings, AlertMsg};
use sea_orm::DatabaseConnection;
use std::{
    cell::{LazyCell, RefCell},
    collections::HashMap,
    rc::Rc,
    str::FromStr,
    sync::LazyLock,
};
use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};
use url::Url;

fn show_error(model: &LoginModel, field: &LoginField) {
    let mut borrow = model.errors.borrow_mut();
    let items = borrow.get_mut(field).unwrap();

    // TODO: We should use `Alert` from `relm4_components` for this.
    adw::MessageDialog::builder()
        .heading(&items.0)
        .body(&items.1)
        .build()
        .show();
}

#[relm4::widget_template]
impl WidgetTemplate for WarningButton {
    view! {
        gtk::Button {
            add_css_class: "flat",
            set_icon_name: relm4_icons::icon_name::WARNING,
            set_valign: gtk::Align::Center
        }
    }
}

#[derive(Clone, Debug, Default, EnumIter, EnumString, IntoStaticStr)]
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

#[derive(Clone, Debug)]
pub enum LoginMsg {
    Open,
    #[doc(hidden)]
    SetProvider(Provider),
    #[doc(hidden)]
    CheckInputs,
}

/// The login fields that we need to check. We use this in [`LoginModel`] below.
#[derive(Clone, Debug, EnumIter, Eq, Hash, PartialEq)]
enum LoginField {
    Name,
    Url,
    Username,
    Password,
    Totp,
}

/// The type we use to store errors. The values are a tuple of, (title, subtitle) messages to pass to a message window.
type Errors = HashMap<LoginField, (String, String)>;

#[derive(Clone, Default)]
pub struct LoginModel {
    visible: bool,
    provider: Provider,
    errors: Rc<RefCell<Errors>>,
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

                    #[name(name_input)]
                    adw::EntryRow {
                        set_title: &tr::tr!("Name"),
                        #[template]
                        add_suffix = &WarningButton {
                            #[watch]
                            set_visible: !model.errors.borrow().get(&LoginField::Name).unwrap().0.is_empty(),
                            connect_clicked[model] => move |_| show_error(&model, &LoginField::Name)
                        },

                        connect_changed[model, sender] => move |name_input| {
                            let name = name_input.text().to_string();

                            // Get a list of already existing config names.
                            let existing_remotes: Vec<String> = rclone::get_remotes()
                                .iter()
                                .map(|config| config.remote_name())
                                .collect();

                            // Check that the new specified remote is valid.
                            static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-zA-Z_.][0-9a-zA-Z_. -]*[0-9a-zA-Z_.-]$").unwrap());

                            let mut err_msg = None;
                            if existing_remotes.contains(&name) {
                                err_msg = (tr::tr!("Name already exists"), String::new()).into();
                            } else if !NAME_REGEX.is_match(&name) {
                                err_msg = (
                                    tr::tr!("Invalid server name"),
                                    format!(
                                        "{}\n{}\n{}\n{}",
                                        tr::tr!("Server names must:"),
                                        tr::tr!("Only contain numbers, letters, underscores, hyphens, periods, and spaces"),
                                        tr::tr!("Not start with a hyphen/space"),
                                        tr::tr!("Not end with a space")
                                    )
                                ).into();
                            }

                            if let Some(msg) = err_msg {
                                *model.errors.borrow_mut().get_mut(&LoginField::Name).unwrap() = msg;
                                name_input.add_css_class(util::css::ERROR);
                            } else {
                                let mut borrow = model.errors.borrow_mut();
                                let items = borrow.get_mut(&LoginField::Name).unwrap();
                                items.0.clear();
                                items.1.clear();
                                name_input.remove_css_class(util::css::ERROR);
                            }

                            sender.input(LoginMsg::CheckInputs)
                        },
                    },

                    #[name(url_input)]
                    adw::EntryRow {
                        set_title: &tr::tr!("Server URL"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::WebDav),
                        #[template]
                        add_suffix = &WarningButton {
                            #[watch]
                            set_visible: !model.errors.borrow().get(&LoginField::Url).unwrap().0.is_empty(),
                            connect_clicked[model] => move |_| show_error(&model, &LoginField::Url)
                        },
                        connect_changed[model, sender] => move |url_input| {
                            let mut err_msg = None;
                            let maybe_url = Url::parse(&url_input.text());

                            if let Ok(url) = maybe_url {
                                if matches!(model.provider, Provider::Nextcloud | Provider::Owncloud) && url.path().contains("/remote.php/") {
                                    static REMOTE_PHP_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/remote\.php/.*").unwrap());
                                    let invalid_url_segment = REMOTE_PHP_REGEX.find(url.path())
                                        .unwrap()
                                        .as_str()
                                        .to_string();
                                    err_msg = (
                                        tr::tr!("Invalid server URL"),
                                        tr::tr!("Don't specify '{invalid_url_segment}' as part of the URL."),
                                    ).into();
                                }
                            } else {
                                err_msg = (
                                    tr::tr!("Invalid server URL"),
                                    tr::tr!("Error: {}", maybe_url.unwrap_err())
                                ).into();
                            }

                            if let Some(msg) = err_msg {
                                *model.errors.borrow_mut().get_mut(&LoginField::Url).unwrap() = msg;
                                url_input.add_css_class(util::css::ERROR);
                            } else {
                                let mut borrow = model.errors.borrow_mut();
                                let items = borrow.get_mut(&LoginField::Url).unwrap();
                                items.0.clear();
                                items.1.clear();
                                url_input.remove_css_class(util::css::ERROR);
                            }

                            sender.input(LoginMsg::CheckInputs)
                        }
                    },

                    #[name(username_input)]
                    adw::EntryRow {
                        set_title: &tr::tr!("Username"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::ProtonDrive | Provider::WebDav),
                        connect_changed => LoginMsg::CheckInputs,
                        #[template]
                        add_suffix = &WarningButton {
                            #[watch]
                            set_visible: !model.errors.borrow().get(&LoginField::Username).unwrap().0.is_empty(),
                            connect_clicked[model] => move |_| show_error(&model, &LoginField::Username)
                        },
                    },

                    #[name(password_input)]
                    adw::PasswordEntryRow {
                        set_title: &tr::tr!("Password"),
                        #[watch]
                        set_visible: matches!(model.provider, Provider::Nextcloud | Provider::Owncloud | Provider::ProtonDrive | Provider::WebDav),
                        connect_changed => LoginMsg::CheckInputs,
                        #[template]
                        add_suffix = &WarningButton {
                            #[watch]
                            set_visible: !model.errors.borrow().get(&LoginField::Password).unwrap().0.is_empty(),
                            connect_clicked[model] => move |_| show_error(&model, &LoginField::Password)
                        },
                    },

                    #[name(totp_input)]
                    adw::EntryRow {
                        set_title: &tr::tr!("2FA Code"),
                        set_editable: false,
                        #[watch]
                        set_visible: matches!(model.provider, Provider::ProtonDrive),
                        #[template]
                        add_suffix = &WarningButton {
                            #[watch]
                            set_visible: !model.errors.borrow().get(&LoginField::Totp).unwrap().0.is_empty(),
                            connect_clicked[model] => move |_| show_error(&model, &LoginField::Totp)
                        },
                        connect_changed[sender] => move |totp_input| {
                            let totp = totp_input.text().to_string();
                            let mut err_msg = None;

                            if totp.chars().any(|c| !c.is_numeric()) {
                                err_msg = tr::tr!("The provided 2FA code is invalid (should only contain digits).").into();
                            } else if totp.len() != 6 {
                                err_msg = tr::tr!("The provided 2FA code is invalid (should be 6 digits long.").into();
                            }

                            if let Some(msg) = err_msg {
                                totp_input.add_css_class(util::css::ERROR);
                            } else {
                                totp_input.remove_css_class(util::css::ERROR);
                            }

                            sender.input(LoginMsg::CheckInputs)
                        },

                        #[name(totp_input_checkmark)]
                        add_prefix = &gtk::CheckButton {
                            connect_toggled[sender, totp_input] => move |check| {
                                let active = check.is_active();
                                totp_input.set_editable(active);

                                if !active {
                                    totp_input.set_text("");
                                    totp_input.remove_css_class(util::css::ERROR);
                                }
                            }
                        }
                    }
                },

                #[name(login_button)]
                gtk::Button {
                    set_label: &tr::tr!("Log in"),
                    set_halign: gtk::Align::End,
                    set_margin_top: 10,
                    add_css_class: "login-window-submit-button",
                    set_sensitive: false
                }
             }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self::default();
        for field in LoginField::iter() {
            model.errors.borrow_mut().insert(field, Default::default());
        }

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    fn post_view() {
        // Disable the login button if any current input fields are empty or contain
        // errors.
        let mut sensitive = true;

        let inputs: Vec<adw::EntryRow> = vec![
            widgets.name_input.clone(),
            widgets.url_input.clone(),
            widgets.username_input.clone(),
            widgets.password_input.clone().into(),
        ];

        for input in inputs {
            if input.is_visible() {
                if input.text().is_empty() || input.has_css_class(util::css::ERROR) {
                    sensitive = false;
                }
            }
        }

        // We have to check the TOTP field separately, as it contains a checkmark
        // toggle.
        if widgets.totp_input.is_visible() && widgets.totp_input_checkmark.is_active() {
            if widgets.totp_input.text().is_empty()
                || widgets.totp_input.has_css_class(util::css::ERROR)
            {
                sensitive = false;
            }
        }

        widgets.login_button.set_sensitive(sensitive);
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        match message {
            LoginMsg::Open => self.visible = true,
            LoginMsg::SetProvider(provider) => self.provider = provider,
            // This is handled in `pre_view` above. Preferrably it would be
            // done here, but we can't access the struct's widgets here.
            LoginMsg::CheckInputs => (),
        }
    }
}
