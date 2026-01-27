#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RovingFocus {
    len: usize,
    active: Option<usize>,
    looped: bool,
}

impl RovingFocus {
    pub fn new(len: usize) -> Self {
        Self::with_active(len, if len > 0 { Some(0) } else { None }, true)
    }

    pub fn with_active(len: usize, active: Option<usize>, looped: bool) -> Self {
        let mut focus = Self {
            len,
            active,
            looped,
        };
        focus.active = focus.clamp_active(focus.active);
        focus
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn active(&self) -> Option<usize> {
        self.active
    }

    pub fn looped(&self) -> bool {
        self.looped
    }

    pub fn set_looped(&mut self, looped: bool) {
        self.looped = looped;
    }

    pub fn set_len(&mut self, len: usize) -> Option<usize> {
        self.len = len;
        self.active = self.clamp_active(self.active);
        self.active
    }

    pub fn set_active(&mut self, index: Option<usize>) -> Option<usize> {
        self.active = self.clamp_active(index);
        self.active
    }

    pub fn move_next(&mut self) -> Option<usize> {
        let len = self.len;
        if len == 0 {
            self.active = None;
            return None;
        }

        let next = match self.active {
            None => Some(0),
            Some(index) if index + 1 < len => Some(index + 1),
            Some(_) if self.looped => Some(0),
            Some(_) => None,
        };

        if let Some(index) = next {
            self.active = Some(index);
        }

        next
    }

    pub fn move_prev(&mut self) -> Option<usize> {
        let len = self.len;
        if len == 0 {
            self.active = None;
            return None;
        }

        let prev = match self.active {
            None => Some(len.saturating_sub(1)),
            Some(index) if index > 0 => Some(index - 1),
            Some(_) if self.looped => Some(len.saturating_sub(1)),
            Some(_) => None,
        };

        if let Some(index) = prev {
            self.active = Some(index);
        }

        prev
    }

    pub fn move_first(&mut self) -> Option<usize> {
        if self.len == 0 {
            self.active = None;
            return None;
        }
        self.active = Some(0);
        self.active
    }

    pub fn move_last(&mut self) -> Option<usize> {
        if self.len == 0 {
            self.active = None;
            return None;
        }
        self.active = Some(self.len - 1);
        self.active
    }

    fn clamp_active(&self, active: Option<usize>) -> Option<usize> {
        match active {
            Some(index) => {
                if self.len == 0 {
                    None
                } else if index < self.len {
                    Some(index)
                } else {
                    Some(self.len - 1)
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RovingFocus;

    #[test]
    fn roving_focus_wraps_when_looped() {
        let mut focus = RovingFocus::with_active(3, Some(2), true);
        assert_eq!(focus.move_next(), Some(0));
        assert_eq!(focus.active(), Some(0));
    }

    #[test]
    fn roving_focus_stops_when_not_looped() {
        let mut focus = RovingFocus::with_active(3, Some(2), false);
        assert_eq!(focus.move_next(), None);
        assert_eq!(focus.active(), Some(2));
    }

    #[test]
    fn roving_focus_handles_empty() {
        let mut focus = RovingFocus::new(0);
        assert_eq!(focus.move_next(), None);
        assert_eq!(focus.active(), None);
    }
}
