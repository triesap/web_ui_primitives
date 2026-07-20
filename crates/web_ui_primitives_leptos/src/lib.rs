//! Leptos bindings for `web_ui_primitives`.
//!
//! The crate exposes two layers:
//!
//! - `attrs`: pure helpers that derive DOM attributes from core state models.
//! - behavioral primitives such as [`use_dialog_layer`], [`FocusScope`],
//!   [`DismissibleLayer`], [`Portal`], [`Presence`], and [`ModalGuard`].
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

#[cfg(any(
    all(feature = "csr", feature = "hydrate"),
    all(feature = "csr", feature = "ssr"),
    all(feature = "hydrate", feature = "ssr")
))]
compile_error!(
    "`csr`, `hydrate`, and `ssr` are mutually exclusive web_ui_primitives_leptos render modes"
);

pub mod attrs;
mod dialog;
mod dismissible;
mod dom_bindings;
mod focus;
mod menu;
mod modal;
mod portal;
mod presence;
mod scroll_lock;

pub use dialog::{
    DialogLayerBinding, DialogLayerError, DialogLayerOptions, use_dialog_layer,
    use_dialog_layer_with_node_ref,
};
pub use dismissible::{
    DismissibleBranch, DismissibleEscapeKeyDownEvent, DismissibleEvent,
    DismissibleFocusOutsideEvent, DismissibleLayer, DismissibleLayerBinding,
    DismissibleLayerOptions, DismissiblePointerDownOutsideEvent, DismissibleReason,
    dismissible_is_escape, dismissible_is_outside, use_dismissible_layer,
    use_dismissible_layer_with_node_ref,
};
pub use dom_bindings::{
    BoundElement, DomAttribute, DomAttributeValue, DomBindingError, DomBindingResult,
    DomEventBinding, DomEventHandler, DomTarget, apply_dom_attribute_delta, use_dom_bindings,
};
pub use focus::{
    FocusScope, FocusScopeBinding, FocusScopeOptions, focus_scope_next_index, focus_scope_selector,
    use_focus_scope, use_focus_scope_with_node_ref,
};
pub use menu::{
    MenuLayerBinding, MenuLayerOptions, MenuPlacementBinding, MenuPlacementOptions,
    placement_align_data_value, placement_side_data_value, use_menu_layer,
    use_menu_layer_with_node_ref, use_menu_placement_with_node_refs,
};
pub use modal::{
    ModalError, ModalGuard, ModalResult, ModalTarget, modal_hide_siblings, modal_restore,
};
pub use portal::{Portal, PortalMount};
pub use presence::use_presence_with_node_ref;
pub use presence::{
    PRESENCE_ABI_VERSION, Presence, PresenceBinding, PresenceState, presence_state_next,
    use_presence,
};
pub use scroll_lock::{
    ScrollLockError, ScrollLockGuard, ScrollLockResult, scroll_lock_acquire, scroll_lock_release,
};
