import { gameBoard } from "./app";
import { Position } from "./GameBoard";

export interface WebSocketClickMessage {
    type: "click";
    data: {
        position: {
            x: number;
            y: number;
        };
    };
}

export type SquareChange = [Position, {
    id: number | null,
    number: number,
}];

export class WebSocketManager {
    private socket: WebSocket;

    constructor(url: string) {
        this.socket = new WebSocket(url);

        this.socket.addEventListener('open', () => {
            console.log('WebSocket connection established');
        });

        this.socket.addEventListener('message', (event) => {
            // console.log(event.data);
            let data;
            try {
                data = JSON.parse(event.data);
            } catch (e) {
                console.log("JSON parse error parsing" + event.data);
                return;
            }
            if (data.changes !== undefined) {
                let changes = data.changes as SquareChange[];
                if (changes.length === 0) {
                    return;
                }
                gameBoard.onChanges(changes);
            }
        });

        this.socket.addEventListener('close', () => {
            console.log('WebSocket connection closed');
        });

        this.socket.addEventListener('error', (error) => {
            console.error('WebSocket error:', error);
        });
    }

    public sendMessage(message: WebSocketClickMessage) {
        this.socket.send(JSON.stringify(message));
    }
}