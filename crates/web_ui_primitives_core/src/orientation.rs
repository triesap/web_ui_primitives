#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}

impl Orientation {
    pub fn as_aria_value(self) -> &'static str {
        match self {
            Orientation::Horizontal => "horizontal",
            Orientation::Vertical => "vertical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, Orientation};

    #[test]
    fn direction_defaults_to_ltr() {
        assert_eq!(Direction::default(), Direction::Ltr);
    }

    #[test]
    fn orientation_aria_value() {
        assert_eq!(Orientation::Horizontal.as_aria_value(), "horizontal");
        assert_eq!(Orientation::Vertical.as_aria_value(), "vertical");
    }
}
