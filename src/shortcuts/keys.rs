// SPDX-License-Identifier: GPL-3.0-only

//! Renders bindings as human-readable key chips.

use cosmic_settings_config::shortcuts::Binding;
use xkbcommon::xkb;

/// Chips for one binding, modifiers first: `["Super", "Shift", "/"]`.
pub fn chips(binding: &Binding) -> Vec<String> {
    let mut chips = Vec::with_capacity(5);
    if binding.modifiers.logo {
        chips.push("Super".to_owned());
    }
    if binding.modifiers.ctrl {
        chips.push("Ctrl".to_owned());
    }
    if binding.modifiers.alt {
        chips.push("Alt".to_owned());
    }
    if binding.modifiers.shift {
        chips.push("Shift".to_owned());
    }
    if let Some(key) = binding.key {
        chips.push(key_label(key));
    }
    chips
}

/// The whole binding as a single string, e.g. `Super + Shift + /`.
pub fn display(binding: &Binding) -> String {
    chips(binding).join(" + ")
}

/// Sort key so that, among several bindings for one action, arrow keys and
/// shorter combinations come first.
pub fn sort_key(binding: &Binding) -> (u8, usize, String) {
    let name = binding.key.map(xkb::keysym_get_name).unwrap_or_default();
    let arrow = match name.as_str() {
        "Left" | "Right" | "Up" | "Down" => 0,
        _ => 1,
    };
    let modifier_count = [
        binding.modifiers.logo,
        binding.modifiers.ctrl,
        binding.modifiers.alt,
        binding.modifiers.shift,
    ]
    .iter()
    .filter(|m| **m)
    .count();
    (arrow, modifier_count, name)
}

/// Human-readable label for a keysym.
pub fn key_label(key: xkb::Keysym) -> String {
    let name = xkb::keysym_get_name(key);
    let label = match name.as_str() {
        "Left" => "←",
        "Right" => "→",
        "Up" => "↑",
        "Down" => "↓",
        "Return" | "KP_Enter" => "⏎",
        "Escape" => "Esc",
        "space" => "Space",
        "Tab" => "Tab",
        "ISO_Left_Tab" => "Tab",
        "BackSpace" => "⌫",
        "Delete" => "Del",
        "Insert" => "Ins",
        "Home" => "Home",
        "End" => "End",
        "Page_Up" | "Prior" => "PgUp",
        "Page_Down" | "Next" => "PgDn",
        "Print" => "PrtSc",
        "slash" => "/",
        "backslash" => "\\",
        "question" => "?",
        "equal" => "=",
        "plus" => "+",
        "minus" => "-",
        "underscore" => "_",
        "period" => ".",
        "comma" => ",",
        "semicolon" => ";",
        "colon" => ":",
        "apostrophe" => "'",
        "quotedbl" => "\"",
        "grave" => "`",
        "asciitilde" => "~",
        "bracketleft" => "[",
        "bracketright" => "]",
        "braceleft" => "{",
        "braceright" => "}",
        "less" => "<",
        "greater" => ">",
        "bar" => "|",
        "asterisk" => "*",
        "ampersand" => "&",
        "percent" => "%",
        "dollar" => "$",
        "numbersign" => "#",
        "at" => "@",
        "exclam" => "!",
        "asciicircum" => "^",
        "parenleft" => "(",
        "parenright" => ")",
        "XF86AudioRaiseVolume" => "Vol +",
        "XF86AudioLowerVolume" => "Vol −",
        "XF86AudioMute" => "Mute",
        "XF86AudioMicMute" => "Mic mute",
        "XF86MonBrightnessUp" => "Brightness +",
        "XF86MonBrightnessDown" => "Brightness −",
        "XF86KbdBrightnessUp" => "Kbd light +",
        "XF86KbdBrightnessDown" => "Kbd light −",
        "XF86AudioPlay" => "Play",
        "XF86AudioPause" => "Pause",
        "XF86AudioStop" => "Stop",
        "XF86AudioPrev" => "Prev track",
        "XF86AudioNext" => "Next track",
        "XF86PowerOff" => "Power",
        "XF86Sleep" => "Sleep",
        "XF86TouchpadToggle" => "Touchpad",
        "XF86Display" => "Display",
        "XF86LaunchA" => "Launch A",
        "XF86Search" => "Search",
        "XF86Explorer" => "Explorer",
        "XF86WWW" => "WWW",
        "XF86HomePage" => "Home page",
        "XF86Calculator" => "Calc",
        "XF86Mail" => "Mail",
        _ => "",
    };
    if !label.is_empty() {
        return label.to_owned();
    }
    // Single characters (letters, digits) are shown upper-case.
    if name.chars().count() == 1 {
        return name.to_uppercase();
    }
    if let Some(rest) = name.strip_prefix("XF86") {
        return rest.to_owned();
    }
    if let Some(rest) = name.strip_prefix("KP_") {
        return format!("Num {rest}");
    }
    name
}
