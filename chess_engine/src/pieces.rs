#[derive(Debug, Copy, Clone)]
pub enum Figure {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug)]
pub enum Color {
    White,
    Black,
}

pub struct Piece {
    pub figure: Figure,
    pub color: Color,
}
