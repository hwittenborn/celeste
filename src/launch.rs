use std::convert::identity;

use adw::prelude::*;
use relm4::{
    component::{
        AsyncComponent, AsyncComponentController, AsyncComponentParts, AsyncComponentSender,
        AsyncController, SimpleAsyncComponent,
    },
    prelude::*,
};

use crate::login::{LoginModel, LoginMsg};

pub enum LauchMsg {
    /// The user is trying to add a new remote.
    NewLogin,
    /// The application window is trying to be opened (i.e. from the tray).
    OpenRequest,
    /// The application is trying to be closed (i.e. from the tray).
    CloseRequest,
    #[doc(hidden)]
    /// A new server has been added (from a succesful login).
    AddServer(String),
}

pub struct LaunchModel {
    hide_on_close: bool,
    visible: bool,
    login: AsyncController<LoginModel>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for LaunchModel {
    type Input = ();
    type Output = ();
    type Init = ();

    view! {
        adw::ApplicationWindow {
            #[watch]
            set_hide_on_close: model.hide_on_close,
            #[watch]
            set_visible: model.visible,

            adw::HeaderBar {}
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let login = LoginModel::builder()
            .transient_for(root.clone())
            .launch(())
            .forward(sender.input_sender(), |resp| todo!());
        let model = Self {
            hide_on_close: false,
            visible: false,
            login,
        };
        let widgets = view_output!();

        model.login.emit(LoginMsg::Open);
        AsyncComponentParts { model, widgets }
    }
}
