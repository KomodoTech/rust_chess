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
pub struct UtilBoards {
    square_120_to_64: [u32; NUM_BOARD_SQUARES],
    square_64_to_120: [u32; 64],
    files_board: [u32; NUM_BOARD_SQUARES],
    ranks_board: [u32; NUM_BOARD_SQUARES],
}

impl UtilBoards {
    fn new() -> Self {
        let mut square = Square::A1; 
        let mut square_64 = 0;

        let mut files_board = [Square::OffBoard as u32; NUM_BOARD_SQUARES];
        let mut ranks_board = [Square::OffBoard as u32; NUM_BOARD_SQUARES];
        let mut square_120_to_64 = [64; NUM_BOARD_SQUARES];
        let mut square_64_to_120 = [120; 64];

        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file as u8, rank as u8).unwrap();
                square_120_to_64[square as usize] = square_64 as u32;
                square_64_to_120[square_64] = square as u32;
                files_board[square as usize] = file as u32;
                ranks_board[square as usize] = rank as u32;
                square_64 += 1;
            }
        }

        UtilBoards { square_120_to_64, square_64_to_120, files_board, ranks_board }
    }
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
    square_120_to_64: [u32; NUM_BOARD_SQUARES],
    square_64_to_120: [u32; 64],
    files_board: [u32; NUM_BOARD_SQUARES],
    ranks_board: [u32; NUM_BOARD_SQUARES],
}

impl Board {
    pub fn new() -> Self {
        let util_boards = UtilBoards::new();
        todo!()
    }


    // Doesn't feel optimal structurally, but easiest to do with access to files_board and ranks_board
    // so currently it needs to be in the Board... TODO: figure out if there is a better way/if the code
    // can be restructured so that Square can have this functionality. Also think about performance of doing
    // math versus accessing memory (stack)
    pub fn get_file(square: Square) -> u8 {
        todo!()
    }

    pub fn get_rank(square: Square) -> u8 {
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
    fn test_init_util_boards() {
        let output = UtilBoards::new();

        // 120 to 64
        assert_eq!(output.square_120_to_64, [
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64,  0,  1,  2,  3,  4,  5,  6,  7, 64,
            64,  8,  9, 10, 11, 12, 13, 14, 15, 64,
            64, 16, 17, 18, 19, 20, 21, 22, 23, 64,
            64, 24, 25, 26, 27, 28, 29, 30, 31, 64,
            64, 32, 33, 34, 35, 36, 37, 38, 39, 64,
            64, 40, 41, 42, 43, 44, 45, 46, 47, 64,
            64, 48, 49, 50, 51, 52, 53, 54, 55, 64,
            64, 56, 57, 58, 59, 60, 61, 62, 63, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
        ]);

        // 64 to 120
        assert_eq!(output.square_64_to_120, [
            21, 22, 23, 24, 25, 26, 27, 28,
            31, 32, 33, 34, 35, 36, 37, 38,
            41, 42, 43, 44, 45, 46, 47, 48,
            51, 52, 53, 54, 55, 56, 57, 58,
            61, 62, 63, 64, 65, 66, 67, 68,
            71, 72, 73, 74, 75, 76, 77, 78,
            81, 82, 83, 84, 85, 86, 87, 88,
            91, 92, 93, 94, 95, 96, 97, 98,
        ]);

        // Files
        assert_eq!(output.files_board, [
           99, 99, 99, 99, 99, 99, 99, 99, 99, 99,  
           99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 
           99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
           99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
           99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
           99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
           99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
           99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
           99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
           99,  0,  1,  2,  3,  4,  5,  6,  7, 99,  
           99, 99, 99, 99, 99, 99, 99, 99, 99, 99,  
           99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 
        ]);
        // Ranks
        assert_eq!(output.ranks_board, [
           99, 99, 99, 99, 99, 99, 99, 99, 99, 99,  
           99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 
           99,  0,  0,  0,  0,  0,  0,  0,  0, 99,  
           99,  1,  1,  1,  1,  1,  1,  1,  1, 99,  
           99,  2,  2,  2,  2,  2,  2,  2,  2, 99,  
           99,  3,  3,  3,  3,  3,  3,  3,  3, 99,  
           99,  4,  4,  4,  4,  4,  4,  4,  4, 99,  
           99,  5,  5,  5,  5,  5,  5,  5,  5, 99,  
           99,  6,  6,  6,  6,  6,  6,  6,  6, 99,  
           99,  7,  7,  7,  7,  7,  7,  7,  7, 99,  
           99, 99, 99, 99, 99, 99, 99, 99, 99, 99,  
           99, 99, 99, 99, 99, 99, 99, 99, 99, 99, 
        ]);
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
