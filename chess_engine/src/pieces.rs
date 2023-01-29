use crate::error::ChessError as Error;
use std::fmt::{self, write};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
            _ => Err(Error::ParsePieceError(value)),
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
