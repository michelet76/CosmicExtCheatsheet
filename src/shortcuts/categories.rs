// SPDX-License-Identifier: GPL-3.0-only

//! Maps compositor actions to cheatsheet categories (mirrors cosmic-settings).

use cosmic_settings_config::shortcuts::Action;
use cosmic_settings_config::shortcuts::action::System;

/// Cheatsheet sections, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CategoryKind {
    Navigation,
    ManageWindows,
    MoveWindows,
    WindowTiling,
    System,
    Accessibility,
    Custom,
    Other,
}

#[allow(deprecated)]
pub fn category_of(action: &Action) -> CategoryKind {
    use CategoryKind as C;
    match action {
        Action::Focus(_)
        | Action::Workspace(_)
        | Action::LastWorkspace
        | Action::NextWorkspace
        | Action::PreviousWorkspace
        | Action::SwitchOutput(_)
        | Action::NextOutput
        | Action::PreviousOutput => C::Navigation,

        Action::Close
        | Action::Maximize
        | Action::Fullscreen
        | Action::Minimize
        | Action::Resizing(_)
        | Action::ToggleSticky => C::ManageWindows,

        Action::Move(_)
        | Action::MoveToWorkspace(_)
        | Action::MoveToLastWorkspace
        | Action::MoveToNextWorkspace
        | Action::MoveToPreviousWorkspace
        | Action::MoveToOutput(_)
        | Action::MoveToNextOutput
        | Action::MoveToPreviousOutput
        | Action::SendToWorkspace(_)
        | Action::SendToLastWorkspace
        | Action::SendToNextWorkspace
        | Action::SendToPreviousWorkspace
        | Action::SendToOutput(_)
        | Action::SendToNextOutput
        | Action::SendToPreviousOutput
        | Action::MigrateWorkspaceToOutput(_)
        | Action::MigrateWorkspaceToNextOutput
        | Action::MigrateWorkspaceToPreviousOutput => C::MoveWindows,

        Action::Orientation(_)
        | Action::ToggleOrientation
        | Action::ToggleTiling
        | Action::ToggleStacking
        | Action::ToggleWindowFloating
        | Action::SwapWindow => C::WindowTiling,

        Action::ZoomIn | Action::ZoomOut | Action::System(System::ScreenReader) => C::Accessibility,

        Action::System(_) => C::System,

        Action::Spawn(_) => C::Custom,

        Action::Terminate | Action::Debug | Action::Disable => C::Other,
    }
}
