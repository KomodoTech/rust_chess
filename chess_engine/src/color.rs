use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, EnumString};

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumCountMacro, Display)]
pub enum Color {
    White,
    Black,
}

impl From<Color> for char {
    fn from(value: Color) -> Self {
        match value {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl Color {
    pub fn toggle(&mut self) {
        match self {
            Color::White => *self = Color::Black,
            Color::Black => *self = Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_toggle_once() {
        let mut output = Color::White;
        output.toggle();
        let expected = Color::Black;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_color_toggle_twice() {
        let mut output = Color::White;
        output.toggle();
        output.toggle();
        let expected = Color::White;
        assert_eq!(output, expected);
    }
}
