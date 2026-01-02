use crate::net::readwrite::{StreamRead, StreamWrite};

pub struct JoinC2SPacket {
    pub lobby_id: i32,
}

impl StreamRead for JoinC2SPacket {
    fn read(reader: &mut impl crate::net::readwrite::ByteReader) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self {
            lobby_id: reader.try_read()?,
        })
    }
}

impl StreamWrite for JoinC2SPacket {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write(self.lobby_id);
    }
}