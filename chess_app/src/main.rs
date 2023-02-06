use macroquad::experimental::collections::storage;

mod gui;
use gui::Scene;

#[macroquad::main("Chess")]
async fn main() {
    let gui_resources = gui::GuiResources::new();
    storage::store(gui_resources);

    let mut next_scene = Scene::MainMenu;
    loop {
        match next_scene {
            Scene::MainMenu => {
                next_scene = gui::main_menu().await;
            }
            Scene::Connect => {
                next_scene = gui::connect().await;
            }
            Scene::QuickGame(socket) => {
                next_scene = gui::game_scene(socket).await;
            }
        }
    }
}
