use config::Config;
use env_logger::Builder;
use log::{debug, info};
use rand::Rng;
use std::io::Error;

use futures::join;
use futures_util::StreamExt;
use tokio::net::{TcpListener, TcpStream};

use chess_engine::gamestate::Gamestate;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let settings = Config::builder()
        .add_source(config::File::with_name("configs/default"))
        .build()
        .unwrap();
    let ws_url: String = settings
        .get("ws_url")
        .expect("Could not get url from config");
    let debug_level: String = settings
        .get("debug_level")
        .expect("Could not get debug_level from confifg");

    let mut builder = Builder::new();
    builder.parse_filters(&debug_level).init();

    let try_socket = TcpListener::bind(&ws_url).await;
    let listener = try_socket.expect("Failed to bind");

    info!("Listening on {}", ws_url);

    let mut game_queue: Option<TcpStream> = None;
    let mut rng = rand::thread_rng();
    while let Ok((stream, addr)) = listener.accept().await {
        debug!("received new stream from {:#?}", addr);
        match game_queue {
            Some(_) => {
                debug!("starting game");
                let (white_stream, black_stream) = if rng.gen_bool(0.5) {
                    (game_queue.take().unwrap(), stream)
                } else {
                    (stream, game_queue.take().unwrap())
                };
                tokio::spawn(start_game(white_stream, black_stream));
            }
            None => {
                debug!("setting game_queue");
                game_queue = Some(stream);
            }
        }
    }

    Ok(())
}

async fn start_game(white_stream: TcpStream, black_stream: TcpStream) {
    let white_stream = tokio_tungstenite::accept_async(white_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    let black_stream = tokio_tungstenite::accept_async(black_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (white_write, white_read) = white_stream.split();
    let (black_write, black_read) = black_stream.split();

    join!(
        white_read.forward(black_write),
        black_read.forward(white_write)
    );
}
