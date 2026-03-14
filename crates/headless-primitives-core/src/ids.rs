//! Deterministic ID helpers for headless widgets.
//!
//! These helpers generate predictable IDs within a local scope. They do not
//! attempt to provide cross-process or cross-tree uniqueness.

use alloc::string::String;
use alloc::vec::Vec;

/// Incremental ID generator with a stable prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdGenerator {
    prefix: String,
    counter: u64,
}

impl IdGenerator {
    /// Creates a generator that prefixes all IDs with `prefix`.
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            counter: 0,
        }
    }

    /// Returns the current ID prefix.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Resets the numeric suffix back to zero.
    pub fn reset(&mut self) {
        self.counter = 0;
    }

    /// Returns the next generated ID.
    pub fn next_id(&mut self) -> String {
        self.counter = self.counter.saturating_add(1);
        alloc::format!("{}-{}", self.prefix, self.counter)
    }
}

/// Generates `count` sequential IDs from a shared prefix.
pub fn generate_ids(prefix: impl Into<String>, count: usize) -> Vec<String> {
    let mut generator = IdGenerator::new(prefix);
    let mut ids = Vec::with_capacity(count);
    for _ in 0..count {
        ids.push(generator.next_id());
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::{IdGenerator, generate_ids};

    #[test]
    fn id_generator_increments() {
        let mut generator = IdGenerator::new("tab");
        assert_eq!(generator.next_id(), "tab-1");
        assert_eq!(generator.next_id(), "tab-2");
    }

    #[test]
    fn generate_ids_returns_count() {
        let ids = generate_ids("item", 3);
        assert_eq!(ids.as_slice(), &["item-1", "item-2", "item-3"]);
    }
}
