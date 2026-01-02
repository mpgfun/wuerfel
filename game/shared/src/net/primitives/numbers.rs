use crate::net::readwrite::{StreamRead, StreamWrite};

impl StreamRead for u8 {
    #[inline]
    fn read(reader: &mut impl crate::net::readwrite::ByteReader) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(reader.try_read_byte()?)
    }
}

impl StreamWrite for u8 {
    #[inline]
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write_byte(*self);
    }
}

impl StreamRead for i32 {
    #[inline]
    fn read(reader: &mut impl crate::net::readwrite::ByteReader) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self::from_be_bytes([
            reader.try_read_byte()?,
            reader.try_read_byte()?,
            reader.try_read_byte()?,
            reader.try_read_byte()?,
        ]))
    }
}

impl StreamWrite for i32 {
    #[inline]
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write_multiple_bytes(&self.to_be_bytes());
    }
}

pub type PlayerID = u64;

impl StreamRead for u64 {
    #[inline]
    fn read(reader: &mut impl crate::net::readwrite::ByteReader) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self::from_be_bytes([
            reader.try_read_byte()?,
            reader.try_read_byte()?,
            reader.try_read_byte()?,
            reader.try_read_byte()?,
            reader.try_read_byte()?,
            reader.try_read_byte()?,
            reader.try_read_byte()?,
            reader.try_read_byte()?,
        ]))
    }
}

impl StreamWrite for u64 {
    #[inline]
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write_multiple_bytes(&self.to_be_bytes());
    }
}
