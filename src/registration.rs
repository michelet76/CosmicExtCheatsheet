// SPDX-License-Identifier: GPL-3.0-only

//! Registers the cheatsheet's global shortcut with the COSMIC compositor by
//! writing a `Spawn` binding into the user's custom shortcuts config
//! (`~/.config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom`).
//! cosmic-comp watches that file and reloads bindings immediately.
//!
//! That file holds every custom shortcut on the system, so every write here
//! is a fresh read-modify-write that touches only our own entry, and any
//! read problem aborts the write instead of being papered over.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use cosmic::cosmic_config::{self, Config, ConfigGet, ConfigSet};
use cosmic_settings_config::shortcuts::{self, Action, Binding, Shortcuts};

use crate::ids::{BIN_NAME, SHORTCUT_DESCRIPTION};

/// Why a registration change could not be made.
#[derive(Debug)]
pub enum Error {
    /// The compositor config could not be read or written.
    Config(cosmic_config::Error),
    /// The custom shortcuts file holds entries this build cannot decode
    /// (a newer compositor, most likely); rewriting it would corrupt them.
    Undecodable(String),
    /// The wanted combination is bound to something else.
    Conflict(Action),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(why) => write!(f, "{why}"),
            Error::Undecodable(entry) => write!(
                f,
                "the custom shortcuts file contains an entry this version cannot read ({entry}); update the app"
            ),
            Error::Conflict(action) => write!(f, "combination already bound to {action:?}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<cosmic_config::Error> for Error {
    fn from(why: cosmic_config::Error) -> Self {
        Error::Config(why)
    }
}

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
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._-+".contains(c))
    {
        s.into_owned()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

/// Split a command the way `spawn_command` quotes it: whitespace-separated
/// words, with single-quoted words allowed to contain anything (and `'\''`
/// for a literal quote). Double quotes and backslashes are not interpreted,
/// which is enough to recognise our own commands and reject everything else.
pub fn split_command(command: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    let mut chars = command.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('\'') => {
                            // `'\''` inside a quoted word is an escaped quote.
                            if chars.peek() == Some(&'\\') {
                                let mut look = chars.clone();
                                look.next();
                                if look.next() == Some('\'') && look.next() == Some('\'') {
                                    chars = look;
                                    current.push('\'');
                                    continue;
                                }
                            }
                            break;
                        }
                        Some(other) => current.push(other),
                        None => break,
                    }
                }
            }
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut current));
                    in_word = false;
                }
            }
            c => {
                in_word = true;
                current.push(c);
            }
        }
    }
    if in_word {
        words.push(current);
    }
    words
}

/// Whether an action is exactly one of ours: this binary (by name or path)
/// invoked with the single `toggle` argument that `spawn_command` writes.
pub fn is_own_action(action: &Action) -> bool {
    match action {
        Action::Spawn(cmd) => {
            let words = split_command(cmd);
            match words.as_slice() {
                [program, arg] => {
                    arg == "toggle"
                        && Path::new(program)
                            .file_name()
                            .is_some_and(|name| name == BIN_NAME)
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Path of the `custom` file cosmic-config reads for this context: the user
/// file if present, else the system one.
fn custom_path() -> Option<PathBuf> {
    let user = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|dir| {
            dir.join("cosmic")
                .join(shortcuts::ID)
                .join("v1")
                .join("custom")
        });
    match user {
        Some(path) if path.exists() => Some(path),
        _ => {
            let system = Path::new("/usr/share/cosmic")
                .join(shortcuts::ID)
                .join("v1")
                .join("custom");
            system.exists().then_some(system)
        }
    }
}

/// Make sure every action in the custom file decodes with this build's
/// `Action` type. The typed reader silently turns unknown actions into
/// `Disable`, and writing that back would destroy them.
fn check_decodable() -> Result<(), Error> {
    let Some(path) = custom_path() else {
        return Ok(());
    };
    let text = std::fs::read_to_string(&path).map_err(cosmic_config::Error::from)?;
    let raw: HashMap<Binding, Box<ron::value::RawValue>> =
        ron::from_str(&text).map_err(|why| Error::Undecodable(why.to_string()))?;
    for (binding, value) in raw {
        if value.into_rust::<Action>().is_err() {
            return Err(Error::Undecodable(format!(
                "{}: {}",
                binding.to_string(),
                value.get_ron().trim()
            )));
        }
    }
    Ok(())
}

/// Read the user's custom shortcuts. A missing file is an empty map; any
/// other problem is an error, because the callers write the map back.
pub fn read_custom(ctx: &Config) -> Result<Shortcuts, Error> {
    match ctx.get::<Shortcuts>("custom") {
        Ok(shortcuts) => {
            check_decodable()?;
            Ok(shortcuts)
        }
        Err(why) if !why.is_err() => Ok(Shortcuts::default()),
        Err(why) => Err(Error::Config(why)),
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
pub fn is_registered(ctx: &Config, binding: &Binding) -> Result<bool, Error> {
    Ok(read_custom(ctx)?.0.get(binding).is_some_and(is_own_action))
}

/// Write `binding -> Spawn(<cheatsheet>)` into the custom shortcuts.
///
/// Removes our previous registration at `old` (only if it is still ours) and
/// whatever is bound to `new`; the caller is responsible for having checked
/// [`conflict`] first. Every other entry is preserved. The file is re-read
/// right before writing to keep the window for concurrent edits minimal.
pub fn register(ctx: &Config, old: Option<&Binding>, new: &Binding) -> Result<(), Error> {
    let mut custom = read_custom(ctx)?;

    if let Some(old) = old {
        custom.0.retain(|b, a| !(b == old && is_own_action(a)));
    }

    // `Binding`'s Eq/Hash ignore `description`, and `HashMap::insert` keeps the
    // old key, so remove first to make sure our description is stored.
    custom.0.remove(new);

    let mut binding = new.clone();
    binding.keycode = None;
    if binding.description.is_none() {
        binding.description = Some(SHORTCUT_DESCRIPTION.to_owned());
    }
    custom.0.insert(binding, Action::Spawn(spawn_command()));

    ctx.set("custom", custom)?;
    Ok(())
}

/// Like [`register`], but refuses when the combination is bound to another
/// action unless `force` is set.
pub fn register_checked(
    ctx: &Config,
    old: Option<&Binding>,
    new: &Binding,
    force: bool,
) -> Result<(), Error> {
    if !force && let Some(action) = conflict(&shortcuts::shortcuts(ctx), new) {
        return Err(Error::Conflict(action));
    }
    register(ctx, old, new)
}

/// How "installed" a spawn command is; higher wins when deciding whether to
/// replace the registered command with the current binary's.
fn install_rank(command: &str) -> u8 {
    let words = split_command(command);
    let program = words.first().map(String::as_str).unwrap_or_default();
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

/// Outcome of [`sync_command`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sync {
    /// Our entry is present and already points at the best binary.
    Unchanged,
    /// Our entry was rewritten to point at the current binary.
    Updated,
    /// Nothing is bound to the combination any more.
    Missing,
    /// The user bound the combination to something else; left untouched.
    Foreign,
}

/// Keep our registered entry pointing at the best-installed binary, without
/// ever touching an entry that is no longer ours.
pub fn sync_command(ctx: &Config, binding: &Binding) -> Result<Sync, Error> {
    let custom = read_custom(ctx)?;
    let current = spawn_command();
    match custom.0.get(binding) {
        None => Ok(Sync::Missing),
        Some(action) if !is_own_action(action) => Ok(Sync::Foreign),
        Some(Action::Spawn(cmd))
            if cmd != &current && install_rank(&current) >= install_rank(cmd) =>
        {
            register(ctx, Some(binding), binding)?;
            Ok(Sync::Updated)
        }
        Some(_) => Ok(Sync::Unchanged),
    }
}

/// Remove our registration. With a known `registered` binding only that entry
/// is removed (and only if it is still ours); without one, every entry that
/// exactly matches our spawn signature is removed.
pub fn unregister(ctx: &Config, registered: Option<&Binding>) -> Result<(), Error> {
    let mut custom = read_custom(ctx)?;
    let before = custom.0.len();
    match registered {
        Some(binding) => custom.0.retain(|b, a| !(b == binding && is_own_action(a))),
        None => custom.0.retain(|_, a| !is_own_action(a)),
    }
    if custom.0.len() == before {
        return Ok(());
    }
    ctx.set("custom", custom)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_plain_and_quoted_commands() {
        assert_eq!(
            split_command("cosmic-ext-cheatsheet toggle"),
            ["cosmic-ext-cheatsheet", "toggle"]
        );
        assert_eq!(
            split_command("'/home/u/My Apps/cosmic-ext-cheatsheet' toggle"),
            ["/home/u/My Apps/cosmic-ext-cheatsheet", "toggle"]
        );
        assert_eq!(split_command("'it'\\''s' x"), ["it's", "x"]);
        assert_eq!(split_command("  a   b  "), ["a", "b"]);
    }

    #[test]
    fn recognises_only_exact_toggle_commands() {
        let own = |cmd: &str| is_own_action(&Action::Spawn(cmd.to_owned()));
        assert!(own("cosmic-ext-cheatsheet toggle"));
        assert!(own("/usr/bin/cosmic-ext-cheatsheet toggle"));
        assert!(own("'/home/u/My Apps/cosmic-ext-cheatsheet' toggle"));
        assert!(!own("cosmic-ext-cheatsheet settings"));
        assert!(!own("cosmic-ext-cheatsheet"));
        assert!(!own("cosmic-ext-cheatsheet toggle now"));
        assert!(!own("other-tool toggle"));
        assert!(!is_own_action(&Action::Close));
    }

    #[test]
    fn quoting_round_trips() {
        let path = Path::new("/home/u/My Apps/it's");
        let cmd = format!("{} toggle", shell_quote(path));
        assert_eq!(split_command(&cmd), ["/home/u/My Apps/it's", "toggle"]);
    }
}
