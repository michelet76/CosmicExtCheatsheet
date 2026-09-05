// SPDX-License-Identifier: GPL-3.0-only

//! Turns iced keyboard events into a compositor [`Binding`].

use cosmic::iced::keyboard::key::Named;
use cosmic::iced::keyboard::{Key, Location, Modifiers as IcedModifiers};
use cosmic::iced::platform_specific::shell::wayland::keymap;
use cosmic_settings_config::shortcuts::{Binding, Modifiers};

/// Convert iced modifier flags into compositor modifiers.
pub fn modifiers(modifiers: IcedModifiers) -> Modifiers {
    let mut out = Modifiers::new();
    if modifiers.logo() {
        out = out.logo();
    }
    if modifiers.control() {
        out = out.ctrl();
    }
    if modifiers.alt() {
        out = out.alt();
    }
    if modifiers.shift() {
        out = out.shift();
    }
    out
}

/// Whether the key is itself a modifier (and therefore never the "main" key).
pub fn is_modifier_key(key: &Key) -> bool {
    matches!(
        key,
        Key::Named(
            Named::Super
                | Named::Meta
                | Named::Hyper
                | Named::Alt
                | Named::AltGraph
                | Named::Control
                | Named::Shift
                | Named::CapsLock
                | Named::NumLock
                | Named::ScrollLock
                | Named::Fn
                | Named::FnLock
                | Named::Symbol
                | Named::SymbolLock
        )
    )
}

/// Build a binding from a key press. Returns `None` for pure modifier presses
/// or keys without an xkb keysym.
pub fn binding_from_key(key: &Key, location: Location, mods: IcedModifiers) -> Option<Binding> {
    if is_modifier_key(key) {
        return None;
    }
    let keysym = keymap::key_to_keysym(key.clone(), location)?;
    let mut binding = Binding::new(modifiers(mods), Some(keysym));
    binding.keycode = None;
    Some(binding)
}

/// Validate a binding the user chose. Returns a human-readable reason if it
/// cannot be used as a global shortcut.
pub fn validate(binding: &Binding) -> Result<(), String> {
    if !binding.is_set() {
        return Err("a modifier plus a key is required".to_owned());
    }
    if binding.is_super() {
        return Err("Super alone is reserved for the launcher".to_owned());
    }
    Ok(())
}
