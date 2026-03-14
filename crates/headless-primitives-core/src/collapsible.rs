#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollapsibleState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollapsibleModel {
    open: bool,
    disabled: bool,
}

impl CollapsibleModel {
    pub fn new(open: bool) -> Self {
        Self {
            open,
            disabled: false,
        }
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }

    pub fn state(&self) -> CollapsibleState {
        if self.open {
            CollapsibleState::Open
        } else {
            CollapsibleState::Closed
        }
    }

    pub fn set_open(&mut self, open: bool) -> bool {
        if self.disabled {
            return false;
        }
        let changed = self.open != open;
        self.open = open;
        changed
    }

    pub fn toggle(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        self.open = !self.open;
        true
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

#[cfg(test)]
mod tests {
    use super::{CollapsibleModel, CollapsibleState};

    #[test]
    fn collapsible_toggle() {
        let mut model = CollapsibleModel::new(false);
        assert_eq!(model.state(), CollapsibleState::Closed);
        assert!(model.toggle());
        assert_eq!(model.state(), CollapsibleState::Open);
    }

    #[test]
    fn collapsible_toggle_blocked_when_disabled() {
        let mut model = CollapsibleModel::new(false);
        model.set_disabled(true);
        assert!(!model.toggle());
        assert_eq!(model.state(), CollapsibleState::Closed);
    }
}
