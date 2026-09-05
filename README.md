# COSMIC Cheatsheet

An on-demand keyboard-shortcut cheatsheet for the [COSMIC](https://system76.com/cosmic) desktop.

Press a global key combination (default **Super + Shift + /**, i.e. *Super + ?*) and a full-screen
overlay lists every shortcut the compositor currently has configured, grouped by category.
Press the combination again, hit **Esc**, or click outside the card to close it.

The list is read live from COSMIC's own shortcut configuration
(`com.system76.CosmicSettings.Shortcuts`: system defaults merged with your custom bindings), so it
always matches what your keyboard actually does, including bindings you changed or disabled in
*Settings → Keyboard → Shortcuts*.

The project ships two binaries:

| Binary | Purpose |
| --- | --- |
| `cosmic-ext-cheatsheet` | The overlay, the settings window and a small CLI |
| `cosmic-ext-applet-cheatsheet` | A panel applet: a button that opens the same overlay |

## How the global shortcut works

COSMIC has no portal for global shortcuts yet, so the app registers itself the same way
*Settings → Keyboard → Custom shortcuts* does: it writes a
`Binding → Spawn("cosmic-ext-cheatsheet toggle")` entry into
`~/.config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom`. The compositor watches that
file, so the new combination works immediately, without logging out. Your other custom shortcuts
are left untouched.

On first launch the default combination is registered automatically if it is free. You can change
it at any time in the settings window, either by recording a key press or by typing it
(`Super+Alt+k`, `Ctrl+Alt+slash`, …). If the combination is already used by another action you are
asked whether to replace it.

## Build and install

Requirements: Rust 1.93+, `just`, and the usual libcosmic build dependencies
(`pkgconf`, `libxkbcommon`, `wayland`, `fontconfig`, `expat`, a Vulkan loader).

```sh
# Build and install into ~/.local (bin, applications, icons); no root needed
just install-user

# …or system-wide
just build-release
sudo just install
```

Then:

1. Launch **COSMIC Cheatsheet** once from the app library (or run `cosmic-ext-cheatsheet`).
   This registers **Super + Shift + /**.
2. Optionally add the **Cheatsheet** applet to the panel:
   *Settings → Desktop → Panel → Configure panel applets*.

To remove everything:

```sh
cosmic-ext-cheatsheet unregister   # removes the shortcut from the compositor config
just uninstall-user                # or: sudo just uninstall
```

## Command line

```
cosmic-ext-cheatsheet [toggle|show|hide|settings|register|unregister]
```

* `toggle` (default) — show the overlay, or hide it if it is already open. Only one instance ever
  runs; a second invocation talks to the first over D-Bus.
* `settings` — open the settings window.
* `register` / `unregister` — write or remove the shortcut entry without opening a window.
  `register` refuses to replace a combination that is bound to something else unless you pass
  `--force`. `unregister` also turns off automatic registration, so the next launch will not
  re-add the shortcut; `register` (or choosing a combination in Settings) turns it back on.

The app only ever touches its own entry in the custom shortcuts file, re-reads the file right
before each write, and refuses to write at all if the file contains an entry this version cannot
decode (which can happen when the compositor is newer than the app).

## Configuration

Stored with cosmic-config under `~/.config/cosmic/io.github.michelet76.CosmicExtCheatsheet/v1/`:

| Key | Meaning |
| --- | --- |
| `binding` | The chosen combination |
| `registered` | The combination currently written into the compositor config |
| `resident` | Keep the process alive after closing the overlay (faster to reopen) |
| `auto_register` | Register the default combination on first launch |

## Development

```sh
cargo run --bin cosmic-ext-cheatsheet            # overlay
cargo run --bin cosmic-ext-cheatsheet -- settings
cargo run --example dump                         # print the categorised shortcut list
just check                                       # clippy
```

While developing, the registered spawn command points at the binary in `target/`; after
`just install-user` the app updates it to the installed copy on its next launch.

## License

GPL-3.0-only.
