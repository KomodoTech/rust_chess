use crate::moves::Move;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChessError {
    #[error("illegal move attempted: {0}")]
    IllegalMoveError(Move),
    #[error("Could not convert char {0} into a piece")]
    ParsePieceError(char),
    #[error("Could not convert &str {0} into a square")]
    ParseSquareFromStrError(String),
    #[error("Could not convert u8 {0} into a square")]
    ParseSquareFromU8Error(u8),
}
