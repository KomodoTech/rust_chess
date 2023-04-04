use crate::{
    board::{NUM_EXTERNAL_BOARD_SQUARES, NUM_INTERNAL_BOARD_SQUARES},
    color::Color,
    piece::Piece,
    square::{Square, Square64},
};
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

//=============================== FULLMOVE TO PLY =============================
/// Ply is the number of total halfmoves played during the game.
/// NOTE: fullmove_count starts at 1, so ply should be:
/// ((fullmove_count - 1) * 2) + (active_color as usize)
///  
///  (fullmove_count, ply, active_color)
///  Move 0: (1, 0, W)  ply = ((1 - 1) * 2) + 0 = 0
///  Move 1: (1, 1, B)  ply = ((1 - 1) * 2) + 1 = 1
///  Move 2: (2, 2, W)  ply = ((2 - 1) * 2) + 0 = 2
///  Move 3: (2, 3, B)  ply = ((2 - 1) * 2) + 1 = 3
///  Move 4: (3, 4, W)  ply = ((3 - 1) * 2) + 0 = 4, etc.
macro_rules! to_ply_count {
    ($fullmove_count: expr, $active_color: expr) => {{
        let fullmove_count: usize = $fullmove_count;
        let active_color: Color = $active_color;
        ((fullmove_count - 1) * 2) + (active_color as usize)
    }};
}

//============================ SLIDING PIECES =================================
// TODO: It's probably not necessary to generate
// sliding pieces this way with a macro. Think through
// pros and cons more carefully and see if we should
// do this elsewhere
macro_rules! gen_sliding_pieces {
    ($active_color: expr) => {{
        /// Holds all sliding pieces and is used for move generation
        const MOVE_GEN_PIECE_SLIDING: [Piece; 6] = [
            Piece::WhiteBishop,
            Piece::BlackBishop,
            Piece::WhiteRook,
            Piece::BlackRook,
            Piece::WhiteQueen,
            Piece::BlackQueen,
        ];

        const SLIDING_PIECES_LEN: usize = MOVE_GEN_PIECE_SLIDING.len() / Color::COUNT;
        // TODO: look into MaybeUinit for this
        let mut sliding_pieces = [Piece::WhiteBishop, Piece::WhiteRook, Piece::WhiteQueen];

        // should be a 0 or a 1
        let offset = $active_color as usize;
        let mut index = 0;

        loop {
            let curr_i = (2 * index) + offset;
            if (curr_i) < MOVE_GEN_PIECE_SLIDING.len() {
                sliding_pieces[index] = MOVE_GEN_PIECE_SLIDING[curr_i];
                index += 1;
            } else {
                break;
            }
        }

        sliding_pieces
    }};
}

/// Generates [Piece::WhiteKnight, Piece::WhiteKing] for White and
/// [Piece::BlackKnight, Piece::BlackKing] for Black
macro_rules! gen_non_sliding_pieces {
    ($active_color: expr) => {{
        /// Does not include Pawns since those are dealt with differently
        /// during move generation
        const MOVE_GEN_PIECE_NON_SLIDING_NON_PAWN: [Piece; 4] = [
            Piece::WhiteKnight,
            Piece::BlackKnight,
            Piece::WhiteKing,
            Piece::BlackKing,
        ];

        const SLIDING_PIECES_LEN: usize = MOVE_GEN_PIECE_NON_SLIDING_NON_PAWN.len() / Color::COUNT;
        // TODO: look into MaybeUinit for this
        let mut sliding_pieces = [Piece::WhiteKnight, Piece::WhiteKing];

        // should be a 0 or a 1
        let offset = $active_color as usize;
        let mut index = 0;

        loop {
            let curr_i = (2 * index) + offset;
            if (curr_i) < MOVE_GEN_PIECE_NON_SLIDING_NON_PAWN.len() {
                sliding_pieces[index] = MOVE_GEN_PIECE_NON_SLIDING_NON_PAWN[curr_i];
                index += 1;
            } else {
                break;
            }
        }

        sliding_pieces
    }};
}

//=========================== 10x12 AND 8x8 INDEX CONVERSION ==================
///  Converting between 8x8 and 10x12 index to make
///  certain conversions happen at pre-compile time and avoid the
///  overhead of converting back and forth between Square64 and Square
///  when you're just dealing with an index in a context where you know
///  you have valid Squares.
///
///  One such example is when you want to iterate (using enumerate)
///  through all the Squares in the Board pieces array, but then you want
///  to pass in a corresponding Square64 index as a usize to some other
///  array or function. Instead of taking the index you get back from
///  enumerate which will be a 10x12 index, converting it into a Square,
///  then converting that into a Square64 only to convert it back into
///  a usize, you can just call the macro when you know it is safe to do
///  so.

// TODO: try to improve error handling (Diagnostic on nightly, proc_macro_error crate)
/// Given a 10x12 index convert it to the corresponding 8x8 index
/// NOTE: Should only be used when you know for sure that you are
/// looking at valid Square indices
macro_rules! idx_120_to_64 {
    ($idx_120: expr) => {{
        // TODO: it appears that we don't have to worry about negative values
        // understand why
        if ($idx_120 >= SQUARE_120_TO_64_INDEX.len()) {
            panic!(
                "Expected index to be in range: 0..{}",
                NUM_INTERNAL_BOARD_SQUARES
            );
        }

    #[rustfmt::skip]
        const SQUARE_120_TO_64_INDEX: [isize; NUM_INTERNAL_BOARD_SQUARES] = [
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1,  0,  1,  2,  3,  4,  5,  6,  7, -1,
            -1,  8,  9, 10, 11, 12, 13, 14, 15, -1,
            -1, 16, 17, 18, 19, 20, 21, 22, 23, -1,
            -1, 24, 25, 26, 27, 28, 29, 30, 31, -1,
            -1, 32, 33, 34, 35, 36, 37, 38, 39, -1,
            -1, 40, 41, 42, 43, 44, 45, 46, 47, -1,
            -1, 48, 49, 50, 51, 52, 53, 54, 55, -1,
            -1, 56, 57, 58, 59, 60, 61, 62, 63, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        ];

        match SQUARE_120_TO_64_INDEX[$idx_120] {
            -1 => panic!("Expected macro to only be used when you have a valid Square. Found value -1 at index {}", $idx_120),
            idx_64 => usize::try_from(idx_64).expect("Expected idx_64 to successfully convert to usize."),
        }
    }};
}

/// Given a 8x8 index convert it to the corresponding 8x8 index
/// NOTE: Should only be used when you know for sure that you are
/// looking at valid Square indices
macro_rules! idx_64_to_120 {
    ($idx_64: expr) => {{
        if ($idx_64 >= SQUARE_64_TO_120_INDEX.len()) {
            panic!(
                "Expected index to be in range: 0..{}",
                NUM_EXTERNAL_BOARD_SQUARES
            );
        }

    #[rustfmt::skip]
        const SQUARE_64_TO_120_INDEX: [usize; NUM_EXTERNAL_BOARD_SQUARES] = [
            21, 22, 23, 24, 25, 26, 27, 28,
            31, 32, 33, 34, 35, 36, 37, 38,
            41, 42, 43, 44, 45, 46, 47, 48,
            51, 52, 53, 54, 55, 56, 57, 58,
            61, 62, 63, 64, 65, 66, 67, 68,
            71, 72, 73, 74, 75, 76, 77, 78,
            81, 82, 83, 84, 85, 86, 87, 88,
                                                        91, 92, 93, 94, 95, 96, 97, 98,];

        SQUARE_64_TO_120_INDEX[$idx_64]
    }};
}

#[cfg(test)]
mod test {
    use super::*;

    //======================== FULLMOVE_COUNT TO PLY =========================
    #[test]
    fn test_to_ply_macro_valid_white() {
        ///  Move 5: (3, 5, B)  ply = ((3 - 1) * 2) + 1 = 5
        ///  Move 6: (4, 6, W)  ply = ((4 - 1) * 2) + 0 = 6
        let active_color = Color::White;
        let fullmove_count = 4;
        let output = to_ply_count!(fullmove_count, active_color);
        let expected = 6;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_to_ply_macro_valid_black() {
        ///  Move 7: (4, 7, B)  ply = ((4 - 1) * 2) + 1 = 7
        let active_color = Color::Black;
        let fullmove_count = 4;
        let output = to_ply_count!(fullmove_count, active_color);
        let expected = 7;
        assert_eq!(output, expected);
    }

    //======================== MOVEGEN MACRO ==================================
    #[test]
    fn test_gen_sliding_pieces_macro() {
        let output = gen_sliding_pieces!(Color::White);
        let expected = [Piece::WhiteBishop, Piece::WhiteRook, Piece::WhiteQueen];
        assert_eq!(output, expected);
        let output = gen_sliding_pieces!(Color::Black);
        let expected = [Piece::BlackBishop, Piece::BlackRook, Piece::BlackQueen];
        assert_eq!(output, expected);
    }
    #[test]
    fn test_gen_non_sliding_non_pawn_pieces_macro() {
        let output = gen_non_sliding_pieces!(Color::White);
        let expected = [Piece::WhiteKnight, Piece::WhiteKing];
        assert_eq!(output, expected);
        let output = gen_non_sliding_pieces!(Color::Black);
        let expected = [Piece::BlackKnight, Piece::BlackKing];
        assert_eq!(output, expected);
    }

    //========================== INDEX CONVERSION MACRO =======================
    #[test]
    fn test_idx_120_to_64_valid() {
        let valid_square = Square::B7;
        let valid_idx = valid_square as usize;

        let output: usize = idx_120_to_64!(valid_idx);
        let expected: usize = 49;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_idx_120_to_64_invalid_square() {
        let invalid_idx: usize = 3; // valid usize but not a valid square

        let output: usize = idx_120_to_64!(invalid_idx);
    }

    #[should_panic]
    #[test]
    fn test_idx_120_to_64_invalid_out_of_bounds_too_large() {
        let invalid_idx = 120; // outside array bounds
        let output: usize = idx_120_to_64!(invalid_idx);
    }

    #[test]
    fn test_idx_64_to_120_valid() {
        let valid_square_64 = Square64::B7;
        let valid_idx = valid_square_64 as usize;

        let output: usize = idx_64_to_120!(valid_idx);
        let expected: usize = 82;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_idx_64_to_120_invalid_out_of_bounds_too_large() {
        let invalid_idx = 64; // outside array bounds
        let output: usize = idx_64_to_120!(invalid_idx);
    }
}
