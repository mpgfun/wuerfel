use std::{net::SocketAddr, ops::ControlFlow, sync::Arc};

use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures_util::{SinkExt, stream::SplitSink};
use shared::net::{
    packets::{
        join::JoinC2SPacket,
        join_response::{JoinResponseS2CPacket, JoinResponseS2CPacketData},
    },
    primitives::numbers::PlayerID,
    readwrite::StreamWrite,
};
use tokio::sync::Mutex;

use crate::{ServerGameState, game::GameSnapshotExt, generate_player_id, net::ServerByteWriter};

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

#[allow(unused)]
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
    ) -> ControlFlow<Option<(u16, String)>, PlayerID>;
}

impl ServerPacketSocketAddrHandler for JoinC2SPacket {
    async fn handle(
        &self,
        sender: &mut Sender<'_>,
        server_state: Arc<Mutex<ServerGameState>>,
        addr: SocketAddr,
    ) -> ControlFlow<Option<(u16, String)>, PlayerID> {
        let player_id = generate_player_id();
        let mut guard = server_state.lock().await;
        guard.connected_clients.insert(player_id, addr);
        let map_config_copy = guard.map_config;
        let flow = guard
            .snapshot
            .spawn_new_player(player_id, map_config_copy)
            .await;
        let snapshot_clone = guard.snapshot.clone();
        // unlock early
        drop(guard);
        if flow.is_continue() {
            sender
                .send(JoinResponseS2CPacket {
                    data: Some(JoinResponseS2CPacketData {
                        player_id: player_id,
                        snapshot: snapshot_clone,
                        map_config: map_config_copy,
                    }),
                })
                .await;
        } else {
            sender.send(JoinResponseS2CPacket { data: None }).await;
        }
        ControlFlow::Continue(player_id)
    }
}
