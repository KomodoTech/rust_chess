use super::Scene;
use chess_engine::pieces::Piece;
use macroquad::{
    color::{BLACK, LIGHTGRAY, WHITE},
    input::{is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton},
    math::{vec2, Rect, Vec2},
    text::draw_text,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{clear_background, next_frame, screen_height, screen_width},
};
use quad_net::quad_socket::client::QuadSocket;

enum MouseState {
    Unclicked,
    Clicked {
        row: usize,
        col: usize,
        piece: Option<Piece>,
    },
}

pub async fn game_scene() -> Scene {
    let path = "assets/boards/board.png";
    let board_texture: Texture2D = load_texture(path).await.unwrap();

    let path = "assets/pieces/wiki_chess.png";
    let piece_texture: Texture2D = load_texture(path).await.unwrap();

    let mut piece_array: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];
    for row in &mut piece_array {
        row[0] = 'P'.try_into().ok();
        row[1] = 'p'.try_into().ok();
        row[2] = 'K'.try_into().ok();
        row[3] = 'k'.try_into().ok();
        row[4] = 'Q'.try_into().ok();
        row[5] = 'q'.try_into().ok();
        row[6] = 'R'.try_into().ok();
        row[7] = 'b'.try_into().ok();
    }
    let mut height;
    let mut width;

    let mut game_size;
    let mut square_size;

    let mut board_x_margin;
    let mut board_y_margin;

    let mut mouse_x_pos;
    let mut mouse_y_pos;

    let mut mouse_state = MouseState::Unclicked;

    let mut socket = QuadSocket::connect("ws://localhost:8091").unwrap();
    let mut clock = vec2(0.0, 0.0);
    let mut last_edit_id = 0;
    loop {
        while let Some((mouse_x, mouse_y, id)) = socket.try_recv_bin() {
            clock.x = mouse_x;
            clock.y = mouse_y;
            last_edit_id = id;
        }
        clear_background(LIGHTGRAY);

        height = screen_height();
        width = screen_width().max(1100.0);
        game_size = width.min(height);
        square_size = game_size / 8.0;

        board_x_margin = (width - game_size) / 2.0;
        board_y_margin = (height - game_size) / 2.0;

        draw_texture_ex(
            board_texture,
            board_x_margin,
            board_y_margin,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(game_size)),
                ..Default::default()
            },
        );

        (mouse_x_pos, mouse_y_pos) = mouse_position();

        mouse_state = match mouse_state {
            MouseState::Unclicked => {
                if mouse_x_pos < board_x_margin
                    || board_x_margin + game_size <= mouse_x_pos
                    || mouse_y_pos < board_y_margin
                    || board_y_margin + game_size <= mouse_y_pos
                {
                    MouseState::Unclicked
                } else if is_mouse_button_pressed(MouseButton::Left) {
                    let col = ((mouse_x_pos - board_x_margin) / square_size).floor() as usize;
                    let row = ((mouse_y_pos - board_y_margin) / square_size).floor() as usize;
                    let piece = piece_array[row][col].take();
                    MouseState::Clicked { row, col, piece }
                } else {
                    MouseState::Unclicked
                }
            }
            MouseState::Clicked { row, col, piece } => {
                if is_mouse_button_down(MouseButton::Left) {
                    mouse_state
                } else if mouse_x_pos < board_x_margin
                    || board_x_margin + game_size <= mouse_x_pos
                    || mouse_y_pos < board_y_margin
                    || board_y_margin + game_size <= mouse_y_pos
                {
                    piece_array[row][col] = piece;
                    MouseState::Unclicked
                } else {
                    let new_col = ((mouse_x_pos - board_x_margin) / square_size).floor() as usize;
                    let new_row = ((mouse_y_pos - board_y_margin) / square_size).floor() as usize;
                    piece_array[new_row][new_col] = piece.or(piece_array[new_row][new_col]);
                    MouseState::Unclicked
                }
            }
        };
        for (row_idx, row) in piece_array.iter().enumerate() {
            for (col_idx, piece) in row.iter().enumerate() {
                if let Some(piece) = piece {
                    let piece_y_pos = board_y_margin + square_size * row_idx as f32;
                    let piece_x_pos = board_x_margin + square_size * col_idx as f32;
                    draw_piece(piece_texture, *piece, square_size, piece_y_pos, piece_x_pos);
                }
            }
        }
        if let MouseState::Clicked {
            row: _row,
            col: _col,
            piece: Some(piece),
        } = mouse_state
        {
            socket.send_bin(&(mouse_x_pos, mouse_y_pos));
            draw_piece(
                piece_texture,
                piece,
                square_size,
                mouse_y_pos - square_size / 2.0,
                mouse_x_pos - square_size / 2.0,
            );
        }
        draw_text(
            format!("{}, {}", clock.x, clock.y).as_str(),
            10.0,
            50.0,
            50.0,
            BLACK,
        );
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
