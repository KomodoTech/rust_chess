use crate::{color::Color, piece::Piece};
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

// TODO: It's probably not necessary to generate
// sliding pieces this way with a macro. Think through
// pros and cons more carefully and see if we should
// do this elsewhere
#[macro_export]
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

#[macro_export]
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

#[cfg(test)]
mod test {
    use super::*;
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
}
