use std::fmt;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::{
    board::bitboard::BitBoard,
    error::ChessError as Error,
    util::{File, Rank, FILES_BOARD, RANKS_BOARD, SQUARE_120_TO_64, SQUARE_64_TO_120},
};

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumString)]
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

// TODO: This can end up being helpful in practice for certain calculations but
// conceptually it seems a bit strange since they really represent different
// concepts
// TODO: Get rid of this for now, a square is a bit in a bitboard and not a bitboard
impl TryFrom<BitBoard> for Square64 {
    type Error = Error;

    fn try_from(value: BitBoard) -> Result<Self, Self::Error> {
        let v: u64 = value.into();
        Self::iter()
            .find(|s| *s as u64 == v)
            .ok_or(Error::ParseSquare64FromBitBoardError(value))
    }
}

// TODO: confirm that there isn't a good way to do this for any generic integer type
impl TryFrom<u8> for Square64 {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as u8 == value)
            .ok_or(Error::ParseSquare64FromU8Error(value))
    }
}

impl Square64 {
    // TODO: get rid of optional return
    pub fn from_file_and_rank(file: File, rank: Rank) -> Option<Self> {
        // TODO: Get rid of check given that we're using Enum
        if (rank as u8 | file as u8) >> 3 == 0 {
            let index_64 = (file as u8) + (rank as u8) * 8;
            index_64.try_into().ok()
        } else {
            None
        }
    }

    // TODO: can't fail don't return result
    pub fn get_file(&self) -> Result<File, Error> {
        match FILES_BOARD[*self as usize] {
            Some(file) => Ok(file),
            None => Err(Error::Square64OnInvalidFile(*self)),
        }
    }

    // TODO: can't fail don't return result
    pub fn get_rank(&self) -> Result<Rank, Error> {
        match RANKS_BOARD[*self as usize] {
            Some(rank) => Ok(rank),
            None => Err(Error::Square64OnInvalidRank(*self)),
        }
    }
}

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumString)]
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

impl TryFrom<BitBoard> for Square {
    type Error = Error;

    fn try_from(value: BitBoard) -> Result<Self, Self::Error> {
        let v: u64 = value.into();
        Self::iter()
            .find(|s| *s as u64 == v)
            .ok_or(Error::ParseSquareFromBitBoardError(value))
    }
}

impl TryFrom<u8> for Square {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|s| *s as u8 == value)
            .ok_or(Error::ParseSquareFromU8Error(value))
    }
}

impl Square {
    pub fn from_file_and_rank(file: File, rank: Rank) -> Option<Self> {
        if (rank as u8 | file as u8) >> 3 == 0 {
            let index_120 = (21 + (file as u8) + (10 * (rank as u8)));
            match index_120.try_into() {
                Ok(square) => Some(square),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn from_file_and_rank_u8(file: u8, rank: u8) -> Option<Self> {
        if (rank | file) >> 3 == 0 {
            let index_120 = (21 + file + (10 * rank));
            match index_120.try_into() {
                Ok(square) => Some(square),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn get_file(&self) -> Result<File, Error> {
        match FILES_BOARD[*self as usize] {
            Some(file) => Ok(file),
            None => Err(Error::SquareOnInvalidFile(*self)),
        }
    }

    pub fn get_rank(&self) -> Result<Rank, Error> {
        match RANKS_BOARD[*self as usize] {
            Some(rank) => Ok(rank),
            None => Err(Error::SquareOnInvalidRank(*self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::NUM_BOARD_SQUARES;
    use std::convert::Into;

    use super::*;

    // Conversions
    #[test]
    fn test_square_120_to_square_64() {
        let input = Square::A6;
        let output: Square64 = input.into();
        let expected = Square64::A6;
        assert_eq!(output, expected);
        let output: Square64 = Square64::from(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_to_square_120() {
        let input = Square64::A6;
        let output: Square = input.into();
        let expected = Square::A6;
        assert_eq!(output, expected);
        let output: Square = Square::from(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_bitboard_valid() {
        let input = BitBoard(34);
        let output = Square::try_from(input);
        let expected = Ok(Square::D2);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_into_bitboard_valid() {
        let input = BitBoard(34);
        let output = input.try_into();
        let expected = Ok(Square::D2);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_from_bitboard_invalid() {
        let input = BitBoard(11);
        let output = Square::try_from(input);
        let expected = Err(Error::ParseSquareFromBitBoardError(BitBoard(11)));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_bitboard_valid() {
        let input = BitBoard(34);
        let output = Square64::try_from(input);
        let expected = Ok(Square64::C5);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_from_bitboard_invalid() {
        let input = BitBoard(64);
        let output = Square64::try_from(input);
        let expected = Err(Error::ParseSquare64FromBitBoardError(BitBoard(64)));
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
    fn test_square_120_try_from_str_invalid() {
        let input = "INVALID";
        let output = Square::try_from(input).map_err(Into::into);
        let expected = Err(Error::ParseSquareFromStrError(
            strum::ParseError::VariantNotFound,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_to_string() {
        let input = Square::D2;
        let output: String = input.to_string();
        let expected = "D2".to_string();
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
    fn test_square_64_try_from_str_invalid() {
        let input = "INVALID";
        let output = Square64::try_from(input).map_err(Into::into);
        let expected = Err(Error::ParseSquareFromStrError(
            strum::ParseError::VariantNotFound,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_to_string() {
        let input = Square64::D2;
        let output: String = input.to_string();
        let expected = "D2".to_string();
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
        let input = 11;
        let output = Square::try_from(input);
        let expected = Err(Error::ParseSquareFromU8Error(11));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_into_u8_valid() {
        let input: u8 = 34;
        let output = input.try_into();
        let expected = Ok(Square::D2);
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
        let input = 64;
        let output = Square64::try_from(input);
        let expected = Err(Error::ParseSquare64FromU8Error(64));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_into_u8_valid() {
        let input: u8 = 34;
        let output = input.try_into();
        let expected = Ok(Square64::C5);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_into_u8_invalid() {
        let input: u8 = 64;
        let output: Result<Square64, _> = input.try_into();
        let expected = Err(Error::ParseSquare64FromU8Error(64));
        assert_eq!(output, expected);
    }

    // Other methods

    #[test]
    fn test_from_file_and_rank_valid() {
        let square = Square::from_file_and_rank(File::FileB, Rank::Rank3);
        assert_eq!(square, Some(Square::B3));

        let square = Square::from_file_and_rank(File::FileH, Rank::Rank8);
        assert_eq!(square, Some(Square::H8));
    }

    #[test]
    fn test_get_file() {
        let output = Square::H7.get_file();
        let expected = Ok(File::FileH);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank() {
        let output = Square::H7.get_rank();
        let expected = Ok(Rank::Rank7);
        assert_eq!(output, expected);
    }
}
