// TODO: when bitboard errors are removed, remove pub keyword
pub mod bitboard;
use crate::{
    color::Color,
    error::{
        BoardBuildError, BoardFenDeserializeError, BoardValidityCheckError, RankFenDeserializeError,
    },
    file::File,
    gamestate::ValidityCheck,
    piece::{Piece, PieceType},
    rank::Rank,
    square::{Square, Square64},
};
use bitboard::BitBoard;
use std::{
    collections::HashMap,
    fmt::{self, write},
};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

/// Number of squares for the internal board (10x12)
pub const NUM_INTERNAL_BOARD_SQUARES: usize = 120;
/// Number of squares for the external board (8x8)
pub const NUM_EXTERNAL_BOARD_SQUARES: usize = 64;
/// Number of columns for the internal board (10x12)
pub const NUM_BOARD_COLUMNS: usize = 10;
/// Number of rows for the internal board (10x12)
pub const NUM_BOARD_ROWS: usize = 12;

#[rustfmt::skip]
const STARTING_POSITION_PIECES: [Option<Piece>; NUM_INTERNAL_BOARD_SQUARES] = [
    None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
    None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
    None, Some(Piece::WhiteRook), Some(Piece::WhiteKnight), Some(Piece::WhiteBishop), Some(Piece::WhiteQueen), Some(Piece::WhiteKing), Some(Piece::WhiteBishop), Some(Piece::WhiteKnight), Some(Piece::WhiteRook), None,
    None, Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn),  Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn), None,
    None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
    None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
    None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
    None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
    None, Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn),  Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn), None,
    None, Some(Piece::BlackRook), Some(Piece::BlackKnight), Some(Piece::BlackBishop), Some(Piece::BlackQueen), Some(Piece::BlackKing), Some(Piece::BlackBishop), Some(Piece::BlackKnight), Some(Piece::BlackRook), None,
    None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
    None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
];

#[derive(Debug)]
pub struct BoardBuilder {
    validity_check: ValidityCheck,
    pieces: [Option<Piece>; NUM_INTERNAL_BOARD_SQUARES],
}

impl BoardBuilder {
    pub fn new() -> Self {
        BoardBuilder {
            validity_check: ValidityCheck::Strict,
            pieces: [None; NUM_INTERNAL_BOARD_SQUARES],
        }
    }

    /// Constructor if you want to pass piece values all at once (you can still overwrite them later)
    pub fn new_with_pieces(pieces: [Option<Piece>; NUM_INTERNAL_BOARD_SQUARES]) -> Self {
        BoardBuilder {
            validity_check: ValidityCheck::Strict,
            pieces,
        }
    }

    /// Constructor if you want to pass board values by FEN.
    pub fn new_with_fen(board_fen: &str) -> Result<Self, BoardBuildError> {
        let pieces = Self::pieces_from_fen(board_fen)?;
        Ok(BoardBuilder {
            validity_check: ValidityCheck::Strict,
            pieces,
        })
    }

    /// Set the validity check mode. Defaults to strict to make sure regular chess checks are only off when it is intentional
    pub fn validity_check(&mut self, validity_check: ValidityCheck) -> &mut Self {
        self.validity_check = validity_check;
        self
    }

    /// Change one piece at a time
    pub fn piece(&mut self, piece: Piece, square: Square64) -> &mut Self {
        self.pieces[(Square::from(square)) as usize] = Some(piece);
        self
    }

    /// Finalizer function. Has access to pieces and generates everything else.
    /// Given that the validity check can fail, building has to return a Result
    pub fn build(&self) -> Result<Board, BoardBuildError> {
        let mut pawns: [BitBoard; Color::COUNT] = [BitBoard(0), BitBoard(0)];
        let mut kings_square: [Option<Square>; Color::COUNT] = [None; Color::COUNT];

        let mut piece_count: [u8; Piece::COUNT] = [0; Piece::COUNT];
        let mut big_piece_count: [u8; Color::COUNT] = [0; Color::COUNT];
        let mut major_piece_count: [u8; Color::COUNT] = [0; Color::COUNT];
        let mut minor_piece_count: [u8; Color::COUNT] = [0; Color::COUNT];
        let mut material_score: [u32; Color::COUNT] = [0; Color::COUNT];
        let mut piece_list: [Vec<Square>; Piece::COUNT] = Default::default();

        // Note: pieces are being cloned here so that we can create multiple boards.
        // from the same builder. Optimizer might elide clones/copies if you
        // don't reuse the BoardBuilder, but not sure.
        let pieces = self.pieces;

        for (index, piece) in pieces.into_iter().enumerate() {
            if let Some(piece) = piece {
                // check that no Piece is found on an invalid 10x12 square
                let square = Square::try_from(index)?;
                let color = piece.get_color();

                piece_count[piece as usize] += 1;

                // update material_score
                material_score[piece.get_color() as usize] += piece.get_value();

                match piece {
                    pawn if piece.get_piece_type() == PieceType::Pawn => {
                        pawns[color as usize].set_bit(Square64::from(square))
                    }
                    king if piece.get_piece_type() == PieceType::King => {
                        big_piece_count[color as usize] += 1;
                        major_piece_count[color as usize] += 1;

                        kings_square[color as usize] = Some(square)
                    }
                    other_piece => {
                        big_piece_count[color as usize] += 1;
                        match other_piece {
                            major if other_piece.is_major() => {
                                major_piece_count[color as usize] += 1
                            }
                            minor => minor_piece_count[color as usize] += 1,
                        }
                    }
                }
                piece_list[piece as usize].push(square);
            }
        }

        let board = Board {
            pieces,
            pawns,
            kings_square,
            piece_count,
            big_piece_count,
            major_piece_count,
            minor_piece_count,
            material_score,
            piece_list,
        };

        // NOTE: Basic mode doesn't do any extra board checking and that's probably not going to change
        if let ValidityCheck::Strict = self.validity_check {
            board.check_board(self.validity_check)?;
        }
        Ok(board)
    }

    /// Generates the pieces array of a Board struct given a board fen
    fn pieces_from_fen(
        board_fen: &str,
    ) -> Result<[Option<Piece>; NUM_INTERNAL_BOARD_SQUARES], BoardFenDeserializeError> {
        let mut pieces = [None; NUM_INTERNAL_BOARD_SQUARES];

        let ranks: Vec<&str> = board_fen.split('/').collect();
        match ranks.len() {
            Rank::COUNT => {
                // NOTE: board_fen is in reverse order with regards to ranks compared to our internal board representation
                for (rank, rank_str) in ranks.into_iter().rev().enumerate() {
                    let rank_pieces = Self::rank_from_fen(rank_str)?;

                    for (file, piece) in rank_pieces.into_iter().enumerate() {
                        let square = Square::from_file_and_rank(
                            File::try_from(file).expect("file should be in range 0..=7"),
                            Rank::try_from(rank).expect("rank should be in range 0..=7"),
                        );
                        pieces[square as usize] = piece;
                    }
                }
                Ok(pieces)
            }
            _ => Err(BoardFenDeserializeError::WrongNumRanks {
                board_fen: board_fen.to_owned(),
                num_ranks: ranks.len(),
            }),
        }
    }

    /// Takes a &str that corresponds to a portion of a FEN string for a specific Rank (e.g. rnbqkbnr)
    /// and generates a corresponding Option<Piece> array
    fn rank_from_fen(
        rank_fen: &str,
    ) -> Result<[Option<Piece>; File::COUNT], RankFenDeserializeError> {
        match rank_fen {
            rank_fen if !rank_fen.is_empty() => {
                let mut rank = [None; File::COUNT];
                let mut is_last_char_digit = false;
                let mut square_counter = 0;

                for char in rank_fen.chars() {
                    match char.to_digit(10) {
                        Some(digit) => {
                            match is_last_char_digit {
                                // check if there are two digits in a row to catch invalid rank fen
                                // e.g. "ppp12pp"
                                true => {
                                    return Err(RankFenDeserializeError::TwoConsecutiveDigits {
                                        rank_fen: rank_fen.to_owned(),
                                    })
                                }
                                false => {
                                    is_last_char_digit = true;
                                }
                            }

                            // if the char is a digit in the range (1..=8) we need to check
                            // that adding digit-number of unoccupied squares would not push us past our 8 square per rank limit
                            match digit {
                                digit if (1..=File::COUNT).contains(&(digit as usize)) => {
                                    let index_after_insert = square_counter + digit;
                                    match index_after_insert {
                                        index_after_insert
                                            if index_after_insert <= File::COUNT as u32 =>
                                        {
                                            for index in square_counter..index_after_insert {
                                                rank[index as usize] = None;
                                            }
                                            square_counter += digit;
                                        }
                                        _ => {
                                            return Err(
                                                RankFenDeserializeError::InvalidNumSquares {
                                                    rank_fen: rank_fen.to_owned(),
                                                },
                                            )
                                        }
                                    }
                                }
                                _ => {
                                    return Err(RankFenDeserializeError::InvalidDigit {
                                        rank_fen: rank_fen.to_owned(),
                                        invalid_digit: digit as usize,
                                    })
                                }
                            }
                        }
                        None => {
                            is_last_char_digit = false;

                            let piece = Piece::try_from(char)?;
                            let index_after_insert = square_counter + 1;
                            match index_after_insert {
                                index_after_insert if index_after_insert <= File::COUNT as u32 => {
                                    rank[square_counter as usize] = Some(piece);
                                    square_counter += 1;
                                }
                                _ => {
                                    return Err(RankFenDeserializeError::InvalidNumSquares {
                                        rank_fen: rank_fen.to_owned(),
                                    })
                                }
                            }
                        }
                    }
                }
                match square_counter as usize {
                    File::COUNT => Ok(rank),
                    _ => Err(RankFenDeserializeError::InvalidNumSquares {
                        rank_fen: rank_fen.to_owned(),
                    }),
                }
            }
            empty => Err(RankFenDeserializeError::Empty),
        }
    }
}

impl Default for BoardBuilder {
    fn default() -> Self {
        let mut default_board_builder = BoardBuilder::new_with_pieces(STARTING_POSITION_PIECES);
        default_board_builder.validity_check(ValidityCheck::Basic);
        default_board_builder
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Board {
    // TODO: Consider making board field private
    pub pieces: [Option<Piece>; NUM_INTERNAL_BOARD_SQUARES],
    pawns: [BitBoard; Color::COUNT],
    kings_square: [Option<Square>; Color::COUNT],
    piece_count: [u8; Piece::COUNT],
    big_piece_count: [u8; Color::COUNT],
    major_piece_count: [u8; Color::COUNT],
    minor_piece_count: [u8; Color::COUNT],
    material_score: [u32; Color::COUNT],
    /// Stores position of each piece to avoid searching through all squares
    piece_list: [Vec<Square>; Piece::COUNT],
}

/// Returns an a Board with the default starting position in regular chess.
impl Default for Board {
    fn default() -> Self {
        BoardBuilder::default()
            .build()
            .expect("starting position should never fail to build")
    }
}

/// Attempts to deserialize board fen into Board
impl TryFrom<&str> for Board {
    type Error = BoardBuildError;
    fn try_from(board_fen: &str) -> Result<Self, Self::Error> {
        BoardBuilder::new_with_fen(board_fen)?.build()
    }
}

impl Board {
    //======================== GETTERS ========================================
    pub fn get_piece_count(&self) -> [u8; Piece::COUNT] {
        self.piece_count
    }

    pub fn get_piece_list(&self) -> &[Vec<Square>; Piece::COUNT] {
        &self.piece_list
    }
    //=========================================================================

    /// Checks the board to make sure that it is consistent with the ValidityCheck/mode
    pub fn check_board(
        &self,
        validity_check: ValidityCheck,
    ) -> Result<(), BoardValidityCheckError> {
        // TODO:
        // check that there aren't more than 6 pawns in a single file
        // check minimum number of enemy missing pieces doesn't contradict number of pawns in a single file
        // check general version of if there are white pawns in A2 and A3, there can't be one in B2
        // pawn + (pawn || bishop || knight) ||  (knight + knight)
        // check for non-jumpers in impossible positions
        // look for bishops trapped behind non-enemy pawns (or behind 3 pawns)
        // check that bishops are on squares that have the same color as them

        if let ValidityCheck::Strict = validity_check {
            // TODO: be sure that the piece counts can't go out of sync and don't need to be checked
            // check that there is exactly one BlackKing and one WhiteKing
            if !(self.piece_count[Piece::WhiteKing as usize] == 1
                && self.piece_count[Piece::BlackKing as usize] == 1)
            {
                return Err(BoardValidityCheckError::StrictOneBlackKingOneWhiteKing {
                    num_white_kings: self.piece_count[Piece::WhiteKing as usize],
                    num_black_kings: self.piece_count[Piece::BlackKing as usize],
                });
            }

            // check that kings are separated by at least 1 square
            let white_king_square = self.kings_square[Color::White as usize]
                .expect("White king should be on board since we checked piece_counts was 1 for it");
            let black_king_square = self.kings_square[Color::Black as usize]
                .expect("Black king should be on board since we checked piece_counts was 1 for it");
            let kings_distance =
                Square::get_chebyshev_distance(white_king_square, black_king_square);
            if kings_distance < 2 {
                return Err(
                    BoardValidityCheckError::StrictKingsLessThanTwoSquaresApart {
                        white_king_square,
                        black_king_square,
                        kings_distance,
                    },
                );
            }

            let mut num_excess_big_pieces = [0, 0];

            for (index, piece_count) in self.piece_count.into_iter().enumerate() {
                // check that the max amount of piece type leq max allowed for that piece
                let piece = Piece::try_from(index).unwrap(); // index should always be in range 0..12
                let max_allowed = piece.get_max_num_allowed();
                if piece_count > max_allowed {
                    return Err(BoardValidityCheckError::StrictExceedsMaxNumForPieceType {
                        piece_count,
                        piece,
                        max_allowed,
                    });
                }

                let num_excess_big = piece_count as i8 - piece.get_starting_num() as i8;
                if num_excess_big > 0 {
                    num_excess_big_pieces[piece.get_color() as usize] += num_excess_big as u8;
                }
            }

            // NOTE: Needs to be after check that makes sure that you can't have more than 8 pawns
            // otherwise you can subtract with overflow
            let num_missing_pawns = [
                File::COUNT as u8 - self.pawns[0].count_bits(),
                File::COUNT as u8 - self.pawns[1].count_bits(),
            ];

            // NOTE: this check is trying to detect obvious promoted pieces in conflict with number of missing pawns
            // but if you lost a piece and then promoted to the same piece type, that can't be detected from the board

            // check that there aren't more excess big pieces than missing pawns per color
            if num_excess_big_pieces[Color::White as usize]
                > num_missing_pawns[Color::White as usize]
                || num_excess_big_pieces[Color::Black as usize]
                    > num_missing_pawns[Color::Black as usize]
            {
                return Err(
                    BoardValidityCheckError::StrictMoreExcessBigPiecesThanMissingPawns {
                        num_excess_big_pieces_white: num_excess_big_pieces[Color::White as usize],
                        num_missing_pawns_white: num_missing_pawns[Color::White as usize],
                        num_excess_big_pieces_black: num_excess_big_pieces[Color::Black as usize],
                        num_missing_pawns_black: num_missing_pawns[Color::Black as usize],
                    },
                );
            }

            for (index, piece) in self.pieces.into_iter().enumerate() {
                if let Some(piece) = piece {
                    let square = Square::try_from(index).expect("building the board should guarantee that there are no pieces on invalid squares");
                    match piece {
                        Piece::WhitePawn => {
                            // check that there aren't any WhitePawns in first rank
                            if let Rank::Rank1 = square.get_rank() {
                                return Err(BoardValidityCheckError::StrictWhitePawnInFirstRank);
                            }
                        }
                        Piece::BlackPawn => {
                            // check that there aren't any BlackPawns in last rank
                            if let Rank::Rank8 = square.get_rank() {
                                return Err(BoardValidityCheckError::StrictBlackPawnInLastRank);
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        Ok(())
    }

    /// Serializes board position into board FEN. Does not do any validity checking so will just
    /// ignore any pieces on invalid squares
    pub fn to_board_fen(&self) -> String {
        let mut board_fen = String::new();
        let mut empty_count: u32 = 0;

        for rank in Rank::iter().rev() {
            // Reset here so you don't add up trailing Nones from each row
            empty_count = 0;

            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);

                match self.pieces[square as usize] {
                    Some(piece) => {
                        if empty_count > 0 {
                            board_fen.push(char::from_digit(empty_count, 10).expect(
                                "should not fail since File::COUNT is 8 and we reset empty_count",
                            ));
                        }
                        board_fen.push(piece.into());
                        empty_count = 0;
                    }

                    None => {
                        empty_count += 1;
                        // If you end a rank on a None, then you need to append the digit (empty_count)
                        if file == File::FileH {
                            board_fen.push(char::from_digit(empty_count, 10).expect(
                                "should not fail since File::COUNT is 8 and we reset empty_count",
                            ))
                        }
                    }
                }
            }

            // Don't print out a trailing '/'
            if (rank as usize) > 0 {
                board_fen.push('/')
            }
        }

        board_fen
    }

    // /// Combines white and black pawn positions into one BitBoard. Assumes that you never
    // /// have a black and a white pawn occupying the same position
    // pub fn get_all_pawns(&self) -> BitBoard {
    //     BitBoard((self.pawns[0]).0 | (self.pawns[1]).0)
    // }

    // /// Returns piece occupying given square or None if square is empty
    // pub fn get_piece_at(&self, square: Square) -> Option<Piece> {
    //     todo!()
    // }

    // /// Clears a given square and returns the piece occupying square or None if square was empty
    // pub fn clear_square(&mut self, square: Square) -> Option<Piece> {
    //     todo!()
    // }

    // /// Places new piece on given square.
    // /// Returns the piece previously occupying square or None if square was empty
    // pub fn add_piece(&mut self, square: Square, piece: Piece) -> Option<Piece> {
    //     todo!()
    // }

    // /// Clears board
    // pub fn clear_board(&mut self) {
    //     self.pieces = [None; NUM_INTERNAL_BOARD_SQUARES];
    //     self.pawns = [BitBoard(0); Color::COUNT];
    //     self.kings_square = [None; Color::COUNT];
    //     self.big_piece_count = [0; Color::COUNT];
    //     self.major_piece_count = [0; Color::COUNT];
    //     self.minor_piece_count = [0; Color::COUNT];
    //     self.piece_count = [0; Piece::COUNT];
    //     self.piece_list = Default::default();
    // }
}

// TODO: flip board display so black is at the top and white at the bottom
// TODO: use shorter version of rank and file names
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Reverse ranks so the board is presented like a chess board typically is
        // with white at the bottom
        for rank in Rank::iter().rev() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                let piece = self.pieces[square as usize];
                match file {
                    // Add rank number at the start of each rank
                    File::FileA => {
                        write!(f, "{}\t", rank as u8 + 1);
                        match piece {
                            Some(p) => {
                                write!(f, "{}\t", p);
                            }
                            None => {
                                write!(f, ".\t");
                            }
                        }
                    }
                    // Don't add tab at the end of rank
                    File::FileH => match piece {
                        Some(p) => {
                            write!(f, "{}", p);
                        }
                        _ => {
                            write!(f, ".");
                        }
                    },
                    _ => match piece {
                        Some(p) => {
                            write!(f, "{}\t", p);
                        }
                        _ => {
                            write!(f, ".\t");
                        }
                    },
                }
            }
            if rank != Rank::Rank1 {
                writeln!(f);
            }
        }
        // Add File legend at the bottom
        write!(f, "\n\n\t");
        for file in File::iter() {
            match file {
                // Don't add trailing tab
                File::FileH => {
                    write!(f, "{}", char::from(file));
                }
                _ => {
                    write!(f, "{}\t", char::from(file));
                }
            }
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use std::{default, fmt::format};

    use crate::error::{PieceConversionError, SquareConversionError};

    use super::*;

    const EMPTY_BOARD_FEN: &str = "8/8/8/8/8/8/8/8";
    #[rustfmt::skip]
    const EMPTY_BOARD: Board = Board {
        pieces: [
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None,
        ],
        pawns: [BitBoard(0), BitBoard(0)],
        kings_square: [None, None],
        piece_count: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        big_piece_count: [0, 0],
        major_piece_count: [0, 0],
        minor_piece_count: [0, 0],
        material_score: [0, 0],
        piece_list: [ vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![]]
    };

    //-----------------------------------------------------------------------------
    //============================== Miscellaneous Tests ==========================

    //============================== Display ======================================
    #[rustfmt::skip]
    #[test]
    fn test_board_display() {
        let input = Board::default();
        let output = input.to_string();
        let expected = format!("{}{}{}{}{}{}{}{}{}",
                            "8\t♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n",
                            "7\t♟\t♟\t♟\t♟\t♟\t♟\t♟\t♟\n",
                            "6\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "5\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "4\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "3\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "2\t♙\t♙\t♙\t♙\t♙\t♙\t♙\t♙\n",
                            "1\t♖\t♘\t♗\t♕\t♔\t♗\t♘\t♖\n\n",
                            "\tA\tB\tC\tD\tE\tF\tG\tH\n"
                        );
        println!("Base Board:\n{}", output);
        assert_eq!(output, expected);
    }

    //-----------------------------------------------------------------------------
    //============================== Basic Board Building =========================

    #[test]
    fn test_board_build_empty() {
        let output = BoardBuilder::new()
            .validity_check(ValidityCheck::Basic)
            .build();
        let expected = Ok(EMPTY_BOARD);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_default() {
        let output = Board::default();
        #[rustfmt::skip]
        let expected = Board {
            pieces: [
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, Some(Piece::WhiteRook), Some(Piece::WhiteKnight), Some(Piece::WhiteBishop), Some(Piece::WhiteQueen), Some(Piece::WhiteKing), Some(Piece::WhiteBishop), Some(Piece::WhiteKnight), Some(Piece::WhiteRook), None,
                None, Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn),  Some(Piece::WhitePawn), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn), None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn),  Some(Piece::BlackPawn), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn), None,
                None, Some(Piece::BlackRook), Some(Piece::BlackKnight), Some(Piece::BlackBishop), Some(Piece::BlackQueen), Some(Piece::BlackKing), Some(Piece::BlackBishop), Some(Piece::BlackKnight), Some(Piece::BlackRook), None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                   None,                     None,                     None,                   None,
            ],      
            pawns: [BitBoard(0x000000000000FF00), BitBoard(0x00FF000000000000)],
            kings_square: [Some(Square::E1), Some(Square::E8)],
            piece_count: [8, 2, 2, 2, 1, 1, 8, 2, 2, 2, 1, 1],
            big_piece_count: [8, 8],
            // NOTE: King considered major piece for us
            major_piece_count: [4, 4],
            minor_piece_count: [4, 4],
            material_score: [54_200, 54_200],
            piece_list: [
                // WhitePawns
                vec![Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2],
                // WhiteKnights
                vec![Square::B1, Square::G1],
                // WhiteBishops
                vec![Square::C1, Square::F1],
                // WhiteRooks
                vec![Square::A1, Square::H1],
                // WhiteQueens
                vec![Square::D1],
                // WhiteKing
                vec![Square::E1],
                // BlackPawns
                vec![Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7],
                // BlackKnights
                vec![Square::B8, Square::G8],
                // BlackBishops
                vec![Square::C8, Square::F8],
                // BlackRooks
                vec![Square::A8, Square::H8],
                // BlackQueens
                vec![Square::D8],
                // BlackKing
                vec![Square::E8],
            ]
        };

        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_piece_on_invalid_square() {
        #[rustfmt::skip]
        let pieces = [
            Some(Piece::WhitePawn), None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
            None,                   None, None, None, None, None, None, None, None, None,
        ];

        let output = BoardBuilder::new_with_pieces(pieces).build();
        // let expected = Err(BoardBuildError::from(SquareConversionError::FromUsize{index: 0}));
        let expected = Err(BoardBuildError::PieceOnInvalidSquare {
            invalid_square_index: 0,
            source: SquareConversionError::FromUsize { index: 0 },
        });
        assert_eq!(output, expected);
    }

    //-----------------------------------------------------------------------------
    //============================== Board Validity Checks ========================
    //============================== Strict Mode ==================================

    #[test]
    fn test_board_build_strict_validity_check_invalid_no_pieces() {
        // 8/8/8/8/8/8/8/8
        let output = BoardBuilder::new().build();
        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictOneBlackKingOneWhiteKing {
                num_white_kings: 0,
                num_black_kings: 0,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_strict_validity_check_invalid_too_few_kings() {
        // 8/8/8/8/8/8/2K5/8
        let output = BoardBuilder::new()
            .piece(Piece::WhiteKing, Square64::C2)
            .build();
        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictOneBlackKingOneWhiteKing {
                num_white_kings: 1,
                num_black_kings: 0,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_strict_validity_check_invalid_too_many_kings() {
        // 8/2k5/8/8/8/8/2K1K3/8
        let output = BoardBuilder::new()
            .piece(Piece::WhiteKing, Square64::C2)
            .piece(Piece::WhiteKing, Square64::E2)
            .piece(Piece::BlackKing, Square64::C7)
            .build();
        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictOneBlackKingOneWhiteKing {
                num_white_kings: 2,
                num_black_kings: 1,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_strict_validity_check_invalid_kings_too_close() {
        // 8/8/8/4k3/4K3/8/8/8
        let output = BoardBuilder::new()
            .piece(Piece::WhiteKing, Square64::E4)
            .piece(Piece::BlackKing, Square64::E5)
            .build();

        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictKingsLessThanTwoSquaresApart {
                white_king_square: Square::E4,
                black_king_square: Square::E5,
                kings_distance: 1,
            },
        ));

        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_strict_validity_check_invalid_too_many_white_queens() {
        // 6rk/6bp/8/8/8/8/QQQQQQQQ/3QKQ2 chosen in the hopes that it would be valid otherwise (bk is not in checkmate)

        let mut output = BoardBuilder::new();
        output
            .piece(Piece::WhiteQueen, Square64::D1)
            .piece(Piece::WhiteKing, Square64::E1)
            .piece(Piece::WhiteQueen, Square64::F1)
            .piece(Piece::BlackBishop, Square64::G7)
            .piece(Piece::BlackPawn, Square64::H7)
            .piece(Piece::BlackRook, Square64::G8)
            .piece(Piece::BlackKing, Square64::H8);

        for i in 8_u8..=15 {
            output.piece(Piece::WhiteQueen, Square64::try_from(i).unwrap());
        }
        let output = output.build();

        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictExceedsMaxNumForPieceType {
                piece_count: 10,
                piece: Piece::WhiteQueen,
                max_allowed: 9,
            },
        ));

        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_strict_validity_check_invalid_too_many_white_pawns() {
        // 4k3/8/8/8/8/P7/PPPPPPPP/4K3
        let mut output = BoardBuilder::new();
        output
            .piece(Piece::WhiteKing, Square64::E1)
            .piece(Piece::BlackKing, Square64::E8);

        let mut square = Square64::A2;
        for i in 0..9 {
            output.piece(Piece::WhitePawn, square);
            square = (square + 1).unwrap();
        }

        let output = output.build();
        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictExceedsMaxNumForPieceType {
                piece_count: 9,
                piece: Piece::WhitePawn,
                max_allowed: 8,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_strict_validity_check_invalid_more_excess_big_pieces_than_missing_pawns() {
        // 3k4/8/8/8/8/8/PPPPPPPP/1NNK2N1
        let mut output = BoardBuilder::new();
        output
            .piece(Piece::WhiteKing, Square64::D1)
            .piece(Piece::WhiteKnight, Square64::B1)
            .piece(Piece::WhiteKnight, Square64::C1)
            .piece(Piece::WhiteKnight, Square64::G1)
            .piece(Piece::BlackKing, Square64::D8);

        for i in 8_u8..=15 {
            output.piece(Piece::WhitePawn, Square64::try_from(i).unwrap());
        }

        let output = output.build();
        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictMoreExcessBigPiecesThanMissingPawns {
                num_excess_big_pieces_white: 1,
                num_missing_pawns_white: 0,
                num_excess_big_pieces_black: 0,
                num_missing_pawns_black: 8,
            },
        ));

        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_strict_validity_check_invalid_white_pawn_in_rank_1() {
        let output = BoardBuilder::new()
            .piece(Piece::WhitePawn, Square64::A1)
            .piece(Piece::WhiteKing, Square64::B1)
            .piece(Piece::BlackKing, Square64::B8)
            .build();
        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictWhitePawnInFirstRank,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_build_strict_validity_check_invalid_black_pawn_in_rank_8() {
        let output = BoardBuilder::new()
            .piece(Piece::BlackPawn, Square64::A8)
            .piece(Piece::WhiteKing, Square64::B1)
            .piece(Piece::BlackKing, Square64::B8)
            .build();
        let expected = Err(BoardBuildError::BoardValidityCheck(
            BoardValidityCheckError::StrictBlackPawnInLastRank,
        ));
        assert_eq!(output, expected);
    }

    //============================== Basic Mode ===================================

    #[test]
    fn test_board_build_basic_validity_check_black_pawn_in_rank_8() {
        let output = BoardBuilder::new()
            .validity_check(ValidityCheck::Basic)
            .piece(Piece::BlackPawn, Square64::A8)
            .piece(Piece::WhiteKing, Square64::B1)
            .piece(Piece::BlackKing, Square64::B8)
            .build();
        #[rustfmt::skip]
        let expected = Ok(Board {
            pieces: [
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, None,                   Some(Piece::WhiteKing), None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, Some(Piece::BlackPawn), Some(Piece::BlackKing), None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
                None, None,                   None,                   None, None, None, None, None, None, None,
            ],
            pawns: [BitBoard(0), BitBoard(0x01_00_00_00_00_00_00_00)],
            kings_square: [Some(Square::B1), Some(Square::B8)],
            piece_count: [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1],
            big_piece_count: [1, 1],
            major_piece_count: [1, 1],
            minor_piece_count: [0, 0],
            material_score: [50_000, 50_100],
            piece_list: [
                // WhitePawns
                vec![],
                // WhiteKnights
                vec![],
                // WhiteBishops
                vec![],
                // WhiteRooks
                vec![],
                // WhiteQueens
                vec![],
                // WhiteKing
                vec![Square::B1],
                // BlackPawns
                vec![Square::A8],
                // BlackKnights
                vec![],
                // BlackBishops
                vec![],
                // BlackRooks
                vec![],
                // BlackQueens
                vec![],
                // BlackKing
                vec![Square::B8],
            ],
        });
        assert_eq!(output, expected);
    }

    //------------------------------------------------------------------------------
    //============================= FEN PARSING ====================================
    //======================== Rank Level FEN Parsing ==============================
    #[test]
    fn test_get_rank_from_fen_valid_black_back_row_starting_position() {
        let input = "rnbqkbnr";
        let output = BoardBuilder::rank_from_fen(input);
        let expected = Ok([
            Some(Piece::BlackRook),
            Some(Piece::BlackKnight),
            Some(Piece::BlackBishop),
            Some(Piece::BlackQueen),
            Some(Piece::BlackKing),
            Some(Piece::BlackBishop),
            Some(Piece::BlackKnight),
            Some(Piece::BlackRook),
        ]);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_valid_gaps() {
        let input = "rn2kb1r";
        let output = BoardBuilder::rank_from_fen(input);
        let expected = Ok([
            Some(Piece::BlackRook),
            Some(Piece::BlackKnight),
            None,
            None,
            Some(Piece::BlackKing),
            Some(Piece::BlackBishop),
            None,
            Some(Piece::BlackRook),
        ]);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_valid_no_pieces() {
        let input = "8";
        let output = BoardBuilder::rank_from_fen(input);
        let expected = Ok([None; 8]);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_empty() {
        let input = "";
        let output = BoardBuilder::rank_from_fen(input);
        let expected = Err(RankFenDeserializeError::Empty);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_char() {
        let input = "rn2Xb1r";
        let output = BoardBuilder::rank_from_fen(input);
        let expected = Err(RankFenDeserializeError::InvalidChar(
            PieceConversionError::FromChar { invalid_char: 'X' },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_digit() {
        let input = "rn0kb1rqN"; // num squares would be valid
        let output = BoardBuilder::rank_from_fen(input);
        let expected = Err(RankFenDeserializeError::InvalidDigit {
            rank_fen: input.to_owned(),
            invalid_digit: 0,
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_too_many_squares() {
        let input = "rn2kb1rqN";
        let output = BoardBuilder::rank_from_fen(input);
        let expected = Err(RankFenDeserializeError::InvalidNumSquares {
            rank_fen: input.to_owned(),
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_too_few_squares() {
        let input = "rn2kb";
        let output = BoardBuilder::rank_from_fen(input);
        let expected = Err(RankFenDeserializeError::InvalidNumSquares {
            rank_fen: input.to_owned(),
        });
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_two_consecutive_digits() {
        let input = "pppp12p"; // adds up to 8 squares but isn't valid
        let ouput = BoardBuilder::rank_from_fen(input);
        let expected = Err(RankFenDeserializeError::TwoConsecutiveDigits {
            rank_fen: input.to_owned(),
        });
        assert_eq!(ouput, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_two_consecutive_digits_invalid_num_squares() {
        let input = "pppp18p"; // adds up to more than 8 squares but gets caught for consecutive digits
        let ouput = BoardBuilder::rank_from_fen(input);
        let expected = Err(RankFenDeserializeError::TwoConsecutiveDigits {
            rank_fen: input.to_owned(),
        });
        assert_eq!(ouput, expected);
    }

    //==================================== Board Level FEN Serialization  ================
    #[test]
    fn test_board_serialization_sliding_and_kings() {
        let input = BoardBuilder::new()
            .piece(Piece::WhiteRook, Square64::A1)
            .piece(Piece::WhiteKing, Square64::E1)
            .piece(Piece::WhiteRook, Square64::H1)
            .piece(Piece::WhiteBishop, Square64::H4)
            .piece(Piece::BlackBishop, Square64::B7)
            .piece(Piece::BlackKing, Square64::E7)
            .piece(Piece::BlackBishop, Square64::G7)
            .piece(Piece::BlackQueen, Square64::H7)
            .piece(Piece::BlackRook, Square64::A8)
            .piece(Piece::BlackRook, Square64::H8)
            .build()
            .unwrap();

        let output = input.to_board_fen();
        let expected = "r6r/1b2k1bq/8/8/7B/8/8/R3K2R".to_owned();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_serialization_empty() {
        let input = BoardBuilder::new()
            .validity_check(ValidityCheck::Basic)
            .build()
            .unwrap();
        let output = input.to_board_fen();
        let expected = "8/8/8/8/8/8/8/8";
        assert_eq!(output, expected);
    }

    // to_board_fen does not check for pieces on invalid squares
    #[test]
    fn test_board_serialization_pieces_on_invalid_squares() {
        let input = Board {
            #[rustfmt::skip]
            pieces:  [
                Some(Piece::BlackBishop), None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
                None,                     None, None, None, None, None, None, None, None, None,
            ],

            pawns: [BitBoard(0), BitBoard(0)],
            kings_square: [None, None],
            piece_count: [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
            big_piece_count: [0, 1],
            major_piece_count: [0, 0],
            minor_piece_count: [0, 1],
            material_score: [0, 0],
            piece_list: [
                // WhitePawns
                vec![],
                // WhiteKnights
                vec![],
                // WhiteBishops
                vec![],
                // WhiteRooks
                vec![],
                // WhiteQueens
                vec![],
                // WhiteKing
                vec![],
                // BlackPawns
                vec![],
                // BlackKnights
                vec![],
                // BlackBishops
                vec![],
                // BlackRooks
                vec![],
                // BlackQueens
                vec![],
                // BlackKing
                vec![],
            ],
        };

        let output = input.to_board_fen();
        let expected = "8/8/8/8/8/8/8/8";
        assert_eq!(output, expected);
    }

    //==================================== Board Level FEN Deserialization  ================
    #[test]
    fn test_board_try_from_valid_board_fen_sliding_and_kings() {
        let input = "r6r/1b2k1bq/8/8/7B/8/8/R3K2R";
        let output = Board::try_from(input);
        #[rustfmt::skip]
        let expected = Ok(Board {
            pieces: [
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                     None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                     None,
                None, Some(Piece::WhiteRook), None,                     None,                     None,                    Some(Piece::WhiteKing),  None,                     None,                     Some(Piece::WhiteRook),   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                     None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                     None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     Some(Piece::WhiteBishop), None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                     None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                     None,
                None, None,                   Some(Piece::BlackBishop), None,                     None,                    Some(Piece::BlackKing),  None,                     Some(Piece::BlackBishop), Some(Piece::BlackQueen),  None,
                None, Some(Piece::BlackRook), None,                     None,                     None,                    None,                    None,                     None,                     Some(Piece::BlackRook),   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                     None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                     None,
            ],
            pawns: [BitBoard(0), BitBoard(0)],
            kings_square: [Some(Square::E1), Some(Square::E7)],
            piece_count: [0, 0, 1, 2, 0, 1, 0, 0, 2, 2, 1, 1],
            big_piece_count: [4, 6],
            major_piece_count: [3, 4],
            minor_piece_count: [1, 2],
            material_score: [51_425, 52_750],
            piece_list: [
                // WhitePawns
                vec![],
                // WhiteKnights
                vec![],
                // WhiteBishops
                vec![Square::H4],
                // WhiteRooks
                vec![Square::A1, Square::H1],
                // WhiteQueens
                vec![],
                // WhiteKing
                vec![Square::E1],
                // BlackPawns
                vec![],
                // BlackKnights
                vec![],
                // BlackBishops
                vec![Square::B7, Square::G7],
                // BlackRooks
                vec![Square::A8, Square::H8],
                // BlackQueens
                vec![Square::H7],
                // BlackKing
                vec![Square::E7]
            ]
        });

        // // pieces
        // assert_eq!(
        //     output.as_ref().unwrap().pieces,
        //     expected.as_ref().unwrap().pieces
        // );
        // // pawns
        // assert_eq!(
        //     output.as_ref().unwrap().pawns,
        //     expected.as_ref().unwrap().pawns
        // );
        // // kings_index
        // assert_eq!(
        //     output.as_ref().unwrap().kings_square,
        //     expected.as_ref().unwrap().kings_square
        // );
        // // piece_count
        // assert_eq!(
        //     output.as_ref().unwrap().piece_count,
        //     expected.as_ref().unwrap().piece_count
        // );
        // // big_piece_count
        // assert_eq!(
        //     output.as_ref().unwrap().big_piece_count,
        //     expected.as_ref().unwrap().big_piece_count
        // );
        // // major_piece_count
        // assert_eq!(
        //     output.as_ref().unwrap().major_piece_count,
        //     expected.as_ref().unwrap().major_piece_count
        // );
        // // minor_piece_count
        // assert_eq!(
        //     output.as_ref().unwrap().minor_piece_count,
        //     expected.as_ref().unwrap().minor_piece_count
        // );
        // // piece list
        // assert_eq!(
        //     output.as_ref().unwrap().piece_list,
        //     expected.as_ref().unwrap().piece_list
        // );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_valid_board_fen_no_captures_no_promotions() {
        let input = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1";
        let output = Board::try_from(input);
        #[rustfmt::skip]
        let expected = Ok(Board {
            pieces: [
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, Some(Piece::WhiteRook), None,                     None,                     None,                    None,                    Some(Piece::WhiteRook),   Some(Piece::WhiteKing),   None,                   None,
                None, None,                   Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   None,                    Some(Piece::WhiteQueen), Some(Piece::WhitePawn),   Some(Piece::WhitePawn),   Some(Piece::WhitePawn), None,
                None, Some(Piece::WhitePawn), None,                     Some(Piece::WhiteKnight), Some(Piece::WhitePawn),  None,                    Some(Piece::WhiteKnight), None,                     None,                   None,
                None, None,                   None,                     Some(Piece::WhiteBishop), None,                    Some(Piece::WhitePawn),  None,                     Some(Piece::BlackBishop), None,                   None,
                None, None,                   None,                     Some(Piece::BlackBishop), None,                    Some(Piece::BlackPawn),  None,                     Some(Piece::WhiteBishop), None,                   None,
                None, Some(Piece::BlackPawn), None,                     Some(Piece::BlackKnight), Some(Piece::BlackPawn),  None,                    Some(Piece::BlackKnight), None,                     None,                   None,
                None, None,                   Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   None,                    Some(Piece::BlackQueen), Some(Piece::BlackPawn),   Some(Piece::BlackPawn),   Some(Piece::BlackPawn), None,
                None, Some(Piece::BlackRook), None,                     None,                     None,                    None,                    Some(Piece::BlackRook),   Some(Piece::BlackKing),   None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
            ],
            pawns: [BitBoard(0b_0001_0000_0000_1001_1110_0110_0000_0000), BitBoard(0b_1110_0110_0000_1001_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000)],
            kings_square: [Some(Square::G1), Some(Square::G8)],
            piece_count: [8, 2, 2, 2, 1, 1, 8, 2, 2, 2, 1, 1],
            big_piece_count: [8, 8],
            major_piece_count: [4, 4],
            minor_piece_count: [4, 4],
            material_score: [54_200, 54_200],
            piece_list: [
                // WhitePawns
                vec![Square::B2, Square::C2, Square::F2, Square::G2, Square::H2, Square::A3, Square::D3, Square::E4],
                // WhiteKnights
                vec![Square::C3, Square::F3],
                // WhiteBishops
                vec![Square::C4, Square::G5],
                // WhiteRooks
                vec![Square::A1, Square::F1],
                // WhiteQueens
                vec![Square::E2],
                // WhiteKing
                vec![Square::G1],
                // BlackPawns
                vec![Square::E5, Square::A6, Square::D6, Square::B7, Square::C7, Square::F7, Square::G7, Square::H7],
                // BlackKnights
                vec![Square::C6, Square::F6],
                // BlackBishops
                vec![Square::G4, Square::C5],
                // BlackRooks
                vec![Square::A8, Square::F8],
                // BlackQueens
                vec![Square::E7],
                // BlackKing
                vec![Square::G8]
            ]
        });
        // // pieces
        // assert_eq!(
        //     output.as_ref().unwrap().pieces,
        //     expected.as_ref().unwrap().pieces
        // );
        // // pawns
        // assert_eq!(
        //     output.as_ref().unwrap().pawns,
        //     expected.as_ref().unwrap().pawns
        // );
        // // kings_index
        // assert_eq!(
        //     output.as_ref().unwrap().kings_square,
        //     expected.as_ref().unwrap().kings_square
        // );
        // // piece_count
        // assert_eq!(
        //     output.as_ref().unwrap().piece_count,
        //     expected.as_ref().unwrap().piece_count
        // );
        // // big_piece_count
        // assert_eq!(
        //     output.as_ref().unwrap().big_piece_count,
        //     expected.as_ref().unwrap().big_piece_count
        // );
        // // major_piece_count
        // assert_eq!(
        //     output.as_ref().unwrap().major_piece_count,
        //     expected.as_ref().unwrap().major_piece_count
        // );
        // // minor_piece_count
        // assert_eq!(
        //     output.as_ref().unwrap().minor_piece_count,
        //     expected.as_ref().unwrap().minor_piece_count
        // );
        // // piece list
        // assert_eq!(
        //     output.as_ref().unwrap().piece_list,
        //     expected.as_ref().unwrap().piece_list
        // );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_too_few_ranks() {
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP";
        let output = Board::try_from(input);
        let expected = Err(BoardBuildError::BoardFenDeserialize(
            BoardFenDeserializeError::WrongNumRanks {
                board_fen: input.to_owned(),
                num_ranks: 7,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_too_many_ranks() {
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP/8/";
        let output = Board::try_from(input);
        let expected = Err(BoardBuildError::BoardFenDeserialize(
            BoardFenDeserializeError::WrongNumRanks {
                board_fen: input.to_owned(),
                num_ranks: 9,
            },
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_empty_ranks() {
        let input = "8/8/rbkqn2p//8/8/PPKPP1PP/8";
        let output = Board::try_from(input);
        let expected = Err(BoardBuildError::BoardFenDeserialize(
            BoardFenDeserializeError::RankFenDeserialize(RankFenDeserializeError::Empty),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_valid_board_fen_untrimmed() {
        // NOTE: Gamestate will be responsible for trimming
        let input = "  rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR ";
        let output = Board::try_from(input);
        let expected = Err(BoardBuildError::BoardFenDeserialize(
            BoardFenDeserializeError::RankFenDeserialize(RankFenDeserializeError::InvalidChar(
                PieceConversionError::FromChar { invalid_char: ' ' },
            )),
        ));
        assert_eq!(output, expected);
    }
}
