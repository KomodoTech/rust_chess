// TODO: when bitboard errors are removed, remove pub keyword
pub mod bitboard;
use crate::{
    error::{BoardFENParseError, RankFENParseError},
    gamestate::NUM_BOARD_SQUARES,
    pieces::Piece,
    squares::{Square, Square64},
    util::{Color, File, Rank},
};
use bitboard::BitBoard;
use std::{
    collections::HashMap,
    fmt::{self, write},
};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

/// There can be a max of 10 pieces (not king obviously) for each type of
/// piece if all 8 pawns were to somehow promote to the same piece
const MAX_NUM_PIECE_TYPE_INSTANCES: usize = 10;

// TODO: redo testing setup so that inner fields can stay private
#[derive(Debug, PartialEq, Eq)]
pub struct Board {
    // TODO: evaluate whether exposing pieces for Zobrist hashing is acceptable
    pub pieces: [Option<Piece>; NUM_BOARD_SQUARES],
    pub pawns: [BitBoard; Color::COUNT],
    pub kings_square: [Option<Square>; Color::COUNT],
    pub piece_count: [u32; Piece::COUNT],
    pub big_piece_count: [u32; Color::COUNT],
    pub major_piece_count: [u32; Color::COUNT],
    pub minor_piece_count: [u32; Color::COUNT],
    pub piece_list: [[Option<Square>; MAX_NUM_PIECE_TYPE_INSTANCES]; Piece::COUNT], // stores position of each piece to avoid searching through all squares
}

/// Returns an empty board
impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<&str> for Board {
    type Error = BoardFENParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::gen_board_from_fen(value)
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: [None; NUM_BOARD_SQUARES],
            pawns: [BitBoard(0), BitBoard(0)],
            kings_square: [None, None],
            piece_count: [0; Piece::COUNT],
            big_piece_count: [0; Color::COUNT],
            major_piece_count: [0; Color::COUNT],
            minor_piece_count: [0; Color::COUNT],
            piece_list: [[None; MAX_NUM_PIECE_TYPE_INSTANCES]; Piece::COUNT],
        }
    }

    /// Combines white and black pawn positions into one BitBoard. Assumes that you never
    /// have a black and a white pawn occupying the same position
    pub fn get_all_pawns(&self) -> BitBoard {
        BitBoard((self.pawns[0]).0 | (self.pawns[1]).0)
    }

    // TODO: Look for bishops trapped behind non-enemy pawns (or behind any 3 pawns)
    // TODO: Check for non-jumpers in impossible positions
    /// Generates a Board instance corresponding to the board section of a FEN string
    fn gen_board_from_fen(value: &str) -> Result<Self, BoardFENParseError> {
        let mut board = Board::new();
        let mut freq_counter: HashMap<char, usize> = HashMap::with_capacity(Piece::COUNT);
        let ranks: Vec<&str> = value.split('/').collect();
        // Check that we have the right number of ranks and for each valid rank update board accordingly
        match ranks.len() {
            Rank::COUNT => {
                // NOTE: FEN is in reverse order compared with our internal board representation
                // with regards to rank (chars within rank are in correct order)
                for (rank, rank_str) in ranks.iter().rev().enumerate() {
                    // does rank validation in separate function that will return Some(Piece)s or Nones in an array if valid
                    // use ? to convert from RankFENParseError to BoardFENParseError automatically and throw Err if not Ok
                    let rank_pieces = Self::gen_rank_from_fen(rank_str)?;
                    // if the rank is valid update the board
                    for (file, &piece) in rank_pieces.iter().enumerate() {
                        // get square from file and rank and use it to update board's pieces array
                        let square = Square::from_file_and_rank(
                            File::try_from(file).expect("file should be in range 0..=7"),
                            Rank::try_from(rank).expect("rank should be in range 0..=7"),
                        );
                        board.pieces[square as usize] = piece;

                        // for each piece in rank we got back do other updates that can be done for any
                        // piece type (piece_count, piece_list, big/major/minor_piece_count)
                        if let Some(p) = piece {
                            // update freq_counter
                            freq_counter
                                .entry(char::from(p))
                                .and_modify(|count| *count += 1)
                                .or_insert(1);

                            // update piece_list
                            let piece_type_index = p as usize; // outer/row (i) index for piece_list
                                                               // inner/column (j) index for piece_list
                            let piece_index = *freq_counter.get(&char::from(p)).expect(
                                "value to be at least 1 since freq_counter was just updated",
                            ) - 1;

                            // check for max amount of piece type leq max allowed for that piece
                            if piece_index >= p.get_max_num_allowed() as usize {
                                return Err(BoardFENParseError::InvalidNumOfPiece(
                                    value.to_string(),
                                    char::from(p),
                                ));
                            }

                            board.piece_list[piece_type_index][piece_index] = Some(square);

                            // update piece counts
                            let color = p.get_color();
                            let is_big = p.is_big();
                            let is_major = p.is_major();
                            let is_minor = p.is_minor();

                            board.piece_count[p as usize] += 1;
                            if is_big {
                                board.big_piece_count[color as usize] += 1;
                            }
                            if is_major {
                                board.major_piece_count[color as usize] += 1;
                            }
                            if is_minor {
                                board.minor_piece_count[color as usize] += 1;
                            }

                            // TODO: check that there aren't any pawns in first or last rank
                            // update fields of board that are dependent on the piece type
                            // (pawns, kings_square)
                            // TODO: check that there aren't more promoted pieces than missing
                            // pawns
                            // TODO: it's impossible to have more than 6 pawns in a single file
                            // TODO: Check minimum number of enemy missing pieces doesn't contradict number of pawns in a single file
                            // TODO: if there are white pawns in A2 and A3 there can't be one in B2... Generalize
                            match p {
                                Piece::WhitePawn => board.pawns[p.get_color() as usize]
                                    .set_bit(Square64::from(square)),
                                Piece::BlackPawn => board.pawns[p.get_color() as usize]
                                    .set_bit(Square64::from(square)),
                                Piece::WhiteKing => {
                                    board.kings_square[p.get_color() as usize] = Some(square)
                                }
                                Piece::BlackKing => {
                                    board.kings_square[p.get_color() as usize] = Some(square)
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
            _ => {
                return Err(BoardFENParseError::WrongNumRanks(
                    value.to_string(),
                    ranks.len(),
                ))
            }
        }
        // TODO: Check that kings are separated by at least 1 square
        // check freq counter for kings
        if freq_counter.get(&'k') != Some(&1) || freq_counter.get(&'K') != Some(&1) {
            return Err(BoardFENParseError::InvalidKingNum(value.to_string()));
        }

        // TODO: Check that non-active player is not in check
        // TODO: Check that active color is checked less than 3 times (if 2 can't be by (pawn + (pawn || bishop || knight) || (bishop + bishop) || (knight + knight)))
        Ok(board)
    }

    /// Takes a &str that corresponds to a portion of a FEN string for a specific Rank (e.g. rnbqkbnr)
    /// and generates a corresponding Option<Piece> array
    fn gen_rank_from_fen(
        fen_rank: &str,
    ) -> Result<[Option<Piece>; File::COUNT], RankFENParseError> {
        if fen_rank.is_empty() {
            return Err(RankFENParseError::Empty);
        }

        let mut rank: [Option<Piece>; File::COUNT] = [None; File::COUNT];
        let mut square_counter: u8 = 0;
        let mut is_last_char_digit: bool = false;
        // NOTE: Rank order is reversed in FEN but not char order within rank
        for char in fen_rank.chars() {
            match char.to_digit(10) {
                Some(digit) => {
                    // check if there two digits in row to catch an invalid string like
                    // "ppp12pp"
                    match is_last_char_digit {
                        true => {
                            return Err(RankFENParseError::TwoConsecutiveDigits(
                                fen_rank.to_string(),
                            ))
                        }
                        false => {
                            is_last_char_digit = true;
                        }
                    }
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
                                    return Err(RankFENParseError::InvalidNumSquares(
                                        fen_rank.to_string(),
                                    ))
                                }
                            }
                            square_counter += d as u8;
                        }
                        _ => {
                            return Err(RankFENParseError::InvalidDigit(
                                fen_rank.to_string(),
                                digit as usize,
                            ))
                        }
                    }
                }
                // Not a digit so we need to check if char represents a valid piece
                None => {
                    // reset is_last_char_digit
                    is_last_char_digit = false;

                    match Piece::try_from(char) {
                        Ok(piece) => {
                            // push Some(piece) onto rank if there is space
                            match square_counter {
                                sq_count if sq_count < File::COUNT as u8 => {
                                    rank[sq_count as usize] = Some(piece)
                                }
                                _ => {
                                    return Err(RankFENParseError::InvalidNumSquares(
                                        fen_rank.to_string(),
                                    ))
                                }
                            }

                            square_counter += 1;
                        }
                        Err(_) => {
                            return Err(RankFENParseError::InvalidChar(fen_rank.to_string(), char))
                        }
                    }
                }
            }
        }
        // check the square_counter is exactly 8
        match square_counter {
            8 => Ok(rank),
            _ => Err(RankFENParseError::InvalidNumSquares(fen_rank.to_string())),
        }
    }

    /// Returns FEN based on board position
    pub fn to_board_fen(&self) -> String {
        todo!()
    }

    /// Returns piece occupying given square or None if square is empty
    pub fn get_piece_at(&self, square: Square) -> Option<Piece> {
        todo!()
    }

    /// Returns square occupied by the king of a given color or None if no king exists
    pub fn get_king_square(&self, color: Color) -> Option<Square> {
        self.kings_square[color as usize]
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
        self.kings_square = [None; Color::COUNT];
        self.big_piece_count = [0; Color::COUNT];
        self.major_piece_count = [0; Color::COUNT];
        self.minor_piece_count = [0; Color::COUNT];
        self.piece_count = [0; Piece::COUNT];
        self.piece_list = [[None; MAX_NUM_PIECE_TYPE_INSTANCES]; Piece::COUNT];
    }
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
                        write!(f, "{}\t", u8::from(rank));
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
    use std::fmt::format;

    use super::*;

    // NOTE: we can consider this board invalid since there are no kings. Might be useful later
    // if we allow editing the board, so we'll keep it for now
    const EMPTY_BOARD_FEN: &str = "8/8/8/8/8/8/8/8";

    #[rustfmt::skip]
    #[test]
    fn test_board_display() {
        let input = DEFAULT_BOARD;
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

    #[test]
    fn test_clear_board() {
        let mut input = DEFAULT_BOARD;
        input.clear_board();
        let expected = EMPTY_BOARD;
        assert_eq!(input, expected);
    }

    #[test]
    fn test_get_all_pawns() {
        let board_str = "5k2/1p2p3/8/1p1p2p1/5P2/3P4/P7/2K5";
        let board = Board::try_from(board_str).unwrap();
        let output = board.get_all_pawns();

        // let mut white_pawns = BitBoard(0);
        // white_pawns.set_bit(Square64::A2);
        // white_pawns.set_bit(Square64::D3);
        // white_pawns.set_bit(Square64::F4);

        // let mut black_pawns = BitBoard(0);
        // black_pawns.set_bit(Square64::B7);
        // black_pawns.set_bit(Square64::E7);
        // black_pawns.set_bit(Square64::B5);
        // black_pawns.set_bit(Square64::D5);
        // black_pawns.set_bit(Square64::G5);

        // println!("white pawns:\n{}\nblack pawns:\n{}", white_pawns, black_pawns);

        let mut expected = BitBoard(0);
        expected.set_bit(Square64::A2);
        expected.set_bit(Square64::D3);
        expected.set_bit(Square64::F4);
        expected.set_bit(Square64::B7);
        expected.set_bit(Square64::E7);
        expected.set_bit(Square64::B5);
        expected.set_bit(Square64::D5);
        expected.set_bit(Square64::G5);

        println!("output:\n{}\nexpected:\n{}", output, expected);

        assert_eq!(output, expected);
    }

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
        piece_list: [
            // WhitePawns
            [None, None, None, None, None, None, None, None, None, None],
            // WhiteKnights
            [None, None, None, None, None, None, None, None, None, None],
            // WhiteBishops
            [None, None, None, None, None, None, None, None, None, None],
            // WhiteRooks
            [None, None, None, None, None, None, None, None, None, None],
            // WhiteQueens
            [None, None, None, None, None, None, None, None, None, None],
            // WhiteKing
            [None, None, None, None, None, None, None, None, None, None],
            // BlackPawns
            [None, None, None, None, None, None, None, None, None, None],
            // BlackKnights
            [None, None, None, None, None, None, None, None, None, None],
            // BlackBishops
            [None, None, None, None, None, None, None, None, None, None],
            // BlackRooks
            [None, None, None, None, None, None, None, None, None, None],
            // BlackQueens
            [None, None, None, None, None, None, None, None, None, None],
            // BlackKing
            [None, None, None, None, None, None, None, None, None, None],
        ]
    };

    // FEN PARSING
    // Rank Level FEN Parsing Tests:
    #[test]
    fn test_get_rank_from_fen_valid_black_back_row_starting_position() {
        let input = "rnbqkbnr";
        let output = Board::gen_rank_from_fen(input);
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
        let output = Board::gen_rank_from_fen(input);
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
    fn test_get_rank_from_fen_valid_empty() {
        let input = "8";
        let output = Board::gen_rank_from_fen(input);
        let expected = Ok([None; 8]);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_empty() {
        let input = "";
        let output = Board::gen_rank_from_fen(input);
        let expected = Err(RankFENParseError::Empty);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_char() {
        let input = "rn2Xb1r";
        let output = Board::gen_rank_from_fen(input);
        let expected = Err(RankFENParseError::InvalidChar(input.to_string(), 'X'));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_digit() {
        let input = "rn0kb1rqN"; // num squares would be valid
        let output = Board::gen_rank_from_fen(input);
        let expected = Err(RankFENParseError::InvalidDigit(input.to_string(), 0));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_too_many_squares() {
        let input = "rn2kb1rqN";
        let output = Board::gen_rank_from_fen(input);
        let expected = Err(RankFENParseError::InvalidNumSquares(input.to_string()));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_too_few_squares() {
        let input = "rn2kb";
        let output = Board::gen_rank_from_fen(input);
        let expected = Err(RankFENParseError::InvalidNumSquares(input.to_string()));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_two_consecutive_digits() {
        let input = "pppp12p"; // adds up to 8 squares but isn't valid
        let ouput = Board::gen_rank_from_fen(input);
        let expected = Err(RankFENParseError::TwoConsecutiveDigits(input.to_string()));
        assert_eq!(ouput, expected);
    }

    #[test]
    fn test_get_rank_from_fen_invalid_two_consecutive_digits_invalid_num_squares() {
        let input = "pppp18p"; // adds up to more than 8 squares but gets caught for consecutive digits
        let ouput = Board::gen_rank_from_fen(input);
        let expected = Err(RankFENParseError::TwoConsecutiveDigits(input.to_string()));
        assert_eq!(ouput, expected);
    }

    // Full Base FEN Board Parsing:
    #[rustfmt::skip]
    const DEFAULT_BOARD: Board = Board {
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

    #[test]
    fn test_board_try_from_valid_board_fen_default() {
        let input = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let output = Board::try_from(input);
        let expected: Result<Board, BoardFENParseError> = Ok(DEFAULT_BOARD);
        // pieces
        assert_eq!(
            output.as_ref().unwrap().pieces,
            expected.as_ref().unwrap().pieces
        );
        // pawns
        assert_eq!(
            output.as_ref().unwrap().pawns,
            expected.as_ref().unwrap().pawns
        );
        // kings_index
        assert_eq!(
            output.as_ref().unwrap().kings_square,
            expected.as_ref().unwrap().kings_square
        );
        // piece_count
        assert_eq!(
            output.as_ref().unwrap().piece_count,
            expected.as_ref().unwrap().piece_count
        );
        // big_piece_count
        assert_eq!(
            output.as_ref().unwrap().big_piece_count,
            expected.as_ref().unwrap().big_piece_count
        );
        // major_piece_count
        assert_eq!(
            output.as_ref().unwrap().major_piece_count,
            expected.as_ref().unwrap().major_piece_count
        );
        // minor_piece_count
        assert_eq!(
            output.as_ref().unwrap().minor_piece_count,
            expected.as_ref().unwrap().minor_piece_count
        );
        // piece list
        assert_eq!(
            output.as_ref().unwrap().piece_list,
            expected.as_ref().unwrap().piece_list
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_valid_board_fen_sliding_and_kings() {
        let input = "r6r/1b2k1bq/8/8/7B/8/8/R3K2R";
        let output = Board::try_from(input);
        // TODO: pull this into utils it will be useful for testing later
        #[rustfmt::skip]
        let expected: Result<Board, BoardFENParseError> = Ok(Board {
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
            piece_list: [
                // WhitePawns
                [None, None, None, None, None, None, None, None, None, None],
                // WhiteKnights
                [None, None, None, None, None, None, None, None, None, None],
                // WhiteBishops
                [Some(Square::H4), None, None, None, None, None, None, None, None, None],
                // WhiteRooks
                [Some(Square::A1), Some(Square::H1), None, None, None, None, None, None, None, None],
                // WhiteQueens
                [None, None, None, None, None, None, None, None, None, None],
                // WhiteKing
                [Some(Square::E1), None, None, None, None, None, None, None, None, None],
                // BlackPawns
                [None, None, None, None, None, None, None, None, None, None],
                // BlackKnights
                [None, None, None, None, None, None, None, None, None, None],
                // BlackBishops
                [Some(Square::B7), Some(Square::G7), None, None, None, None, None, None, None, None],
                // BlackRooks
                [Some(Square::A8), Some(Square::H8), None, None, None, None, None, None, None, None],
                // BlackQueens
                [Some(Square::H7), None, None, None, None, None, None, None, None, None],
                // BlackKing
                [Some(Square::E7), None, None, None, None, None, None, None, None, None]
            ]
        });
        // pieces
        assert_eq!(
            output.as_ref().unwrap().pieces,
            expected.as_ref().unwrap().pieces
        );
        // pawns
        assert_eq!(
            output.as_ref().unwrap().pawns,
            expected.as_ref().unwrap().pawns
        );
        // kings_index
        assert_eq!(
            output.as_ref().unwrap().kings_square,
            expected.as_ref().unwrap().kings_square
        );
        // piece_count
        assert_eq!(
            output.as_ref().unwrap().piece_count,
            expected.as_ref().unwrap().piece_count
        );
        // big_piece_count
        assert_eq!(
            output.as_ref().unwrap().big_piece_count,
            expected.as_ref().unwrap().big_piece_count
        );
        // major_piece_count
        assert_eq!(
            output.as_ref().unwrap().major_piece_count,
            expected.as_ref().unwrap().major_piece_count
        );
        // minor_piece_count
        assert_eq!(
            output.as_ref().unwrap().minor_piece_count,
            expected.as_ref().unwrap().minor_piece_count
        );
        // piece list
        assert_eq!(
            output.as_ref().unwrap().piece_list,
            expected.as_ref().unwrap().piece_list
        );
        // assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_valid_board_fen_no_captures_no_promotions() {
        let input = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1";
        let output = Board::try_from(input);
        // TODO: pull this into utils it will be useful for testing later
        #[rustfmt::skip]
        let expected: Result<Board, BoardFENParseError> = Ok(Board {
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

            piece_list: [
                // WhitePawns
                [Some(Square::B2), Some(Square::C2), Some(Square::F2), Some(Square::G2), Some(Square::H2), Some(Square::A3), Some(Square::D3), Some(Square::E4), None, None],
                // WhiteKnights
                [Some(Square::C3), Some(Square::F3), None, None, None, None, None, None, None, None],
                // WhiteBishops
                [Some(Square::C4), Some(Square::G5), None, None, None, None, None, None, None, None],
                // WhiteRooks
                [Some(Square::A1), Some(Square::F1), None, None, None, None, None, None, None, None],
                // WhiteQueens
                [Some(Square::E2), None, None, None, None, None, None, None, None, None],
                // WhiteKing
                [Some(Square::G1), None, None, None, None, None, None, None, None, None],
                // BlackPawns
                [Some(Square::E5), Some(Square::A6), Some(Square::D6), Some(Square::B7), Some(Square::C7), Some(Square::F7), Some(Square::G7), Some(Square::H7), None, None],
                // BlackKnights
                [Some(Square::C6), Some(Square::F6), None, None, None, None, None, None, None, None],
                // BlackBishops
                [Some(Square::G4), Some(Square::C5), None, None, None, None, None, None, None, None],
                // BlackRooks
                [Some(Square::A8), Some(Square::F8), None, None, None, None, None, None, None, None],
                // BlackQueens
                [Some(Square::E7), None, None, None, None, None, None, None, None, None],
                // BlackKing
                [Some(Square::G8), None, None, None, None, None, None, None, None, None]
            ]
        });
        // pieces
        assert_eq!(
            output.as_ref().unwrap().pieces,
            expected.as_ref().unwrap().pieces
        );
        // pawns
        assert_eq!(
            output.as_ref().unwrap().pawns,
            expected.as_ref().unwrap().pawns
        );
        // kings_index
        assert_eq!(
            output.as_ref().unwrap().kings_square,
            expected.as_ref().unwrap().kings_square
        );
        // piece_count
        assert_eq!(
            output.as_ref().unwrap().piece_count,
            expected.as_ref().unwrap().piece_count
        );
        // big_piece_count
        assert_eq!(
            output.as_ref().unwrap().big_piece_count,
            expected.as_ref().unwrap().big_piece_count
        );
        // major_piece_count
        assert_eq!(
            output.as_ref().unwrap().major_piece_count,
            expected.as_ref().unwrap().major_piece_count
        );
        // minor_piece_count
        assert_eq!(
            output.as_ref().unwrap().minor_piece_count,
            expected.as_ref().unwrap().minor_piece_count
        );
        // piece list
        assert_eq!(
            output.as_ref().unwrap().piece_list,
            expected.as_ref().unwrap().piece_list
        );
        // assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_all_8() {
        let input = "8/8/8/8/8/8/8/8";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::InvalidKingNum(input.to_string()));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_too_few_ranks() {
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::WrongNumRanks(input.to_string(), 7));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_too_many_ranks() {
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP/8/";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::WrongNumRanks(input.to_string(), 9));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_empty_ranks() {
        let input = "8/8/rbkqn2p//8/8/PPKPP1PP/8";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::RankFENParseError(
            RankFENParseError::Empty,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_too_few_kings() {
        let input = "8/8/rbqn3p/8/8/8/PPKPP1PP/8";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::InvalidKingNum(input.to_string()));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_too_many_kings() {
        let input = "8/8/rbqnkkpr/8/8/8/PPKPP1PP/8";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::InvalidNumOfPiece(
            input.to_string(),
            'k',
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_too_many_white_queens() {
        let input = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/QQQQQQQQ";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::InvalidNumOfPiece(
            input.to_string(),
            'Q',
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_board_fen_too_many_white_pawns() {
        let input = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/PPPPPPPP";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::InvalidNumOfPiece(
            input.to_string(),
            'P',
        ));
        assert_eq!(output, expected);
    }
    // Tests that check that Rank FEN Errors are properly converted to BoardFENParseErrors:
    #[test]
    fn test_board_try_from_valid_board_fen_untrimmed() {
        // NOTE: Gamestate will be responsible for trimming
        let input = "  rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR ";
        let output = Board::try_from(input);
        let expected: Result<Board, BoardFENParseError> =
            Err(BoardFENParseError::RankFENParseError(
                RankFENParseError::InvalidChar("RNBQKBNR ".to_string(), ' '),
            ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_empty() {
        let input = "/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::RankFENParseError(
            RankFENParseError::Empty,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_char() {
        let invalid_rank_str = "rn2Xb1r";
        let input = "rn2Xb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::RankFENParseError(
            RankFENParseError::InvalidChar(invalid_rank_str.to_string(), 'X'),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_digit() {
        let invalid_rank_str = "rn0kb1rqN"; // num squares would be valid
        let input = "rn0kb1rqN/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::RankFENParseError(
            RankFENParseError::InvalidDigit(invalid_rank_str.to_string(), 0),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_too_many_squares() {
        let invalid_rank_str = "rn2kb1rqN";
        let input = "rn2kb1rqN/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::RankFENParseError(
            RankFENParseError::InvalidNumSquares(invalid_rank_str.to_string()),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_too_few_squares() {
        let invalid_rank_str = "rn2kb";
        let input = "rn2kb/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let output = Board::try_from(input);
        let expected = Err(BoardFENParseError::RankFENParseError(
            RankFENParseError::InvalidNumSquares(invalid_rank_str.to_string()),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_two_consecutive_digits() {
        let invalid_rank_str = "pppp12p"; // adds up to 8 squares but isn't valid
        let input = "pppp12p/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let ouput = Board::try_from(input);
        let expected = Err(BoardFENParseError::RankFENParseError(
            RankFENParseError::TwoConsecutiveDigits(invalid_rank_str.to_string()),
        ));
        assert_eq!(ouput, expected);
    }

    #[test]
    fn test_board_invalid_two_consecutive_digits_invalid_num_squares() {
        let invalid_rank_str = "pppp18p"; // adds up to more than 8 squares but gets caught for consecutive digits
        let input = "pppp18p/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let ouput = Board::try_from(input);
        let expected = Err(BoardFENParseError::RankFENParseError(
            RankFENParseError::TwoConsecutiveDigits(invalid_rank_str.to_string()),
        ));
        assert_eq!(ouput, expected);
    }
}
