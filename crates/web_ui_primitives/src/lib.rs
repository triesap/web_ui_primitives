//! Umbrella crate for the `web_ui_primitives` family.
//!
//! # Features
//!
//! - `core` (default): re-exports [`web_ui_primitives_core`] as [`core`].
//! - `leptos`: re-exports [`web_ui_primitives_leptos`] as [`leptos`].
//!
//! Enable the `leptos` feature when you want to consume the Leptos bindings from
//! this crate:
//!
//! ```toml
//! [dependencies]
//! web_ui_primitives = { version = "0.1.0", features = ["leptos"] }
//! ```
//!
//! Applications can also depend on `web_ui_primitives_core` and
//! `web_ui_primitives_leptos` directly when they want tighter feature control.
//!
#![forbid(unsafe_code)]

#[cfg(feature = "core")]
pub use web_ui_primitives_core as core;

#[cfg(feature = "leptos")]
pub use web_ui_primitives_leptos as leptos;
