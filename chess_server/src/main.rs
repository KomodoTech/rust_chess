use std::io::Error;

use futures_util::{future, SinkExt, StreamExt, TryStreamExt};
use log::info;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let addr = "127.0.0.1:8091".to_string();
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    info!("Peer address: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("New WebSocket connection: {}", addr);

    let (mut write, mut read) = ws_stream.split();
    // We should not forward messages other than text or binary.

    while let Some(msg) = read.next().await {
        let msg = msg.unwrap();
        info!("message recieved: {:#?}", msg);
        write.send(msg).await;
    }
}
