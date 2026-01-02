use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{ConnectInfo, WebSocketUpgrade},
    routing::any,
};
use axum_extra::{TypedHeader, headers};
use shared::net::primitives::numbers::PlayerID;
use tower_http::services::ServeDir;

use crate::net::ws_handler;

mod net;

struct ServerGameState {
    connected_clients: HashMap<PlayerID, SocketAddr>,
}

impl ServerGameState {
    pub fn new() -> Self {
        Self {
            connected_clients: HashMap::new(),
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
