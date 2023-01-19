use crate::{
    error::ChessError as Error,
    pieces::Piece,
    squares::Square,
};
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::bitboard::BB_RANK_1;

const DEFAULT_BASE_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
const NUM_BOARD_SQUARES: usize = 120;

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
enum File {
    FileA,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH
}

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
enum Rank {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug)]
pub struct Board {
    pieces: [u32; NUM_BOARD_SQUARES],
    pawns: [u64; 3],
    kings_index: [u32; 2],
    piece_count: [u32; 12],
    big_piece_count: [u32; 3],
    major_piece_count: [u32; 3], 
    minor_piece_count: [u32; 3], 
    files_board: [u32; NUM_BOARD_SQUARES],
    ranks_board: [u32; NUM_BOARD_SQUARES],
}

impl Board {
    /// Returns empty board
    pub fn new() -> Self {
        let files_and_ranks_boards = Self::init_files_ranks_board();
        todo!()
    }

    fn init_files_ranks_board() -> ([u32; NUM_BOARD_SQUARES], [u32; NUM_BOARD_SQUARES]) {
        let mut index = 0;
        let mut file = File::FileA;
        let mut rank = Rank::Rank1;
        let mut square = Square::A1; 
        let mut square_64 = 0;

        let mut files_board = [Square::OffBoard as u32; NUM_BOARD_SQUARES];
        let mut ranks_board = [Square::OffBoard as u32; NUM_BOARD_SQUARES];

        for rank in Rank::iter() {
            for file in File::iter() {
                square = Square::from_file_and_rank(file as u8, rank as u8).unwrap();
                files_board[square as usize] = file as u32;
                ranks_board[square as usize] = rank as u32;
            }
        }
        (files_board, ranks_board)
    }

    /// Returns board from position FEN. Returns error if FEN is invalid
    pub fn from_base_fen(fen: &str) -> Result<Self, Error> {
        todo!()
    }

    /// Returns FEN based on board position
    pub fn to_base_fen(&self) -> String {
        todo!()
    }

    /// Returns piece occupying given square or None if square is empty
    pub fn get_piece_at(&self, square: Square) -> Option<Piece> {
        todo!()
    }

    /// Returns square occupied by the king of a given color or None if no king exists
    pub fn get_king_square(&self, color: Color) -> Option<Square> {
        unimplemented!()
    }

    /// Clears a given square and returns the piece occupying square or None if square was empty
    pub fn clear_square(&mut self, square: Square) -> Option<Piece> {
        todo!()
    }

    /// Places new piece on given square.
    /// Returns the piece previously occupying square or None if square was empty
    pub fn add_piece(&mut self, square: Square, piece: Piece) -> Option<Piece> {
        todo!()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
        // generate some output_str
        // write!(f, "{}", output_str)
    }
}

/// Returns a board with default initial position
impl Default for Board {
    fn default() -> Self {
        Self::from_base_fen(DEFAULT_BASE_FEN).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_to_string() {
        let board = Board::default();
        let ref_string = "r n b q k b n r\np p p p p p p p\n. . . . . . . .\n. . . . . . . .\n. . . . . . . .\n. . . . . . . .\nP P P P P P P P\nR N B Q K B N R";
        let output_string = board.to_string(); // autoderived from impl Display
        assert_eq!(ref_string, output_string);
    }

    #[test]
    fn test_fen_parsing() {
        let empty_fen = "8/8/8/8/8/8/8/8";
        let board = Board::new();
        assert_eq!(empty_fen, board.to_base_fen());

        let board = Board::default();
        assert_eq!(DEFAULT_BASE_FEN, board.to_base_fen());

        let sicilian_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR";
        let board = Board::from_base_fen(sicilian_fen).unwrap();
        assert_eq!(sicilian_fen, board.to_base_fen());
    }

    #[test]
    fn test_add_piece() {
        let mut board = Board::new();
        let square = Square::from_name("e7").unwrap();
        let pawn: Piece = 'p'.try_into().unwrap(); //black pawn

        board.add_piece(square, pawn);

        let new_fen = "8/4p3/8/8/8/8/8/8";
        assert_eq!(new_fen, board.to_base_fen());
        assert_eq!(board.get_piece_at(square), Some(pawn));
    }
}
