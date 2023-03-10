use std::{
    default,
    fmt::{self, write},
    num::ParseIntError,
};
use strum::EnumCount;
use strum_macros::{Display as EnumDisplay, EnumCount as EnumCountMacro};

use crate::{
    board::{Board, BoardBuilder, NUM_BOARD_COLUMNS, NUM_BOARD_ROWS, NUM_BOARD_SQUARES},
    castle_perm::{self, CastlePerm, NUM_CASTLE_PERM},
    color::Color,
    error::{
        BoardFenDeserializeError, GamestateBuildError, GamestateFenDeserializeError,
        GamestateValidityCheckError, RankFenDeserializeError, SquareConversionError,
    },
    file::File,
    moves::Move,
    piece::{self, Piece, PieceType},
    rank::Rank,
    square::{Square, Square64},
    zobrist::Zobrist,
};

// CONSTANTS:
/// Maximum number of full moves we expect
pub const MAX_GAME_MOVES: usize = 1024;
/// When we reach 50 moves (aka 100 half moves) without a pawn advance or a piece capture the game ends
/// immediately in a tie
pub const HALF_MOVE_MAX: u8 = 100;
pub const NUM_FEN_SECTIONS: usize = 6;
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Undo {
    move_: Move,
    castle_permissions: CastlePerm,
    en_passant: Option<Square>,
    halfmove_clock: u8,
    position_key: u64,
}

// NOTE: There might be more variants in the future like Strict, or Chess960
// TODO: consider actually using the builder pattern here to construct more flexible group of checks down the road
// TODO: the invalid square check is not necessary when you create a board from a FEN actually. Potentially
// rework the modes in order to remove that extra work. If Validity Checks get built with builders this could
// actually be fairly easy to do. Create a director class to have easy "recipes" for building from FEN for example

/// Mode explanation:
///
/// For Board:
/// Basic : If you pass in pieces, the type system will give you basic guarantees. If you pass in a FEN,
/// then Basic mode will make sure that that FEN has 8 sections, each rank FEN corresponds to the correct number of squares,
/// and each symbol corresponds to a valid Piece. This mode can be useful for testing purposes.
///
///
/// Strict: Strict mode adds additional checks to make sure that the board is valid given the rules of regular chess.
///
#[derive(Debug)]
pub enum ValidityCheck {
    Basic,
    Strict,
}

#[derive(Debug)]
pub struct GamestateBuilder {
    validity_check: ValidityCheck,
    board: Board,
    active_color: Color,
    castle_permissions: CastlePerm,
    en_passant: Option<Square64>,
    halfmove_clock: u8,
    fullmove_number: u32,
    history: Vec<Undo>,
}

// TODO: Revisit question of clones and performance and see if you can improve ergonomics:
// https://users.rust-lang.org/t/builder-pattern-in-rust-self-vs-mut-self-and-method-vs-associated-function/72892/2
// currently shouldn't be copying because Board doesn't implement Copy. Which seems reasonable for now
// That being said, why make the Board reusable if the Gamestate isn't?
impl GamestateBuilder {
    pub fn new() -> Self {
        GamestateBuilder {
            validity_check: ValidityCheck::Strict,
            board: BoardBuilder::new()
                .build()
                .expect("new() version of board should never fail"),
            active_color: Color::White,
            castle_permissions: CastlePerm::default(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            history: vec![],
        }
    }

    pub fn new_with_board(board: Board) -> Self {
        GamestateBuilder {
            validity_check: ValidityCheck::Strict,
            board,
            active_color: Color::White,
            castle_permissions: CastlePerm::default(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            history: vec![],
        }
    }

    // TODO: make sure that on the frontend the number of characters that can be passed is limited to something reasonable
    // TODO: look into X-FEN and Shredder-FEN for Chess960)
    pub fn new_with_fen(gamestate_fen: &str) -> Result<Self, GamestateFenDeserializeError> {
        let mut board = None;
        let mut active_color = None;
        let mut castle_permissions = None;
        let mut en_passant = None;
        let mut halfmove_clock = None;
        let mut fullmove_number = None;

        // Allow for extra spaces in between sections but not in the middle of sections
        let fen_sections = gamestate_fen
            .split(' ')
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>();

        match fen_sections.len() {
            NUM_FEN_SECTIONS => {
                for (index, section) in fen_sections.into_iter().enumerate() {
                    match index {
                        // Turn off board checking default so that it can be set by Gamestate
                        0 => {
                            board = Some(
                                BoardBuilder::new_with_fen(section)?
                                    .validity_check(ValidityCheck::Basic)
                                    .build()?,
                            )
                        }
                        // active_color should be either "w" or "b"
                        1 => {
                            active_color = match section {
                                white if white == char::from(Color::White).to_string() => {
                                    Some(Color::White)
                                }
                                black if black == char::from(Color::Black).to_string() => {
                                    Some(Color::Black)
                                }
                                _ => {
                                    return Err(GamestateFenDeserializeError::ActiveColor {
                                        gamestate_fen: gamestate_fen.to_owned(),
                                        invalid_color: section.to_owned(),
                                    });
                                }
                            }
                        }
                        2 => castle_permissions = Some(CastlePerm::try_from(section)?),
                        3 => {
                            en_passant = match section {
                                "-" => None,
                                _ => Some(Square64::try_from(section.to_uppercase().as_str())?),
                            }
                        }
                        4 => {
                            halfmove_clock = Some(section.parse::<u8>().map_err(|_err| {
                                GamestateFenDeserializeError::HalfmoveClock {
                                    halfmove_fen: section.to_owned(),
                                }
                            })?)
                        }
                        5 => {
                            fullmove_number = Some(section.parse::<u32>().map_err(|_err| {
                                GamestateFenDeserializeError::FullmoveNumber {
                                    fullmove_fen: section.to_owned(),
                                }
                            })?)
                        }
                        _ => panic!("index should be in range 0..=5"),
                    }
                }

                let board = board.unwrap();
                let active_color = active_color.unwrap();
                let castle_permissions = castle_permissions.unwrap();
                let halfmove_clock = halfmove_clock.unwrap();
                let fullmove_number = fullmove_number.unwrap();

                Ok(GamestateBuilder {
                    validity_check: ValidityCheck::Strict,
                    board,
                    active_color,
                    castle_permissions,
                    en_passant,
                    halfmove_clock,
                    fullmove_number,
                    history: vec![],
                })
            }
            _ => Err(GamestateFenDeserializeError::WrongNumFENSections {
                num_fen_sections: fen_sections.len(),
            }),
        }
    }

    pub fn validity_check(mut self, validity_check: ValidityCheck) -> Self {
        self.validity_check = validity_check;
        self
    }

    pub fn active_color(mut self, active_color: Color) -> Self {
        self.active_color = active_color;
        self
    }

    pub fn castle_permissions(mut self, castle_permissions: CastlePerm) -> Self {
        self.castle_permissions = castle_permissions;
        self
    }

    pub fn en_passant(mut self, en_passant: Option<Square64>) -> Self {
        self.en_passant = en_passant;
        self
    }

    pub fn halfmove_clock(mut self, halfmove_clock: u8) -> Self {
        self.halfmove_clock = halfmove_clock;
        self
    }

    pub fn fullmove_number(mut self, fullmove_number: u32) -> Self {
        self.fullmove_number = fullmove_number;
        self
    }

    pub fn history(mut self, history: Vec<Undo>) -> Self {
        self.history = history;
        self
    }

    pub fn build(&self) -> Result<Gamestate, GamestateBuildError> {
        let gamestate = Gamestate {
            board: self.board.clone(),
            active_color: self.active_color,
            castle_permissions: self.castle_permissions,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
            history: self.history.clone(),
            zobrist: Zobrist::default(),
        };

        if let ValidityCheck::Strict = self.validity_check {
            gamestate.check_gamestate(&self.validity_check)?;
        }

        Ok(gamestate)
    }
}

impl Default for GamestateBuilder {
    fn default() -> Self {
        GamestateBuilder::new_with_board(Board::default()).validity_check(ValidityCheck::Basic)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Gamestate {
    board: Board,
    active_color: Color,
    castle_permissions: CastlePerm,
    en_passant: Option<Square64>,
    /// number of moves both players have made since last pawn advance of piece capture
    halfmove_clock: u8,
    /// number of completed turns in the game (incremented when black moves)
    fullmove_number: u32,
    history: Vec<Undo>,
    zobrist: Zobrist,
}

impl Default for Gamestate {
    fn default() -> Self {
        GamestateBuilder::new_with_board(Board::default())
            .validity_check(ValidityCheck::Basic)
            .build()
            .expect("starting gamestate should never fail to build")
    }
}

/// Attempts to deserialize a gamestate fen into a Gamestate
impl TryFrom<&str> for Gamestate {
    type Error = GamestateBuildError;
    fn try_from(gamestate_fen: &str) -> Result<Self, Self::Error> {
        GamestateBuilder::new_with_fen(gamestate_fen)?.build()
    }
}

impl fmt::Display for Gamestate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.board);
        writeln!(f, "Active Color: {}", self.active_color);
        write!(f, "En Passant: ");
        match self.en_passant {
            Some(ep) => {
                writeln!(f, "{}", ep);
            }
            None => {
                writeln!(f, "None");
            }
        }
        writeln!(f, "Castle Permissions: {}", self.castle_permissions);
        writeln!(f, "Position Key: {}", self.gen_position_key())
    }
}

impl Gamestate {
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

    /// Check that the gamestate is valid for the given a validity check mode
    fn check_gamestate(
        &self,
        validity_check: &ValidityCheck,
    ) -> Result<(), GamestateValidityCheckError> {
        if let ValidityCheck::Strict = validity_check {
            // TODO:
            // check that the non-active player is not in check
            // check that the active player is checked less than 3 times
            // check that if the active player is checked 2 times it can't be:
            // check if active color can win in one move (not allowed)
            // check that the castling permissions don't contradict the position of rooks and kings

            // check board is valid
            self.board.check_board(validity_check)?;

            // check that halfmove clock doesn't violate the 50 move rule
            if self.halfmove_clock >= HALF_MOVE_MAX {
                return Err(GamestateValidityCheckError::StrictHalfmoveClockExceedsMax {
                    halfmove_clock: self.halfmove_clock,
                });
            }

            // check that fullmove number is in valid range 1..=MAX_GAME_MOVES
            if !(1..=MAX_GAME_MOVES).contains(&(self.fullmove_number as usize)) {
                return Err(
                    GamestateValidityCheckError::StrictFullmoveNumberNotInRange {
                        fullmove_number: self.fullmove_number,
                    },
                );
            }

            // check that fullmove number and halfmove clock are plausible
            // NOTE: fullmove_number starts at 1 and increments every time black moves
            // halfmove_clock starts at 0 and increases everytime a player makes a move that does not
            // move a pawn or capture a piece.
            // Initial setup is active color: white, fullmove: 1, halfmove: 0
            // so 2 * (1 - 1) + 0 = 0 which is not less than 0
            // Now if white moves a knight out, you should have color: black, fullmove: 1, halfmove: 1
            // so 2*(1 - 1) + 1 = 1 which is not less than 1
            // Now say black moves a pawn but we get back color: white, fullmove: 2, halfmove: 2
            // so 2*(2 - 1) + 0 = 2  which is not less than 2
            // we can't catch that kind of mistake here because for all we know black moved a knight
            // but let's say that we get back color: white, fullmove: 1, halfmove: 2
            // in order to get halfmove: 2, white had to play a knight, then black had to play a knight
            // as well which should have incremented fullmove. That's what's being caught here
            if (2 * (self.fullmove_number - 1) + self.active_color as u32)
                < self.halfmove_clock as u32
            {
                return Err(
                GamestateValidityCheckError::StrictFullmoveNumberLessThanHalfmoveClockDividedByTwo {
                    fullmove_number: self.fullmove_number,
                    halfmove_clock: self.halfmove_clock,
                });
            }

            //====================== EN PASSANT CHECKS ========================

            if let Some(en_passant) = self.en_passant {
                // Take in a Square64 so that you can't be given an invalid square but then cast it to 120 format
                // so that we can use it as an index properly
                let en_passant = Square::from(en_passant);

                // check that if there is an en passant square, the halfmove clock must be 0 (pawn just moved resets clock)
                if !matches!(self.halfmove_clock, 0) {
                    return Err(
                        GamestateValidityCheckError::StrictEnPassantHalfmoveClockNotZero {
                            halfmove_clock: self.halfmove_clock,
                        },
                    );
                }

                // check that en passant square is on proper rank given active color
                let rank = en_passant.get_rank();
                match rank {
                    // White pawn just moved up by two spaces
                    // If active color is black then en_passant rank has to be 3.
                    Rank::Rank3 => {
                        match self.active_color {
                            Color::Black => {
                                // check that the en passant square is empty
                                if let Some(_piece) = self.board.pieces[en_passant as usize] {
                                    return Err(
                                        GamestateValidityCheckError::StrictEnPassantNotEmpty {
                                            en_passant_square: en_passant,
                                        },
                                    );
                                }

                                // check that the square behind the en_passant square is empty
                                let square_behind_index = en_passant as usize - NUM_BOARD_COLUMNS;
                                if let Some(_piece) = self.board.pieces[square_behind_index] {
                                    return Err(
                                    GamestateValidityCheckError::StrictEnPassantSquareBehindNotEmpty {
                                        square_behind: Square::try_from(square_behind_index)
                                            .expect(
                                            "should never fail since we know that we are on rank 3",
                                        ),
                                    },
                                );
                                }

                                let square_ahead_index = en_passant as usize + NUM_BOARD_COLUMNS;
                                match self.board.pieces[square_ahead_index] {
                                    // check that white pawn is in front of en passant square
                                    Some(piece) => {
                                        if !matches!(piece, Piece::WhitePawn) {
                                            return Err(GamestateValidityCheckError::StrictEnPassantSquareAheadUnexpectedPiece {
                                            square_ahead: Square::try_from(square_ahead_index)
                                            .expect("should never fail since we know that we are on rank 3"),
                                            invalid_piece: piece,
                                            expected_piece: Piece::WhitePawn
                                        });
                                        }
                                    }
                                    None => {
                                        return Err(GamestateValidityCheckError::StrictEnPassantSquareAheadEmpty {
                                    square_ahead: Square::try_from(square_ahead_index)
                                    .expect("should never fail since we know that we are on rank 3")
                                });
                                    }
                                }
                            }

                            Color::White => {
                                return Err(GamestateValidityCheckError::StrictColorRankMismatch {
                                    active_color: self.active_color,
                                    rank,
                                });
                            }
                        }
                    }

                    // Black pawn just moved up by two spaces
                    // If active color is white then en_passant rank has to be 6.
                    Rank::Rank6 => {
                        match self.active_color {
                            Color::White => {
                                // check that the en passant square is empty
                                if let Some(_piece) = self.board.pieces[en_passant as usize] {
                                    return Err(
                                        GamestateValidityCheckError::StrictEnPassantNotEmpty {
                                            en_passant_square: en_passant,
                                        },
                                    );
                                }

                                // check that the square behind the en_passant square is empty
                                let square_behind_index = en_passant as usize + NUM_BOARD_COLUMNS;
                                if let Some(_piece) = self.board.pieces[square_behind_index] {
                                    return Err(
                                    GamestateValidityCheckError::StrictEnPassantSquareBehindNotEmpty {
                                        square_behind: Square::try_from(square_behind_index)
                                            .expect(
                                            "should never fail since we know that we are on rank 6",
                                        ),
                                    },
                                );
                                }

                                let square_ahead_index = en_passant as usize - NUM_BOARD_COLUMNS;
                                match self.board.pieces[square_ahead_index] {
                                    // check that black pawn is in front of en passant square
                                    Some(piece) => {
                                        if !matches!(piece, Piece::BlackPawn) {
                                            return Err(GamestateValidityCheckError::StrictEnPassantSquareAheadUnexpectedPiece {
                                            square_ahead: Square::try_from(square_ahead_index)
                                            .expect("should never fail since we know that we are on rank 6"),
                                            invalid_piece: piece,
                                            expected_piece: Piece::BlackPawn
                                        });
                                        }
                                    }
                                    None => {
                                        return Err(GamestateValidityCheckError::StrictEnPassantSquareAheadEmpty {
                                    square_ahead: Square::try_from(square_ahead_index)
                                    .expect("should never fail since we know that we are on rank 6")
                                });
                                    }
                                }
                            }

                            Color::Black => {
                                return Err(GamestateValidityCheckError::StrictColorRankMismatch {
                                    active_color: self.active_color,
                                    rank,
                                });
                            }
                        }
                    }
                    _ => {
                        return Err(GamestateValidityCheckError::StrictColorRankMismatch {
                            active_color: self.active_color,
                            rank,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Serialize Gamestate into FEN. Does not do any validity checking
    pub fn to_fen(&self) -> String {
        // board
        let mut fen = self.board.to_board_fen();
        fen.push(' ');

        // active_color
        fen.push(self.active_color.into());
        fen.push(' ');

        // castle_permissions
        fen.push_str(self.castle_permissions.to_string().as_str());
        fen.push(' ');

        // en_passant
        match self.en_passant {
            Some(square) => {
                fen.push_str(square.to_string().to_lowercase().as_str());
            }
            None => {
                fen.push('-');
            }
        }
        fen.push(' ');

        // halfmove_clock
        fen.push_str(self.halfmove_clock.to_string().as_str());
        fen.push(' ');

        // fullmove_number
        fen.push_str(self.fullmove_number.to_string().as_str());

        fen
    }

    /// Determine if the provided square is currently under attack by the
    /// provided color
    fn is_square_attacked(&self, color: Color, square: Square) -> bool {
        // depending on active_color determine which pieces to check
        let mut pieces_to_check: [Piece; 6];
        match color {
            Color::White => {
                pieces_to_check = [
                    Piece::WhitePawn,
                    Piece::WhiteKnight,
                    Piece::WhiteBishop,
                    Piece::WhiteRook,
                    Piece::WhiteQueen,
                    Piece::WhiteKing,
                ]
            }
            Color::Black => {
                pieces_to_check = [
                    Piece::BlackPawn,
                    Piece::BlackKnight,
                    Piece::BlackBishop,
                    Piece::BlackRook,
                    Piece::BlackQueen,
                    Piece::BlackKing,
                ]
            }
        }
        // Going through each type of piece that could be attacking the given square
        // check each square an attacker could be occupying and see if there is in fact
        // the corresponding piece on that attacking square
        for piece in pieces_to_check {
            let directions = piece.get_attack_directions();
            match piece {
                // To check sliding pieces we need to check squares offset by multiples
                // of the direction offset, and early out of a direction when we hit a blocking piece
                // and early out entirely if we find an attacking piece
                sliding if piece.is_sliding() => {
                    // Optimization: bishops can never attack a square that is a different color than they are
                    if (sliding.is_bishop() && (square.get_color() != sliding.get_color())) {
                        continue;
                    }

                    for direction in directions {
                        let mut offset = direction;
                        while let Ok(next_square) = square + offset {
                            match self.board.pieces[next_square as usize] {
                                Some(p) => match p {
                                    attacker if p == piece => return true,
                                    blocker => break,
                                },
                                None => offset += direction,
                            }
                        }
                    }
                }
                // Non-sliding pieces only need to check the squares offset by each direction (no need
                // to check multiples of offset or blocking pieces)
                non_sliding => {
                    for direction in directions {
                        // check if moving in direction places you on a valid square
                        if let Ok(valid_square) = square + direction {
                            // check if the type of piece that could attack our square from the current evaluated square
                            // is present or not.
                            if let Some(p) = self.board.pieces[valid_square as usize] {
                                match p {
                                    attacker if p == piece => return true,
                                    _ => (),
                                }
                            }
                        }
                    }
                }
            }
        }
        // if we never early returned true, then our square is not under attack
        false
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;
    use crate::{
        board::bitboard::BitBoard,
        error::{BoardBuildError, BoardValidityCheckError, PieceConversionError},
        file::File,
        gamestate,
    };

    //========================= REUSABLE BUILDER ==============================
    #[test]
    fn test_gamestate_builder_is_reusable() {
        let mut gamestate_builder = GamestateBuilder::new_with_board(
            BoardBuilder::new()
            .validity_check(ValidityCheck::Basic)
            .piece(Piece::BlackBishop, Square64::A1)
            .build()
            .unwrap()
        )
        .validity_check(ValidityCheck::Basic);

        let gamestate_0 = gamestate_builder.build().unwrap();
        let gamestate_1 = gamestate_builder.build().unwrap();

        assert_eq!(gamestate_0, gamestate_1);
    }


    //=========================== FEN parsing tests ===========================
    // Full FEN parsing
    #[test]
    fn test_gamestate_try_from_valid_fen_default() {
        let input = DEFAULT_FEN;
        let output = Gamestate::try_from(input);
        let default = Gamestate::default();

        #[rustfmt::skip]
        let pieces = [
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, Some(Piece::WhiteRook), Some(Piece::WhiteKnight), Some(Piece::WhiteBishop), Some(Piece::WhiteQueen), Some(Piece::WhiteKing), Some(Piece::WhiteBishop), Some(Piece::WhiteKnight), Some(Piece::WhiteRook), None,
                None, Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn),  Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn), None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn),  Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn), None,
                None, Some(Piece::BlackRook), Some(Piece::BlackKnight), Some(Piece::BlackBishop), Some(Piece::BlackQueen), Some(Piece::BlackKing), Some(Piece::BlackBishop), Some(Piece::BlackKnight), Some(Piece::BlackRook), None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        ];

        let board = BoardBuilder::new_with_pieces(pieces).build().unwrap();

        // let board = Board {
        //     pieces: [
        //         None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        //         None, Some(Piece::WhiteRook), Some(Piece::WhiteKnight), Some(Piece::WhiteBishop), Some(Piece::WhiteQueen), Some(Piece::WhiteKing), Some(Piece::WhiteBishop), Some(Piece::WhiteKnight), Some(Piece::WhiteRook), None,
        //         None, Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn),  Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn), None,
        //         None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        //         None, Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn),  Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn), None,
        //         None, Some(Piece::BlackRook), Some(Piece::BlackKnight), Some(Piece::BlackBishop), Some(Piece::BlackQueen), Some(Piece::BlackKing), Some(Piece::BlackBishop), Some(Piece::BlackKnight), Some(Piece::BlackRook), None,
        //         None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        //     ],
        //     pawns: [BitBoard(0x000000000000FF00), BitBoard(0x00FF000000000000)],
        //     kings_square: [Some(Square::E1), Some(Square::E8)],
        //     piece_count: [8, 2, 2, 2, 1, 1, 8, 2, 2, 2, 1, 1],
        //     big_piece_count: [8, 8],
        //     // NOTE: King considered major piece for us
        //     major_piece_count: [4, 4],
        //     minor_piece_count: [4, 4],
        // piece_list: [
        //     // WhitePawns
        //     vec![Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2],
        //     // WhiteKnights
        //     vec![Square::B1, Square::G1],
        //     // WhiteBishops
        //     vec![Square::C1, Square::F1],
        //     // WhiteRooks
        //     vec![Square::A1, Square::H1],
        //     // WhiteQueens
        //     vec![Square::D1],
        //     // WhiteKing
        //     vec![Square::E1],
        //     // BlackPawns
        //     vec![Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7],
        //     // BlackKnights
        //     vec![Square::B8, Square::G8],
        //     // BlackBishops
        //     vec![Square::C8, Square::F8],
        //     // BlackRooks
        //     vec![Square::A8, Square::H8],
        //     // BlackQueens
        //     vec![Square::D8],
        //     // BlackKing
        //     vec![Square::E8],
        // ]
        // };

        let active_color = Color::White;
        let castle_permissions = CastlePerm::default();
        let en_passant = None;
        let halfmove_clock = 0;
        let fullmove_number = 1;
        let history = Vec::new();
        let zobrist = Zobrist::default();

        let expected = Ok(Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        });

        // board
        assert_eq!(
            output.as_ref().unwrap().board,
            expected.as_ref().unwrap().board
        );
        // active_color
        assert_eq!(
            output.as_ref().unwrap().active_color,
            expected.as_ref().unwrap().active_color
        );
        // castle_permissions
        assert_eq!(
            output.as_ref().unwrap().castle_permissions,
            expected.as_ref().unwrap().castle_permissions
        );
        // en_passant
        assert_eq!(
            output.as_ref().unwrap().en_passant,
            expected.as_ref().unwrap().en_passant
        );
        // halfmove_clock
        assert_eq!(
            output.as_ref().unwrap().halfmove_clock,
            expected.as_ref().unwrap().halfmove_clock
        );
        // fullmove_number
        assert_eq!(
            output.as_ref().unwrap().fullmove_number,
            expected.as_ref().unwrap().fullmove_number
        );
        // history
        assert_eq!(
            output.as_ref().unwrap().history,
            expected.as_ref().unwrap().history
        );
        // zobrist
        assert_eq!(
            output.as_ref().unwrap().zobrist,
            expected.as_ref().unwrap().zobrist
        );
        // println!("output zobrist:{:?}\nexpected zobrist:{:?}", output.as_ref().unwrap().zobrist, output.as_ref().unwrap().zobrist);
        assert_eq!(output, expected);
        assert_eq!(default, expected.unwrap());
    }

    // Square Attacks
    #[test]
    fn test_square_attacked_queen_no_blockers() {
        // const FEN_1: &str = "8/3q4/8/8/4Q3/8/8/8 w - - 0 2";
        let board = BoardBuilder::new()
            .validity_check(ValidityCheck::Basic)
            .piece(Piece::WhiteQueen, Square64::E4)
            .piece(Piece::BlackQueen, Square64::D7)
            .build()
            .unwrap();

        // #[rustfmt::skip]
        // let board = Board {
        //     pieces: [
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, None,                    Some(Piece::WhiteQueen), None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, Some(Piece::BlackQueen), None,                    None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //         None, None, None, None, None,                    None,                    None, None, None, None,
        //     ],
        //     pawns: [BitBoard(0), BitBoard(0)],
        //     kings_square: [None; Color::COUNT],
        //     piece_count: [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0],
        //     big_piece_count: [1, 1],
        //     // NOTE: King considered major piece for us
        //     major_piece_count: [1, 1],
        //     minor_piece_count: [0, 0],
        //     piece_list: [
        //         // WhitePawns
        //         vec![],
        //         // WhiteKnights
        //         vec![],
        //         // WhiteBishops
        //         vec![],
        //         // WhiteRooks
        //         vec![],
        //         // WhiteQueens
        //         vec![Square::E4],
        //         // WhiteKing
        //         vec![],
        //         // BlackPawns
        //         vec![],
        //         // BlackKnights
        //         vec![],
        //         // BlackBishops
        //         vec![],
        //         // BlackRooks
        //         vec![],
        //         // BlackQueens
        //         vec![Square::D7],
        //         // BlackKing
        //         vec![]
        //     ]
        // };

        let active_color = Color::White;
        let castle_permissions = CastlePerm::try_from(0).unwrap();
        let en_passant = None;
        let halfmove_clock = 0;
        let fullmove_number = 2;
        let history = Vec::new();
        let zobrist = Zobrist::default();

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] = gamestate.is_square_attacked(active_color, square);
            }
        }

        #[rustfmt::skip]
        let expected = [
            [false, true,  false, false, true,  false, false, true],
            [false, false, true,  false, true,  false, true,  false],
            [false, false, false, true,  true,  true,  false, false],
            [true,  true,  true,  true,  false, true,  true,  true],
            [false, false, false, true,  true,  true,  false, false],
            [false, false, true,  false, true,  false, true,  false],
            [false, true,  false, false, true,  false, false, true],
            [true,  false, false, false, true,  false, false, false]
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_attacked_queen_with_blocker() {
        // const FEN_1: &str = "8/3q4/8/8/4Q3/8/2P5/8 w - - 0 2";
        let board = BoardBuilder::new()
            .validity_check(ValidityCheck::Basic)
            .piece(Piece::WhitePawn, Square64::C2)
            .piece(Piece::WhiteQueen, Square64::E4)
            .piece(Piece::BlackQueen, Square64::D7)
            .build()
            .unwrap();

        // #[rustfmt::skip]
        // let board = Board {
        //     pieces: [
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     Some(Piece::WhitePawn),   None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    Some(Piece::WhiteQueen), None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     Some(Piece::BlackQueen), None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //     ],
        //     pawns: [BitBoard(0x00_00_00_00_00_00_04_00), BitBoard(0)],
        //     kings_square: [None; Color::COUNT],
        //     piece_count: [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0],
        //     big_piece_count: [1, 1],
        //     // NOTE: King considered major piece for us
        //     major_piece_count: [1, 1],
        //     minor_piece_count: [0, 0],
        //     piece_list: [
        //         // WhitePawns
        //         vec![Square::C2],
        //         // WhiteKnights
        //         vec![],
        //         // WhiteBishops
        //         vec![],
        //         // WhiteRooks
        //         vec![],
        //         // WhiteQueens
        //         vec![Square::E4],
        //         // WhiteKing
        //         vec![],
        //         // BlackPawns
        //         vec![],
        //         // BlackKnights
        //         vec![],
        //         // BlackBishops
        //         vec![],
        //         // BlackRooks
        //         vec![],
        //         // BlackQueens
        //         vec![Square::D7],
        //         // BlackKing
        //         vec![]
        //     ]
        // };

        println!("{}", board);

        let active_color = Color::White;
        let castle_permissions = CastlePerm::try_from(0).unwrap();
        let en_passant = None;
        let halfmove_clock = 0;
        let fullmove_number = 2;
        let history = Vec::new();
        let zobrist = Zobrist::default();

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] = gamestate.is_square_attacked(active_color, square);
            }
        }

        #[rustfmt::skip]
        let expected = [
            [false, false, false, false, true,  false, false, true],
            [false, false, true,  false, true,  false, true,  false],
            [false, true,  false, true,  true,  true,  false, false],
            [true,  true,  true,  true,  false, true,  true,  true],
            [false, false, false, true,  true,  true,  false, false],
            [false, false, true,  false, true,  false, true,  false],
            [false, true,  false, false, true,  false, false, true],
            [true,  false, false, false, true,  false, false, false]
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_attacked_white_bishop_on_black_square() {
        // const FEN: &str = "8/8/8/8/8/2B4K/8/k7 w - - 0 1";
        let board = BoardBuilder::new()
            .piece(Piece::BlackKing, Square64::A1)
            .piece(Piece::WhiteBishop, Square64::C3)
            .piece(Piece::WhiteKing, Square64::H3)
            .build()
            .unwrap();

        // #[rustfmt::skip]
        // let board = Board {
        //     pieces: [
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, Some(Piece::BlackKing), None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     Some(Piece::WhiteBishop), None,                    None,                    None,                     None,                     Some(Piece::WhiteKing), None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //         None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
        //     ],
        //     pawns: [BitBoard(0), BitBoard(0)],
        //     kings_square: [Some(Square::H3), Some(Square::A1)],
        //     piece_count: [0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0],
        //     big_piece_count: [2, 1],
        //     // NOTE: King considered major piece for us
        //     major_piece_count: [1, 1],
        //     minor_piece_count: [1, 0],
        //     piece_list: [
        //         // WhitePawns
        //         vec![],
        //         // WhiteKnights
        //         vec![],
        //         // WhiteBishops
        //         vec![Square::C3],
        //         // WhiteRooks
        //         vec![],
        //         // WhiteQueens
        //         vec![],
        //         // WhiteKing
        //         vec![Square::H3],
        //         // BlackPawns
        //         vec![],
        //         // BlackKnights
        //         vec![],
        //         // BlackBishops
        //         vec![],
        //         // BlackRooks
        //         vec![],
        //         // BlackQueens
        //         vec![],
        //         // BlackKing
        //         vec![Square::A1],
        //     ]
        // };
        let active_color = Color::White;
        let castle_permissions = CastlePerm::try_from(0).unwrap();
        let en_passant = None;
        let halfmove_clock = 0;
        let fullmove_number = 1;
        let history = Vec::new();
        let zobrist = Zobrist::default();

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] = gamestate.is_square_attacked(active_color, square);
            }
        }

        #[rustfmt::skip]
        let expected = [
            [false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, true,  true],
            [false, false, false, false, false, false, true,  false],
            [false, false, false, false, false, false, true,  true],
            [false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, false, false],
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_attacked_visual_inspection() {
        const FEN_0: &str = "4k3/pppppppp/8/8/8/8/PPPPPPPP/3K4 w - - 0 1";
        const FEN_1: &str = "rnbqkbnr/1p1ppppp/8/2p5/4p3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
        const FEN_2: &str = "rnbqkbnr/1p1ppppp/8/2p5/4p3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        const FEN_3: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let fens = [FEN_0, FEN_1, FEN_2, FEN_3];

        for fen in fens {
            let gamestate = Gamestate::try_from(fen).unwrap();
            println!("FEN: {}", fen);
            println!("Board:\n{}", gamestate.board);
            println!("All squares attacked by {}:", gamestate.active_color);
            for rank in Rank::iter().rev() {
                for file in File::iter() {
                    let square = Square::from_file_and_rank(file, rank);
                    match gamestate.is_square_attacked(gamestate.active_color, square) {
                        true => print!("X"),
                        false => print!("-"),
                    }
                }
                println!()
            }
            println!()
        }
    }

    // Display
    // TODO: When perft testing is built get rid of this test since it really isn't worth testing the display like this
    #[rustfmt::skip]
    #[test]
    fn test_gamestate_display() {
        let fen_start = DEFAULT_FEN;
        let gs_start = Gamestate::try_from(fen_start).unwrap();
        let gs_start_string = gs_start.to_string();
        let fen_wpe4 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let gs_wpe4 = Gamestate::try_from(fen_wpe4).unwrap();
        let gs_wpe4_string = gs_wpe4.to_string();
        let fen_bpc5 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
        let gs_bpc5 = Gamestate::try_from(fen_bpc5).unwrap();
        let gs_bpc5_string = gs_bpc5.to_string();
        let fen_wnf3 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        let gs_wnf3 = Gamestate::try_from(fen_wnf3).unwrap();
        let gs_wnf3_string = gs_wnf3.to_string();

        println!("Starting Position:\n{}", gs_start);
        println!("Move white pawn to E4:\n{}", gs_wpe4);
        println!("Move black pawn to C5:\n{}", gs_bpc5);
        println!("Move white knight to F3:\n{}", gs_wnf3);

        let expected_board_start = format!("{}{}{}{}{}{}{}{}{}",
                            "8\t♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n",
                            "7\t♟\t♟\t♟\t♟\t♟\t♟\t♟\t♟\n",
                            "6\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "5\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "4\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "3\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "2\t♙\t♙\t♙\t♙\t♙\t♙\t♙\t♙\n",
                            "1\t♖\t♘\t♗\t♕\t♔\t♗\t♘\t♖\n\n",
                            "\tA\tB\tC\tD\tE\tF\tG\tH\n"
                        );
        let expected_active_color_start = "White";
        let expected_en_passant_start = "None";
        let expected_castle_permissions_start = "KQkq";
        let expected_position_key_start = gs_start.gen_position_key();
        let expected_start = format!(
                                            "{}\nActive Color: {}\nEn Passant: {}\nCastle Permissions: {}\nPosition Key: {}\n", 
                                            expected_board_start,
                                            expected_active_color_start,
                                            expected_en_passant_start,
                                            expected_castle_permissions_start,
                                            expected_position_key_start
                                        );
        

        let expected_board_wpe4 = format!("{}{}{}{}{}{}{}{}{}",
                            "8\t♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n",
                            "7\t♟\t♟\t♟\t♟\t♟\t♟\t♟\t♟\n",
                            "6\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "5\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "4\t.\t.\t.\t.\t♙\t.\t.\t.\n",
                            "3\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "2\t♙\t♙\t♙\t♙\t.\t♙\t♙\t♙\n",
                            "1\t♖\t♘\t♗\t♕\t♔\t♗\t♘\t♖\n\n",
                            "\tA\tB\tC\tD\tE\tF\tG\tH\n"
                        );
        let expected_active_color_wpe4 = "Black";
        let expected_en_passant_wpe4 = "E3"; 
        let expected_castle_permissions_wpe4 = "KQkq";
        let expected_position_key_wpe4 = gs_wpe4.gen_position_key();
        let expected_wpe4 = format!(
                                            "{}\nActive Color: {}\nEn Passant: {}\nCastle Permissions: {}\nPosition Key: {}\n", 
                                            expected_board_wpe4,
                                            expected_active_color_wpe4,
                                            expected_en_passant_wpe4,
                                            expected_castle_permissions_wpe4,
                                            expected_position_key_wpe4
                                        );

        let expected_board_bpc5 = format!("{}{}{}{}{}{}{}{}{}",
                            "8\t♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n",
                            "7\t♟\t♟\t.\t♟\t♟\t♟\t♟\t♟\n",
                            "6\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "5\t.\t.\t♟\t.\t.\t.\t.\t.\n",
                            "4\t.\t.\t.\t.\t♙\t.\t.\t.\n",
                            "3\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "2\t♙\t♙\t♙\t♙\t.\t♙\t♙\t♙\n",
                            "1\t♖\t♘\t♗\t♕\t♔\t♗\t♘\t♖\n\n",
                            "\tA\tB\tC\tD\tE\tF\tG\tH\n"
                        );
        let expected_active_color_bpc5 = "White";
        let expected_en_passant_bpc5 = "C6"; 
        let expected_castle_permissions_bpc5 = "KQkq";
        let expected_position_key_bpc5 = gs_bpc5.gen_position_key();
        let expected_bpc5 = format!(
                                            "{}\nActive Color: {}\nEn Passant: {}\nCastle Permissions: {}\nPosition Key: {}\n", 
                                            expected_board_bpc5,
                                            expected_active_color_bpc5,
                                            expected_en_passant_bpc5,
                                            expected_castle_permissions_bpc5,
                                            expected_position_key_bpc5
                                        );

        let expected_board_wnf3 = format!("{}{}{}{}{}{}{}{}{}",
                            "8\t♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n",
                            "7\t♟\t♟\t.\t♟\t♟\t♟\t♟\t♟\n",
                            "6\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "5\t.\t.\t♟\t.\t.\t.\t.\t.\n",
                            "4\t.\t.\t.\t.\t♙\t.\t.\t.\n",
                            "3\t.\t.\t.\t.\t.\t♘\t.\t.\n",
                            "2\t♙\t♙\t♙\t♙\t.\t♙\t♙\t♙\n",
                            "1\t♖\t♘\t♗\t♕\t♔\t♗\t.\t♖\n\n",
                            "\tA\tB\tC\tD\tE\tF\tG\tH\n"
                        );
        let expected_active_color_wnf3 = "Black";
        let expected_en_passant_wnf3 = "None";
        let expected_castle_permissions_wnf3 = "KQkq";
        let expected_position_key_wnf3 = gs_wnf3.gen_position_key();
        let expected_wnf3 = format!(
                                            "{}\nActive Color: {}\nEn Passant: {}\nCastle Permissions: {}\nPosition Key: {}\n", 
                                            expected_board_wnf3,
                                            expected_active_color_wnf3,
                                            expected_en_passant_wnf3,
                                            expected_castle_permissions_wnf3,
                                            expected_position_key_wnf3
                                        );

        assert_eq!(gs_start_string, expected_start);
        assert_eq!(gs_wpe4_string, expected_wpe4);
        assert_eq!(gs_bpc5_string, expected_bpc5);
        assert_eq!(gs_wnf3_string, expected_wnf3);
    }

    //=================================== Serialization to FEN ================
    #[test]
    fn test_gamestate_serialization_en_passant_opening() {
        let expected = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3";
        let input = GamestateBuilder::new_with_fen(expected)
            .unwrap()
            .build()
            .unwrap();

        let output = input.to_fen();
        assert_eq!(output, expected);
    }

    // Example of how to create an invalid Gamestate from fen
    #[test]
    fn test_gamestate_serialization_validity_basic_empty() {
        let expected = "8/8/8/8/8/8/8/8 w KQkq - 0 1";
        let input = GamestateBuilder::new_with_fen(expected)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        let output = input.to_fen();
        assert_eq!(output, expected);
    }

    // If you don't turn off strict checks you should get an error when your board is invalid
    #[test]
    fn test_gamestate_serialization_validity_strict_empty() {
        let input = "8/8/8/8/8/8/8/8 w KQkq - 0 1";
        let output = GamestateBuilder::new_with_fen(input).unwrap().build();

        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::BoardValidityCheck(
                BoardValidityCheckError::StrictOneBlackKingOneWhiteKing {
                    num_white_kings: 0,
                    num_black_kings: 0,
                },
            ),
        ));

        assert_eq!(output, expected);
    }

    // Deserialization from FEN:

    // Tests for extra spaces
    #[test]
    fn test_gamestate_try_from_valid_fen_untrimmed() {
        let input = "   rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ";
        let output = Gamestate::try_from(input);
        let expected = Ok(Gamestate::default());
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_valid_fen_spaces_between_sections() {
        let input = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR  w    KQkq    - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Ok(Gamestate::default());
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_valid_fen_spaces_wrong_number_of_sections() {
        let input = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ kq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateFenDeserialize(
            GamestateFenDeserializeError::WrongNumFENSections {
                num_fen_sections: 7,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fen_spaces_in_board_section() {
        let invalid_board_section = "rnbqkbnr/pppppppp/";
        let input = "rnbqkbnr/pppppppp/ 8/8/8/8/PPPPPPPP/RNBQK BNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateFenDeserialize(
            GamestateFenDeserializeError::WrongNumFENSections {
                num_fen_sections: 8,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fen_spaces_in_board_section_end() {
        let input = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK BN w KQkq - 0"; // NOTE: had to remove a section or else wrong num sections is hit first
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateFenDeserialize(
            GamestateFenDeserializeError::BoardBuild(BoardBuildError::BoardFenDeserialize(
                BoardFenDeserializeError::RankFenDeserialize(
                    RankFenDeserializeError::InvalidNumSquares {
                        rank_fen: "RNBQK".to_owned(),
                    },
                ),
            )),
        ));
        assert_eq!(output, expected);
    }

    // NOTE: enpassant testing for - is done by the tests that use default FENs
    #[test]
    fn test_gamestate_try_from_valid_en_passant_uppercase() {
        let en_passant_str = "E6";
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq E6 0 3";
        let output = Gamestate::try_from(input);
        #[rustfmt::skip]
        let pieces = [
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, Some(Piece::WhiteRook), Some(Piece::WhiteKnight), Some(Piece::WhiteBishop), Some(Piece::WhiteQueen), Some(Piece::WhiteKing), Some(Piece::WhiteBishop), Some(Piece::WhiteKnight), Some(Piece::WhiteRook), None,
                None, Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   None,                    Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn), None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     Some(Piece::WhitePawn),  Some(Piece::BlackPawn), None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     Some(Piece::BlackPawn), None,
                None, Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn),  None,                   Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   None,                   None,
                None, Some(Piece::BlackRook), Some(Piece::BlackKnight), Some(Piece::BlackBishop), Some(Piece::BlackQueen), Some(Piece::BlackKing), Some(Piece::BlackBishop), Some(Piece::BlackKnight), Some(Piece::BlackRook), None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
        ];

        let expected = GamestateBuilder::new_with_board(
            BoardBuilder::new_with_pieces(pieces).build().unwrap(),
        )
        .active_color(Color::White)
        .castle_permissions(CastlePerm::default())
        .en_passant(Some(Square64::E6))
        .halfmove_clock(0)
        .fullmove_number(3)
        .build();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_en_passant_square() {
        let en_passant_str = "e9";
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e9 0 3";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateFenDeserialize(
            GamestateFenDeserializeError::EnPassant(strum::ParseError::VariantNotFound),
        ));
        assert_eq!(output, expected);
    }

    // En passant square can't be occupied
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_square_occupied() {
        let input = "rn1qkbnr/ppp2ppp/3pb3/3Pp3/8/8/PPPQPPPP/RNB1KBNR w KQkq e6 0 4";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictEnPassantNotEmpty {
                en_passant_square: Square::try_from("E6").unwrap(),
            },
        ));
        assert_eq!(output, expected);
    }

    // Square behind en passant square can't be occupied
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_square_behind_occupied() {
        let input = "rnbqk1nr/ppp1bppp/3p4/3Pp3/8/8/PPPQPPPP/RNB1KBNR w KQkq e6 0 4";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictEnPassantSquareBehindNotEmpty {
                square_behind: Square::E7,
            },
        ));
        assert_eq!(output, expected);
    }

    // Pawn has to be in front of en passant square
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_no_pawn_in_front() {
        let input = "rnbqkbnr/ppp2ppp/3p4/3P4/4p3/8/PPPQPPPP/RNB1KBNR w KQkq e6 0 4";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictEnPassantSquareAheadEmpty {
                square_ahead: Square::E5,
            },
        ));
        assert_eq!(output, expected);
    }

    // Correct color pawn has to be in front of en passant square
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_wrong_pawn_in_front() {
        let input = "rnbqkbnr/pp1p1ppp/2p5/4P3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictEnPassantSquareAheadUnexpectedPiece {
                square_ahead: Square::E5,
                invalid_piece: Piece::WhitePawn,
                expected_piece: Piece::BlackPawn,
            },
        ));
        assert_eq!(output, expected);
    }

    // Halfmove and Fullmove
    #[test]
    fn test_gamestate_try_from_invalid_halfmove_exceeds_max() {
        let halfmove: u8 = 100;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 100 1024";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictHalfmoveClockExceedsMax {
                halfmove_clock: halfmove,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fullmove_exceeds_max() {
        let fullmove: u32 = 1025;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1025";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictFullmoveNumberNotInRange {
                fullmove_number: fullmove,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fullmove_zero() {
        let fullmove: u32 = 0;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 0";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictFullmoveNumberNotInRange {
                fullmove_number: fullmove,
            },
        ));
        assert_eq!(output, expected);
    }

    // Tests for if Board and Rank Errors are being converted correctly to Gamestate Errors:
    #[test]
    fn test_gamestate_try_from_invalid_board_fen_all_8() {
        let invalid_board_str = "8/8/8/8/8/8/8/8";
        let input = "8/8/8/8/8/8/8/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::BoardValidityCheck(
                BoardValidityCheckError::StrictOneBlackKingOneWhiteKing {
                    num_white_kings: 0,
                    num_black_kings: 0,
                },
            ),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_few_ranks() {
        let invalid_board_str = "8/8/rbkqn2p/8/8/8/PPKPP1PP";
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateFenDeserialize(
            GamestateFenDeserializeError::BoardBuild(BoardBuildError::BoardFenDeserialize(
                BoardFenDeserializeError::WrongNumRanks {
                    board_fen: invalid_board_str.to_owned(),
                    num_ranks: 7,
                },
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_ranks() {
        let invalid_board_str = "8/8/rbkqn2p/8/8/8/PPKPP1PP/8/";
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP/8/ w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateFenDeserialize(
            GamestateFenDeserializeError::BoardBuild(BoardBuildError::BoardFenDeserialize(
                BoardFenDeserializeError::WrongNumRanks {
                    board_fen: invalid_board_str.to_owned(),
                    num_ranks: 9,
                },
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_empty_ranks() {
        let invalid_board_str = "8/8/rbkqn2p//8/8/PPKPP1PP/8";
        let input = "8/8/rbkqn2p//8/8/PPKPP1PP/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateFenDeserialize(
            GamestateFenDeserializeError::BoardBuild(BoardBuildError::BoardFenDeserialize(
                BoardFenDeserializeError::RankFenDeserialize(RankFenDeserializeError::Empty),
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_few_kings() {
        let invalid_board_str = "8/8/rbqn3p/8/8/8/PPKPP1PP/8";
        let input = "8/8/rbqn3p/8/8/8/PPKPP1PP/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::BoardValidityCheck(
                BoardValidityCheckError::StrictOneBlackKingOneWhiteKing {
                    num_white_kings: 1,
                    num_black_kings: 0,
                },
            ),
        ));
        assert_eq!(output, expected);
    }
}
