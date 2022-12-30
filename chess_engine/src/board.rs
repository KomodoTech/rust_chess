use crate::{error::ChessError, moves::Move, pieces::Piece, squares::Square};
use std::fmt;

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub struct Board {
    // add whatever fields you want here
}

impl Board {
    pub fn from_fen(fen: &str) -> Board {
        todo!()
    }

    pub fn to_fen(&self) -> String {
        todo!()
    }

    /// Pushes a new move onto the board and updates the state of the board
    /// Returns error if the move is illegal
    pub fn push(&mut self, move_: &Move) -> Result<(), ChessError> {
        unimplemented!()
    }

    /// Updates provided square to now hold a given piece.
    /// If piece is None, sets the square to have no piece.
    /// Returns the piece that was previously on the square (or None if there was no piece)
    fn update_square(&mut self, square: Square, piece: Option<Piece>) -> Option<Piece> {
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

impl Default for Board {
    fn default() -> Self {
        Board::from_fen(DEFAULT_FEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen_parsing() {
        let board: Board = Board::from_fen(DEFAULT_FEN);
        let output_fen = board.to_fen();
        assert_eq!(DEFAULT_FEN, output_fen);

        let sicilian_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
        let board: Board = Board::from_fen(sicilian_fen);
        let output_fen = board.to_fen();
        assert_eq!(sicilian_fen, output_fen)
    }

    #[test]
    fn test_board_to_string() {
        let board: Board = Board::from_fen(DEFAULT_FEN);
        let ref_string = "r n b q k b n r\np p p p p p p p\n. . . . . . . .\n. . . . . . . .\n. . . . . . . .\n. . . . . . . .\nP P P P P P P P\nR N B Q K B N R";
        let output_string = board.to_string(); // autoderived from impl Display
        assert_eq!(ref_string, output_string);
    }

    #[ignore]
    #[test]
    fn test_push_move() {
        let mut board: Board = Board::from_fen(DEFAULT_FEN);
        let e4: Move = Move::from_uci("e2e4");
        board.push(&e4).expect("could not push e4 to new board");

        let ref_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        let output_fen = board.to_string();
        assert_eq!(ref_fen, output_fen);

        let ref_board = "r n b q k b n r\np p p p p p p p\n. . . . . . . .\n. . . . . . . .\n. . . . P . . .\n. . . . . . . .\nP P P P . P P P\nR N B Q K B N R";
        let output_board = board.to_string(); // autoderived from impl Display
        assert_eq!(ref_board, output_board);
    }
}
