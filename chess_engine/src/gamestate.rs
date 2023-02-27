use std::{
    default,
    fmt::{self, write},
    num::ParseIntError,
};
use strum::EnumCount;
use strum_macros::{Display as EnumDisplay, EnumCount as EnumCountMacro};

use crate::{
    board::{Board, NUM_BOARD_SQUARES},
    castle_perm::{self, CastlePerm, NUM_CASTLE_PERM},
    color::Color,
    error::{
        BoardFENParseError, CastlePermConversionError, EnPassantFENParseError,
        FullmoveCounterFENParseError, GamestateFENParseError, HalfmoveClockFENParseError,
        RankFENParseError, SquareConversionError,
    },
    piece::{self, Piece, PieceType},
    rank::Rank,
    square::{Square, Square64},
    zobrist::Zobrist,
};

// CONSTANTS:
/// Maximum number of full moves we expect
pub const MAX_GAME_MOVES: usize = 1024;
/// When we reach 50 moves (aka 100 half moves) without a pawn advance or a piece capture the game ends
/// immediately in a tie
pub const HALF_MOVE_MAX: usize = 100;
pub const NUM_FEN_SECTIONS: usize = 6;
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Undo {
    move_: u32,
    castle_permissions: CastlePerm,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    position_key: u64,
}

// TODO: consider making the Gamestate with the builder pattern
// TODO: make Zobrist generate at compile time with proc macro
#[derive(Debug, PartialEq, Eq)]
pub struct Gamestate {
    board: Board,
    active_color: Color,
    castle_permissions: CastlePerm,
    en_passant: Option<Square>,
    halfmove_clock: u32, // number of moves both players have made since last pawn advance of piece capture
    fullmove_number: u32, // number of completed turns in the game (incremented when black moves)
    history: Vec<Undo>,
    zobrist: Zobrist,
}

impl Default for Gamestate {
    fn default() -> Self {
        Gamestate::try_from(DEFAULT_FEN)
            .expect("Default Gamestate failed to initialize with Default FEN")
    }
}

/// Generates a new Gamestate from a FEN &str. Base FEN gets converted to board via TryFrom<&str>
/// Color and En Passant square must be lower case.
impl TryFrom<&str> for Gamestate {
    type Error = GamestateFENParseError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::gen_gamestate_from_fen(value)
    }
}

impl fmt::Display for Gamestate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.board);
        writeln!(f, "Active Color: {}", self.active_color);
        write!(f, "En Passant: ");
        match self.en_passant {
            Some(ep) => {
                writeln!(f, "{}", ep);
            }
            None => {
                writeln!(f, "None");
            }
        }
        writeln!(f, "Castle Permissions: {}", self.castle_permissions);
        writeln!(f, "Position Key: {}", self.gen_position_key())
    }
}

impl Gamestate {
    // TODO: determine if new should be this zeroed out version of the board
    // or if it should just be the default board
    pub fn new() -> Self {
        let board = Board::new();
        let active_color = Color::White;
        let castle_permissions = CastlePerm::default();
        let en_passant: Option<Square> = None;
        let halfmove_clock: u32 = 0;
        let fullmove_number: u32 = 0;
        let history = Vec::new();
        let zobrist = Zobrist::new();

        Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        }
    }

    /// Determine if the provided square is currently under attack
    fn is_square_attacked(&self, square: Square) -> bool {
        // depending on active_color determine which pieces to check
        let mut pieces_to_check: [Piece; 6];
        match self.active_color {
            Color::White => {
                pieces_to_check = [
                    Piece::WhitePawn,
                    Piece::WhiteKnight,
                    Piece::WhiteBishop,
                    Piece::WhiteRook,
                    Piece::WhiteQueen,
                    Piece::WhiteKing,
                ]
            }
            Color::Black => {
                pieces_to_check = [
                    Piece::BlackPawn,
                    Piece::BlackKnight,
                    Piece::BlackBishop,
                    Piece::BlackRook,
                    Piece::BlackQueen,
                    Piece::BlackKing,
                ]
            }
        }
        // Going through each type of piece that could be attacking the given square
        // check each square an attacker could be occupying and see if there is in fact
        // the corresponding piece on that attacking square
        for piece in pieces_to_check {
            let directions = piece.get_attack_directions();
            match piece {
                // To check sliding pieces we need to check squares offset by multiples
                // of the direction offset, and early out of a direction when we hit a blocking piece
                // and early out entirely if we find an attacking piece
                sliding if piece.is_sliding() => {
                    // Optimization: bishops can never attack a square that is a different color than they are
                    if (sliding.is_bishop() && (square.get_color() != sliding.get_color())) {
                        continue;
                    }

                    for direction in directions {
                        let mut offset = direction;
                        while let Ok(next_square) = square + offset {
                            match self.board.pieces[next_square as usize] {
                                Some(p) => match p {
                                    attacker if p == piece => return true,
                                    blocker => break,
                                },
                                None => offset += direction,
                            }
                        }
                    }
                }
                // Non-sliding pieces only need to check the squares offset by each direction (no need
                // to check multiples of offset or blocking pieces)
                non_sliding => {
                    for direction in directions {
                        // check if moving in direction places you on a valid square
                        if let Ok(valid_square) = square + direction {
                            // check if the type of piece that could attack our square from the current evaluated square
                            // is present or not.
                            if let Some(p) = self.board.pieces[valid_square as usize] {
                                match p {
                                    attacker if p == piece => return true,
                                    _ => (),
                                }
                            }
                        }
                    }
                }
            }
        }
        // if we never early returned true, then our square is not under attack
        false
    }

    // TODO: make sure that on the frontend the number of characters that can be passed is limited to something reasonable
    // TODO: check that bishops are on squares that have the same color as them
    /// Validate full FEN string and generate valid Gamestate object if validation succeeds
    fn gen_gamestate_from_fen(fen: &str) -> Result<Self, GamestateFENParseError> {
        let fen_sections: Vec<&str> = fen.trim().split(' ').collect();
        let mut fen_sections_iterator = fen_sections.iter();

        // deal with board FEN separately because spaces in the middle will break parse and complicate things
        let board_str = *fen_sections_iterator
            .next()
            .ok_or(GamestateFENParseError::Empty)?;
        let board = Board::try_from(board_str)?;

        // go through rest of FEN sections and clean up any empty sections aka extra spaces
        let mut remaining_sections: Vec<&str> = vec![]; // will not include any empty sections
        for &section in fen_sections_iterator {
            // TODO: get rid of is_empty and use patten matching if let [first, ..]
            // Check if current fen section is empty (if so just a space that we can ignore)
            if !section.is_empty() {
                remaining_sections.push(section);
            }
        }

        match remaining_sections.len() {
            len if len == NUM_FEN_SECTIONS - 1 => {
                let active_color_str = remaining_sections[0];
                // active_color_str here should be either "w" or "b"
                let active_color = match active_color_str {
                    white if white == char::from(Color::White).to_string() => Color::White,
                    black if black == char::from(Color::Black).to_string() => Color::Black,
                    _ => {
                        return Err(GamestateFENParseError::ActiveColor(
                            active_color_str.to_string(),
                        ))
                    }
                };

                // Check that castling permissions don't contradict position of rooks and kings
                // TODO: look into X-FEN and Shredder-FEN for Chess960
                let castle_permissions_str = remaining_sections[1];
                let castle_permissions = match CastlePerm::try_from(castle_permissions_str) {
                    Ok(cp) => cp,
                    Err(_) => {
                        return Err(GamestateFENParseError::CastlePerm(
                            castle_permissions_str.to_string(),
                        ))
                    }
                };

                let en_passant_str = remaining_sections[2];
                let en_passant = match Square::try_from(en_passant_str.to_uppercase().as_str()) {
                    // en_passant_str must be lowercase
                    Ok(ep) => match en_passant_str {
                        uppercase if uppercase == en_passant_str.to_uppercase() => {
                            return Err(GamestateFENParseError::EnPassantFENParseError(
                                EnPassantFENParseError::EnPassantUppercase,
                            ))
                        }
                        _ => {
                            // If active color is black then en_passant rank has to be 3.
                            let ep_rank = ep.get_rank();
                            match ep_rank {
                                Rank::Rank3 => match active_color {
                                    Color::Black => {
                                        // check that the en passant square and the one behind it are empty
                                        let ep_empty = board.pieces[ep as usize].is_none();
                                        let square_behind =
                                            Square::from_file_and_rank(ep.get_file(), Rank::Rank2);
                                        let square_behind_empty =
                                            board.pieces[square_behind as usize].is_none();
                                        let square_ahead =
                                            Square::from_file_and_rank(ep.get_file(), Rank::Rank4);
                                        match (ep_empty & square_behind_empty) {
                                            // check that white pawn is in front of en passant square
                                            true => match board.pawns[0].check_bit(Square64::from(square_ahead)) {
                                                true => Some(ep),
                                                false => return Err(GamestateFENParseError::EnPassantFENParseError(EnPassantFENParseError::CorrectPawnNotInFront(Color::White, ep)))
                                            },
                                            false => return Err(GamestateFENParseError::EnPassantFENParseError(EnPassantFENParseError::NonEmptySquares)),
                                        }
                                    }
                                    Color::White => {
                                        return Err(GamestateFENParseError::EnPassantFENParseError(
                                            EnPassantFENParseError::ColorRankMismatch(
                                                active_color,
                                                ep_rank,
                                            ),
                                        ))
                                    }
                                },
                                // If active color is white then en_passant rank has to be 6.
                                Rank::Rank6 => match active_color {
                                    Color::White => {
                                        // check that the en passant square and the one behind it are empty
                                        let ep_empty = board.pieces[ep as usize].is_none();
                                        let square_behind =
                                            Square::from_file_and_rank(ep.get_file(), Rank::Rank7);
                                        let square_behind_empty =
                                            board.pieces[square_behind as usize].is_none();
                                        let square_ahead =
                                            Square::from_file_and_rank(ep.get_file(), Rank::Rank5);
                                        match (ep_empty & square_behind_empty) {
                                            // check that black pawn is in front of en passant square
                                            true => match board.pawns[1].check_bit(Square64::from(square_ahead)) {
                                                true => Some(ep),
                                                false => return Err(GamestateFENParseError::EnPassantFENParseError(EnPassantFENParseError::CorrectPawnNotInFront(Color::Black, ep)))
                                            },
                                            false => return Err(GamestateFENParseError::EnPassantFENParseError(EnPassantFENParseError::NonEmptySquares)),
                                        }
                                    }
                                    Color::Black => {
                                        return Err(GamestateFENParseError::EnPassantFENParseError(
                                            EnPassantFENParseError::ColorRankMismatch(
                                                active_color,
                                                ep_rank,
                                            ),
                                        ))
                                    }
                                },
                                _ => {
                                    return Err(GamestateFENParseError::EnPassantFENParseError(
                                        EnPassantFENParseError::Rank(ep_rank),
                                    ))
                                }
                            }
                        }
                    },
                    Err(_) => match en_passant_str {
                        "-" => None,
                        _ => {
                            return Err(GamestateFENParseError::EnPassantFENParseError(
                                EnPassantFENParseError::SquareConversionError(
                                    SquareConversionError::FromStr(
                                        strum::ParseError::VariantNotFound,
                                    ),
                                ),
                            ))
                        }
                    },
                };

                let halfmove_clock_str = remaining_sections[3];
                let halfmove_clock = match halfmove_clock_str.parse::<u32>() {
                    Ok(num) => match num {
                        // if this num was 100 the game would immediately tie, so this is considered invalid
                        n if (0..HALF_MOVE_MAX).contains(&(n as usize)) => {
                            // if there is an en passant square, the half move clock must equal 0 (pawn must have moved for en passant to be active)
                            // TODO: get rid of is_some
                            match (n != 0) && en_passant.is_some() {
                                true => {
                                    return Err(GamestateFENParseError::HalfmoveClockFENParseError(
                                        HalfmoveClockFENParseError::NonZeroWhileEnPassant,
                                    ))
                                }
                                false => n,
                            }
                        }
                        _ => {
                            return Err(GamestateFENParseError::HalfmoveClockFENParseError(
                                HalfmoveClockFENParseError::ExceedsMax(num),
                            ))
                        }
                    },
                    Err(e) => {
                        return Err(GamestateFENParseError::HalfmoveClockFENParseError(
                            HalfmoveClockFENParseError::ParseIntError(e),
                        ))
                    }
                };

                let fullmove_number_str = remaining_sections[4];
                // check that fullmove_number is a valid u32
                let fullmove_number = match fullmove_number_str.parse::<u32>() {
                    Ok(num) => match num {
                        // check that fullmove number is less than MAX_GAME_MOVES
                        n if (1..=MAX_GAME_MOVES).contains(&(n as usize)) => {
                            // Check that halfmove and fullmove aren't mutually exclusive
                            let halfmove_clock = halfmove_clock_str.parse::<u32>().expect(
                                "halfmove_clock_str should be a valid u32 since we check it above",
                            );
                            let offset: u32 = match active_color {
                                Color::White => 0,
                                Color::Black => 1,
                            };
                            match n {
                                plausible if ((2*(plausible - 1) + offset) >= halfmove_clock) => plausible,
                                _ => return Err(GamestateFENParseError::FullmoveCounterFENParseError(FullmoveCounterFENParseError::SmallerThanHalfmoveClockDividedByTwo(n, halfmove_clock)))
                            }
                        }
                        _ => {
                            return Err(GamestateFENParseError::FullmoveCounterFENParseError(
                                FullmoveCounterFENParseError::NotInRange(num),
                            ))
                        }
                    },
                    Err(e) => {
                        return Err(GamestateFENParseError::FullmoveCounterFENParseError(
                            FullmoveCounterFENParseError::ParseIntError(e),
                        ))
                    }
                };

                let history = Vec::new();
                let zobrist = Zobrist::default();

                // TODO: Check if active color can win in one move and disallow

                Ok(Gamestate {
                    board,
                    active_color,
                    castle_permissions,
                    en_passant,
                    halfmove_clock,
                    fullmove_number,
                    history,
                    zobrist,
                })
            }
            _ => Err(GamestateFENParseError::WrongNumFENSections(
                fen_sections.len(),
            )),
        }
    }

    fn gen_position_key(&self) -> u64 {
        let mut position_key: u64 = 0;

        // Piece location component
        for (square_index, piece_at_square) in self.board.pieces.iter().enumerate() {
            if let Some(piece) = *piece_at_square {
                position_key ^= self.zobrist.piece_keys[piece as usize][square_index];
            }
        }
        // Color (which player's turn) component
        if self.active_color == Color::White {
            position_key ^= self.zobrist.color_key
        };
        // En Passant component
        if let Some(square) = self.en_passant {
            position_key ^= self.zobrist.en_passant_keys[square as usize];
        }
        // Castle Permissions component
        let castle_permissions: u8 = self.castle_permissions.into();
        position_key ^= self.zobrist.castle_keys[castle_permissions as usize];

        position_key
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;
    use crate::{board::bitboard::BitBoard, error::BoardBuildError, file::File, gamestate};

    // FEN parsing tests
    // Full FEN parsing
    #[test]
    fn test_gamestate_try_from_valid_fen_default() {
        let input = DEFAULT_FEN;
        let output = Gamestate::try_from(input);
        let default = Gamestate::default();
        #[rustfmt::skip]
        let board = Board {
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

        let active_color = Color::White;
        let castle_permissions = CastlePerm::default();
        let en_passant = None;
        let halfmove_clock = 0;
        let fullmove_number = 1;
        let history = Vec::new();
        let zobrist = Zobrist::default();

        let expected: Result<Gamestate, GamestateFENParseError> = Ok(Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        });

        // board
        assert_eq!(
            output.as_ref().unwrap().board,
            expected.as_ref().unwrap().board
        );
        // active_color
        assert_eq!(
            output.as_ref().unwrap().active_color,
            expected.as_ref().unwrap().active_color
        );
        // castle_permissions
        assert_eq!(
            output.as_ref().unwrap().castle_permissions,
            expected.as_ref().unwrap().castle_permissions
        );
        // en_passant
        assert_eq!(
            output.as_ref().unwrap().en_passant,
            expected.as_ref().unwrap().en_passant
        );
        // halfmove_clock
        assert_eq!(
            output.as_ref().unwrap().halfmove_clock,
            expected.as_ref().unwrap().halfmove_clock
        );
        // fullmove_number
        assert_eq!(
            output.as_ref().unwrap().fullmove_number,
            expected.as_ref().unwrap().fullmove_number
        );
        // history
        assert_eq!(
            output.as_ref().unwrap().history,
            expected.as_ref().unwrap().history
        );
        // zobrist
        assert_eq!(
            output.as_ref().unwrap().zobrist,
            expected.as_ref().unwrap().zobrist
        );
        // println!("output zobrist:{:?}\nexpected zobrist:{:?}", output.as_ref().unwrap().zobrist, output.as_ref().unwrap().zobrist);
        assert_eq!(output, expected);
        assert_eq!(default, expected.unwrap());
    }

    // Square Attacks
    #[test]
    fn test_square_attacked_queen_no_blockers() {
        // const FEN_1: &str = "8/3q4/8/8/4Q3/8/8/8 w - - 0 2";
        #[rustfmt::skip]
        let board = Board {
            pieces: [
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    Some(Piece::WhiteQueen), None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     Some(Piece::BlackQueen), None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
            ],      
            pawns: [BitBoard(0), BitBoard(0)],
            kings_square: [None; Color::COUNT],
            piece_count: [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0],
            big_piece_count: [1, 1],
            // NOTE: King considered major piece for us
            major_piece_count: [1, 1],
            minor_piece_count: [0, 0],
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
                vec![Square::E4],
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
                vec![Square::D7],
                // BlackKing
                vec![]
            ]
        };
        let active_color = Color::White;
        let castle_permissions = CastlePerm::try_from(0).unwrap();
        let en_passant = None;
        let halfmove_clock = 0;
        let fullmove_number = 2;
        let history = Vec::new();
        let zobrist = Zobrist::default();

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] = gamestate.is_square_attacked(square);
            }
        }

        #[rustfmt::skip]
        let expected = [
            [false, true,  false, false, true,  false, false, true],
            [false, false, true,  false, true,  false, true,  false],
            [false, false, false, true,  true,  true,  false, false],
            [true,  true,  true,  true,  false, true,  true,  true],
            [false, false, false, true,  true,  true,  false, false],
            [false, false, true,  false, true,  false, true,  false],
            [false, true,  false, false, true,  false, false, true],
            [true,  false, false, false, true,  false, false, false]
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_attacked_queen_with_blocker() {
        // const FEN_1: &str = "8/3q4/8/8/4Q3/8/2P5/8 w - - 0 2";
        #[rustfmt::skip]
        let board = Board {
            pieces: [
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     Some(Piece::WhitePawn),   None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    Some(Piece::WhiteQueen), None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     Some(Piece::BlackQueen), None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
            ],      
            pawns: [BitBoard(0x00_00_00_00_00_00_04_00), BitBoard(0)],
            kings_square: [None; Color::COUNT],
            piece_count: [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0],
            big_piece_count: [1, 1],
            // NOTE: King considered major piece for us
            major_piece_count: [1, 1],
            minor_piece_count: [0, 0],
            piece_list: [
                // WhitePawns
                vec![Square::C2],
                // WhiteKnights
                vec![],
                // WhiteBishops
                vec![],
                // WhiteRooks
                vec![],
                // WhiteQueens
                vec![Square::E4],
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
                vec![Square::D7],
                // BlackKing
                vec![]
            ]
        };

        println!("{}", board);

        let active_color = Color::White;
        let castle_permissions = CastlePerm::try_from(0).unwrap();
        let en_passant = None;
        let halfmove_clock = 0;
        let fullmove_number = 2;
        let history = Vec::new();
        let zobrist = Zobrist::default();

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] = gamestate.is_square_attacked(square);
            }
        }

        #[rustfmt::skip]
        let expected = [
            [false, false, false, false, true,  false, false, true],
            [false, false, true,  false, true,  false, true,  false],
            [false, true,  false, true,  true,  true,  false, false],
            [true,  true,  true,  true,  false, true,  true,  true],
            [false, false, false, true,  true,  true,  false, false],
            [false, false, true,  false, true,  false, true,  false],
            [false, true,  false, false, true,  false, false, true],
            [true,  false, false, false, true,  false, false, false]
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_attacked_white_bishop_on_black_square() {
        // const FEN: &str = "8/8/8/8/8/2B4K/8/k7 w - - 0 1";
        #[rustfmt::skip]
        let board = Board {
            pieces: [
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, Some(Piece::BlackKing), None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     Some(Piece::WhiteBishop), None,                    None,                    None,                     None,                     Some(Piece::WhiteKing), None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
                None, None,                   None,                     None,                     None,                    None,                    None,                     None,                     None,                   None,
            ],      
            pawns: [BitBoard(0), BitBoard(0)],
            kings_square: [Some(Square::H3), Some(Square::A1)],
            piece_count: [0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0],
            big_piece_count: [2, 1],
            // NOTE: King considered major piece for us
            major_piece_count: [1, 1],
            minor_piece_count: [1, 0],
            piece_list: [
                // WhitePawns
                vec![],
                // WhiteKnights
                vec![],
                // WhiteBishops
                vec![Square::C3],
                // WhiteRooks
                vec![],
                // WhiteQueens
                vec![],
                // WhiteKing
                vec![Square::H3],
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
                vec![Square::A1],
            ]
        };
        let active_color = Color::White;
        let castle_permissions = CastlePerm::try_from(0).unwrap();
        let en_passant = None;
        let halfmove_clock = 0;
        let fullmove_number = 1;
        let history = Vec::new();
        let zobrist = Zobrist::default();

        let gamestate = Gamestate {
            board,
            active_color,
            castle_permissions,
            en_passant,
            halfmove_clock,
            fullmove_number,
            history,
            zobrist,
        };

        let mut output = [[false; File::COUNT]; Rank::COUNT];
        for rank in Rank::iter() {
            for file in File::iter() {
                let square = Square::from_file_and_rank(file, rank);
                output[rank as usize][file as usize] = gamestate.is_square_attacked(square);
            }
        }

        #[rustfmt::skip]
        let expected = [
            [false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, true,  true],
            [false, false, false, false, false, false, true,  false],
            [false, false, false, false, false, false, true,  true],
            [false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, false, false],
            [false, false, false, false, false, false, false, false],
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_square_attacked_visual_inspection() {
        const FEN_0: &str = "4k3/pppppppp/8/8/8/8/PPPPPPPP/3K4 w - - 0 1";
        const FEN_1: &str = "rnbqkbnr/1p1ppppp/8/2p5/4p3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
        const FEN_2: &str = "rnbqkbnr/1p1ppppp/8/2p5/4p3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        const FEN_3: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let fens = [FEN_0, FEN_1, FEN_2, FEN_3];

        for fen in fens {
            let gamestate = Gamestate::try_from(fen).unwrap();
            println!("FEN: {}", fen);
            println!("Board:\n{}", gamestate.board);
            println!("All squares attacked by {}:", gamestate.active_color);
            for rank in Rank::iter().rev() {
                for file in File::iter() {
                    let square = Square::from_file_and_rank(file, rank);
                    match gamestate.is_square_attacked(square) {
                        true => print!("X"),
                        false => print!("-"),
                    }
                }
                println!()
            }
            println!()
        }
    }

    // Display
    // TODO: When perft testing is built get rid of this test since it really isn't worth testing the display like this
    #[rustfmt::skip]
    #[test]
    fn test_gamestate_display() {
        let fen_start = DEFAULT_FEN;
        let gs_start = Gamestate::try_from(fen_start).unwrap();
        let gs_start_string = gs_start.to_string();
        let fen_wpe4 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let gs_wpe4 = Gamestate::try_from(fen_wpe4).unwrap();
        let gs_wpe4_string = gs_wpe4.to_string();
        let fen_bpc5 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
        let gs_bpc5 = Gamestate::try_from(fen_bpc5).unwrap();
        let gs_bpc5_string = gs_bpc5.to_string();
        let fen_wnf3 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        let gs_wnf3 = Gamestate::try_from(fen_wnf3).unwrap();
        let gs_wnf3_string = gs_wnf3.to_string();

        println!("Starting Position:\n{}", gs_start);
        println!("Move white pawn to E4:\n{}", gs_wpe4);
        println!("Move black pawn to C5:\n{}", gs_bpc5);
        println!("Move white knight to F3:\n{}", gs_wnf3);

        let expected_board_start = format!("{}{}{}{}{}{}{}{}{}",
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
        let expected_active_color_start = "White";
        let expected_en_passant_start = "None";
        let expected_castle_permissions_start = "KQkq";
        let expected_position_key_start = gs_start.gen_position_key();
        let expected_start = format!(
                                            "{}\nActive Color: {}\nEn Passant: {}\nCastle Permissions: {}\nPosition Key: {}\n", 
                                            expected_board_start,
                                            expected_active_color_start,
                                            expected_en_passant_start,
                                            expected_castle_permissions_start,
                                            expected_position_key_start
                                        );
        

        let expected_board_wpe4 = format!("{}{}{}{}{}{}{}{}{}",
                            "8\t♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n",
                            "7\t♟\t♟\t♟\t♟\t♟\t♟\t♟\t♟\n",
                            "6\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "5\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "4\t.\t.\t.\t.\t♙\t.\t.\t.\n",
                            "3\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "2\t♙\t♙\t♙\t♙\t.\t♙\t♙\t♙\n",
                            "1\t♖\t♘\t♗\t♕\t♔\t♗\t♘\t♖\n\n",
                            "\tA\tB\tC\tD\tE\tF\tG\tH\n"
                        );
        let expected_active_color_wpe4 = "Black";
        let expected_en_passant_wpe4 = "E3"; 
        let expected_castle_permissions_wpe4 = "KQkq";
        let expected_position_key_wpe4 = gs_wpe4.gen_position_key();
        let expected_wpe4 = format!(
                                            "{}\nActive Color: {}\nEn Passant: {}\nCastle Permissions: {}\nPosition Key: {}\n", 
                                            expected_board_wpe4,
                                            expected_active_color_wpe4,
                                            expected_en_passant_wpe4,
                                            expected_castle_permissions_wpe4,
                                            expected_position_key_wpe4
                                        );

        let expected_board_bpc5 = format!("{}{}{}{}{}{}{}{}{}",
                            "8\t♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n",
                            "7\t♟\t♟\t.\t♟\t♟\t♟\t♟\t♟\n",
                            "6\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "5\t.\t.\t♟\t.\t.\t.\t.\t.\n",
                            "4\t.\t.\t.\t.\t♙\t.\t.\t.\n",
                            "3\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "2\t♙\t♙\t♙\t♙\t.\t♙\t♙\t♙\n",
                            "1\t♖\t♘\t♗\t♕\t♔\t♗\t♘\t♖\n\n",
                            "\tA\tB\tC\tD\tE\tF\tG\tH\n"
                        );
        let expected_active_color_bpc5 = "White";
        let expected_en_passant_bpc5 = "C6"; 
        let expected_castle_permissions_bpc5 = "KQkq";
        let expected_position_key_bpc5 = gs_bpc5.gen_position_key();
        let expected_bpc5 = format!(
                                            "{}\nActive Color: {}\nEn Passant: {}\nCastle Permissions: {}\nPosition Key: {}\n", 
                                            expected_board_bpc5,
                                            expected_active_color_bpc5,
                                            expected_en_passant_bpc5,
                                            expected_castle_permissions_bpc5,
                                            expected_position_key_bpc5
                                        );

        let expected_board_wnf3 = format!("{}{}{}{}{}{}{}{}{}",
                            "8\t♜\t♞\t♝\t♛\t♚\t♝\t♞\t♜\n",
                            "7\t♟\t♟\t.\t♟\t♟\t♟\t♟\t♟\n",
                            "6\t.\t.\t.\t.\t.\t.\t.\t.\n",
                            "5\t.\t.\t♟\t.\t.\t.\t.\t.\n",
                            "4\t.\t.\t.\t.\t♙\t.\t.\t.\n",
                            "3\t.\t.\t.\t.\t.\t♘\t.\t.\n",
                            "2\t♙\t♙\t♙\t♙\t.\t♙\t♙\t♙\n",
                            "1\t♖\t♘\t♗\t♕\t♔\t♗\t.\t♖\n\n",
                            "\tA\tB\tC\tD\tE\tF\tG\tH\n"
                        );
        let expected_active_color_wnf3 = "Black";
        let expected_en_passant_wnf3 = "None";
        let expected_castle_permissions_wnf3 = "KQkq";
        let expected_position_key_wnf3 = gs_wnf3.gen_position_key();
        let expected_wnf3 = format!(
                                            "{}\nActive Color: {}\nEn Passant: {}\nCastle Permissions: {}\nPosition Key: {}\n", 
                                            expected_board_wnf3,
                                            expected_active_color_wnf3,
                                            expected_en_passant_wnf3,
                                            expected_castle_permissions_wnf3,
                                            expected_position_key_wnf3
                                        );

        assert_eq!(gs_start_string, expected_start);
        assert_eq!(gs_wpe4_string, expected_wpe4);
        assert_eq!(gs_bpc5_string, expected_bpc5);
        assert_eq!(gs_wnf3_string, expected_wnf3);
    }

    // FEN PARSING:
    // TODO:
    //     let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3";
    // Tests for extra spaces
    #[test]
    fn test_gamestate_try_from_valid_fen_untrimmed() {
        let input = "   rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ";
        let output = Gamestate::try_from(input);
        let expected = Ok(Gamestate::default());
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_valid_fen_spaces_between_sections() {
        let input = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR  w    KQkq    - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Ok(Gamestate::default());
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_valid_fen_spaces_wrong_number_of_sections() {
        let input = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ kq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::WrongNumFENSections(7));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fen_spaces_in_board_section() {
        let invalid_board_section = "rnbqkbnr/pppppppp/";
        let input = "rnbqkbnr/pppppppp/ 8/8/8/8/PPPPPPPP/RNBQK BNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(BoardFENParseError::WrongNumRanks(
                invalid_board_section.to_string(),
                3,
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fen_spaces_in_board_section_end() {
        let input = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK BNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(BoardFENParseError::RankFENParseError(
                RankFENParseError::InvalidNumSquares("RNBQK".to_string()),
            )),
        ));
        assert_eq!(output, expected);
    }

    // NOTE: enpassant testing for - is done by the tests that use default FENs
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_uppercase() {
        let en_passant_str = "E6";
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq E6 0 3";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::EnPassantFENParseError(
            EnPassantFENParseError::EnPassantUppercase,
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_en_passant_square() {
        let en_passant_str = "e9";
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e9 0 3";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::EnPassantFENParseError(
            EnPassantFENParseError::SquareConversionError(SquareConversionError::FromStr(
                strum::ParseError::VariantNotFound,
            )),
        ));
        assert_eq!(output, expected);
    }

    // En passant square can't be occupied
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_square_occupied() {
        let input = "rn1qkbnr/ppp2ppp/3pb3/3Pp3/8/8/PPPQPPPP/RNB1KBNR w KQkq e6 2 4";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::EnPassantFENParseError(
            EnPassantFENParseError::NonEmptySquares,
        ));
        assert_eq!(output, expected);
    }

    // Square behind en passant square can't be occupied
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_square_behind_occupied() {
        let input = "rnbqk1nr/ppp1bppp/3p4/3Pp3/8/8/PPPQPPPP/RNB1KBNR w KQkq e6 2 4";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::EnPassantFENParseError(
            EnPassantFENParseError::NonEmptySquares,
        ));
        assert_eq!(output, expected);
    }

    // Pawn has to be in front of en passant square
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_no_pawn_in_front() {
        let input = "rnbqkbnr/ppp2ppp/3p4/3P4/4p3/8/PPPQPPPP/RNB1KBNR w KQkq e6 0 4";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::EnPassantFENParseError(
            EnPassantFENParseError::CorrectPawnNotInFront(Color::Black, Square::E6),
        ));
        assert_eq!(output, expected);
    }

    // Correct color pawn has to be in front of en passant square
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_wrong_pawn_in_front() {
        let input = "rnbqkbnr/pp1p1ppp/2p5/4P3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 3";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::EnPassantFENParseError(
            EnPassantFENParseError::CorrectPawnNotInFront(Color::Black, Square::E6),
        ));
        assert_eq!(output, expected);
    }

    // Halfmove and Fullmove
    #[test]
    fn test_gamestate_try_from_invalid_halfmove_exceeds_max() {
        let halfmove: u32 = 100;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 100 1024";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::HalfmoveClockFENParseError(
            HalfmoveClockFENParseError::ExceedsMax(halfmove),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fullmove_exceeds_max() {
        let fullmove: u32 = 1025;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1025";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::FullmoveCounterFENParseError(
            FullmoveCounterFENParseError::NotInRange(fullmove),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fullmove_zero() {
        let fullmove: u32 = 0;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 0";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::FullmoveCounterFENParseError(
            FullmoveCounterFENParseError::NotInRange(fullmove),
        ));
        assert_eq!(output, expected);
    }

    // Tests for if Board and Rank Errors are being converted correctly to Gamestate Errors:
    #[test]
    fn test_gamestate_try_from_invalid_board_fen_all_8() {
        let invalid_board_str = "8/8/8/8/8/8/8/8";
        let input = "8/8/8/8/8/8/8/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(BoardFENParseError::InvalidKingNum(
                invalid_board_str.to_string(),
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_few_ranks() {
        let invalid_board_str = "8/8/rbkqn2p/8/8/8/PPKPP1PP";
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(BoardFENParseError::WrongNumRanks(
                invalid_board_str.to_string(),
                7,
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_ranks() {
        let invalid_board_str = "8/8/rbkqn2p/8/8/8/PPKPP1PP/8/";
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP/8/ w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::WrongNumRanks(invalid_board_str.to_string(), 9),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_empty_ranks() {
        let invalid_board_str = "8/8/rbkqn2p//8/8/PPKPP1PP/8";
        let input = "8/8/rbkqn2p//8/8/PPKPP1PP/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::Empty),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_few_kings() {
        let invalid_board_str = "8/8/rbqn3p/8/8/8/PPKPP1PP/8";
        let input = "8/8/rbqn3p/8/8/8/PPKPP1PP/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::InvalidKingNum(invalid_board_str.to_string()),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_kings() {
        let invalid_board_str = "8/8/rbqnkkpr/8/8/8/PPKPP1PP/8";
        let input = "8/8/rbqnkkpr/8/8/8/PPKPP1PP/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::InvalidNumOfPiece(invalid_board_str.to_string(), 'k'),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_white_queens() {
        let invalid_board_str = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/QQQQQQQQ";
        let input = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/QQQQQQQQ w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::InvalidNumOfPiece(invalid_board_str.to_string(), 'Q'),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_white_pawns() {
        let invalid_board_str = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/PPPPPPPP";
        let input = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/PPPPPPPP w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::InvalidNumOfPiece(invalid_board_str.to_string(), 'P'),
        )));
        assert_eq!(output, expected);
    }

    // Rank Level:
    #[test]
    fn test_gamestate_try_from_invalid_rank_fen_empty() {
        let invalid_board_str = "";
        let input = "/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::Empty),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_rank_fen_char() {
        let invalid_rank_str = "rn2Xb1r";
        let input = "rn2Xb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::InvalidChar(
                invalid_rank_str.to_string(),
                'X',
            )),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_rank_fen_digit() {
        let invalid_rank_str = "rn0kb1rqN"; // num squares would be valid
        let input = "rn0kb1rqN/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::InvalidDigit(
                invalid_rank_str.to_string(),
                0,
            )),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_rank_fen_too_many_squares() {
        let invalid_rank_str = "rn2kb1rqN";
        let input = "rn2kb1rqN/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::InvalidNumSquares(
                invalid_rank_str.to_string(),
            )),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_rank_fen_too_few_squares() {
        let invalid_rank_str = "rn2kb";
        let input = "rn2kb/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::InvalidNumSquares(
                invalid_rank_str.to_string(),
            )),
        )));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_rank_fen_two_consecutive_digits() {
        let invalid_rank_str = "pppp12p"; // adds up to 8 squares but isn't valid
        let input = "pppp12p/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let ouput = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(BoardFENParseError::RankFENParseError(
                RankFENParseError::TwoConsecutiveDigits(invalid_rank_str.to_string()),
            )),
        ));
        assert_eq!(ouput, expected);
    }

    #[test]
    fn test_board_invalid_rank_fen_two_consecutive_digits_invalid_num_squares() {
        let invalid_rank_str = "pppp18p"; // adds up to more than 8 squares but gets caught for consecutive digits
        let input = "pppp18p/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let ouput = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardBuildError(
            BoardBuildError::BoardFENParseError(BoardFENParseError::RankFENParseError(
                RankFENParseError::TwoConsecutiveDigits(invalid_rank_str.to_string()),
            )),
        ));
        assert_eq!(ouput, expected);
    }
}
