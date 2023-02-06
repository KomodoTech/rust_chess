use super::Scene;
use macroquad::{
    color::{BLACK, WHITE},
    prelude::{get_time, info},
    text::draw_text,
    window::{clear_background, next_frame, screen_height, screen_width},
};
use quad_net::quad_socket::client::QuadSocket;

pub async fn connect() -> Scene {
    let socket = QuadSocket::connect("ws://localhost:8091").unwrap();
    info!("connected to socket");
    #[cfg(target_arch = "wasm32")]
    {
        while socket.is_wasm_websocket_connected() == false {
            clear_background(BLACK);
            draw_text(
                &format!(
                    "Looking for opponent{}",
                    ".".repeat(((get_time() * 2.0) as usize) % 4)
                ),
                screen_width() / 2.0 - 160.0,
                screen_height() / 2.0,
                40.,
                WHITE,
            );
            next_frame().await;
        }
    }
    Scene::QuickGame(socket)
}
