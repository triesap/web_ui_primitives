#![forbid(unsafe_code)]

mod dom_bindings;
pub mod builders;
mod dismissible;
mod focus;
mod modal;
mod portal;
mod presence;
mod scroll_lock;

pub use dom_bindings::{
    BoundElement, DomAttribute, DomAttributeValue, DomBindingError, DomBindingResult,
    DomEventBinding, DomEventHandler, DomTarget, apply_dom_attribute_delta, use_dom_bindings,
};
pub use dismissible::{
    DismissibleLayer, DismissibleReason, dismissible_is_escape, dismissible_is_outside,
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

#[deprecated(note = "use DomAttribute instead")]
pub type PrimitiveAttribute = DomAttribute;
#[deprecated(note = "use DomAttributeValue instead")]
pub type PrimitiveAttributeValue = DomAttributeValue;
#[deprecated(note = "use BoundElement instead")]
pub type PrimitiveElement<E> = BoundElement<E>;
#[deprecated(note = "use DomBindingError instead")]
pub type PrimitiveError = DomBindingError;
#[deprecated(note = "use DomBindingResult instead")]
pub type PrimitiveResult<T> = DomBindingResult<T>;
#[deprecated(note = "use DomEventBinding instead")]
pub type PrimitiveEvent = DomEventBinding;
#[deprecated(note = "use DomEventHandler instead")]
pub type PrimitiveEventHandler = DomEventHandler;
#[deprecated(note = "use DomTarget instead")]
pub type PrimitiveTarget = DomTarget;
#[deprecated(note = "use DismissibleReason instead")]
pub type DismissableReason = DismissibleReason;

#[deprecated(note = "use apply_dom_attribute_delta instead")]
pub use dom_bindings::apply_dom_attribute_delta as apply_attribute_delta;
#[deprecated(note = "use use_dom_bindings instead")]
pub use dom_bindings::use_dom_bindings as use_primitive;
#[deprecated(note = "use DismissibleLayer instead")]
pub use dismissible::DismissibleLayer as DismissableLayer;
#[deprecated(note = "use dismissible_is_escape instead")]
pub use dismissible::dismissible_is_escape as dismissable_is_escape;
#[deprecated(note = "use dismissible_is_outside instead")]
pub use dismissible::dismissible_is_outside as dismissable_is_outside;
