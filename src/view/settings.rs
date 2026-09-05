// SPDX-License-Identifier: GPL-3.0-only

//! Settings page: choose the global key combination.

use cosmic::Element;
use cosmic::iced::widget::{Column, Row};
use cosmic::iced::{Alignment, Length};
use cosmic::theme;
use cosmic::widget::{button, container, settings, text, text_input, toggler, warning};
use cosmic_settings_config::shortcuts::Binding;

use crate::config::CheatsheetConfig;
use crate::fl;
use crate::shortcuts::keys;

/// Messages emitted by the settings page.
#[derive(Debug, Clone)]
pub enum Message {
    StartRecording,
    CancelRecording,
    TextChanged(String),
    ApplyText,
    ReplaceConfirm,
    ReplaceCancel,
    Remove,
    ToggleResident(bool),
}

/// Transient UI state of the settings page.
#[derive(Debug, Default)]
pub struct State {
    pub recording: bool,
    /// Modifiers currently held while recording (for live feedback).
    pub pending: Option<Binding>,
    pub text: String,
    pub status: Option<Status>,
    /// A binding that conflicts with an existing shortcut, waiting for the
    /// user to confirm replacement. Holds (binding, label of the old action).
    pub replace: Option<(Binding, String)>,
}

#[derive(Debug, Clone)]
pub enum Status {
    Ok(String),
    Error(String),
}

pub fn page<'a>(
    state: &'a State,
    config: &'a CheatsheetConfig,
    registered: bool,
) -> Element<'a, Message> {
    let spacing = theme::spacing();

    let current = if registered {
        text::body(keys::display(&config.binding))
    } else {
        text::body(fl!("settings-not-registered"))
    };

    let record_button = if state.recording {
        button::standard(fl!("replace-cancel")).on_press(Message::CancelRecording)
    } else {
        button::suggested(fl!("settings-record")).on_press(Message::StartRecording)
    };

    let mut shortcut_section = settings::section()
        .title(fl!("settings-shortcut-section"))
        .add(settings::item(fl!("settings-current-binding"), current))
        .add(settings::item(
            fl!("settings-type-binding"),
            Row::new()
                .spacing(spacing.space_xs)
                .align_y(Alignment::Center)
                .push(
                    text_input("Super+Shift+slash", &state.text)
                        .on_input(Message::TextChanged)
                        .on_submit(|_| Message::ApplyText)
                        .width(Length::Fixed(220.0)),
                )
                .push(button::standard(fl!("settings-apply")).on_press(Message::ApplyText)),
        ));

    let recording_hint: Option<Element<'a, Message>> = if state.recording {
        let pending = state
            .pending
            .as_ref()
            .map(keys::display)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "…".to_owned());
        Some(
            Row::new()
                .spacing(spacing.space_s)
                .align_y(Alignment::Center)
                .push(text::body(fl!("settings-recording")).width(Length::Fill))
                .push(text::heading(pending))
                .into(),
        )
    } else {
        None
    };

    shortcut_section = shortcut_section.add(settings::item(
        fl!("settings-record"),
        Row::new()
            .spacing(spacing.space_xs)
            .align_y(Alignment::Center)
            .push(record_button)
            .push(
                button::destructive(fl!("settings-remove"))
                    .on_press_maybe(registered.then_some(Message::Remove)),
            ),
    ));
    if let Some(hint) = recording_hint {
        shortcut_section = shortcut_section.add(hint);
    }

    let behaviour_section = settings::section()
        .title(fl!("settings-behaviour-section"))
        .add(
            settings::item::builder(fl!("settings-resident"))
                .description(fl!("settings-resident-desc"))
                .control(toggler(config.resident).on_toggle(Message::ToggleResident)),
        );

    let mut page = Column::new()
        .spacing(spacing.space_l)
        .push(shortcut_section);

    if let Some((binding, action)) = &state.replace {
        let body = fl!(
            "replace-body",
            binding = keys::display(binding),
            action = action.clone()
        );
        page = page.push(
            container(
                Column::new()
                    .spacing(spacing.space_s)
                    .push(text::heading(fl!("replace-title")))
                    .push(text::body(body))
                    .push(
                        Row::new()
                            .spacing(spacing.space_xs)
                            .push(
                                button::suggested(fl!("replace-confirm"))
                                    .on_press(Message::ReplaceConfirm),
                            )
                            .push(
                                button::standard(fl!("replace-cancel"))
                                    .on_press(Message::ReplaceCancel),
                            ),
                    ),
            )
            .padding(spacing.space_m)
            .class(theme::Container::Card),
        );
    }

    if let Some(status) = &state.status {
        page = match status {
            Status::Ok(msg) => page.push(text::body(msg)),
            Status::Error(msg) => page.push(warning(msg.as_str())),
        };
    }

    page = page.push(behaviour_section);

    cosmic::widget::scrollable(container(page).padding(spacing.space_m).width(Length::Fill)).into()
}
