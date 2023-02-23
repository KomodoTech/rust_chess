use crate::{
    board::{bitboard::BitBoard, NUM_BOARD_SQUARES},
    color::Color,
    error::{Square64ConversionError, SquareConversionError},
    file::{File, FILES_BOARD},
    rank::{Rank, RANKS_BOARD},
};
use num::Integer;
use std::{
    fmt,
    ops::{Add, AddAssign, Sub},
    str::FromStr,
};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, EnumString};

// Conversion Arrays:
#[rustfmt::skip]
pub const SQUARE_120_TO_64: [Option<Square64>; NUM_BOARD_SQUARES] = [
    None, None,                None,                None,                None,                None,                None,                None,                None,               None,
    None, None,                None,                None,                None,                None,                None,                None,                None,               None,
    None, Some(Square64::A1),  Some(Square64::B1),  Some(Square64::C1),  Some(Square64::D1),  Some(Square64::E1),  Some(Square64::F1),  Some(Square64::G1),  Some(Square64::H1), None,
    None, Some(Square64::A2),  Some(Square64::B2),  Some(Square64::C2),  Some(Square64::D2),  Some(Square64::E2),  Some(Square64::F2),  Some(Square64::G2),  Some(Square64::H2), None,
    None, Some(Square64::A3),  Some(Square64::B3),  Some(Square64::C3),  Some(Square64::D3),  Some(Square64::E3),  Some(Square64::F3),  Some(Square64::G3),  Some(Square64::H3), None,
    None, Some(Square64::A4),  Some(Square64::B4),  Some(Square64::C4),  Some(Square64::D4),  Some(Square64::E4),  Some(Square64::F4),  Some(Square64::G4),  Some(Square64::H4), None,
    None, Some(Square64::A5),  Some(Square64::B5),  Some(Square64::C5),  Some(Square64::D5),  Some(Square64::E5),  Some(Square64::F5),  Some(Square64::G5),  Some(Square64::H5), None,
    None, Some(Square64::A6),  Some(Square64::B6),  Some(Square64::C6),  Some(Square64::D6),  Some(Square64::E6),  Some(Square64::F6),  Some(Square64::G6),  Some(Square64::H6), None,
    None, Some(Square64::A7),  Some(Square64::B7),  Some(Square64::C7),  Some(Square64::D7),  Some(Square64::E7),  Some(Square64::F7),  Some(Square64::G7),  Some(Square64::H7), None,
    None, Some(Square64::A8),  Some(Square64::B8),  Some(Square64::C8),  Some(Square64::D8),  Some(Square64::E8),  Some(Square64::F8),  Some(Square64::G8),  Some(Square64::H8), None,
    None, None,                None,                None,                None,                None,                None,                None,                None,               None,
    None, None,                None,                None,                None,                None,                None,                None,                None,               None,
];

#[rustfmt::skip]
pub const SQUARE_64_TO_120: [Option<Square>; 64] = [
    Some(Square::A1), Some(Square::B1), Some(Square::C1), Some(Square::D1), Some(Square::E1), Some(Square::F1), Some(Square::G1), Some(Square::H1),
    Some(Square::A2), Some(Square::B2), Some(Square::C2), Some(Square::D2), Some(Square::E2), Some(Square::F2), Some(Square::G2), Some(Square::H2),
    Some(Square::A3), Some(Square::B3), Some(Square::C3), Some(Square::D3), Some(Square::E3), Some(Square::F3), Some(Square::G3), Some(Square::H3),
    Some(Square::A4), Some(Square::B4), Some(Square::C4), Some(Square::D4), Some(Square::E4), Some(Square::F4), Some(Square::G4), Some(Square::H4),
    Some(Square::A5), Some(Square::B5), Some(Square::C5), Some(Square::D5), Some(Square::E5), Some(Square::F5), Some(Square::G5), Some(Square::H5),
    Some(Square::A6), Some(Square::B6), Some(Square::C6), Some(Square::D6), Some(Square::E6), Some(Square::F6), Some(Square::G6), Some(Square::H6),
    Some(Square::A7), Some(Square::B7), Some(Square::C7), Some(Square::D7), Some(Square::E7), Some(Square::F7), Some(Square::G7), Some(Square::H7),
    Some(Square::A8), Some(Square::B8), Some(Square::C8), Some(Square::D8), Some(Square::E8), Some(Square::F8), Some(Square::G8), Some(Square::H8)
];

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumString, EnumCountMacro)]
#[rustfmt::skip]
#[strum(use_phf)]
pub enum Square64 {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl From<Square> for Square64 {
    fn from(square_120: Square) -> Self {
        SQUARE_120_TO_64[square_120 as usize]
            .expect("10x12 Square should have a corresponding 8x8 Square64")
    }
}

impl Add<usize> for Square64 {
    type Output = Result<Self, Square64ConversionError>;
    fn add(self, rhs: usize) -> Self::Output {
        let result: Result<Self, Square64ConversionError> = (self as usize + rhs).try_into();
        result
    }
}

impl TryFrom<u8> for Square64 {
    type Error = Square64ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as u8 == value)
            .ok_or(Square64ConversionError::FromU8(value))
    }
}

impl TryFrom<u32> for Square64 {
    type Error = Square64ConversionError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as u32 == value)
            .ok_or(Square64ConversionError::FromU32(value))
    }
}

impl TryFrom<usize> for Square64 {
    type Error = Square64ConversionError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as usize == value)
            .ok_or(Square64ConversionError::FromUsize(value))
    }
}

impl Square64 {
    pub fn from_file_and_rank(file: File, rank: Rank) -> Self {
        let index_64 = (file as u8) + (rank as u8) * 8;
        index_64.try_into().expect(
            "index_64 should map to valid Square64 since rank and file must be in range 0..=7",
        )
    }

    pub fn get_file(&self) -> File {
        FILES_BOARD[*self as usize].expect("should return valid File since every Square64 has one")
    }

    pub fn get_rank(&self) -> Rank {
        RANKS_BOARD[*self as usize].expect("should return valid File since every Square64 has one")
    }

    pub fn get_color(&self) -> Color {
        // NOTE: Rank1's value as u8 is 0
        // XOR relationship:
        // (rank_even, index_even) -> black
        // (rank_even, index_odd) -> white
        // (rank_odd, index_even) -> white
        // (rank_odd, index_odd) -> black
        let parity_xor = (self.get_rank() as u8).is_even() ^ (*self as u8).is_even();
        match parity_xor {
            true => Color::White,
            false => Color::Black,
        }
    }
}

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumString, EnumCountMacro)]
#[rustfmt::skip]
#[strum(use_phf)]
pub enum Square {
    A1 = 21, B1, C1, D1, E1, F1, G1, H1,
    A2 = 31, B2, C2, D2, E2, F2, G2, H2,
    A3 = 41, B3, C3, D3, E3, F3, G3, H3,
    A4 = 51, B4, C4, D4, E4, F4, G4, H4,
    A5 = 61, B5, C5, D5, E5, F5, G5, H5,
    A6 = 71, B6, C6, D6, E6, F6, G6, H6,
    A7 = 81, B7, C7, D7, E7, F7, G7, H7,
    A8 = 91, B8, C8, D8, E8, F8, G8, H8,
}

impl From<Square64> for Square {
    fn from(square_64: Square64) -> Self {
        SQUARE_64_TO_120[square_64 as usize]
            .expect("8x8 Square64 should have a corresponding 10x12 Square")
    }
}

impl Add<i8> for Square {
    type Output = Result<Self, SquareConversionError>;
    fn add(self, rhs: i8) -> Self::Output {
        (self as i8 + rhs).try_into()
    }
}

impl Sub<i8> for Square {
    type Output = Result<Self, SquareConversionError>;
    fn sub(self, rhs: i8) -> Self::Output {
        (self as i8 - rhs).try_into()
    }
}

impl TryFrom<i8> for Square {
    type Error = SquareConversionError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as i8 == value)
            .ok_or(SquareConversionError::FromI8(value))
    }
}

impl TryFrom<u8> for Square {
    type Error = SquareConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as u8 == value)
            .ok_or(SquareConversionError::FromU8(value))
    }
}

impl TryFrom<u32> for Square {
    type Error = SquareConversionError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as u32 == value)
            .ok_or(SquareConversionError::FromU32(value))
    }
}

impl TryFrom<usize> for Square {
    type Error = SquareConversionError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as usize == value)
            .ok_or(SquareConversionError::FromUsize(value))
    }
}

impl Square {
    pub fn from_file_and_rank(file: File, rank: Rank) -> Self {
        let index_120 = (21 + (file as u8) + (10 * (rank as u8)));
        index_120.try_into().expect(
            "index_64 should map to valid Square since rank and file must be in range 0..=7",
        )
    }

    pub fn get_file(&self) -> File {
        FILES_BOARD[*self as usize].expect("should return valid File since every Square has one")
    }

    pub fn get_rank(&self) -> Rank {
        RANKS_BOARD[*self as usize].expect("should return valid Rank since every Square has one")
    }

    pub fn get_color(&self) -> Color {
        // NOTE: Rank1's value as u8 is 0
        // XOR relationship:
        // (rank_even, index_even) -> white
        // (rank_even, index_odd) -> black
        // (rank_odd, index_even) -> black
        // (rank_odd, index_odd) -> white
        let parity_xor = (self.get_rank() as u8).is_even() ^ (*self as u8).is_even();
        match parity_xor {
            true => Color::Black,
            false => Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Conversions
    #[test]
    fn test_square_120_to_square_64() {
        let input = Square::A6;
        let output: Square64 = input.into();
        let expected = Square64::A6;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_to_square_120() {
        let input = Square64::A6;
        let output: Square = input.into();
        let expected = Square::A6;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_addition_usize_valid() {
        let lhs = Square64::B1;
        let rhs: usize = 8;
        let output = lhs + rhs;
        let expected = Ok(Square64::B2);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_addition_usize_valid_zero() {
        let lhs = Square64::B1;
        let rhs: usize = 0;
        let output = lhs + rhs;
        let expected = Ok(Square64::B1);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_addition_usize_invalid() {
        let lhs = Square64::B1;
        let rhs: usize = 63;
        let output = lhs + rhs;
        let expected = Err(Square64ConversionError::FromUsize(lhs as usize + rhs));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_str_valid() {
        let input = "D2";
        let output = Square::try_from(input);
        let expected = Ok(Square::D2);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_str_lowercase_invalid() {
        let input = "d2";
        let output = Square::try_from(input).map_err(Into::into);
        let expected = Err(SquareConversionError::FromStr(
            strum::ParseError::VariantNotFound,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_str_invalid() {
        let input = "INVALID";
        let output = Square::try_from(input).map_err(Into::into);
        let expected = Err(SquareConversionError::FromStr(
            strum::ParseError::VariantNotFound,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_to_string() {
        let input = Square::D2;
        let output: String = input.to_string();
        let expected = "D2".to_owned();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_str_valid() {
        let input = "D2";
        let output = Square64::try_from(input);
        let expected = Ok(Square64::D2);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_str_lowercase_invalid() {
        let input = "d2";
        let output = Square64::try_from(input).map_err(Into::into);
        let expected = Err(Square64ConversionError::FromStr(
            strum::ParseError::VariantNotFound,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_str_invalid() {
        let input = "INVALID";
        let output = Square64::try_from(input).map_err(Into::into);
        let expected = Err(Square64ConversionError::FromStr(
            strum::ParseError::VariantNotFound,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_to_string() {
        let input = Square64::D2;
        let output: String = input.to_string();
        let expected = "D2".to_owned();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_u8_valid() {
        let input: u8 = 34;
        let output = Square::try_from(input);
        let expected = Ok(Square::D2);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_u8_invalid() {
        let input: u8 = 11;
        let output = Square::try_from(input);
        let expected = Err(SquareConversionError::FromU8(11));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_u8_valid() {
        let input: u8 = 34;
        let output = Square64::try_from(input);
        let expected = Ok(Square64::C5);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_u8_invalid() {
        let input: u8 = 64;
        let output = Square64::try_from(input);
        let expected = Err(Square64ConversionError::FromU8(64));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_u32_valid() {
        let input: u32 = 34;
        let output = Square::try_from(input);
        let expected = Ok(Square::D2);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_u32_invalid() {
        let input: u32 = 11;
        let output = Square::try_from(input);
        let expected = Err(SquareConversionError::FromU32(11));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_u32_valid() {
        let input: u32 = 34;
        let output = Square64::try_from(input);
        let expected = Ok(Square64::C5);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_u32_invalid() {
        let input: u32 = 64;
        let output = Square64::try_from(input);
        let expected = Err(Square64ConversionError::FromU32(64));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_usize_valid() {
        let input: usize = 34;
        let output = Square::try_from(input);
        let expected = Ok(Square::D2);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_usize_invalid() {
        let input: usize = 11;
        let output = Square::try_from(input);
        let expected = Err(SquareConversionError::FromUsize(11));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_usize_valid() {
        let input: usize = 34;
        let output = Square64::try_from(input);
        let expected = Ok(Square64::C5);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_usize_invalid() {
        let input: usize = 64;
        let output = Square64::try_from(input);
        let expected = Err(Square64ConversionError::FromUsize(64));
        assert_eq!(output, expected);
    }

    // Other methods
    #[test]
    fn test_from_file_and_rank() {
        let square = Square::from_file_and_rank(File::FileB, Rank::Rank3);
        assert_eq!(square, Square::B3);
    }

    #[test]
    fn test_get_file() {
        let output = Square::H7.get_file();
        let expected = File::FileH;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank() {
        let output = Square::H7.get_rank();
        let expected = Rank::Rank7;
        assert_eq!(output, expected);
    }
}
