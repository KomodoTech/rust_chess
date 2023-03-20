use crate::{board::NUM_INTERNAL_BOARD_SQUARES, error::CastlePermConversionError, square::Square};
use std::fmt;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display as EnumDisplay, EnumCount as EnumCountMacro, EnumIter, EnumString};

// CONSTANTS:
/// Number of permutations for castle permissions
pub const NUM_CASTLE_PERM: usize = 16;
pub const MAX_CASTLE_PERM_FEN_LEN: usize = 4;

/// Ordered so that adding numeric values of pieces will give you correct index
pub const CASTLE_PERM_FENS: [&str; NUM_CASTLE_PERM] = [
    "-", "K", "Q", "KQ", "k", "Kk", "Qk", "KQk", "q", "Kq", "Qq", "KQq", "kq", "Kkq", "Qkq", "KQkq",
];

/// CASTLE_PERM array allows use to quickly update castle permissions given
/// the square that a move started from, by performing a bitwise AND with the
/// current CastlePerm's value and the value received when indexing into
/// CASTLE_PERM at the index corresponding the to the move's start square.
#[rustfmt::skip]
pub const CASTLE_PERM: [u8; NUM_INTERNAL_BOARD_SQUARES] = [
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_1101, 0b_1111, 0b_1111, 0b_1111, 0b_1100, 0b_1111, 0b_1111, 0b_1110, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_0111, 0b_1111, 0b_1111, 0b_1111, 0b_0011, 0b_1111, 0b_1111, 0b_1011, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
    0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 0b_1111, 
];

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, EnumString, EnumDisplay, EnumCountMacro)]
pub enum Castle {
    WhiteKing = 1,
    WhiteQueen = 2,
    BlackKing = 4,
    BlackQueen = 8,
}

/// CastlePerm just holds a value between 0000 and 1111 which encodes the castling permissions
/// in the following manner:
/// BlackQueen BlackKing WhiteQueen WhiteKing
/// 1             1         1          1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlePerm(pub u8);
impl Default for CastlePerm {
    fn default() -> Self {
        CastlePerm(0b_1111)
    }
}

impl CastlePerm {
    pub fn new() -> Self {
        CastlePerm(0)
    }

    fn from_fen(value: &str) -> Result<Self, CastlePermConversionError> {
        let mut castle_perm = CastlePerm::new();
        for char in value.chars() {
            match char {
                'K' => {
                    castle_perm.0 += Castle::WhiteKing as u8;
                }
                'Q' => {
                    castle_perm.0 += Castle::WhiteQueen as u8;
                }
                'k' => {
                    castle_perm.0 += Castle::BlackKing as u8;
                }
                'q' => {
                    castle_perm.0 += Castle::BlackQueen as u8;
                }
                '-' => {}
                _ => {
                    return Err(CastlePermConversionError::FromStrInvalidChar {
                        invalid_string: value.to_owned(),
                        invalid_char: char,
                    });
                }
            }
        }
        match castle_perm.0 as usize {
            index if (0..=15).contains(&index) => match value {
                v if v == CASTLE_PERM_FENS[castle_perm.0 as usize] => Ok(castle_perm),
                _ => Err(CastlePermConversionError::FromStr {
                    invalid_string: value.to_owned(),
                }),
            },
            _ => Err(CastlePermConversionError::FromStrDuplicates {
                invalid_string: value.to_owned(),
            }),
        }
    }

    pub fn to_castle_perm_fen(&self) -> String {
        let mut castle_perms_fen = String::with_capacity(MAX_CASTLE_PERM_FEN_LEN);

        // Check each bit (aka each Castle/perm) in CastlePerm's value and build
        // castle_perms_fen according to whether or not the permission is set
        for perm in Castle::iter() {
            if self.0 & (perm as u8) > 0 {
                match perm {
                    Castle::WhiteKing => {
                        castle_perms_fen.push('K');
                    }
                    Castle::WhiteQueen => castle_perms_fen.push('Q'),
                    Castle::BlackKing => {
                        castle_perms_fen.push('k');
                    }
                    Castle::BlackQueen => {
                        castle_perms_fen.push('q');
                    }
                }
            }
        }
        match castle_perms_fen.len() {
            0 => "-".to_owned(),
            _ => castle_perms_fen,
        }
    }

    /// Update the CastlePermissions given start square of Move
    pub fn update(&mut self, start_square: Square) {
        self.0 &= CASTLE_PERM[start_square as usize];
    }
}

impl TryFrom<u8> for CastlePerm {
    type Error = CastlePermConversionError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            // only 16 different possible castle permissions
            v if v <= 0x0F => Ok(CastlePerm(v)),
            _ => Err(CastlePermConversionError::FromU8ValueTooLarge { invalid_u8: value }),
        }
    }
}

impl TryFrom<&str> for CastlePerm {
    type Error = CastlePermConversionError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_fen(value)
    }
}

/// Display in FEN style
impl fmt::Display for CastlePerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_castle_perm_fen().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //========================== UPDATE =======================================
    #[test]
    fn test_castle_perm_update_no_change() {
        // Not one of the squares we care about
        let start_square = Square::B1;
        let mut output = CastlePerm(0x_0B);
        output.update(start_square);
        let expected = CastlePerm(0x_0B);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_update_lose_white_queenside_perm() {
        let start_square = Square::A1; // White Queenside Rook
        let mut output = CastlePerm(0x_0F);
        output.update(start_square);
        let expected = CastlePerm(0b_1101);
        assert_eq!(output, expected);
    }

    // Every subsequent time after a piece moves off a relevant square
    // the castle permissions shouldn't change
    #[test]
    fn test_castle_perm_update_idempotent() {
        let start_square = Square::A1; // White Queenside Rook
        let mut output = CastlePerm(0x_0F);
        output.update(start_square);
        output.update(start_square);
        let expected = CastlePerm(0b_1101);
        assert_eq!(output, expected);
    }

    //========================== CONVERSIONS ==================================
    #[test]
    fn test_u8_from_castle_perm() {
        let input = CastlePerm::default();
        let output: u8 = input.0;
        let expected: u8 = 0x0F;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_u8_valid_input() {
        let input: u8 = 0b0000_0101;
        let output = CastlePerm::try_from(input);
        let expected = Ok(CastlePerm(input));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_u8_invalid_input() {
        let input: u8 = 0b0100_0101;
        let output = CastlePerm::try_from(input);
        let expected = Err(CastlePermConversionError::FromU8ValueTooLarge { invalid_u8: input });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_valid_default() {
        let input = "KQkq";
        let output = CastlePerm::try_from(input);
        let expected = Ok(CastlePerm(0b_1111));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_valid_empty() {
        let input = "-";
        let output = CastlePerm::try_from(input);
        let expected = Ok(CastlePerm(0));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_valid_half() {
        let input = "Qk";
        let output = CastlePerm::try_from(input);
        let expected = Ok(CastlePerm(0b_0110));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_invalid_char() {
        let input = "qX";
        let output = CastlePerm::try_from(input);
        let expected = Err(CastlePermConversionError::FromStrInvalidChar {
            invalid_string: input.to_owned(),
            invalid_char: 'X',
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_invalid_order() {
        let input = "qKQ";
        let output = CastlePerm::try_from(input);
        let expected = Err(CastlePermConversionError::FromStr {
            invalid_string: input.to_owned(),
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_too_many_chars() {
        let input = "KQkqK";
        let output = CastlePerm::try_from(input);
        let expected = Err(CastlePermConversionError::FromStrDuplicates {
            invalid_string: input.to_owned(),
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_dupe_dash() {
        let input = "--";
        let output = CastlePerm::try_from(input);
        let expected = Err(CastlePermConversionError::FromStr {
            invalid_string: input.to_owned(),
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_dupe_dash_and_too_long() {
        let input = "KQkq--";
        let output = CastlePerm::try_from(input);
        let expected = Err(CastlePermConversionError::FromStr {
            invalid_string: input.to_owned(),
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_display_default() {
        let input: CastlePerm = CastlePerm::default();
        let output = input.to_string();
        let expected = "KQkq";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_display_empty() {
        let input: CastlePerm = CastlePerm(0);
        let output = input.to_string();
        let expected = "-";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_display_half() {
        let input: CastlePerm = CastlePerm(0b_0110);
        let output = input.to_string();
        let expected = "Qk";
        assert_eq!(output, expected);
    }
}
