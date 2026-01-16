import { GameBoard } from './GameBoard';
import { WebSocketManager } from './WebSocketManager';

const canvas = document.createElement('canvas');
canvas.width = window.innerWidth;
canvas.height = window.innerHeight;
document.body.appendChild(canvas);

export const gameBoard = new GameBoard(canvas);

const socketManager = new WebSocketManager('ws://localhost:3000/ws');

canvas.addEventListener('click', (event) => {
    const rect = canvas.getBoundingClientRect();
    const clickX = (event.clientX - rect.left) / gameBoard.scale + gameBoard.offsetX;
    const clickY = (event.clientY - rect.top) / gameBoard.scale + gameBoard.offsetY;

    let squareSize = gameBoard.squareSize;
    let boardSize = gameBoard.boardSize;

    const boardX = Math.floor(clickX / squareSize);
    const boardY = Math.floor(clickY / squareSize);

    if (boardX >= 0 && boardY >= 0 && boardX < boardSize / squareSize && boardY < boardSize / squareSize) {
        const data = {
            position: { x: boardX, y: boardY },
        };

        socketManager.sendMessage({
            type: "click",
            data: data,
        });
    }
});

function loop() {
    requestAnimationFrame(loop);
    gameBoard.drawBoard();
}

loop();