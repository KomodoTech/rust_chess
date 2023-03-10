use std::fmt;

use crate::{
    error::MoveDeserializeError,
    piece::Piece,
    square::{Square, Square64},
};

//====================== CONSTANTS ============================================
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

/// Information that defines a move is stored in a 32-bit word with the following format
///
/// 0000 0000 0000 0000 0000 0000 0111 1111 START:          0x7F       bits 0-6 represent the Square120 where the move was initiated
/// 0000 0000 0000 0000 0011 1111 1000 0000 END:            >> 7, 0x7F bits 7-13 represent the Square120 where the move ended
/// 0000 0000 0000 0011 1100 0000 0000 0000 PIECE_CAPTURED: >> 14, 0xF bits 14-17 represent the Piece that was captured (in None then 0)
/// 0000 0000 0000 0100 0000 0000 0000 0000 EN_PASSANT:     0x40000    bit 18 represents whether or not a capture was an en passant capture
/// 0000 0000 0000 1000 0000 0000 0000 0000 PAWN_START:     0x80000    bit 19 represents whether the move was a starting pawn move (which can move two spaces)
/// 0000 0000 1111 0000 0000 0000 0000 0000 PIECE_PROMOTED: >> 20, 0xF bits 20-23 represent what piece a pawn was promoted to (if None then 0)
/// 0000 0001 0000 0000 0000 0000 0000 0000 CASTLE:         0x1000000  bit 24 represents whether or not a move was a castling move
///
/// bits 25-31 will be unused
/// IMPORTANT: 000 0000 indicates Square 0 in theory (in practice we should avoid this with the type system) and not absence
///            0000 indicates absence for Pieces. 0001 indicated White Pawn
/// NOTE: The number of pieces can fit in 4 bits while the number of 120 squares can fit in 7 bits
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Move(u32);

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
    ) -> Self {
        // let mut _move = start as u32;
        // _move |= (end as u32) << MOVE_END_SHIFT;
        // _move |= piece_captured.map_or_else(|| 0, |p| (p as u32) + 1) << MOVE_PIECE_CAPTURED_SHIFT;
        // _move |= (en_passant as u32) * MOVE_EN_PASSANT_MASK;
        // _move |= (pawn_start as u32) * MOVE_PAWN_START_MASK;
        // _move |= piece_promoted.map_or_else(|| 0, |p| (p as u32) + 1) << MOVE_PIECE_PROMOTED_SHIFT;
        // _move |= (castle as u32) * MOVE_CASTLE_MASK;

        Move( (start as u32)
                | ((end as u32) << MOVE_END_SHIFT)

                | (piece_captured.map_or_else(
                    || 0,
                    |p| (p as u32) + 1, // add 1 to make room for 0 to indicate absence
                ) << MOVE_PIECE_CAPTURED_SHIFT)

                | ((en_passant as u32) * MOVE_EN_PASSANT_MASK)
                | ((pawn_start as u32) * MOVE_PAWN_START_MASK)

                | (piece_promoted.map_or_else(|| 0, |p| (p as u32) + 1)
                    << MOVE_PIECE_PROMOTED_SHIFT)

                | ((castle as u32) * MOVE_CASTLE_MASK))
    }

    pub fn get_start(&self) -> Result<Square, MoveDeserializeError> {
        let start = self.0 & MOVE_SQUARE_MASK;
        Square::try_from(start).map_err(|_err| MoveDeserializeError::Start {
            start,
            _move: self.to_string(),
        })
    }

    pub fn get_end(&self) -> Result<Square, MoveDeserializeError> {
        let end = (self.0 >> MOVE_END_SHIFT) & MOVE_SQUARE_MASK;
        Square::try_from(end).map_err(|_err| MoveDeserializeError::End {
            end,
            _move: self.to_string(),
        })
    }

    pub fn get_piece_captured(&self) -> Result<Option<Piece>, MoveDeserializeError> {
        let piece = (self.0 >> MOVE_PIECE_CAPTURED_SHIFT) & MOVE_PIECE_MASK;
        match piece {
            0 => Ok(None),
            _ => match Piece::try_from(piece - 1) {
                Ok(piece) => Ok(Some(piece)),
                Err(_) => Err(MoveDeserializeError::Captured {
                    piece,
                    _move: self.to_string(),
                }),
            },
        }
    }

    pub fn get_en_passant(&self) -> bool {
        self.0 & MOVE_EN_PASSANT_MASK != 0
    }

    pub fn get_pawn_start(&self) -> bool {
        self.0 & MOVE_PAWN_START_MASK != 0
    }

    pub fn get_piece_promoted(&self) -> Result<Option<Piece>, MoveDeserializeError> {
        let piece = (self.0 >> MOVE_PIECE_PROMOTED_SHIFT) & MOVE_PIECE_MASK;
        match piece {
            0 => Ok(None),
            _ => match Piece::try_from(piece - 1) {
                Ok(piece) => Ok(Some(piece)),
                Err(_) => Err(MoveDeserializeError::Promoted {
                    piece,
                    _move: self.to_string(),
                }),
            },
        }
    }

    pub fn get_castle(&self) -> bool {
        self.0 & MOVE_CASTLE_MASK != 0
    }

    pub fn is_capture(&self) -> bool {
        (self.0 & MOVE_IS_CAPTURE_MASK) != 0
    }

    pub fn is_promotion(&self) -> bool {
        (self.0 & MOVE_IS_PROMOTED_MASK) != 0
    }

    // pub fn from_uci(uci: &str) -> Self {
    //     todo!()
    // }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let piece_captured = self.get_piece_captured();
        let piece_promoted = self.get_piece_promoted();

        match self.get_start() {
            Ok(square) => {
                writeln!(f, "Start Square: {}", square);
            }
            Err(_) => {
                writeln!(f, "Start Square: Invalid Square");
            }
        }

        match self.get_end() {
            Ok(square) => {
                writeln!(f, "End Square: {}", square);
            }
            Err(_) => {
                writeln!(f, "End Square: Invalid Square");
            }
        }

        match piece_captured {
            Ok(piece) => match piece {
                Some(piece) => {
                    writeln!(f, "Piece Captured: {}", piece);
                }
                None => {
                    writeln!(f, "Piece Captured: None");
                }
            },
            Err(_) => {
                writeln!(f, "Piece Captured: Invalid Piece");
            }
        }

        writeln!(f, "En Passant Capture: {}", self.get_en_passant());
        writeln!(f, "Pawn Start: {}", self.get_pawn_start());

        match piece_promoted {
            Ok(piece) => match piece {
                Some(piece) => {
                    writeln!(f, "Piece Promoted: {}", piece);
                }
                None => {
                    writeln!(f, "Piece Promoted: None");
                }
            },
            Err(_) => {
                writeln!(f, "Piece Promoted: Invalid Piece");
            }
        }

        writeln!(f, "Castling Move: {}", self.get_castle())
    }
}

#[cfg(test)]
mod tests {
    use crate::gamestate::GamestateBuilder;

    use super::*;

    //================================ DISPLAY ================================
    #[test]
    fn test_move_display_visual() {
        println!("Game Starting State:");

        let fen_0 = "rnbqkbnr/ppp2ppp/3p4/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3";
        let mut gamestate = GamestateBuilder::new_with_fen(fen_0).unwrap().build().unwrap();
        println!("{}", gamestate);

        // D5 0x40 E6 0x4B
        println!("D5E6 White Pawn captures Black Pawn via En Passant");
        #[allow(clippy::unusual_byte_groupings)]
        //                        unused      cstl prom ps ep capt end     start
        let move_1 = Move(0b_00000000000_0____0000_0__1__0111_1001011_1000000);
        println!("{}", move_1);

        let fen_1 = "rnbqkbnr/ppp2ppp/3pP3/8/8/8/PPP1PPPP/RNBQKBNR b KQkq - 0 3";
        gamestate = GamestateBuilder::new_with_fen(fen_1).unwrap().build().unwrap();
        println!("{}", gamestate);

        
        // C8 0x5D E6 0x4B
        println!("C8E6 Black Bishop captures White Pawn");
        #[allow(clippy::unusual_byte_groupings)]
        //                        unused      cstl prom ps ep capt end     start
        let move_2 = Move(0b_00000000000_0____0000_0__0__0001_1001011_1011101);
        println!("{}", move_2);

        let fen_2 = "rn1qkbnr/ppp2ppp/3pb3/8/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 4";
        gamestate = GamestateBuilder::new_with_fen(fen_2).unwrap().build().unwrap();
        println!("{}", gamestate);


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

        let output = Move::new(
            start,
            end,
            piece_captured,
            en_passant,
            pawn_start,
            piece_promoted,
            castle,
        );

        // start is 0x40
        // end is 0x4B
        // captured black pawn is 6 so stored as 7 to make room for 0 to be absence
        // piece promoted None so 0
        let expected = Move(
            #[allow(clippy::unusual_byte_groupings)]
            // unused      cstl prom ps ep capt end     start
            0b_00000000000_0____0000_0__1__0111_1001011_1000000,
        );

        assert_eq!(output, expected)
    }

    // #[test]
    // fn test_from_uci() {
    //     let ref_string = "e2e4";
    //     let new_move: Move = Move::from_uci(ref_string);
    //     let output_string = new_move.to_string();
    //     assert_eq!(ref_string, output_string);
    // }
}
