use crate::{
    error::{FileConversionError, RankConversionError},
    gamestate::NUM_BOARD_SQUARES,
    pieces::Piece,
    squares::{Square, Square64},
};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, EnumString};

// CONSTANTS:

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq, Display, EnumCountMacro)]
pub enum File {
    FileA,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH,
}

impl TryFrom<usize> for File {
    type Error = FileConversionError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|r| *r as usize == value)
            .ok_or(FileConversionError::FromUsize(value))
    }
}

impl From<File> for char {
    fn from(value: File) -> Self {
        match value {
            File::FileA => 'A',
            File::FileB => 'B',
            File::FileC => 'C',
            File::FileD => 'D',
            File::FileE => 'E',
            File::FileF => 'F',
            File::FileG => 'G',
            File::FileH => 'H',
        }
    }
}

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq, Display, EnumCountMacro)]
pub enum Rank {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,
}

impl TryFrom<usize> for Rank {
    type Error = RankConversionError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|r| *r as usize == value)
            .ok_or(RankConversionError::FromUsize(value))
    }
}

// impl Add<usize> for Rank {
//     type Output = Result<Self, ConversionError>;
//     fn add(self, rhs: usize) -> Self::Output {
//         let result: Result<Self, ConversionError> = (self as usize + rhs).try_into();
//         result
//     }
// }

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

#[rustfmt::skip]
pub const FILES_BOARD: [Option<File>; NUM_BOARD_SQUARES] = [
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
];

#[rustfmt::skip]
pub const RANKS_BOARD: [Option<Rank>; NUM_BOARD_SQUARES] = [
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1), None,
    None,  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2), None,
    None,  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3), None,
    None,  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4), None,
    None,  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5), None,
    None,  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6), None,
    None,  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7), None,
    None,  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8), None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
];
