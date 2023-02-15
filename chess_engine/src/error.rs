use std::num::ParseIntError;

use crate::{
    board::bitboard::BitBoard,
    gamestate::{HALF_MOVE_MAX, MAX_GAME_MOVES, NUM_FEN_SECTIONS},
    moves::Move,
    squares::{Square, Square64},
    util::{File, Rank},
};
use strum::{EnumCount, ParseError as StrumParseError};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ChessError {
    #[error("illegal move attempted: {0}")]
    IllegalMove(Move),
}

#[derive(Error, Debug, PartialEq)]
pub enum RankFENParseError {
    #[error("Rank FEN is empty")]
    Empty,

    #[error("Rank FEN {0} should represent {} squares but does not", File::COUNT)]
    InvalidNumSquares(String),

    #[error(
        "Rank FEN {0} has character {1} which represents invalid digit (needs to be in 1..=8)"
    )]
    InvalidDigit(String, usize),

    #[error("Rank FEN {0} includes invalid char {1}")]
    InvalidChar(String, char),

    #[error("Rank FEN {0} includes two consecutive digits")]
    TwoConsecutiveDigits(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum BoardFENParseError {
    #[error("Encountered error while trying to parse a Rank FEN")]
    RankFENParseError(#[from] RankFENParseError),

    #[error(
        "board FEN {0} has {1} ranks separated by / delimiter instead of {}",
        Rank::COUNT
    )]
    WrongNumRanks(String, usize),

    #[error("FEN {0} includes too many {1}s")]
    InvalidNumOfPiece(String, char),

    #[error("FEN {0} must have exactly one white king and exactly one black king")]
    InvalidKingNum(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum GamestateFENParseError {
    #[error("board FEN is invalid")]
    BoardFENParseError(#[from] BoardFENParseError),

    #[error("Halfmove Clock is invalid")]
    HalfmoveClockFENParseError(#[from] HalfmoveClockFENParseError),

    #[error("Fullmove Counter is invalid")]
    FullmoveCounterFENParseError(#[from] FullmoveCounterFENParseError),

    #[error("FEN is invalid because it is empty")]
    Empty,

    #[error(
        "number of subsections of FEN &str is {0}, but should be {}",
        NUM_FEN_SECTIONS
    )]
    WrongNumFENSections(usize),

    #[error("active color section of substring {0} is an invalid Color")]
    ActiveColor(String),

    #[error("castle permissions {0} are invalid")]
    CastlePerm(String),

    #[error("en passant square {0} is invalid")]
    EnPassant(String),

    #[error("en passant square is invalid. It should be lowercase.")]
    EnPassantUppercase,
}

#[derive(Error, Debug, PartialEq)]
pub enum FullmoveCounterFENParseError {
    #[error("full moves is not a valid u32")]
    Parse(#[from] ParseIntError),

    #[error("full moves {0} must be in range 1..={}", MAX_GAME_MOVES)]
    NotInRange(u32),

    #[error("full moves {0} should be at least half the amount of half moves {1}")]
    SmallerThanHalfmoveClockDividedByTwo(u32, u32),
}

#[derive(Error, Debug, PartialEq)]
pub enum HalfmoveClockFENParseError {
    #[error("half moves is not a valid u32")]
    Parse(#[from] ParseIntError),

    #[error(
        "half moves {0} exceeds maximum number of half moves before a tie is called {}",
        HALF_MOVE_MAX
    )]
    ExceedsMax(u32),

    #[error("half moves must be 0 if there is en passant square")]
    NonZeroWhileEnPassant,
}

#[derive(Error, Debug, PartialEq)]
pub enum BitBoardError {
    #[error("cannot check bit at index {0}, which is greater than 63")]
    BitBoardCheckBitInvalidIndex(u8),

    #[error("cannot set bit at index {0}, which is greater than 63")]
    BitBoardSetBitInvalidIndex(u8),

    #[error("cannot unset bit at index {0}, which is greater than 63")]
    BitBoardUnsetBitInvalidIndex(u8),
}

#[derive(Error, Debug, PartialEq)]
pub enum PieceConversionError {
    #[error("could not convert char {0} into a Piece")]
    FromChar(char),
}

#[derive(Error, Debug, PartialEq)]
pub enum CastlePermConversionError {
    #[error("could not convert u8 {0} into a CastlePerm because {0} is greater than 0x0F")]
    FromU8ValueTooLarge(u8),

    #[error("could not convert {0} into a CastlePerm because char {0} is invalid")]
    FromStrInvalidChar(String, char),

    // Won't catch '-' duplicates
    #[error("could not convert {0} into a CastlePerm encountered duplicates")]
    FromStrDuplicates(String),

    #[error("could not convert {0} into a CastlePerm")]
    FromStr(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum SquareConversionError {
    #[error("could not convert &str {0} into a Square")]
    FromStr(#[from] StrumParseError),

    #[error("could not convert u8 {0} into a Square")]
    FromU8(u8),

    #[error("could not convert u32 {0} into a Square")]
    FromU32(u32),

    #[error("could not convert usize {0} into a Square")]
    FromUsize(usize),
}

#[derive(Error, Debug, PartialEq)]
pub enum Square64ConversionError {
    #[error("could not convert &str {0} into a Square64")]
    FromStr(#[from] StrumParseError),

    #[error("could not convert u8 {0} into a Square64")]
    FromU8(u8),

    #[error("could not convert u32 {0} into a Square64")]
    FromU32(u32),

    #[error("could not convert usize {0} into a Square64")]
    FromUsize(usize),
}

#[derive(Error, Debug, PartialEq)]
pub enum RankConversionError {
    #[error("could not convert usize {0} into a Rank")]
    FromUsize(usize),
}

#[derive(Error, Debug, PartialEq)]
pub enum FileConversionError {
    #[error("could not convert usize {0} into a File")]
    FromUsize(usize),
}
