use rand::{random, thread_rng, Rng};
use std::{char::MAX, default, fmt};
use strum::EnumCount;
use strum_macros::{Display as EnumDisplay, EnumCount as EnumCountMacro};

use crate::{
    board::Board,
    castle_perms::{CastlePerm, NUM_CASTLE_PERM},
    error::{ConversionError, FENParseError},
    pieces::Piece,
    squares::Square,
    util::{Color, NUM_BOARD_SQUARES, NUM_FEN_SECTIONS},
};

// CONSTANTS:

const MAX_GAME_MOVES: usize = 2048;
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

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
    castle_permissions: CastlePerm,
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
        let castle_permissions = CastlePerm::default();
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

    // Check each side has exactly a king
    // each piece type is valid
    // each row has 8 squares
    // no check for max amount of piece type leq 10
    // optionally at the end check if active color can win in one move and disallow
    // checking castle rights actually match the board
    pub fn new_from_fen(fen: &str) -> Result<Self, FENParseError> {
        let fen_sections: Vec<&str> = fen.trim().split(' ').collect();

        match fen_sections.len() {
            len if len == NUM_FEN_SECTIONS => {
                let mut active_color: Color = Color::White;
                match fen_sections[1] {
                    white if white == Color::White.to_string().as_str() => {}
                    black if black == Color::Black.to_string().as_str() => {
                        active_color = Color::Black;
                    }
                    _ => {
                        return Err(FENParseError::ActiveColorInvalid(
                            fen_sections[1].to_string(),
                        ));
                    }
                }

                // TODO: look into X-FEN and Shredder-FEN for Chess960
                // add values given lookup in array at that sum index
                // let castle_perm = CastlePerm::try_from(fen_sections[2]);
                // match castle_perm {
                //     Ok(cp) => {todo!()},
                //     Err(_) => {todo!()}
                // }
                todo!()
                // en passant
                // halfmove clock
                // fullmove number

                // board
            }
            _ => {
                return Err(FENParseError::WrongNumFENSections(fen_sections.len()));
            }
        }

        // send the first substring to the board to validate passing in the board as well
        // if the board succeeds in parsing the base FEN, then it will pass back the reference to the board in an Ok
        // you can then commit the local values for the color, en passant, castling rights, moves, etc
        // to the gamestate
        // otherwise deal with errors
        todo!()
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
        let castle_permissions: u8 = self.castle_permissions.into();
        position_key ^= self.zobrist.castle_keys[castle_permissions as usize];

        position_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_position_key_deterministic() {
        let gamestate = Gamestate::default();
        let output = gamestate.gen_position_key();
        let expected = gamestate.gen_position_key();
        println!("output: {}, expected: {}", output, expected);
        assert_eq!(output, expected);
    }

    // TODO: properly seed and test Zobrist key gen to check for collision rate in norm
}
