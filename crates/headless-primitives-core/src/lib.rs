//! Core interaction models for `headless-primitives`.
//!
//! Most consumers should start with the widget-level models re-exported at the
//! crate root:
//!
//! - [`CollapsibleModel`]
//! - [`DialogModel`]
//! - [`TabsModel`]
//!
//! Lower-level modules such as [`roving_focus`], [`typeahead`], [`controlled`],
//! [`ids`], and [`state_machine`] remain public for advanced composition and
//! backward compatibility, but they are supporting utilities rather than the
//! main entry point.
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
/// Advanced controlled-value helper for custom interaction state.
pub mod controlled;
/// Headless state model for dialogs.
pub mod dialog;
/// Deterministic ID helpers for headless widgets.
pub mod ids;
/// Shared orientation enum used by interaction models.
pub mod orientation;
/// Low-level roving focus utilities used by composite widgets.
pub mod roving_focus;
/// Generic state machine helper retained for advanced composition.
pub mod state_machine;
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
