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
}

impl TabsModel {
    pub fn new(len: usize) -> Self {
        Self::with_activation(len, TabsActivation::Automatic)
    }

    pub fn with_activation(len: usize, activation: TabsActivation) -> Self {
        let focus = RovingFocus::with_active(len, if len > 0 { Some(0) } else { None }, true);
        let selected = if len > 0 { Some(0) } else { None };
        Self {
            focus,
            selected,
            activation,
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
    }

    pub fn set_len(&mut self, len: usize) {
        self.focus.set_len(len);
        self.selected = self.clamp_index(self.selected);
    }

    pub fn focus_index(&mut self, index: Option<usize>) -> Option<usize> {
        let focused = self.focus.set_active(index);
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn focus_next(&mut self) -> Option<usize> {
        let focused = self.focus.move_next();
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn focus_prev(&mut self) -> Option<usize> {
        let focused = self.focus.move_prev();
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn focus_first(&mut self) -> Option<usize> {
        let focused = self.focus.move_first();
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn focus_last(&mut self) -> Option<usize> {
        let focused = self.focus.move_last();
        if self.activation == TabsActivation::Automatic {
            self.selected = focused;
        }
        focused
    }

    pub fn select(&mut self, index: Option<usize>) -> Option<usize> {
        self.selected = self.clamp_index(index);
        if self.activation == TabsActivation::Automatic {
            self.focus.set_active(self.selected);
        }
        self.selected
    }

    pub fn activate_focused(&mut self) -> Option<usize> {
        if self.activation == TabsActivation::Manual {
            self.selected = self.focus.active();
        }
        self.selected
    }

    fn clamp_index(&self, index: Option<usize>) -> Option<usize> {
        match index {
            Some(index) => {
                if self.len() == 0 {
                    None
                } else if index < self.len() {
                    Some(index)
                } else {
                    Some(self.len() - 1)
                }
            }
            None => None,
        }
    }
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
}
