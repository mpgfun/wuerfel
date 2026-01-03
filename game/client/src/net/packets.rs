use std::ops::ControlFlow;

use futures::channel::mpsc::UnboundedSender;
use shared::net::packets::join_response::JoinResponseS2CPacket;

use crate::{ClientGame, com::MpscMessage, console_log};

pub trait ClientPacketHandler {
    fn apply(self, tx: UnboundedSender<MpscMessage>) -> ControlFlow<(), ()>;
}

impl ClientPacketHandler for JoinResponseS2CPacket {
    fn apply(self, tx: UnboundedSender<MpscMessage>) -> ControlFlow<(), ()> {
        let Some(data) = self.data else {
            return ControlFlow::Break(());
        };
        console_log!("Data from join: {:?}", data);
        let _ = tx.unbounded_send(MpscMessage::MutateClientState(Box::new(|state| {
            state.game = Some(ClientGame::new(data));
        })));
        ControlFlow::Continue(())
    }
}
