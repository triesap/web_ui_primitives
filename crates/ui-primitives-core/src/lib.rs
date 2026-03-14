//! Compatibility shim for the renamed `headless-primitives-core` crate.
//!
//! Prefer depending on `headless-primitives-core` directly for new work.
//!
#![no_std]
#![forbid(unsafe_code)]

pub use headless_primitives_core::*;
