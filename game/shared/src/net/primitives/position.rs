use crate::net::readwrite::{StreamRead, StreamWrite};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl StreamRead for Position {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self {
            x: reader.try_read()?,
            y: reader.try_read()?,
        })
    }
}

impl StreamWrite for Position {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write(self.x);
        writer.write(self.y);
    }
}
