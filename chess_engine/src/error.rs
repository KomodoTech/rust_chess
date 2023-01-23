use crate::moves::Move;
use crate::squares::{Square, Square64};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ChessError {
    #[error("illegal move attempted: {0}")]
    IllegalMoveError(Move),
    #[error("Could not convert char {0} into a Piece")]
    ParsePieceError(char),
    #[error("Could not convert &str {0} into a Square")]
    ParseSquareFromStrError(String),
    #[error("Could not convert &str {0} into a Square64")]
    ParseSquare64FromStrError(String),
    #[error("Could not convert u8 {0} into a Square")]
    ParseSquareFromU8Error(u8),
    #[error("Could not convert u8 {0} into a Square64")]
    ParseSquare64FromU8Error(u8),
    #[error("Square {0} is on invalid File")]
    SquareOnInvalidFile(Square),
    #[error("Square {0} is on invalid Rank")]
    SquareOnInvalidRank(Square),
    #[error("Square64 {0} is on invalid File")]
    Square64OnInvalidFile(Square64),
    #[error("Square64 {0} is on invalid Rank")]
    Square64OnInvalidRank(Square64),
}
