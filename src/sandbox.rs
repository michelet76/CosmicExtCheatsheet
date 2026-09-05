// SPDX-License-Identifier: GPL-3.0-only

//! Helpers for behaving correctly inside a Flatpak sandbox.
//!
//! Two things change when the app is packaged as a Flatpak:
//!
//! * the compositor lives on the host, so the shortcut it stores must launch
//!   the app through `flatpak run` rather than by path;
//! * commands the user clicks in the cheatsheet belong to the host session,
//!   so they must be handed to `flatpak-spawn --host` instead of being run
//!   inside the sandbox, where they do not exist.

use std::process::Command;
use std::sync::LazyLock;

use crate::ids::{APP_ID, BIN_NAME};

/// Whether this process is running inside a Flatpak sandbox.
pub fn active() -> bool {
    static ACTIVE: LazyLock<bool> =
        LazyLock::new(|| std::path::Path::new("/.flatpak-info").exists());
    *ACTIVE
}

/// The command a host process (the compositor) must run to toggle the app.
pub fn host_launch_command() -> String {
    format!("flatpak run --command={BIN_NAME} {APP_ID} toggle")
}

/// Prepare a command that runs `/bin/sh -c <command>` on the host when
/// sandboxed, and directly otherwise.
pub fn shell_command(command: &str) -> Command {
    let mut spawned = if active() {
        let mut c = Command::new("flatpak-spawn");
        c.arg("--host").arg("/bin/sh");
        c
    } else {
        Command::new("/bin/sh")
    };
    spawned.arg("-c").arg(command);
    spawned
}
