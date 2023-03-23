use std::fmt;

use crate::{
    color::Color,
    error::{MoveDeserializeError, MoveValidityError},
    gamestate::{Gamestate, ValidityCheck},
    piece::Piece,
    rank::Rank,
    square::{Square, Square64},
};

//====================== CONSTANTS ============================================
// For any given position this is a generous upper bound for how many different
// moves can be made from that position
pub const MAX_GAME_POSITIONS: usize = 256;

// any bit representation of a 120 square will occupy at most 7 bits
const MOVE_SQUARE_MASK: u32 = 0x7F;
// any bit representation of a piece will occupy at most 4 bits
const MOVE_PIECE_MASK: u32 = 0xF;
const MOVE_EN_PASSANT_MASK: u32 = 0x4_0000;
const MOVE_PAWN_START_MASK: u32 = 0x8_0000;
const MOVE_CASTLE_MASK: u32 = 0x100_0000;

const MOVE_IS_PROMOTED_MASK: u32 = 0xF00000;
const MOVE_IS_CAPTURE_MASK: u32 = 0x7c000; // En Passant flag and Piece Captured

const MOVE_END_SHIFT: u8 = 7;
const MOVE_PIECE_CAPTURED_SHIFT: u8 = 14;
const MOVE_PIECE_PROMOTED_SHIFT: u8 = 20;
const MOVE_PIECE_MOVED_SHIFT: u8 = 25;

//============================= MOVE GENERATION ===============================

// TODO: look into arrayvec/smallvec/tinyvec for MoveList moves
#[derive(Debug, PartialEq)]
pub struct MoveList {
    pub moves: [Option<Move>; MAX_GAME_POSITIONS],
    pub count: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: update scores
// NOTE: Doesn't need a builder since we can add_move and we won't be making
// multiple instances of a MoveList TODO: make sure this stays true
impl MoveList {
    pub fn new() -> MoveList {
        MoveList {
            moves: [None; MAX_GAME_POSITIONS],
            count: 0,
        }
    }

    // TODO: consider performance and think about inline attributes
    pub fn add_move(&mut self, _move: Move) {
        self.moves[self.count] = Some(_move);
        self.count += 1;
    }
}

impl fmt::Display for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MoveList (Count: {})", self.count);
        writeln!(f, "========================================");
        for (index, _move) in self.moves.iter().flatten().enumerate() {
            writeln!(f, "{}", _move);
            writeln!(f, "========================================");
        }
        writeln!(f)
    }
}

//============================== MOVE STRUCTURE ===============================

/// Information that defines a move is stored in a 32-bit word with the following format
///
/// 0000 0000 0000 0000 0000 0000 0111 1111 START:          0x7F       bits 0-6 represent the Square120 where the move was initiated
/// 0000 0000 0000 0000 0011 1111 1000 0000 END:            >> 7, 0x7F bits 7-13 represent the Square120 where the move ended
/// 0000 0000 0000 0011 1100 0000 0000 0000 PIECE_CAPTURED: >> 14, 0xF bits 14-17 represent the Piece that was captured (in None then 0)
/// 0000 0000 0000 0100 0000 0000 0000 0000 EN_PASSANT:     0x40000    bit 18 represents whether or not a capture was an en passant capture
/// 0000 0000 0000 1000 0000 0000 0000 0000 PAWN_START:     0x80000    bit 19 represents whether the move was a starting pawn move that moved two spaces
/// 0000 0000 1111 0000 0000 0000 0000 0000 PIECE_PROMOTED: >> 20, 0xF bits 20-23 represent what piece a pawn was promoted to (if None then 0)
/// 0000 0001 0000 0000 0000 0000 0000 0000 CASTLE:         0x1000000  bit 24 represents whether or not a move was a castling move
/// 0001 1110 0000 0000 0000 0000 0000 0000 PIECE_MOVED:    >> 25 0x7F bits 25-28 represent which piece was initially moved
///
/// bits 29-31 will be unused
/// IMPORTANT: 000 0000 indicates Square 0 in theory (in practice we should avoid this with the type system) and not absence
///            0000 indicates absence for Pieces. 0001 indicated White Pawn
/// NOTE: The number of pieces can fit in 4 bits while the number of 120 squares can fit in 7 bits
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct Move {
    move_: u32,
    score: u16,
}

// TODO: make builder for Move if performance allows it
impl Move {
    /// IMPORTANT: Keep in mind that Piece 0 is a White Pawn but inside move 0
    /// symbolizes absence.
    pub fn new(
        start: Square,
        end: Square,
        piece_captured: Option<Piece>,
        en_passant: bool,
        pawn_start: bool,
        piece_promoted: Option<Piece>,
        castle: bool,
        piece_moved: Piece,
    ) -> Self {
        let move_ = (start as u32)
            | ((end as u32) << MOVE_END_SHIFT)
            | (piece_captured.map_or_else(
                || 0,
                |p| (p as u32) + 1, // add 1 to make room for 0 to indicate absence
            ) << MOVE_PIECE_CAPTURED_SHIFT)
            | ((en_passant as u32) * MOVE_EN_PASSANT_MASK)
            | ((pawn_start as u32) * MOVE_PAWN_START_MASK)
            | (piece_promoted.map_or_else(|| 0, |p| (p as u32) + 1) << MOVE_PIECE_PROMOTED_SHIFT)
            | ((castle as u32) * MOVE_CASTLE_MASK)
            | (((piece_moved as u32) + 1) << MOVE_PIECE_MOVED_SHIFT);

        Move { move_, score: 0 }
    }

    // TODO: revisit when performance tuning. Some of these checks are probably not be needed
    /// Additional checks that can be made when making moves to make sure that
    /// the move is consistent
    pub fn check_move(&self, active_color: Color) -> Result<(), MoveValidityError> {
        let start_square = self.get_start()?;
        let start_rank = start_square.get_rank();

        let en_passant = self.is_en_passant();
        let pawn_start = self.is_pawn_start();
        let piece_promoted = self.get_piece_promoted()?;
        let captured = self.get_piece_captured()?;
        let castle = self.is_castle();
        let piece_moved = self.get_piece_moved()?;

        // Check that active_color matches the piece moved
        if piece_moved.get_color() != active_color {
            return Err(MoveValidityError::PieceMovedActiveColorMismatch {
                piece_moved,
                active_color,
            });
        }

        // Check that if piece is captured, it's of the opposite color
        match captured {
            Some(captured_piece) => match active_color {
                Color::White => {
                    if let Color::White = captured_piece.get_color() {
                        return Err(MoveValidityError::CaptureActiveColor { captured_piece });
                    }
                }
                Color::Black => {
                    if let Color::Black = captured_piece.get_color() {
                        return Err(MoveValidityError::CaptureActiveColor { captured_piece });
                    }
                }
            },
            // if no piece was captured, then en_passant can't be true
            None => {
                if en_passant {
                    return Err(MoveValidityError::EnPassantNoCapture);
                }
            }
        }

        if pawn_start {
            // pawn_starts can't be captures, castling, en passant, nor promotions
            if (captured.is_some() || castle || en_passant || piece_promoted.is_some()) {
                return Err(MoveValidityError::PawnStartExclusive);
            }

            // check that pawn_start is consistent with rank
            let expected_rank = match active_color {
                Color::White => Rank::Rank3,
                Color::Black => Rank::Rank6,
            };

            if start_rank != expected_rank {
                return Err(MoveValidityError::RankPawnStartMismatch {
                    active_color,
                    start_square,
                    start_rank,
                });
            }
        }

        if castle {
            // a castling move cannot be a promotion nor an en passant
            if (en_passant || piece_promoted.is_some()) {
                return Err(MoveValidityError::CastleExclusive);
            }
            // Check that castle move initiates from the appropriate king
            match piece_moved {
                Piece::WhiteKing => {
                    if active_color == Color::Black {
                        return Err(MoveValidityError::WrongKingCastled);
                    }
                }
                Piece::BlackKing => {
                    if active_color == Color::White {
                        return Err(MoveValidityError::WrongKingCastled);
                    }
                }
                _ => return Err(MoveValidityError::NonKingInitiatedCastle),
            }
        }

        // Can't promote to pawn
        if let Some(piece) = piece_promoted {
            if piece.is_pawn() {
                return Err(MoveValidityError::PromotionToPawn);
            }
        }

        Ok(())
    }

    pub fn get_start(&self) -> Result<Square, MoveDeserializeError> {
        let start = self.move_ & MOVE_SQUARE_MASK;
        Square::try_from(start).map_err(|_err| MoveDeserializeError::Start {
            start,
            move_: self.move_,
        })
    }

    pub fn get_start_raw(&self) -> u32 {
        self.move_ & MOVE_SQUARE_MASK
    }

    pub fn get_end(&self) -> Result<Square, MoveDeserializeError> {
        let end = (self.move_ >> MOVE_END_SHIFT) & MOVE_SQUARE_MASK;
        Square::try_from(end).map_err(|_err| MoveDeserializeError::End {
            end,
            move_: self.move_,
        })
    }

    pub fn get_end_raw(&self) -> u32 {
        (self.move_ >> MOVE_END_SHIFT) & MOVE_SQUARE_MASK
    }

    pub fn get_piece_captured(&self) -> Result<Option<Piece>, MoveDeserializeError> {
        let piece = (self.move_ >> MOVE_PIECE_CAPTURED_SHIFT) & MOVE_PIECE_MASK;
        match piece {
            0 => Ok(None),
            _ => match Piece::try_from(piece - 1) {
                Ok(piece) => Ok(Some(piece)),
                Err(_) => Err(MoveDeserializeError::Captured {
                    piece,
                    move_: self.move_,
                }),
            },
        }
    }

    pub fn get_piece_captured_raw(&self) -> u32 {
        (self.move_ >> MOVE_PIECE_CAPTURED_SHIFT) & MOVE_PIECE_MASK
    }

    pub fn is_en_passant(&self) -> bool {
        self.move_ & MOVE_EN_PASSANT_MASK != 0
    }

    /// Tells us whether or not this was a pawn moved up two positions
    pub fn is_pawn_start(&self) -> bool {
        self.move_ & MOVE_PAWN_START_MASK != 0
    }

    pub fn get_piece_promoted(&self) -> Result<Option<Piece>, MoveDeserializeError> {
        let piece = (self.move_ >> MOVE_PIECE_PROMOTED_SHIFT) & MOVE_PIECE_MASK;
        match piece {
            0 => Ok(None),
            _ => match Piece::try_from(piece - 1) {
                Ok(piece) => Ok(Some(piece)),
                Err(_) => Err(MoveDeserializeError::Promoted {
                    piece,
                    move_: self.move_,
                }),
            },
        }
    }

    fn get_piece_promoted_raw(&self) -> u32 {
        (self.move_ >> MOVE_PIECE_PROMOTED_SHIFT) & MOVE_PIECE_MASK
    }

    pub fn is_castle(&self) -> bool {
        self.move_ & MOVE_CASTLE_MASK != 0
    }

    pub fn get_piece_moved(&self) -> Result<Piece, MoveDeserializeError> {
        let piece = (self.move_ >> MOVE_PIECE_MOVED_SHIFT) & MOVE_PIECE_MASK;
        match piece {
            0 => Err(MoveDeserializeError::Moved {
                piece,
                move_: self.move_,
            }),
            _ => match Piece::try_from(piece - 1) {
                Ok(piece) => Ok(piece),
                Err(_) => Err(MoveDeserializeError::Moved {
                    piece,
                    move_: self.move_,
                }),
            },
        }
    }

    fn get_piecemove_d_raw(&self) -> u32 {
        (self.move_ >> MOVE_PIECE_MOVED_SHIFT) & MOVE_PIECE_MASK
    }

    pub fn is_capture(&self) -> bool {
        (self.move_ & MOVE_IS_CAPTURE_MASK) != 0
    }

    pub fn is_promotion(&self) -> bool {
        (self.move_ & MOVE_IS_PROMOTED_MASK) != 0
    }

    pub fn get_score(&self) -> u16 {
        self.score
    }

    // pub fn from_uci(uci: &str) -> Self {
    //     todo!()
    // }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let piece_captured = self.get_piece_captured().expect(
            "piece_captured can be None but calling self.get_piece_captured() should not fail
        since we should always be able to parse bits into a valid piece",
        );

        let piece_promoted = self.get_piece_promoted().expect(
            "piece_promoted can be None but calling self.get_piece_promoted() should not fail
        since we should always be able to parse bits into a valid piece",
        );

        let piecemove_d = self
            .get_piece_moved()
            .expect("piecemove_d should always be valid");

        let start = self.get_start().expect("start should always be valid");

        let end = self.get_end().expect("end should always be valid");

        writeln!(f, "Dec: {}", self.move_);
        writeln!(f, "Bin: {:032b}", self.move_);

        writeln!(f, "Start Square: {} {:07b}", start, self.get_start_raw());
        writeln!(f, "End Square: {} {:07b}", end, self.get_end_raw());

        match piece_captured {
            Some(piece) => {
                writeln!(
                    f,
                    "Piece Captured: {} {:04b}",
                    piece,
                    self.get_piece_captured_raw()
                );
            }
            None => {
                writeln!(
                    f,
                    "Piece Captured: None {:04b}",
                    self.get_piece_captured_raw()
                );
            }
        }

        writeln!(f, "En Passant Capture: {}", self.is_en_passant());
        writeln!(f, "Pawn Start: {}", self.is_pawn_start());

        match piece_promoted {
            Some(piece) => {
                writeln!(
                    f,
                    "Piece Promoted: {} {:04b}",
                    piece,
                    self.get_piece_promoted_raw()
                );
            }
            None => {
                writeln!(
                    f,
                    "Piece Promoted: None {:04b}",
                    self.get_piece_promoted_raw()
                );
            }
        }

        writeln!(f, "Castling Move: {}", self.is_castle());

        writeln!(
            f,
            "Piece Moved: {} {:04b}",
            piecemove_d,
            self.get_piecemove_d_raw()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::gamestate::GamestateBuilder;

    use super::*;

    //================================ DISPLAY ================================
    // TODO: these display tests rely heavily on Gamestate functionality
    // should right some decoupled tests
    #[test]
    fn test_move_list_display_visual() {
        let fen = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/2P1P3/RNBQKBNR w KQkq e6 0 1";
        let gamestate = GamestateBuilder::new_with_fen(fen)
            .unwrap()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();

        println!("{}", gamestate.gen_move_list().unwrap());
    }

    #[test]
    fn test_move_display_visual() {
        println!("Game Starting State:");

        let fen_0 = "rnbqkbnr/ppp2ppp/3p4/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3";
        let mut gamestate = GamestateBuilder::new_with_fen(fen_0)
            .unwrap()
            .build()
            .unwrap();
        println!("{}", gamestate);

        // D5 0x40 E6 0x4B
        println!("D5E6 White Pawn captures Black Pawn via En Passant");
        #[allow(clippy::unusual_byte_groupings)]
        let move_1 = Move {
            //        unused  pm   cstl prom ps ep capt end     start
            move_: 0b_0000000_0001_0____0000_0__1__0111_1001011_1000000,
            score: 0,
        };
        println!("{}", move_1);

        let fen_1 = "rnbqkbnr/ppp2ppp/3pP3/8/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 3";
        gamestate = GamestateBuilder::new_with_fen(fen_1)
            .unwrap()
            .build()
            .unwrap();
        println!("{}", gamestate);

        // C8 0x5D E6 0x4B
        println!("C8E6 Black Bishop captures White Pawn");
        #[allow(clippy::unusual_byte_groupings)]
        let move_2 = Move {
            //        unused  pm   cstl prom ps ep capt end     start
            move_: 0b_0000000_1001_0____0000_0__0__0001_1001011_1011101,
            score: 0,
        };
        println!("{}", move_2);

        let fen_2 = "rn1qkbnr/ppp2ppp/3pb3/8/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 4";
        gamestate = GamestateBuilder::new_with_fen(fen_2)
            .unwrap()
            .build()
            .unwrap();
        println!("{}", gamestate);
    }

    #[test]
    #[should_panic]
    fn test_move_display_invalid_piece_captured_panic() {
        #[allow(clippy::unusual_byte_groupings)]
        let invalidmove_ = Move {
            //        unused  pm   cstl prom ps ep capt end     start
            move_: 0b_0000000_0001_0____0000_0__1__1111_1001011_1000000,
            score: 0,
        };
        println!("{}", invalidmove_);
    }

    #[test]
    #[should_panic]
    fn test_move_display_invalid_piece_promoted_panic() {
        #[allow(clippy::unusual_byte_groupings)]
        let invalidmove_ = Move {
            //        unused  pm   cstl prom ps ep capt end     start
            move_: 0b_0000000_0001_0____1111_0__1__0001_1001011_1000000,
            score: 0,
        };
        println!("{}", invalidmove_);
    }

    #[test]
    #[should_panic]
    fn test_move_display_invalid_piecemove_d_panic() {
        #[allow(clippy::unusual_byte_groupings)]
        let invalidmove_ = Move {
            //        unused  pm   cstl prom ps ep capt end     start
            move_: 0b_0000000_1111_0____0000_0__1__0111_1001011_1000000,
            score: 0,
        };
        println!("{}", invalidmove_);
    }

    #[test]
    #[should_panic]
    fn test_move_display_invalid_start_panic() {
        #[allow(clippy::unusual_byte_groupings)]
        let invalidmove_ = Move {
            //        unused  pm   cstl prom ps ep capt end     start
            move_: 0b_0000000_0001_0____0000_0__1__0111_1001011_1111111,
            score: 0,
        };
        println!("{}", invalidmove_);
    }

    #[test]
    #[should_panic]
    fn test_move_display_invalid_end_panic() {
        #[allow(clippy::unusual_byte_groupings)]
        let invalidmove_ = Move {
            //        unused  pm   cstl prom ps ep capt end     start
            move_: 0b_0000000_0001_0____0000_0__1__0111_1111111_1000000,
            score: 0,
        };
        println!("{}", invalidmove_);
    }

    //================================= BUILD =================================
    #[test]
    fn test_move_build_en_passant_capture() {
        // rnbqkbnr/ppp2ppp/3p4/3Pp3/8/8/PPP1PPPP/RNBQKBNR  WP captures bp via ep E6
        let start = Square::D5;
        let end = Square::E6;

        let piece_captured = Some(Piece::BlackPawn);

        let en_passant = true;
        let pawn_start = false;

        let piece_promoted = None;
        let castle = false;

        let piecemove_d = Piece::WhitePawn;

        let output = Move::new(
            start,
            end,
            piece_captured,
            en_passant,
            pawn_start,
            piece_promoted,
            castle,
            piecemove_d,
        );

        // start is 0x40
        // end is 0x4B
        // captured black pawn is 6 so stored as 7 to make room for 0 to be absence
        // piece promoted None so 0
        let expected = Move {
            #[allow(clippy::unusual_byte_groupings)]
            //        unused  pm   cstl prom ps ep capt end     start
            move_: 0b_0000000_0001_0____0000_0__1__0111_1001011_1000000,
            score: 0
        };

        assert_eq!(output, expected)
    }

    // #[test]
    // fn test_from_uci() {
    //     let ref_string = "e2e4";
    //     let newmove_: Move = Move::from_uci(ref_string);
    //     let output_string = newmove_.to_string();
    //     assert_eq!(ref_string, output_string);
    // }
}
