use crate::{
    board::Board,
    error::ChessError as Error,
    moves::Move,
    pieces::{Color, Piece},
    squares::Square,
};

#[derive(Debug)]
pub struct Gamestate {
    board: Board,
    active_color: Color,
    white_king_castle: bool,
    white_queen_castle: bool,
    black_king_castle: bool,
    black_queen_castle: bool,
    en_passant: Option<String>,
    halfmove_clock: u32,
    fullmove_number: u32,
}

impl Gamestate {
    /// Returns game from FEN. Returns error if FEN is invalid
    pub fn from_fen(fen: &str) -> Result<Self, Error> {
        unimplemented!()
    }

    /// Returns FEN based on game state
    pub fn to_fen(&self) -> String {
        unimplemented!()
    }

    /// push a move from UCI onto the move stack and update the position
    /// returns move on legal move, error on invalid move
    pub fn push_uci(&mut self, uci: &str) -> Result<Move, Error> {
        let move_ = Move::from_uci(uci);
        self.push_move(&move_)
    }

    /// push a move onto the move stack and update the position
    /// returns move on legal move, error on invalid move
    pub fn push_move(&mut self, move_: &Move) -> Result<Move, Error> {
        unimplemented!()
    }

    /// checks if given color's king is in check
    pub fn is_in_check(&self, color: Color) -> bool {
        unimplemented!()
    }

    /// returns true if active player can claim a draw
    pub fn can_claim_draw(&self) -> bool {
        unimplemented!()
    }
}

/// Returns a board with default initial position
impl Default for Gamestate {
    fn default() -> Self {
        Gamestate {
            board: Board::default(),
            active_color: Color::White,
            white_king_castle: true,
            white_queen_castle: true,
            black_king_castle: true,
            black_queen_castle: true,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
}
