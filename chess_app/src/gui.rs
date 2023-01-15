mod game;
mod main_menu;
mod style;

pub use game::game_scene;
pub use main_menu::main_menu;
pub use style::GuiResources;

pub enum Scene {
    MainMenu,
    Login,
    QuickGame,
}
