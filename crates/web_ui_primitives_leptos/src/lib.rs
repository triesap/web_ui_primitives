//! Leptos bindings for `web_ui_primitives`.
//!
//! The crate exposes two layers:
//!
//! - `attrs`: pure helpers that derive DOM attributes from core state models.
//! - behavioral primitives such as [`FocusScope`], [`DismissibleLayer`],
//!   [`Portal`], [`Presence`], and [`ModalGuard`].
//!
//! The DOM binding surface uses `Dom*` and `Dismissible*` terminology.
//!
//! ```rust,no_run
//! use web_ui_primitives_core::collapsible::CollapsibleModel;
//! use web_ui_primitives_leptos::{attrs::collapsible_trigger_attrs, use_dom_bindings};
//! use leptos::html;
//! use leptos::prelude::*;
//!
//! let model = RwSignal::new(CollapsibleModel::new(false));
//! let attrs = Signal::derive(move || collapsible_trigger_attrs(&model.get(), Some("details")));
//! let bindings = use_dom_bindings::<html::Button>(attrs, vec![]);
//!
//! let _view = view! {
//!     <button node_ref=bindings.node_ref()>
//!         "Toggle"
//!     </button>
//! };
//! ```
//!
#![forbid(unsafe_code)]

pub mod attrs;
mod dismissible;
mod dom_bindings;
mod focus;
mod modal;
mod portal;
mod presence;
mod scroll_lock;

pub use dismissible::{
    DismissibleLayer, DismissibleLayerBinding, DismissibleLayerOptions, DismissibleReason,
    dismissible_is_escape, dismissible_is_outside, use_dismissible_layer,
};
pub use dom_bindings::{
    BoundElement, DomAttribute, DomAttributeValue, DomBindingError, DomBindingResult,
    DomEventBinding, DomEventHandler, DomTarget, apply_dom_attribute_delta, use_dom_bindings,
};
pub use focus::{
    FocusScope, FocusScopeBinding, FocusScopeOptions, focus_scope_next_index, focus_scope_selector,
    use_focus_scope,
};
pub use modal::{
    ModalError, ModalGuard, ModalResult, ModalTarget, modal_hide_siblings, modal_restore,
};
pub use portal::{Portal, PortalMount};
pub use presence::{Presence, PresenceBinding, PresenceState, presence_state_next, use_presence};
pub use scroll_lock::{
    ScrollLockError, ScrollLockGuard, ScrollLockResult, scroll_lock_acquire, scroll_lock_release,
};
