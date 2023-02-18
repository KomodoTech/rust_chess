use super::Scene;
use chess_client::types::{WebSocketResponse, WebsocketMessage};
use macroquad::{
    color::{BLACK, WHITE},
    prelude::{get_time, info},
    text::draw_text,
    window::{clear_background, next_frame, screen_height, screen_width},
};
use quad_net::quad_socket::client::QuadSocket;

pub async fn connect() -> Scene {
    let mut socket = QuadSocket::connect("ws://localhost:8091").unwrap();
    #[cfg(target_arch = "wasm32")]
    {
        while !socket.is_wasm_websocket_connected() {
            draw_loading_screen("Connecting");
            next_frame().await;
        }
    }
    info!("socket connection accepted");
    socket.send_bin(&WebsocketMessage::GameVsHuman);

    loop {
        if let Some(WebSocketResponse::GameStarted(color)) = socket.try_recv_bin() {
            return Scene::QuickGame(color, socket);
        } else {
            draw_loading_screen("Searching for opponent");
            next_frame().await;
        }
    }
}

fn draw_loading_screen(msg: &str) {
    clear_background(BLACK);
    draw_text(
        &format!("{}{}", msg, ".".repeat(((get_time() * 2.0) as usize) % 4)),
        screen_width() / 2.0 - 160.0,
        screen_height() / 2.0,
        40.,
        WHITE,
    );
}
