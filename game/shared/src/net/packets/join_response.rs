use crate::net::{
    primitives::{game_snapshot::GameSnapshot, map_config::MapConfiguration, numbers::PlayerID},
    readwrite::{StreamRead, StreamWrite},
};

#[derive(Debug)]
pub struct JoinResponseS2CPacketData {
    pub player_id: PlayerID,
    pub snapshot: GameSnapshot,
    pub map_config: MapConfiguration,
}

impl StreamRead for JoinResponseS2CPacketData {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self {
            player_id: reader.try_read()?,
            snapshot: reader.try_read()?,
            map_config: reader.try_read()?,
        })
    }
}

impl StreamWrite for JoinResponseS2CPacketData {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        self.player_id.write(writer);
        self.snapshot.write(writer);
        self.map_config.write(writer);
    }
}

#[derive(Debug)]
pub struct JoinResponseS2CPacket {
    /// If this is `None`, the player may not join.
    pub data: Option<JoinResponseS2CPacketData>,
}

impl StreamRead for JoinResponseS2CPacket {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self {
            data: reader.try_read()?,
        })
    }
}

impl StreamWrite for JoinResponseS2CPacket {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write_packet_id(1);
        self.data.write(writer);
    }
}
