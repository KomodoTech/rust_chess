use crate::error::ChessError as Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Figure {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Piece {
    pub figure: Figure,
    pub color: Color,
}

impl TryFrom<char> for Piece {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'P' => Ok(Piece {
                figure: Figure::Pawn,
                color: Color::White,
            }),
            'R' => Ok(Piece {
                figure: Figure::Rook,
                color: Color::White,
            }),
            'B' => Ok(Piece {
                figure: Figure::Bishop,
                color: Color::White,
            }),
            'N' => Ok(Piece {
                figure: Figure::Knight,
                color: Color::White,
            }),
            'Q' => Ok(Piece {
                figure: Figure::Queen,
                color: Color::White,
            }),
            'K' => Ok(Piece {
                figure: Figure::King,
                color: Color::White,
            }),
            'p' => Ok(Piece {
                figure: Figure::Pawn,
                color: Color::Black,
            }),
            'r' => Ok(Piece {
                figure: Figure::Rook,
                color: Color::Black,
            }),
            'b' => Ok(Piece {
                figure: Figure::Bishop,
                color: Color::Black,
            }),
            'n' => Ok(Piece {
                figure: Figure::Knight,
                color: Color::Black,
            }),
            'q' => Ok(Piece {
                figure: Figure::Queen,
                color: Color::Black,
            }),
            'k' => Ok(Piece {
                figure: Figure::King,
                color: Color::Black,
            }),
            _ => Err(Error::ParsePieceError(value)),
        }
    }
}
