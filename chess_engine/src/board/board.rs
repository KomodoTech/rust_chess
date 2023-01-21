use crate::{
    error::ChessError as Error,
    pieces::Piece,
    squares::{Square, Square64},
};
use std::fmt;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::bitboard::BB_RANK_1;


const DEFAULT_BASE_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
const NUM_BOARD_SQUARES: usize = 120;

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq)]
pub enum File {
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
pub enum Rank {
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
    pieces: [Option<Piece>; NUM_BOARD_SQUARES],
    // pawns: [u64; 3],
    // kings_index: [u32; 2],
    // piece_count: [u32; 12],
    // big_piece_count: [u32; 3],
    // major_piece_count: [u32; 3], 
    // minor_piece_count: [u32; 3], 

    // NOTE: there can be a max of 10 pieces (not king obviously) for each type of
    // piece if all 8 pawns were to somehow promote to the same piece
    // piece_list: [[Option<Square>; 13]; 10],   // stores position of each piece to avoid searching through all squares

    square_120_to_64: [Option<Square64>; NUM_BOARD_SQUARES],
    square_64_to_120: [Option<Square>; 64],
    files_board: [Option<File>; NUM_BOARD_SQUARES],
    ranks_board: [Option<Rank>; NUM_BOARD_SQUARES],
}

impl Board {

    // TODO: evaluate performance cost of wrapping everything in Option
    // and later unwrapping
    pub fn new() -> Self {
        // Init square_120_to_64, square_64_to_120, files_board, ranks_board
        let mut square = Square::A1; 
        let mut square_64 = Square64::A1;

        let mut files_board = [None; NUM_BOARD_SQUARES];
        let mut ranks_board = [None; NUM_BOARD_SQUARES];
        let mut square_120_to_64 = [None; NUM_BOARD_SQUARES];
        let mut square_64_to_120 = [None; 64];

        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file as u8, rank as u8).unwrap();
                square_120_to_64[square as usize] = Some(square_64);
                square_64_to_120[square_64 as usize] = Some(square);
                files_board[square as usize] = Some(file);
                ranks_board[square as usize] = Some(rank);

                if (square_64 as u8) < 63 {
                    square_64 = (square_64 as u8 + 1)
                                .try_into()
                                .expect("square_64 should be between in range 0..=63");
                }
            }
        }

        Self {
            pieces: [None; NUM_BOARD_SQUARES],
            square_120_to_64,
            square_64_to_120,
            files_board,
            ranks_board,
        }
    }


    // Doesn't feel optimal structurally, but easiest to do with access to files_board and ranks_board
    // so currently it needs to be in the Board... TODO: figure out if there is a better way/if the code
    // can be restructured so that Square can have this functionality. Also think about performance of doing
    // math versus accessing memory (stack)
    pub fn get_file(&self, square: &Square) -> Result<File, Error> {
        match self.files_board[*square as usize]  {
            Some(file) => Ok(file),
            None => Err(Error::SquareOnInvalidFile(*square)),
        }
    }

    pub fn get_rank(&self, square: &Square) -> Result<Rank, Error> {
        match self.ranks_board[*square as usize]  {
            Some(rank) => Ok(rank),
            None => Err(Error::SquareOnInvalidRank(*square)),
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
    fn test_empty_board_new() {
        let output = Board::new();

        // No Pieces on the board by default
        assert_eq!(output.pieces, [
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
        ]);

        // 120 to 64
        // assert_eq!(output.square_120_to_64, [
        //     64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        //     64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        //     64,  0,  1,  2,  3,  4,  5,  6,  7, 64,
        //     64,  8,  9, 10, 11, 12, 13, 14, 15, 64,
        //     64, 16, 17, 18, 19, 20, 21, 22, 23, 64,
        //     64, 24, 25, 26, 27, 28, 29, 30, 31, 64,
        //     64, 32, 33, 34, 35, 36, 37, 38, 39, 64,
        //     64, 40, 41, 42, 43, 44, 45, 46, 47, 64,
        //     64, 48, 49, 50, 51, 52, 53, 54, 55, 64,
        //     64, 56, 57, 58, 59, 60, 61, 62, 63, 64,
        //     64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        //     64, 64, 64, 64, 64, 64, 64, 64, 64, 64
        // ]);
        assert_eq!(output.square_120_to_64, [
            None, None,                None,                None,                None,                None,                None,                None,                None,               None,
            None, None,                None,                None,                None,                None,                None,                None,                None,               None,
            None, Some(Square64::A1),  Some(Square64::B1),  Some(Square64::C1),  Some(Square64::D1),  Some(Square64::E1),  Some(Square64::F1),  Some(Square64::G1),  Some(Square64::H1), None,
            None, Some(Square64::A2),  Some(Square64::B2),  Some(Square64::C2),  Some(Square64::D2),  Some(Square64::E2),  Some(Square64::F2),  Some(Square64::G2),  Some(Square64::H2), None,
            None, Some(Square64::A3),  Some(Square64::B3),  Some(Square64::C3),  Some(Square64::D3),  Some(Square64::E3),  Some(Square64::F3),  Some(Square64::G3),  Some(Square64::H3), None,
            None, Some(Square64::A4),  Some(Square64::B4),  Some(Square64::C4),  Some(Square64::D4),  Some(Square64::E4),  Some(Square64::F4),  Some(Square64::G4),  Some(Square64::H4), None,
            None, Some(Square64::A5),  Some(Square64::B5),  Some(Square64::C5),  Some(Square64::D5),  Some(Square64::E5),  Some(Square64::F5),  Some(Square64::G5),  Some(Square64::H5), None,
            None, Some(Square64::A6),  Some(Square64::B6),  Some(Square64::C6),  Some(Square64::D6),  Some(Square64::E6),  Some(Square64::F6),  Some(Square64::G6),  Some(Square64::H6), None,
            None, Some(Square64::A7),  Some(Square64::B7),  Some(Square64::C7),  Some(Square64::D7),  Some(Square64::E7),  Some(Square64::F7),  Some(Square64::G7),  Some(Square64::H7), None,
            None, Some(Square64::A8),  Some(Square64::B8),  Some(Square64::C8),  Some(Square64::D8),  Some(Square64::E8),  Some(Square64::F8),  Some(Square64::G8),  Some(Square64::H8), None,
            None, None,                None,                None,                None,                None,                None,                None,                None,               None,
            None, None,                None,                None,                None,                None,                None,                None,                None,               None,
        ]);

        // 64 to 120
        // assert_eq!(output.square_64_to_120, [
        //     21, 22, 23, 24, 25, 26, 27, 28,
        //     31, 32, 33, 34, 35, 36, 37, 38,
        //     41, 42, 43, 44, 45, 46, 47, 48,
        //     51, 52, 53, 54, 55, 56, 57, 58,
        //     61, 62, 63, 64, 65, 66, 67, 68,
        //     71, 72, 73, 74, 75, 76, 77, 78,
        //     81, 82, 83, 84, 85, 86, 87, 88,
        //     91, 92, 93, 94, 95, 96, 97, 98
        // ]);
        // assert_eq!(output.square_64_to_120, [
        //     Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
        //     Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
        //     Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
        //     Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
        //     Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
        //     Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
        //     Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
        //     Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8
        // ]);
        assert_eq!(output.square_64_to_120, [
            Some(Square::A1), Some(Square::B1), Some(Square::C1), Some(Square::D1), Some(Square::E1), Some(Square::F1), Some(Square::G1), Some(Square::H1),
            Some(Square::A2), Some(Square::B2), Some(Square::C2), Some(Square::D2), Some(Square::E2), Some(Square::F2), Some(Square::G2), Some(Square::H2),
            Some(Square::A3), Some(Square::B3), Some(Square::C3), Some(Square::D3), Some(Square::E3), Some(Square::F3), Some(Square::G3), Some(Square::H3),
            Some(Square::A4), Some(Square::B4), Some(Square::C4), Some(Square::D4), Some(Square::E4), Some(Square::F4), Some(Square::G4), Some(Square::H4),
            Some(Square::A5), Some(Square::B5), Some(Square::C5), Some(Square::D5), Some(Square::E5), Some(Square::F5), Some(Square::G5), Some(Square::H5),
            Some(Square::A6), Some(Square::B6), Some(Square::C6), Some(Square::D6), Some(Square::E6), Some(Square::F6), Some(Square::G6), Some(Square::H6),
            Some(Square::A7), Some(Square::B7), Some(Square::C7), Some(Square::D7), Some(Square::E7), Some(Square::F7), Some(Square::G7), Some(Square::H7),
            Some(Square::A8), Some(Square::B8), Some(Square::C8), Some(Square::D8), Some(Square::E8), Some(Square::F8), Some(Square::G8), Some(Square::H8)
        ]);

        // Files
        // assert_eq!(output.files_board, [
        //    99, 99, 99, 99, 99, 99, 99, 99, 99, 99,  
        //    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 
        //    99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
        //    99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
        //    99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
        //    99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
        //    99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
        //    99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
        //    99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
        //    99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
        //    99, 99, 99, 99, 99, 99, 99, 99, 99, 99,  
        //    99, 99, 99, 99, 99, 99, 99, 99, 99, 99 
        // ]);
        assert_eq!(output.files_board, [
           None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
           None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
           None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
           None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
           None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
           None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
           None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
           None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
           None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
           None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
           None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
           None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
        ]);
        // Ranks
        // assert_eq!(output.ranks_board, [
        //    99, 99, 99, 99, 99, 99, 99, 99, 99, 99,  
        //    99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 
        //    99,  0,  0,  0,  0,  0,  0,  0,  0, 99,  
        //    99,  1,  1,  1,  1,  1,  1,  1,  1, 99,  
        //    99,  2,  2,  2,  2,  2,  2,  2,  2, 99,  
        //    99,  3,  3,  3,  3,  3,  3,  3,  3, 99,  
        //    99,  4,  4,  4,  4,  4,  4,  4,  4, 99,  
        //    99,  5,  5,  5,  5,  5,  5,  5,  5, 99,  
        //    99,  6,  6,  6,  6,  6,  6,  6,  6, 99,  
        //    99,  7,  7,  7,  7,  7,  7,  7,  7, 99,  
        //    99, 99, 99, 99, 99, 99, 99, 99, 99, 99,  
        //    99, 99, 99, 99, 99, 99, 99, 99, 99, 99 
        // ]);
        assert_eq!(output.ranks_board, [
           None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
           None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
           None,  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1), None,
           None,  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2), None,
           None,  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3), None,
           None,  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4), None,
           None,  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5), None,
           None,  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6), None,
           None,  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7), None,
           None,  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8), None,
           None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
           None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
        ]);
    }

    #[test]
    fn test_get_file() {
        let board = Board::new();
        let input = Square::H7;
        let output = board.get_file(&input).unwrap();
        let expected = File::FileH;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank() {
        let board = Board::new();
        let input = Square::H7;
        let output = board.get_rank(&input).unwrap();
        let expected = Rank::Rank7;
        assert_eq!(output, expected);
    }

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
