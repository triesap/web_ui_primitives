#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
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

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Horizontal
    }
}

#[cfg(test)]
mod tests {
    use super::Orientation;

    #[test]
    fn orientation_aria_value() {
        assert_eq!(Orientation::Horizontal.as_aria_value(), "horizontal");
        assert_eq!(Orientation::Vertical.as_aria_value(), "vertical");
    }
}
