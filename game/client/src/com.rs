use std::ops::ControlFlow;

use futures::{
    StreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
};
use shared::net::{packets::C2SPacket, readwrite::StreamWrite};
use web_sys::{MessageEvent, WebSocket};

use crate::{
    ClientState, console_log,
    graphics::{RenderingInfo, render, resize, update_zoom},
    net::{ClientByteWriter, handle_websocket_message_event},
};

pub enum MpscMessage {
    SocketOpened,
    WebSocketMessage(MessageEvent),
    SendPacket(C2SPacket),
    Scrolling(f64),
    Draw,
    MutateClientState(Box<dyn FnOnce(&mut ClientState)>),
}

enum SocketStatus {
    NotOpen(Vec<C2SPacket>),
    Open,
}

pub async fn mpsc_receiver_loop(
    mut rx: UnboundedReceiver<MpscMessage>,
    tx: UnboundedSender<MpscMessage>,
    ws: WebSocket,
    mut info: RenderingInfo,
    mut state: ClientState,
) {
    let mut socket_status = SocketStatus::NotOpen(Vec::new());
    while let Some(msg) = rx.next().await {
        match msg {
            MpscMessage::WebSocketMessage(event) => {
                match handle_websocket_message_event(event, tx.clone()) {
                    ControlFlow::Continue(()) => {}
                    ControlFlow::Break(()) => {
                        console_log!("Packet handler returned ControlFlow::Break(()), stopping.");
                        return;
                    }
                }
            }
            MpscMessage::Scrolling(delta) => update_zoom(&mut info, delta),
            MpscMessage::Draw => {
                resize(&mut info);
                render(&info, &state);
            }
            MpscMessage::SocketOpened => {
                if let SocketStatus::NotOpen(queue) = socket_status {
                    for packet in queue {
                        let mut writer = ClientByteWriter::new();
                        packet.write(&mut writer);
                        let _ = ws.send_with_u8_array(&writer.destroy());
                    }
                    socket_status = SocketStatus::Open;
                }
            }
            MpscMessage::MutateClientState(f) => f(&mut state),
            MpscMessage::SendPacket(packet) => match &mut socket_status {
                SocketStatus::Open => {
                    let mut writer = ClientByteWriter::new();
                    packet.write(&mut writer);
                    let _ = ws.send_with_u8_array(&writer.destroy());
                }
                SocketStatus::NotOpen(queue) => queue.push(packet),
            },
        }
    }
}
