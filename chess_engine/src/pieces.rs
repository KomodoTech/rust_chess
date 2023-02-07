use crate::{error::ConversionError as Error, util::Color};

use std::fmt::{self, write};
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

// CONSTANTS:
const PIECE_BIG: [bool; Piece::COUNT] = [
    //  wp     wn    wb    wr    wq    wk    bp     bn    bb    br    bq    bk
    false, true, true, true, true, true, false, true, true, true, true, true,
];
const PIECE_MAJOR: [bool; Piece::COUNT] = [
    // wp  wn     wb     wr    wq    wk    bp     bn     bb     br    bq    bk
    false, false, false, true, true, true, false, false, false, true, true, true,
];
const PIECE_MINOR: [bool; Piece::COUNT] = [
    // wp  wn    wb    wr     wq     wk     bp     bn    bb    br     bq     bk
    false, true, true, false, false, false, false, true, true, false, false, false,
];
const PIECE_VALUE: [u32; Piece::COUNT] = [
    //wp wn   wb   wr   wq     wk      bp   bn   bb   br   bq     bk
    100, 325, 325, 550, 1_000, 50_000, 100, 325, 325, 550, 1_000, 50_000,
];
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
    fn is_big(&self) -> bool {
        PIECE_BIG[*self as usize]
    }
    fn is_major(&self) -> bool {
        PIECE_MAJOR[*self as usize]
    }
    fn is_minor(&self) -> bool {
        PIECE_MINOR[*self as usize]
    }
    fn get_value(&self) -> u32 {
        PIECE_VALUE[*self as usize]
    }
    fn get_color(&self) -> Color {
        PIECE_COLOR[*self as usize]
    }
}

impl TryFrom<char> for Piece {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'P' => Ok(Piece::WhitePawn),
            'R' => Ok(Piece::WhiteRook),
            'B' => Ok(Piece::WhiteBishop),
            'N' => Ok(Piece::WhiteKnight),
            'Q' => Ok(Piece::WhiteQueen),
            'K' => Ok(Piece::WhiteKing),
            'p' => Ok(Piece::BlackPawn),
            'r' => Ok(Piece::BlackRook),
            'b' => Ok(Piece::BlackBishop),
            'n' => Ok(Piece::BlackKnight),
            'q' => Ok(Piece::BlackQueen),
            'k' => Ok(Piece::BlackKing),
            _ => Err(Error::ParsePieceFromCharError(value)),
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
        let expected = Err(Error::ParsePieceFromCharError(input));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_piece_display() {
        let input = Piece::BlackRook;
        let output = input.to_string();
        let expected = "♜".to_string();
        assert_eq!(output, expected);
    }
}
