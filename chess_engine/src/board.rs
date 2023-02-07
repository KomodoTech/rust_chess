// TODO: when bitboard errors are removed, remove pub keyword
pub mod bitboard;
use crate::{
    error::ConversionError,
    pieces::Piece,
    squares::{Square, Square64},
    util::{Color, File, Rank, NUM_BOARD_SQUARES},
};
use bitboard::BitBoard;
use rand::seq::index;
use regex::{CaptureMatches, Regex};
use std::fmt;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

/// There can be a max of 10 pieces (not king obviously) for each type of
/// piece if all 8 pawns were to somehow promote to the same piece
const MAX_NUM_PIECE_TYPE_INSTANCES: usize = 10;

#[derive(Debug)]
pub struct Board {
    // TODO: evaluate whether exposing pieces for Zobrist hashing is acceptable
    pub pieces: [Option<Piece>; NUM_BOARD_SQUARES],
    pawns: [BitBoard; Color::COUNT],
    kings_index: [Option<Square>; Color::COUNT],
    piece_count: [u32; Piece::COUNT],
    big_piece_count: [u32; Color::COUNT],
    major_piece_count: [u32; Color::COUNT],
    minor_piece_count: [u32; Color::COUNT],
    piece_list: [[Option<Square>; MAX_NUM_PIECE_TYPE_INSTANCES]; Piece::COUNT], // stores position of each piece to avoid searching through all squares
}

/// Returns an empty board
impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: [None; NUM_BOARD_SQUARES],
            pawns: [BitBoard(0), BitBoard(0)],
            kings_index: [None, None],
            piece_count: [0; Piece::COUNT],
            big_piece_count: [0; Color::COUNT],
            major_piece_count: [0; Color::COUNT],
            minor_piece_count: [0; Color::COUNT],
            piece_list: [[None; MAX_NUM_PIECE_TYPE_INSTANCES]; Piece::COUNT],
        }
    }

    // iterate left to right
    // check each char corresponds to valid piece or empty
    // build frequency counter
    // counter var to track sum adding up to 8
    // check freq counter for kings (optionally for max num of pieces per type)
    //

    // /// Given a substring of a base FEN that represents a rank, count the number of squares
    // /// that are represented
    // fn count_squares_in_FEN_rank(fen_rank: &str) -> usize {
    //     fen_rank.chars().fold(0, |acc, c| {
    //                     let num_empty = c.to_digit(10);
    //                     match num_empty {
    //                          Some(d) => {acc + d as usize},
    //                          None => {acc + 1}
    //                     }
    //                 })
    // }

    // ///
    // fn validate_base_fen(fen: &str) -> Result<Vec<&str>, FENParseError> {
    //     #[rustfmt::skip]
    //     const fen_regex_str: &str =
    //                                         r"(?x)^
    //                                         (?P<Rank1>[pnbrqkPNBRQK1-8]{1,8}/)
    //                                         (?P<Rank2>[pnbrqkPNBRQK1-8]{1,8}/)
    //                                         (?P<Rank3>[pnbrqkPNBRQK1-8]{1,8}/)
    //                                         (?P<Rank4>[pnbrqkPNBRQK1-8]{1,8}/)
    //                                         (?P<Rank5>[pnbrqkPNBRQK1-8]{1,8}/)
    //                                         (?P<Rank6>[pnbrqkPNBRQK1-8]{1,8}/)
    //                                         (?P<Rank7>[pnbrqkPNBRQK1-8]{1,8}/)
    //                                         (?P<Rank8>[pnbrqkPNBRQK1-8]{1,8}$)";

    //     let fen_regex = Regex::new(fen_regex_str).expect("regex string to be valid regex");

    //     let mut ranks: Vec<&str> = Vec::with_capacity(Rank::COUNT);
    //     let mut rank_num: usize = 1;
    //     for caps in fen_regex.captures_iter(fen) {
    //         // make sure you have exactly 8 FEN ranks
    //         match rank_num {
    //             r if r <= Rank::COUNT => {
    //                 // Make sure that each FEN rank represents exactly 8 squares
    //                 let num_squares = Self::count_squares_in_FEN_rank(&caps[r]);
    //                 match num_squares {
    //                     n if n == 8 => {
    //                         // TODO: check that there aren't more than 10 of any piece
    //                         todo!()
    //                     },
    //                     _ => {return Err(FENParseError::GenerateBoardFromBaseFENRankDoesNotContain8SquaresError(caps[r].to_string()))}
    //                 }
    //                 rank_num += 1;
    //             },
    //             _ => {return Err(FENParseError::GenerateBoardFromBaseFENTooManyRanksError(rank_num));}
    //         }
    //     }
    //     todo!()
    // }

    // /// Returns board from position FEN. Returns error if FEN is invalid
    // pub fn from_base_fen(fen: &str) -> Result<Self, FENParseError> {
    //     // each element of rows will contain a &str that represents all the
    //     // pieces (or lack thereof) for that row (e.g. 4P3)
    //     let rows: Vec<&str> = fen.split('/').collect();
    //     match rows.len() {
    //         num_rows if num_rows == 8 => {

    //             let mut board = Board::new();
    //             let mut square_64: Square64 = Square64::A1;

    //             for row in rows {
    //             // TODO: write test passing in graphemes that aren't pure ascii
    //                 for char in row.chars() {
    //                     let piece: Result<Piece, ConversionError> = char.try_into();

    //                     match piece {
    //                         Ok(p) => {
    //                             match p {
    //                                 Piece::WhitePawn => {
    //                                     // update pawn bitboard
    //                                     board.pawns[0].set_bit(square_64);
    //                                     // update piece count
    //                                     board.piece_count[0] += 1;
    //                                     // update pieces
    //                                     board.pieces[square_64 as usize] = Some(p);
    //                                     let next_square_64 = square_64 + 1;
    //                                     match next_square_64 {
    //                                         Ok(s_64) => { square_64 = next_square_64.unwrap() },
    //                                         Err(_) => { return Err(FENParseError::GenerateBoardFromBaseFEN)}
    //                                     }
    //                                 },
    //                                 Piece::WhiteKnight => {todo!()},
    //                                 Piece::WhiteBishop => {todo!()},
    //                                 Piece::WhiteRook => {todo!()},
    //                                 Piece::WhiteQueen => {todo!()},
    //                                 Piece::WhiteKing => {todo!()},
    //                                 Piece::BlackPawn => {todo!()},
    //                                 Piece::BlackKnight => {todo!()},
    //                                 Piece::BlackBishop => {todo!()},
    //                                 Piece::BlackRook => {todo!()},
    //                                 Piece::BlackQueen => {todo!()},
    //                                 Piece::BlackKing => {todo!()}
    //                             }
    //                             todo!()
    //                         },
    //                         Err(_) => {
    //                             match char.to_digit(10) {
    //                                 Some(digit) => {
    //                                     match digit {
    //                                         // if character represents a digit leq than 8, increment index_64
    //                                         // to not put a piece in that square
    //                                         d if d <= 8 => {
    //                                             // NOTE: can't use or implement AddAssign because adding can fail here
    //                                             let next_square_64 = (square_64 + d as usize);
    //                                             match next_square_64 {
    //                                                 Ok(s_64) => { square_64 = next_square_64.unwrap() },
    //                                                 Err(_) => { return Err(FENParseError::GenerateBoardFromBaseFENInvalidSquare64Index(digit))}
    //                                             }

    //                                         },
    //                                         _ => { return Err(FENParseError::GenerateBoardFromBaseFENInvalidDigit(digit));}
    //                                     }
    //                                 },
    //                                 None => { return Err(FENParseError::GenerateBoardFromBaseFENInvalidChar(char)); }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //             Ok(board)
    //         },
    //         _ => {Err(FENParseError::GenerateBoardFromBaseFENNumberOfRowsError(fen.to_string()))}
    //     }
    //     // create index counter
    //     // iterate through array and for each &str aka row parse and increment index counter
    //     // appropriately
    //     // use index counter to convert to Square and add pieces as needed
    // }

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

    /// Clears board
    pub fn clear_board(&mut self) {
        self.pieces = [None; NUM_BOARD_SQUARES];
        self.pawns = [BitBoard(0); Color::COUNT];
        self.big_piece_count = [0; Color::COUNT];
        self.major_piece_count = [0; Color::COUNT];
        self.minor_piece_count = [0; Color::COUNT];
        self.piece_count = [0; Piece::COUNT];
        // TODO: clear piece_list
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
    use super::*;

    const DEFAULT_BASE_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

    // #[test]
    // fn test_base_fen_regex_default() {
    //     let output = Board::is_valid_base_fen(DEFAULT_BASE_FEN);
    //     let expected = true;
    //     assert_eq!(output, expected);
    // }

    // #[test]
    // fn test_base_fen_regex_move_e4() {
    //     let input = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR";
    //     let output = Board::is_valid_base_fen(input);
    //     let expected = true;
    //     assert_eq!(output, expected);
    // }

    // #[test]
    // fn test_base_fen_regex_move_e4_c5() {
    //     let input = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR";
    //     let output = Board::is_valid_base_fen(input);
    //     let expected = true;
    //     assert_eq!(output, expected);
    // }

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
            pawns: [BitBoard(0x00FF000000000000), BitBoard(0x000000000000FF00)],
            kings_index: [Some(Square::E1), Some(Square::E8)],
            piece_count: [8, 2, 2, 2, 1, 1, 8, 2, 2, 2, 1, 1],
            big_piece_count: [8, 8],
            major_piece_count: [3, 3],
            minor_piece_count: [4, 4],
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
