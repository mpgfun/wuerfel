use crate::net::readwrite::{StreamRead, StreamWrite};

#[derive(Debug, Copy, Clone)]
pub struct MapConfiguration {
    pub size_x: u32,
    pub size_y: u32,
}

impl StreamRead for MapConfiguration {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self {
            size_x: reader.try_read()?,
            size_y: reader.try_read()?,
        })
    }
}

impl StreamWrite for MapConfiguration {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        self.size_x.write(writer);
        self.size_y.write(writer);
    }
}
