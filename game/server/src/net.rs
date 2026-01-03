use std::{net::SocketAddr, ops::ControlFlow, sync::Arc, vec::IntoIter};

use axum::{
    extract::{
        ConnectInfo, WebSocketUpgrade,
        ws::{CloseFrame, Message, Utf8Bytes, WebSocket},
    },
    response::IntoResponse,
};
use axum_extra::{TypedHeader, headers};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use shared::net::{
    packets::C2SPacket,
    primitives::numbers::PlayerID,
    readwrite::{ByteReader, ByteWriter, StreamRead},
};

use crate::{
    ServerGameState,
    net::packets::{Sender, ServerPacketSocketAddrHandler},
};

mod packets;

struct ServerByteReader(IntoIter<u8>);

impl ServerByteReader {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes.into_iter())
    }
}

impl ByteReader for ServerByteReader {
    fn read_next_byte(&mut self) -> Option<u8> {
        self.0.next()
    }
}

struct ServerByteWriter(Vec<u8>);

impl ServerByteWriter {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn destroy(self) -> Vec<u8> {
        self.0
    }
}

impl ByteWriter for ServerByteWriter {
    fn write_byte(&mut self, byte: u8) {
        self.0.push(byte);
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    server_state: Arc<tokio::sync::Mutex<ServerGameState>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("unknown")
    };

    println!("'{user_agent}' at {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr, server_state))
}

async fn process_message(
    message: Message,
    sender: &mut SplitSink<WebSocket, Message>,
    addr: SocketAddr,
    server_state: Arc<tokio::sync::Mutex<ServerGameState>>,
) -> ControlFlow<Option<(u16, String)>, Option<PlayerID>> {
    match message {
        Message::Binary(data) => {
            let mut reader = ServerByteReader::new(data.to_vec());
            let packet = match C2SPacket::read(&mut reader) {
                Ok(packet) => packet,
                Err(e) => {
                    println!("Failed to read packet: {e}");
                    return ControlFlow::Break(Some((1, String::from("Invalid packet"))));
                }
            };
            match packet {
                C2SPacket::Join(packet) => {
                    match packet
                        .handle(&mut Sender::new(sender), server_state, addr)
                        .await
                    {
                        ControlFlow::Break(v) => ControlFlow::Break(v),
                        ControlFlow::Continue(id) => ControlFlow::Continue(Some(id)),
                    }
                }
            }
        }
        _ => ControlFlow::Break(Some((1, String::from("Sent invalid data")))),
    }
}

async fn handle_socket(
    socket: WebSocket,
    addr: SocketAddr,
    server_state: Arc<tokio::sync::Mutex<ServerGameState>>,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut player_id: Option<PlayerID> = None;

    loop {
        if let Some(message) = receiver.next().await {
            let Ok(message) = message else {
                return;
            };
            let processed =
                process_message(message, &mut sender, addr, Arc::clone(&server_state)).await;
            match processed {
                ControlFlow::Break(close) => {
                    if let Some(close) = close {
                        if let Some(id) = player_id {
                            let mut guard = server_state.lock().await;
                            guard.remove_player(id);
                        }
                        if let Err(_e) = sender
                            .send(Message::Close(Some(CloseFrame {
                                code: close.0,
                                reason: Utf8Bytes::from(close.1),
                            })))
                            .await
                        {
                            // error likely means that the socket was already closed
                            // println!("Error sending close message to {addr}: {e}");
                        }
                    }
                    return;
                }
                ControlFlow::Continue(maybe_id) => {
                    if let Some(id) = maybe_id {
                        player_id = Some(id);
                    }
                }
            }
        }
    }
}
