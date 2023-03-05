use crate::{color::Color, error::PieceConversionError};

use std::fmt::{self, write};
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

// CONSTANTS:
const PIECE_BIG: [bool; Piece::COUNT] = [
    //wp     wn    wb    wr    wq    wk    bp     bn    bb    br    bq    bk
    false, true, true, true, true, true, false, true, true, true, true, true,
];
// NOTE: in most chess vocabulary King is not a major piece, but here it is considered one
const PIECE_MAJOR: [bool; Piece::COUNT] = [
    // wp  wn     wb     wr    wq    wk    bp     bn     bb     br    bq    bk
    false, false, false, true, true, true, false, false, false, true, true, true,
];
const PIECE_MINOR: [bool; Piece::COUNT] = [
    // wp  wn    wb    wr     wq     wk     bp     bn    bb    br     bq     bk
    false, true, true, false, false, false, false, true, true, false, false, false,
];
const PIECE_SLIDING: [bool; Piece::COUNT] = [
    // wp  wn     wb    wr    wq    wk     bp     bn     bb    br    bq    bk
    false, false, true, true, true, false, false, false, true, true, true, false,
];
const PIECE_VALUE: [u32; Piece::COUNT] = [
    //wp wn   wb   wr   wq     wk      bp   bn   bb   br   bq     bk
    100, 325, 325, 550, 1_000, 50_000, 100, 325, 325, 550, 1_000, 50_000,
];

/// Allows us to associate a color with a piece
const PIECE_COLOR: [Color; Piece::COUNT] = [
    Color::White, // wp
    Color::White, // wn
    Color::White, // wb
    Color::White, // wr
    Color::White, // wq
    Color::White, // wk
    Color::Black, // bp
    Color::Black, // bn
    Color::Black, // bb
    Color::Black, // br
    Color::Black, // bq
    Color::Black, // bk
];

/// Allows us to determine piece type
const PIECE_TYPE: [PieceType; Piece::COUNT] = [
    PieceType::Pawn,   // wp
    PieceType::Knight, // wn
    PieceType::Bishop, // wb
    PieceType::Rook,   // wr
    PieceType::Queen,  // wq
    PieceType::King,   // wk
    PieceType::Pawn,   // bp
    PieceType::Knight, // bn
    PieceType::Bishop, // bb
    PieceType::Rook,   // br
    PieceType::Queen,  // bq
    PieceType::King,   // bk
];

/// Allows us to know if a piece is a Pawn
const PIECE_PAWN: [bool; Piece::COUNT] = [
    // wp wn     wb     wr     wq     wk     bp    bn     bb     br     bq     bk
    true, false, false, false, false, false, true, false, false, false, false, false,
];

/// Allows us to know if a piece is a Knight
const PIECE_KNIGHT: [bool; Piece::COUNT] = [
    // wp  wn    wb     wr     wq     wk     bp     bn    bb     br     bq     bk
    false, true, false, false, false, false, false, true, false, false, false, false,
];

/// Allows us to know if a piece is a Bishop
const PIECE_BISHOP: [bool; Piece::COUNT] = [
    // wp  wn     wb    wr     wq     wk     bp     bn     bb    br     bq     bk
    false, false, true, false, false, false, false, false, true, false, false, false,
];

/// Allows us to know if a piece is a Rook
const PIECE_ROOK: [bool; Piece::COUNT] = [
    // wp  wn     wb     wr    wq     wk     bp     bn     bb     br    bq     bk
    false, false, false, true, false, false, false, false, false, true, false, false,
];

/// Allows us to know if a piece is a Queen
const PIECE_QUEEN: [bool; Piece::COUNT] = [
    // wp  wn     wb     wr     wq    wk     bp     bn     bb     br     bq    bk
    false, false, false, false, true, false, false, false, false, false, true, false,
];

/// Allows us to know if a piece is a King
const PIECE_KING: [bool; Piece::COUNT] = [
    // wp  wn     wb     wr     wq     wk    bp     bn     bb     br     bq     bk
    false, false, false, false, false, true, false, false, false, false, false, true,
];

/// For regular chess these are the max number of pieces that you could imagineably get per type
const MAX_NUM_PIECES_ALLOWED: [u8; Piece::COUNT] = [
    //wp wn wb wr  wq wk bp bn  bb  br  bq bk
    8, 10, 10, 10, 9, 1, 8, 10, 10, 10, 9, 1,
];

/// For regular chess these are the starting number of pieces per type
const STARTING_NUM_PIECES: [u8; Piece::COUNT] = [
    //wp wn wb wr wq wk bp bn bb br bq bk
    8, 2, 2, 2, 1, 1, 8, 2, 2, 2, 1, 1,
];

// ATTACKING

/// Given a starting Square (10x12) index, these values are all the offsets where a White Pawn could move
/// NOTE: for pawns when checking whether or not a square is being attacked, you have to SUBTRACT these values
const WHITE_PAWN_ATTACK_DIRECTIONS: [i8; 2] = [
    -9,  // Up Right
    -11, // Up Left
];

/// Given a starting Square (10x12) index, these values are all the offsets where a Black Pawn could move
/// NOTE: for pawns when checking whether or not a square is being attacked, you have to SUBTRACT these values
const BLACK_PAWN_ATTACK_DIRECTIONS: [i8; 2] = [
    9,  // Down Left
    11, // Down Right
];

/// Given a starting Square (10x12) index, these values are all the offsets where a Knight could move
const KNIGHT_DIRECTIONS: [i8; 8] = [
    -8,  // 1 Down 2 Right
    -19, // 2 Down 1 Right
    -21, // 2 Down 1 Left
    -12, // 1 Down 2 Left
    8,   // 1 Up   2 Left
    19,  // 2 Up   1 Left
    21,  // 2 Up   1 Right
    12,  // 1 Up   2 Right
];

// NOTE: Queen is a combination of Bishop and Rook

/// Given a starting Square (10x12) index, these values are the offsets that correspond to a legal
/// direction where a Bishop can move (mutliply by a constant to move than one space at a time)
const BISHOP_DIRECTIONS: [i8; 4] = [
    -9,  // Up Right Direction
    -11, // Up Left Direction
    9,   // Down Right Direction
    11,  // Down Left Direction
];

/// Given a starting Square (10x12) index, these values are the offsets that correspond to a legal
/// direction where a Rook can move (mutliply by a constant to move than one space at a time)
const ROOK_DIRECTIONS: [i8; 4] = [
    -1,  // Left Direction
    -10, // Up Direction
    1,   // Right Direction
    10,  // Down Direction
];

/// Given a starting Square (10x12) index, these values are all the offsets where a King could move
const KING_DIRECTIONS: [i8; 8] = [
    -1,  // Right
    -9,  // Up Right
    -10, // Up
    -11, // Up Left
    1,   // Left
    9,   // Down Left
    10,  // Down
    11,  // Down Right
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumCountMacro)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

impl Piece {
    pub fn is_big(&self) -> bool {
        PIECE_BIG[*self as usize]
    }
    pub fn is_major(&self) -> bool {
        PIECE_MAJOR[*self as usize]
    }
    pub fn is_minor(&self) -> bool {
        PIECE_MINOR[*self as usize]
    }
    pub fn is_sliding(&self) -> bool {
        PIECE_SLIDING[*self as usize]
    }
    pub fn get_value(&self) -> u32 {
        PIECE_VALUE[*self as usize]
    }
    pub fn get_color(&self) -> Color {
        PIECE_COLOR[*self as usize]
    }
    pub fn get_max_num_allowed(&self) -> u8 {
        MAX_NUM_PIECES_ALLOWED[*self as usize]
    }
    pub fn get_starting_num(&self) -> u8 {
        STARTING_NUM_PIECES[*self as usize]
    }

    pub fn is_pawn(&self) -> bool {
        PIECE_PAWN[*self as usize]
    }
    pub fn is_knight(&self) -> bool {
        PIECE_KNIGHT[*self as usize]
    }
    pub fn is_bishop(&self) -> bool {
        PIECE_BISHOP[*self as usize]
    }
    pub fn is_rook(&self) -> bool {
        PIECE_ROOK[*self as usize]
    }
    pub fn is_queen(&self) -> bool {
        PIECE_QUEEN[*self as usize]
    }
    pub fn is_king(&self) -> bool {
        PIECE_KING[*self as usize]
    }
    pub fn get_piece_type(&self) -> PieceType {
        PIECE_TYPE[*self as usize]
    }

    // TODO: Test performance
    pub fn get_attack_directions(&self) -> Vec<i8> {
        let mut attack_directions: Vec<i8> = vec![];
        match self.get_piece_type() {
            PieceType::Pawn => match self.get_color() {
                Color::White => attack_directions.extend_from_slice(&WHITE_PAWN_ATTACK_DIRECTIONS),
                Color::Black => attack_directions.extend_from_slice(&BLACK_PAWN_ATTACK_DIRECTIONS),
            },
            PieceType::Knight => attack_directions.extend_from_slice(&KNIGHT_DIRECTIONS),
            PieceType::Bishop => attack_directions.extend_from_slice(&BISHOP_DIRECTIONS),
            PieceType::Rook => attack_directions.extend_from_slice(&ROOK_DIRECTIONS),
            PieceType::Queen => {
                attack_directions.extend_from_slice(&BISHOP_DIRECTIONS);
                attack_directions.extend_from_slice(&ROOK_DIRECTIONS)
            }
            PieceType::King => attack_directions.extend_from_slice(&KING_DIRECTIONS),
        }
        attack_directions
    }
}

impl TryFrom<char> for Piece {
    type Error = PieceConversionError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'P' => Ok(Piece::WhitePawn),
            'N' => Ok(Piece::WhiteKnight),
            'B' => Ok(Piece::WhiteBishop),
            'R' => Ok(Piece::WhiteRook),
            'Q' => Ok(Piece::WhiteQueen),
            'K' => Ok(Piece::WhiteKing),
            'p' => Ok(Piece::BlackPawn),
            'n' => Ok(Piece::BlackKnight),
            'b' => Ok(Piece::BlackBishop),
            'r' => Ok(Piece::BlackRook),
            'q' => Ok(Piece::BlackQueen),
            'k' => Ok(Piece::BlackKing),
            _ => Err(PieceConversionError::FromChar {
                invalid_char: value,
            }),
        }
    }
}

impl From<Piece> for char {
    fn from(value: Piece) -> Self {
        match value {
            Piece::WhitePawn => 'P',
            Piece::WhiteKnight => 'N',
            Piece::WhiteBishop => 'B',
            Piece::WhiteRook => 'R',
            Piece::WhiteQueen => 'Q',
            Piece::WhiteKing => 'K',
            Piece::BlackPawn => 'p',
            Piece::BlackKnight => 'n',
            Piece::BlackBishop => 'b',
            Piece::BlackRook => 'r',
            Piece::BlackQueen => 'q',
            Piece::BlackKing => 'k',
        }
    }
}

impl TryFrom<usize> for Piece {
    type Error = PieceConversionError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Piece::WhitePawn),
            1 => Ok(Piece::WhiteKnight),
            2 => Ok(Piece::WhiteBishop),
            3 => Ok(Piece::WhiteRook),
            4 => Ok(Piece::WhiteQueen),
            5 => Ok(Piece::WhiteKing),
            6 => Ok(Piece::BlackPawn),
            7 => Ok(Piece::BlackKnight),
            8 => Ok(Piece::BlackBishop),
            9 => Ok(Piece::BlackRook),
            10 => Ok(Piece::BlackQueen),
            11 => Ok(Piece::BlackKing),
            _ => Err(PieceConversionError::FromUsize {
                invalid_usize: value,
            }),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::WhitePawn => write!(f, "♙"),
            Piece::WhiteRook => write!(f, "♖"),
            Piece::WhiteBishop => write!(f, "♗"),
            Piece::WhiteKnight => write!(f, "♘"),
            Piece::WhiteQueen => write!(f, "♕"),
            Piece::WhiteKing => write!(f, "♔"),
            Piece::BlackPawn => write!(f, "♟"),
            Piece::BlackRook => write!(f, "♜"),
            Piece::BlackBishop => write!(f, "♝"),
            Piece::BlackKnight => write!(f, "♞"),
            Piece::BlackQueen => write!(f, "♛"),
            Piece::BlackKing => write!(f, "♚"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_piece_is_big_true() {
        let input = Piece::WhiteBishop;
        let output = input.is_big();
        let expected = true;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_is_big_false() {
        let input = Piece::BlackPawn;
        let output = input.is_major();
        let expected = false;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_is_major_true() {
        let input = Piece::WhiteRook;
        let output = input.is_major();
        let expected = true;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_is_major_false() {
        let input = Piece::BlackBishop;
        let output = input.is_major();
        let expected = false;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_is_minor_true() {
        let input = Piece::BlackKnight;
        let output = input.is_minor();
        let expected = true;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_is_minor_false() {
        let input = Piece::WhitePawn;
        let output = input.is_minor();
        let expected = false;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_get_value() {
        let input = Piece::WhitePawn;
        let output = input.get_value();
        let expected = 100;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_get_color() {
        let input = Piece::WhitePawn;
        let output = input.get_color();
        let expected = Color::White;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_try_from_char_valid_input() {
        let input = 'P';
        let output = Piece::try_from(input);
        let expected = Ok(Piece::WhitePawn);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_try_from_char_invalid_input() {
        let input = 'M';
        let output = Piece::try_from(input);
        let expected = Err(PieceConversionError::FromChar {
            invalid_char: input,
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_char_from_char() {
        let input = Piece::BlackBishop;
        let output = char::from(input);
        let expected = 'b';
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_display() {
        let input = Piece::BlackRook;
        let output = input.to_string();
        let expected = "♜".to_owned();
        assert_eq!(output, expected);
    }
}
