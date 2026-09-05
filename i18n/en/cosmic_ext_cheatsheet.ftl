app-title = COSMIC Cheatsheet
applet-tooltip = Keyboard shortcuts
shortcut-description = COSMIC Cheatsheet

# Overlay
overlay-hint = Press Esc or click outside to close
overlay-settings = Settings
overlay-close = Close
overlay-empty = No shortcuts found. Is COSMIC running?

# Settings window
settings-title = Cheatsheet settings
settings-shortcut-section = Global shortcut
settings-current-binding = Current combination
settings-not-registered = Not registered
settings-record = Record
settings-recording = Press the new combination… (Esc to cancel)
settings-type-binding = Or type it, e.g. Super+Shift+slash
settings-apply = Apply
settings-remove = Remove shortcut
settings-behaviour-section = Behaviour
settings-resident = Keep running in the background after closing
settings-resident-desc = Reopens faster, uses a little memory while idle.
settings-invalid-binding = This combination is not valid: { $error }
settings-registered-ok = Shortcut { $binding } is active.
settings-write-error = Could not update the shortcuts config: { $error }
replace-title = Replace shortcut?
replace-body = { $binding } is already used for "{ $action }". Replace it with the cheatsheet?
replace-confirm = Replace
replace-cancel = Cancel

# Categories
cat-navigation = Navigation
cat-manage-windows = Manage windows
cat-move-windows = Move windows
cat-window-tiling = Window tiling
cat-system = System
cat-accessibility = Accessibility
cat-custom = Custom shortcuts
cat-other = Other

# Actions
action-close = Close window
action-debug = Debug overlay
action-disable = Disabled
action-focus = Focus window { $direction ->
    [in] in
    [out] out
    [left] left
    [right] right
    [up] up
   *[down] down
}
action-last-workspace = Switch to last workspace
action-maximize = Maximize window
action-fullscreen = Fullscreen window
action-minimize = Minimize window
action-migrate-workspace-next-output = Move workspace to next display
action-migrate-workspace-prev-output = Move workspace to previous display
action-migrate-workspace-output = Move workspace to display { $direction ->
    [left] left
    [right] right
    [up] above
   *[down] below
}
action-move = Move window { $direction ->
    [left] left
    [right] right
    [up] up
   *[down] down
}
action-move-last-workspace = Move window to last workspace
action-move-next-workspace = Move window to next workspace
action-move-prev-workspace = Move window to previous workspace
action-move-next-output = Move window to next display
action-move-prev-output = Move window to previous display
action-move-output = Move window to display { $direction ->
    [left] left
    [right] right
    [up] above
   *[down] below
}
action-move-workspace = Move window to workspace { $num }
action-next-output = Focus next display
action-prev-output = Focus previous display
action-next-workspace = Switch to next workspace
action-prev-workspace = Switch to previous workspace
action-orientation-horizontal = Set horizontal orientation
action-orientation-vertical = Set vertical orientation
action-resize-inwards = Resize window inwards
action-resize-outwards = Resize window outwards
action-swap-window = Swap window
action-switch-output = Focus display { $direction ->
    [left] left
    [right] right
    [up] above
   *[down] below
}
action-terminate = Exit the COSMIC session
action-toggle-orientation = Toggle orientation
action-toggle-stacking = Toggle window stacking
action-toggle-sticky = Toggle sticky window
action-toggle-tiling = Toggle window tiling
action-toggle-floating = Toggle window floating
action-workspace = Switch to workspace { $num }
action-zoom-in = Zoom in
action-zoom-out = Zoom out

system-app-library = Open the app library
system-brightness-down = Decrease display brightness
system-brightness-up = Increase display brightness
system-display-toggle = Toggle internal display
system-home-folder = Open home folder
system-input-source-switch = Switch input source
system-keyboard-brightness-down = Decrease keyboard brightness
system-keyboard-brightness-up = Increase keyboard brightness
system-launcher = Open the launcher
system-lock-screen = Lock the screen
system-log-out = Log out
system-mute = Mute audio output
system-mute-mic = Mute microphone
system-play-pause = Play / pause
system-play-next = Next track
system-play-prev = Previous track
system-power-off = Power off
system-screen-reader = Toggle screen reader
system-screenshot = Take a screenshot
system-suspend = Suspend
system-terminal = Open a terminal
system-touchpad-toggle = Toggle touchpad
system-volume-lower = Decrease volume
system-volume-raise = Increase volume
system-web-browser = Open a web browser
system-window-switcher = Switch between open windows
system-window-switcher-previous = Switch between open windows (reverse)
system-workspace-overview = Open the workspace overview
