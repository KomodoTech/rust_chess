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
