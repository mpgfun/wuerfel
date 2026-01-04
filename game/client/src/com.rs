use std::ops::{AddAssign, ControlFlow, DivAssign, SubAssign};

use futures::{
    StreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
};
use shared::net::{
    packets::C2SPacket,
    primitives::{map_config::MapConfiguration, position::Position, square::Square},
    readwrite::StreamWrite,
};
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
    let mut is_mouse_down = false;
    let mut mouse_down_start_position = (0.0, 0.0);
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
                    if let Some(some_last_mouse_pos) = info.last_mouse_pos {
                        let delta_x = x - some_last_mouse_pos.0;
                        let delta_y = y - some_last_mouse_pos.1;
                        info.camera_position.0 += delta_x / info.camera_zoom;
                        info.camera_position.1 += delta_y / info.camera_zoom;
                    }
                }
                info.last_mouse_pos = Some((x, y));
            }
            MpscMessage::MouseDown => {
                if let Some(last_mouse_pos) = info.last_mouse_pos {
                    mouse_down_start_position = last_mouse_pos;
                }
                is_mouse_down = true;
            }
            MpscMessage::MouseUp => {
                if let Some(game) = &mut state.game
                    && let Some((x, y)) = info.last_mouse_pos
                {
                    if mouse_down_start_position == (x, y) {
                        let rect = info.canvas.get_bounding_client_rect();
                        let x = x - rect.x();
                        let y = y - rect.y();
                        let world_coords = to_world_coords((x, y), &info, game.data.map_config);
                        game.data.snapshot.squares.insert(
                            Position::new(world_coords.0, world_coords.1),
                            Square {
                                num: 1,
                                owner: game.data.player_id,
                            },
                        );
                    }
                }
                is_mouse_down = false;
            }
        }
    }
}

fn to_world_coords(
    elem_coords: (f64, f64),
    info: &RenderingInfo,
    map_config: MapConfiguration,
) -> (u32, u32) {
    let mut coords = F64CoordsWrapper::new(elem_coords);

    coords -= info.zoom_transform;
    coords /= info.camera_zoom;
    coords += info.zoom_transform;
    coords -= info.camera_position;
    let square_size = info.width as f64 / map_config.size_x as f64;
    let coords = (coords.0.0 / square_size, coords.0.1 / square_size);
    (coords.0 as u32, coords.1 as u32)
}

struct F64CoordsWrapper((f64, f64));

impl F64CoordsWrapper {
    pub fn new(coords: (f64, f64)) -> Self {
        Self(coords)
    }
}

impl AddAssign<Self> for F64CoordsWrapper {
    fn add_assign(&mut self, rhs: Self) {
        self.0.0 += rhs.0.0;
        self.0.1 += rhs.0.1;
    }
}

impl AddAssign<(f64, f64)> for F64CoordsWrapper {
    fn add_assign(&mut self, rhs: (f64, f64)) {
        self.0.0 += rhs.0;
        self.0.1 += rhs.1;
    }
}

impl SubAssign<(f64, f64)> for F64CoordsWrapper {
    fn sub_assign(&mut self, rhs: (f64, f64)) {
        self.0.0 -= rhs.0;
        self.0.1 -= rhs.1;
    }
}

impl DivAssign<f64> for F64CoordsWrapper {
    fn div_assign(&mut self, rhs: f64) {
        self.0.0 /= rhs;
        self.0.1 /= rhs;
    }
}
