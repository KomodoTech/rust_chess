use config::Config;
use log::{debug, info};
use nanoserde::{DeBin, DeBinErr, SerBin};
use rand::{thread_rng, Rng};
use std::io::Error;
use std::sync::Arc;

use futures::join;
use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use chess_client::types::{PlayerColor, WebSocketResponse, WebsocketMessage};
use chess_engine::gamestate::Gamestate;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let settings = Config::builder()
        .add_source(config::File::with_name("configs/default"))
        .build()
        .unwrap();
    let websocket_url: String = settings
        .get("ws_url")
        .expect("Could not get url from config");
    let debug_level: String = settings
        .get("debug_level")
        .expect("Could not get debug_level from confifg");

    let mut builder = env_logger::Builder::new();
    builder.parse_filters(&debug_level).init();

    run_server(&websocket_url).await
}

async fn run_server(url: &str) -> Result<(), Error> {
    let (queue_tx, queue_rx) = mpsc::unbounded_channel::<WebSocketStream<TcpStream>>();
    let queue_tx = Arc::new(queue_tx);

    tokio::spawn(run_match_making(queue_rx));

    let listener = TcpListener::bind(url).await.expect("Failed to bind");
    info!("Listening on {}", url);

    while let Ok((stream, addr)) = listener.accept().await {
        debug!("received new stream from {:#?}", addr);
        let socket = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");
        tokio::spawn(process_socket(socket, Arc::clone(&queue_tx)));
    }
    Ok(())
}

async fn run_match_making(mut queue_rx: UnboundedReceiver<WebSocketStream<TcpStream>>) {
    info!("running match making");
    let mut waiting_room: Option<WebSocketStream<TcpStream>> = None;
    while let Some(socket) = queue_rx.recv().await {
        match waiting_room {
            Some(queue_socket) => {
                debug!("starting game");
                tokio::spawn(start_game_with_human(socket, queue_socket));
                waiting_room = None;
            }
            None => {
                debug!("setting waiting_room");
                waiting_room = Some(socket);
            }
        }
    }
}

async fn process_socket(
    mut socket: WebSocketStream<TcpStream>,
    queue_tx: Arc<UnboundedSender<WebSocketStream<TcpStream>>>,
) {
    let msg: Message = socket.next().await.unwrap().unwrap();
    let msg: WebsocketMessage = try_decode_msg(msg).unwrap();
    match msg {
        WebsocketMessage::GameVsComputer => {
            start_game_with_computer(socket).await;
        }
        WebsocketMessage::GameVsHuman => {
            queue_tx.send(socket).unwrap();
        }
    }
}

async fn start_game_with_computer(mut socket: WebSocketStream<TcpStream>) {
    socket.close(None).await.unwrap();
}

async fn start_game_with_human(
    left_socket: WebSocketStream<TcpStream>,
    right_socket: WebSocketStream<TcpStream>,
) {
    let (white_socket, black_socket) = {
        let mut rng = thread_rng();
        if rng.gen_bool(0.5) {
            (left_socket, right_socket)
        } else {
            (right_socket, left_socket)
        }
    };
    let (mut white_write, white_read) = white_socket.split();
    let (mut black_write, black_read) = black_socket.split();

    let msg = encode_resp(WebSocketResponse::GameStarted(PlayerColor::White));
    white_write.send(msg).await.unwrap();

    let msg = encode_resp(WebSocketResponse::GameStarted(PlayerColor::Black));
    black_write.send(msg).await.unwrap();

    let (x, y) = join!(
        white_read.forward(black_write),
        black_read.forward(white_write)
    );
    x.unwrap();
    y.unwrap();
}

fn try_decode_msg(msg: Message) -> Result<WebsocketMessage, DeBinErr> {
    DeBin::deserialize_bin(&msg.into_data())
}

fn encode_resp(msg: WebSocketResponse) -> Message {
    Message::Binary(msg.serialize_bin())
}
