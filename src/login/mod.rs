//! Functionality for logging into a server.
use crate::{entities::RemotesModel, mpsc, util};
use adw::{prelude::*, Application, ApplicationWindow};
use relm4::{component::{SimpleAsyncComponent, AsyncComponentSender, AsyncComponentParts}, prelude::*};
use sea_orm::DatabaseConnection;

#[derive(Debug)]
pub enum LoginMsg {
    Open,
}

pub struct LoginModel {
    visible: bool,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for LoginModel {
    type Input = LoginMsg;
    type Output = ();
    type Init = ();

    view! {
        ApplicationWindow {
            set_title: Some(&util::get_title!("Log in")),
            set_default_width: 400,
            add_css_class: "celeste-global-padding",
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
        let model = Self { visible: false };
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>) {
        match message {
            LoginMsg::Open => self.visible = true
        }
    }
}
