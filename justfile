# Binary names
name := 'cosmic-ext-cheatsheet'
applet-name := 'cosmic-ext-applet-cheatsheet'
# Application IDs
appid := 'io.github.michelet76.CosmicExtCheatsheet'
applet-appid := 'io.github.michelet76.CosmicExtAppletCheatsheet'

# Path to root file system, which defaults to `/`.
rootdir := ''
# The prefix for the `/usr` directory.
prefix := '/usr'
# The location of the cargo target directory.
cargo-target-dir := env('CARGO_TARGET_DIR', 'target')

base-dir := absolute_path(clean(rootdir / prefix))
bin-dir := base-dir / 'bin'
desktop-dir := base-dir / 'share' / 'applications'
icons-dir := base-dir / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Runs a clippy check
check *args:
    cargo clippy --all-targets {{args}} -- -W clippy::pedantic

# Run the overlay for testing purposes
run *args:
    env RUST_BACKTRACE=full cargo run --bin {{name}} -- {{args}}

# Run the settings window
run-settings:
    env RUST_BACKTRACE=full cargo run --bin {{name}} -- settings

# Installs files (system-wide by default; use `just prefix=~/.local install` for a user install)
install:
    install -Dm0755 {{ cargo-target-dir / 'release' / name }} {{ bin-dir / name }}
    install -Dm0755 {{ cargo-target-dir / 'release' / applet-name }} {{ bin-dir / applet-name }}
    install -Dm0644 {{ 'res' / appid + '.desktop' }} {{ desktop-dir / appid + '.desktop' }}
    install -Dm0644 {{ 'res' / applet-appid + '.desktop' }} {{ desktop-dir / applet-appid + '.desktop' }}
    install -Dm0644 {{ 'res/icons/hicolor/scalable/apps' / appid + '.svg' }} {{ icons-dir / appid + '.svg' }}
    install -Dm0644 {{ 'res/icons/hicolor/scalable/apps' / appid + '-symbolic.svg' }} {{ icons-dir / appid + '-symbolic.svg' }}

# Builds and installs into ~/.local (no root needed)
install-user: build-release
    just prefix={{ env('HOME') / '.local' }} install

# Uninstalls installed files
uninstall:
    rm -f {{ bin-dir / name }} {{ bin-dir / applet-name }}
    rm -f {{ desktop-dir / appid + '.desktop' }} {{ desktop-dir / applet-appid + '.desktop' }}
    rm -f {{ icons-dir / appid + '.svg' }} {{ icons-dir / appid + '-symbolic.svg' }}

uninstall-user:
    just prefix={{ env('HOME') / '.local' }} uninstall
