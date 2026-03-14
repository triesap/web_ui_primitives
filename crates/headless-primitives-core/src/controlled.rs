//! Generic controlled-value helpers for advanced state coordination.
//!
//! These types are intentionally lower-level than the widget models exposed by
//! this crate. They remain public for composition and backward compatibility.

use alloc::boxed::Box;
use core::mem;

/// Describes a value change observed by [`Controlled`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change<'a, T> {
    pub previous: &'a T,
    pub next: &'a T,
}

/// Change callback used by [`Controlled`].
pub type OnChange<T> = Box<dyn for<'a> FnMut(Change<'a, T>)>;

/// Stores a value and optionally reports transitions when it changes.
pub struct Controlled<T> {
    value: T,
    on_change: Option<OnChange<T>>,
}

impl<T> Controlled<T> {
    /// Creates a controlled value without a change callback.
    pub fn new(value: T) -> Self {
        Self {
            value,
            on_change: None,
        }
    }

    /// Creates a controlled value with an initial change callback.
    pub fn with_on_change(value: T, on_change: OnChange<T>) -> Self {
        Self {
            value,
            on_change: Some(on_change),
        }
    }

    /// Returns the current value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the wrapper and returns the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Replaces the registered change callback.
    pub fn set_on_change(&mut self, on_change: Option<OnChange<T>>) {
        self.on_change = on_change;
    }

    /// Sets a new value and notifies the callback when present.
    pub fn set(&mut self, next: T) {
        let previous = mem::replace(&mut self.value, next);
        if let Some(on_change) = self.on_change.as_mut() {
            let change = Change {
                previous: &previous,
                next: &self.value,
            };
            on_change(change);
        }
    }

    /// Sets a new value only when it differs from the current value.
    pub fn set_if_changed(&mut self, next: T) -> bool
    where
        T: PartialEq,
    {
        if self.value == next {
            return false;
        }
        self.set(next);
        true
    }

    /// Produces a new value from the current value and stores it.
    pub fn update(&mut self, f: impl FnOnce(&T) -> T) {
        let next = f(&self.value);
        self.set(next);
    }

    /// Updates the value only when the produced value differs.
    pub fn update_if_changed(&mut self, f: impl FnOnce(&T) -> T) -> bool
    where
        T: PartialEq,
    {
        let next = f(&self.value);
        self.set_if_changed(next)
    }
}

#[cfg(test)]
mod tests {
    use super::{Change, Controlled};
    use std::boxed::Box;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::vec::Vec;

    #[test]
    fn controlled_set_calls_on_change() {
        let changes: Rc<RefCell<Vec<(i32, i32)>>> = Rc::new(RefCell::new(Vec::new()));
        let changes_handle = Rc::clone(&changes);
        let mut controlled = Controlled::with_on_change(
            1,
            Box::new(move |change: Change<'_, i32>| {
                changes_handle
                    .borrow_mut()
                    .push((*change.previous, *change.next));
            }),
        );

        controlled.set(2);

        let recorded = changes.borrow();
        assert_eq!(recorded.as_slice(), &[(1, 2)]);
    }

    #[test]
    fn controlled_set_if_changed_skips_equal() {
        let changes: Rc<RefCell<Vec<(i32, i32)>>> = Rc::new(RefCell::new(Vec::new()));
        let changes_handle = Rc::clone(&changes);
        let mut controlled = Controlled::with_on_change(
            1,
            Box::new(move |change: Change<'_, i32>| {
                changes_handle
                    .borrow_mut()
                    .push((*change.previous, *change.next));
            }),
        );

        assert!(!controlled.set_if_changed(1));
        assert!(controlled.set_if_changed(3));

        let recorded = changes.borrow();
        assert_eq!(recorded.as_slice(), &[(1, 3)]);
    }

    #[test]
    fn controlled_update() {
        let mut controlled = Controlled::new(10);
        controlled.update(|value| value + 5);
        assert_eq!(*controlled.value(), 15);
    }
}
