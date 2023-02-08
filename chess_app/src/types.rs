use nanoserde::{DeBin, SerBin};

#[derive(Clone, Debug, DeBin, SerBin)]
pub enum WebsocketMessage {
    GameVsComputer,
    GameVsHuman,
}

#[derive(Clone, Debug, DeBin, SerBin)]
pub enum WebSocketResponse {
    GameStarted(PlayerColor),
}

#[derive(Clone, Debug, DeBin, SerBin)]
pub enum PlayerColor {
    White,
    Black,
}
