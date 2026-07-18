use alloc::vec::Vec;

use crate::orientation::Direction;
use crate::roving_focus::{
    RovingFocus, RovingFocusAction, RovingFocusOrientation, roving_focus_action_from_key,
};
use crate::typeahead::{Typeahead, TypeaheadKeyResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuLoop {
    Wrap,
    Clamp,
}

impl MenuLoop {
    fn as_looped(self) -> bool {
        match self {
            Self::Wrap => true,
            Self::Clamp => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuModel {
    open: bool,
    focus: RovingFocus,
    disabled: Vec<bool>,
    checked: Option<usize>,
    typeahead: Typeahead,
}

impl MenuModel {
    pub fn new(len: usize) -> Self {
        Self::with_loop_and_disabled(len, MenuLoop::Wrap, [])
    }

    pub fn with_loop(len: usize, loop_policy: MenuLoop) -> Self {
        Self::with_loop_and_disabled(len, loop_policy, [])
    }

    pub fn with_checked(len: usize, checked: Option<usize>) -> Self {
        let mut model = Self::new(len);
        model.set_checked(checked);
        model
    }

    pub fn with_loop_and_disabled<I>(len: usize, loop_policy: MenuLoop, disabled: I) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let disabled = normalize_disabled(len, disabled);
        Self {
            open: false,
            focus: RovingFocus::with_active(len, None, loop_policy.as_looped()),
            disabled,
            checked: None,
            typeahead: Typeahead::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.focus.len()
    }

    pub fn is_empty(&self) -> bool {
        self.focus.is_empty()
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn state(&self) -> MenuState {
        if self.open {
            MenuState::Open
        } else {
            MenuState::Closed
        }
    }

    pub fn focused(&self) -> Option<usize> {
        self.focus.active()
    }

    pub fn checked(&self) -> Option<usize> {
        self.checked
    }

    pub fn item_checked(&self, index: usize) -> bool {
        self.checked == Some(index)
    }

    pub fn disabled(&self, index: usize) -> bool {
        self.disabled.get(index).copied().unwrap_or(false)
    }

    pub fn loop_policy(&self) -> MenuLoop {
        if self.focus.looped() {
            MenuLoop::Wrap
        } else {
            MenuLoop::Clamp
        }
    }

    pub fn typeahead_query(&self) -> &str {
        self.typeahead.query()
    }

    pub fn set_open(&mut self, open: bool) -> bool {
        let changed = self.open != open;
        self.open = open;
        if open {
            self.repair_focus(self.checked.or(self.focus.active()));
        } else {
            self.focus.set_active(None);
            self.typeahead.clear();
        }
        changed
    }

    pub fn toggle(&mut self) -> bool {
        self.set_open(!self.open)
    }

    pub fn set_len(&mut self, len: usize) {
        self.focus.set_len(len);
        self.disabled.resize(len, false);
        self.repair_checked();
        if self.open {
            self.repair_focus(self.focus.active());
        } else {
            self.focus.set_active(None);
        }
    }

    pub fn set_loop_policy(&mut self, loop_policy: MenuLoop) {
        self.focus.set_looped(loop_policy.as_looped());
    }

    pub fn set_checked(&mut self, checked: Option<usize>) -> Option<usize> {
        self.checked = checked.filter(|index| self.is_enabled_index(*index));
        if self.open && self.focused().is_none() {
            self.repair_focus(self.checked);
        }
        self.checked
    }

    pub fn set_disabled(&mut self, index: usize, disabled: bool) {
        if let Some(current) = self.disabled.get_mut(index) {
            *current = disabled;
            self.repair_checked();
            if self.open {
                self.repair_focus(self.focus.active());
            }
        }
    }

    pub fn set_disabled_all<I>(&mut self, disabled: I)
    where
        I: IntoIterator<Item = bool>,
    {
        self.disabled = normalize_disabled(self.len(), disabled);
        self.repair_checked();
        if self.open {
            self.repair_focus(self.focus.active());
        }
    }

    pub fn focus_index(&mut self, index: Option<usize>) -> Option<usize> {
        if !self.open {
            return None;
        }

        match index {
            Some(index) if self.is_enabled_index(index) => self.focus.set_active(Some(index)),
            Some(_) => self.focus.active(),
            None => self.focus.set_active(None),
        }
    }

    pub fn focus_next(&mut self) -> Option<usize> {
        self.move_focus(true)
    }

    pub fn focus_prev(&mut self) -> Option<usize> {
        self.move_focus(false)
    }

    pub fn focus_first(&mut self) -> Option<usize> {
        if !self.open {
            return None;
        }
        self.focus.set_active(first_enabled_index(&self.disabled))
    }

    pub fn focus_last(&mut self) -> Option<usize> {
        if !self.open {
            return None;
        }
        self.focus.set_active(last_enabled_index(&self.disabled))
    }

    pub fn focus_by_key(&mut self, key: &str, direction: Direction) -> Option<usize> {
        if !self.open {
            return None;
        }

        let action =
            roving_focus_action_from_key(key, RovingFocusOrientation::Vertical, direction)?;
        self.focus_action(action)
    }

    pub fn close_by_key(&mut self, key: &str) -> bool {
        if key == "Escape" {
            self.set_open(false)
        } else {
            false
        }
    }

    pub fn activate_focused(&mut self) -> Option<usize> {
        let index = self.focus.active()?;
        self.activate_index(index)
    }

    pub fn activate_index(&mut self, index: usize) -> Option<usize> {
        if !self.open || !self.is_enabled_index(index) {
            return None;
        }

        self.focus.set_active(Some(index));
        self.set_open(false);
        Some(index)
    }

    pub fn typeahead_by_key<T>(
        &mut self,
        key: &str,
        now_ms: u64,
        items: &[T],
        label: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        if !self.open || self.typeahead.input(key, now_ms) == TypeaheadKeyResult::Ignored {
            return None;
        }

        let count = items.len().min(self.len());
        let index = self.typeahead.match_index(
            &items[..count],
            self.focus.active(),
            |index, _| self.disabled(index),
            label,
        )?;
        self.focus.set_active(Some(index))
    }

    fn focus_action(&mut self, action: RovingFocusAction) -> Option<usize> {
        match action {
            RovingFocusAction::Next => self.focus_next(),
            RovingFocusAction::Prev => self.focus_prev(),
            RovingFocusAction::First => self.focus_first(),
            RovingFocusAction::Last => self.focus_last(),
        }
    }

    fn move_focus(&mut self, forward: bool) -> Option<usize> {
        if !self.open || first_enabled_index(&self.disabled).is_none() {
            return None;
        }

        let next = self.next_enabled_index(self.focus.active(), forward);
        if let Some(index) = next {
            self.focus.set_active(Some(index));
        }
        next
    }

    fn next_enabled_index(&self, current: Option<usize>, forward: bool) -> Option<usize> {
        match current {
            None => {
                if forward {
                    first_enabled_index(&self.disabled)
                } else {
                    last_enabled_index(&self.disabled)
                }
            }
            Some(current) if self.focus.looped() => {
                for step in 1..=self.len() {
                    let index = if forward {
                        (current + step) % self.len()
                    } else {
                        (current + self.len() - (step % self.len())) % self.len()
                    };
                    if self.is_enabled_index(index) {
                        return Some(index);
                    }
                }
                None
            }
            Some(current) if forward => {
                (current + 1..self.len()).find(|&index| self.is_enabled_index(index))
            }
            Some(current) => (0..current)
                .rev()
                .find(|&index| self.is_enabled_index(index)),
        }
    }

    fn repair_focus(&mut self, anchor: Option<usize>) {
        let anchor = self
            .focus
            .active()
            .filter(|index| self.is_enabled_index(*index))
            .or_else(|| self.checked.filter(|index| self.is_enabled_index(*index)))
            .filter(|index| self.is_enabled_index(*index))
            .or_else(|| anchor.filter(|index| self.is_enabled_index(*index)))
            .or_else(|| self.fallback_enabled_index(anchor));
        self.focus.set_active(anchor);
    }

    fn repair_checked(&mut self) {
        if self
            .checked
            .is_some_and(|index| !self.is_enabled_index(index))
        {
            self.checked = None;
        }
    }

    fn fallback_enabled_index(&self, anchor: Option<usize>) -> Option<usize> {
        let Some(anchor) = clamp_index(self.len(), anchor) else {
            return first_enabled_index(&self.disabled);
        };

        for index in anchor + 1..self.len() {
            if self.is_enabled_index(index) {
                return Some(index);
            }
        }
        for index in (0..anchor).rev() {
            if self.is_enabled_index(index) {
                return Some(index);
            }
        }
        if self.is_enabled_index(anchor) {
            Some(anchor)
        } else {
            None
        }
    }

    fn is_enabled_index(&self, index: usize) -> bool {
        index < self.len() && !self.disabled(index)
    }
}

fn clamp_index(len: usize, index: Option<usize>) -> Option<usize> {
    match index {
        Some(index) => {
            if len == 0 {
                None
            } else if index < len {
                Some(index)
            } else {
                Some(len - 1)
            }
        }
        None => None,
    }
}

fn normalize_disabled<I>(len: usize, disabled: I) -> Vec<bool>
where
    I: IntoIterator<Item = bool>,
{
    let mut values: Vec<bool> = disabled.into_iter().take(len).collect();
    values.resize(len, false);
    values
}

fn first_enabled_index(disabled: &[bool]) -> Option<usize> {
    disabled.iter().position(|disabled| !*disabled)
}

fn last_enabled_index(disabled: &[bool]) -> Option<usize> {
    disabled.iter().rposition(|disabled| !*disabled)
}

#[cfg(test)]
mod tests {
    use crate::orientation::Direction;

    use super::{MenuLoop, MenuModel, MenuState};

    #[test]
    fn menu_opens_with_first_enabled_item() {
        let mut model = MenuModel::with_loop_and_disabled(3, MenuLoop::Wrap, [true, false, false]);
        assert_eq!(model.state(), MenuState::Closed);
        assert!(model.set_open(true));
        assert_eq!(model.state(), MenuState::Open);
        assert_eq!(model.focused(), Some(1));
    }

    #[test]
    fn menu_prefers_checked_item_when_opening() {
        let mut model = MenuModel::with_checked(3, Some(2));
        model.set_open(true);
        assert_eq!(model.checked(), Some(2));
        assert_eq!(model.focused(), Some(2));
    }

    #[test]
    fn menu_closes_and_clears_focus_on_escape() {
        let mut model = MenuModel::new(2);
        model.set_open(true);
        assert_eq!(model.focused(), Some(0));
        assert!(model.close_by_key("Escape"));
        assert_eq!(model.state(), MenuState::Closed);
        assert_eq!(model.focused(), None);
        assert!(!model.close_by_key("Enter"));
    }

    #[test]
    fn menu_arrow_navigation_skips_disabled_items() {
        let mut model = MenuModel::with_loop_and_disabled(4, MenuLoop::Wrap, [false, true, true]);
        model.set_open(true);
        assert_eq!(model.focus_next(), Some(3));
        assert_eq!(model.focus_next(), Some(0));
        assert_eq!(model.focus_by_key("ArrowDown", Direction::Ltr), Some(3));
        assert_eq!(model.focus_by_key("Home", Direction::Ltr), Some(0));
        assert_eq!(model.focus_by_key("End", Direction::Ltr), Some(3));
    }

    #[test]
    fn menu_clamp_policy_stops_at_edges() {
        let mut model = MenuModel::with_loop(2, MenuLoop::Clamp);
        model.set_open(true);
        assert_eq!(model.focus_prev(), None);
        assert_eq!(model.focused(), Some(0));
        assert_eq!(model.focus_next(), Some(1));
        assert_eq!(model.focus_next(), None);
        assert_eq!(model.focused(), Some(1));
    }

    #[test]
    fn menu_activation_closes_enabled_item() {
        let mut model = MenuModel::new(2);
        model.set_open(true);
        model.focus_next();
        assert_eq!(model.activate_focused(), Some(1));
        assert_eq!(model.state(), MenuState::Closed);
        assert_eq!(model.focused(), None);
    }

    #[test]
    fn menu_does_not_activate_disabled_items() {
        let mut model = MenuModel::with_loop_and_disabled(2, MenuLoop::Wrap, [false, true]);
        model.set_open(true);
        assert_eq!(model.activate_index(1), None);
        assert_eq!(model.state(), MenuState::Open);
        assert_eq!(model.focused(), Some(0));
    }

    #[test]
    fn menu_clears_checked_when_item_becomes_disabled() {
        let mut model = MenuModel::with_checked(3, Some(1));
        assert_eq!(model.checked(), Some(1));
        model.set_disabled(1, true);
        assert_eq!(model.checked(), None);
    }

    #[test]
    fn menu_preserves_valid_focus_when_other_item_becomes_disabled() {
        let mut model = MenuModel::with_checked(3, Some(2));
        model.set_open(true);
        model.focus_index(Some(0));
        model.set_disabled(2, true);
        assert_eq!(model.checked(), None);
        assert_eq!(model.focused(), Some(0));
    }

    #[test]
    fn menu_typeahead_focuses_matching_enabled_item() {
        let mut model = MenuModel::with_loop_and_disabled(4, MenuLoop::Wrap, [false, true, false]);
        let items = ["Apple", "Banana", "Blueberry", "Carrot"];
        model.set_open(true);
        assert_eq!(model.typeahead_by_key("b", 0, &items, |item| item), Some(2));
        assert_eq!(model.focused(), Some(2));
        assert_eq!(model.typeahead_by_key("c", 500, &items, |item| item), None);
        assert_eq!(
            model.typeahead_by_key("c", 1_501, &items, |item| item),
            Some(3)
        );
    }
}
