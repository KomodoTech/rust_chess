use super::gamestate::GameState;
use super::screen::ScreenDimensions;
use chess_client::types::{Move, Square};
use macroquad::input::{
    is_mouse_button_down, is_mouse_button_pressed, mouse_position, MouseButton,
};

#[derive(Default)]
pub struct MouseState {
    pub coords: (f32, f32),
    pub last_clicked: Option<Square>,
}

impl MouseState {
    pub fn update_gamestate(
        &mut self,
        dimensions: &ScreenDimensions,
        gamestate: &mut GameState,
    ) -> Option<Move> {
        self.coords = mouse_position();
        if let Some(clicked_square) = self.last_clicked {
            if is_mouse_button_down(MouseButton::Left) {
                return None;
            } else {
                let move_ = dimensions
                    .get_square(self.coords.0, self.coords.1)
                    .map(|s| {
                        gamestate.get_square(clicked_square).map(|_| Move {
                            from: clicked_square,
                            to: s,
                        })
                    })
                    .flatten();
                gamestate.set_visibility(clicked_square, true);
                self.last_clicked = None;
                return move_;
            }
        } else {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.last_clicked = dimensions
                    .get_square(self.coords.0, self.coords.1)
                    .map(|s| {
                        gamestate.get_square(s).map(|_| {
                            gamestate.set_visibility(s, false);
                            s
                        })
                    })
                    .flatten();
            }
            return None;
        }
    }
}
