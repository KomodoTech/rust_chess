use crate::{
    squares::{Square, Square64},
    board::bitboard::BitBoard,
    moves::Move,
};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ChessError {
    #[error("illegal move attempted: {0}")]
    IllegalMoveError(Move),

    #[error("Could not convert char {0} into a Piece")]
    ParsePieceError(char),

    #[error("Could not convert str {0} into a Piece")]
    ParseSquareFromStrError(String),

    #[error("Could not convert &str {0} into a Square64")]
    ParseSquare64FromStrError(String),

    #[error("Could not convert u8 {0} into a Square")]
    ParseSquareFromU8Error(u8),

    #[error("Could not convert u8 {0} into a Square64")]
    ParseSquare64FromU8Error(u8),

    #[error("Could not convert BitBoard {0} into a Square")]
    ParseSquareFromBitBoardError(BitBoard),

    #[error("Could not convert BitBoard {0} into a Square64")]
    ParseSquare64FromBitBoardError(BitBoard),

    #[error("Square {0} is on invalid File")]
    SquareOnInvalidFile(Square),

    #[error("Square {0} is on invalid Rank")]
    SquareOnInvalidRank(Square),

    #[error("Square64 {0} is on invalid File")]
    Square64OnInvalidFile(Square64),

    #[error("Square64 {0} is on invalid Rank")]
    Square64OnInvalidRank(Square64),
    
    #[error("Cannot check bit at index {0}, which is greater than 63")]
    BitBoardCheckBitInvalidIndex(u8),

    #[error("Cannot set bit at index {0}, which is greater than 63")]
    BitBoardSetBitInvalidIndex(u8),

    #[error("Cannot unset bit at index {0}, which is greater than 63")]
    BitBoardUnsetBitInvalidIndex(u8),

    #[error("Cannot unset bit at index {0} that was not set to begin with")]
    BitBoardUnsetNonSetBit(u8),
}
