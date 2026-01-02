use crate::net::readwrite::{StreamRead, StreamWrite};

impl<T: StreamRead> StreamRead for Option<T> {
    fn read(reader: &mut impl crate::net::readwrite::ByteReader) -> Result<Self, crate::net::readwrite::StreamReadError> {
        let is_some: bool = reader.try_read()?;
        if is_some {
            Ok(Some(reader.try_read()?))
        } else {
            Ok(None)
        }
    }
}

impl<T: StreamWrite> StreamWrite for Option<T> {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        match self {
            Some(value) => {
                writer.write(true);
                writer.write_ref(value);
            }
            None => {
                writer.write(false);
            }
        }
    }
}