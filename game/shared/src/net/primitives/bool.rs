use crate::net::readwrite::{StreamRead, StreamWrite};

impl StreamRead for bool {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(reader.try_read_byte()? != 0)
    }
}

impl StreamWrite for bool {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write_byte(*self as u8);
    }
}
