use config::Config;
use log::{debug, info};
use nanoserde::{DeBin, DeBinErr, SerBin};
use rand::{thread_rng, Rng};
use std::io::Error;
use std::sync::Arc;

use futures::join;
use futures_util::{stream::select, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use chess_client::types::{Move, PlayerColor, PlayerMessage, ServerResponse};

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
    let msg: PlayerMessage = try_decode_msg(msg).unwrap();
    match msg {
        PlayerMessage::GameVsComputer => {
            start_game_with_computer(socket).await;
        }
        PlayerMessage::GameVsHuman => {
            queue_tx.send(socket).unwrap();
        }
        _ => {
            socket.close(None).await.unwrap();
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
    let (mut white_socket, mut black_socket) = {
        let mut rng = thread_rng();
        if rng.gen_bool(0.5) {
            (left_socket, right_socket)
        } else {
            (right_socket, left_socket)
        }
    };

    let mut game = Gamestate::new();
    let white_resp = encode_resp(ServerResponse::GameStarted(PlayerColor::White));
    let black_resp = encode_resp(ServerResponse::GameStarted(PlayerColor::Black));

    let (x, y) = join!(white_socket.send(white_resp), black_socket.send(black_resp));
    x.unwrap();
    y.unwrap();

    let (mut white_write, white_read) = white_socket.split();
    let (mut black_write, black_read) = black_socket.split();

    let white_read =
        white_read.map(|msg| (PlayerColor::White, try_decode_msg(msg.unwrap()).unwrap()));
    let black_read =
        black_read.map(|msg| (PlayerColor::Black, try_decode_msg(msg.unwrap()).unwrap()));

    let mut player_msg_stream = select(white_read, black_read);

    while let Some(msg) = player_msg_stream.next().await {
        debug!("Recieved message: {:#?}", msg);
        match msg {
            (color, PlayerMessage::MovePiece(move_)) => {
                if color == game.active_color {
                    game.history.push(move_);
                    game.active_color = !color;
                    let resp = encode_resp(ServerResponse::MoveMade {
                        player: color,
                        move_,
                    });
                    let (x, y) = join!(white_write.send(resp.clone()), black_write.send(resp));
                    x.unwrap();
                    y.unwrap();
                }
            }
            (color, PlayerMessage::Resign) => {
                let resp = encode_resp(ServerResponse::GameWon(!color));
                let (x, y) = join!(white_write.send(resp.clone()), black_write.send(resp));
                x.unwrap();
                y.unwrap();
            }
            _ => {}
        }
    }
}

fn try_decode_msg(msg: Message) -> Result<PlayerMessage, DeBinErr> {
    DeBin::deserialize_bin(&msg.into_data())
}

fn encode_resp(msg: ServerResponse) -> Message {
    Message::Binary(msg.serialize_bin())
}

#[derive(Debug)]
struct Gamestate {
    active_color: PlayerColor,
    history: Vec<Move>,
}

impl Gamestate {
    fn new() -> Gamestate {
        Gamestate {
            active_color: PlayerColor::White,
            history: Vec::new(),
        }
    }
}
