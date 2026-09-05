// SPDX-License-Identifier: GPL-3.0-only

//! Panel applet: a button that toggles the cheatsheet overlay.

use std::process::Command;

use cosmic::app::{Core, Task};
use cosmic::iced::window;
use cosmic::Element;
use cosmic_ext_cheatsheet::ids::{APPLET_ID, BIN_NAME, ICON_SYMBOLIC};
use cosmic_ext_cheatsheet::{fl, i18n};

fn main() -> cosmic::iced::Result {
    i18n::init_from_desktop();
    cosmic::applet::run::<Applet>(())
}

struct Applet {
    core: Core,
}

#[derive(Debug, Clone)]
enum Message {
    Launch,
    Surface(cosmic::surface::Action),
}

impl cosmic::Application for Applet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APPLET_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        (Self { core }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Launch => {
                spawn_overlay();
                Task::none()
            }
            Message::Surface(action) => {
                cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(action)))
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let button = self.core.applet.icon_button(ICON_SYMBOLIC).on_press(Message::Launch);
        Element::from(
            self.core
                .applet
                .applet_tooltip::<Message>(button, fl!("applet-tooltip"), false, Message::Surface, None),
        )
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        cosmic::widget::text("").into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

/// Launch (or toggle, via single-instance activation) the overlay binary.
///
/// cosmic-panel gives applets a private `WAYLAND_SOCKET`; it must not leak to
/// the child, which has to connect to the real display. The applet desktop
/// entry sets `X-HostWaylandDisplay=true` so `WAYLAND_DISPLAY` is available.
fn spawn_overlay() {
    let program = std::env::current_exe()
        .ok()
        .map(|exe| exe.with_file_name(BIN_NAME))
        .filter(|sibling| sibling.exists())
        .map_or_else(|| BIN_NAME.into(), |p| p.into_os_string());

    let mut command = Command::new(program);
    command
        .arg("toggle")
        .env_remove("WAYLAND_SOCKET")
        .env_remove("X_PRIVILEGED_WAYLAND_SOCKET")
        .env_remove("X_MINIMIZE_APPLET");

    match command.spawn() {
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(why) => eprintln!("cosmic-ext-applet-cheatsheet: could not launch {BIN_NAME}: {why}"),
    }
}
