use std::{net::SocketAddr, ops::ControlFlow};

use axum::{Router, body::Bytes, extract::{ConnectInfo, WebSocketUpgrade, ws::{CloseFrame, Message, Utf8Bytes, WebSocket}}, response::IntoResponse, routing::any};
use axum_extra::{TypedHeader, headers};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let static_files = ServeDir::new("web").append_index_html_on_directories(true);
    let app = Router::new()
        .fallback_service(static_files)
        // .route("/", get_service(static_files));
        .route("/ws", any(ws_handler));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        println!("Error serving: {}", e);
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("unknown")
    };

    println!("'{user_agent}' at {addr} connected.");

    let _ = ws.on_upgrade(move |socket| handle_socket(socket, addr));
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    if socket.send(Message::Ping(Bytes::from_static(&[1, 2, 3]))).await.is_ok() {
        println!("pinged!");
    } else {
        println!("Failed to ping {addr}");
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    loop {
        if let Some(message) = receiver.next().await {
            let Ok(message) = message else {
                return;
            };
            if let ControlFlow::Break(close) = process_message(message, &mut sender, addr) {
                if let Some(close) = close {
                    if let Err(e) = sender.send(Message::Close(Some(CloseFrame {
                        code: close.0,
                        reason: Utf8Bytes::from(close.1),
                    }))).await {
                        println!("Error sending close message to {addr}: {e}");
                    }
                }
                return;
            }
        }
    }
}

fn process_message(message: Message, sender: &mut SplitSink<WebSocket, Message>, addr: SocketAddr) -> ControlFlow<Option<(u16, String)>> {
    match message {
        Message::Binary(data) => {
            // TODO read packet
            ControlFlow::Continue(())
        },
        _ => ControlFlow::Break(Some((1, String::from("Sent invalid data")))),
    }
}