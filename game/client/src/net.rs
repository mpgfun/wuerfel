use std::{cell::RefCell, ops::ControlFlow, rc::Rc, vec::IntoIter};

use shared::net::{
    packets::{S2CPacket, join::JoinC2SPacket},
    readwrite::{ByteReader, ByteWriter, StreamRead},
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{MessageEvent, WebSocket, js_sys};

use crate::{ClientGameState, com::MpscMessage, console_log, net::packets::ClientPacketHandler};

pub use packets::Sender;

mod packets;

pub struct ClientByteReader(IntoIter<u8>);

impl ClientByteReader {
    pub fn new(vec: Vec<u8>) -> Self {
        Self(vec.into_iter())
    }
}

impl ByteReader for ClientByteReader {
    fn read_next_byte(&mut self) -> Option<u8> {
        self.0.next()
    }
}

pub struct ClientByteWriter(Vec<u8>);

impl ClientByteWriter {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn destroy(self) -> Vec<u8> {
        self.0
    }
}

impl ByteWriter for ClientByteWriter {
    fn write_byte(&mut self, byte: u8) {
        self.0.push(byte);
    }
}

pub fn create_ws() -> WebSocket {
    let ws = WebSocket::new("/ws").unwrap();
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    let mut cloned_ws = ws.clone();
    let onopen = Closure::<dyn FnMut()>::new(move || {
        let mut sender = Sender::new(&mut cloned_ws);
        sender.send(JoinC2SPacket { lobby_id: 123 });
    });
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();
    ws
}

fn create_on_message(ws: WebSocket, state: Rc<RefCell<ClientGameState>>) {
    let ws_clone = ws.clone();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        let Ok(buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() else {
            return;
        };
        if let ControlFlow::Break(_) = decode_and_apply_packet(
            state.clone(),
            js_sys::Uint8Array::new(&buffer).to_vec(),
            &mut ws_clone.clone(),
        ) {
            state.borrow_mut().should_close = true;
        }
    });
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}

pub fn handle_mpsc_message(
    msg: MpscMessage,
    ws: &mut WebSocket,
    state: Rc<RefCell<ClientGameState>>,
) {
    match msg {
        MpscMessage::CreateOnMessage => create_on_message(ws.clone(), state),
        MpscMessage::SendPacket(packet) => {
            let mut sender = Sender::new(ws);
            sender.send(packet);
        }
    }
}

pub fn decode_and_apply_packet(
    state: Rc<RefCell<ClientGameState>>,
    received_bytes: Vec<u8>,
    socket: &mut WebSocket,
) -> ControlFlow<(), ()> {
    let mut stream_reader = ClientByteReader::new(received_bytes);
    let packet = match S2CPacket::read(&mut stream_reader) {
        Ok(packet) => packet,
        Err(e) => {
            console_log!("Error reading packet: {e}");
            return ControlFlow::Break(());
        }
    };

    match packet {
        S2CPacket::JoinResponse(packet) => packet.apply(state, socket),
    }
}
