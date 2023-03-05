use std::num::ParseIntError;

use crate::{
    board::bitboard::BitBoard,
    color::Color,
    file::File,
    gamestate::{HALF_MOVE_MAX, MAX_GAME_MOVES, NUM_FEN_SECTIONS},
    moves::Move,
    piece::Piece,
    rank::Rank,
    square::{Square, Square64},
};
use strum::{EnumCount, ParseError as StrumParseError};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ChessError {
    #[error("illegal move attempted: {illegal_move}")]
    IllegalMove { illegal_move: Move },
}

#[derive(Error, Debug, PartialEq)]
pub enum RankFenDeserializeError {
    #[error("Failed to deserialize pieces of rank from rank fen due to invalid char")]
    InvalidChar(#[from] PieceConversionError),

    #[error("Rank FEN is empty")]
    Empty,

    #[error(
        "Rank FEN {rank_fen} should represent {} squares but does not",
        File::COUNT
    )]
    InvalidNumSquares { rank_fen: String },

    #[error(
        "Rank FEN {rank_fen} includes {invalid_digit} which represents invalid digit (needs to be in 1..=8)"
    )]
    InvalidDigit {
        rank_fen: String,
        invalid_digit: usize,
    },

    #[error("Rank FEN {rank_fen} includes two consecutive digits")]
    TwoConsecutiveDigits { rank_fen: String },
}

#[derive(Error, Debug, PartialEq)]
pub enum BoardFenDeserializeError {
    #[error(transparent)]
    RankFenDeserialize(#[from] RankFenDeserializeError),

    #[error(
        "board FEN {board_fen} has {num_ranks} ranks separated by / delimiter instead of {}",
        Rank::COUNT
    )]
    WrongNumRanks { board_fen: String, num_ranks: usize },
}

#[derive(Error, Debug, PartialEq)]
pub enum BoardValidityCheckError {
    #[error("Board has {num_white_kings} WhiteKings and {num_black_kings} BlackKings, but should have exactly one of each")]
    StrictOneBlackKingOneWhiteKing {
        num_white_kings: u8,
        num_black_kings: u8,
    },

    #[error("Board has {piece_count} {piece}s, which exceeds {max_allowed} which is the maximum allowed number for that piece type")]
    StrictExceedsMaxNumForPieceType {
        piece_count: u8,
        piece: Piece,
        max_allowed: u8,
    },

    #[error(
        "A player has more excess big pieces than missing pawns which is not allowed:
            \nWhite:
            \nNumber of excess big pieces: {num_excess_big_pieces_white}
            \nNumber of missing pawns: {num_missing_pawns_white}
            \nBlack:
            \nNumber of excess big pieces: {num_excess_big_pieces_black}
            \nNumber of missing pawns: {num_missing_pawns_black}"
    )]
    StrictMoreExcessBigPiecesThanMissingPawns {
        num_excess_big_pieces_white: u8,
        num_missing_pawns_white: u8,
        num_excess_big_pieces_black: u8,
        num_missing_pawns_black: u8,
    },

    #[error("Board has a WhitePawn in Rank1 which is not a valid position")]
    StrictWhitePawnInFirstRank,

    #[error("Board has a BlackPawn in Rank8 which is not a valid position")]
    StrictBlackPawnInLastRank,

    #[error(
        "Board has Kings less than 2 squares apart from each other which is not allowed. 
    WhiteKing is at Square {white_king_square}, BlackKing is at Square{black_king_square} 
    and the distance between them is {kings_distance}"
    )]
    StrictKingsLessThanTwoSquaresApart {
        white_king_square: Square,
        black_king_square: Square,
        kings_distance: u8,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum BoardBuildError {
    #[error("Found Piece on invalid square index {invalid_square_index}")]
    PieceOnInvalidSquare {
        invalid_square_index: usize,
        #[source]
        source: SquareConversionError,
    },

    #[error(transparent)]
    BoardFenDeserialize(#[from] BoardFenDeserializeError),

    #[error(transparent)]
    BoardValidityCheck(#[from] BoardValidityCheckError),
}

impl From<SquareConversionError> for BoardBuildError {
    fn from(err: SquareConversionError) -> Self {
        match err {
            SquareConversionError::FromUsize {index} => BoardBuildError::PieceOnInvalidSquare {
                invalid_square_index: index,
                source: err
            },
            _ => panic!("Board builder check for pieces on invalid square only expects SquareConversionError of variant FromUsize")
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum GamestateBuildError {
    #[error(transparent)]
    GamestateFenDeserialize(#[from] GamestateFenDeserializeError),

    #[error(transparent)]
    GamestateValidityCheck(#[from] GamestateValidityCheckError),
}

#[derive(Error, Debug, PartialEq)]
pub enum GamestateValidityCheckError {
    #[error("Board is invalid")]
    BoardValidityCheck(#[from] BoardValidityCheckError),

    #[error("En passant square exists so the halfmove clock: {halfmove_clock} should be 0")]
    StrictEnPassantHalfmoveClockNotZero { halfmove_clock: u8 },

    #[error("En passant square {en_passant_square} is not empty")]
    StrictEnPassantNotEmpty { en_passant_square: Square },

    #[error("Square behind en passant square {square_behind} is not empty")]
    StrictEnPassantSquareBehindNotEmpty { square_behind: Square },

    #[error("Square ahead en passant square {square_ahead} is empty")]
    StrictEnPassantSquareAheadEmpty { square_ahead: Square },

    #[error("Square ahead en passant square {square_ahead} is occupied by {invalid_piece} which is not a {expected_piece}")]
    StrictEnPassantSquareAheadUnexpectedPiece {
        square_ahead: Square,
        invalid_piece: Piece,
        expected_piece: Piece,
    },

    #[error("En passant square has rank {rank} which is impossible given color {active_color}. Only valid combinations are (White, 6) and (Black, 3)")]
    StrictColorRankMismatch { active_color: Color, rank: Rank },

    #[error(
        "Halfmove clock: {halfmove_clock} should be in range 0..{}",
        HALF_MOVE_MAX
    )]
    StrictHalfmoveClockExceedsMax { halfmove_clock: u8 },

    #[error(
        "Fullmove number: {fullmove_number} should be in range 1..={}",
        MAX_GAME_MOVES
    )]
    StrictFullmoveNumberNotInRange { fullmove_number: u32 },

    #[error(
        "Given halfmove clock: {halfmove_clock}, fullmove number: {fullmove_number} is too small"
    )]
    StrictFullmoveNumberLessThanHalfmoveClockDividedByTwo {
        fullmove_number: u32,
        halfmove_clock: u8,
    },

    #[error("Non-active player in check")]
    StrictNonActivePlayerCheck,
}

#[derive(Error, Debug, PartialEq)]
pub enum GamestateFenDeserializeError {
    #[error(transparent)]
    BoardBuild(#[from] BoardBuildError),

    #[error("Gamestate failed to deserialize due to castle permissions section of FEN not representing a valid CastlePerm")]
    CastlePerm(#[from] CastlePermConversionError),

    #[error("Gamestate failed to deserialize due to en passant section of FEN not representing a valid Square")]
    EnPassant(#[from] StrumParseError),

    #[error("Gamestate failed to deserialize due to full move number section of FEN {fullmove_fen} not representing a valid number")]
    FullmoveNumber { fullmove_fen: String },

    #[error("Gamestate failed to deserialize due to half move number section of FEN {halfmove_fen} not representing a valid number")]
    HalfmoveClock { halfmove_fen: String },

    #[error(
        "active color section of {gamestate_fen} is {invalid_color}, which is an invalid Color"
    )]
    ActiveColor {
        gamestate_fen: String,
        invalid_color: String,
    },

    #[error("FEN is invalid because it is empty")]
    Empty,

    #[error(
        "number of subsections of FEN &str is {num_fen_sections}, but should be {}",
        NUM_FEN_SECTIONS
    )]
    WrongNumFENSections { num_fen_sections: usize },
}

#[derive(Error, Debug, PartialEq)]
pub enum BitBoardError {
    #[error("cannot check bit at index {invalid_index}, which is greater than 63")]
    BitBoardCheckBitInvalidIndex { invalid_index: u8 },

    #[error("cannot set bit at index {invalid_index}, which is greater than 63")]
    BitBoardSetBitInvalidIndex { invalid_index: u8 },

    #[error("cannot unset bit at index {invalid_index}, which is greater than 63")]
    BitBoardUnsetBitInvalidIndex { invalid_index: u8 },
}

#[derive(Error, Debug, PartialEq)]
pub enum PieceConversionError {
    #[error("could not convert char {invalid_char} into a Piece")]
    FromChar { invalid_char: char },

    #[error("could not convert usize {invalid_usize} into a Piece")]
    FromUsize { invalid_usize: usize },
}

#[derive(Error, Debug, PartialEq)]
pub enum CastlePermConversionError {
    #[error("could not convert u8 {invalid_u8} into a CastlePerm because {invalid_u8} is greater than 0x0F")]
    FromU8ValueTooLarge { invalid_u8: u8 },

    #[error("could not convert {invalid_string} into a CastlePerm because char {invalid_char} is invalid")]
    FromStrInvalidChar {
        invalid_string: String,
        invalid_char: char,
    },

    // Won't catch '-' duplicates
    #[error("could not convert {invalid_string} into a CastlePerm encountered duplicates")]
    FromStrDuplicates { invalid_string: String },

    #[error("could not convert {invalid_string} into a CastlePerm")]
    FromStr { invalid_string: String },
}

#[derive(Error, Debug, PartialEq)]
pub enum SquareConversionError {
    #[error(transparent)]
    FromStr(#[from] StrumParseError),

    #[error("could not convert i8: {index} into a Square")]
    FromI8 { index: i8 },

    #[error("could not convert u8: {index} into a Square")]
    FromU8 { index: u8 },

    #[error("could not convert u32: {index} into a Square")]
    FromU32 { index: u32 },

    #[error("could not convert usize: {index} into a Square")]
    FromUsize { index: usize },
}

#[derive(Error, Debug, PartialEq)]
pub enum Square64ConversionError {
    #[error("could not convert &str into a Square64. It does not represent a valid Square64")]
    FromStr(#[from] StrumParseError),

    #[error("could not convert u8 {index} into a Square64")]
    FromU8 { index: u8 },

    #[error("could not convert u32 {index} into a Square64")]
    FromU32 { index: u32 },

    #[error("could not convert usize {index} into a Square64")]
    FromUsize { index: usize },
}

#[derive(Error, Debug, PartialEq)]
pub enum RankConversionError {
    #[error("could not convert usize {invalid_usize} into a Rank")]
    FromUsize { invalid_usize: usize },
}

#[derive(Error, Debug, PartialEq)]
pub enum FileConversionError {
    #[error("could not convert usize {invalid_usize} into a File")]
    FromUsize { invalid_usize: usize },
}
