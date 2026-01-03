use crate::net::primitives::{game_snapshot::GameSnapshot, position::Position, square::Square};

impl GameSnapshot {
    #[inline]
    pub fn set_square(&mut self, position: Position, square: Square) {
        self.squares.insert(position, square);
    }
    #[inline]
    pub fn remove_square(&mut self, position: Position) {
        self.squares.remove(&position);
    }
}
