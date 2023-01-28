use chess_engine::board::Board;
use macroquad::experimental::collections::storage;
use web_sys::WebSocket;

mod gui;
use gui::Scene;

#[macroquad::main("Chess")]
async fn main() {
    let _ = Board {};

    let gui_resources = gui::GuiResources::new();
    storage::store(gui_resources);

    let mut next_scene = Scene::MainMenu;
    loop {
        match next_scene {
            Scene::MainMenu => {
                next_scene = gui::main_menu().await;
            }
            Scene::QuickGame => {
                next_scene = gui::game_scene().await;
            }
            _ => todo!(),
        }
    }
}
