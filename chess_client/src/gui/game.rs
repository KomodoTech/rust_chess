use super::Scene;
use chess_client::types::{Move, PlayerColor, PlayerMessage, ServerResponse, Square};
use chess_engine::pieces::Piece;
use macroquad::{
    color::{LIGHTGRAY, WHITE},
    input::{is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton},
    math::{Rect, Vec2},
    prelude::info,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{clear_background, next_frame, screen_height, screen_width},
};
use quad_net::quad_socket::client::QuadSocket;

enum MouseState {
    Unclicked,
    Clicked {
        clicked_square: Square,
        piece: Piece,
    },
}

#[derive(Default)]
struct ScreenDimensions {
    height: f32,
    width: f32,
    game_size: f32,
    square_size: f32,
    hor_margin: f32,
    vert_margin: f32,
}

impl ScreenDimensions {
    fn update(&mut self) {
        self.height = screen_height();
        self.width = screen_width();
        self.game_size = self.width.min(self.height);
        self.square_size = self.game_size / 8.0;

        self.hor_margin = (self.width - self.game_size) / 2.0;
        self.vert_margin = (self.height - self.game_size) / 2.0;
    }
}

struct GameState {
    player_color: PlayerColor,
    turn: PlayerColor,
    board: [[Option<Piece>; 8]; 8],
}

impl GameState {
    fn new(player_color: PlayerColor) -> GameState {
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
        GameState {
            player_color,
            turn,
            board,
        }
    }

    fn take_square(&mut self, square: Square) -> Option<Piece> {
        self.board[square.rank as usize][square.file as usize].take()
    }

    fn set_square(&mut self, square: Square, piece: Piece) -> Option<Piece> {
        self.board[square.rank as usize][square.file as usize].replace(piece)
    }

    fn move_piece(&mut self, move_: Move) {
        let piece = self.take_square(move_.from).unwrap();
        self.set_square(move_.to, piece);
    }
}

pub async fn game_scene(color: PlayerColor, mut socket: QuadSocket) -> Scene {
    info!("new game started as color {:#?}", color);
    let path = "assets/boards/board.png";
    let board_texture: Texture2D = load_texture(path).await.unwrap();

    let path = "assets/pieces/wiki_chess.png";
    let piece_texture: Texture2D = load_texture(path).await.unwrap();

    let mut gamestate = GameState::new(color);
    let mut dimensions = ScreenDimensions::default();

    let mut mouse_x_pos;
    let mut mouse_y_pos;

    let mut mouse_state = MouseState::Unclicked;
    loop {
        while let Some(resp) = socket.try_recv_bin::<ServerResponse>() {
            match resp {
                ServerResponse::MoveMade { player, move_ } => {
                    gamestate.move_piece(move_);
                    gamestate.turn = !player;
                }
                _ => {}
            }
        }
        clear_background(LIGHTGRAY);
        dimensions.update();

        draw_texture_ex(
            board_texture,
            dimensions.hor_margin,
            dimensions.vert_margin,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(dimensions.game_size)),
                ..Default::default()
            },
        );

        for (row_idx, row) in gamestate.board.iter().enumerate() {
            for (col_idx, piece) in row.iter().enumerate() {
                if let Some(piece) = piece {
                    let piece_y_pos =
                        dimensions.vert_margin + dimensions.square_size * row_idx as f32;
                    let piece_x_pos =
                        dimensions.hor_margin + dimensions.square_size * col_idx as f32;
                    draw_piece(
                        piece_texture,
                        *piece,
                        dimensions.square_size,
                        piece_y_pos,
                        piece_x_pos,
                    );
                }
            }
        }

        (mouse_x_pos, mouse_y_pos) = mouse_position();

        mouse_state = match mouse_state {
            MouseState::Unclicked => {
                if mouse_x_pos < dimensions.hor_margin
                    || dimensions.hor_margin + dimensions.game_size <= mouse_x_pos
                    || mouse_y_pos < dimensions.vert_margin
                    || dimensions.vert_margin + dimensions.game_size <= mouse_y_pos
                {
                    MouseState::Unclicked
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    let file = ((mouse_x_pos - dimensions.hor_margin) / dimensions.square_size)
                        .floor() as u32;
                    let rank = ((mouse_y_pos - dimensions.vert_margin) / dimensions.square_size)
                        .floor() as u32;
                    let clicked_square = Square { rank, file };
                    let piece = gamestate.take_square(clicked_square);
                    if let Some(piece) = piece {
                        MouseState::Clicked {
                            clicked_square,
                            piece,
                        }
                    } else {
                        MouseState::Unclicked
                    }
                } else {
                    MouseState::Unclicked
                }
            }
            MouseState::Clicked {
                clicked_square,
                piece,
            } => {
                if is_mouse_button_down(MouseButton::Left) {
                    mouse_state
                } else if mouse_x_pos < dimensions.hor_margin
                    || dimensions.hor_margin + dimensions.game_size <= mouse_x_pos
                    || mouse_y_pos < dimensions.vert_margin
                    || dimensions.vert_margin + dimensions.game_size <= mouse_y_pos
                {
                    gamestate.set_square(clicked_square, piece);
                    MouseState::Unclicked
                } else {
                    let new_file = ((mouse_x_pos - dimensions.hor_margin) / dimensions.square_size)
                        .floor() as u32;
                    let new_rank = ((mouse_y_pos - dimensions.vert_margin) / dimensions.square_size)
                        .floor() as u32;
                    let new_square = Square {
                        rank: new_rank,
                        file: new_file,
                    };
                    let move_ = PlayerMessage::MovePiece(Move {
                        from: clicked_square,
                        to: new_square,
                    });
                    if gamestate.player_color == gamestate.turn {
                        socket.send_bin(&move_);
                    }
                    gamestate.set_square(clicked_square, piece);
                    MouseState::Unclicked
                }
            }
        };
        if let MouseState::Clicked {
            clicked_square: _,
            piece,
        } = mouse_state
        {
            draw_piece(
                piece_texture,
                piece,
                dimensions.square_size,
                mouse_y_pos - dimensions.square_size / 2.0,
                mouse_x_pos - dimensions.square_size / 2.0,
            );
        }
        next_frame().await
    }
}

fn draw_piece(texture: Texture2D, piece: Piece, size: f32, y_pos: f32, x_pos: f32) {
    let rectangle = match piece {
        Piece::WhiteKing => WK_RECTANGLE,
        Piece::WhiteQueen => WQ_RECTANGLE,
        Piece::WhiteBishop => WB_RECTANGLE,
        Piece::WhiteKnight => WN_RECTANGLE,
        Piece::WhiteRook => WR_RECTANGLE,
        Piece::WhitePawn => WP_RECTANGLE,
        Piece::BlackKing => BK_RECTANGLE,
        Piece::BlackQueen => BQ_RECTANGLE,
        Piece::BlackBishop => BB_RECTANGLE,
        Piece::BlackKnight => BN_RECTANGLE,
        Piece::BlackRook => BR_RECTANGLE,
        Piece::BlackPawn => BP_RECTANGLE,
    };
    draw_texture_ex(
        texture,
        x_pos,
        y_pos,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::splat(size)),
            source: Some(rectangle),
            ..Default::default()
        },
    );
}

const WK_RECTANGLE: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 170.0,
    h: 170.0,
};
const WQ_RECTANGLE: Rect = Rect {
    x: 171.0,
    y: 0.0,
    w: 170.0,
    h: 170.0,
};
const WB_RECTANGLE: Rect = Rect {
    x: 342.0,
    y: 0.0,
    w: 170.0,
    h: 170.0,
};
const WN_RECTANGLE: Rect = Rect {
    x: 513.0,
    y: 0.0,
    w: 170.0,
    h: 170.0,
};
const WR_RECTANGLE: Rect = Rect {
    x: 684.0,
    y: 0.0,
    w: 170.0,
    h: 170.0,
};
const WP_RECTANGLE: Rect = Rect {
    x: 855.0,
    y: 0.0,
    w: 170.0,
    h: 170.0,
};
const BK_RECTANGLE: Rect = Rect {
    x: 0.0,
    y: 171.0,
    w: 170.0,
    h: 170.0,
};
const BQ_RECTANGLE: Rect = Rect {
    x: 171.0,
    y: 171.0,
    w: 170.0,
    h: 170.0,
};
const BB_RECTANGLE: Rect = Rect {
    x: 342.0,
    y: 171.0,
    w: 170.0,
    h: 170.0,
};
const BN_RECTANGLE: Rect = Rect {
    x: 513.0,
    y: 171.0,
    w: 170.0,
    h: 170.0,
};
const BR_RECTANGLE: Rect = Rect {
    x: 684.0,
    y: 171.0,
    w: 170.0,
    h: 170.0,
};
const BP_RECTANGLE: Rect = Rect {
    x: 855.0,
    y: 171.0,
    w: 170.0,
    h: 170.0,
};
