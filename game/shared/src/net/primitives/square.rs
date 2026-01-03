use crate::net::{
    primitives::numbers::PlayerID,
    readwrite::{StreamRead, StreamWrite},
};

#[derive(Debug, Clone, Copy)]
pub struct Square {
    pub num: u8,
    pub owner: PlayerID,
}

impl StreamRead for Square {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self {
            num: reader.try_read()?,
            owner: reader.try_read()?,
        })
    }
}

impl StreamWrite for Square {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        self.num.write(writer);
        self.owner.write(writer);
    }
}
