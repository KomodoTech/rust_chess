use std::fmt;

const SQUARE_NAMES: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square(usize);

impl Square {
    pub fn from_name(name: &str) -> Option<Self> {
        // Returns the index position of square name.
        SQUARE_NAMES.iter().position(|&x| x == name).map(Self)
    }

    pub fn from_file_and_rank(file_: u8, rank: u8) -> Option<Self> {
        // Returns square with given file and rank
        // file_: ranged from 0-7, where 0 == a_file, 7 == h_file, etc
        // rank: ranged from 0-7, where 0 == 1st_rank, 7 == 8th_rank, etc
        if (rank | file_) >> 3 == 0 {
            let u = (rank << 3) | file_;
            Some(Self(u as usize))
        } else {
            None
        }
    }

    pub fn name(&self) -> &str {
        // Returns indexed square
        SQUARE_NAMES[self.0]
    }

    pub fn file(&self) -> u8 {
        // Get file_index (a_file == 0, h_file == 7, etc)
        self.0 as u8 & 7
    }

    pub fn rank(&self) -> u8 {
        // Get rank_index (1st_rank == 0, 8th_rank == 7, etc)
        self.0 as u8 >> 3
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

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
