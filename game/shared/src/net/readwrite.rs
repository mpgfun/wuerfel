use std::fmt::Display;

pub trait ByteReader: Sized {
    fn read_next_byte(&mut self) -> Option<u8>;
    #[inline]
    fn read_multiple_bytes(&mut self, bytes: usize) -> Option<Vec<u8>> {
        let mut vec = Vec::with_capacity(bytes);
        for _ in 0..bytes {
            vec.push(self.read_next_byte()?);
        }
        Some(vec)
    }
    #[inline]
    fn try_read_byte(&mut self) -> Result<u8, StreamReadError> {
        match self.read_next_byte() {
            Some(data) => Ok(data),
            None => Err(StreamReadError::UnexpectedEof),
        }
    }
    #[inline]
    fn try_read<T: StreamRead>(&mut self) -> Result<T, StreamReadError> {
        T::read(self)
    }
}

pub trait ByteWriter: Sized {
    fn write_byte(&mut self, byte: u8);
    #[inline]
    fn write_multiple_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_byte(*byte);
        }
    }
    #[inline]
    fn write<T: StreamWrite>(&mut self, value: T) {
        value.write(self);
    }
    #[inline]
    fn write_ref<T: StreamWrite>(&mut self, value: &T) {
        value.write(self);
    }
}

pub enum StreamReadError {
    UnexpectedEof,
    MalformedData,
    UnknownPacketId(u8),
}

impl Display for StreamReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("Unexpected end of stream"),
            Self::MalformedData => f.write_str("Received malformed data"),
            Self::UnknownPacketId(id) => f.write_fmt(format_args!("Unknown packet id {id}")),
        }
    }
}

pub trait StreamRead: Sized {
    fn read(reader: &mut impl ByteReader) -> Result<Self, StreamReadError>;
}

pub trait StreamWrite {
    fn write(&self, writer: &mut impl ByteWriter);
}
