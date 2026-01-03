use std::ops::ControlFlow;

use rand::random_range;
use shared::net::primitives::{
    game_snapshot::GameSnapshot, map_config::MapConfiguration, numbers::PlayerID,
    position::Position, square::Square,
};

pub trait GameSnapshotExt {
    async fn find_starting_square(&self, map_config: MapConfiguration) -> Option<Position>;
    async fn spawn_new_player(
        &mut self,
        player_id: PlayerID,
        map_config: MapConfiguration,
    ) -> ControlFlow<()>;
}

impl GameSnapshotExt for GameSnapshot {
    async fn find_starting_square(&self, map_config: MapConfiguration) -> Option<Position> {
        let mut iteration = 0;
        loop {
            let rand_x = random_range(0..map_config.size_x);
            let rand_y = random_range(0..map_config.size_y);
            let position = Position::new(rand_x, rand_y);
            if !self.squares.contains_key(&position) {
                break Some(position);
            }
            iteration += 1;
            if iteration > 100 {
                break None;
            }
        }
    }

    async fn spawn_new_player(
        &mut self,
        player_id: PlayerID,
        map_config: MapConfiguration,
    ) -> ControlFlow<()> {
        let starting_square = self.find_starting_square(map_config).await;
        let Some(starting_square) = starting_square else {
            return ControlFlow::Break(());
        };
        self.players.insert(
            player_id,
            (format!("Player {}", player_id), String::from("#ff0000")),
        );
        self.squares.insert(
            starting_square,
            Square {
                num: 1,
                owner: player_id,
            },
        );
        ControlFlow::Continue(())
    }
}
