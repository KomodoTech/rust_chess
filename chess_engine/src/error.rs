use crate::{
    board::bitboard::BitBoard,
    moves::Move,
    squares::{Square, Square64},
    util::NUM_FEN_SECTIONS,
};
use strum::ParseError as StrumParseError;
use thiserror::Error;

// TODO: Clean up names and explore thiserror #from to see if you can convert
// from one type of error to another
// TODO: determine whether using &str instead of Strings is worth it
#[derive(Error, Debug, PartialEq)]
pub enum ChessError {
    #[error("illegal move attempted: {0}")]
    IllegalMoveError(Move),
}

#[derive(Error, Debug, PartialEq)]
pub enum FENParseError {
    #[error(
        "number of subsections of FEN &str is {0}, but should be {}",
        NUM_FEN_SECTIONS
    )]
    WrongNumFENSections(usize),

    #[error("active color section of substring {0} is an invalid Color")]
    ActiveColorInvalid(String),

    #[error("base FEN {0} did not have 8 rows separated by / delimiter")]
    GenerateBoardFromBaseFENNumberOfRowsError(String),

    #[error("could not parse invalid char {0} while parsing base FEN")]
    GenerateBoardFromBaseFENInvalidCharError(char),

    #[error("could not parse invalid digit {0} while parsing base FEN. digit needs to be smaller or equal to 8")]
    GenerateBoardFromBaseFENInvalidDigitError(u32),

    #[error("number of ranks {0} parsed from fen exceeds 8")]
    GenerateBoardFromBaseFENTooManyRanksError(usize),

    #[error("FEN rank {0} does not represent 8 squares")]
    GenerateBoardFromBaseFENRankDoesNotContain8SquaresError(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum BitBoardError {
    #[error("cannot check bit at index {0}, which is greater than 63")]
    BitBoardCheckBitInvalidIndexError(u8),

    #[error("cannot set bit at index {0}, which is greater than 63")]
    BitBoardSetBitInvalidIndexError(u8),

    #[error("cannot unset bit at index {0}, which is greater than 63")]
    BitBoardUnsetBitInvalidIndexError(u8),
}

#[derive(Error, Debug, PartialEq)]
pub enum ConversionError {
    #[error("could not convert char {0} into a Piece")]
    ParsePieceFromCharError(char),

    #[error("could not convert u8 {0} into a CastlePerm because {0} is greater than 0x0F")]
    ParseCastlePermFromU8ErrorValueTooLargeError(u8),

    #[error("could not convert {0} into a CastlePerm because char {0} is invalid")]
    ParseCastlePermFromStrInvalidCharError(String, char),

    // Won't catch - duplicates
    #[error("could not convert {0} into a CastlePerm encountered duplicates")]
    ParseCastlePermFromStrDuplicatesError(String),

    #[error("could not convert {0} into a CastlePerm")]
    ParseCastlePermFromStrError(String),

    #[error("could not convert &str {0} into a Square")]
    ParseSquareFromStrError(#[from] StrumParseError),

    #[error("could not convert u8 {0} into a Square")]
    ParseSquareFromU8Error(u8),

    #[error("could not convert u8 {0} into a Square64")]
    ParseSquare64FromU8Error(u8),

    #[error("could not convert u32 {0} into a Square")]
    ParseSquareFromU32Error(u32),

    #[error("could not convert u32 {0} into a Square64")]
    ParseSquare64FromU32Error(u32),

    #[error("could not convert usize {0} into a Square")]
    ParseSquareFromUsizeError(usize),

    #[error("could not convert usize {0} into a Square64")]
    ParseSquare64FromUsizeError(usize),

    #[error("could not convert usize {0} into a Rank")]
    ParseRankFromUsizeError(usize),
}
