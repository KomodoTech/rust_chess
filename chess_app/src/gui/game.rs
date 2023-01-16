use super::Scene;
use macroquad::{
    color::{DARKGRAY, LIGHTGRAY, WHITE},
    math::{Rect, Vec2},
    text::draw_text,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{clear_background, next_frame, screen_height, screen_width},
};

pub async fn game_scene() -> Scene {
    let path = "assets/boards/board.png";
    let board_texture: Texture2D = load_texture(path).await.unwrap();

    let path = "assets/pieces/wiki_chess.png";
    let piece_texture: Texture2D = load_texture(path).await.unwrap();
    loop {
        clear_background(LIGHTGRAY);
        let height = screen_height();
        let width = screen_width();
        let game_size = width.min(height);

        draw_texture_ex(
            board_texture,
            (width - game_size) / 2.,
            (height - game_size) / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(game_size)),
                ..Default::default()
            },
        );
        draw_texture_ex(
            piece_texture,
            (width - game_size) / 2.,
            (height - game_size) / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(game_size / 8.)),
                source: Some(Rect {
                    x: 0.,
                    y: 0.,
                    w: 170.,
                    h: 170.,
                }),
                ..Default::default()
            },
        );

        draw_text("CLOCK", 20., 20., 20., DARKGRAY);
        draw_text("CLOCK", 20., height - 20., 20., DARKGRAY);

        next_frame().await
    }
}
