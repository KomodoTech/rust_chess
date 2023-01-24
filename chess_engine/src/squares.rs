use std::fmt;
use std::str::FromStr;
use strum_macros::Display;

use crate::{
    board::bitboard::BitBoard,
    error::ChessError as Error,
    util::{File, Rank, FILES_BOARD, RANKS_BOARD, SQUARE_120_TO_64, SQUARE_64_TO_120},
};

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq)]
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
            .expect("Conversion from 10x12 Square to 8x8 Square failed")
    }
}

impl TryFrom<BitBoard> for Square64 {
    type Error = Error;

    fn try_from(value: BitBoard) -> Result<Self, Self::Error> {
        let v: u64 = value.into();
        match v {
            v if v == Square64::A1 as u64 => Ok(Square64::A1),
            v if v == Square64::B1 as u64 => Ok(Square64::B1),
            v if v == Square64::C1 as u64 => Ok(Square64::C1),
            v if v == Square64::D1 as u64 => Ok(Square64::D1),
            v if v == Square64::E1 as u64 => Ok(Square64::E1),
            v if v == Square64::F1 as u64 => Ok(Square64::F1),
            v if v == Square64::G1 as u64 => Ok(Square64::G1),
            v if v == Square64::H1 as u64 => Ok(Square64::H1),
            v if v == Square64::A2 as u64 => Ok(Square64::A2),
            v if v == Square64::B2 as u64 => Ok(Square64::B2),
            v if v == Square64::C2 as u64 => Ok(Square64::C2),
            v if v == Square64::D2 as u64 => Ok(Square64::D2),
            v if v == Square64::E2 as u64 => Ok(Square64::E2),
            v if v == Square64::F2 as u64 => Ok(Square64::F2),
            v if v == Square64::G2 as u64 => Ok(Square64::G2),
            v if v == Square64::H2 as u64 => Ok(Square64::H2),
            v if v == Square64::A3 as u64 => Ok(Square64::A3),
            v if v == Square64::B3 as u64 => Ok(Square64::B3),
            v if v == Square64::C3 as u64 => Ok(Square64::C3),
            v if v == Square64::D3 as u64 => Ok(Square64::D3),
            v if v == Square64::E3 as u64 => Ok(Square64::E3),
            v if v == Square64::F3 as u64 => Ok(Square64::F3),
            v if v == Square64::G3 as u64 => Ok(Square64::G3),
            v if v == Square64::H3 as u64 => Ok(Square64::H3),
            v if v == Square64::A4 as u64 => Ok(Square64::A4),
            v if v == Square64::B4 as u64 => Ok(Square64::B4),
            v if v == Square64::C4 as u64 => Ok(Square64::C4),
            v if v == Square64::D4 as u64 => Ok(Square64::D4),
            v if v == Square64::E4 as u64 => Ok(Square64::E4),
            v if v == Square64::F4 as u64 => Ok(Square64::F4),
            v if v == Square64::G4 as u64 => Ok(Square64::G4),
            v if v == Square64::H4 as u64 => Ok(Square64::H4),
            v if v == Square64::A5 as u64 => Ok(Square64::A5),
            v if v == Square64::B5 as u64 => Ok(Square64::B5),
            v if v == Square64::C5 as u64 => Ok(Square64::C5),
            v if v == Square64::D5 as u64 => Ok(Square64::D5),
            v if v == Square64::E5 as u64 => Ok(Square64::E5),
            v if v == Square64::F5 as u64 => Ok(Square64::F5),
            v if v == Square64::G5 as u64 => Ok(Square64::G5),
            v if v == Square64::H5 as u64 => Ok(Square64::H5),
            v if v == Square64::A6 as u64 => Ok(Square64::A6),
            v if v == Square64::B6 as u64 => Ok(Square64::B6),
            v if v == Square64::C6 as u64 => Ok(Square64::C6),
            v if v == Square64::D6 as u64 => Ok(Square64::D6),
            v if v == Square64::E6 as u64 => Ok(Square64::E6),
            v if v == Square64::F6 as u64 => Ok(Square64::F6),
            v if v == Square64::G6 as u64 => Ok(Square64::G6),
            v if v == Square64::H6 as u64 => Ok(Square64::H6),
            v if v == Square64::A7 as u64 => Ok(Square64::A7),
            v if v == Square64::B7 as u64 => Ok(Square64::B7),
            v if v == Square64::C7 as u64 => Ok(Square64::C7),
            v if v == Square64::D7 as u64 => Ok(Square64::D7),
            v if v == Square64::E7 as u64 => Ok(Square64::E7),
            v if v == Square64::F7 as u64 => Ok(Square64::F7),
            v if v == Square64::G7 as u64 => Ok(Square64::G7),
            v if v == Square64::H7 as u64 => Ok(Square64::H7),
            v if v == Square64::A8 as u64 => Ok(Square64::A8),
            v if v == Square64::B8 as u64 => Ok(Square64::B8),
            v if v == Square64::C8 as u64 => Ok(Square64::C8),
            v if v == Square64::D8 as u64 => Ok(Square64::D8),
            v if v == Square64::E8 as u64 => Ok(Square64::E8),
            v if v == Square64::F8 as u64 => Ok(Square64::F8),
            v if v == Square64::G8 as u64 => Ok(Square64::G8),
            v if v == Square64::H8 as u64 => Ok(Square64::H8),
            _ => Err(Error::ParseSquare64FromBitBoardError(value)),
        }
    }
}

impl TryFrom<&str> for Square64 {
    type Error = Error;

    fn try_from(square_name: &str) -> Result<Self, Self::Error> {
        match square_name {
            "A1" => Ok(Square64::A1),
            "B1" => Ok(Square64::B1),
            "C1" => Ok(Square64::C1),
            "D1" => Ok(Square64::D1),
            "E1" => Ok(Square64::E1),
            "F1" => Ok(Square64::F1),
            "G1" => Ok(Square64::G1),
            "H1" => Ok(Square64::H1),
            "A2" => Ok(Square64::A2),
            "B2" => Ok(Square64::B2),
            "C2" => Ok(Square64::C2),
            "D2" => Ok(Square64::D2),
            "E2" => Ok(Square64::E2),
            "F2" => Ok(Square64::F2),
            "G2" => Ok(Square64::G2),
            "H2" => Ok(Square64::H2),
            "A3" => Ok(Square64::A3),
            "B3" => Ok(Square64::B3),
            "C3" => Ok(Square64::C3),
            "D3" => Ok(Square64::D3),
            "E3" => Ok(Square64::E3),
            "F3" => Ok(Square64::F3),
            "G3" => Ok(Square64::G3),
            "H3" => Ok(Square64::H3),
            "A4" => Ok(Square64::A4),
            "B4" => Ok(Square64::B4),
            "C4" => Ok(Square64::C4),
            "D4" => Ok(Square64::D4),
            "E4" => Ok(Square64::E4),
            "F4" => Ok(Square64::F4),
            "G4" => Ok(Square64::G4),
            "H4" => Ok(Square64::H4),
            "A5" => Ok(Square64::A5),
            "B5" => Ok(Square64::B5),
            "C5" => Ok(Square64::C5),
            "D5" => Ok(Square64::D5),
            "E5" => Ok(Square64::E5),
            "F5" => Ok(Square64::F5),
            "G5" => Ok(Square64::G5),
            "H5" => Ok(Square64::H5),
            "A6" => Ok(Square64::A6),
            "B6" => Ok(Square64::B6),
            "C6" => Ok(Square64::C6),
            "D6" => Ok(Square64::D6),
            "E6" => Ok(Square64::E6),
            "F6" => Ok(Square64::F6),
            "G6" => Ok(Square64::G6),
            "H6" => Ok(Square64::H6),
            "A7" => Ok(Square64::A7),
            "B7" => Ok(Square64::B7),
            "C7" => Ok(Square64::C7),
            "D7" => Ok(Square64::D7),
            "E7" => Ok(Square64::E7),
            "F7" => Ok(Square64::F7),
            "G7" => Ok(Square64::G7),
            "H7" => Ok(Square64::H7),
            "A8" => Ok(Square64::A8),
            "B8" => Ok(Square64::B8),
            "C8" => Ok(Square64::C8),
            "D8" => Ok(Square64::D8),
            "E8" => Ok(Square64::E8),
            "F8" => Ok(Square64::F8),
            "G8" => Ok(Square64::G8),
            "H8" => Ok(Square64::H8),
            _ => Err(Error::ParseSquare64FromStrError(square_name.to_string())),
        }
    }
}

// TODO: confirm that there isn't a good way to do this for any generic integer type
impl TryFrom<u8> for Square64 {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == Square64::A1 as u8 => Ok(Square64::A1),
            v if v == Square64::B1 as u8 => Ok(Square64::B1),
            v if v == Square64::C1 as u8 => Ok(Square64::C1),
            v if v == Square64::D1 as u8 => Ok(Square64::D1),
            v if v == Square64::E1 as u8 => Ok(Square64::E1),
            v if v == Square64::F1 as u8 => Ok(Square64::F1),
            v if v == Square64::G1 as u8 => Ok(Square64::G1),
            v if v == Square64::H1 as u8 => Ok(Square64::H1),
            v if v == Square64::A2 as u8 => Ok(Square64::A2),
            v if v == Square64::B2 as u8 => Ok(Square64::B2),
            v if v == Square64::C2 as u8 => Ok(Square64::C2),
            v if v == Square64::D2 as u8 => Ok(Square64::D2),
            v if v == Square64::E2 as u8 => Ok(Square64::E2),
            v if v == Square64::F2 as u8 => Ok(Square64::F2),
            v if v == Square64::G2 as u8 => Ok(Square64::G2),
            v if v == Square64::H2 as u8 => Ok(Square64::H2),
            v if v == Square64::A3 as u8 => Ok(Square64::A3),
            v if v == Square64::B3 as u8 => Ok(Square64::B3),
            v if v == Square64::C3 as u8 => Ok(Square64::C3),
            v if v == Square64::D3 as u8 => Ok(Square64::D3),
            v if v == Square64::E3 as u8 => Ok(Square64::E3),
            v if v == Square64::F3 as u8 => Ok(Square64::F3),
            v if v == Square64::G3 as u8 => Ok(Square64::G3),
            v if v == Square64::H3 as u8 => Ok(Square64::H3),
            v if v == Square64::A4 as u8 => Ok(Square64::A4),
            v if v == Square64::B4 as u8 => Ok(Square64::B4),
            v if v == Square64::C4 as u8 => Ok(Square64::C4),
            v if v == Square64::D4 as u8 => Ok(Square64::D4),
            v if v == Square64::E4 as u8 => Ok(Square64::E4),
            v if v == Square64::F4 as u8 => Ok(Square64::F4),
            v if v == Square64::G4 as u8 => Ok(Square64::G4),
            v if v == Square64::H4 as u8 => Ok(Square64::H4),
            v if v == Square64::A5 as u8 => Ok(Square64::A5),
            v if v == Square64::B5 as u8 => Ok(Square64::B5),
            v if v == Square64::C5 as u8 => Ok(Square64::C5),
            v if v == Square64::D5 as u8 => Ok(Square64::D5),
            v if v == Square64::E5 as u8 => Ok(Square64::E5),
            v if v == Square64::F5 as u8 => Ok(Square64::F5),
            v if v == Square64::G5 as u8 => Ok(Square64::G5),
            v if v == Square64::H5 as u8 => Ok(Square64::H5),
            v if v == Square64::A6 as u8 => Ok(Square64::A6),
            v if v == Square64::B6 as u8 => Ok(Square64::B6),
            v if v == Square64::C6 as u8 => Ok(Square64::C6),
            v if v == Square64::D6 as u8 => Ok(Square64::D6),
            v if v == Square64::E6 as u8 => Ok(Square64::E6),
            v if v == Square64::F6 as u8 => Ok(Square64::F6),
            v if v == Square64::G6 as u8 => Ok(Square64::G6),
            v if v == Square64::H6 as u8 => Ok(Square64::H6),
            v if v == Square64::A7 as u8 => Ok(Square64::A7),
            v if v == Square64::B7 as u8 => Ok(Square64::B7),
            v if v == Square64::C7 as u8 => Ok(Square64::C7),
            v if v == Square64::D7 as u8 => Ok(Square64::D7),
            v if v == Square64::E7 as u8 => Ok(Square64::E7),
            v if v == Square64::F7 as u8 => Ok(Square64::F7),
            v if v == Square64::G7 as u8 => Ok(Square64::G7),
            v if v == Square64::H7 as u8 => Ok(Square64::H7),
            v if v == Square64::A8 as u8 => Ok(Square64::A8),
            v if v == Square64::B8 as u8 => Ok(Square64::B8),
            v if v == Square64::C8 as u8 => Ok(Square64::C8),
            v if v == Square64::D8 as u8 => Ok(Square64::D8),
            v if v == Square64::E8 as u8 => Ok(Square64::E8),
            v if v == Square64::F8 as u8 => Ok(Square64::F8),
            v if v == Square64::G8 as u8 => Ok(Square64::G8),
            v if v == Square64::H8 as u8 => Ok(Square64::H8),
            _ => Err(Error::ParseSquare64FromU8Error(value)),
        }
    }
}

impl Square64 {
    // TODO: figure out if this can be made generic to accept Enum or u8
    pub fn from_file_and_rank(file: File, rank: Rank) -> Option<Self> {
        if (rank as u8 | file as u8) >> 3 == 0 {
            let index_64 = (file as u8) + (rank as u8) * 8;
            match index_64.try_into() {
                Ok(square_64) => Some(square_64),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn from_file_and_rank_u8(file: u8, rank: u8) -> Option<Self> {
        if (rank | file) >> 3 == 0 {
            let index_64 = file + rank * 8;
            match index_64.try_into() {
                Ok(square_64) => Some(square_64),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn get_file(&self) -> Result<File, Error> {
        match FILES_BOARD[*self as usize] {
            Some(file) => Ok(file),
            None => Err(Error::Square64OnInvalidFile(*self)),
        }
    }

    pub fn get_rank(&self) -> Result<Rank, Error> {
        match RANKS_BOARD[*self as usize] {
            Some(rank) => Ok(rank),
            None => Err(Error::Square64OnInvalidRank(*self)),
        }
    }
}

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq)]
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
            .expect("Conversion from 8x8 Square to 10x12 Square failed")
    }
}

impl TryFrom<BitBoard> for Square {
    type Error = Error;

    fn try_from(value: BitBoard) -> Result<Self, Self::Error> {
        let v: u64 = value.into();
        match v {
            v if v == Square::A1 as u64 => Ok(Square::A1),
            v if v == Square::B1 as u64 => Ok(Square::B1),
            v if v == Square::C1 as u64 => Ok(Square::C1),
            v if v == Square::D1 as u64 => Ok(Square::D1),
            v if v == Square::E1 as u64 => Ok(Square::E1),
            v if v == Square::F1 as u64 => Ok(Square::F1),
            v if v == Square::G1 as u64 => Ok(Square::G1),
            v if v == Square::H1 as u64 => Ok(Square::H1),
            v if v == Square::A2 as u64 => Ok(Square::A2),
            v if v == Square::B2 as u64 => Ok(Square::B2),
            v if v == Square::C2 as u64 => Ok(Square::C2),
            v if v == Square::D2 as u64 => Ok(Square::D2),
            v if v == Square::E2 as u64 => Ok(Square::E2),
            v if v == Square::F2 as u64 => Ok(Square::F2),
            v if v == Square::G2 as u64 => Ok(Square::G2),
            v if v == Square::H2 as u64 => Ok(Square::H2),
            v if v == Square::A3 as u64 => Ok(Square::A3),
            v if v == Square::B3 as u64 => Ok(Square::B3),
            v if v == Square::C3 as u64 => Ok(Square::C3),
            v if v == Square::D3 as u64 => Ok(Square::D3),
            v if v == Square::E3 as u64 => Ok(Square::E3),
            v if v == Square::F3 as u64 => Ok(Square::F3),
            v if v == Square::G3 as u64 => Ok(Square::G3),
            v if v == Square::H3 as u64 => Ok(Square::H3),
            v if v == Square::A4 as u64 => Ok(Square::A4),
            v if v == Square::B4 as u64 => Ok(Square::B4),
            v if v == Square::C4 as u64 => Ok(Square::C4),
            v if v == Square::D4 as u64 => Ok(Square::D4),
            v if v == Square::E4 as u64 => Ok(Square::E4),
            v if v == Square::F4 as u64 => Ok(Square::F4),
            v if v == Square::G4 as u64 => Ok(Square::G4),
            v if v == Square::H4 as u64 => Ok(Square::H4),
            v if v == Square::A5 as u64 => Ok(Square::A5),
            v if v == Square::B5 as u64 => Ok(Square::B5),
            v if v == Square::C5 as u64 => Ok(Square::C5),
            v if v == Square::D5 as u64 => Ok(Square::D5),
            v if v == Square::E5 as u64 => Ok(Square::E5),
            v if v == Square::F5 as u64 => Ok(Square::F5),
            v if v == Square::G5 as u64 => Ok(Square::G5),
            v if v == Square::H5 as u64 => Ok(Square::H5),
            v if v == Square::A6 as u64 => Ok(Square::A6),
            v if v == Square::B6 as u64 => Ok(Square::B6),
            v if v == Square::C6 as u64 => Ok(Square::C6),
            v if v == Square::D6 as u64 => Ok(Square::D6),
            v if v == Square::E6 as u64 => Ok(Square::E6),
            v if v == Square::F6 as u64 => Ok(Square::F6),
            v if v == Square::G6 as u64 => Ok(Square::G6),
            v if v == Square::H6 as u64 => Ok(Square::H6),
            v if v == Square::A7 as u64 => Ok(Square::A7),
            v if v == Square::B7 as u64 => Ok(Square::B7),
            v if v == Square::C7 as u64 => Ok(Square::C7),
            v if v == Square::D7 as u64 => Ok(Square::D7),
            v if v == Square::E7 as u64 => Ok(Square::E7),
            v if v == Square::F7 as u64 => Ok(Square::F7),
            v if v == Square::G7 as u64 => Ok(Square::G7),
            v if v == Square::H7 as u64 => Ok(Square::H7),
            v if v == Square::A8 as u64 => Ok(Square::A8),
            v if v == Square::B8 as u64 => Ok(Square::B8),
            v if v == Square::C8 as u64 => Ok(Square::C8),
            v if v == Square::D8 as u64 => Ok(Square::D8),
            v if v == Square::E8 as u64 => Ok(Square::E8),
            v if v == Square::F8 as u64 => Ok(Square::F8),
            v if v == Square::G8 as u64 => Ok(Square::G8),
            v if v == Square::H8 as u64 => Ok(Square::H8),
            _ => Err(Error::ParseSquareFromBitBoardError(value)),
        }
    }
}

impl TryFrom<&str> for Square {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "A1" => Ok(Square::A1),
            "B1" => Ok(Square::B1),
            "C1" => Ok(Square::C1),
            "D1" => Ok(Square::D1),
            "E1" => Ok(Square::E1),
            "F1" => Ok(Square::F1),
            "G1" => Ok(Square::G1),
            "A2" => Ok(Square::A2),
            "B2" => Ok(Square::B2),
            "C2" => Ok(Square::C2),
            "D2" => Ok(Square::D2),
            "E2" => Ok(Square::E2),
            "F2" => Ok(Square::F2),
            "G2" => Ok(Square::G2),
            "A3" => Ok(Square::A3),
            "B3" => Ok(Square::B3),
            "C3" => Ok(Square::C3),
            "D3" => Ok(Square::D3),
            "E3" => Ok(Square::E3),
            "F3" => Ok(Square::F3),
            "G3" => Ok(Square::G3),
            "A4" => Ok(Square::A4),
            "B4" => Ok(Square::B4),
            "C4" => Ok(Square::C4),
            "D4" => Ok(Square::D4),
            "E4" => Ok(Square::E4),
            "F4" => Ok(Square::F4),
            "G4" => Ok(Square::G4),
            "A5" => Ok(Square::A5),
            "B5" => Ok(Square::B5),
            "C5" => Ok(Square::C5),
            "D5" => Ok(Square::D5),
            "E5" => Ok(Square::E5),
            "F5" => Ok(Square::F5),
            "G5" => Ok(Square::G5),
            "A6" => Ok(Square::A6),
            "B6" => Ok(Square::B6),
            "C6" => Ok(Square::C6),
            "D6" => Ok(Square::D6),
            "E6" => Ok(Square::E6),
            "F6" => Ok(Square::F6),
            "G6" => Ok(Square::G6),
            "A7" => Ok(Square::A7),
            "B7" => Ok(Square::B7),
            "C7" => Ok(Square::C7),
            "D7" => Ok(Square::D7),
            "E7" => Ok(Square::E7),
            "F7" => Ok(Square::F7),
            "G7" => Ok(Square::G7),
            "A8" => Ok(Square::A8),
            "B8" => Ok(Square::B8),
            "C8" => Ok(Square::C8),
            "D8" => Ok(Square::D8),
            "E8" => Ok(Square::E8),
            "F8" => Ok(Square::F8),
            "G8" => Ok(Square::G8),
            _ => Err(Error::ParseSquareFromStrError(value.to_string())),
        }
    }
}

impl TryFrom<u8> for Square {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == Square::A1 as u8 => Ok(Square::A1),
            v if v == Square::B1 as u8 => Ok(Square::B1),
            v if v == Square::C1 as u8 => Ok(Square::C1),
            v if v == Square::D1 as u8 => Ok(Square::D1),
            v if v == Square::E1 as u8 => Ok(Square::E1),
            v if v == Square::F1 as u8 => Ok(Square::F1),
            v if v == Square::G1 as u8 => Ok(Square::G1),
            v if v == Square::H1 as u8 => Ok(Square::H1),
            v if v == Square::A2 as u8 => Ok(Square::A2),
            v if v == Square::B2 as u8 => Ok(Square::B2),
            v if v == Square::C2 as u8 => Ok(Square::C2),
            v if v == Square::D2 as u8 => Ok(Square::D2),
            v if v == Square::E2 as u8 => Ok(Square::E2),
            v if v == Square::F2 as u8 => Ok(Square::F2),
            v if v == Square::G2 as u8 => Ok(Square::G2),
            v if v == Square::H2 as u8 => Ok(Square::H2),
            v if v == Square::A3 as u8 => Ok(Square::A3),
            v if v == Square::B3 as u8 => Ok(Square::B3),
            v if v == Square::C3 as u8 => Ok(Square::C3),
            v if v == Square::D3 as u8 => Ok(Square::D3),
            v if v == Square::E3 as u8 => Ok(Square::E3),
            v if v == Square::F3 as u8 => Ok(Square::F3),
            v if v == Square::G3 as u8 => Ok(Square::G3),
            v if v == Square::H3 as u8 => Ok(Square::H3),
            v if v == Square::A4 as u8 => Ok(Square::A4),
            v if v == Square::B4 as u8 => Ok(Square::B4),
            v if v == Square::C4 as u8 => Ok(Square::C4),
            v if v == Square::D4 as u8 => Ok(Square::D4),
            v if v == Square::E4 as u8 => Ok(Square::E4),
            v if v == Square::F4 as u8 => Ok(Square::F4),
            v if v == Square::G4 as u8 => Ok(Square::G4),
            v if v == Square::H4 as u8 => Ok(Square::H4),
            v if v == Square::A5 as u8 => Ok(Square::A5),
            v if v == Square::B5 as u8 => Ok(Square::B5),
            v if v == Square::C5 as u8 => Ok(Square::C5),
            v if v == Square::D5 as u8 => Ok(Square::D5),
            v if v == Square::E5 as u8 => Ok(Square::E5),
            v if v == Square::F5 as u8 => Ok(Square::F5),
            v if v == Square::G5 as u8 => Ok(Square::G5),
            v if v == Square::H5 as u8 => Ok(Square::H5),
            v if v == Square::A6 as u8 => Ok(Square::A6),
            v if v == Square::B6 as u8 => Ok(Square::B6),
            v if v == Square::C6 as u8 => Ok(Square::C6),
            v if v == Square::D6 as u8 => Ok(Square::D6),
            v if v == Square::E6 as u8 => Ok(Square::E6),
            v if v == Square::F6 as u8 => Ok(Square::F6),
            v if v == Square::G6 as u8 => Ok(Square::G6),
            v if v == Square::H6 as u8 => Ok(Square::H6),
            v if v == Square::A7 as u8 => Ok(Square::A7),
            v if v == Square::B7 as u8 => Ok(Square::B7),
            v if v == Square::C7 as u8 => Ok(Square::C7),
            v if v == Square::D7 as u8 => Ok(Square::D7),
            v if v == Square::E7 as u8 => Ok(Square::E7),
            v if v == Square::F7 as u8 => Ok(Square::F7),
            v if v == Square::G7 as u8 => Ok(Square::G7),
            v if v == Square::H7 as u8 => Ok(Square::H7),
            v if v == Square::A8 as u8 => Ok(Square::A8),
            v if v == Square::B8 as u8 => Ok(Square::B8),
            v if v == Square::C8 as u8 => Ok(Square::C8),
            v if v == Square::D8 as u8 => Ok(Square::D8),
            v if v == Square::E8 as u8 => Ok(Square::E8),
            v if v == Square::F8 as u8 => Ok(Square::F8),
            v if v == Square::G8 as u8 => Ok(Square::G8),
            v if v == Square::H8 as u8 => Ok(Square::H8),
            _ => Err(Error::ParseSquareFromU8Error(value)),
        }
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
        let output = Square::try_from(input).unwrap();
        let expected = Square::D2;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_120_try_into_bitboard_valid() {
        let input = BitBoard(34);
        let output: Square = input.try_into().unwrap();
        let expected = Square::D2;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_square_120_try_from_bitboard_invalid() {
        let input = BitBoard(11);
        let output = Square::try_from(input).unwrap();
    }

    #[should_panic]
    #[test]
    fn test_square_120_try_into_bitboard_invalid() {
        let input = BitBoard(0);
        let output: Square = input.try_into().unwrap();
    }

    #[test]
    fn test_square_64_try_from_bitboard_valid() {
        let input = BitBoard(34);
        let output = Square64::try_from(input).unwrap();
        let expected = Square64::C5;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_64_try_into_bitboard_valid() {
        let input = BitBoard(34);
        let output: Square64 = input.try_into().unwrap();
        let expected = Square64::C5;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_square_64_try_from_bitboard_invalid() {
        let input = BitBoard(64);
        let output = Square64::try_from(input).unwrap();
    }

    #[should_panic]
    #[test]
    fn test_square_64_try_into_bitboard_invalid() {
        let input = BitBoard(64);
        let output: Square64 = input.try_into().unwrap();
    }

    #[test]
    fn test_square_120_try_from_str_valid() {
        let input = "D2";
        let output = Square::try_from(input).unwrap();
        let expected = Square::D2;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_square_120_try_from_str_invalid() {
        let input = "INVALID";
        let output = Square::try_from(input).unwrap();
        let expected = Square::D2;
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
        let output = Square64::try_from(input).unwrap();
        let expected = Square64::D2;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_square_64_try_from_str_invalid() {
        let input = "INVALID";
        let output = Square64::try_from(input).unwrap();
        let expected = Square64::D2;
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
        let output: Square = Square::try_from(input).unwrap();
        let expected = Square::D2;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_square_120_try_from_u8_invalid() {
        let input = 11;
        let output: Square = Square::try_from(input).unwrap();
    }

    #[test]
    fn test_square_120_try_into_u8_valid() {
        let input: u8 = 34;
        let output: Square = input.try_into().unwrap();
        let expected = Square::D2;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_square_120_try_into_u8_invalid() {
        let input: u8 = 11;
        let output: Square = input.try_into().unwrap();
    }

    #[test]
    fn test_square_64_try_from_u8_valid() {
        let input: u8 = 34;
        let output: Square64 = Square64::try_from(input).unwrap();
        let expected = Square64::C5;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_square_64_try_from_u8_invalid() {
        let input = 64;
        let output: Square64 = Square64::try_from(input).unwrap();
    }

    #[test]
    fn test_square_64_try_into_u8_valid() {
        let input: u8 = 34;
        let output: Square64 = input.try_into().unwrap();
        let expected = Square64::C5;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_square_64_try_into_u8_invalid() {
        let input: u8 = 64;
        let output: Square64 = input.try_into().unwrap();
    }


    // Other methods

    #[test]
    fn test_from_file_and_rank_u8_valid() {
        let square = Square::from_file_and_rank_u8(1, 2);
        assert_eq!(square, Some(Square::B3));

        let square = Square::from_file_and_rank_u8(7, 7);
        assert_eq!(square, Some(Square::H8));
    }

    #[test]
    fn test_from_file_and_rank_valid() {
        let square = Square::from_file_and_rank(File::FileB, Rank::Rank3);
        assert_eq!(square, Some(Square::B3));

        let square = Square::from_file_and_rank(File::FileH, Rank::Rank8);
        assert_eq!(square, Some(Square::H8));
    }

    #[test]
    fn test_from_file_and_rank_u8_invalid() {
        let square = Square::from_file_and_rank_u8(0, 8);
        assert_eq!(square, None);

        let square = Square::from_file_and_rank_u8(8, 0);
        assert_eq!(square, None);
    }

    #[test]
    fn test_get_file() {
        let output = Square::H7.get_file().unwrap();
        let expected = File::FileH;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank() {
        let output = Square::H7.get_rank().unwrap();
        let expected = Rank::Rank7;
        assert_eq!(output, expected);
    }
}
