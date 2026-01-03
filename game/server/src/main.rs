use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{ConnectInfo, WebSocketUpgrade},
    routing::any,
};
use axum_extra::{TypedHeader, headers};
use rand::random;
use shared::net::primitives::{
    game_snapshot::GameSnapshot, map_config::MapConfiguration, numbers::PlayerID,
    position::Position,
};
use tower_http::services::ServeDir;

use crate::net::ws_handler;

mod game;
mod net;

struct ServerGameState {
    connected_clients: HashMap<PlayerID, SocketAddr>,
    snapshot: GameSnapshot,
    map_config: MapConfiguration,
}

impl ServerGameState {
    pub fn new() -> Self {
        Self {
            connected_clients: HashMap::new(),
            snapshot: GameSnapshot::new(),
            map_config: MapConfiguration {
                size_x: 100,
                size_y: 100,
            },
        }
    }

    pub fn remove_player(&mut self, id: PlayerID) {
        self.connected_clients.remove(&id);
        self.snapshot.players.remove(&id);
        let mut squares_to_remove = Vec::<Position>::new();
        for (pos, sq) in &self.snapshot.squares {
            if sq.owner == id {
                squares_to_remove.push(*pos);
            }
        }
        for pos in squares_to_remove {
            self.snapshot.remove_square(pos);
        }
    }
}

#[tokio::main]
async fn main() {
    let state = Arc::new(tokio::sync::Mutex::new(ServerGameState::new()));
    let cloned_state = Arc::clone(&state);
    let static_files = ServeDir::new("web").append_index_html_on_directories(true);
    let app = Router::new().fallback_service(static_files).route(
        "/ws",
        any(
            |ws: WebSocketUpgrade,
             user_agent: Option<TypedHeader<headers::UserAgent>>,
             ConnectInfo(addr): ConnectInfo<SocketAddr>| {
                ws_handler(ws, user_agent, ConnectInfo(addr), cloned_state)
            },
        ),
    );
    let addr = "127.0.0.1:3000";
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        println!("Error serving: {}", e);
    }
}

fn generate_player_id() -> PlayerID {
    random()
}
