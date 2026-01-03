use std::collections::HashMap;

use crate::net::{
    primitives::{color::PlayerColor, numbers::PlayerID, position::Position, square::Square},
    readwrite::{StreamRead, StreamWrite},
};

#[derive(Debug, Clone)]
pub struct GameSnapshot {
    pub players: HashMap<PlayerID, (String, PlayerColor)>,
    pub squares: HashMap<Position, Square>,
}

impl GameSnapshot {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            squares: HashMap::new(),
        }
    }
}

impl StreamRead for (PlayerID, String, PlayerColor) {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok((reader.try_read()?, reader.try_read()?, reader.try_read()?))
    }
}

impl StreamWrite for (PlayerID, String, PlayerColor) {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        self.0.write(writer);
        self.1.write(writer);
        self.2.write(writer);
    }
}

impl StreamRead for (Position, Square) {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok((reader.try_read()?, reader.try_read()?))
    }
}

impl StreamWrite for (Position, Square) {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        self.0.write(writer);
        self.1.write(writer);
    }
}

impl StreamRead for GameSnapshot {
    fn read(
        reader: &mut impl crate::net::readwrite::ByteReader,
    ) -> Result<Self, crate::net::readwrite::StreamReadError> {
        Ok(Self {
            players: reader
                .try_read::<Vec<(PlayerID, String, PlayerColor)>>()?
                .into_hashmap(),
            squares: reader.try_read::<Vec<(Position, Square)>>()?.into_hashmap(),
        })
    }
}

impl StreamWrite for GameSnapshot {
    fn write(&self, writer: &mut impl crate::net::readwrite::ByteWriter) {
        self.players.into_vec().write(writer);
        self.squares.into_vec().write(writer);
    }
}

trait HashMapExt {
    fn into_vec(&self) -> Vec<(PlayerID, String, PlayerColor)>;
}
trait HashMapExt2 {
    fn into_vec(&self) -> Vec<(Position, Square)>;
}

trait VecExt {
    fn into_hashmap(&self) -> HashMap<PlayerID, (String, PlayerColor)>;
}
trait VecExt2 {
    fn into_hashmap(&self) -> HashMap<Position, Square>;
}

impl HashMapExt for HashMap<PlayerID, (String, PlayerColor)> {
    fn into_vec(&self) -> Vec<(PlayerID, String, PlayerColor)> {
        let mut vec = Vec::new();
        for elem in self {
            vec.push((*elem.0, elem.1.0.clone(), elem.1.1.clone()));
        }
        vec
    }
}
impl HashMapExt2 for HashMap<Position, Square> {
    fn into_vec(&self) -> Vec<(Position, Square)> {
        let mut vec = Vec::new();
        for elem in self {
            vec.push((*elem.0, *elem.1));
        }
        vec
    }
}

impl VecExt for Vec<(PlayerID, String, PlayerColor)> {
    fn into_hashmap(&self) -> HashMap<PlayerID, (String, PlayerColor)> {
        let mut hashmap = HashMap::new();
        for elem in self {
            hashmap.insert(elem.0, (elem.1.clone(), elem.2.clone()));
        }
        hashmap
    }
}
impl VecExt2 for Vec<(Position, Square)> {
    fn into_hashmap(&self) -> HashMap<Position, Square> {
        let mut hashmap = HashMap::new();
        for elem in self {
            hashmap.insert(elem.0, elem.1);
        }
        hashmap
    }
}
