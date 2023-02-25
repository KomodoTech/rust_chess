use chess_client::types::{Move, Piece, PlayerColor, Square};

pub struct GameState {
    pub player_color: PlayerColor,
    pub turn: PlayerColor,
    board: [[Option<Piece>; 8]; 8],
    is_visible: [[bool; 8]; 8],
}

impl GameState {
    pub fn new(player_color: PlayerColor) -> Self {
        let mut board: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];
        board[0][0] = 'R'.try_into().ok();
        board[0][1] = 'N'.try_into().ok();
        board[0][2] = 'B'.try_into().ok();
        board[0][3] = 'Q'.try_into().ok();
        board[0][4] = 'K'.try_into().ok();
        board[0][5] = 'B'.try_into().ok();
        board[0][6] = 'N'.try_into().ok();
        board[0][7] = 'R'.try_into().ok();
        for col in &mut board[1] {
            *col = 'P'.try_into().ok();
        }
        for col in &mut board[6] {
            *col = 'p'.try_into().ok();
        }
        board[7][0] = 'r'.try_into().ok();
        board[7][1] = 'n'.try_into().ok();
        board[7][2] = 'b'.try_into().ok();
        board[7][3] = 'q'.try_into().ok();
        board[7][4] = 'k'.try_into().ok();
        board[7][5] = 'b'.try_into().ok();
        board[7][6] = 'n'.try_into().ok();
        board[7][7] = 'r'.try_into().ok();
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

    pub fn move_piece(&mut self, move_: Move) {
        if let Some(piece) = self.set_square(move_.from, None) {
            self.set_square(move_.to, Some(piece));
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
