#![forbid(unsafe_code)]

mod attach;
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
