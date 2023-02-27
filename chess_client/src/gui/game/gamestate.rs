use chess_client::types::{Move, PlayerColor, Square};

pub struct GameState {
    pub player_color: PlayerColor,
    pub turn: PlayerColor,
    board: [[Option<Piece>; 8]; 8],
    is_visible: [[bool; 8]; 8],
}

impl GameState {
    pub fn new(player_color: PlayerColor) -> Self {
        let mut board: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];
        board[0][0] = 'r'.try_into().ok();
        board[0][1] = 'n'.try_into().ok();
        board[0][2] = 'b'.try_into().ok();
        board[0][3] = 'q'.try_into().ok();
        board[0][4] = 'k'.try_into().ok();
        board[0][5] = 'b'.try_into().ok();
        board[0][6] = 'n'.try_into().ok();
        board[0][7] = 'r'.try_into().ok();
        for col in &mut board[1] {
            *col = 'p'.try_into().ok();
        }
        for col in &mut board[6] {
            *col = 'P'.try_into().ok();
        }
        board[7][0] = 'R'.try_into().ok();
        board[7][1] = 'N'.try_into().ok();
        board[7][2] = 'B'.try_into().ok();
        board[7][3] = 'Q'.try_into().ok();
        board[7][4] = 'K'.try_into().ok();
        board[7][5] = 'B'.try_into().ok();
        board[7][6] = 'N'.try_into().ok();
        board[7][7] = 'R'.try_into().ok();
        let turn = PlayerColor::White;
        let is_visible = [[true; 8]; 8];
        GameState {
            player_color,
            turn,
            board,
            is_visible,
        }
    }

    pub fn get_square(&self, square: Square) -> Option<Piece> {
        self.board[square.rank as usize][square.file as usize]
    }

    pub fn set_square(&mut self, square: Square, piece: Option<Piece>) -> Option<Piece> {
        let mut piece = piece;
        std::mem::swap(
            &mut self.board[square.rank as usize][square.file as usize],
            &mut piece,
        );
        piece
    }

    pub fn is_legal_move(&self, move_: Move) -> bool {
        (move_.from != move_.to)
            && self
                .get_square(move_.from)
                .filter(|p| self.turn == Into::<PlayerColor>::into(*p))
                .is_some()
            && self
                .get_square(move_.to)
                .filter(|p| self.turn == Into::<PlayerColor>::into(*p))
                .is_none()
    }

    pub fn move_piece(&mut self, move_: Move) {
        if let Some(piece) = self.set_square(move_.from, None) {
            self.set_square(move_.to, Some(piece));
            self.turn = !self.turn;
        }
    }

    pub fn set_visibility(&mut self, square: Square, is_visible: bool) {
        self.is_visible[square.rank as usize][square.file as usize] = is_visible;
    }
}

pub struct GameStateIter<'a> {
    counter: usize,
    gamestate: &'a GameState,
}

impl<'a> Iterator for GameStateIter<'a> {
    type Item = (Square, Piece);
    fn next(&mut self) -> Option<Self::Item> {
        while self.counter < 64 {
            let rank = self.counter >> 3;
            let file = self.counter & 7;
            self.counter += 1;
            if let Some(piece) = self.gamestate.board[rank][file] {
                if self.gamestate.is_visible[rank][file] {
                    return Some((
                        Square {
                            rank: rank as u32,
                            file: file as u32,
                        },
                        piece,
                    ));
                }
            }
        }
        None
    }
}

impl<'a> IntoIterator for &'a GameState {
    type Item = (Square, Piece);
    type IntoIter = GameStateIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        GameStateIter {
            counter: 0,
            gamestate: self,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

impl TryFrom<char> for Piece {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'P' => Ok(Piece::WhitePawn),
            'N' => Ok(Piece::WhiteKnight),
            'B' => Ok(Piece::WhiteBishop),
            'R' => Ok(Piece::WhiteRook),
            'Q' => Ok(Piece::WhiteQueen),
            'K' => Ok(Piece::WhiteKing),
            'p' => Ok(Piece::BlackPawn),
            'n' => Ok(Piece::BlackKnight),
            'b' => Ok(Piece::BlackBishop),
            'r' => Ok(Piece::BlackRook),
            'q' => Ok(Piece::BlackQueen),
            'k' => Ok(Piece::BlackKing),
            _ => Err(()),
        }
    }
}

impl From<Piece> for char {
    fn from(piece: Piece) -> Self {
        match piece {
            Piece::WhitePawn => 'P',
            Piece::WhiteKnight => 'N',
            Piece::WhiteBishop => 'B',
            Piece::WhiteRook => 'R',
            Piece::WhiteQueen => 'Q',
            Piece::WhiteKing => 'K',
            Piece::BlackPawn => 'p',
            Piece::BlackKnight => 'n',
            Piece::BlackBishop => 'b',
            Piece::BlackRook => 'r',
            Piece::BlackQueen => 'q',
            Piece::BlackKing => 'k',
        }
    }
}

impl From<Piece> for PlayerColor {
    fn from(piece: Piece) -> Self {
        match piece {
            Piece::WhitePawn
            | Piece::WhiteKnight
            | Piece::WhiteBishop
            | Piece::WhiteRook
            | Piece::WhiteQueen
            | Piece::WhiteKing => PlayerColor::White,
            Piece::BlackPawn
            | Piece::BlackKnight
            | Piece::BlackBishop
            | Piece::BlackRook
            | Piece::BlackQueen
            | Piece::BlackKing => PlayerColor::Black,
        }
    }
}
