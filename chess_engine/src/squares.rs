use std::fmt;
use std::str::FromStr;
use crate::error::ChessError as Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Square {
    A1 = 21, B1, C1, D1, E1, F1, G1, H1,
    A2 = 31, B2, C2, D2, E2, F2, G2, H2,
    A3 = 41, B3, C3, D3, E3, F3, G3, H3,
    A4 = 51, B4, C4, D4, E4, F4, G4, H4,
    A5 = 61, B5, C5, D5, E5, F5, G5, H5,
    A6 = 71, B6, C6, D6, E6, F6, G6, H6,
    A7 = 81, B7, C7, D7, E7, F7, G7, H7,
    A8 = 91, B8, C8, D8, E8, F8, G8, H8, OffBoard,
}

// TODO: Clean this up when you have equivalent features
const SQUARE_NAMES: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

impl FromStr for Square {
    type Err = Error;
    
    fn from_str(square_name: &str) -> Result<Self, Self::Err> {
        match square_name {
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
            "OffBoard" => Ok(Square::OffBoard),
            _ => Err(Error::ParseSquareError(square_name.to_string())),
        }
    }
}

impl Square {
    pub fn from_file_and_rank(file: u8, rank: u8) -> Option<Self> {
        todo!()
    }
}


// #[derive(Debug, Copy, Clone, PartialEq, Eq)]
// pub struct Square(usize);

// impl Square {
//     pub fn from_name(name: &str) -> Option<Self> {
//         // Returns the index position of square name.

//         SQUARE_NAMES.iter().position(|&x| x == name).map(Self)
//     }

//     pub fn from_file_and_rank(file_: u8, rank: u8) -> Option<Self> {
//         // Returns square with given file and rank
//         // file_: ranged from 0-7, where 0 == a_file, 7 == h_file, etc
//         // rank: ranged from 0-7, where 0 == 1st_rank, 7 == 8th_rank, etc
//         if (rank | file_) >> 3 == 0 {
//             let u = ((rank << 3) | file_) as usize;
//             Some(Self(u))
//         } else {
//             None
//         }
//     }

//     pub const fn name(&self) -> &str {
//         // Returns indexed square
//         SQUARE_NAMES[self.0]
//     }

//     pub const fn file(&self) -> u8 {
//         // Get file_index (a_file == 0, h_file == 7, etc)
//         self.0 as u8 & 7
//     }

//     pub const fn rank(&self) -> u8 {
//         // Get rank_index (1st_rank == 0, 8th_rank == 7, etc)
//         self.0 as u8 >> 3
//     }
// }

// impl fmt::Display for Square {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{}", self.name())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_and_from_name() {
        let ref_string = "b3";
        let square: Square = Square::from_name(ref_string).unwrap();
        assert_eq!(square, Square(17));
        let output_string = square.name();
        assert_eq!(ref_string, output_string);
    }

    #[test]
    fn test_from_file_and_rank_valid() {
        let square = Square::from_file_and_rank(1, 2);
        assert_eq!(square, Some(Square(17)));
        let square = Square::from_file_and_rank(7, 7);
        assert_eq!(square, Some(Square(63)));
    }

    #[test]
    fn test_from_file_and_rank_invalid() {
        let square = Square::from_file_and_rank(0, 8);
        assert_eq!(square, None);

        let square = Square::from_file_and_rank(8, 0);
        assert_eq!(square, None);
    }

    #[test]
    fn test_file_rank_name_getters() {
        let square = Square::from_file_and_rank(7, 2).unwrap();
        assert_eq!(square.file(), 7);
        assert_eq!(square.rank(), 2);
        assert_eq!(square.name(), "h3");
    }
}
