// SPDX-License-Identifier: GPL-3.0-only

//! Shared library for the COSMIC Cheatsheet overlay and panel applet.

pub mod config;
pub mod i18n;
pub mod ids;
pub mod keycapture;
pub mod registration;
pub mod sandbox;
pub mod shortcuts;
pub mod view;

pub use cosmic_settings_config::shortcuts::{Action, Binding, Modifiers};
