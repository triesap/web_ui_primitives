#![forbid(unsafe_code)]

mod attach;
mod dismissable;
mod focus;
mod modal;
pub mod builders;

pub use attach::{
    apply_attribute_delta,
    PrimitiveAttribute,
    PrimitiveAttributeValue,
    PrimitiveElement,
    PrimitiveError,
    PrimitiveEvent,
    PrimitiveResult,
    use_primitive,
};
pub use dismissable::{
    dismissable_is_escape,
    dismissable_is_outside,
    DismissableLayer,
    DismissableReason,
};
pub use focus::{
    focus_scope_next_index,
    focus_scope_selector,
    FocusScope,
};
pub use modal::{
    modal_hide_siblings,
    modal_restore,
    ModalError,
    ModalGuard,
    ModalResult,
    ModalTarget,
};
