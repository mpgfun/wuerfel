use std::{ops::ControlFlow, vec::IntoIter};

use futures::channel::mpsc::UnboundedSender;
use shared::net::{
    packets::S2CPacket,
    readwrite::{ByteReader, ByteWriter, StreamRead},
};
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{
    MessageEvent, WebSocket,
    js_sys::{ArrayBuffer, Uint8Array},
};

use crate::{com::MpscMessage, console_log, log_js_err, net::packets::ClientPacketHandler};

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

pub fn create_ws(tx: UnboundedSender<MpscMessage>) -> Option<WebSocket> {
    let ws = match WebSocket::new("/ws") {
        Ok(ws) => ws,
        Err(e) => {
            log_js_err!(e);
            return None;
        }
    };
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    let cloned_tx = tx.clone();
    let onopen = Closure::<dyn FnMut()>::new(move || {
        let _ = cloned_tx.unbounded_send(MpscMessage::SocketOpened);
    });
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event| {
        let _ = tx.unbounded_send(MpscMessage::WebSocketMessage(event));
    });
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
    Some(ws)
}

pub fn decode_and_apply_packet(
    received_bytes: Vec<u8>,
    tx: UnboundedSender<MpscMessage>,
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
        S2CPacket::JoinResponse(packet) => packet.apply(tx),
    }
}

pub fn handle_websocket_message_event(
    event: MessageEvent,
    tx: UnboundedSender<MpscMessage>,
) -> ControlFlow<(), ()> {
    let Ok(buffer) = event.data().dyn_into::<ArrayBuffer>() else {
        return ControlFlow::Break(());
    };
    decode_and_apply_packet(Uint8Array::new(&buffer).to_vec(), tx)
}
