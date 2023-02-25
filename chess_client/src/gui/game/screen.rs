use chess_client::types::Square;
use macroquad::window::{screen_height, screen_width};

#[derive(Default)]
pub struct ScreenDimensions {
    pub height: f32,
    pub width: f32,
    pub game_size: f32,
    pub square_size: f32,
    pub hor_margin: f32,
    pub vert_margin: f32,
}

impl ScreenDimensions {
    pub fn update(&mut self) {
        self.height = screen_height();
        self.width = screen_width();
        self.game_size = self.width.min(self.height);
        self.square_size = self.game_size / 8.0;

        self.hor_margin = (self.width - self.game_size) / 2.0;
        self.vert_margin = (self.height - self.game_size) / 2.0;
    }

    pub fn get_square(&self, x_coord: f32, y_coord: f32) -> Option<Square> {
        let rel_x_coord = x_coord - self.hor_margin;
        let rel_y_coord = y_coord - self.vert_margin;
        if rel_x_coord < 0.
            || rel_x_coord >= self.game_size
            || rel_y_coord < 0.
            || rel_y_coord >= self.game_size
        {
            None
        } else {
            let file = (rel_x_coord / self.square_size).floor() as u32;
            let rank = (rel_y_coord / self.square_size).floor() as u32;
            Some(Square { rank, file })
        }
    }
}
