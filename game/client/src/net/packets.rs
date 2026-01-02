use std::{cell::RefCell, ops::ControlFlow, rc::Rc};

use shared::net::{packets::join_response::JoinResponseS2CPacket, readwrite::StreamWrite};
use web_sys::WebSocket;

use crate::{ClientGameState, net::ClientByteWriter};

pub struct Sender<'a> {
    ws: &'a mut WebSocket,
}

impl<'a> Sender<'a> {
    pub fn new(ws: &'a mut WebSocket) -> Self {
        Self { ws }
    }

    pub fn send<T: StreamWrite>(&mut self, packet: T) {
        let mut writer = ClientByteWriter::new();
        packet.write(&mut writer);
        self.ws
            .send_with_u8_array(writer.destroy().as_slice())
            .unwrap();
    }
}

pub trait ClientPacketHandler {
    fn apply(
        &self,
        state: Rc<RefCell<ClientGameState>>,
        socket: &mut WebSocket,
    ) -> ControlFlow<(), ()>;
}

impl ClientPacketHandler for JoinResponseS2CPacket {
    fn apply(
        &self,
        _state: Rc<RefCell<ClientGameState>>,
        _socket: &mut WebSocket,
    ) -> ControlFlow<(), ()> {
        if !self.may_join {
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }
}
