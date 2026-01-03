use crate::net::readwrite::{StreamRead, StreamWrite};

impl<T: StreamRead> StreamRead for Vec<T> {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        let len: u32 = reader.try_read()?;
        let mut vec: Self = Self::new();
        for _ in 0..len {
            vec.push(reader.try_read()?);
        }
        Ok(vec)
    }
}

impl<T: StreamWrite> StreamWrite for Vec<T> {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        writer.write(self.len() as u32);
        for elem in self {
            elem.write(writer);
        }
    }
}
