use crate::{board::Board, pieces::Color};

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
pub struct Gamestate {
    board: Board,
    active_color: Color,
    white_king_castle: bool,
    white_queen_castle: bool,
    black_king_castle: bool,
    black_queen_castle: bool,
    en_passant: String,
    halfmove_clock: u32,
    fullmove_number: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
}
