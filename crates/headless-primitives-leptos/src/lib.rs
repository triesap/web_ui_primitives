//! Leptos bindings for `headless-primitives`.
//!
//! The crate exposes two layers:
//!
//! - `builders`: pure helpers that derive DOM attributes from core state models.
//! - behavioral primitives such as [`FocusScope`], [`DismissibleLayer`],
//!   [`Portal`], [`Presence`], and [`ModalGuard`].
//!
//! The DOM binding surface now uses `Dom*` and `Dismissible*` terminology.
//! Deprecated `Primitive*`, `use_primitive`, and `Dismissable*` aliases remain
//! available for migration.
//!
//! ```rust,no_run
//! use headless_primitives_core::collapsible::CollapsibleModel;
//! use headless_primitives_leptos::{builders::collapsible_trigger_attrs, use_dom_bindings};
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

pub mod builders;
mod dismissible;
mod dom_bindings;
mod focus;
mod modal;
mod portal;
mod presence;
mod scroll_lock;

pub use dismissible::{
    DismissibleLayer, DismissibleReason, dismissible_is_escape, dismissible_is_outside,
};
pub use dom_bindings::{
    BoundElement, DomAttribute, DomAttributeValue, DomBindingError, DomBindingResult,
    DomEventBinding, DomEventHandler, DomTarget, apply_dom_attribute_delta, use_dom_bindings,
};
pub use focus::{FocusScope, focus_scope_next_index, focus_scope_selector};
pub use modal::{
    ModalError, ModalGuard, ModalResult, ModalTarget, modal_hide_siblings, modal_restore,
};
pub use portal::{Portal, PortalMount};
pub use presence::{Presence, PresenceState, presence_state_next};
pub use scroll_lock::{
    ScrollLockError, ScrollLockGuard, ScrollLockResult, scroll_lock_acquire, scroll_lock_release,
};

/// Deprecated alias for [`DomAttribute`].
#[deprecated(note = "use DomAttribute instead")]
pub type PrimitiveAttribute = DomAttribute;
/// Deprecated alias for [`DomAttributeValue`].
#[deprecated(note = "use DomAttributeValue instead")]
pub type PrimitiveAttributeValue = DomAttributeValue;
/// Deprecated alias for [`BoundElement`].
#[deprecated(note = "use BoundElement instead")]
pub type PrimitiveElement<E> = BoundElement<E>;
/// Deprecated alias for [`DomBindingError`].
#[deprecated(note = "use DomBindingError instead")]
pub type PrimitiveError = DomBindingError;
/// Deprecated alias for [`DomBindingResult`].
#[deprecated(note = "use DomBindingResult instead")]
pub type PrimitiveResult<T> = DomBindingResult<T>;
/// Deprecated alias for [`DomEventBinding`].
#[deprecated(note = "use DomEventBinding instead")]
pub type PrimitiveEvent = DomEventBinding;
/// Deprecated alias for [`DomEventHandler`].
#[deprecated(note = "use DomEventHandler instead")]
pub type PrimitiveEventHandler = DomEventHandler;
/// Deprecated alias for [`DomTarget`].
#[deprecated(note = "use DomTarget instead")]
pub type PrimitiveTarget = DomTarget;
/// Deprecated alias for [`DismissibleReason`].
#[deprecated(note = "use DismissibleReason instead")]
pub type DismissableReason = DismissibleReason;

/// Deprecated alias for [`DismissibleLayer`].
#[deprecated(note = "use DismissibleLayer instead")]
pub use dismissible::DismissibleLayer as DismissableLayer;
/// Deprecated alias for [`dismissible_is_escape`].
#[deprecated(note = "use dismissible_is_escape instead")]
pub use dismissible::dismissible_is_escape as dismissable_is_escape;
/// Deprecated alias for [`dismissible_is_outside`].
#[deprecated(note = "use dismissible_is_outside instead")]
pub use dismissible::dismissible_is_outside as dismissable_is_outside;
/// Deprecated alias for [`apply_dom_attribute_delta`].
#[deprecated(note = "use apply_dom_attribute_delta instead")]
pub use dom_bindings::apply_dom_attribute_delta as apply_attribute_delta;
/// Deprecated alias for [`use_dom_bindings`].
#[deprecated(note = "use use_dom_bindings instead")]
pub use dom_bindings::use_dom_bindings as use_primitive;
