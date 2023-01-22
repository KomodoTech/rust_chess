use crate::{
    error::ChessError as Error,
    pieces::{Color, Piece},
    squares::Square,
};
use std::fmt;

mod bitboard;
use bitboard::BB_RANK_1;

const DEFAULT_BASE_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

#[derive(Debug)]
pub struct Board {
    // add whatever fields you want here
}

impl Board {
    /// Returns empty board
    pub fn new() -> Self {
        todo!()
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
