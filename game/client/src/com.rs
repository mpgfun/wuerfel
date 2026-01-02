use shared::net::packets::C2SPacket;

pub enum MpscMessage {
    CreateOnMessage,
    SendPacket(C2SPacket),
}
