//! Core interaction models for `headless-primitives`.
//!
//! Most consumers should start with the widget-level models re-exported at the
//! crate root:
//!
//! - [`CollapsibleModel`]
//! - [`DialogModel`]
//! - [`TabsModel`]
//!
//! Lower-level interaction utilities such as [`roving_focus`] and
//! [`typeahead`] remain public for custom composite widgets. They complement
//! the widget models above, but they are not the primary entry point for most
//! consumers.
//!
//! ```rust
//! use headless_primitives_core::{TabsActivation, TabsModel};
//!
//! let mut tabs = TabsModel::with_activation(3, TabsActivation::Manual);
//! assert_eq!(tabs.selected(), Some(0));
//!
//! tabs.focus_next();
//! assert_eq!(tabs.focused(), Some(1));
//! assert_eq!(tabs.selected(), Some(0));
//!
//! tabs.activate_focused();
//! assert_eq!(tabs.selected(), Some(1));
//! ```
//!
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

/// Headless state model for collapsible regions.
pub mod collapsible;
/// Headless state model for dialogs.
pub mod dialog;
/// Shared orientation enum used by interaction models.
pub mod orientation;
/// Low-level roving focus utilities used by composite widgets.
pub mod roving_focus;
/// Headless state model for tabs.
pub mod tabs;
/// Low-level typeahead matching helper.
pub mod typeahead;

pub use collapsible::{CollapsibleModel, CollapsibleState};
pub use dialog::{DialogModel, DialogState};
pub use orientation::Orientation;
pub use tabs::{TabsActivation, TabsModel};

#[cfg(test)]
extern crate std;
