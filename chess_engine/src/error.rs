use crate::{
    board::bitboard::BitBoard,
    gamestate::{MAX_GAME_MOVES, NUM_FEN_SECTIONS},
    moves::Move,
    squares::{Square, Square64},
    util::{File, Rank},
};
use strum::{EnumCount, ParseError as StrumParseError};

use thiserror::Error;

// TODO: Clean up names and explore thiserror #from to see if you can convert
// from one type of error to another
// TODO: determine whether using &str instead of Strings is worth it
#[derive(Error, Debug, PartialEq)]
pub enum ChessError {
    #[error("illegal move attempted: {0}")]
    IllegalMove(Move),
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

    #[error("castle permissions {0} are invalid")]
    CastlePermInvalid(String),

    #[error("en passant square {0} is invalid")]
    EnPassantInvalid(String),

    #[error("half moves {0} is not a valid unsigned integer")]
    HalfmoveClockInvalid(String),

    #[error(
        "half moves {0} exceeds maximum number of half moves expected {}",
        MAX_GAME_MOVES
    )]
    HalfmoveClockExceedsMaxGameMoves(u32),

    #[error("full moves {0} is not a valid unsigned integer")]
    FullmoveClockInvalid(String),

    #[error("half moves {0} exceeds maximum number of full moves expected {}",
        MAX_GAME_MOVES/2
    )]
    FullmoveClockExceedsMaxGameMoves(u32),

    #[error("base FEN for Board {0} is invalid")]
    BoardInvalid(String),

    #[error(
        "base FEN {0} has {1} ranks separated by / delimiter instead of {}",
        Rank::COUNT
    )]
    BaseFENWrongNumRanks(String, usize),

    #[error("Rank FEN {0} should represent {} squares but does not", File::COUNT)]
    RankInvalidNumSquares(String),

    #[error(
        "Rank FEN {0} has character {1} which represents invalid digit (needs to be in 1..=8)"
    )]
    RankInvalidDigit(String, usize),

    #[error("Rank FEN {0} includes invalid char {1}")]
    RankInvalidChar(String, char),

    #[error("FEN {0} must have exactly one white king and exactly one black king")]
    InvalidKingNum(String),

    //TODO: Currently going to be a bit difficult to read with glyph
    #[error("FEN {0} includes too many {1}s (counted {2})")]
    InvalidNumOfPiece(String, String, usize),
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
pub enum ConversionError {
    #[error("could not convert char {0} into a Piece")]
    ParsePieceFromChar(char),

    #[error("could not convert u8 {0} into a CastlePerm because {0} is greater than 0x0F")]
    ParseCastlePermFromU8ErrorValueTooLarge(u8),

    #[error("could not convert {0} into a CastlePerm because char {0} is invalid")]
    ParseCastlePermFromStrInvalidChar(String, char),

    // Won't catch - duplicates
    #[error("could not convert {0} into a CastlePerm encountered duplicates")]
    ParseCastlePermFromStrDuplicates(String),

    #[error("could not convert {0} into a CastlePerm")]
    ParseCastlePermFromStr(String),

    #[error("could not convert &str {0} into a Square")]
    ParseSquareFromStr(#[from] StrumParseError),

    #[error("could not convert u8 {0} into a Square")]
    ParseSquareFromU8(u8),

    #[error("could not convert u8 {0} into a Square64")]
    ParseSquare64FromU8(u8),

    #[error("could not convert u32 {0} into a Square")]
    ParseSquareFromU32(u32),

    #[error("could not convert u32 {0} into a Square64")]
    ParseSquare64FromU32(u32),

    #[error("could not convert usize {0} into a Square")]
    ParseSquareFromUsize(usize),

    #[error("could not convert usize {0} into a Square64")]
    ParseSquare64FromUsize(usize),

    #[error("could not convert usize {0} into a Rank")]
    ParseRankFromUsize(usize),

    #[error("could not convert usize {0} into a File")]
    ParseFileFromUsize(usize),
}
