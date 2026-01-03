use std::{cell::RefCell, ops::ControlFlow, rc::Rc};

use shared::net::{packets::join_response::JoinResponseS2CPacket, readwrite::StreamWrite};
use web_sys::WebSocket;

use crate::{ClientGame, ClientState, console_log, net::ClientByteWriter};

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
    fn apply(self, state: Rc<RefCell<ClientState>>, socket: &mut WebSocket) -> ControlFlow<(), ()>;
}

#[allow(unused)]
pub trait ClientPacketHandlerBorrow {
    fn apply(&self, state: Rc<RefCell<ClientState>>, socket: &mut WebSocket)
    -> ControlFlow<(), ()>;
}
#[allow(unused)]
pub trait ClientPacketHandlerBorrowMut {
    fn apply(
        &mut self,
        state: Rc<RefCell<ClientState>>,
        socket: &mut WebSocket,
    ) -> ControlFlow<(), ()>;
}

impl ClientPacketHandler for JoinResponseS2CPacket {
    fn apply(
        self,
        state: Rc<RefCell<ClientState>>,
        _socket: &mut WebSocket,
    ) -> ControlFlow<(), ()> {
        let Some(data) = self.data else {
            state.borrow_mut().should_close = true;
            return ControlFlow::Break(());
        };
        console_log!("Data from join: {:?}", data);
        state.borrow_mut().game = Some(ClientGame::new(data));
        ControlFlow::Continue(())
    }
}
