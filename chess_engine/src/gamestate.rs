use rand::{random, thread_rng, Rng};
use std::{char::MAX, default, fmt};
use strum::EnumCount;
use strum_macros::{Display as EnumDisplay, EnumCount as EnumCountMacro};

use crate::{
    board::Board,
    castle_perms::{self, CastlePerm, NUM_CASTLE_PERM},
    error::{ConversionError, FENParseError},
    pieces::Piece,
    squares::Square,
    util::Color,
};

// CONSTANTS:
/// Maximum number of half moves we expect
pub const MAX_GAME_MOVES: usize = 2048;
pub const NUM_FEN_SECTIONS: usize = 6;
/// Number of squares for the internal board (10x12)
pub const NUM_BOARD_SQUARES: usize = 120;
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

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
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

    // TODO: TEST!
    /// Generates a new Gamestate from a FEN &str. Base FEN gets converted to board via TryFrom<&str>
    pub fn new_from_fen(fen: &str) -> Result<Self, FENParseError> {
        let fen_sections: Vec<&str> = fen.trim().split(' ').collect();

        match fen_sections.len() {
            len if len == NUM_FEN_SECTIONS => {
                let active_color_str = fen_sections[1];
                let active_color = match active_color_str {
                    white if white == Color::White.to_string().as_str() => Color::White,
                    black if black == Color::Black.to_string().as_str() => Color::Black,
                    _ => {
                        return Err(FENParseError::ActiveColorInvalid(
                            active_color_str.to_string(),
                        ))
                    }
                };

                // TODO: look into X-FEN and Shredder-FEN for Chess960
                let castle_permissions_str = fen_sections[2];
                let castle_permissions_try = CastlePerm::try_from(castle_permissions_str);
                let castle_permissions = match castle_permissions_try {
                    Ok(cp) => cp,
                    Err(_) => {
                        return Err(FENParseError::CastlePermInvalid(
                            castle_permissions_str.to_string(),
                        ))
                    }
                };

                let en_passant_str = fen_sections[3];
                let en_passant_try = Square::try_from(en_passant_str.to_uppercase().as_str());
                let en_passant = match en_passant_try {
                    Ok(ep) => Some(ep),
                    Err(_) => match en_passant_str {
                        "-" => None,
                        _ => {
                            return Err(FENParseError::EnPassantInvalid(en_passant_str.to_string()))
                        }
                    },
                };

                let halfmove_clock_str = fen_sections[4];
                let halfmove_clock_try = halfmove_clock_str.parse::<u32>();
                let halfmove_clock = match halfmove_clock_try {
                    Ok(num) => match num {
                        n if (0..=MAX_GAME_MOVES).contains(&(n as usize)) => n,
                        _ => return Err(FENParseError::HalfmoveClockExceedsMaxGameMoves(num)),
                    },
                    Err(_) => {
                        return Err(FENParseError::HalfmoveClockInvalid(
                            halfmove_clock_str.to_string(),
                        ))
                    }
                };

                let fullmove_number_str = fen_sections[5];
                let fullmove_number_try = fullmove_number_str.parse::<u32>();
                let fullmove_number = match fullmove_number_try {
                    Ok(num) => match num {
                        n if (0..=MAX_GAME_MOVES / 2).contains(&(n as usize)) => n,
                        _ => return Err(FENParseError::FullmoveClockExceedsMaxGameMoves(num)),
                    },
                    Err(_) => {
                        return Err(FENParseError::FullmoveClockInvalid(
                            fullmove_number_str.to_string(),
                        ));
                    }
                };

                let board_str = fen_sections[0];
                let board_try = Board::try_from(board_str);
                let board = match board_try {
                    Ok(b) => b,
                    Err(_) => {
                        return Err(FENParseError::BoardInvalid(board_str.to_string()));
                    }
                };

                let history: [Option<Undo>; MAX_GAME_MOVES] = [None; MAX_GAME_MOVES];
                let zobrist = Zobrist::default();

                // TODO: check that the castle permissions actually match the board
                // TODO: optionally at the end check if active color can win in one move and disallow

                Ok(Gamestate {
                    board,
                    active_color,
                    castle_permissions,
                    en_passant,
                    halfmove_clock,
                    fullmove_number,
                    history,
                    zobrist,
                })
            }
            _ => Err(FENParseError::WrongNumFENSections(fen_sections.len())),
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
