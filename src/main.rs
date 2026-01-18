use std::{fs, ops::ControlFlow, time::Duration};

use tokio::time::sleep;
use warp::Filter;

use crate::game::{ServerCommand, ServerSender};

mod game;
mod schemas;

async fn handle_ws(sender: ServerSender, ws: warp::ws::WebSocket) {
    if let Err(e) = sender.send(ServerCommand::AddPlayer(Box::new(ws))).await {
        panic!("Error sending ServerCommand: {}", e);
    }
}

#[tokio::main]
async fn main() {
    let mut game_state = game::GameState::new();
    let cloned_tx = game_state.tx.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let cloned_tx = cloned_tx.clone();
            ws.on_upgrade(move |ws| handle_ws(cloned_tx, ws))
        });

    let static_files = warp::fs::dir("./frontend/dist");

    let routes = ws_route.or(static_files);

    let cloned_tx = game_state.tx.clone();
    tokio::spawn(async move {
        let tps = game_state.tps as u64;
        let cloned_tx = cloned_tx.clone();
        loop {
            if cloned_tx.send(ServerCommand::Tick).await.is_err() {
                break;
            }
            sleep(Duration::from_millis(1000 / tps)).await;
        }
    });

    tokio::spawn(async move {
        while let Some(message) = game_state.rx.recv().await {
            if let ControlFlow::Break(()) = game_state.handle_message(message).await {
                break;
            }
        }
    });

    let port = fs::read_to_string("port").unwrap_or(String::from("3002"));
    let port = port.parse::<u16>().unwrap_or(3002);

    println!("Live on http://127.0.0.1:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
