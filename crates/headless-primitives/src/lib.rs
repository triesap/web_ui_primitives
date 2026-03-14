#![forbid(unsafe_code)]

#[cfg(feature = "core")]
pub use headless_primitives_core as core;

#[cfg(feature = "leptos")]
pub use headless_primitives_leptos as leptos;
