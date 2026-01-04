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
    MouseDown,
    MouseUp,
    MouseMove(i32, i32),
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
    let mut last_mouse_pos: Option<(f64, f64)> = None;
    let mut is_mouse_down = false;
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
            MpscMessage::MouseMove(x, y) => {
                let rect = info.canvas.get_bounding_client_rect();
                let x = x as f64 - rect.x();
                let y = y as f64 - rect.y();
                if is_mouse_down {
                    if let Some(some_last_mouse_pos) = last_mouse_pos {
                        let delta_x = x - some_last_mouse_pos.0;
                        let delta_y = y - some_last_mouse_pos.1;
                        info.camera_position.0 += delta_x;
                        info.camera_position.1 += delta_y;
                    }
                }
                last_mouse_pos = Some((x, y));
            }
            MpscMessage::MouseDown => is_mouse_down = true,
            MpscMessage::MouseUp => is_mouse_down = false,
        }
    }
}
