// TODO: when bitboard errors are removed, remove pub keyword
pub mod bitboard;
use crate::{
    error::{ConversionError, FENParseError},
    gamestate::NUM_BOARD_SQUARES,
    pieces::Piece,
    squares::{Square, Square64},
    util::{Color, File, Rank},
};
use bitboard::BitBoard;
use rand::seq::index;
use regex::{CaptureMatches, Regex};
use std::{collections::HashMap, fmt};
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

impl TryFrom<&str> for Board {
    type Error = FENParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut board = Board::new();
        let mut freq_counter: HashMap<char, usize> = HashMap::with_capacity(Piece::COUNT);

        let ranks: Vec<&str> = value.split('/').collect();
        // Check that we have the right number of ranks and for each valid rank update board accordingly
        match ranks.len() {
            Rank::COUNT => {
                for (rank, rank_str) in ranks.iter().enumerate() {
                    // do rank validation in separate function that will return Some(Piece)s or Nones in an array if valid
                    let rank_pieces: Result<[Option<Piece>; File::COUNT], FENParseError> =
                        Self::gen_rank_from_fen(rank_str, &mut freq_counter);
                    match rank_pieces {
                        Ok(rp) => {
                            // if the rank is valid update the board
                            for (file, &piece) in rp.iter().enumerate() {
                                // get square from file and rank and use it to update board's pieces array
                                let square = Square::from_file_and_rank(
                                    File::try_from(file).expect("file should be in range 0..=7"),
                                    Rank::try_from(rank).expect("rank should be in range 0..=7"),
                                );
                                board.pieces[square as usize] = piece;

                                // for each piece in rank we got back do other updates that can be done for any
                                // piece type (piece_count, piece_list, big/major/minor_piece_count)
                                if let Some(p) = piece {
                                    let color = p.get_color();
                                    let is_big = p.is_big();
                                    let is_major = p.is_major();
                                    let is_minor = p.is_minor();

                                    // update board piece_list
                                    let piece_type_index = p as usize; // outer (i) index for piece_list
                                                                       // go to freq_counter and look for piece char as key. our rank validation
                                                                       // has updated freq_counter so it will at least be 1. We subtract 1 for 0-indexing
                                    let piece_index: usize = *freq_counter.get(&(p.into()))
                                                                                .expect("white pawn should already be in freq_counter from rank level parsing") - 1; // inner (j) index for piece_list
                                    board.piece_list[piece_type_index][piece_index] = Some(square);

                                    // update piece counts
                                    board.piece_count[color as usize] += 1;
                                    if is_big {
                                        board.big_piece_count[color as usize] += 1;
                                    }
                                    if is_major {
                                        board.major_piece_count[color as usize] += 1;
                                    }
                                    if is_minor {
                                        board.minor_piece_count[color as usize] += 1;
                                    }

                                    // update fields of board that are dependent on the piece type
                                    // (pawns, kings_index)
                                    match p {
                                        Piece::WhitePawn => board.pawns[p.get_color() as usize]
                                            .set_bit(Square64::from(square)),
                                        Piece::BlackPawn => board.pawns[p.get_color() as usize]
                                            .set_bit(Square64::from(square)),
                                        Piece::WhiteKing => {
                                            board.kings_index[p.get_color() as usize] = Some(square)
                                        }
                                        Piece::BlackKing => {
                                            board.kings_index[p.get_color() as usize] = Some(square)
                                        }
                                        _ => (),
                                    }
                                };
                            }
                        }
                        // rank validation failed. pass along error
                        Err(e) => return Err(e),
                    }
                }
            }
            _ => {
                return Err(FENParseError::BaseFENWrongNumRanks(
                    value.to_string(),
                    ranks.len(),
                ))
            }
        }
        // TODO:
        // check freq counter for kings (optionally for max num of pieces per type)
        if freq_counter.get(&'k') != Some(&1) || freq_counter.get(&'K') != Some(&1) {
            return Err(FENParseError::InvalidKingNum(value.to_string()));
        }
        // check for max amount of piece type leq 9-10 (can disable later)
        for (&key, &val) in freq_counter.iter() {
            let piece = Piece::try_from(key).expect("key should always represent a valid piece");
            if piece.get_max_num_allowed() as usize > val {
                return Err(FENParseError::InvalidNumOfPiece(
                    value.to_string(),
                    piece.to_string(),
                ));
            }
        }
        Ok(board)
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

    // TODO: TEST!
    fn gen_rank_from_fen(
        fen_rank: &str,
        freq_counter: &mut HashMap<char, usize>,
    ) -> Result<[Option<Piece>; File::COUNT], FENParseError> {
        let mut rank: [Option<Piece>; File::COUNT] = [None; File::COUNT];
        let mut square_counter: u8 = 0;
        for char in fen_rank.chars() {
            match char.to_digit(10) {
                Some(digit) => {
                    // if the char is a digit in the range (1..=8) we need to check
                    // that it's not pushing us past our 8 square limit
                    match digit {
                        d if (1..=File::COUNT).contains(&(digit as usize)) => {
                            // push Nones if there is space in rank array
                            match (square_counter + (d as u8) - 1 < File::COUNT as u8) {
                                true => {
                                    for i in square_counter..(square_counter + (d as u8)) {
                                        rank[i as usize] = None;
                                    }
                                }
                                false => {
                                    return Err(FENParseError::RankInvalidNumSquares(
                                        fen_rank.to_string(),
                                    ))
                                }
                            }
                            square_counter += d as u8;
                        }
                        _ => {
                            return Err(FENParseError::RankInvalidDigit(
                                fen_rank.to_string(),
                                digit as usize,
                            ))
                        }
                    }
                }
                // Not a digit so we need to check if char represents a valid piece
                None => {
                    match Piece::try_from(char) {
                        Ok(piece) => {
                            // push Some(piece) onto rank if space
                            match square_counter {
                                sq_count if sq_count < File::COUNT as u8 => {
                                    rank[sq_count as usize] = Some(piece)
                                }
                                _ => {
                                    return Err(FENParseError::RankInvalidNumSquares(
                                        fen_rank.to_string(),
                                    ))
                                }
                            }

                            square_counter += 1;
                            // update freq_counter
                            freq_counter
                                .entry(char)
                                .and_modify(|count| *count += 1)
                                .or_insert(1);
                        }
                        Err(_) => {
                            return Err(FENParseError::RankInvalidChar(fen_rank.to_string(), char))
                        }
                    }
                }
            }
        }
        Ok(rank)
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
