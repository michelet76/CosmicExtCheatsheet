// SPDX-License-Identifier: GPL-3.0-only

//! Persistent configuration of the cheatsheet app itself.

use std::str::FromStr;

use cosmic::cosmic_config::{self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use cosmic_settings_config::shortcuts::Binding;

use crate::ids::{APP_ID, SHORTCUT_DESCRIPTION};

/// Default key combination: Super+Shift+/ (i.e. Super+?).
pub const DEFAULT_BINDING: &str = "Super+Shift+slash";

#[derive(Debug, Clone, PartialEq, CosmicConfigEntry)]
#[version = 1]
pub struct CheatsheetConfig {
    /// The key combination the user wants to open the cheatsheet with.
    pub binding: Binding,
    /// The binding that was last written into the compositor's custom shortcuts.
    /// Used to remove the previous entry when the user changes the combination.
    pub registered: Option<Binding>,
    /// Keep the process alive after hiding the overlay (faster re-open).
    pub resident: bool,
    /// Register the default binding automatically on first launch.
    pub auto_register: bool,
}

impl Default for CheatsheetConfig {
    fn default() -> Self {
        Self {
            binding: default_binding(),
            registered: None,
            resident: false,
            auto_register: true,
        }
    }
}

/// The default binding, with the description the compositor shows for it.
pub fn default_binding() -> Binding {
    let mut binding = Binding::from_str(DEFAULT_BINDING).expect("default binding is valid");
    binding.description = Some(SHORTCUT_DESCRIPTION.to_owned());
    binding
}

impl CheatsheetConfig {
    /// Open the cosmic-config context for this app.
    pub fn context() -> Result<Config, cosmic_config::Error> {
        Config::new(APP_ID, Self::VERSION)
    }

    /// Load the configuration, falling back to defaults for missing keys.
    pub fn load() -> (Self, Option<Config>) {
        match Self::context() {
            Ok(context) => {
                let config = match Self::get_entry(&context) {
                    Ok(config) => config,
                    Err((errors, config)) => {
                        for why in errors {
                            if !why.is_err() {
                                continue;
                            }
                            tracing::warn!("cheatsheet config error: {why}");
                        }
                        config
                    }
                };
                (config, Some(context))
            }
            Err(why) => {
                tracing::error!("could not open cheatsheet config: {why}");
                (Self::default(), None)
            }
        }
    }

    /// Persist the whole configuration.
    pub fn save(&self, context: &Config) {
        if let Err(why) = self.write_entry(context) {
            tracing::error!("could not write cheatsheet config: {why}");
        }
    }
}
