## Keycode-only compositor shortcuts are omitted
- **Location:** src/shortcuts/mod.rs:103
- **Claimed:** Bindings with `key == None, keycode != None` are skipped, so some active shortcuts never appear.
- **Rejected because:** Deliberate. No COSMIC tool produces keycode bindings: cosmic-settings explicitly sets `keycode: None` when saving, the shipped defaults contain none, and a raw keycode has no layout-independent label to show. Revisit only if a real source of keycode bindings appears.
- **Date:** 2026-09-05

## Enter selection stops at the first non-runnable result
- **Location:** src/shortcuts/mod.rs:53 (`Search::enter_target`)
- **Claimed:** Enter should run the first runnable entry anywhere in the results.
- **Rejected because:** Intended behaviour, introduced to fix the previous review's finding that Enter ran an entry far from the top of the visible list. Enter acts only on the first visible match, and that row is highlighted so the target is unambiguous.
- **Date:** 2026-09-05

## `just install` does not build release artifacts first
- **Location:** justfile:47
- **Claimed:** `install` should depend on `build-release`.
- **Rejected because:** Deliberate. `install` is run as root (`sudo just install`); a build dependency would compile as root into the user's target dir. README documents `just build-release` then `sudo just install`; `install-user` builds first because it needs no root.
- **Date:** 2026-09-05
