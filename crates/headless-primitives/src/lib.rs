//! Umbrella crate for the `headless-primitives` family.
//!
//! # Features
//!
//! - `core` (default): re-exports [`headless_primitives_core`] as [`core`].
//! - `leptos`: re-exports [`headless_primitives_leptos`] as [`leptos`].
//!
//! Enable the `leptos` feature when you want to consume the Leptos bindings from
//! this crate:
//!
//! ```toml
//! [dependencies]
//! headless-primitives = { version = "0.1.0", features = ["leptos"] }
//! ```
//!
//! Applications can also depend on `headless-primitives-core` and
//! `headless-primitives-leptos` directly when they want tighter feature control.
//!
#![forbid(unsafe_code)]

#[cfg(feature = "core")]
pub use headless_primitives_core as core;

#[cfg(feature = "leptos")]
pub use headless_primitives_leptos as leptos;
