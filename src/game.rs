use std::{collections::HashMap, ops::ControlFlow};

use tokio::task::JoinHandle;
use warp::filters::ws::{Message, WebSocket};

use crate::{
    game::player::{Player, PlayerCommand, generate_random_color},
    schemas::{
        Color, GameConfig, LoginDataS2CMessage, PlayerID, PlayerJoinS2CMessage, Position, Square,
        SquareChange, TickS2CMessage,
    },
};

mod player;

pub enum ServerCommand {
    PlayerClick(PlayerID, Position),
    Tick,
    #[allow(unused)]
    Stop,
    /// Box<T> due to large size
    AddPlayer(Box<WebSocket>),
}

type Squares = HashMap<Position, Square>;
type Players = HashMap<PlayerID, (Color, tokio::sync::mpsc::Sender<PlayerCommand>)>;
pub type ServerSender = tokio::sync::mpsc::Sender<ServerCommand>;
pub type SquareChanges = HashMap<Position, SquareChange>;

pub struct GameState {
    players: Players,
    squares: Squares,
    config: GameConfig,
    pub rx: tokio::sync::mpsc::Receiver<ServerCommand>,
    pub tx: tokio::sync::mpsc::Sender<ServerCommand>,
    pub tps: u8,
    click_queue: Vec<(PlayerID, Position)>,
    square_changes: SquareChanges,
}

impl GameState {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        Self {
            players: HashMap::new(),
            squares: HashMap::new(),
            rx,
            tx,
            tps: 60,
            click_queue: Vec::with_capacity(64),
            config: GameConfig {
                size: 100,
                max_number: 4,
            },
            square_changes: SquareChanges::new(),
        }
    }

    pub async fn handle_message(&mut self, message: ServerCommand) -> ControlFlow<()> {
        match message {
            ServerCommand::PlayerClick(id, pos) => self.click_queue.push((id, pos)),
            ServerCommand::Tick => self.tick().await,
            ServerCommand::Stop => return ControlFlow::Break(()),
            ServerCommand::AddPlayer(ws) => {
                self.add_player(ws).await;
            }
        }
        ControlFlow::Continue(())
    }

    async fn tick(&mut self) {
        self.apply_clicks();
        let square_changes = &mut self.square_changes;
        let _ = Self::broadcast(
            &mut self.players,
            Message::text(
                serde_json::to_string(&TickS2CMessage {
                    changes: square_changes.drain().collect(),
                })
                .unwrap(),
            ),
        )
        .await;
    }

    fn apply_clicks(&mut self) {
        for click in self.click_queue.drain(..) {
            Self::apply_click(
                &mut self.squares,
                &mut self.square_changes,
                self.config,
                click,
            );
        }
    }

    fn apply_click(
        squares: &mut Squares,
        square_changes: &mut SquareChanges,
        config: GameConfig,
        click: (PlayerID, Position),
    ) {
        let Some(square) = squares.get_mut(&click.1) else {
            squares.insert(
                click.1,
                Square {
                    owner: click.0,
                    number: 1,
                },
            );
            return;
        };
        if square.owner != click.0 {
            return;
        }
        square.number += 1;
        if square.number > config.max_number {
            Self::expand_square(squares, square_changes, config, click.1);
        } else {
            square_changes.insert(click.1, (*square).into());
        }
    }

    /// # Panics
    /// If the `pos` has no owner (meaning it isn't present in the `squares` HashMap)
    fn expand_square(
        squares: &mut Squares,
        square_changes: &mut SquareChanges,
        config: GameConfig,
        pos: Position,
    ) {
        let Some(origin_square) = squares.remove(&pos) else {
            // this shouldn't happen if the function is called correctly
            panic!("Square to expand has no owner.");
        };

        square_changes.insert(pos, SquareChange::create_removed());

        let adjacent_squares = Self::adjacent_squares(config, pos);
        for adjacent_square_pos in adjacent_squares {
            if let Some(adjacent_square) = squares.get_mut(&adjacent_square_pos) {
                adjacent_square.owner = origin_square.owner;
                adjacent_square.number += 1;
                if adjacent_square.number > config.max_number {
                    Self::expand_square(squares, square_changes, config, adjacent_square_pos);
                } else {
                    square_changes.insert(adjacent_square_pos, (*adjacent_square).into());
                }
            } else {
                let square = Square {
                    owner: origin_square.owner,
                    number: 1,
                };
                squares.insert(adjacent_square_pos, square);
                square_changes.insert(adjacent_square_pos, square.into());
            }
        }
    }

    fn adjacent_squares(config: GameConfig, pos: Position) -> Vec<Position> {
        // wrapping_add and wrapping_sub assuming the map size will never be u32::MAX-1 or u32::MAX (which *should* never be the case anyways)
        // so that values <0 will just be wrapped to u32::MAX so that they can be filtered out because they are outside of the map
        let adjacent = [
            Position {
                x: pos.x.wrapping_sub(1),
                y: pos.y.wrapping_sub(0),
            },
            Position {
                x: pos.x.wrapping_sub(0),
                y: pos.y.wrapping_sub(1),
            },
            Position {
                x: pos.x.wrapping_add(1),
                y: pos.y.wrapping_add(0),
            },
            Position {
                x: pos.x.wrapping_add(0),
                y: pos.y.wrapping_add(1),
            },
        ];
        let mut vec = Vec::new();
        for pos in adjacent {
            // Since the x and y are unsigned, they will never be less than 0
            if pos.x < config.size && pos.y < config.size {
                vec.push(pos);
            }
        }
        vec
    }

    async fn add_player(&mut self, ws: Box<WebSocket>) -> JoinHandle<()> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let player = Player::new(ws, rx);
        let color = generate_random_color();
        self.players.insert(player.id, (color, tx.clone()));
        let tx_clone = self.tx.clone();
        let config = self.config;
        let squares_clone = self.squares.clone();
        let players_clone = self.players.clone();
        tx.send(PlayerCommand::SendMessage(Message::text(
            serde_json::to_string(&LoginDataS2CMessage {
                id: player.id,
                color,
                spawn_point: crate::schemas::Position { x: 0, y: 0 },
                snapshot: crate::schemas::GameSnapshot {
                    players: players_clone
                        .iter()
                        .map(|elem| (*elem.0, elem.1.0))
                        .collect(),
                    squares: squares_clone
                        .iter()
                        .map(|elem| (*elem.0, *elem.1))
                        .collect(),
                },
                config,
            })
            .unwrap(),
        )))
        .await
        .unwrap();

        Self::broadcast(
            &mut self.players,
            Message::text(
                serde_json::to_string(&PlayerJoinS2CMessage {
                    player_join: (player.id, color),
                })
                .unwrap(),
            ),
        )
        .await
        .unwrap();

        tokio::spawn(async move {
            let disconnect_reason = player.handle_connection(tx_clone).await;
            if let Err(reason) = disconnect_reason {
                println!("Player disconnected: {:?}", reason);
            }
        })
    }

    /// Returns `Err` if either a JoinError occurred, or if a SendError occurred
    async fn broadcast(players: &mut Players, message: Message) -> Result<(), ()> {
        let mut tasks: Vec<JoinHandle<Option<PlayerID>>> = Vec::new();

        // tokio::spawn allows running all send()s concurrently here
        for player in &mut *players {
            let message = message.clone();
            let tx = player.1.1.clone();
            let id = *player.0;
            tasks.push(tokio::spawn(async move {
                if tx.send(PlayerCommand::SendMessage(message)).await.is_err() {
                    Some(id)
                } else {
                    None
                }
            }));
        }

        for task in tasks {
            match task.await {
                Ok(result) => {
                    if let Some(id) = result {
                        // disconnect player on error
                        players.remove(&id);
                    }
                }
                Err(_) => return Err(()),
            }
        }

        Ok(())
    }
}
