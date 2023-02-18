use chess_client::types::PlayerColor;
use quad_net::quad_socket::client::QuadSocket;

mod connect;
mod game;
mod main_menu;
mod style;

pub use connect::connect;
pub use game::game_scene;
pub use main_menu::main_menu;
pub use style::GuiResources;

pub enum Scene {
    MainMenu,
    Connect,
    QuickGame(PlayerColor, QuadSocket),
}
