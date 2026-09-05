// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;
use std::str::FromStr;

use clap::{Parser, Subcommand};
use cosmic::app::CosmicFlags;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "cosmic-ext-cheatsheet",
    version,
    about = "Keyboard shortcut cheatsheet for COSMIC"
)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    /// Show the cheatsheet, or hide it if it is already visible (default)
    Toggle,
    /// Show the cheatsheet
    Show,
    /// Hide the cheatsheet
    Hide,
    /// Open the settings window
    Settings,
    /// Write the configured shortcut into the compositor config and exit
    Register {
        /// Replace whatever is currently bound to the combination
        #[arg(long)]
        force: bool,
    },
    /// Remove the shortcut from the compositor config and exit
    Unregister,
}

impl Cmd {
    pub const fn as_str(self) -> &'static str {
        match self {
            Cmd::Toggle => "toggle",
            Cmd::Show => "show",
            Cmd::Hide => "hide",
            Cmd::Settings => "settings",
            Cmd::Register { .. } => "register",
            Cmd::Unregister => "unregister",
        }
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Cmd {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "toggle" => Cmd::Toggle,
            "show" => Cmd::Show,
            "hide" => Cmd::Hide,
            "settings" => Cmd::Settings,
            "register" => Cmd::Register { force: false },
            "unregister" => Cmd::Unregister,
            _ => return Err(()),
        })
    }
}

impl CosmicFlags for Args {
    type SubCommand = Cmd;
    type Args = Vec<String>;

    fn action(&self) -> Option<&Cmd> {
        self.cmd.as_ref()
    }
}
