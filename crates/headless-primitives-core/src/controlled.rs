use alloc::boxed::Box;
use core::mem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change<'a, T> {
    pub previous: &'a T,
    pub next: &'a T,
}

pub type OnChange<T> = Box<dyn for<'a> FnMut(Change<'a, T>)>;

pub struct Controlled<T> {
    value: T,
    on_change: Option<OnChange<T>>,
}

impl<T> Controlled<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            on_change: None,
        }
    }

    pub fn with_on_change(value: T, on_change: OnChange<T>) -> Self {
        Self {
            value,
            on_change: Some(on_change),
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub fn set_on_change(&mut self, on_change: Option<OnChange<T>>) {
        self.on_change = on_change;
    }

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

    pub fn update(&mut self, f: impl FnOnce(&T) -> T) {
        let next = f(&self.value);
        self.set(next);
    }

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
    use std::cell::RefCell;
    use std::boxed::Box;
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
