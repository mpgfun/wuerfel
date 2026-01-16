use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde_json::Value;
use warp::filters::ws::{Message, WebSocket};

use crate::{
    game::ServerCommand,
    schemas::{ClickC2SMessage, Color, PlayerID},
};

pub enum PlayerCommand {
    #[allow(unused)]
    Disconnect,
    SendMessage(Message),
}

#[derive(Debug)]
pub enum PlayerDisconnectReason {
    Disconnected,
    #[allow(unused)]
    WarpError(warp::Error),
    InvalidData,
    ServerError,
}

pub struct Player {
    socket_tx: SplitSink<warp::ws::WebSocket, warp::ws::Message>,
    socket_rx: SplitStream<warp::ws::WebSocket>,
    pub id: PlayerID,
    rx: tokio::sync::mpsc::Receiver<PlayerCommand>,
}

impl Player {
    pub fn new(ws: WebSocket, rx: tokio::sync::mpsc::Receiver<PlayerCommand>) -> Self {
        let (socket_tx, socket_rx) = ws.split();
        Self {
            socket_tx,
            socket_rx,
            id: generate_id(),
            rx,
        }
    }

    pub async fn handle_connection(
        mut self,
        tx: tokio::sync::mpsc::Sender<ServerCommand>,
    ) -> Result<(), PlayerDisconnectReason> {
        loop {
            tokio::select! {
                Some(player_command) = self.rx.recv() => {
                    match player_command {
                        PlayerCommand::Disconnect => return Err(PlayerDisconnectReason::Disconnected),
                        PlayerCommand::SendMessage(msg) => self.try_send_message(msg).await?,
                    }
                },
                Some(Ok(msg)) = self.socket_rx.next() => {
                    let Ok(msg) = msg.to_str() else {
                        return Err(PlayerDisconnectReason::InvalidData);
                    };
                    let Value::Object(obj) = serde_json::from_str(msg).map_err(|_| PlayerDisconnectReason::InvalidData)? else {
                        return Err(PlayerDisconnectReason::InvalidData);
                    };
                    let Some(Value::String(msg_type)) = obj.get("type") else {
                        return Err(PlayerDisconnectReason::InvalidData);
                    };
                    let Some(data) = obj.get("data") else {
                        return Err(PlayerDisconnectReason::InvalidData);
                    };
                    match msg_type.as_str() {
                        "click" => {
                            let Ok(click_message) = serde_json::from_value::<ClickC2SMessage>(data.clone()) else {
                                return Err(PlayerDisconnectReason::InvalidData);
                            };
                            tx.send(ServerCommand::PlayerClick(self.id, click_message.position)).await.map_err(|_| PlayerDisconnectReason::ServerError)?;
                        },
                        _ => return Err(PlayerDisconnectReason::InvalidData),
                    }
                },
                else => break,
            }
        }

        Err(PlayerDisconnectReason::Disconnected)
    }

    async fn try_send_message(&mut self, msg: Message) -> Result<(), PlayerDisconnectReason> {
        self.socket_tx
            .send(msg)
            .await
            .map_err(|e| PlayerDisconnectReason::WarpError(e))
    }
}

fn generate_id() -> PlayerID {
    rand::random::<PlayerID>()
}

pub fn generate_random_color() -> Color {
    (rand::random(), rand::random(), rand::random())
}
