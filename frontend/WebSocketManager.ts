import { gameBoard } from "./app";
import { Position, Square } from "./GameBoard";

export interface WebSocketClickMessage {
  type: "click";
  data: {
    position: {
      x: number;
      y: number;
    };
  };
}

export type SquareChange = [
  Position,
  {
    id: number | null;
    number: number;
  }
];

export interface GameConfig {
  size: number;
  max_number: number;
}

export interface GameSnapshot {
  players: [number, [number, number, number]][];
  squares: [Position, Square][];
}

export interface LoginDataS2CMessage {
  id: number;
  color: [number, number, number];
  spawn_point: Position;
  config: GameConfig;
  snapshot: GameSnapshot;
}

export class WebSocketManager {
  private socket: WebSocket;

  constructor(url: string) {
    this.socket = new WebSocket(url);

    this.socket.addEventListener("open", () => {
      console.log("WebSocket connection established");
    });

    this.socket.addEventListener("message", (event) => {
      let data;
      try {
        data = JSON.parse(event.data);
      } catch (e) {
        console.error(e);
        console.error("JSON parse error parsing " + event.data);
        return;
      }
      if (data.changes !== undefined) {
        let changes = data.changes as SquareChange[];
        if (changes.length === 0) {
          return;
        }
        gameBoard.onChanges(changes);
      } else if (data.id !== undefined) {
        const loginData = data as LoginDataS2CMessage;
        console.log("Received login data:");
        console.log(loginData);
        gameBoard.players = loginData.snapshot.players.map((value) => {
          return {
            id: value[0],
            color: value[1],
          };
        });
        gameBoard.squares = loginData.snapshot.squares.map((value) => {
          return {
            pos: value[0],
            square: value[1],
          };
        });
        gameBoard.myID = loginData.id;
        gameBoard.boardSize = loginData.config.size * 10;
        gameBoard.squareSize = loginData.config.size * 0.05 * 10;
      }
    });

    this.socket.addEventListener("close", () => {
      console.log("WebSocket connection closed");
    });

    this.socket.addEventListener("error", (error) => {
      console.error("WebSocket error:", error);
    });
  }

  public sendMessage(message: WebSocketClickMessage) {
    this.socket.send(JSON.stringify(message));
  }
}
