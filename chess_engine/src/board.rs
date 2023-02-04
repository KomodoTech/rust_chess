// TODO: when bitboard errors are removed, remove pub keyword
pub mod bitboard;
use crate::{
    error::ChessError as Error,
    pieces::Piece,
    squares::{Square, Square64},
    util::{Color, File, Rank},
};
use bitboard::BitBoard;
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const DEFAULT_BASE_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
const NUM_BOARD_SQUARES: usize = 120;

#[derive(Debug)]
pub struct Board {
    // TODO: evaluate whether exposing pieces for Zobrist hashing is acceptable
    pub pieces: [Option<Piece>; NUM_BOARD_SQUARES],
    pawns: [BitBoard; 3],
    kings_index: [Option<Square>; 2],
    piece_count: [u32; 12],
    big_piece_count: [u32; 3],
    major_piece_count: [u32; 3],
    minor_piece_count: [u32; 3],
    // NOTE: there can be a max of 10 pieces (not king obviously) for each type of
    // piece if all 8 pawns were to somehow promote to the same piece
    piece_list: [[Option<Square>; 10]; 12], // stores position of each piece to avoid searching through all squares
}

/// Returns a board with default initial position
impl Default for Board {
    fn default() -> Self {
        Self::from_base_fen(DEFAULT_BASE_FEN).unwrap()
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: [None; NUM_BOARD_SQUARES],
            pawns: [BitBoard(0), BitBoard(0), BitBoard(0)],
            kings_index: [None, None],
            piece_count: [0; 12],
            big_piece_count: [0; 3],
            major_piece_count: [0; 3],
            minor_piece_count: [0; 3],
            piece_list: [[None; 10]; 12],
        }
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
        self.kings_index[color as usize]
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
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                let piece = self.pieces[square as usize];
                match file {
                    File::FileH => match piece {
                        Some(p) => {
                            write!(f, "{}", p);
                        }
                        _ => {
                            write!(f, "_");
                        }
                    },
                    _ => match piece {
                        Some(p) => {
                            write!(f, "{}\t", p);
                        }
                        _ => {
                            write!(f, "_\t");
                        }
                    },
                }
            }
            if rank != Rank::Rank8 {
                writeln!(f);
            }
        }
        writeln!(f)
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use std::process::Output;

    use super::*;

    #[test]
    fn test_board_display() {
        let input = Board {
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
                None, None,                None,                None,                None,                None,                None,                None,                None,               None,
                None, None,                None,                None,                None,                None,                None,                None,                None,               None,
            ],
            pawns: [BitBoard(0x00FF000000000000), BitBoard(0x000000000000FF00), BitBoard(0x00FF00000000FF00)],
            kings_index: [Some(Square::E1), Some(Square::E8)],
            piece_count: [8, 2, 2, 2, 1, 1, 8, 2, 2, 2, 1, 1],
            big_piece_count: [8, 8, 16],
            major_piece_count: [3, 3, 6],
            minor_piece_count: [4, 4, 8],
            piece_list: [
               // WhitePawns
               [Some(Square::A2), Some(Square::B2), Some(Square::C2), Some(Square::D2), Some(Square::E2), Some(Square::F2), Some(Square::G2), Some(Square::H2), None, None],
               // WhiteKnights
               [Some(Square::B1), Some(Square::G1), None, None, None, None, None, None, None, None],
               // WhiteBishops
               [Some(Square::C1), Some(Square::F1), None, None, None, None, None, None, None, None],
               // WhiteRooks
               [Some(Square::A1), Some(Square::H1), None, None, None, None, None, None, None, None],
               // WhiteQueens
               [Some(Square::D1), None, None, None, None, None, None, None, None, None],
               // WhiteKing
               [Some(Square::E1), None, None, None, None, None, None, None, None, None],
               // BlackPawns
               [Some(Square::A7), Some(Square::B7), Some(Square::C7), Some(Square::D7), Some(Square::E7), Some(Square::F7), Some(Square::G7), Some(Square::H7), None, None],
               // BlackKnights
               [Some(Square::B8), Some(Square::G8), None, None, None, None, None, None, None, None],
               // BlackBishops
               [Some(Square::C8), Some(Square::F8), None, None, None, None, None, None, None, None],
               // BlackRooks
               [Some(Square::A8), Some(Square::H8), None, None, None, None, None, None, None, None],
               // BlackQueens
               [Some(Square::D8), None, None, None, None, None, None, None, None, None],
               // BlackKing
               [Some(Square::E8), None, None, None, None, None, None, None, None, None],
            ]
        };

        let output = input.to_string();
        let expected = "♖\t♘\t♗\t♕\t♔\t♗\t♘\t♖\n♙\t♙\t♙\t♙\t♙\t♙\t♙\t♙\n_\t_\t_\t_\t_\t_\t_\t_\n_\t_\t_\t_\t_\t_\t_\t_\n_\t_\t_\t_\t_\t_\t_\t_\n_\t_\t_\t_\t_\t_\t_\t_\n♟\t♟\t♟\t♟\t♟\t♟\t♟\t♟\n♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n".to_string();
        println!("Base Board:\n{}", output);
        assert_eq!(output, expected);
    }

    // #[test]
    // fn test_board_to_string() {
    //     let board = Board::default();
    //     let ref_string = "r n b q k b n r\np p p p p p p p\n. . . . . . . .\n. . . . . . . .\n. . . . . . . .\n. . . . . . . .\nP P P P P P P P\nR N B Q K B N R";
    //     let output_string = board.to_string(); // autoderived from impl Display
    //     assert_eq!(ref_string, output_string);
    // }

    // #[test]
    // fn test_fen_parsing() {
    //     let empty_fen = "8/8/8/8/8/8/8/8";
    //     let board = Board::new();
    //     assert_eq!(empty_fen, board.to_base_fen());

    //     let board = Board::default();
    //     assert_eq!(DEFAULT_BASE_FEN, board.to_base_fen());

    //     let sicilian_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR";
    //     let board = Board::from_base_fen(sicilian_fen).unwrap();
    //     assert_eq!(sicilian_fen, board.to_base_fen());
    // }

    // #[test]
    // fn test_add_piece() {
    //     let mut board = Board::new();
    //     let square = Square::from_name("e7").unwrap();
    //     let pawn: Piece = 'p'.try_into().unwrap(); //black pawn

    //     board.add_piece(square, pawn);

    //     let new_fen = "8/4p3/8/8/8/8/8/8";
    //     assert_eq!(new_fen, board.to_base_fen());
    //     assert_eq!(board.get_piece_at(square), Some(pawn));
    // }
}
