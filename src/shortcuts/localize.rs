// SPDX-License-Identifier: GPL-3.0-only

//! Localised labels for actions and categories.

use cosmic_settings_config::shortcuts::action::{
    Direction, FocusDirection, Orientation, ResizeDirection, System,
};
use cosmic_settings_config::shortcuts::{Action, Binding};

use super::CategoryKind;
use crate::fl;

pub fn category_title(kind: CategoryKind) -> String {
    match kind {
        CategoryKind::Navigation => fl!("cat-navigation"),
        CategoryKind::ManageWindows => fl!("cat-manage-windows"),
        CategoryKind::MoveWindows => fl!("cat-move-windows"),
        CategoryKind::WindowTiling => fl!("cat-window-tiling"),
        CategoryKind::System => fl!("cat-system"),
        CategoryKind::Accessibility => fl!("cat-accessibility"),
        CategoryKind::Custom => fl!("cat-custom"),
        CategoryKind::Other => fl!("cat-other"),
    }
}

fn direction(d: Direction) -> &'static str {
    match d {
        Direction::Left => "left",
        Direction::Right => "right",
        Direction::Up => "up",
        Direction::Down => "down",
    }
}

fn focus_direction(d: FocusDirection) -> &'static str {
    match d {
        FocusDirection::Left => "left",
        FocusDirection::Right => "right",
        FocusDirection::Up => "up",
        FocusDirection::Down => "down",
        FocusDirection::In => "in",
        FocusDirection::Out => "out",
    }
}

/// Label for an action. `binding` is used for custom `Spawn` entries, whose
/// description lives on the binding rather than the action.
#[allow(deprecated)]
pub fn action_label(action: &Action, binding: Option<&Binding>) -> String {
    match action {
        Action::Close => fl!("action-close"),
        Action::Debug => fl!("action-debug"),
        Action::Disable => fl!("action-disable"),
        Action::Focus(d) => fl!("action-focus", direction = focus_direction(*d)),
        Action::LastWorkspace => fl!("action-last-workspace"),
        Action::Maximize => fl!("action-maximize"),
        Action::Fullscreen => fl!("action-fullscreen"),
        Action::Minimize => fl!("action-minimize"),
        Action::MigrateWorkspaceToNextOutput => fl!("action-migrate-workspace-next-output"),
        Action::MigrateWorkspaceToPreviousOutput => fl!("action-migrate-workspace-prev-output"),
        Action::MigrateWorkspaceToOutput(d) => {
            fl!("action-migrate-workspace-output", direction = direction(*d))
        }
        Action::Move(d) => fl!("action-move", direction = direction(*d)),
        Action::MoveToLastWorkspace | Action::SendToLastWorkspace => {
            fl!("action-move-last-workspace")
        }
        Action::MoveToNextWorkspace | Action::SendToNextWorkspace => {
            fl!("action-move-next-workspace")
        }
        Action::MoveToPreviousWorkspace | Action::SendToPreviousWorkspace => {
            fl!("action-move-prev-workspace")
        }
        Action::MoveToNextOutput | Action::SendToNextOutput => fl!("action-move-next-output"),
        Action::MoveToPreviousOutput | Action::SendToPreviousOutput => {
            fl!("action-move-prev-output")
        }
        Action::MoveToOutput(d) | Action::SendToOutput(d) => {
            fl!("action-move-output", direction = direction(*d))
        }
        Action::MoveToWorkspace(n) | Action::SendToWorkspace(n) => {
            fl!("action-move-workspace", num = n.to_string())
        }
        Action::NextOutput => fl!("action-next-output"),
        Action::PreviousOutput => fl!("action-prev-output"),
        Action::NextWorkspace => fl!("action-next-workspace"),
        Action::PreviousWorkspace => fl!("action-prev-workspace"),
        Action::Orientation(Orientation::Horizontal) => fl!("action-orientation-horizontal"),
        Action::Orientation(Orientation::Vertical) => fl!("action-orientation-vertical"),
        Action::Resizing(ResizeDirection::Inwards) => fl!("action-resize-inwards"),
        Action::Resizing(ResizeDirection::Outwards) => fl!("action-resize-outwards"),
        Action::SwapWindow => fl!("action-swap-window"),
        Action::SwitchOutput(d) => fl!("action-switch-output", direction = direction(*d)),
        Action::System(system) => system_label(system),
        Action::Spawn(command) => binding
            .and_then(|b| b.description.clone())
            .filter(|d| !d.trim().is_empty())
            .unwrap_or_else(|| command.clone()),
        Action::Terminate => fl!("action-terminate"),
        Action::ToggleOrientation => fl!("action-toggle-orientation"),
        Action::ToggleStacking => fl!("action-toggle-stacking"),
        Action::ToggleSticky => fl!("action-toggle-sticky"),
        Action::ToggleTiling => fl!("action-toggle-tiling"),
        Action::ToggleWindowFloating => fl!("action-toggle-floating"),
        Action::Workspace(n) => fl!("action-workspace", num = n.to_string()),
        Action::ZoomIn => fl!("action-zoom-in"),
        Action::ZoomOut => fl!("action-zoom-out"),
    }
}

pub fn system_label(system: &System) -> String {
    match system {
        System::AppLibrary => fl!("system-app-library"),
        System::BrightnessDown => fl!("system-brightness-down"),
        System::BrightnessUp => fl!("system-brightness-up"),
        System::DisplayToggle => fl!("system-display-toggle"),
        System::HomeFolder => fl!("system-home-folder"),
        System::InputSourceSwitch => fl!("system-input-source-switch"),
        System::KeyboardBrightnessDown => fl!("system-keyboard-brightness-down"),
        System::KeyboardBrightnessUp => fl!("system-keyboard-brightness-up"),
        System::Launcher => fl!("system-launcher"),
        System::LockScreen => fl!("system-lock-screen"),
        System::LogOut => fl!("system-log-out"),
        System::Mute => fl!("system-mute"),
        System::MuteMic => fl!("system-mute-mic"),
        System::PlayPause => fl!("system-play-pause"),
        System::PlayNext => fl!("system-play-next"),
        System::PlayPrev => fl!("system-play-prev"),
        System::PowerOff => fl!("system-power-off"),
        System::ScreenReader => fl!("system-screen-reader"),
        System::Screenshot => fl!("system-screenshot"),
        System::Suspend => fl!("system-suspend"),
        System::Terminal => fl!("system-terminal"),
        System::TouchpadToggle => fl!("system-touchpad-toggle"),
        System::VolumeLower => fl!("system-volume-lower"),
        System::VolumeRaise => fl!("system-volume-raise"),
        System::WebBrowser => fl!("system-web-browser"),
        System::WindowSwitcher => fl!("system-window-switcher"),
        System::WindowSwitcherPrevious => fl!("system-window-switcher-previous"),
        System::WorkspaceOverview => fl!("system-workspace-overview"),
    }
}
