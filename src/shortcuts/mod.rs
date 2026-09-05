// SPDX-License-Identifier: GPL-3.0-only

//! Loads the effective COSMIC shortcuts (system defaults merged with the
//! user's custom bindings) and turns them into a categorised cheatsheet model.

pub mod categories;
pub mod keys;
pub mod localize;

use std::collections::BTreeMap;

use cosmic_settings_config::shortcuts::{self, Action, Binding, Shortcuts};

pub use categories::CategoryKind;

/// One row of the cheatsheet: an action and all key combinations bound to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub action: Action,
    pub label: String,
    /// Each inner vec is one key combination rendered as chips, e.g.
    /// `["Super", "Shift", "←"]`.
    pub bindings: Vec<Vec<String>>,
}

/// A titled group of entries.
#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub kind: CategoryKind,
    pub title: String,
    pub entries: Vec<Entry>,
}

/// The whole cheatsheet.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CheatsheetModel {
    pub categories: Vec<Category>,
}

impl CheatsheetModel {
    /// Load from the compositor config (defaults + custom).
    pub fn load() -> Self {
        match shortcuts::context() {
            Ok(ctx) => Self::from_shortcuts(&shortcuts::shortcuts(&ctx)),
            Err(why) => {
                tracing::error!("could not open shortcuts config: {why}");
                Self::default()
            }
        }
    }

    /// Build from an already merged map of bindings.
    pub fn from_shortcuts(shortcuts: &Shortcuts) -> Self {
        // Group every binding by its action. `Action: Ord`, and the enum's
        // declaration order keeps related actions (Focus(..), Workspace(n),
        // System(..)) next to each other.
        let mut by_action: BTreeMap<&Action, Vec<&Binding>> = BTreeMap::new();
        for (binding, action) in shortcuts.iter() {
            if matches!(action, Action::Disable | Action::Debug) {
                continue;
            }
            // Keycode-only bindings can't be displayed meaningfully.
            if binding.key.is_none() && !binding.is_super() {
                continue;
            }
            by_action.entry(action).or_default().push(binding);
        }

        let mut categories: BTreeMap<CategoryKind, Vec<Entry>> = BTreeMap::new();
        for (action, mut bindings) in by_action {
            bindings.sort_by_key(|b| keys::sort_key(b));
            let entry = Entry {
                action: action.clone(),
                label: localize::action_label(action, bindings.first().copied()),
                bindings: bindings.into_iter().map(keys::chips).collect(),
            };
            categories
                .entry(categories::category_of(action))
                .or_default()
                .push(entry);
        }

        Self {
            categories: categories
                .into_iter()
                .map(|(kind, entries)| Category {
                    kind,
                    title: localize::category_title(kind),
                    entries,
                })
                .collect(),
        }
    }

    /// Total number of entries across all categories.
    pub fn len(&self) -> usize {
        self.categories.iter().map(|c| c.entries.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
