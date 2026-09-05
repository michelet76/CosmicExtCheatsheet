// SPDX-License-Identifier: GPL-3.0-only

//! The overlay + settings application.

use std::str::FromStr;

use cosmic::app::{ApplicationExt, Core, Task};
use cosmic::cosmic_config::Config;
use cosmic::dbus_activation::Details;
use cosmic::iced::event::{listen_raw, listen_with};
use cosmic::iced::keyboard::key::Named;
use cosmic::iced::keyboard::{self, Key, Location, Modifiers};
use cosmic::iced::platform_specific::runtime::wayland::layer_surface::{
    IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
};
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    Anchor, KeyboardInteractivity, Layer, destroy_layer_surface, get_layer_surface,
};
use cosmic::iced::platform_specific::shell::wayland::commands::keyboard_shortcuts_inhibit;
use cosmic::iced::runtime::core::event::wayland::LayerEvent;
use cosmic::iced::runtime::core::event::{PlatformSpecific, wayland};
use cosmic::iced::runtime::core::layout::Limits;
use cosmic::iced::{self, Color, Length, Subscription, window};
use cosmic::widget::{container, header_bar};
use cosmic::{Element, theme};
use cosmic_ext_cheatsheet::config::CheatsheetConfig;
use cosmic_ext_cheatsheet::ids::APP_ID;
use cosmic_ext_cheatsheet::shortcuts::{CheatsheetModel, keys};
use cosmic_ext_cheatsheet::view::cheatsheet;
use cosmic_ext_cheatsheet::view::settings::{self, Status};
use cosmic_ext_cheatsheet::{fl, keycapture, registration};
use cosmic_settings_config::shortcuts::{self, Binding, Shortcuts};

use crate::cli::{Args, Cmd};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// No main window; the overlay is a layer surface, settings open in an extra window.
    Overlay,
    /// Started with `settings`: the main window is the settings page.
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    Show,
    Hide,
    Toggle,
    CloseWindow(window::Id),
    WindowClosed(window::Id),
    Layer(LayerEvent, window::Id),
    ConfigUpdated(CheatsheetConfig),
    /// The compositor shortcuts config changed on disk.
    ShortcutsUpdated,
    Settings(settings::Message),
    Cheatsheet(cheatsheet::Message),
    /// Escape pressed while the overlay has focus (routed through the raw
    /// event listener, because a focused text input captures the key).
    OverlayEscape(window::Id),
    KeyPressed(window::Id, Key, Location, Modifiers),
    ModifiersChanged(Modifiers),
    Exit,
    Noop,
}

pub struct App {
    core: Core,
    mode: Mode,
    config: CheatsheetConfig,
    config_ctx: Option<Config>,
    shortcuts_ctx: Option<Config>,
    merged: Shortcuts,
    model: CheatsheetModel,
    query: String,
    overlay_id: window::Id,
    overlay_visible: bool,
    settings_window: Option<window::Id>,
    settings: settings::State,
}

/// Exit shortly after the current message is processed, so pending work such
/// as the D-Bus activation reply and the layer-surface destroy reach the
/// compositor before the connection is torn down.
fn delayed_exit() -> Task<Message> {
    cosmic::task::future(async {
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
        Message::Exit
    })
}

impl App {
    /// Rebuild the cheatsheet from the current merged bindings.
    fn rebuild_model(&mut self) {
        let system_actions = self
            .shortcuts_ctx
            .as_ref()
            .map(shortcuts::system_actions)
            .unwrap_or_default();
        self.model = CheatsheetModel::from_shortcuts(&self.merged, &system_actions);
    }

    fn reload_shortcuts(&mut self) {
        if let Some(ctx) = &self.shortcuts_ctx {
            self.merged = shortcuts::shortcuts(ctx);
            self.rebuild_model();
        }
    }

    fn clear_search(&mut self) -> Task<Message> {
        self.query.clear();
        cosmic::widget::text_input::focus(cheatsheet::search_id())
    }

    /// Run a shell command the way the compositor does for `Spawn` bindings.
    fn run_command(command: &str) {
        tracing::info!("running: {command}");
        // The activation token in our environment was issued for this process;
        // a stale one is worse than none for the child.
        let spawned = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg(command)
            .env_remove("XDG_ACTIVATION_TOKEN")
            .env_remove("DESKTOP_STARTUP_ID")
            .spawn();
        match spawned {
            Ok(mut child) => {
                let command = command.to_owned();
                std::thread::spawn(move || match child.wait() {
                    Ok(status) if status.success() => {}
                    Ok(status) => tracing::warn!("{command:?} exited with {status}"),
                    Err(why) => tracing::warn!("could not wait for {command:?}: {why}"),
                });
            }
            Err(why) => tracing::error!("could not run {command:?}: {why}"),
        }
    }

    fn is_registered(&self) -> bool {
        self.config.registered.is_some()
            && self.shortcuts_ctx.as_ref().is_some_and(|ctx| {
                registration::is_registered(ctx, &self.config.binding).unwrap_or(false)
            })
    }

    /// Persist the app config, reporting failure in the settings page.
    fn save_config(&mut self) {
        let Some(app_ctx) = &self.config_ctx else {
            self.settings.status = Some(Status::Error(fl!(
                "settings-write-error",
                error = "app config directory unavailable".to_owned()
            )));
            return;
        };
        if let Err(why) = self.config.save(app_ctx) {
            self.settings.status = Some(Status::Error(fl!(
                "settings-write-error",
                error = why.to_string()
            )));
        }
    }

    fn show(&mut self) -> Task<Message> {
        if self.overlay_visible {
            return Task::none();
        }
        self.overlay_visible = true;
        // Every open starts from the full list, whichever way it was closed.
        self.query.clear();
        get_layer_surface(SctkLayerSurfaceSettings {
            id: self.overlay_id,
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            input_zone: None,
            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
            output: IcedOutput::Active,
            namespace: "cosmic-ext-cheatsheet".into(),
            margin: IcedMargin::default(),
            size: Some((None, None)),
            exclusive_zone: -1,
            size_limits: Limits::NONE,
        })
    }

    fn hide(&mut self) -> Task<Message> {
        if !self.overlay_visible {
            return Task::none();
        }
        self.overlay_visible = false;
        let destroy = destroy_layer_surface(self.overlay_id);
        if self.should_exit() {
            Task::batch([destroy, delayed_exit()])
        } else {
            destroy
        }
    }

    /// Exit once nothing is shown, unless configured to stay resident.
    fn should_exit(&self) -> bool {
        self.mode == Mode::Overlay
            && !self.config.resident
            && !self.overlay_visible
            && self.settings_window.is_none()
    }

    fn open_settings(&mut self) -> Task<Message> {
        match self.mode {
            Mode::Settings => Task::none(),
            Mode::Overlay => {
                if let Some(id) = self.settings_window {
                    return iced::window::gain_focus(id);
                }
                let (id, task) = iced::window::open(iced::window::Settings {
                    size: iced::Size::new(640.0, 720.0),
                    decorations: false,
                    transparent: true,
                    ..Default::default()
                });
                self.settings_window = Some(id);
                task.map(|_| cosmic::Action::App(Message::Noop))
            }
        }
    }

    fn start_recording(&mut self) -> Task<Message> {
        self.settings.recording = true;
        self.settings.pending = None;
        self.settings.status = None;
        self.settings.replace = None;
        keyboard_shortcuts_inhibit::inhibit_shortcuts(true).discard()
    }

    fn stop_recording(&mut self) -> Task<Message> {
        self.settings.recording = false;
        self.settings.pending = None;
        keyboard_shortcuts_inhibit::inhibit_shortcuts(false).discard()
    }

    /// Validate, check for conflicts (unless `replace`), then register.
    fn apply_binding(&mut self, binding: Binding, replace: bool) {
        if let Err(why) = keycapture::validate(&binding) {
            self.settings.status =
                Some(Status::Error(fl!("settings-invalid-binding", error = why)));
            return;
        }
        if !replace && let Some(action) = registration::conflict(&self.merged, &binding) {
            let label = cosmic_ext_cheatsheet::shortcuts::localize::action_label(
                &action,
                self.merged.0.get_key_value(&binding).map(|(b, _)| b),
            );
            self.settings.replace = Some((binding, label));
            return;
        }
        self.settings.replace = None;
        let Some(ctx) = &self.shortcuts_ctx else {
            self.settings.status = Some(Status::Error(fl!(
                "settings-write-error",
                error = "shortcuts config unavailable".to_owned()
            )));
            return;
        };
        match registration::register(ctx, self.config.registered.as_ref(), &binding) {
            Ok(()) => {
                self.config.binding = binding.clone();
                self.config.registered = Some(binding.clone());
                self.config.auto_register = true;
                self.settings.text.clear();
                self.settings.status = Some(Status::Ok(fl!(
                    "settings-registered-ok",
                    binding = keys::display(&binding)
                )));
                self.save_config();
                self.reload_shortcuts();
            }
            Err(why) => {
                self.settings.status = Some(Status::Error(fl!(
                    "settings-write-error",
                    error = why.to_string()
                )));
            }
        }
    }

    fn remove_binding(&mut self) {
        let Some(ctx) = &self.shortcuts_ctx else {
            return;
        };
        match registration::unregister(ctx, self.config.registered.as_ref()) {
            Ok(()) => {
                self.config.registered = None;
                // An explicit removal must not be undone by the next launch.
                self.config.auto_register = false;
                self.settings.status = None;
                self.save_config();
                self.reload_shortcuts();
            }
            Err(why) => {
                self.settings.status = Some(Status::Error(fl!(
                    "settings-write-error",
                    error = why.to_string()
                )));
            }
        }
    }

    fn update_settings(&mut self, message: settings::Message) -> Task<Message> {
        use settings::Message as M;
        match message {
            M::StartRecording => return self.start_recording(),
            M::CancelRecording => return self.stop_recording(),
            M::TextChanged(text) => self.settings.text = text,
            M::ApplyText => {
                let text = self.settings.text.trim().to_owned();
                if text.is_empty() {
                    return Task::none();
                }
                match Binding::from_str(&text) {
                    Ok(binding) => self.apply_binding(binding, false),
                    Err(why) => {
                        self.settings.status =
                            Some(Status::Error(fl!("settings-invalid-binding", error = why)));
                    }
                }
            }
            M::ReplaceConfirm => {
                if let Some((binding, _)) = self.settings.replace.take() {
                    self.apply_binding(binding, true);
                }
            }
            M::ReplaceCancel => self.settings.replace = None,
            M::Remove => self.remove_binding(),
            M::ToggleResident(value) => {
                self.config.resident = value;
                self.save_config();
            }
        }
        Task::none()
    }

    fn update_cheatsheet(&mut self, message: cheatsheet::Message) -> Task<Message> {
        use cheatsheet::Message as M;
        match message {
            M::Close => self.hide(),
            M::OpenSettings => {
                let hide = self.hide_keep_alive();
                hide.chain(self.open_settings())
            }
            M::Run(command) => {
                Self::run_command(&command);
                self.hide()
            }
            M::SearchChanged(query) => {
                self.query = query;
                Task::none()
            }
            M::SearchClear => self.clear_search(),
            M::SearchSubmit => {
                let command = self
                    .model
                    .search(&self.query)
                    .enter_target()
                    .and_then(|e| e.command.clone());
                match command {
                    Some(command) => self.update_cheatsheet(M::Run(command)),
                    None => Task::none(),
                }
            }
            M::Noop => Task::none(),
        }
    }

    fn settings_page(&self) -> Element<'_, Message> {
        settings::page(&self.settings, &self.config, self.is_registered()).map(Message::Settings)
    }
}

impl cosmic::Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = Args;
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, flags: Args) -> (Self, Task<Message>) {
        let mode = match flags.cmd {
            Some(Cmd::Settings) => Mode::Settings,
            _ => Mode::Overlay,
        };
        if mode == Mode::Overlay {
            core.set_app_type(cosmic::core::AppType::System);
        }
        core.window.show_headerbar = true;

        let (mut config, config_ctx) = CheatsheetConfig::load();
        let shortcuts_ctx = match registration::context() {
            Ok(ctx) => Some(ctx),
            Err(why) => {
                tracing::error!("could not open shortcuts config: {why}");
                None
            }
        };

        if let Some(ctx) = &shortcuts_ctx {
            let mut changed = false;
            if config.auto_register && config.registered.is_none() {
                // First run: make the default shortcut work without visiting settings.
                let binding = config.binding.clone();
                match registration::register_checked(ctx, None, &binding, false) {
                    Ok(()) => {
                        config.registered = Some(binding);
                        changed = true;
                        tracing::info!(
                            "registered default shortcut {}",
                            keys::display(&config.binding)
                        );
                    }
                    Err(registration::Error::Conflict(_)) => {
                        tracing::warn!(
                            "default shortcut already in use; open settings to pick one"
                        );
                    }
                    Err(why) => tracing::error!("could not register default shortcut: {why}"),
                }
            } else if config.registered.is_some() {
                // Keep the spawn command pointing at the best-installed binary,
                // and notice when the user removed or reassigned the combination.
                match registration::sync_command(ctx, &config.binding) {
                    Ok(registration::Sync::Updated) => {
                        tracing::info!(
                            "updated shortcut command to {}",
                            registration::spawn_command()
                        );
                    }
                    Ok(registration::Sync::Unchanged) => {}
                    Ok(outcome @ (registration::Sync::Missing | registration::Sync::Foreign)) => {
                        tracing::warn!(
                            "shortcut {} is {outcome:?}; treating as unregistered",
                            keys::display(&config.binding)
                        );
                        config.registered = None;
                        config.auto_register = false;
                        changed = true;
                    }
                    Err(why) => tracing::error!("could not check shortcut registration: {why}"),
                }
            }
            if changed && let Some(app_ctx) = &config_ctx {
                let _ = config.save(app_ctx);
            }
        }

        let mut app = App {
            core,
            mode,
            config,
            config_ctx,
            shortcuts_ctx,
            merged: Shortcuts::default(),
            model: CheatsheetModel::default(),
            query: String::new(),
            overlay_id: window::Id::unique(),
            overlay_visible: false,
            settings_window: None,
            settings: settings::State::default(),
        };
        app.reload_shortcuts();

        let task = match (mode, flags.cmd) {
            (Mode::Settings, _) => {
                let title = fl!("settings-title");
                app.core.set_header_title(title.clone());
                if let Some(id) = app.core.main_window_id() {
                    app.set_window_title(title, id)
                } else {
                    Task::none()
                }
            }
            (Mode::Overlay, Some(Cmd::Hide)) => delayed_exit(),
            (Mode::Overlay, _) => app.show(),
        };
        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Show => self.show(),
            Message::Hide => self.hide(),
            Message::Toggle => {
                if self.overlay_visible {
                    self.hide()
                } else {
                    self.show()
                }
            }
            Message::CloseWindow(id) => iced::window::close(id),
            Message::WindowClosed(id) => {
                if self.settings_window == Some(id) {
                    self.settings_window = None;
                    let stop = if self.settings.recording {
                        self.stop_recording()
                    } else {
                        Task::none()
                    };
                    if self.should_exit() {
                        return Task::batch([stop, delayed_exit()]);
                    }
                    return stop;
                }
                if id == self.overlay_id && self.overlay_visible {
                    // Closed by the compositor (e.g. output gone).
                    self.overlay_visible = false;
                    if self.should_exit() {
                        return delayed_exit();
                    }
                }
                Task::none()
            }
            Message::Layer(event, id) => {
                if id != self.overlay_id || !self.overlay_visible {
                    return Task::none();
                }
                match event {
                    LayerEvent::Unfocused => self.hide(),
                    LayerEvent::Focused => {
                        cosmic::widget::text_input::focus(cheatsheet::search_id())
                    }
                    _ => Task::none(),
                }
            }
            Message::ConfigUpdated(config) => {
                self.config = config;
                Task::none()
            }
            Message::ShortcutsUpdated => {
                // Re-read through the same merge as at startup.
                self.reload_shortcuts();
                Task::none()
            }
            Message::Cheatsheet(message) => self.update_cheatsheet(message),
            Message::OverlayEscape(id) => {
                if id != self.overlay_id || !self.overlay_visible {
                    return Task::none();
                }
                if self.query.is_empty() {
                    self.hide()
                } else {
                    self.clear_search()
                }
            }
            Message::Settings(message) => self.update_settings(message),
            Message::ModifiersChanged(modifiers) => {
                if self.settings.recording {
                    let mut pending = Binding::new(keycapture::modifiers(modifiers), None);
                    pending.keycode = None;
                    self.settings.pending = Some(pending);
                }
                Task::none()
            }
            Message::KeyPressed(id, key, location, modifiers) => {
                // Only keys pressed in the window that hosts the settings page.
                let settings_id = self.settings_window.or_else(|| self.core.main_window_id());
                if !self.settings.recording || self.overlay_visible || Some(id) != settings_id {
                    return Task::none();
                }
                if key == Key::Named(Named::Escape) && modifiers.is_empty() {
                    return self.stop_recording();
                }
                if keycapture::is_modifier_key(&key) {
                    let mut pending = Binding::new(keycapture::modifiers(modifiers), None);
                    pending.keycode = None;
                    self.settings.pending = Some(pending);
                    return Task::none();
                }
                match keycapture::binding_from_key(&key, location, modifiers) {
                    Some(binding) => {
                        let stop = self.stop_recording();
                        self.apply_binding(binding, false);
                        stop
                    }
                    None => Task::none(),
                }
            }
            Message::Exit => {
                // A show() or settings window may have arrived during the delay.
                if self.should_exit() {
                    iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::Noop => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Only used in Settings mode (there is no main window otherwise).
        self.settings_page()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        if id == self.overlay_id {
            return cheatsheet::overlay(&self.model, &self.query).map(Message::Cheatsheet);
        }
        if Some(id) == self.settings_window {
            let focused = self.core.focused_window().is_some_and(|f| f == id);
            return container(
                cosmic::iced::widget::Column::new()
                    .push(
                        header_bar()
                            .title(fl!("settings-title"))
                            .focused(focused)
                            .on_close(Message::CloseWindow(id)),
                    )
                    .push(self.settings_page()),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .class(theme::Container::WindowBackground)
            .into();
        }
        cosmic::widget::text("").into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            self.core
                .watch_config::<CheatsheetConfig>(APP_ID)
                .map(|update| Message::ConfigUpdated(update.config)),
            self.core
                .watch_config::<shortcuts::Config>(shortcuts::ID)
                .map(|_| Message::ShortcutsUpdated),
            listen_raw(|event, _status, id| match event {
                iced::Event::PlatformSpecific(PlatformSpecific::Wayland(
                    wayland::Event::Layer(event, _, layer_id),
                )) => Some(Message::Layer(event, layer_id)),
                iced::Event::Window(window::Event::Closed) => Some(Message::WindowClosed(id)),
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: Key::Named(Named::Escape),
                    modifiers,
                    ..
                }) if modifiers.is_empty() => Some(Message::OverlayEscape(id)),
                _ => None,
            }),
        ];
        if self.settings.recording {
            subs.push(listen_with(|event, _status, id| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key,
                    location,
                    modifiers,
                    ..
                }) => Some(Message::KeyPressed(id, key, location, modifiers)),
                iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                    Some(Message::ModifiersChanged(modifiers))
                }
                _ => None,
            }));
        }
        Subscription::batch(subs)
    }

    fn on_escape(&mut self) -> Task<Message> {
        // The overlay handles Escape itself (see the raw listener), because a
        // focused text input captures the key before keyboard navigation sees it.
        if self.settings.recording {
            return self.stop_recording();
        }
        Task::none()
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::WindowClosed(id))
    }

    fn style(&self) -> Option<iced::theme::Style> {
        match self.mode {
            Mode::Overlay => {
                let theme = self.core.system_theme().cosmic();
                Some(iced::theme::Style {
                    background_color: Color::TRANSPARENT,
                    text_color: Color::from(theme.on_bg_color()),
                    icon_color: Color::from(theme.on_bg_color()),
                })
            }
            Mode::Settings => None,
        }
    }

    fn dbus_activation(&mut self, msg: cosmic::dbus_activation::Message) -> Task<Message> {
        let cmd = match msg.msg {
            Details::Activate => Cmd::Toggle,
            Details::ActivateAction { action, .. } => match Cmd::from_str(&action) {
                Ok(cmd) => cmd,
                Err(()) => {
                    tracing::warn!("unknown activation action {action:?}");
                    return Task::none();
                }
            },
            Details::Open { .. } => return Task::none(),
        };
        tracing::debug!("activation: {cmd}");
        match cmd {
            Cmd::Toggle => self.update(Message::Toggle),
            Cmd::Show => self.update(Message::Show),
            Cmd::Hide => self.update(Message::Hide),
            Cmd::Settings => self.update_cheatsheet(cheatsheet::Message::OpenSettings),
            Cmd::Register { .. } | Cmd::Unregister => Task::none(),
        }
    }
}

impl App {
    /// Hide the overlay without letting the process exit (a settings window
    /// is about to open).
    fn hide_keep_alive(&mut self) -> Task<Message> {
        if !self.overlay_visible {
            return Task::none();
        }
        self.overlay_visible = false;
        destroy_layer_surface(self.overlay_id)
    }
}
