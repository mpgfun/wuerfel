use crate::net::{
    primitives::{numbers::PlayerID, position::Position},
    readwrite::{StreamRead, StreamWrite},
};

#[derive(Debug)]
pub struct JoinResponseS2CPacket {
    /// If this is false, all other fields will be `None`, otherwise they will be `Some(...)`
    pub may_join: bool,
    pub player_id: Option<PlayerID>,
    pub position: Option<Position>,
}

impl StreamRead for JoinResponseS2CPacket {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self {
            may_join: reader.try_read()?,
            player_id: reader.try_read()?,
            position: reader.try_read()?,
        })
    }
}

impl StreamWrite for JoinResponseS2CPacket {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write_packet_id(1);
        writer.write(self.may_join);
        writer.write(self.player_id);
        writer.write(self.position);
    }
}
