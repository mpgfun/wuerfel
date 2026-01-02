use crate::net::{
    packets::{join::JoinC2SPacket, join_response::JoinResponseS2CPacket},
    readwrite::{ByteReader, StreamRead, StreamReadError, StreamWrite},
};

pub mod join;
pub mod join_response;

pub enum C2SPacket {
    Join(JoinC2SPacket),
}

pub enum S2CPacket {
    JoinResponse(JoinResponseS2CPacket),
}

impl StreamRead for C2SPacket {
    fn read(reader: &mut impl ByteReader) -> Result<Self, StreamReadError> {
        let packet_id: u8 = reader.try_read()?;
        match packet_id {
            1 => Ok(Self::Join(JoinC2SPacket::read(reader)?)),
            _ => Err(StreamReadError::UnknownPacketId(packet_id)),
        }
    }
}

impl StreamWrite for C2SPacket {
    fn write(&self, writer: &mut impl super::readwrite::ByteWriter) {
        match self {
            Self::Join(packet) => packet.write(writer),
        }
    }
}

impl StreamRead for S2CPacket {
    fn read(reader: &mut impl ByteReader) -> Result<Self, StreamReadError> {
        let packet_id: u8 = reader.try_read()?;
        match packet_id {
            1 => Ok(Self::JoinResponse(JoinResponseS2CPacket::read(reader)?)),
            _ => Err(StreamReadError::UnknownPacketId(packet_id)),
        }
    }
}

impl StreamWrite for S2CPacket {
    fn write(&self, writer: &mut impl super::readwrite::ByteWriter) {
        match self {
            Self::JoinResponse(packet) => packet.write(writer),
        }
    }
}
