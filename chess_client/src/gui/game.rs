mod gamestate;
mod mouse;
mod screen;

use super::Scene;
use chess_client::types::{Piece, PlayerColor, PlayerMessage, ServerResponse, Square};
use gamestate::GameState;
use macroquad::{
    color::{LIGHTGRAY, WHITE},
    math::{Rect, Vec2},
    prelude::info,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{clear_background, next_frame},
};
use mouse::MouseState;
use quad_net::quad_socket::client::QuadSocket;
use screen::ScreenDimensions;

pub async fn game_scene(color: PlayerColor, mut socket: QuadSocket) -> Scene {
    info!("new game started as color {:#?}", color);
    let path = "assets/boards/board.png";
    let board_texture: Texture2D = load_texture(path).await.unwrap();

    let path = "assets/pieces/wiki_chess.png";
    let piece_texture: Texture2D = load_texture(path).await.unwrap();

    let mut gamestate = GameState::new(color);
    let mut dimensions = ScreenDimensions::default();
    let mut mouse_state = MouseState::default();
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
        dimensions.update();
        if let Some(move_) = mouse_state.update_gamestate(&dimensions, &mut gamestate) {
            if gamestate.player_color == gamestate.turn {
                let msg = PlayerMessage::MovePiece(move_);
                socket.send_bin(&msg);
            }
        }
        clear_background(LIGHTGRAY);
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

        for (square, piece) in gamestate.into_iter() {
            draw_piece_from_square(piece_texture, piece, square, &dimensions);
        }

        if let Some(square) = mouse_state.last_clicked {
            let _ = gamestate.get_square(square).map(|p| {
                draw_piece(
                    piece_texture,
                    p,
                    dimensions.square_size,
                    mouse_state.coords.1 - dimensions.square_size / 2.0,
                    mouse_state.coords.0 - dimensions.square_size / 2.0,
                );
            });
        };
        next_frame().await
    }
}

fn draw_piece(texture: Texture2D, piece: Piece, size: f32, y_coord: f32, x_coord: f32) {
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
        x_coord,
        y_coord,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::splat(size)),
            source: Some(rectangle),
            ..Default::default()
        },
    );
}

fn draw_piece_from_square(
    texture: Texture2D,
    piece: Piece,
    square: Square,
    dimensions: &ScreenDimensions,
) {
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

    let x_coord = dimensions.hor_margin + dimensions.square_size * square.file as f32;
    let y_coord = dimensions.vert_margin + dimensions.square_size * square.rank as f32;

    draw_texture_ex(
        texture,
        x_coord,
        y_coord,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::splat(dimensions.square_size)),
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
