use nanoserde::{DeBin, SerBin};
use std::ops::Not;

#[derive(Clone, Debug, DeBin, SerBin)]
pub enum PlayerMessage {
    GameVsComputer,
    GameVsHuman,
    MovePiece(Move),
    Resign,
}

#[derive(Clone, Debug, DeBin, SerBin)]
pub enum ServerResponse {
    GameStarted(PlayerColor),
    GameWon(PlayerColor),
    GameDraw,
    MoveMade { player: PlayerColor, move_: Move },
}

#[derive(Clone, Copy, Debug, DeBin, SerBin, PartialEq, Eq)]
pub enum PlayerColor {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, DeBin, SerBin)]
pub struct Move {
    pub from: Square,
    pub to: Square,
}

#[derive(Clone, Copy, Debug, DeBin, SerBin)]
pub struct Square {
    pub rank: u32,
    pub file: u32,
}

impl Not for PlayerColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

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
    type Error = ();

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
            _ => Err(()),
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
