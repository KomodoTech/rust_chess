use crate::{
    file::File,
    rank::Rank,
    square::{Square, Square64},
    square::{SQUARE_120_TO_64, SQUARE_64_TO_120},
};
use std::{fmt, ops::BitAnd};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Least significant bit is A1, and most significant bit is H8:
///     a  b  c  d  e  f  g  h
///   -------------------------
/// 8 | 56 57 58 59 60 61 62 63
/// 7 | 48 49 50 51 52 53 54 55
/// 6 | 40 41 42 43 44 45 46 47
/// 5 | 32 33 34 35 36 37 38 39
/// 4 | 24 25 26 27 28 29 30 31
/// 3 | 16 17 18 19 20 21 22 23
/// 2 | 8  9  10 11 12 13 14 15
/// 1 | 0  1  2  3  4  5  6  7
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitBoard(pub u64);

// https://stackoverflow.com/questions/30680559/how-to-find-magic-bitboards
// TODO: generate own Magic Bitboard and implement
// const BIT_TABLE: [Square; NUM_EXTERNAL_BOARD_SQUARES = [
//     Square::H8, Square::G4, Square::D1, Square::A5, Square::B4, Square::B6, Square::G3, Square::B5,
//     Square::H2, Square::C7, Square::C6, Square::F2, Square::D2, Square::F7, Square::D3, Square::C5,
//     Square::F8, Square::F4, Square::C1, Square::D7, Square::F3, Square::D6, Square::F6, Square::C2,
//     Square::C3, Square::H6, Square::B1, Square::G7, Square::B2, Square::B8, Square::A1, Square::D5,
//     Square::G8, Square::H4, Square::A6, Square::E1, Square::B7, Square::F1, Square::E7, Square::C4,
//     Square::E8, Square::G1, Square::H3, Square::E6, Square::G6, Square::D4, Square::A8, Square::A3,
//     Square::H1, Square::H5, Square::A7, Square::A4, Square::D8, Square::G2, Square::E2, Square::H7,
//     Square::G5, Square::E4, Square::C8, Square::E3, Square::F5, Square::B3, Square::E5, Square::A2
// ];

impl BitBoard {
    /// Counts number of set bits
    pub fn count_bits(&self) -> u8 {
        // NOTE: not sure how count_ones is implemented, but these are some useful resources
        // divide and conquer: https://www.youtube.com/watch?v=ZusiKXcz_ac
        // https://arxiv.org/pdf/1611.07612.pdf
        self.0.count_ones() as u8
        // let mut count: u8 = 0;
        // let mut b = self.0;
        // while b > 0 {
        //     count += 1;
        //     // converts the current least significant 1 into 0111... with the -1
        //     // then removes trailing 1s into 0s with the & (1000 & 0111 = 0000)
        //     b &= b - 1;
        // }
        // count
    }

    /// Sets the first set LSB to 0 and returns the index corresponding to it
    // NOTE: this is slow in comparison to magic bitboard implementation which
    // has a very real effect on performance of move generation and thus on bot ability
    fn pop_bit(&mut self) -> Option<Square64> {
        let lsb_index = self.0.trailing_zeros();
        match lsb_index {
            // all zeros
            64 => None,
            _ => {
                let mask: u64 = 1 << lsb_index;
                self.0 ^= mask;
                Some(
                    lsb_index
                        .try_into()
                        .expect("lsb_index should be in range 0..=63"),
                )
            }
        }
    }

    // TODO: implement magic bitboard version
    // // Relies on Magic BitBoard (see BIT_TABLE for more information)
    // fn pop_bit(&mut self) -> Square {
    //     let mut b = self.0 ^ (self.0 - 1);
    //     let fold = (b & 0xFF_FF_FF_FF) ^ (b >> 32);
    //     self.0 &= self.0 - 1;
    //     BIT_TABLE[((fold * 0x783a9b23) >> 26) as usize]
    // }

    /// Check if bit at index is set
    pub fn check_bit(&self, square: Square64) -> bool {
        self.0 & (1 << (square as u8)) != 0
    }

    /// Sets bit at index
    pub fn set_bit(&mut self, square: Square64) {
        self.0 |= 1 << (square as u8);
    }

    /// Sets bit at index to 0
    pub fn unset_bit(&mut self, square: Square64) {
        // XOR will toggle value at index so we should only call it
        // if the bit at index was already set
        if self.check_bit(square) {
            self.0 ^= 1 << (square as u8);
        }
    }
}

impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<BitBoard> for u64 {
    fn from(value: BitBoard) -> Self {
        value.0
    }
}

// impl BitAnd for BitBoard {
//     type Output = Self;

//     fn bitand(self, rhs: Self) -> Self::Output {
//         Self(self.0 & rhs.0)
//     }
// }

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in Rank::iter() {
            for file in File::iter() {
                let square_64 = Square64::from_file_and_rank(file, rank);
                match self.check_bit(square_64) {
                    true => {
                        write!(f, "1");
                    }
                    _ => {
                        write!(f, "0");
                    }
                }
            }
            if rank != Rank::Rank8 {
                writeln!(f);
            }
        }
        writeln!(f)
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

    #[test]
    fn test_count_bits_starting_white_pawn_position() {
        let input = BitBoard(0xFF00);
        let output = input.count_bits();
        let expected: u8 = 8;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_count_bits_empty() {
        let input = BitBoard(0);
        let output = input.count_bits();
        let expected: u8 = 0;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_check_set_bit() {
        let index = Square64::A2;
        let input = BitBoard(0x00_00_00_00_00_00_01_00);
        let output = input.check_bit(index);
        let expected = true;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_check_non_set_bit() {
        let index = Square64::A2;
        let input = BitBoard(0x00_0F_00_00_00_00_00_00);
        let output = input.check_bit(index);
        let expected = false;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_set_bit() {
        let index = Square64::A2;
        let mut input = BitBoard(0);
        input.set_bit(index);
        let output = input.0;
        let expected =
            0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_unset_set_bit() {
        let index = Square64::A2;
        let mut input = BitBoard(0x00_00_00_00_00_00_01_00);
        input.unset_bit(index);
        let output = input.0;
        let expected = 0;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_unset_non_set_bit() {
        let index = Square64::A2;
        let mut input = BitBoard(0x00_0F_00_00_00_00_00_00);
        input.unset_bit(index);
        let output = input.0;
        let expected = 0x00_0F_00_00_00_00_00_00;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_pop_bit_single_set_bit() {
        let mut input = BitBoard(0x80_00_00_00_00_00_00_00);
        let output = input.pop_bit();
        let expected_index = Some(Square64::H8);
        let expected_board = BitBoard(0);
        assert_eq!(output, expected_index);
        assert_eq!(input, expected_board);
    }

    #[test]
    fn test_pop_bit_multiple_set_bit() {
        let mut input = BitBoard(0x0C_0F_00_D0_00_00_01_00);
        let output = input.pop_bit();
        let expected_index = Some(Square64::A2);
        let expected_board = BitBoard(0x0C_0F_00_D0_00_00_00_00);
        assert_eq!(output, expected_index);
        assert_eq!(input, expected_board);
    }

    #[test]
    fn test_pop_bit_empty_board() {
        let mut input = BitBoard(0);
        let output = input.pop_bit();
        let expected = None;
        assert_eq!(output, expected);
    }
}
