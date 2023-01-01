use crate::moves::Move;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChessError {
    #[error("illegal move attempted: {0}")]
    IllegalMoveError(Move),
}
