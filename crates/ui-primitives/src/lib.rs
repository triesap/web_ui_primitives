#![forbid(unsafe_code)]

#[cfg(feature = "core")]
pub use ui_primitives_core as core;

#[cfg(feature = "leptos")]
pub use ui_primitives_leptos as leptos;
