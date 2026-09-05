# Packaging

Everything needed to publish COSMIC Cheatsheet. The app ID is
`io.github.michelet76.CosmicExtCheatsheet`, which matches this repository so that Flathub's
app-ID check resolves.

## Flatpak and the COSMIC Store

The COSMIC Store lists Flatpak applications, so Flathub is the route into the Store.

### Files

| File | Purpose |
| --- | --- |
| `flatpak/io.github.michelet76.CosmicExtCheatsheet.yml` | The manifest |
| `flatpak/cargo-sources.json` | Generated offline sources for every crate |
| `flatpak/flathub-exceptions.json` | Justifications for the three permissions Flathub reviews |

Regenerate the cargo sources after any dependency change:

```sh
python3 -m venv /tmp/fpvenv && /tmp/fpvenv/bin/pip install aiohttp tomlkit
curl -O https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
/tmp/fpvenv/bin/python flatpak-cargo-generator.py Cargo.lock -o packaging/flatpak/cargo-sources.json
```

### Build and lint locally

```sh
flatpak install -y --user flathub org.flatpak.Builder
flatpak run org.flatpak.Builder --user --install-deps-from=flathub --force-clean \
    --repo=/tmp/fprepo /tmp/fpbuild packaging/flatpak/io.github.michelet76.CosmicExtCheatsheet.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest \
    packaging/flatpak/io.github.michelet76.CosmicExtCheatsheet.yml
```

### Three permissions need a Flathub exception

The linter reports these as errors. All three are in the category Flathub grants "on sufficient
explanation", and the reasoning is written out in `flatpak/flathub-exceptions.json`:

- **`--filesystem=host-os:ro`** reads the compositor's shipped shortcut defaults from
  `/usr/share/cosmic`. Flatpak reserves `/usr`, so there is no narrower option; a request for
  `--filesystem=/usr/share/cosmic:ro` is refused outright with "Path /usr is reserved by Flatpak".
  Without this the sheet can only show custom bindings.
- **`--filesystem=xdg-config/cosmic`** writes the single entry that registers the global shortcut.
  This is how COSMIC Settings does it too, and there is no GlobalShortcuts portal in
  xdg-desktop-portal-cosmic.
- **`--talk-name=org.freedesktop.Flatpak`** lets a clicked row run its command on the host through
  `flatpak-spawn`, since those commands belong to the session and not to the sandbox.

### Submitting

1. Fork <https://github.com/flathub/flathub> without copying only the master branch, then
   `git clone --branch=new-pr git@github.com:<you>/flathub.git`.
2. `git checkout -b cosmic-ext-cheatsheet new-pr`, add the manifest and `cargo-sources.json` at the
   top level, commit.
3. Open a pull request against the `new-pr` branch titled
   `Add io.github.michelet76.CosmicExtCheatsheet`. Explain the three permissions in the description
   and link to the exception reasons.
4. Separately, open a pull request against
   <https://github.com/flathub/flatpak-builder-lint> adding the contents of
   `flatpak/flathub-exceptions.json` to `flatpak_builder_lint/staticfiles/exceptions.json`.
5. Comment `bot, build` on the submission PR once it is ready for a test build.
6. Two-factor authentication must be enabled on the GitHub account.

Updates after the first publication do not go through review; they are pushed to the
`flathub/io.github.michelet76.CosmicExtCheatsheet` repository you are granted.

## AUR

`aur/PKGBUILD` builds the tagged release, `aur/PKGBUILD-git` builds from `main`. Both install
through the project's `justfile`, so they stay in step with a source install.

```sh
git clone ssh://aur@aur.archlinux.org/cosmic-ext-cheatsheet.git
cp packaging/aur/PKGBUILD cosmic-ext-cheatsheet/
cd cosmic-ext-cheatsheet
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO && git commit -m "Initial import" && git push
```

Publishing needs an SSH key registered on an aur.archlinux.org account. Repeat with
`PKGBUILD-git` in a `cosmic-ext-cheatsheet-git` repository.

## Where else to announce

- **cosmic-utils**, a community organisation hosting many COSMIC applets. They accept transfers of
  existing projects and hand out membership on request through their Mattermost channel.
- The COSMIC and Pop!\_OS community channels, and r/pop_os.
