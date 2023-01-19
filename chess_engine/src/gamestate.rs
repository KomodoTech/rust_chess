use crate::{board::{Board, Color}};

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const MAX_GAME_MOVES: usize = 2048;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Castle {
    WhiteKing = 1,
    WhiteQueen = 2,
    BlackKing = 4,
    BlackQueen = 8,
}

#[derive(Debug)]
struct Undo {
    move_: u32,
    castle_permissions: u32,
    en_passant: u32,
    halfmove_clock: u32,
    position_key: u64,
}

#[derive(Debug)]
pub struct Gamestate {
    board: Board,
    active_color: Color,
    castle_permissions: u32,
    en_passant: u32,
    halfmove_clock: u32,
    fullmove_number: u32,
    position_key: u64,
    history: [Undo; MAX_GAME_MOVES],
}

#[cfg(test)]
mod tests {
    use super::*;
}
