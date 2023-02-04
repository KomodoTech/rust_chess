use std::{char::MAX, default};

use rand::{random, thread_rng, Rng};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, EnumString};

use crate::{
    board::Board,
    error::ChessError as Error,
    pieces::Piece,
    squares::Square,
    util::{Color, NUM_BOARD_SQUARES, NUM_CASTLE_PERM},
};

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const MAX_GAME_MOVES: usize = 2048;

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumIter, EnumString, Display)]
enum Castle {
    WhiteKing = 1,
    WhiteQueen = 2,
    BlackKing = 4,
    BlackQueen = 8,
}

#[derive(Debug, Clone, Copy)]
struct CastlePerm {
    white_king: Option<Castle>,
    white_queen: Option<Castle>,
    black_king: Option<Castle>,
    black_queen: Option<Castle>,
}

impl Default for CastlePerm {
    fn default() -> Self {
        Self {
            white_king: Some(Castle::WhiteKing),
            white_queen: Some(Castle::WhiteQueen),
            black_king: Some(Castle::BlackKing),
            black_queen: Some(Castle::BlackQueen),
        }
    }
}

// TODO: test and test that try_into works as expected
impl TryFrom<u8> for CastlePerm {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v <= 0x0F => {
                // NOTE: default castle_perm are all some/set
                let mut castle_perm = Self::default();
                for castle in Castle::iter() {
                    // check if bit corresponding to Castle permission is not set
                    if ((v & (castle as u8)) != v) {
                        let castle_variant = castle.to_string();
                        match castle_variant.as_str() {
                            "WhiteKing" => {
                                castle_perm.white_king = None;
                            }
                            "WhiteQueen" => {
                                castle_perm.white_queen = None;
                            }
                            "BlackKing" => {
                                castle_perm.black_king = None;
                            }
                            "BlackQueen" => {
                                castle_perm.black_queen = None;
                            }
                            _ => {
                                return Err(Error::ParseCastlePermFromU8ErrorInvalidCastleString(
                                    value,
                                    castle_variant,
                                ));
                            }
                        }
                    }
                }
                todo!()
            }
            _ => Err(Error::ParseCastlePermFromU8ErrorValueTooLarge(value)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Undo {
    move_: u32,
    castle_permissions: CastlePerm,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    position_key: u64,
}

#[derive(Debug)]
struct Zobrist {
    color_key: u64,
    piece_keys: [[u64; Piece::COUNT]; NUM_BOARD_SQUARES],
    en_passant_keys: [u64; NUM_BOARD_SQUARES],
    castle_keys: [u64; NUM_CASTLE_PERM],
}

impl Zobrist {
    // TODO: revisit collisions and RNG (seed for testing)
    // NOTE: https://craftychess.com/hyatt/collisions.html
    fn new() -> Self {
        let color_key: u64 = random();
        // TODO: couldn't find a way to initialize them with random numbers directly
        let mut piece_keys = [[0u64; Piece::COUNT]; NUM_BOARD_SQUARES];
        for mut square_array in piece_keys {
            thread_rng().fill(&mut square_array[..]);
        }
        let mut en_passant_keys = [0u64; NUM_BOARD_SQUARES];
        thread_rng().fill(&mut en_passant_keys[..]);
        let mut castle_keys = [0u64; NUM_CASTLE_PERM];
        thread_rng().fill(&mut castle_keys[..]);

        Zobrist {
            color_key,
            piece_keys,
            en_passant_keys,
            castle_keys,
        }
    }
}

// TODO: make Zobrist generate at compile time with proc macro
#[derive(Debug)]
pub struct Gamestate {
    board: Board,
    active_color: Color,
    castle_permissions: u32,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    fullmove_number: u32,
    history: [Option<Undo>; MAX_GAME_MOVES],
    zobrist: Zobrist,
}

impl Default for Gamestate {
    fn default() -> Self {
        Gamestate::new()
    }
}

impl Gamestate {
    pub fn new() -> Self {
        let board = Board::new();
        // TODO: call init on board with starting FEN
        let active_color = Color::White;
        let castle_permissions: u32 = 0b0000_0000_0000_0000_0000_0000_0000_1111;
        // TODO: figure out what this should really be initially
        let en_passant: Option<Square> = None;
        let halfmove_clock: u32 = 0;
        let fullmove_number: u32 = 0;
        let history: [Option<Undo>; MAX_GAME_MOVES] = [None; MAX_GAME_MOVES];
        let zobrist = Zobrist::new();

        Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        }
    }

    fn gen_position_key(&self) -> u64 {
        let mut position_key: u64 = 0;

        // Piece location component
        for (square_index, piece_at_square) in self.board.pieces.iter().enumerate() {
            if let Some(piece) = *piece_at_square {
                position_key ^= self.zobrist.piece_keys[piece as usize][square_index];
            }
        }
        // Color (which player's turn) component
        if self.active_color == Color::White {
            position_key ^= self.zobrist.color_key
        };
        // En Passant component
        if let Some(square) = self.en_passant {
            position_key ^= self.zobrist.en_passant_keys[square as usize];
        }
        // Castle Permissions component
        let castle_permissions: u8 = self.castle_permissions.try_into().expect("castle_permissions should be in the range 0..=15");
        position_key ^= self.zobrist.castle_keys[castle_permissions as usize];

        position_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    // #[test]
    // fn test_gen_position_key() {
    //     let gamestate = Gamestate::default();
    //     let output = gamestate.gen_position_key();
    //     let expected = 0;
    //     assert_eq!(output, expected);
    // }
}
