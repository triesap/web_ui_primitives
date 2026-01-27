#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogState {
    Open,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogModel {
    open: bool,
    modal: bool,
}

impl DialogModel {
    pub fn new(open: bool) -> Self {
        Self { open, modal: true }
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn modal(&self) -> bool {
        self.modal
    }

    pub fn set_modal(&mut self, modal: bool) {
        self.modal = modal;
    }

    pub fn state(&self) -> DialogState {
        if self.open {
            DialogState::Open
        } else {
            DialogState::Closed
        }
    }

    pub fn set_open(&mut self, open: bool) -> bool {
        let changed = self.open != open;
        self.open = open;
        changed
    }

    pub fn toggle(&mut self) -> bool {
        self.open = !self.open;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{DialogModel, DialogState};

    #[test]
    fn dialog_toggle_changes_state() {
        let mut model = DialogModel::new(false);
        assert_eq!(model.state(), DialogState::Closed);
        model.toggle();
        assert_eq!(model.state(), DialogState::Open);
    }

    #[test]
    fn dialog_set_open_reports_change() {
        let mut model = DialogModel::new(false);
        assert!(model.set_open(true));
        assert!(!model.set_open(true));
    }
}
