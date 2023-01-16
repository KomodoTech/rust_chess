use macroquad::{
    color::{Color, DARKGRAY, LIGHTGRAY, WHITE},
    math::vec2,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
    window::{clear_background, next_frame, screen_height, screen_width},
};

use super::Scene;

const SQUARES: u8 = 8;
const LIGHT_COLOR: Color = Color::new(234. / 255., 233. / 255., 212. / 255., 1.);
const DARK_COLOR: Color = Color::new(84. / 255., 114. / 255., 150. / 255., 1.);

pub async fn game_scene() -> Scene {
    let path = "assets/boards/board.png";
    let board_texture: Texture2D = load_texture(path).await.unwrap();
    loop {
        clear_background(LIGHTGRAY);
        let game_size = screen_width().min(screen_height());
        let margin = (screen_height() - game_size) / 2. + 10.;
        let sq_size = (screen_height() - margin) / SQUARES as f32;

        draw_texture_ex(
            board_texture,
            (screen_width() - 8. * sq_size) / 2.,
            margin / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(8. * sq_size, 8. * sq_size)),
                ..Default::default()
            },
        );

        draw_text("CLOCK", 20., 20., 20., DARKGRAY);
        draw_text("CLOCK", 20., screen_height() - 20., 20., DARKGRAY);

        next_frame().await
    }
}
