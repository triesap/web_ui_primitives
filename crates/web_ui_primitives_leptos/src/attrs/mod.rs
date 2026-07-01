//! Attribute helpers for headless widget state models.
//!
//! These helpers are pure functions: they derive [`crate::DomAttribute`] values
//! from core state without attaching listeners or mutating the DOM.

pub mod collapsible;
pub mod dialog;
pub mod menu;
pub mod tabs;

pub use collapsible::{collapsible_content_attrs, collapsible_trigger_attrs};
pub use dialog::{
    DialogContentAttrs, DialogName, DialogNameError, DialogRole, dialog_content_attrs,
    dialog_trigger_attrs,
};
pub use menu::{
    MenuContentAttrs, MenuItemAttrs, MenuItemDisabledPolicy, MenuItemElement, MenuItemKind,
    MenuTriggerAttrs, MenuTriggerElement, menu_content_attrs, menu_item_attrs,
    menu_item_indicator_attrs, menu_trigger_attrs,
};
pub use tabs::{
    TabsTriggerAttrs, TabsTriggerDisabledPolicy, TabsTriggerElement, tabs_list_attrs,
    tabs_panel_attrs, tabs_trigger_attrs,
};
