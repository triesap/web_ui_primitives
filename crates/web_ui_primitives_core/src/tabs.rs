use alloc::vec::Vec;

use crate::roving_focus::RovingFocus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsActivation {
    Automatic,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsModel {
    focus: RovingFocus,
    selected: Option<usize>,
    activation: TabsActivation,
    disabled: Vec<bool>,
}

impl TabsModel {
    pub fn new(len: usize) -> Self {
        Self::with_activation(len, TabsActivation::Automatic)
    }

    pub fn with_activation(len: usize, activation: TabsActivation) -> Self {
        Self::with_activation_and_disabled(len, activation, [])
    }

    pub fn with_activation_and_disabled<I>(
        len: usize,
        activation: TabsActivation,
        disabled: I,
    ) -> Self
    where
        I: IntoIterator<Item = bool>,
    {
        let disabled = normalize_disabled(len, disabled);
        let initial = first_enabled_index(&disabled);
        let focus = RovingFocus::with_active(len, initial, true);
        Self {
            focus,
            selected: initial,
            activation,
            disabled,
        }
    }

    pub fn len(&self) -> usize {
        self.focus.len()
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn focused(&self) -> Option<usize> {
        self.focus.active()
    }

    pub fn activation(&self) -> TabsActivation {
        self.activation
    }

    pub fn set_activation(&mut self, activation: TabsActivation) {
        self.activation = activation;
        if activation == TabsActivation::Automatic {
            self.sync_automatic_activation();
        }
    }

    pub fn set_len(&mut self, len: usize) {
        self.focus.set_len(len);
        self.disabled.resize(len, false);
        self.repair_state(None);
    }

    pub fn disabled(&self, index: usize) -> bool {
        self.disabled.get(index).copied().unwrap_or(false)
    }

    pub fn set_disabled(&mut self, index: usize, disabled: bool) {
        if let Some(current) = self.disabled.get_mut(index) {
            *current = disabled;
            self.repair_state(Some(index));
        }
    }

    pub fn set_disabled_all<I>(&mut self, disabled: I)
    where
        I: IntoIterator<Item = bool>,
    {
        self.disabled = normalize_disabled(self.len(), disabled);
        self.repair_state(None);
    }

    pub fn focus_index(&mut self, index: Option<usize>) -> Option<usize> {
        let focused = match index {
            Some(index) if self.is_enabled_index(index) => self.focus.set_active(Some(index)),
            Some(_) => self.focus.active(),
            None => self.focus.set_active(None),
        };
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn focus_next(&mut self) -> Option<usize> {
        let focused = self.move_focus(true);
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn focus_prev(&mut self) -> Option<usize> {
        let focused = self.move_focus(false);
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn focus_first(&mut self) -> Option<usize> {
        let focused = self.focus.set_active(first_enabled_index(&self.disabled));
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn focus_last(&mut self) -> Option<usize> {
        let focused = self.focus.set_active(last_enabled_index(&self.disabled));
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn select(&mut self, index: Option<usize>) -> Option<usize> {
        self.selected = match index {
            Some(index) if self.is_enabled_index(index) => Some(index),
            Some(_) => self.selected,
            None => None,
        };
        if self.activation == TabsActivation::Automatic {
            self.focus.set_active(self.selected);
        }
        self.selected
    }

    pub fn activate_focused(&mut self) -> Option<usize> {
        if self.activation == TabsActivation::Manual
            && self
                .focus
                .active()
                .is_some_and(|index| self.is_enabled_index(index))
        {
            self.selected = self.focus.active();
        }
        self.selected
    }

    fn sync_automatic_activation(&mut self) {
        if self
            .selected
            .is_some_and(|index| self.is_enabled_index(index))
        {
            self.focus.set_active(self.selected);
        } else if self
            .focus
            .active()
            .is_some_and(|index| self.is_enabled_index(index))
        {
            self.selected = self.focus.active();
        } else {
            let first_enabled = first_enabled_index(&self.disabled);
            self.focus.set_active(first_enabled);
            self.selected = first_enabled;
        }
    }

    fn repair_state(&mut self, anchor: Option<usize>) {
        let focused = if self
            .focus
            .active()
            .is_some_and(|index| self.is_enabled_index(index))
        {
            self.focus.active()
        } else {
            self.focus
                .set_active(self.fallback_enabled_index(anchor.or(self.focus.active())))
        };

        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
            self.focus.set_active(focused);
            return;
        }

        self.selected = if self
            .selected
            .is_some_and(|index| self.is_enabled_index(index))
        {
            self.selected
        } else {
            focused.or_else(|| first_enabled_index(&self.disabled))
        };
    }

    fn move_focus(&mut self, forward: bool) -> Option<usize> {
        let next = self.next_enabled_index(self.focus.active(), forward);
        self.focus.set_active(next);
        next
    }

    fn next_enabled_index(&self, current: Option<usize>, forward: bool) -> Option<usize> {
        if self.len() == 0 || first_enabled_index(&self.disabled).is_none() {
            return None;
        }

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
    use super::{TabsActivation, TabsModel};

    #[test]
    fn tabs_auto_activation_tracks_focus() {
        let mut model = TabsModel::new(3);
        assert_eq!(model.selected(), Some(0));
        model.focus_next();
        assert_eq!(model.selected(), Some(1));
    }

    #[test]
    fn tabs_manual_activation_requires_activate() {
        let mut model = TabsModel::with_activation(3, TabsActivation::Manual);
        model.focus_next();
        assert_eq!(model.selected(), Some(0));
        model.activate_focused();
        assert_eq!(model.selected(), Some(1));
    }

    #[test]
    fn tabs_clamps_selection_on_len_change() {
        let mut model = TabsModel::new(3);
        model.select(Some(2));
        model.set_len(1);
        assert_eq!(model.selected(), Some(0));
    }

    #[test]
    fn tabs_initial_state_skips_disabled_tabs() {
        let model = TabsModel::with_activation_and_disabled(
            3,
            TabsActivation::Automatic,
            [true, false, false],
        );
        assert_eq!(model.focused(), Some(1));
        assert_eq!(model.selected(), Some(1));
    }

    #[test]
    fn tabs_skip_disabled_tabs_when_moving_focus() {
        let mut model = TabsModel::with_activation_and_disabled(
            4,
            TabsActivation::Manual,
            [false, true, true, false],
        );
        assert_eq!(model.focus_next(), Some(3));
        assert_eq!(model.focus_prev(), Some(0));
    }

    #[test]
    fn tabs_cannot_focus_or_select_disabled_tabs() {
        let mut model = TabsModel::with_activation_and_disabled(
            3,
            TabsActivation::Manual,
            [false, true, false],
        );
        assert_eq!(model.focus_index(Some(1)), Some(0));
        assert_eq!(model.select(Some(1)), Some(0));
        assert_eq!(model.focused(), Some(0));
        assert_eq!(model.selected(), Some(0));
    }

    #[test]
    fn tabs_repair_focus_and_selection_when_current_tab_becomes_disabled() {
        let mut model = TabsModel::new(3);
        model.select(Some(1));
        model.set_disabled(1, true);
        assert_eq!(model.focused(), Some(2));
        assert_eq!(model.selected(), Some(2));
    }

    #[test]
    fn tabs_clear_focus_and_selection_when_all_tabs_are_disabled() {
        let mut model =
            TabsModel::with_activation_and_disabled(2, TabsActivation::Automatic, [false, false]);
        model.set_disabled_all([true, true]);
        assert_eq!(model.focused(), None);
        assert_eq!(model.selected(), None);
    }

    #[test]
    fn tabs_switching_to_automatic_keeps_selection_and_focus_in_sync() {
        let mut model = TabsModel::with_activation(3, TabsActivation::Manual);
        model.focus_next();
        model.activate_focused();
        model.focus_prev();
        model.set_activation(TabsActivation::Automatic);
        assert_eq!(model.focused(), Some(1));
        assert_eq!(model.selected(), Some(1));
    }
}
