// SPDX-License-Identifier: GPL-3.0-only

//! Registers the cheatsheet's global shortcut with the COSMIC compositor by
//! writing a `Spawn` binding into the user's custom shortcuts config
//! (`~/.config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom`).
//! cosmic-comp watches that file and reloads bindings immediately.

use std::path::Path;

use cosmic::cosmic_config::{self, Config, ConfigGet, ConfigSet};
use cosmic_settings_config::shortcuts::{self, Action, Binding, Shortcuts};

use crate::ids::{BIN_NAME, SHORTCUT_DESCRIPTION};

/// Open the compositor shortcuts config context.
pub fn context() -> Result<Config, cosmic_config::Error> {
    shortcuts::context()
}

/// The command the compositor should spawn to toggle the cheatsheet.
///
/// Uses the bare binary name when installed system-wide (so the entry stays
/// valid across upgrades), otherwise the absolute path of the running binary,
/// because the compositor's `PATH` may not include `~/.local/bin` or a cargo
/// target directory.
pub fn spawn_command() -> String {
    let exe = std::env::current_exe().ok();
    let program = match exe {
        Some(path) if !path.starts_with("/usr/bin") && !path.starts_with("/usr/local/bin") => {
            // Prefer a sibling binary named like the installed one when running
            // from a cargo target dir or ~/.local/bin.
            let sibling = path.with_file_name(BIN_NAME);
            if sibling.exists() {
                shell_quote(&sibling)
            } else {
                shell_quote(&path)
            }
        }
        _ => BIN_NAME.to_owned(),
    };
    format!("{program} toggle")
}

fn shell_quote(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.chars().all(|c| c.is_ascii_alphanumeric() || "/._-+".contains(c)) {
        s.into_owned()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

/// Whether an action is one of ours (a spawn of this binary in any form).
pub fn is_own_action(action: &Action) -> bool {
    match action {
        Action::Spawn(cmd) => {
            let program = cmd.split_whitespace().next().unwrap_or_default();
            Path::new(program).file_name().is_some_and(|name| name == BIN_NAME)
        }
        _ => false,
    }
}

/// Read the user's custom shortcuts; a missing file is treated as empty.
pub fn read_custom(ctx: &Config) -> Shortcuts {
    match ctx.get::<Shortcuts>("custom") {
        Ok(shortcuts) => shortcuts,
        Err(why) => {
            if why.is_err() {
                tracing::warn!("could not read custom shortcuts: {why}");
            }
            Shortcuts::default()
        }
    }
}

/// The action that would be shadowed or replaced by binding `binding`, if any.
///
/// Looks at the merged (defaults + custom) map, ignoring disabled entries and
/// our own registration.
pub fn conflict(merged: &Shortcuts, binding: &Binding) -> Option<Action> {
    merged
        .0
        .get(binding)
        .filter(|action| !matches!(action, Action::Disable) && !is_own_action(action))
        .cloned()
}

/// Whether our binding is currently present in the custom shortcuts.
pub fn is_registered(ctx: &Config, binding: &Binding) -> bool {
    read_custom(ctx)
        .0
        .get(binding)
        .is_some_and(is_own_action)
}

/// Write `binding -> Spawn(<cheatsheet>)` into the custom shortcuts, removing
/// any previous registration of ours (`old`, plus any stale entry pointing at
/// this binary). Other custom entries are preserved.
pub fn register(ctx: &Config, old: Option<&Binding>, new: &Binding) -> Result<(), cosmic_config::Error> {
    let mut custom = read_custom(ctx);

    if let Some(old) = old {
        custom.0.retain(|b, a| !(b == old && is_own_action(a)));
    }
    // Any other stale entry that spawns us (e.g. from a previous install path).
    custom.0.retain(|_, a| !is_own_action(a));

    // `Binding`'s Eq/Hash ignore `description`, and `HashMap::insert` keeps the
    // old key, so remove first to make sure our description is stored.
    custom.0.remove(new);

    let mut binding = new.clone();
    binding.keycode = None;
    if binding.description.is_none() {
        binding.description = Some(SHORTCUT_DESCRIPTION.to_owned());
    }
    custom.0.insert(binding, Action::Spawn(spawn_command()));

    ctx.set("custom", custom)
}

/// How "installed" a spawn command is; higher wins when deciding whether to
/// replace the registered command with the current binary's.
fn install_rank(command: &str) -> u8 {
    let program = command.split_whitespace().next().unwrap_or_default();
    if program == BIN_NAME {
        return 3;
    }
    let path = Path::new(program);
    let home_bin = std::env::var_os("HOME").map(|h| Path::new(&h).join(".local/bin"));
    if home_bin.is_some_and(|dir| path.starts_with(dir)) {
        return 2;
    }
    if path.exists() { 1 } else { 0 }
}

/// Make sure the registered entry for `binding` spawns the current binary,
/// unless the existing entry points at a better-installed copy (system or
/// `~/.local/bin`) that still exists. Returns `Ok(true)` when it rewrote it.
pub fn sync_command(ctx: &Config, binding: &Binding) -> Result<bool, cosmic_config::Error> {
    let custom = read_custom(ctx);
    let current = spawn_command();
    let existing = custom.0.get(binding).filter(|a| is_own_action(a));
    let replace = match existing {
        None => true,
        Some(Action::Spawn(cmd)) => cmd != &current && install_rank(&current) >= install_rank(cmd),
        Some(_) => false,
    };
    if !replace {
        return Ok(false);
    }
    register(ctx, Some(binding), binding).map(|()| true)
}

/// Remove every custom entry that spawns this binary.
pub fn unregister(ctx: &Config) -> Result<(), cosmic_config::Error> {
    let mut custom = read_custom(ctx);
    let before = custom.0.len();
    custom.0.retain(|_, a| !is_own_action(a));
    if custom.0.len() == before {
        return Ok(());
    }
    ctx.set("custom", custom)
}
