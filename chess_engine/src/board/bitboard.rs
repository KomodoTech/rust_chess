use std::fmt;
use crate::{
    squares::{Square, Square64},
    util::{
        Rank,
        File,
        SQUARE_120_TO_64,
        SQUARE_64_TO_120,
    },
};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub const BB_RANK_: u64 = 255;

// TODO: figure out if this is optimal for x86 or should be flipped
// LSB is A1, MSB H8
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitBoard(u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shifter: u64 = 0x1;

        for rank in Rank::iter() {
            for file in File::iter() {
                let square_64 = Square64::from_file_and_rank(file, rank).expect("Could not create Square from given file and rank.");
                match shifter << (square_64 as u8) & self.0 {
                    0 => { write!(f, "0"); },
                    _ => { write!(f, "1"); },
                }
            }
            write!(f, "\n");
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard_display() {
        let input = BitBoard(0xFF00);
        let output = input.to_string();
        let expected = 
            "00000000\n11111111\n00000000\n00000000\n00000000\n00000000\n00000000\n00000000\n";
        assert_eq!(output, expected);
    }
}