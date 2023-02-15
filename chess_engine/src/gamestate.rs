use rand::prelude::*;
use rand_pcg::Lcg128Xsl64;
use std::{default, fmt, num::ParseIntError};
use strum::EnumCount;
use strum_macros::{Display as EnumDisplay, EnumCount as EnumCountMacro};

use crate::{
    board::Board,
    castle_perms::{self, CastlePerm, NUM_CASTLE_PERM},
    error::{
        BoardFENParseError, CastlePermConversionError, FullmoveCounterFENParseError,
        GamestateFENParseError, HalfmoveClockFENParseError, RankFENParseError,
    },
    pieces::Piece,
    squares::Square,
    util::Color,
};

// CONSTANTS:
/// Maximum number of full moves we expect
pub const MAX_GAME_MOVES: usize = 1024;
/// When we reach 50 moves (aka 100 half moves) without a pawn advance or a piece capture the game ends
/// immediately in a tie
pub const HALF_MOVE_MAX: usize = 100;
pub const NUM_FEN_SECTIONS: usize = 6;
/// Number of squares for the internal board (10x12)
pub const NUM_BOARD_SQUARES: usize = 120;
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
// TODO: test to make sure seed is a good choice
/// Seed used for Zobrist Hashing. Note that many PRNG implementations will behave poorly
/// if the seed is poorly distributed (it should have roughly equal number of 0s and 1s)
// NOTE: the seed data comes from this article: https://www.pcg-random.org/posts/simple-portable-cpp-seed-entropy.html
const ZOBRIST_SEED: [u8; 32] = [
    0x67, 0x0e, 0x5a, 0x45, 0x9a, 0xc9, 0xea, 0x9c, 0x88, 0x85, 0x36, 0x20, 0xc4, 0xc8, 0x36, 0xf9,
    0x07, 0xab, 0x56, 0x40, 0xb2, 0x0b, 0x31, 0x3e, 0x7b, 0x94, 0x50, 0x51, 0x37, 0xf5, 0x0e, 0x84,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Undo {
    move_: u32,
    castle_permissions: CastlePerm,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    position_key: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct Zobrist {
    color_key: u64,
    piece_keys: [[u64; NUM_BOARD_SQUARES]; Piece::COUNT],
    en_passant_keys: [u64; NUM_BOARD_SQUARES],
    castle_keys: [u64; NUM_CASTLE_PERM],
}

// NOTE: https://craftychess.com/hyatt/collisions.html
// NOTE: https://www.unf.edu/~cwinton/html/cop4300/s09/class.notes/LCGinfo.pdf
// NOTE: https://www.stmintz.com/ccc/index.php?id=29863
// NOTE: https://www.pcg-random.org/index.html
// NOTE: https://rust-random.github.io/book/portability.html
// NOTE: https://rust-random.github.io/book/guide-rngs.html
// NOTE: https://www.pcg-random.org/posts/cpp-seeding-surprises.html
/// Zobrist hashing using rand_pcg variant that should work decently well on 32bit and 64bit machines
/// We don't require cryptographically secure PRNG's, but there have historically been
/// many truly terribly implemented random number generators, so we're doing our best to choose
/// a decent one, even though the effect of collisions seems to be fairly minimal for Zobrist
/// hashing.
impl Zobrist {
    fn new() -> Self {
        // declare seed deterministically from const we declared
        let mut seed: <Lcg128Xsl64 as SeedableRng>::Seed = ZOBRIST_SEED;
        // build Permuted Congruential Generator to do pseudo random number generation
        let mut rng: Lcg128Xsl64 = Lcg128Xsl64::from_seed(seed);
        // initialize Zobrist keys we want to fill with pseudo random values
        let mut color_key: u64 = rng.gen();
        let mut piece_keys = [[0u64; NUM_BOARD_SQUARES]; Piece::COUNT];
        for square_array in &mut piece_keys {
            rng.fill(square_array)
        }
        let mut en_passant_keys = [0u64; NUM_BOARD_SQUARES];
        rng.fill(&mut en_passant_keys);
        let mut castle_keys = [0u64; NUM_CASTLE_PERM];
        rng.fill(&mut castle_keys);

        Zobrist {
            color_key,
            piece_keys,
            en_passant_keys,
            castle_keys,
        }
    }
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

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

    // TODO: make sure that on the frontend the number of characters that can be passed is limited to something reasonable
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

                // TODO: Check if en_passant makes sense (x3 or x6 rank, must be a pawn of the correct color in front,
                // en passant square and the one behind it are empty since pawn just moved up two spaces)
                let en_passant_str = remaining_sections[2];
                let en_passant = match Square::try_from(en_passant_str.to_uppercase().as_str()) {
                    // en_passant_str must be lowercase
                    Ok(ep) => match en_passant_str {
                        uppercase if uppercase == en_passant_str.to_uppercase() => {
                            return Err(GamestateFENParseError::EnPassantUppercase)
                        }
                        _ => Some(ep),
                    },
                    Err(_) => match en_passant_str {
                        "-" => None,
                        _ => {
                            return Err(GamestateFENParseError::EnPassant(
                                en_passant_str.to_string(),
                            ))
                        }
                    },
                };

                let halfmove_clock_str = remaining_sections[3];
                let halfmove_clock = match halfmove_clock_str.parse::<u32>() {
                    Ok(num) => match num {
                        // if there is an en passant square, the half move clock must equal 0 (pawn must have moved for en passant to be active)
                        zero if zero == 0 => match en_passant {
                            None => zero,
                            Some(ep) => {
                                return Err(GamestateFENParseError::from(
                                    HalfmoveClockFENParseError::ZeroWhileEnPassant,
                                ))
                            }
                        },
                        // if this num was 100 the game would immediately tie, so this is considered invalid
                        n if (1..HALF_MOVE_MAX).contains(&(n as usize)) => n,
                        _ => {
                            return Err(GamestateFENParseError::from(
                                HalfmoveClockFENParseError::ExceedsMax(num),
                            ))
                        }
                    },
                    Err(e) => {
                        return Err(GamestateFENParseError::from(
                            HalfmoveClockFENParseError::from(e),
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
                                _ => return Err(GamestateFENParseError::from(FullmoveCounterFENParseError::SmallerThanHalfmoveClockDividedByTwo(n, halfmove_clock)))
                            }
                        }
                        _ => {
                            return Err(GamestateFENParseError::from(
                                FullmoveCounterFENParseError::NotInRange(num),
                            ))
                        }
                    },
                    Err(e) => {
                        return Err(GamestateFENParseError::from(
                            FullmoveCounterFENParseError::from(e),
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
    use super::*;
    use crate::board::bitboard::BitBoard;

    // TODO: properly seed and test Zobrist key gen to check for collision rate in norm
    #[test]
    fn test_gen_position_key_deterministic() {
        let gamestate = Gamestate::default();
        let output = gamestate.gen_position_key();
        let expected = gamestate.gen_position_key();
        println!("output: {}, expected: {}", output, expected);
        assert_eq!(output, expected);
    }

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
        let expected = Err(GamestateFENParseError::from(
            BoardFENParseError::WrongNumRanks(invalid_board_section.to_string(), 3),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fen_spaces_in_board_section_end() {
        let input = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK BNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::from(BoardFENParseError::from(
            RankFENParseError::InvalidNumSquares("RNBQK".to_string()),
        )));
        assert_eq!(output, expected);
    }

    // NOTE: enpassant testing for - is done by the tests that use default FENs
    #[test]
    fn test_gamestate_try_from_invalid_en_passant_uppercase() {
        let en_passant_str = "E6";
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq E6 0 3";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::EnPassantUppercase);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_en_passant_square() {
        let en_passant_str = "e9";
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e9 0 3";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::EnPassant(
            en_passant_str.to_string(),
        ));
        assert_eq!(output, expected);
    }

    // Halfmove and Fullmove
    #[test]
    fn test_gamestate_try_from_invalid_halfmove_exceeds_max() {
        let halfmove: u32 = 100;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 100 1024";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::from(
            HalfmoveClockFENParseError::ExceedsMax(halfmove),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fullmove_exceeds_max() {
        let fullmove: u32 = 1025;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1025";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::from(
            FullmoveCounterFENParseError::NotInRange(fullmove),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_fullmove_zero() {
        let fullmove: u32 = 0;
        let input = "rnbqkbnr/pppp1pp1/7p/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq - 0 0";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::from(
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
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::InvalidKingNum(invalid_board_str.to_string()),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_few_ranks() {
        let invalid_board_str = "8/8/rbkqn2p/8/8/8/PPKPP1PP";
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::WrongNumRanks(invalid_board_str.to_string(), 7),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_ranks() {
        let invalid_board_str = "8/8/rbkqn2p/8/8/8/PPKPP1PP/8/";
        let input = "8/8/rbkqn2p/8/8/8/PPKPP1PP/8/ w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::WrongNumRanks(invalid_board_str.to_string(), 9),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_empty_ranks() {
        let invalid_board_str = "8/8/rbkqn2p//8/8/PPKPP1PP/8";
        let input = "8/8/rbkqn2p//8/8/PPKPP1PP/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::Empty),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_few_kings() {
        let invalid_board_str = "8/8/rbqn3p/8/8/8/PPKPP1PP/8";
        let input = "8/8/rbqn3p/8/8/8/PPKPP1PP/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::InvalidKingNum(invalid_board_str.to_string()),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_kings() {
        let invalid_board_str = "8/8/rbqnkkpr/8/8/8/PPKPP1PP/8";
        let input = "8/8/rbqnkkpr/8/8/8/PPKPP1PP/8 w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::InvalidNumOfPiece(invalid_board_str.to_string(), 'k'),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_white_queens() {
        let invalid_board_str = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/QQQQQQQQ";
        let input = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/QQQQQQQQ w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::InvalidNumOfPiece(invalid_board_str.to_string(), 'Q'),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_board_fen_too_many_white_pawns() {
        let invalid_board_str = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/PPPPPPPP";
        let input = "8/8/rbqnkppr/8/8/8/PQKPP1PQ/PPPPPPPP w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::InvalidNumOfPiece(invalid_board_str.to_string(), 'P'),
        ));
        assert_eq!(output, expected);
    }

    // Rank Level:
    #[test]
    fn test_gamestate_try_from_invalid_rank_fen_empty() {
        let invalid_board_str = "";
        let input = "/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::Empty),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gamestate_try_from_invalid_rank_fen_char() {
        let invalid_rank_str = "rn2Xb1r";
        let input = "rn2Xb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::InvalidChar(
                invalid_rank_str.to_string(),
                'X',
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_try_from_invalid_rank_fen_digit() {
        let invalid_rank_str = "rn0kb1rqN"; // num squares would be valid
        let input = "rn0kb1rqN/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::InvalidDigit(
                invalid_rank_str.to_string(),
                0,
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_rank_fen_too_many_squares() {
        let invalid_rank_str = "rn2kb1rqN";
        let input = "rn2kb1rqN/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::InvalidNumSquares(
                invalid_rank_str.to_string(),
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_rank_fen_too_few_squares() {
        let invalid_rank_str = "rn2kb";
        let input = "rn2kb/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let output = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::InvalidNumSquares(
                invalid_rank_str.to_string(),
            )),
        ));
        assert_eq!(output, expected);
    }

    #[test]
    fn test_board_invalid_rank_fen_two_consecutive_digits() {
        let invalid_rank_str = "pppp12p"; // adds up to 8 squares but isn't valid
        let input = "pppp12p/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let ouput = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::TwoConsecutiveDigits(
                invalid_rank_str.to_string(),
            )),
        ));
        assert_eq!(ouput, expected);
    }

    #[test]
    fn test_board_invalid_rank_fen_two_consecutive_digits_invalid_num_squares() {
        let invalid_rank_str = "pppp18p"; // adds up to more than 8 squares but gets caught for consecutive digits
        let input = "pppp18p/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let ouput = Gamestate::try_from(input);
        let expected = Err(GamestateFENParseError::BoardFENParseError(
            BoardFENParseError::RankFENParseError(RankFENParseError::TwoConsecutiveDigits(
                invalid_rank_str.to_string(),
            )),
        ));
        assert_eq!(ouput, expected);
    }
}
