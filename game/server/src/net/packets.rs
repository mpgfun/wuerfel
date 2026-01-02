use std::{net::SocketAddr, ops::ControlFlow, sync::Arc};

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures_util::{SinkExt, stream::SplitSink};
use shared::net::{
    packets::{join::JoinC2SPacket, join_response::JoinResponseS2CPacket},
    primitives::{numbers::PlayerID, position::Position},
    readwrite::StreamWrite,
};
use tokio::sync::Mutex;

use crate::{ServerGameState, net::ServerByteWriter};

pub struct Sender<'a> {
    sender: &'a mut SplitSink<WebSocket, Message>,
}

impl<'a> Sender<'a> {
    pub fn new(sender: &'a mut SplitSink<WebSocket, Message>) -> Self {
        Self { sender }
    }

    pub async fn send<T: StreamWrite>(&mut self, packet: T) {
        let mut writer = ServerByteWriter::new();
        packet.write(&mut writer);
        let _ = self
            .sender
            .send(Message::Binary(Bytes::from(writer.destroy())))
            .await;
    }
}

pub trait ServerPacketHandler {
    async fn handle(
        &self,
        sender: &mut Sender<'_>,
        server_state: Arc<Mutex<ServerGameState>>,
        player_id: PlayerID,
    ) -> ControlFlow<Option<(u16, String)>>;
}

pub trait ServerPacketSocketAddrHandler {
    async fn handle(
        &self,
        sender: &mut Sender<'_>,
        server_state: Arc<Mutex<ServerGameState>>,
        addr: SocketAddr,
    ) -> ControlFlow<Option<(u16, String)>>;
}

impl ServerPacketSocketAddrHandler for JoinC2SPacket {
    async fn handle(
        &self,
        sender: &mut Sender<'_>,
        server_state: Arc<Mutex<ServerGameState>>,
        addr: SocketAddr,
    ) -> ControlFlow<Option<(u16, String)>> {
        dbg!(self);
        let mut guard = server_state.lock().await;
        guard.connected_clients.insert(1234, addr);
        // unlock early
        drop(guard);
        sender
            .send(JoinResponseS2CPacket {
                may_join: true,
                player_id: Some(1234),
                position: Some(Position::new(123, 456)),
            })
            .await;
        ControlFlow::Continue(())
    }
}
