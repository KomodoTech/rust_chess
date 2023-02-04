use crate::{
    board::bitboard::BitBoard,
    moves::Move,
    squares::{Square, Square64},
};
use strum::ParseError as StrumParseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ChessError {
    #[error("illegal move attempted: {0}")]
    IllegalMoveError(Move),

    #[error("Could not convert char {0} into a Piece")]
    ParsePieceError(char),

    #[error("Could not convert u8 {0} into a CastlePerm because {0} is greater than 0x0F")]
    ParseCastlePermFromU8ErrorValueTooLarge(u8),

    #[error("Could not convert u8 {0} into a CastlePerm due to a Castle having an invalid String representation")]
    ParseCastlePermFromU8ErrorInvalidCastleString(u8, String),

    #[error("Could not convert &str {0} into a Square")]
    ParseSquareFromStrError(#[from] StrumParseError),

    #[error("Could not convert u8 {0} into a Square")]
    ParseSquareFromU8Error(u8),

    #[error("Could not convert u8 {0} into a Square64")]
    ParseSquare64FromU8Error(u8),

    #[error("Could not convert u32 {0} into a Square")]
    ParseSquareFromU32Error(u32),

    #[error("Could not convert u32 {0} into a Square64")]
    ParseSquare64FromU32Error(u32),

    #[error("Cannot check bit at index {0}, which is greater than 63")]
    BitBoardCheckBitInvalidIndex(u8),

    #[error("Cannot set bit at index {0}, which is greater than 63")]
    BitBoardSetBitInvalidIndex(u8),

    #[error("Cannot unset bit at index {0}, which is greater than 63")]
    BitBoardUnsetBitInvalidIndex(u8),
}
