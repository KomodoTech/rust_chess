use macroquad::{
    color::BLACK,
    experimental::collections::storage,
    math::vec2,
    ui::{root_ui, widgets},
    window::{clear_background, next_frame, screen_width},
};

use super::{GuiResources, Scene};

const BUTTON_WIDTH: f32 = 500.0;
const BUTTON_HEIGHT: f32 = 300.0;
const LABEL_HEIGHT: f32 = 300.0;
const BUTTON_OFFSET: f32 = 50.0;
const HALF_MARGIN: f32 = 10.0;

pub async fn main_menu() -> Scene {
    let resources = storage::get::<GuiResources>();
    loop {
        clear_background(BLACK);
        root_ui().push_skin(&resources.title_skin);

        let title = "CHESS";
        let label_size = root_ui().calc_size(title);
        let label_pos = vec2(screen_width() / 2. - label_size.x / 2., LABEL_HEIGHT);
        root_ui().label(Some(label_pos), title);

        if widgets::Button::new("Computer")
            .size(vec2(BUTTON_WIDTH, BUTTON_HEIGHT))
            .position(vec2(
                screen_width() / 2. - BUTTON_WIDTH - HALF_MARGIN,
                label_pos.y + label_size.y + BUTTON_OFFSET,
            ))
            .ui(&mut root_ui())
        {
            root_ui().pop_skin();
            return Scene::Connect;
        }

        if widgets::Button::new("Human")
            .size(vec2(BUTTON_WIDTH, BUTTON_HEIGHT))
            .position(vec2(
                screen_width() / 2. + HALF_MARGIN,
                label_pos.y + label_size.y + BUTTON_OFFSET,
            ))
            .ui(&mut root_ui())
        {
            root_ui().pop_skin();
            return Scene::Connect;
        }

        root_ui().pop_skin();
        next_frame().await;
    }
}
