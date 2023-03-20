use crate::{
    castle_perm::{self, CastlePerm},
    color::Color,
    piece::Piece,
    square::Square64,
    zobrist::{Zobrist, ZOBRIST},
};
use std::fmt;

// TODO: make builder for PositionKey to allow building it up in pieces if performance allows it

/// Holds the Zobrist hashed key for the current Gamestate
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PositionKey(pub u64);

impl PositionKey {
    /// Hash in a random number stored in the Zobrist struct corresponding to
    /// when the active_color is White. When the active_color is Black, we
    /// hash in the same key, effectively zeroing it out which denotes Black
    /// is the active_color. Used when active player changes
    pub fn hash_color(&mut self) {
        let color_key = ZOBRIST
            .lock()
            .expect("Mutex holding ZOBRIST should not be poisoned")
            .color_key;

        self.0 ^= color_key;
    }

    /// Hash in a random number stored in the Zobrist struct corresponding to a
    /// Piece and a Square, to the PositionKey.
    /// Used when Gamestate is updated and a Piece gets added to or cleared
    /// from a Square to keep the PositionKey up to date
    pub fn hash_piece(&mut self, piece: Piece, square: Square64) {
        let piece_keys = ZOBRIST
            .lock()
            .expect("Mutex holding ZOBRIST should not be poisoned")
            .piece_keys;

        self.0 ^= piece_keys[piece as usize][square as usize];
    }

    /// Hash in a random number stored in the Zobrist struct corresponding to
    /// the en passant square. Only gets called when
    /// Used when Gamestate detects a change to its en_passant field
    pub fn hash_en_passant(&mut self, en_passant: Square64) {
        let en_passant_keys = ZOBRIST
            .lock()
            .expect("Mutex holding ZOBRIST should not be poisoned")
            .en_passant_keys;

        self.0 ^= en_passant_keys[en_passant.get_file() as usize];
    }

    /// Hash in a random number stored in the Zobrist struct corresponding to a
    /// set of Castle Permissions
    /// Used when Gamestate is updated and a move change the castling rights
    pub fn hash_castle_perm(&mut self, castle_perm: &CastlePerm) {
        let castle_keys = ZOBRIST
            .lock()
            .expect("Mutex holding ZOBRIST should not be poisoned")
            .castle_keys;

        self.0 ^= castle_keys[castle_perm.0 as usize];
    }
}

impl fmt::Display for PositionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Position: {}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_active_color_white() {
        // color_key is: 8209426193815160997 =
        // 0b_0111_0001_1110_1101_1011_1101_1010_0000_1011_1100_0010_0011_1100_1000_1010_0101
        let mut output = PositionKey(
            0b_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1011_1100_0010,
        );
        output.hash_color();

        let expected = PositionKey(
            0b_0111_0001_1110_1101_1011_1101_1010_0000_1011_1100_0010_0011_1100_0011_0110_0111,
        );

        assert_eq!(output, expected);
    }

    // NOTE: this better work, it's just showing that XOR is an involuntary function
    #[test]
    fn test_update_active_color_black() {
        // color_key is: 8209426193815160997 =
        // 0b_0111_0001_1110_1101_1011_1101_1010_0000_1011_1100_0010_0011_1100_1000_1010_0101
        let mut output = PositionKey(
            0b_0111_0001_1110_1101_1011_1101_1010_0000_1011_1100_0010_0011_1100_0011_0110_0111,
        );
        output.hash_color();

        let expected = PositionKey(
            0b_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1011_1100_0010,
        );

        assert_eq!(output, expected);
    }
}
