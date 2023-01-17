use super::Scene;
use macroquad::{
    color::{DARKGRAY, LIGHTGRAY, WHITE},
    input::{is_mouse_button_down, mouse_position, MouseButton},
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
        for (i, pos) in [
            (4, 1),
            (3, 1),
            (2, 1),
            (1, 1),
            (0, 1),
            (2, 1),
            (3, 1),
            (4, 1),
        ]
        .into_iter()
        .enumerate()
        {
            draw_texture_ex(
                piece_texture,
                (width - game_size) / 2. + game_size * i as f32 / 8.,
                (height - game_size) / 2.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::splat(game_size / 8.)),
                    source: Some(Rect {
                        x: 170. * pos.0 as f32,
                        y: 170. * pos.1 as f32,
                        w: 170.,
                        h: 170.,
                    }),
                    ..Default::default()
                },
            );
        }
        let (pos_x, pos_y) = if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = mouse_position();
            (x - game_size / 16., y - game_size / 16.)
        } else {
            (
                (width - game_size) / 2.,
                (height - game_size) / 2. + game_size * 6. / 8.,
            )
        };
        draw_texture_ex(
            piece_texture,
            pos_x,
            pos_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(game_size / 8.)),
                source: Some(Rect {
                    x: 170. * 5.,
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
