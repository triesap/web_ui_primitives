//! Attribute helpers for headless widget state models.
//!
//! These helpers are pure functions: they derive [`crate::DomAttribute`] values
//! from core state without attaching listeners or mutating the DOM.

pub mod collapsible;
pub mod dialog;
pub mod tabs;

pub use collapsible::{collapsible_content_attrs, collapsible_trigger_attrs};
pub use dialog::{dialog_content_attrs, dialog_trigger_attrs};
pub use tabs::{tabs_list_attrs, tabs_panel_attrs, tabs_trigger_attrs};
