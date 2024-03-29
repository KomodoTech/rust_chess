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
pub enum UndoMoveError {
    // #[error(transparent)]
    // MoveValidity(#[from] MoveValidityError),
    #[error(transparent)]
    GamestateValidity(#[from] GamestateValidityCheckError),

    #[error(transparent)]
    SquareConversion(#[from] SquareConversionError),

    #[error("Captured Piece is invalid")]
    PieceConversion(#[from] PieceConversionError),

    #[error(transparent)]
    MoveDeserialize(#[from] MoveDeserializeError),

    #[error(transparent)]
    AddPiece(#[from] AddPieceError),

    #[error(transparent)]
    ClearPiece(#[from] ClearPieceError),

    #[error(
        "Attempted to undo a Move, but even the initial state dummy Move was not found in history"
    )]
    NoInitialState,

    #[error("Attempted to undo a Move, but no Move was found in history")]
    NoMoveToUndo,

    #[error("Move that was encoded as a castling move ends on {end_square} which is not a valid ending square for a castling move")]
    CastleEndSquare { end_square: Square },
}

#[derive(Error, Debug, PartialEq)]
pub enum MakeMoveError {
    #[error(transparent)]
    GamestateValidity(#[from] GamestateValidityCheckError),

    #[error(transparent)]
    MoveValidity(#[from] MoveValidityError),

    #[error(transparent)]
    MovePiece(#[from] MovePieceError),

    #[error(transparent)]
    ClearPiece(#[from] ClearPieceError),

    // TODO: figure out why I have to do this right now?
    #[error(transparent)]
    MoveDeserialize(#[from] MoveDeserializeError),

    #[error(transparent)]
    SquareConversion(#[from] SquareConversionError),

    #[error("Move that was encoded as a castling move ends on {end_square} which is not a valid ending square for a castling move")]
    CastleEndSquare { end_square: Square },

    #[error("Moved Piece was not found in Board pieces array")]
    MovedPieceNotInPieces,

    #[error("Cannot move into position that would put the moving side in check")]
    MoveWouldPutMovingSideInCheck,
}

#[derive(Error, Debug, PartialEq)]
pub enum MovePieceError {
    #[error("Cannot clear square {start_square}, since it is already empty")]
    NoPieceAtMoveStart { start_square: Square },

    #[error(
        "Cannot move piece {piece} to {end_square}, since it is already occupied by a {end_piece}"
    )]
    MoveEndsOnOccupiedSquare {
        piece: Piece,
        end_square: Square,
        end_piece: Piece,
    },

    #[error("Cannot find square {missing_square} in piece_list under piece {piece}")]
    SquareNotFoundInPieceList {
        missing_square: Square,
        piece: Piece,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum AddPieceError {
    #[error("Cannot add to square {occupied_square}, since it is already occupied by a {piece_at_square}")]
    AddToOccupiedSquare {
        occupied_square: Square,
        piece_at_square: Piece,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum ClearPieceError {
    #[error("Cannot clear square {empty_square}, since it is already empty")]
    NoPieceToClear { empty_square: Square },

    #[error("Cannot find square {missing_square} in piece_list under piece {piece}")]
    SquareNotFoundInPieceList {
        missing_square: Square,
        piece: Piece,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum MoveBuilderError {
    #[error(transparent)]
    MoveValidity(#[from] MoveValidityError),
}

#[derive(Error, Debug, PartialEq)]
pub enum MoveValidityError {
    #[error(transparent)]
    MoveDeserialize(#[from] MoveDeserializeError),

    #[error(transparent)]
    SquareConversion(#[from] SquareConversionError),

    #[error("This is a pawn start move, but the move begins with {active_color} at square {start_square}, which is on {start_rank} which is a contradiction.")]
    RankPawnStartMismatch {
        active_color: Color,
        start_square: Square,
        start_rank: Rank,
    },

    #[error("Move encodes an attempt to capture a {captured_piece} of the same color as the active color")]
    CaptureActiveColor { captured_piece: Piece },

    #[error("En passant move has no captured piece")]
    EnPassantNoCapture,

    #[error("Move is encoded as an en passant move, but ends on square {end_square}, which has the wrong rank. The expected rank is: {expected_rank}")]
    EnPassantWrongRank {
        end_square: Square,
        expected_rank: Rank,
    },

    #[error("Move is encoded as an en passant move, but the piece that is moved should be a {expected_piece} but is in fact a {piece_moved}")]
    EnPassantWrongPieceMoved {
        expected_piece: Piece,
        piece_moved: Piece,
    },

    #[error("If Move is encoded as being a pawn start, it cannot be a capture, a promotion, an en passant move, nor a castling move")]
    PawnStartExclusive,

    #[error("Move is encoded as pawn start but moved piece {piece_moved} is not a pawn")]
    PawnStartNonPawnMoved { piece_moved: Piece },

    #[error("Move is encoded as pawn start but the pawn is not moving two 'ahead' as it should.\nStart square: {start_square}\nEnd square:{end_square}")]
    PawnStartNotMovingTwoSpacesAhead {
        start_square: Square,
        end_square: Square,
    },

    #[error(
        "The square {end_square} is not a valid end square for castling with the {piece_moved}"
    )]
    CastleEndSquare {
        end_square: Square,
        piece_moved: Piece,
    },

    #[error(
        "The square {start_square} is not a valid end square for castling with the {piece_moved}"
    )]
    CastleStartSquare {
        start_square: Square,
        piece_moved: Piece,
    },

    #[error("Only kings can initiate castle")]
    NonKingInitiatedCastle,

    #[error("Cannot promote into a pawn")]
    PromotionToPawn,

    #[error("Cannot promote to a piece of the non-active color")]
    PromotedToNonActiveColorPiece,

    #[error("The square {end_square} is not of the valid rank for a promotion for {active_color}")]
    PromotionEndRank {
        end_square: Square,
        active_color: Color,
    },

    #[error("Move encodes a promotion but the piece moved {piece_moved} is not a pawn")]
    PromotionNonPawnMoved { piece_moved: Piece },
}

#[derive(Error, Debug, PartialEq)]
pub enum MoveDeserializeError {
    // Can't pass in move_ as String because that would cause
    // stack overflow on Display/to_string() call
    #[error("The start square {start} is invalid for move:\n {move_}")]
    Start { start: u32, move_: u32 },

    #[error("The end square {end} is invalid for move:\n {move_}")]
    End { end: u32, move_: u32 },

    #[error("The captured piece {piece} is invalid for move:\n {move_}")]
    Captured { piece: u32, move_: u32 },

    #[error("The promoted piece {piece} is invalid for move:\n {move_}")]
    Promoted { piece: u32, move_: u32 },

    #[error("The moved piece {piece} is invalid for move:\n {move_}")]
    Moved { piece: u32, move_: u32 },
}

#[derive(Error, Debug, PartialEq)]
pub enum MoveGenError {
    #[error("Cannot generate moves for invalid Gamestate")]
    GamestateValidityCheck(#[from] GamestateValidityCheckError),
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
            _ => panic!("Expected SquareConversionError of variant FromUsize (as BoardBuilder only checks for pieces on invalid squares).")
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
        "Fullmove count: {fullmove_count} should be in range 1..={}",
        MAX_GAME_MOVES
    )]
    StrictFullmoveCountNotInRange { fullmove_count: usize },

    #[error(
        "Given halfmove clock: {halfmove_clock}, fullmove count: {fullmove_count} is too small"
    )]
    StrictFullmoveCountLessThanHalfmoveClockDividedByTwo {
        fullmove_count: usize,
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

    #[error("Gamestate failed to deserialize due to full move count section of FEN {fullmove_fen} not representing a valid number")]
    FullmoveCount { fullmove_fen: String },

    #[error("Gamestate failed to deserialize due to half move clock section of FEN {halfmove_fen} not representing a valid number")]
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

    #[error("could not convert u32 {invalid_u32} into a Piece")]
    FromU32 { invalid_u32: u32 },
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
