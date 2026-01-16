use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Square {
    pub owner: PlayerID,
    pub number: u8,
}

#[derive(Serialize, Copy, Clone)]
pub struct GameConfig {
    pub size: u32,
    pub max_number: u8,
}

pub type PlayerID = u16;

pub type Color = (u8, u8, u8);

#[derive(Serialize)]
pub struct GameSnapshot {
    pub players: Vec<(PlayerID, Color)>,
    pub squares: Vec<(Position, Square)>,
}

#[derive(Serialize)]
pub struct LoginDataS2CMessage {
    pub id: PlayerID,
    pub color: Color,
    pub spawn_point: Position,
    pub config: GameConfig,
    pub snapshot: GameSnapshot,
}

#[derive(Deserialize)]
pub struct ClickC2SMessage {
    pub position: Position,
}

#[derive(Serialize, Copy, Clone, Debug)]
pub struct SquareChange {
    /// `None` if the square was removed
    pub id: Option<PlayerID>,
    /// Ignored if the square has no owner
    pub number: u8,
}

impl From<Square> for SquareChange {
    fn from(value: Square) -> Self {
        Self {
            id: Some(value.owner),
            number: value.number,
        }
    }
}

impl SquareChange {
    pub fn create_removed() -> Self {
        Self {
            id: None,
            number: 0,
        }
    }
}

#[derive(Serialize)]
pub struct TickS2CMessage {
    pub changes: Vec<(Position, SquareChange)>,
}
