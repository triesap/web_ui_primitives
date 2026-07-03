//! Core interaction models for `web_ui_primitives`.
//!
//! Most consumers should start with the widget-level models re-exported at the
//! crate root:
//!
//! - [`CollapsibleModel`]
//! - [`DialogModel`]
//! - [`MenuModel`]
//! - [`TabsModel`]
//!
//! Lower-level interaction utilities such as [`roving_focus`] and
//! [`typeahead`] remain public for custom composite widgets. They complement
//! the widget models above, but they are not the primary entry point for most
//! consumers.
//!
//! ```rust
//! use web_ui_primitives_core::{TabsActivation, TabsModel};
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
/// Headless state model for menus.
pub mod menu;
/// Shared orientation enum used by interaction models.
pub mod orientation;
/// Shared layer placement model for floating UI surfaces.
pub mod placement;
/// Low-level roving focus utilities used by composite widgets.
pub mod roving_focus;
/// Headless state model for tabs.
pub mod tabs;
/// Low-level typeahead matching helper.
pub mod typeahead;

pub use collapsible::{CollapsibleModel, CollapsibleState};
pub use dialog::{DialogModel, DialogState};
pub use menu::{MenuLoop, MenuModel, MenuState};
pub use orientation::{Direction, Orientation};
pub use placement::{
    Placement, PlacementAlign, PlacementOptions, PlacementRect, PlacementSide, PlacementSize,
    place_layer,
};
pub use tabs::{TabsActivation, TabsLoop, TabsModel};
pub use typeahead::{Typeahead, TypeaheadKeyResult};

#[cfg(test)]
extern crate std;
