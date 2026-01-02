use crate::net::{
    packets::{join::JoinC2SPacket, join_response::JoinResponseS2CPacket},
    readwrite::{ByteReader, StreamRead, StreamReadError},
};

pub mod join;
pub mod join_response;

pub enum C2SPacket {
    Join(JoinC2SPacket),
}

pub enum S2CPacket {
    JoinResponse(JoinResponseS2CPacket),
}

impl C2SPacket {
    pub fn read(reader: &mut impl ByteReader) -> Result<Self, StreamReadError> {
        let packet_id: u8 = reader.try_read()?;
        match packet_id {
            1 => Ok(Self::Join(JoinC2SPacket::read(reader)?)),
            _ => Err(StreamReadError::UnknownPacketId(packet_id)),
        }
    }
}

impl S2CPacket {
    pub fn read(reader: &mut impl ByteReader) -> Result<Self, StreamReadError> {
        let packet_id: u8 = reader.try_read()?;
        match packet_id {
            1 => Ok(Self::JoinResponse(JoinResponseS2CPacket::read(reader)?)),
            _ => Err(StreamReadError::UnknownPacketId(packet_id)),
        }
    }
}
