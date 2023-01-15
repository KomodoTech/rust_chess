use macroquad::{
    color::{Color, DARKGRAY, LIGHTGRAY},
    shapes::draw_rectangle,
    text::draw_text,
    window::{clear_background, next_frame, screen_height, screen_width},
};

use super::Scene;

const SQUARES: u8 = 8;
const LIGHT_COLOR: Color = Color::new(234. / 255., 233. / 255., 212. / 255., 1.);
const DARK_COLOR: Color = Color::new(84. / 255., 114. / 255., 150. / 255., 1.);

pub async fn game_scene() -> Scene {
    loop {
        clear_background(LIGHTGRAY);

        let game_size = screen_width().min(screen_height());
        let offset_x = (screen_width() - game_size) / 2. + 10.;
        let offset_y = (screen_height() - game_size) / 2. + 10.;
        let sq_size = (screen_height() - offset_y * 2.) / SQUARES as f32;

        for rank in 0..8 {
            for file_ in 0..8 {
                draw_rectangle(
                    offset_x + file_ as f32 * sq_size,
                    offset_y + rank as f32 * sq_size,
                    sq_size,
                    sq_size,
                    if ((rank ^ file_) & 1) == 0 {
                        LIGHT_COLOR
                    } else {
                        DARK_COLOR
                    },
                );
            }
        }
        draw_text("CLOCK", 20., 20., 20., DARKGRAY);
        draw_text("CLOCK", 20., screen_height() - 20., 20., DARKGRAY);

        next_frame().await
    }
}
