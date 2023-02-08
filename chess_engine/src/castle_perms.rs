use crate::error::ConversionError;
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, EnumString, EnumDisplay, EnumCountMacro)]
enum Castle {
    WhiteKing = 1,
    WhiteQueen = 2,
    BlackKing = 4,
    BlackQueen = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlePerm([Option<Castle>; 4]);
impl Default for CastlePerm {
    fn default() -> Self {
        Self([
            Some(Castle::WhiteKing),
            Some(Castle::WhiteQueen),
            Some(Castle::BlackKing),
            Some(Castle::BlackQueen),
        ])
    }
}

impl From<CastlePerm> for u8 {
    fn from(value: CastlePerm) -> Self {
        let mut result: u8 = 0;
        for perm in value.0.into_iter().flatten() {
            result += perm as u8;
        }
        result
    }
}

impl TryFrom<u8> for CastlePerm {
    type Error = ConversionError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            // only 16 different possible castle permissions
            v if v <= 0x0F => {
                // NOTE: default castle_perm are all some/set
                let mut castle_perm = Self::default();
                for (i, castle) in Castle::iter().enumerate() {
                    // check if bit corresponding to Castle permission is not set ("turn off")
                    if ((v & (castle as u8)) == 0) {
                        castle_perm.0[i] = None;
                    }
                }
                Ok(castle_perm)
            }
            _ => Err(ConversionError::ParseCastlePermFromU8ErrorValueTooLarge(
                value,
            )),
        }
    }
}

impl TryFrom<&str> for CastlePerm {
    type Error = ConversionError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut castle_perm = CastlePerm([None; Castle::COUNT]);
        let mut castle_perm_fens_index: usize = 0;
        for char in value.chars() {
            match char {
                'K' => {
                    castle_perm_fens_index += Castle::WhiteKing as usize;
                    castle_perm.0[0] = Some(Castle::WhiteKing);
                }
                'Q' => {
                    castle_perm_fens_index += Castle::WhiteQueen as usize;
                    castle_perm.0[1] = Some(Castle::WhiteQueen);
                }
                'k' => {
                    castle_perm_fens_index += Castle::BlackKing as usize;
                    castle_perm.0[2] = Some(Castle::BlackKing);
                }
                'q' => {
                    castle_perm_fens_index += Castle::BlackQueen as usize;
                    castle_perm.0[3] = Some(Castle::BlackQueen)
                }
                '-' => {}
                _ => {
                    return Err(ConversionError::ParseCastlePermFromStrInvalidChar(
                        value.to_string(),
                        char,
                    ));
                }
            }
        }
        match castle_perm_fens_index {
            index if (0..=15).contains(&index) => match value {
                v if v == CASTLE_PERM_FENS[castle_perm_fens_index] => Ok(castle_perm),
                _ => Err(ConversionError::ParseCastlePermFromStr(value.to_string())),
            },
            _ => Err(ConversionError::ParseCastlePermFromStrDuplicates(
                value.to_string(),
            )),
        }
    }
}

/// Display in FEN style
impl fmt::Display for CastlePerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut castle_perms_fen = String::with_capacity(MAX_CASTLE_PERM_FEN_LEN);
        for perm in self.0.into_iter().flatten() {
            match perm.to_string().as_str() {
                "WhiteKing" => {
                    castle_perms_fen.push('K');
                }
                "WhiteQueen" => castle_perms_fen.push('Q'),
                "BlackKing" => {
                    castle_perms_fen.push('k');
                }
                "BlackQueen" => {
                    castle_perms_fen.push('q');
                }
                _ => {
                    panic!()
                }
            }
        }
        match castle_perms_fen.len() {
            0 => write!(f, "-"),
            _ => write!(f, "{}", castle_perms_fen.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_from_castle_perm() {
        let input = CastlePerm::default();
        let output: u8 = u8::from(input);
        let expected: u8 = 0x0F;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_u8_valid_input() {
        let input: u8 = 0b0000_0101;
        let output = CastlePerm::try_from(input);
        let expected = Ok(CastlePerm([
            Some(Castle::WhiteKing),
            None,
            Some(Castle::BlackKing),
            None,
        ]));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_u8_invalid_input() {
        let input: u8 = 0b0100_0101;
        let output = CastlePerm::try_from(input);
        let expected = Err(ConversionError::ParseCastlePermFromU8ErrorValueTooLarge(
            input,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_valid_default() {
        let input = "KQkq";
        let output = CastlePerm::try_from(input);
        let expected = Ok(CastlePerm([
            Some(Castle::WhiteKing),
            Some(Castle::WhiteQueen),
            Some(Castle::BlackKing),
            Some(Castle::BlackQueen),
        ]));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_valid_empty() {
        let input = "-";
        let output = CastlePerm::try_from(input);
        let expected = Ok(CastlePerm([None; MAX_CASTLE_PERM_FEN_LEN]));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_valid_half() {
        let input = "Qk";
        let output = CastlePerm::try_from(input);
        let expected = Ok(CastlePerm([
            None,
            Some(Castle::WhiteQueen),
            Some(Castle::BlackKing),
            None,
        ]));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_invalid_char() {
        let input = "qX";
        let output = CastlePerm::try_from(input);
        let expected = Err(ConversionError::ParseCastlePermFromStrInvalidChar(
            input.to_string(),
            'X',
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_invalid_order() {
        let input = "qKQ";
        let output = CastlePerm::try_from(input);
        let expected = Err(ConversionError::ParseCastlePermFromStr(input.to_string()));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_too_many_chars() {
        let input = "KQkqK";
        let output = CastlePerm::try_from(input);
        let expected = Err(ConversionError::ParseCastlePermFromStrDuplicates(
            input.to_string(),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_dupe_dash() {
        let input = "--";
        let output = CastlePerm::try_from(input);
        let expected = Err(ConversionError::ParseCastlePermFromStr(input.to_string()));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_try_from_str_dupe_dash_and_too_long() {
        let input = "KQkq--";
        let output = CastlePerm::try_from(input);
        let expected = Err(ConversionError::ParseCastlePermFromStr(input.to_string()));
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
        let input: CastlePerm = CastlePerm([None; 4]);
        let output = input.to_string();
        let expected = "-";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_castle_perm_display_half() {
        let input: CastlePerm = CastlePerm([
            None,
            Some(Castle::WhiteQueen),
            Some(Castle::BlackKing),
            None,
        ]);
        let output = input.to_string();
        let expected = "Qk";
        assert_eq!(output, expected);
    }
}
