use std::{
    default,
    fmt::{self, write},
    num::ParseIntError,
};
use strum::EnumCount;
use strum_macros::{Display as EnumDisplay, EnumCount as EnumCountMacro};

use crate::{
    board::{Board, BoardBuilder, NUM_BOARD_COLUMNS, NUM_BOARD_ROWS, NUM_INTERNAL_BOARD_SQUARES},
    castle_perm::{self, Castle, CastlePerm, NUM_CASTLE_PERM},
    color::Color,
    error::{
        BoardFenDeserializeError, GamestateBuildError, GamestateFenDeserializeError,
        GamestateValidityCheckError, MakeMoveError, MoveGenError, RankFenDeserializeError,
        SquareConversionError,
    },
    file::File,
    moves::{Move, MoveList},
    piece::{
        self, Piece, PieceType, BLACK_PAWN_PROMOTION_TARGETS, BLACK_PAWN_VERTICAL_DIRECTION,
        WHITE_PAWN_PROMOTION_TARGETS, WHITE_PAWN_VERTICAL_DIRECTION,
    },
    position_key::PositionKey,
    rank::Rank,
    square::{Square, Square64},
    zobrist::ZOBRIST,
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
    en_passant: Option<Square64>,
    halfmove_clock: u8,
    position_key: PositionKey,
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
#[derive(Debug, Clone, Copy)]
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
    fullmove_count: usize,
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
                .validity_check(ValidityCheck::Basic)
                .build()
                .expect("new() version of board should never fail"),
            active_color: Color::White,
            castle_permissions: CastlePerm::new(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_count: 1,
            history: vec![],
        }
    }

    pub fn new_with_board(board: Board) -> Self {
        GamestateBuilder {
            validity_check: ValidityCheck::Strict,
            board,
            active_color: Color::White,
            castle_permissions: CastlePerm::new(),
            en_passant: None,
            halfmove_clock: 0,
            fullmove_count: 1,
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
        let mut fullmove_count = None;

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
                            fullmove_count = Some(section.parse::<usize>().map_err(|_err| {
                                GamestateFenDeserializeError::FullmoveCount {
                                    fullmove_fen: section.to_owned(),
                                }
                            })?)
                        }
                        _ => panic!(
                            "Expected index to be in range 0..=5. Found index greater than 5"
                        ),
                    }
                }

                let board = board.unwrap();
                let active_color = active_color.unwrap();
                let castle_permissions = castle_permissions.unwrap();
                let halfmove_clock = halfmove_clock.unwrap();
                let fullmove_count = fullmove_count.unwrap();

                Ok(GamestateBuilder {
                    validity_check: ValidityCheck::Strict,
                    board,
                    active_color,
                    castle_permissions,
                    en_passant,
                    halfmove_clock,
                    fullmove_count,
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

    pub fn fullmove_count(mut self, fullmove_count: usize) -> Self {
        self.fullmove_count = fullmove_count;
        self
    }

    pub fn history(mut self, history: Vec<Undo>) -> Self {
        self.history = history;
        self
    }

    pub fn build(&self) -> Result<Gamestate, GamestateBuildError> {
        let mut gamestate = Gamestate {
            board: self.board.clone(),
            active_color: self.active_color,
            castle_permissions: self.castle_permissions,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            fullmove_count: self.fullmove_count,
            position_key: PositionKey(0),
            history: self.history.clone(),
        };

        // Update position_key
        gamestate.init_position_key();

        if let ValidityCheck::Strict = self.validity_check {
            gamestate.check_gamestate(self.validity_check)?;
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
    fullmove_count: usize,
    position_key: PositionKey,
    history: Vec<Undo>,
}

impl Default for Gamestate {
    fn default() -> Self {
        GamestateBuilder::new_with_board(Board::default())
            .validity_check(ValidityCheck::Basic)
            .castle_permissions(CastlePerm(0b_1111))
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
        writeln!(f, "Position Key: {}", self.position_key)
    }
}

impl Gamestate {
    //================================= MAKING MOVES ==========================

    pub fn make_move(&mut self, move_: Move) -> Result<(), MakeMoveError> {
        // Save current active_color before we toggle it
        let initial_active_color = self.active_color;

        // Check if move_ is valid
        move_.check_move(self.active_color)?;

        // Set up ability to Undo this move
        let undo = Undo {
            move_,
            castle_permissions: self.castle_permissions,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            position_key: self.position_key,
        };
        self.history.push(undo);

        // Use raw versions of functions since validity was checked in check_move call
        let start_square = Square::try_from(move_.get_start_raw())?;
        let end_square = Square::try_from(move_.get_end_raw())?;

        // deal with en_passant moves
        if move_.is_en_passant() {
            match self.active_color {
                Color::White => {
                    // clear the square/piece that is being captured via en_passant
                    let square_to_clear = (end_square - (NUM_BOARD_COLUMNS as i8))?;
                    self.clear_piece(square_to_clear)?;
                }
                Color::Black => {
                    let square_to_clear = (end_square + (NUM_BOARD_COLUMNS as i8))?;
                    self.clear_piece(square_to_clear)?;
                }
            }
        }

        // deal with castling move
        if move_.is_castle() {
            match end_square {
                // White Queenside Castle. Move Rook from A1 to D1.
                // Presumably King has moved from E1 to C1
                Square::C1 => {
                    self.move_piece(Square::A1, Square::D1);
                }
                // White Kingside Castle. Move Rook from H1 to F1.
                // Presumably King has moved from E1 to G1
                Square::G1 => {
                    self.move_piece(Square::H1, Square::F1);
                }
                // Black Queenside Castle. Move Rook from A8 to D8.
                // Presumably King has moved from E8 to C8
                Square::C8 => {
                    self.move_piece(Square::A1, Square::D1);
                }
                // Black Kingside Castle. Move Rook from H8 to F8.
                // Presumably King has moved from E8 to G8
                Square::G8 => {
                    self.move_piece(Square::H8, Square::F8);
                }
                _ => {
                    return Err(MakeMoveError::CastleEndSquare { end_square });
                }
            }
        }

        // Reset en_passant (they expire after a move)
        self.en_passant = None;

        // Reset position_key
        if let Some(en_passant) = self.en_passant {
            self.position_key.hash_en_passant(Square::from(en_passant));
        }
        self.position_key.hash_castle_perm(self.castle_permissions);

        // Update castle_permissions
        self.castle_permissions.update(start_square, end_square);
        // Update position_key for updated castle_permissions (may or may not
        // have changed)
        self.position_key.hash_castle_perm(self.castle_permissions);

        // Deal with captured pieces and fifty-move rule
        match move_.get_piece_captured()? {
            Some(captured_piece) => {
                self.clear_piece(end_square);
                self.halfmove_clock = 0;
            }
            None => {
                self.halfmove_clock += 1;
            }
        }

        // Update fullmove_count if this is Black's move
        if self.active_color == Color::Black {
            self.fullmove_count += 1;
        }

        // Check if new en_passant square was created
        let moved_piece =
            self.board.pieces[start_square as usize].ok_or(MakeMoveError::MovedPieceNotInPieces)?;

        if moved_piece.is_pawn() {
            // fifty-move rule. reset half moves since last capture or pawn move
            self.halfmove_clock = 0;
            // pawn starts (move up 2) create en_passant squares
            if move_.is_pawn_start() {
                match self.active_color {
                    Color::White => {
                        self.en_passant =
                            Some(Square64::from((start_square + NUM_BOARD_COLUMNS as i8)?));
                    }
                    Color::Black => {
                        self.en_passant =
                            Some(Square64::from((start_square - NUM_BOARD_COLUMNS as i8)?));
                    }
                }
                // hash in new en passant square
                self.position_key
                    .hash_en_passant(Square::from(self.en_passant.expect(
                    "Expected en passant to be Some(square) since we generated one right above.",
                )));
            }
        }

        // Actually move our piece
        self.move_piece(start_square, end_square);

        // Deal with promotions
        if let Some(promoted_piece) = move_.get_piece_promoted()? {
            self.clear_piece(end_square);
            self.add_piece(end_square, promoted_piece);
        }

        // TODO: check if can be removed. probably redundant
        // Update king
        if moved_piece.is_king() {
            self.board.kings_square[self.active_color as usize] = Some(end_square);
        }

        // change active_color and hash it in
        self.active_color.toggle();
        self.position_key.hash_color();

        // TODO: is this necessary?
        self.check_gamestate(ValidityCheck::Strict)?;

        // check if move puts active color in check
        if (self.is_square_attacked(
            self.active_color,
            self.board.kings_square[initial_active_color as usize]
                .expect("Expected King's square to be stored in kings_square"),
        )) {
            // TODO: Call self.undo_move()
            return Err(MakeMoveError::MoveWouldPutMovingSideInCheck);
        }

        Ok(())
    }

    /// Moves a piece and updates all appropriate places in the Board as well as
    /// the position key. Returns an Err if there is no piece on start_square
    /// or a capture is attempted (or if piece not found in piece_list).
    fn move_piece(
        &mut self,
        start_square: Square,
        end_square: Square,
    ) -> Result<(), MakeMoveError> {
        let piece = self.board.pieces[start_square as usize]
            .ok_or(MakeMoveError::NoPieceAtMoveStart { start_square })?;

        let color = piece.get_color();
        let piece_type = piece.get_piece_type();

        let piece_on_end_square = self.board.pieces[end_square as usize];
        match piece_on_end_square {
            Some(end_piece) => {
                return Err(MakeMoveError::MoveEndsOnOccupiedSquare {
                    piece,
                    end_square,
                    end_piece,
                });
            }
            None => {
                // update pieces
                self.board.pieces[start_square as usize] = None;
                self.board.pieces[end_square as usize] = Some(piece);

                // update piece_list
                let mut found_in_piece_list = false;
                let squares_for_piece = &mut self.board.piece_list[piece as usize];
                for (sq_index, &sq) in squares_for_piece.iter().enumerate() {
                    if sq == start_square {
                        squares_for_piece[sq_index] = end_square;
                        found_in_piece_list = true;
                        break; // non-lexical lifetime
                    }
                }
                // if square was not found in piece_list something went wrong
                if !found_in_piece_list {
                    return Err(MakeMoveError::SquareNotFoundInPieceList {
                        missing_square: start_square,
                        piece,
                    });
                }

                // update pawns
                if piece_type == PieceType::Pawn {
                    self.board.pawns[color as usize].unset_bit(Square64::from(start_square));
                    self.board.pawns[color as usize].set_bit(Square64::from(end_square));
                }

                // update position key (hash piece out and back with changed square)
                self.position_key.hash_piece(piece, start_square);
                self.position_key.hash_piece(piece, end_square);
            }
        }

        Ok(())
    }

    /// Adds a piece to all the appropriate places in the Board and updates the
    /// position_key. Returns an Err if you try to add a Piece to an occupied
    /// square.
    fn add_piece(&mut self, square: Square, piece: Piece) -> Result<(), MakeMoveError> {
        let piece_on_square = self.board.pieces[square as usize];

        match piece_on_square {
            Some(occupying_piece) => {
                return Err(MakeMoveError::AddToOccupiedSquare {
                    occupied_square: square,
                    piece_at_square: occupying_piece,
                });
            }
            None => {
                // update pieces
                self.board.pieces[square as usize] = Some(piece);

                let color = piece.get_color();
                let piece_type = piece.get_piece_type();

                // update piece_list
                self.board.piece_list[piece as usize].push(square);

                // update piece counts
                match piece {
                    big_piece if piece.is_big() => {
                        self.board.big_piece_count[color as usize] += 1;
                        match big_piece {
                            major_piece if big_piece.is_major() => {
                                self.board.major_piece_count[color as usize] += 1;
                            }
                            minor_piece => {
                                self.board.minor_piece_count[color as usize] += 1;
                            }
                        }
                    }
                    // Update pawns (not big, major nor minor)
                    pawn => {
                        self.board.pawns[color as usize].set_bit(Square64::from(square));
                    }
                }
                self.board.piece_count[piece as usize] += 1;

                // update position_key (hash it in)
                self.position_key.hash_piece(piece, square);

                // update material_score
                self.board.material_score[color as usize] += piece.get_value();
            }
        }

        Ok(())
    }

    /// Removes a piece from all appropriate places in the Board and updates
    /// the position_key. Returns an Err if you try to clear an empty Square.
    fn clear_piece(&mut self, square: Square) -> Result<Piece, MakeMoveError> {
        let piece = self.board.pieces[square as usize].ok_or(MakeMoveError::NoPieceToClear {
            empty_square: square,
        })?;

        // update pieces
        self.board.pieces[square as usize] = None;

        let color = piece.get_color();
        let piece_type = piece.get_piece_type();

        // update piece_list
        let mut found_in_piece_list = false;
        let squares_for_piece = &mut self.board.piece_list[piece as usize];
        for (sq_index, &sq) in squares_for_piece.iter().enumerate() {
            if sq == square {
                // NOTE: swap_remove is O(1) but changes the order of our
                // piece_list.
                squares_for_piece.swap_remove(sq_index);
                found_in_piece_list = true;
                break; // non-lexical lifetime
            }
        }
        // if square was not found in piece_list something went wrong
        if !found_in_piece_list {
            return Err(MakeMoveError::SquareNotFoundInPieceList {
                missing_square: square,
                piece,
            });
        }

        // update piece counts
        match piece {
            big_piece if piece.is_big() => {
                self.board.big_piece_count[color as usize] -= 1;
                match big_piece {
                    major_piece if big_piece.is_major() => {
                        self.board.major_piece_count[color as usize] -= 1;
                    }
                    minor_piece => {
                        self.board.minor_piece_count[color as usize] -= 1;
                    }
                }
            }
            // Update pawns (not big, major nor minor)
            pawn => {
                self.board.pawns[color as usize].unset_bit(Square64::from(square));
            }
        }
        self.board.piece_count[piece as usize] -= 1;

        // update position_key (hash it out)
        self.position_key.hash_piece(piece, square);

        // update material_score
        self.board.material_score[color as usize] -= piece.get_value();

        Ok(piece)
    }

    //================================= MOVE GENERATION =======================

    // TODO: Splitting up move gen functions is nice but has some
    // performance cost potentially. Measure

    /// Generates castling moves for given Color
    fn gen_castling_moves(&self, active_color: Color, move_list: &mut MoveList) {
        // NOTE: Castling Permission will only be available if King hasn't moved
        // and Rook hasn't either. We won't be checking that here.
        match active_color {
            Color::White => {
                let non_active_color = Color::Black;

                // Check if White has Kingside Castling Permission
                if (self.castle_permissions.0 & (Castle::WhiteKing as u8)) > 0
                    // Check that squares between King and Rook are empty
                    && self.board.pieces[Square::F1 as usize].is_none()
                    && self.board.pieces[Square::G1 as usize].is_none()
                    // The King can't start in check and any square the King crosses
                    // or ends up in can't be attacked.
                    // NOTE: we won't check the square that the King would land on
                    // since we will be checking that when actually trying to make the move
                    // and we don't want to do duplicate work if we can avoid it
                    && !self.is_square_attacked(non_active_color, Square::E1)
                    && !self.is_square_attacked(non_active_color, Square::F1)
                {
                    move_list.add_move(Move::new(
                        Square::E1,
                        Square::G1,
                        None,
                        false,
                        false,
                        None,
                        true,
                        Piece::WhiteKing,
                    ));
                }

                // Check if White has Queenside Castling Permission
                if (self.castle_permissions.0 & (Castle::WhiteQueen as u8)) > 0
                    && self.board.pieces[Square::D1 as usize].is_none()
                    && self.board.pieces[Square::C1 as usize].is_none()
                    && self.board.pieces[Square::B1 as usize].is_none()
                    && !self.is_square_attacked(non_active_color, Square::E1)
                    && !self.is_square_attacked(non_active_color, Square::D1)
                {
                    move_list.add_move(Move::new(
                        Square::E1,
                        Square::C1,
                        None,
                        false,
                        false,
                        None,
                        true,
                        Piece::WhiteKing,
                    ));
                }
            }
            Color::Black => {
                let non_active_color = Color::White;

                // Check if Black has Kingside Castling Permission
                if (self.castle_permissions.0 & (Castle::BlackKing as u8)) > 0
                    && self.board.pieces[Square::F8 as usize].is_none()
                    && self.board.pieces[Square::G8 as usize].is_none()
                    && !self.is_square_attacked(non_active_color, Square::E8)
                    && !self.is_square_attacked(non_active_color, Square::F8)
                {
                    move_list.add_move(Move::new(
                        Square::E8,
                        Square::G8,
                        None,
                        false,
                        false,
                        None,
                        true,
                        Piece::BlackKing,
                    ));
                }

                // Check if Black has Queenside Castling Permission
                if (self.castle_permissions.0 & (Castle::BlackQueen as u8)) > 0
                    && self.board.pieces[Square::D8 as usize].is_none()
                    && self.board.pieces[Square::C8 as usize].is_none()
                    && self.board.pieces[Square::B8 as usize].is_none()
                    && !self.is_square_attacked(non_active_color, Square::E8)
                    && !self.is_square_attacked(non_active_color, Square::D8)
                {
                    move_list.add_move(Move::new(
                        Square::E8,
                        Square::C8,
                        None,
                        false,
                        false,
                        None,
                        true,
                        Piece::BlackKing,
                    ));
                }
            }
        }
    }

    /// Generates quite moves and captures for non pawn Pieces of specified active Color
    fn gen_non_pawn_moves(&self, active_color: Color, move_list: &mut MoveList) {
        let non_sliding_pieces = gen_non_sliding_pieces!(active_color);
        let sliding_pieces = gen_sliding_pieces!(active_color);

        let non_active_color = match active_color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        for piece in non_sliding_pieces {
            let piece_count = self.board.get_piece_count()[piece as usize];
            for piece_index in 0_usize..piece_count as usize {
                // get square that the current piece we are looking at is on
                let start_square = self.board.get_piece_list()[piece as usize][piece_index];
                // get all directions to check move validity
                let directions = piece.get_attack_directions();

                // check if piece can move to square in each direction
                for direction in directions {
                    let end_square = start_square + direction;
                    if let Ok(end_square) = end_square {
                        // If the square current piece is trying to move to is empty
                        // then that move is valid so add it to the move_list
                        // otherwise it's only a valid move if the occupying piece
                        // is of the non-active color
                        match self.board.pieces[end_square as usize] {
                            // NOTE: you cannot capture while castling
                            Some(end_piece) => {
                                // valid capture
                                if end_piece.get_color() == non_active_color {
                                    move_list.add_move(Move::new(
                                        start_square,
                                        end_square,
                                        Some(end_piece),
                                        false,
                                        false,
                                        None,
                                        false,
                                        piece,
                                    ));
                                }
                            }
                            None => {
                                move_list.add_move(Move::new(
                                    start_square,
                                    end_square,
                                    None,
                                    false,
                                    false,
                                    None,
                                    false,
                                    piece,
                                ));
                            }
                        }
                    }
                }
            }
        }

        for piece in sliding_pieces {
            let piece_count = self.board.get_piece_count()[piece as usize];

            for piece_index in 0_usize..piece_count as usize {
                let start_square = self.board.get_piece_list()[piece as usize][piece_index];
                let directions = piece.get_attack_directions();

                for direction in directions {
                    // deal with sliding
                    let mut next_square = start_square;
                    while let Ok(end_square) = next_square + direction {
                        match self.board.pieces[end_square as usize] {
                            // capture (can't be castling)
                            Some(end_piece) => {
                                // valid capture
                                if end_piece.get_color() == non_active_color {
                                    move_list.add_move(Move::new(
                                        start_square,
                                        end_square,
                                        Some(end_piece),
                                        false,
                                        false,
                                        None,
                                        false,
                                        piece,
                                    ));
                                }
                                // if you hit a piece you can't keep sliding
                                break;
                            }
                            // No capture
                            None => {
                                move_list.add_move(Move::new(
                                    start_square,
                                    end_square,
                                    None,
                                    false,
                                    false,
                                    None,
                                    false,
                                    piece,
                                ));
                            }
                        }
                        // set up for next slide check
                        next_square = end_square;
                    }
                }
            }
        }
    }

    /// Generates quiet moves (including starting double move forward), captures (including en passant)
    /// and all promotions for Pawn of specified active Color
    fn gen_pawn_moves(&self, active_color: Color, move_list: &mut MoveList) {
        // Setup all color-dependent values to make the rest of the logic color independent
        let (
            pawn,
            vertical_direction,
            start_rank,
            promotion_rank,
            promotion_targets,
            attack_directions,
            non_active_color,
            non_active_pawn,
        ) = match active_color {
            Color::White => {
                let pawn = Piece::WhitePawn;
                let pawn_vertical_direction = WHITE_PAWN_VERTICAL_DIRECTION;
                let pawn_start_rank = Rank::Rank2;
                let pawn_promotion_rank = Rank::Rank7; // Rank right before promotion occurs
                let pawn_promotion_targets = WHITE_PAWN_PROMOTION_TARGETS;
                let pawn_attack_directions = pawn.get_attack_directions();
                let non_active_color = Color::Black;
                let non_active_pawn = Piece::BlackPawn;
                (
                    pawn,
                    pawn_vertical_direction,
                    pawn_start_rank,
                    pawn_promotion_rank,
                    pawn_promotion_targets,
                    pawn_attack_directions,
                    non_active_color,
                    non_active_pawn,
                )
            }
            Color::Black => {
                let pawn = Piece::BlackPawn;
                let pawn_vertical_direction = BLACK_PAWN_VERTICAL_DIRECTION;
                let pawn_start_rank = Rank::Rank7;
                let pawn_promotion_rank = Rank::Rank2; // Rank right before promotion occurs
                let pawn_promotion_targets = BLACK_PAWN_PROMOTION_TARGETS;
                let pawn_attack_directions = pawn.get_attack_directions();
                let non_active_color = Color::White;
                let non_active_pawn = Piece::WhitePawn;
                (
                    pawn,
                    pawn_vertical_direction,
                    pawn_start_rank,
                    pawn_promotion_rank,
                    pawn_promotion_targets,
                    pawn_attack_directions,
                    non_active_color,
                    non_active_pawn,
                )
            }
        };

        let pawn_count = self.board.get_piece_count()[pawn as usize] as usize;

        for pawn_index in 0_usize..pawn_count {
            let start_square = self.board.get_piece_list()[pawn as usize][pawn_index];

            // Check Pawn forward moves
            let square_ahead = (start_square + vertical_direction);
            if let Ok(square_ahead) = square_ahead {
                let rank = start_square.get_rank();

                let mut is_pawn_start = false;
                let mut is_promotion = false;

                // Add move to move_list if square ahead is empty (possibly two ahead as well)
                if self.board.pieces[square_ahead as usize].is_none() {
                    match rank {
                        // Check if pawn start
                        pawn_start_rank if (pawn_start_rank == start_rank) => {
                            is_pawn_start = true;

                            // Add pawn moves one ahead
                            let _move = Move::new(
                                start_square,
                                square_ahead,
                                None,
                                false,
                                is_pawn_start,
                                None,
                                false,
                                pawn,
                            );

                            move_list.add_move(_move);

                            // Add move two ahead if square vacant
                            let square_two_ahead = (start_square + (vertical_direction * 2));
                            if let Ok(square_two_ahead) = square_two_ahead {
                                if self.board.pieces[square_two_ahead as usize].is_none() {
                                    let _move = Move::new(
                                        start_square,
                                        square_two_ahead,
                                        None,
                                        false,
                                        is_pawn_start,
                                        None,
                                        false,
                                        pawn,
                                    );

                                    move_list.add_move(_move);
                                }
                            }
                        }

                        // Check if promotion (one ahead)
                        pawn_promotion_rank if (pawn_promotion_rank == promotion_rank) => {
                            is_promotion = true; // NOTE: promotion is mandatory

                            for promotion in promotion_targets {
                                let _move = Move::new(
                                    start_square,
                                    square_ahead,
                                    None,
                                    false,
                                    is_pawn_start,
                                    Some(promotion),
                                    false,
                                    pawn,
                                );

                                move_list.add_move(_move);
                            }
                        }
                        _ => {
                            // Add pawn moves one ahead
                            let _move = Move::new(
                                start_square,
                                square_ahead,
                                None,
                                false,
                                is_pawn_start,
                                None,
                                false,
                                pawn,
                            );

                            move_list.add_move(_move);
                        }
                    }
                }

                // Generate Capture Moves
                for &direction in attack_directions.iter() {
                    // Check if there is a valid square in that direction occupied by a non-active color Piece
                    // or if the square is an En Passant square. And deal with promotions
                    let attacked_square = start_square + direction;
                    // square in direction valid
                    if let Ok(attacked_square) = attacked_square {
                        let piece_captured = self.board.pieces[attacked_square as usize];
                        match piece_captured {
                            Some(piece_captured) => {
                                // square in direction occupied by takeable piece
                                if piece_captured.get_color() == non_active_color {
                                    match is_promotion {
                                        // taking piece would result in promotion
                                        true => {
                                            for promotion in promotion_targets {
                                                let _move = Move::new(
                                                    start_square,
                                                    attacked_square,
                                                    Some(piece_captured),
                                                    false,
                                                    is_pawn_start,
                                                    Some(promotion),
                                                    false,
                                                    pawn,
                                                );

                                                move_list.add_move(_move);
                                            }
                                        }

                                        false => {
                                            let _move = Move::new(
                                                start_square,
                                                attacked_square,
                                                Some(piece_captured),
                                                false,
                                                is_pawn_start,
                                                None,
                                                false,
                                                pawn,
                                            );

                                            move_list.add_move(_move);
                                        }
                                    }
                                }
                            }

                            // Could be an En Passant Capture (won't result in promotion)
                            None => {
                                if self.en_passant == Some(Square64::from(attacked_square)) {
                                    // if somehow there is an en_passant square but the square in front
                                    // of it is invalid, something went very wrong
                                    let capture_square = (attacked_square - vertical_direction)
                                        .expect(
                                            "Square ahead of En Passant Square should be valid",
                                        );

                                    // get piece that is being captured via en passant
                                    // if there isn't a non-active color Pawn in front of the en passant square
                                    // we're in trouble
                                    let piece_captured = self.board.pieces[capture_square as usize]
                                    .expect("Square in front of En Passant Square needs to be occupied");

                                    assert_eq!(piece_captured,
                                        non_active_pawn,
                                        "Square in front of En Passant Square needs to be occupied by Pawn of non-active color");

                                    let _move = Move::new(
                                        start_square,
                                        attacked_square,
                                        Some(piece_captured), // better be a Pawn of non-active color
                                        true,
                                        false, // can't take en passant from a pawn start
                                        None,  // can't be a promotion
                                        false,
                                        pawn,
                                    );

                                    move_list.add_move(_move);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generate all possible moves for the current Gamestate
    pub fn gen_move_list(&self) -> Result<MoveList, MoveGenError> {
        // TODO: might be useful to turn strict off
        self.check_gamestate(ValidityCheck::Strict)?;

        let mut move_list = MoveList::new();

        self.gen_pawn_moves(self.active_color, &mut move_list);
        self.gen_non_pawn_moves(self.active_color, &mut move_list);
        self.gen_castling_moves(self.active_color, &mut move_list);

        Ok(move_list)
    }

    //=========================== BUILDING ==============================

    /// Generate a hash that represents the current position via Zobrist Hashing
    fn init_position_key(&mut self) {
        let mut position_key: u64 = 0;

        // Color (which player's turn) component
        if self.active_color == Color::White {
            let color_key = ZOBRIST
                .lock()
                .expect("Mutex holding ZOBRIST should not be poisoned")
                .color_key;

            // Note Color::Black is encoded via absence
            position_key ^= color_key;
        };

        // Piece location component
        for (square_index, piece_at_square) in self.board.pieces.iter().enumerate() {
            if let Some(piece) = *piece_at_square {
                let piece_keys = ZOBRIST
                    .lock()
                    .expect("Mutex holding ZOBRIST should not be poisoned")
                    .piece_keys;

                // for each piece present on the board find its randomly generated value in the Zobrist
                // struct's piece_keys array and XOR with the current Gamestate's position_key
                position_key ^= piece_keys[piece as usize][idx_120_to_64!(square_index)];
            }
        }

        // En Passant component
        if let Some(square) = self.en_passant {
            let en_passant_keys = ZOBRIST
                .lock()
                .expect("Mutex holding ZOBRIST should not be poisoned")
                .en_passant_keys;

            position_key ^= en_passant_keys[square.get_file() as usize];
        }

        // Castle Permissions component
        let castle_keys = ZOBRIST
            .lock()
            .expect("Mutex holding ZOBRIST should not be poisoned")
            .castle_keys;

        position_key ^= castle_keys[self.castle_permissions.0 as usize];

        self.position_key = PositionKey(position_key);
    }

    /// Check that the gamestate is valid for the given a validity check mode
    pub fn check_gamestate(
        &self,
        validity_check: ValidityCheck,
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

            // check that fullmove count is in valid range 1..=MAX_GAME_MOVES
            if !(1..=MAX_GAME_MOVES).contains(&self.fullmove_count) {
                return Err(GamestateValidityCheckError::StrictFullmoveCountNotInRange {
                    fullmove_count: self.fullmove_count,
                });
            }

            // check that fullmove count and halfmove clock are plausible
            // NOTE: fullmove_count starts at 1 and increments every time black moves
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
            if (2 * (self.fullmove_count - 1) + self.active_color as usize)
                < self.halfmove_clock as usize
            {
                return Err(
                GamestateValidityCheckError::StrictFullmoveCountLessThanHalfmoveClockDividedByTwo {
                    fullmove_count: self.fullmove_count,
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

        // fullmove_count
        fen.push_str(self.fullmove_count.to_string().as_str());

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
            // TODO: this is technically doing extra work, but it's clearer and
            // easy for now (could also be nice if we add fantasy pieces with
            // non-symmetric movements).

            // Reverse directions since we're trying to find where attacks could be
            // initiated from and not where a piece could move to. Only matters for
            // pawns
            let directions = piece
                .get_attack_directions()
                .into_iter()
                .map(|direction| -direction)
                .collect::<Vec<_>>();

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
        gamestate, position_key,
    };

    fn assert_fuzzy_eq(output: &Gamestate, expected: &Gamestate) {
        // board.pieces
        assert_eq!(output.board.pieces, expected.board.pieces);
        // board.pawns
        assert_eq!(output.board.pawns, expected.board.pawns);
        // board.kings_square
        assert_eq!(output.board.kings_square, expected.board.kings_square);
        // board.piece_count
        assert_eq!(output.board.piece_count, expected.board.piece_count);
        // board.big_piece_count
        assert_eq!(output.board.big_piece_count, expected.board.big_piece_count);
        // board.major_piece_count
        assert_eq!(
            output.board.major_piece_count,
            expected.board.major_piece_count
        );
        // board.minor_piece_count
        assert_eq!(
            output.board.minor_piece_count,
            expected.board.minor_piece_count
        );
        // board.material_score
        assert_eq!(output.board.material_score, expected.board.material_score);

        // board.piece_list order doesn't matter
        let mut output_piece_list_sorted = [
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        for (index, _squares) in output.board.piece_list.iter().enumerate() {
            output_piece_list_sorted[index] = _squares.clone();
            output_piece_list_sorted[index].sort();
        }
        let mut expected_piece_list_sorted = [
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ];
        for (index, _squares) in expected.board.piece_list.iter().enumerate() {
            expected_piece_list_sorted[index] = _squares.clone();
            expected_piece_list_sorted[index].sort();
        }
        assert_eq!(output_piece_list_sorted, expected_piece_list_sorted);

        // active_color
        assert_eq!(output.active_color, expected.active_color);
        // castle_permissions
        assert_eq!(output.castle_permissions, expected.castle_permissions);
        // en_passant
        assert_eq!(output.en_passant, expected.en_passant);
        // halfmove_clock
        assert_eq!(output.halfmove_clock, expected.halfmove_clock);
        // fullmove_count
        assert_eq!(output.fullmove_count, expected.fullmove_count);
        // history
        assert_eq!(output.history, expected.history);
        // position_key
        assert_eq!(output.position_key, expected.position_key);
    }

    //======================== MAKE MOVES =====================================

    // MOVE PIECE
    #[test]
    fn test_gamestate_move_piece_valid() {
        let fen = DEFAULT_FEN;
        let mut output = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        // move WhiteKnight from B1 to C3 which is empty
        output.move_piece(Square::B1, Square::C3);

        let fen_after_add = "rnbqkbnr/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1";
        let expected = GamestateBuilder::new_with_fen(fen_after_add)
            .unwrap()
            .build()
            .unwrap();

        assert_fuzzy_eq(&output, &expected);
    }

    #[test]
    fn test_gamestate_move_piece_invalid_no_piece_on_start_square() {
        let fen = DEFAULT_FEN;
        let mut input = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        // try to move B3 which is empty
        let output = input.move_piece(Square::B3, Square::B4);
        let expected = Err(MakeMoveError::NoPieceAtMoveStart {
            start_square: Square::B3,
        });

        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_move_piece_invalid_attempted_capture() {
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/8/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1";
        let mut input = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        // try to move WhiteKnight on C3 to D5 which is occupied by BlackPawn
        let output = input.move_piece(Square::C3, Square::D5);
        let expected = Err(MakeMoveError::MoveEndsOnOccupiedSquare {
            piece: Piece::WhiteKnight,
            end_square: Square::D5,
            end_piece: Piece::BlackPawn,
        });
        assert_eq!(output, expected);
    }

    // ADD PIECE
    #[test]
    fn test_gamestate_add_piece_valid() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R1BQKBNR w KQkq - 0 1";
        let mut output = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        // add WhiteKnight to C3
        output.add_piece(Square::C3, Piece::WhiteKnight);

        let fen_after_add = "rnbqkbnr/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1";
        let expected = GamestateBuilder::new_with_fen(fen_after_add)
            .unwrap()
            .build()
            .unwrap();
    }

    #[test]
    fn test_gamestate_add_piece_invalid_square_occupied() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R1BQKBNR w KQkq - 0 1";
        let mut input = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        // add WhiteKnight to D2 which is occupied by WhitePawn
        let output = input.add_piece(Square::D2, Piece::WhiteKnight);

        let expected = Err(MakeMoveError::AddToOccupiedSquare {
            occupied_square: Square::D2,
            piece_at_square: Piece::WhitePawn,
        });

        assert_eq!(output, expected);
    }

    // CLEAR PIECE
    #[test]
    fn test_gamestate_clear_piece_valid() {
        let fen = "8/8/8/8/8/8/3P4/8 w - - 0 1";
        let mut output = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        output.clear_piece(Square::D2);

        let expected = GamestateBuilder::new()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        assert_eq!(output, expected);
    }

    // NOTE: it's important to test a non white pawn move/add/clear since you can easily mix up Piece and PieceType values
    // and for a white pawn both convert to usize 0
    #[test]
    fn test_gamestate_clear_piece_non_white_pawn_valid() {
        // Black Knight's Piece usize value is 7 but PieceType is 1
        // output.board.piece_list[piece_type as usize] is the vec for WhiteKnight
        // so this fen should doubly cause an issue if we mess this up
        let fen = "N7/8/8/8/8/8/3n4/8 w - - 0 1";
        let mut output = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        output.clear_piece(Square::D2);

        let expected = GamestateBuilder::new_with_board(
            BoardBuilder::new()
                .validity_check(ValidityCheck::Basic)
                .piece(Piece::WhiteKnight, Square64::A8)
                .build()
                .unwrap(),
        )
        .validity_check(ValidityCheck::Basic)
        .build()
        .unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_clear_piece_start_valid() {
        let fen = DEFAULT_FEN;
        let mut output = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        output.clear_piece(Square::D2);

        let fen_after_clear = "rnbqkbnr/pppppppp/8/8/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1";
        let mut expected = GamestateBuilder::new_with_fen(fen_after_clear)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        assert_fuzzy_eq(&output, &expected);
    }

    #[test]
    fn test_gamestate_clear_piece_invalid() {
        let fen = "8/8/8/8/8/8/3P4/8 w - - 0 1";
        let mut input = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        let output = input.clear_piece(Square::D1);

        let expected = Err(MakeMoveError::NoPieceToClear {
            empty_square: Square::D1,
        });

        assert_eq!(output, expected);
    }

    //======================== POSITION KEY ===================================

    #[test]
    fn test_gamestate_init_position_key_one_white_pawn() {
        let fen = "8/8/8/8/8/8/3P4/8 w - - 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        let output = gamestate.position_key;

        let mut position_key_value = 0;
        let zobrist = ZOBRIST.lock().unwrap();
        let color_key_component = zobrist.color_key;
        let piece_keys_component =
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::D2 as usize];
        let castle_keys_component = zobrist.castle_keys[0];

        position_key_value ^= color_key_component;
        position_key_value ^= piece_keys_component;
        position_key_value ^= castle_keys_component;

        let expected = PositionKey(position_key_value);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_init_position_key_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        let output = gamestate.position_key;

        let mut position_key_value = 0;
        let zobrist = ZOBRIST.lock().unwrap();
        let color_key_component = zobrist.color_key;

        let mut piece_keys_component = 0;
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhiteRook as usize][Square64::A1 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhiteKnight as usize][Square64::B1 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhiteBishop as usize][Square64::C1 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhiteQueen as usize][Square64::D1 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhiteKing as usize][Square64::E1 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhiteBishop as usize][Square64::F1 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhiteKnight as usize][Square64::G1 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhiteRook as usize][Square64::H1 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::A2 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::B2 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::C2 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::D2 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::E2 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::F2 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::G2 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::WhitePawn as usize][Square64::H2 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackPawn as usize][Square64::A7 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackPawn as usize][Square64::B7 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackPawn as usize][Square64::C7 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackPawn as usize][Square64::D7 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackPawn as usize][Square64::E7 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackPawn as usize][Square64::F7 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackPawn as usize][Square64::G7 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackPawn as usize][Square64::H7 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackRook as usize][Square64::A8 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackKnight as usize][Square64::B8 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackBishop as usize][Square64::C8 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackQueen as usize][Square64::D8 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackKing as usize][Square64::E8 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackBishop as usize][Square64::F8 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackKnight as usize][Square64::G8 as usize];
        piece_keys_component ^=
            zobrist.piece_keys[Piece::BlackRook as usize][Square64::H8 as usize];

        let castle_keys_component = zobrist.castle_keys[15];

        position_key_value ^= color_key_component;
        position_key_value ^= piece_keys_component;
        position_key_value ^= castle_keys_component;

        let expected = PositionKey(position_key_value);
        assert_eq!(output, expected);
    }

    //========================= MOVE GEN ======================================

    #[test]
    fn test_gamestate_move_gen_all_moves_tricky_visual() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        let output = gamestate.gen_move_list().unwrap();

        println!("{}", output);
        assert_eq!(output.count, 48);
    }

    #[test]
    fn test_gamestate_move_gen_castling_moves_basic_black() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        let mut output = MoveList::new();
        gamestate.gen_castling_moves(Color::Black, &mut output);

        let mut expected = MoveList::new();

        // Black Kingside Castle
        expected.add_move(Move::new(
            Square::E8,
            Square::G8,
            None,
            false,
            false,
            None,
            true,
            Piece::BlackKing,
        ));

        //Black Queenside Castle
        expected.add_move(Move::new(
            Square::E8,
            Square::C8,
            None,
            false,
            false,
            None,
            true,
            Piece::BlackKing,
        ));

        println!("Output:\n{}", output);
        println!("Expected:\n{}", expected);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_move_gen_castling_moves_basic_white() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .build()
            .unwrap();

        let mut output = MoveList::new();
        gamestate.gen_castling_moves(Color::White, &mut output);

        let mut expected = MoveList::new();

        // White Kingside Castle
        expected.add_move(Move::new(
            Square::E1,
            Square::G1,
            None,
            false,
            false,
            None,
            true,
            Piece::WhiteKing,
        ));

        // White Queenside Castle
        expected.add_move(Move::new(
            Square::E1,
            Square::C1,
            None,
            false,
            false,
            None,
            true,
            Piece::WhiteKing,
        ));

        println!("Output:\n{}", output);
        println!("Expected:\n{}", expected);

        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_move_gen_castling_moves_under_attack() {
        let fen = "r3k2r/8/8/8/8/8/6p1/R3K2R w KQk - 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        let mut output = MoveList::new();
        gamestate.gen_castling_moves(Color::White, &mut output);

        let mut expected = MoveList::new();

        // White Kingside Castle blocked by Black Pawn on G2
        // White Queenside Castle is valid
        expected.add_move(Move::new(
            Square::E1,
            Square::C1,
            None,
            false,
            false,
            None,
            true,
            Piece::WhiteKing,
        ));

        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_move_gen_sliding_rooks() {
        let fen = "8/8/2p5/8/1pR1P3/8/8/8 w - - 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        let mut output = MoveList::new();
        gamestate.gen_non_pawn_moves(Color::White, &mut output);

        let mut expected = MoveList::new();

        //==================== WHITE ROOK C4 ==================================
        // WR C4 to C3 (1 Down)
        expected.add_move(Move::new(
            Square::C4,
            Square::C3,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteRook,
        ));

        // WR C4 to C2 (2 Down)
        expected.add_move(Move::new(
            Square::C4,
            Square::C2,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteRook,
        ));

        // WR C4 to C1 (3 Down)
        expected.add_move(Move::new(
            Square::C4,
            Square::C1,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteRook,
        ));

        // WR C4 captures BP on B4 (1 Left)
        expected.add_move(Move::new(
            Square::C4,
            Square::B4,
            Some(Piece::BlackPawn),
            false,
            false,
            None,
            false,
            Piece::WhiteRook,
        ));

        // Further moves in that direction blocked

        // WR C4 to D4 (1 Right)
        expected.add_move(Move::new(
            Square::C4,
            Square::D4,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteRook,
        ));

        // Further moves in that direction blocked

        // WR C4 to C5 (1 Up)
        expected.add_move(Move::new(
            Square::C4,
            Square::C5,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteRook,
        ));

        // WR C4 captured BP on C6 (2 Up)
        expected.add_move(Move::new(
            Square::C4,
            Square::C6,
            Some(Piece::BlackPawn),
            false,
            false,
            None,
            false,
            Piece::WhiteRook,
        ));

        let output_count = output.count;
        let expected_count = expected.count;

        println!("OUTPUT:\n{}", output);
        println!("\n\n\nEXPECTED:\n{}", expected);

        assert_eq!(output_count, expected_count);

        let mut output = output.moves.into_iter().flatten().collect::<Vec<Move>>(); // get rid of Nones
        let mut expected = expected.moves.into_iter().flatten().collect::<Vec<Move>>();
        output.sort();
        expected.sort();

        assert_eq!(expected_count, output.len());

        assert_eq!(output, expected);
    }

    // TODO:
    // Queens
    // let fen = "6k1/8/4nq2/8/1nQ5/5N2/1N6/6K1 w - - 0 1";
    // Bishops
    // let fen = "6k1/1b6/4n3/8/1n4B1/1B3N2/1N6/2b3K1 b - - 0 1";

    #[test]
    fn test_gamestate_move_gen_knights_kings() {
        let fen = "5k2/1n6/4n3/6N1/8/3N4/8/5K2 w - - 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        let mut output = MoveList::new();
        gamestate.gen_non_pawn_moves(Color::White, &mut output);
        gamestate.gen_non_pawn_moves(Color::Black, &mut output);

        let mut expected = MoveList::new();

        //==================== WHITE KING ====================
        // WK F1 to E1 (Left)
        expected.add_move(Move::new(
            Square::F1,
            Square::E1,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKing,
        ));

        // WK F1 to G1 (Right)
        expected.add_move(Move::new(
            Square::F1,
            Square::G1,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKing,
        ));

        // WK F1 to E2 (Up Left)
        expected.add_move(Move::new(
            Square::F1,
            Square::E2,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKing,
        ));

        // WK F1 to F2 (Up)
        expected.add_move(Move::new(
            Square::F1,
            Square::F2,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKing,
        ));

        // WK F1 to G2 (Up Right)
        expected.add_move(Move::new(
            Square::F1,
            Square::G2,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKing,
        ));

        //============== WHITE KNIGHT D3 =====================

        // WN D3 to C1 (Down 2 Left 1)
        expected.add_move(Move::new(
            Square::D3,
            Square::C1,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN D3 to E1 (Down 2 Right 1)
        expected.add_move(Move::new(
            Square::D3,
            Square::E1,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN D3 to B2 (Down 1 Left 2)
        expected.add_move(Move::new(
            Square::D3,
            Square::B2,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN D3 to F2 (Down 1 Right 2)
        expected.add_move(Move::new(
            Square::D3,
            Square::F2,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN D3 to B4 (Up 1 Left 2)
        expected.add_move(Move::new(
            Square::D3,
            Square::B4,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN D3 to F4 (Up 1 Right 2)
        expected.add_move(Move::new(
            Square::D3,
            Square::F4,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN D3 to C5 (Up 2 Left 1)
        expected.add_move(Move::new(
            Square::D3,
            Square::C5,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN D3 to E5 (Up 2 Right 1)
        expected.add_move(Move::new(
            Square::D3,
            Square::E5,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        //============== WHITE KNIGHT G5 =====================

        // WN G5 to F3 (Down 2 Left 1)
        expected.add_move(Move::new(
            Square::G5,
            Square::F3,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN G5 to H3 (Down 2 Right 1)
        expected.add_move(Move::new(
            Square::G5,
            Square::H3,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN G5 to E4 (Down 1 Left 2)
        expected.add_move(Move::new(
            Square::G5,
            Square::E4,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN G5 (Down 1 Right 2) is offboard

        // WN G5 Captures BN on E6 (Up 1 Left 2)
        expected.add_move(Move::new(
            Square::G5,
            Square::E6,
            Some(Piece::BlackKnight),
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN G5 (Up 1 Right 2) is offboard

        // WN G5 to F7 (Up 2 Left 1)
        expected.add_move(Move::new(
            Square::G5,
            Square::F7,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        // WN G5 to H7 (Up 2 Right 1)
        expected.add_move(Move::new(
            Square::G5,
            Square::H7,
            None,
            false,
            false,
            None,
            false,
            Piece::WhiteKnight,
        ));

        //================== BLACK KNIGHT E6 ==============

        // BN E6 to D4 (Down 2 Left 1)
        expected.add_move(Move::new(
            Square::E6,
            Square::D4,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK E6 to F4 (Down 2 Right 1)
        expected.add_move(Move::new(
            Square::E6,
            Square::F4,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK E6 to C5 (Down 1 Left 2)
        expected.add_move(Move::new(
            Square::E6,
            Square::C5,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK F8 capture WN on G5 (Down 1 Right 2)
        expected.add_move(Move::new(
            Square::E6,
            Square::G5,
            Some(Piece::WhiteKnight),
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK E6 to C7 (Up 1 Left 2)
        expected.add_move(Move::new(
            Square::E6,
            Square::C7,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK E6 to G7 (Up 1 Right 2)
        expected.add_move(Move::new(
            Square::E6,
            Square::G7,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK E6 to D8 (Up 2 Left 1)
        expected.add_move(Move::new(
            Square::E6,
            Square::D8,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK E6 to F8 blocked by BK

        //================== BLACK KNIGHT B7 ==============

        // BK B7 to A5 (Down 2 Left 1)
        expected.add_move(Move::new(
            Square::B7,
            Square::A5,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK B7 to C5 (Down 2 Right 1)
        expected.add_move(Move::new(
            Square::B7,
            Square::C5,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK B7 (1 Down 2 Left) offboard

        // BK B7 to D6 (1 Down 2 Right)
        expected.add_move(Move::new(
            Square::B7,
            Square::D6,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK B7 (1 Up 2 Left) offboard

        // BK B7 to D8 (1 Up 2 Right)
        expected.add_move(Move::new(
            Square::B7,
            Square::D8,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKnight,
        ));

        // BK B7 (2 Up 1 Left) offboard
        // BK B7 (2 Up 1 Right) offboard

        //================== BLACK KING ===================
        // BK F8 to E7 (Down Left)
        expected.add_move(Move::new(
            Square::F8,
            Square::E7,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKing,
        ));

        // BK F8 to F7 (Down)
        expected.add_move(Move::new(
            Square::F8,
            Square::F7,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKing,
        ));

        // BK F8 to G7 (Down Right)
        expected.add_move(Move::new(
            Square::F8,
            Square::G7,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKing,
        ));

        // BK F8 to E8 (Left)
        expected.add_move(Move::new(
            Square::F8,
            Square::E8,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKing,
        ));

        // BK F8 to G8 (Right)
        expected.add_move(Move::new(
            Square::F8,
            Square::G8,
            None,
            false,
            false,
            None,
            false,
            Piece::BlackKing,
        ));

        let output_count = output.count;
        let expected_count = expected.count;

        println!("OUTPUT:\n{}", output);
        println!("\n\n\nEXPECTED:\n{}", expected);

        assert_eq!(output_count, expected_count);

        let mut output = output.moves.into_iter().flatten().collect::<Vec<Move>>(); // get rid of Nones
        let mut expected = expected.moves.into_iter().flatten().collect::<Vec<Move>>();
        output.sort();
        expected.sort();

        assert_eq!(expected_count, output.len());

        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_move_gen_black_pawn() {
        let fen = "rnbqkbnr/p1p1p3/3p3p/1p1p4/2P1Pp2/8/PP1P1PpP/RNBQKB1R b - e3 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        let mut output = MoveList::new();
        gamestate.gen_pawn_moves(Color::Black, &mut output);

        let piece_moved = Piece::BlackPawn;

        let mut expected = MoveList::new();

        // BP A7 move ahead one
        expected.add_move(Move::new(
            Square::A7,
            Square::A6,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // BP A7 move ahead two
        expected.add_move(Move::new(
            Square::A7,
            Square::A5,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // BP B5 move ahead one
        expected.add_move(Move::new(
            Square::B5,
            Square::B4,
            None,
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // BP B5 move capture WP on C4
        expected.add_move(Move::new(
            Square::B5,
            Square::C4,
            Some(Piece::WhitePawn),
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // BP C7 move ahead one
        expected.add_move(Move::new(
            Square::C7,
            Square::C6,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // BP C7 move ahead two
        expected.add_move(Move::new(
            Square::C7,
            Square::C5,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // BP D6 is blocked by BP D5

        // BP D5 move ahead one
        expected.add_move(Move::new(
            Square::D5,
            Square::D4,
            None,
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // BP D5 move capture WP on C4
        expected.add_move(Move::new(
            Square::D5,
            Square::C4,
            Some(Piece::WhitePawn),
            false,
            false,
            None,
            false,
            piece_moved,
        ));
        // BP D5 move capture WP on E4
        expected.add_move(Move::new(
            Square::D5,
            Square::E4,
            Some(Piece::WhitePawn),
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // BP E7 move ahead one
        expected.add_move(Move::new(
            Square::E7,
            Square::E6,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // BP E7 move ahead two
        expected.add_move(Move::new(
            Square::E7,
            Square::E5,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // BP F4 move ahead one
        expected.add_move(Move::new(
            Square::F4,
            Square::F3,
            None,
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // BP F4 move capture WP on E3 via En Passant
        expected.add_move(Move::new(
            Square::F4,
            Square::E3,
            Some(Piece::WhitePawn),
            true,
            false,
            None,
            false,
            piece_moved,
        ));

        // BP G2 move ahead one promote BlackKnight
        expected.add_move(Move::new(
            Square::G2,
            Square::G1,
            None,
            false,
            false,
            Some(Piece::BlackKnight),
            false,
            piece_moved,
        ));

        // BP G2 move ahead one promote BlackBishop
        expected.add_move(Move::new(
            Square::G2,
            Square::G1,
            None,
            false,
            false,
            Some(Piece::BlackBishop),
            false,
            piece_moved,
        ));

        // BP G2 move ahead one promote BlackRook
        expected.add_move(Move::new(
            Square::G2,
            Square::G1,
            None,
            false,
            false,
            Some(Piece::BlackRook),
            false,
            piece_moved,
        ));

        // BP G2 move ahead one promote BlackQueen
        expected.add_move(Move::new(
            Square::G2,
            Square::G1,
            None,
            false,
            false,
            Some(Piece::BlackQueen),
            false,
            piece_moved,
        ));

        // BP G2 move capture WB on F1 promote BlackKnight
        expected.add_move(Move::new(
            Square::G2,
            Square::F1,
            Some(Piece::WhiteBishop),
            false,
            false,
            Some(Piece::BlackKnight),
            false,
            piece_moved,
        ));

        // BP G2 move capture WB on F1 promote BlackBishop
        expected.add_move(Move::new(
            Square::G2,
            Square::F1,
            Some(Piece::WhiteBishop),
            false,
            false,
            Some(Piece::BlackBishop),
            false,
            piece_moved,
        ));

        // BP G2 move capture WB on F1 promote BlackRook
        expected.add_move(Move::new(
            Square::G2,
            Square::F1,
            Some(Piece::WhiteBishop),
            false,
            false,
            Some(Piece::BlackRook),
            false,
            piece_moved,
        ));

        // BP G2 move capture WB on F1 promote BlackQueen
        expected.add_move(Move::new(
            Square::G2,
            Square::F1,
            Some(Piece::WhiteBishop),
            false,
            false,
            Some(Piece::BlackQueen),
            false,
            piece_moved,
        ));

        // BP G2 move capture WR on H1 promote BlackKnight
        expected.add_move(Move::new(
            Square::G2,
            Square::H1,
            Some(Piece::WhiteRook),
            false,
            false,
            Some(Piece::BlackKnight),
            false,
            piece_moved,
        ));

        // BP G2 move capture WR on H1 promote BlackBishop
        expected.add_move(Move::new(
            Square::G2,
            Square::H1,
            Some(Piece::WhiteRook),
            false,
            false,
            Some(Piece::BlackBishop),
            false,
            piece_moved,
        ));

        // BP G2 move capture WR on H1 promote BlackRook
        expected.add_move(Move::new(
            Square::G2,
            Square::H1,
            Some(Piece::WhiteRook),
            false,
            false,
            Some(Piece::BlackRook),
            false,
            piece_moved,
        ));

        // BP G2 move capture WR on H1 promote BlackQueen
        expected.add_move(Move::new(
            Square::G2,
            Square::H1,
            Some(Piece::WhiteRook),
            false,
            false,
            Some(Piece::BlackQueen),
            false,
            piece_moved,
        ));

        // BP H6 move ahead one to H5
        expected.add_move(Move::new(
            Square::H6,
            Square::H5,
            None,
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        let output_count = output.count;
        let expected_count = expected.count;

        println!("OUTPUT:\n{}", output);
        println!("\n\n\nEXPECTED:\n{}", expected);

        assert_eq!(output_count, expected_count);

        // Order doesn't need to match exactly right now since the order is
        // tricky to make intuitive
        let mut output = output.moves.into_iter().flatten().collect::<Vec<Move>>(); // get rid of Nones
        let mut expected = expected.moves.into_iter().flatten().collect::<Vec<Move>>();
        output.sort();
        expected.sort();

        assert_eq!(expected_count, output.len());

        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_move_gen_white_pawn() {
        let fen = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        let mut output = MoveList::new();
        gamestate.gen_pawn_moves(Color::White, &mut output);

        let piece_moved = Piece::WhitePawn;

        let mut expected = MoveList::new();
        // WP A2 move ahead one
        expected.add_move(Move::new(
            Square::A2,
            Square::A3,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // WP A2 move ahead two
        expected.add_move(Move::new(
            Square::A2,
            Square::A4,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // WP B4 move ahead one
        expected.add_move(Move::new(
            Square::B4,
            Square::B5,
            None,
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // WP B4 move capture BP on C5
        //           31-29   28-25 24   23-20 19 18 17-14 13-7     6-0
        //           unused  pm    cstl prom  ps ep capt  end      start
        // 33677236:    000  0001  0    0000  0  0  0111  011_1111 011_0100
        expected.add_move(Move::new(
            Square::B4,
            Square::C5,
            Some(Piece::BlackPawn),
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // WP C2 move ahead one
        expected.add_move(Move::new(
            Square::C2,
            Square::C3,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // WP C2 move ahead two
        expected.add_move(Move::new(
            Square::C2,
            Square::C4,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // WP D3 is blocked by WP D4

        // WP D4 move ahead one
        expected.add_move(Move::new(
            Square::D4,
            Square::D5,
            None,
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // WP D4 move capture BP on C5
        expected.add_move(Move::new(
            Square::D4,
            Square::C5,
            Some(Piece::BlackPawn),
            false,
            false,
            None,
            false,
            piece_moved,
        ));
        // WP D4 move capture BP on E5
        expected.add_move(Move::new(
            Square::D4,
            Square::E5,
            Some(Piece::BlackPawn),
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // WP E2 move ahead one
        expected.add_move(Move::new(
            Square::E2,
            Square::E3,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // WP E2 move ahead two
        expected.add_move(Move::new(
            Square::E2,
            Square::E4,
            None,
            false,
            true,
            None,
            false,
            piece_moved,
        ));

        // WP F5 move ahead one
        expected.add_move(Move::new(
            Square::F5,
            Square::F6,
            None,
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        // WP F5 move capture BP on E5 via En Passant E6
        expected.add_move(Move::new(
            Square::F5,
            Square::E6,
            Some(Piece::BlackPawn),
            true,
            false,
            None,
            false,
            piece_moved,
        ));

        // WP G7 move ahead one promote WhiteKnight
        expected.add_move(Move::new(
            Square::G7,
            Square::G8,
            None,
            false,
            false,
            Some(Piece::WhiteKnight),
            false,
            piece_moved,
        ));

        // WP G7 move ahead one promote WhiteBishop
        expected.add_move(Move::new(
            Square::G7,
            Square::G8,
            None,
            false,
            false,
            Some(Piece::WhiteBishop),
            false,
            piece_moved,
        ));

        // WP G7 move ahead one promote WhiteRook
        expected.add_move(Move::new(
            Square::G7,
            Square::G8,
            None,
            false,
            false,
            Some(Piece::WhiteRook),
            false,
            piece_moved,
        ));

        // WP G7 move ahead one promote WhiteQueen
        expected.add_move(Move::new(
            Square::G7,
            Square::G8,
            None,
            false,
            false,
            Some(Piece::WhiteQueen),
            false,
            piece_moved,
        ));

        // WP G7 move capture BB on F8 promote WhiteKnight
        expected.add_move(Move::new(
            Square::G7,
            Square::F8,
            Some(Piece::BlackBishop),
            false,
            false,
            Some(Piece::WhiteKnight),
            false,
            piece_moved,
        ));

        // WP G7 move capture BB on F8 promote WhiteBishop
        expected.add_move(Move::new(
            Square::G7,
            Square::F8,
            Some(Piece::BlackBishop),
            false,
            false,
            Some(Piece::WhiteBishop),
            false,
            piece_moved,
        ));

        // WP G7 move capture BB on F8 promote WhiteRook
        expected.add_move(Move::new(
            Square::G7,
            Square::F8,
            Some(Piece::BlackBishop),
            false,
            false,
            Some(Piece::WhiteRook),
            false,
            piece_moved,
        ));

        // WP G7 move capture BB on F8 promote WhiteQueen
        expected.add_move(Move::new(
            Square::G7,
            Square::F8,
            Some(Piece::BlackBishop),
            false,
            false,
            Some(Piece::WhiteQueen),
            false,
            piece_moved,
        ));

        // WP G7 move capture BR on H8 promote WhiteKnight
        expected.add_move(Move::new(
            Square::G7,
            Square::H8,
            Some(Piece::BlackRook),
            false,
            false,
            Some(Piece::WhiteKnight),
            false,
            piece_moved,
        ));

        // WP G7 move capture BR on H8 promote WhiteBishop
        expected.add_move(Move::new(
            Square::G7,
            Square::H8,
            Some(Piece::BlackRook),
            false,
            false,
            Some(Piece::WhiteBishop),
            false,
            piece_moved,
        ));

        // 10 0100 0010 1011 0001 0101 0111
        // 37,925,207
        // WP G7 move capture BR on H8 promote WhiteRook
        expected.add_move(Move::new(
            Square::G7,
            Square::H8,
            Some(Piece::BlackRook),
            false,
            false,
            Some(Piece::WhiteRook),
            false,
            piece_moved,
        ));

        // WP G7 move capture BR on H8 promote WhiteQueen
        expected.add_move(Move::new(
            Square::G7,
            Square::H8,
            Some(Piece::BlackRook),
            false,
            false,
            Some(Piece::WhiteQueen),
            false,
            piece_moved,
        ));

        // WP H3 move ahead one to H4
        expected.add_move(Move::new(
            Square::H3,
            Square::H4,
            None,
            false,
            false,
            None,
            false,
            piece_moved,
        ));

        let output_count = output.count;
        let expected_count = expected.count;

        println!("OUTPUT:\n{}", output);
        println!("\n\n\nEXPECTED:\n{}", expected);

        assert_eq!(output_count, expected_count);

        // Order doesn't need to match exactly right now since the order is
        // tricky to make intuitive
        let mut output = output.moves.into_iter().flatten().collect::<Vec<Move>>(); // get rid of Nones
        let mut expected = expected.moves.into_iter().flatten().collect::<Vec<Move>>();
        output.sort();
        expected.sort();

        assert_eq!(expected_count, output.len());

        assert_eq!(output, expected);
    }

    //========================= REUSABLE BUILDER ==============================
    #[test]
    fn test_gamestate_builder_is_reusable() {
        let mut gamestate_builder = GamestateBuilder::new_with_board(
            BoardBuilder::new()
                .validity_check(ValidityCheck::Basic)
                .piece(Piece::BlackBishop, Square64::A1)
                .build()
                .unwrap(),
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
        let fullmove_count = 1;
        let history = Vec::new();
        let position_key = PositionKey(6527259550795953174);

        let expected = Ok(Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_count,
            history,
            position_key,
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
        // fullmove_count
        assert_eq!(
            output.as_ref().unwrap().fullmove_count,
            expected.as_ref().unwrap().fullmove_count
        );
        // history
        assert_eq!(
            output.as_ref().unwrap().history,
            expected.as_ref().unwrap().history
        );
        // position_key
        assert_eq!(
            output.as_ref().unwrap().position_key,
            expected.as_ref().unwrap().position_key
        );
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
        let fullmove_count = 2;
        let history = Vec::new();

        // NOTE: this is why you shouldn't initialize Gamestate like this
        // The builder is taking care of initiallizing the position key
        let position_key = PositionKey(0);

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_count,
            history,
            position_key,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] =
                    gamestate.is_square_attacked(active_color, square);
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
        let fullmove_count = 2;
        let history = Vec::new();

        // NOTE: should use builder
        let position_key = PositionKey(0);

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_count,
            history,
            position_key,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] =
                    gamestate.is_square_attacked(active_color, square);
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
        let fullmove_count = 1;
        let history = Vec::new();

        // NOTE: should use the builder
        let position_key = PositionKey(0);

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_count,
            history,
            position_key,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] =
                    gamestate.is_square_attacked(active_color, square);
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
        let expected_position_key_start = gs_start.position_key;
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
        let expected_position_key_wpe4 = gs_wpe4.position_key;
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
        let expected_position_key_bpc5 = gs_bpc5.position_key;
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
        let expected_position_key_wnf3 = gs_wnf3.position_key;
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
        .fullmove_count(3)
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
        let fullmove: usize = 1025;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1025";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictFullmoveCountNotInRange {
                fullmove_count: fullmove,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fullmove_zero() {
        let fullmove: usize = 0;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 0";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateBuildError::GamestateValidityCheck(
            GamestateValidityCheckError::StrictFullmoveCountNotInRange {
                fullmove_count: fullmove,
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
