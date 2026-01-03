use crate::net::readwrite::{StreamRead, StreamWrite};

impl StreamRead for String {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        let length: u16 = reader.try_read()?;
        let bytes = reader.read_multiple_bytes(length as usize);
        let Some(bytes) = bytes else {
            return Err(crate::net::readwrite::StreamReadError::UnexpectedEof);
        };
        match Self::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(crate::net::readwrite::StreamReadError::MalformedData),
        }
    }
}

impl StreamWrite for String {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        let bytes = self.as_bytes();
        writer.write(bytes.len() as u16);
        writer.write_multiple_bytes(bytes);
    }
}
