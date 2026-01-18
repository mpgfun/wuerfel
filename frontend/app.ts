import { GameBoard } from './GameBoard';
import { WebSocketManager } from './WebSocketManager';

const canvas = document.createElement('canvas');
canvas.width = window.innerWidth;
canvas.height = window.innerHeight;
canvas.style.position = 'absolute';
canvas.style.top = '0';
canvas.style.left = '0';
canvas.style.width = '100%';
canvas.style.height = '100%';
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

    console.log("pos: " + boardX + " " + boardY);
    if (boardX >= 0 && boardY >= 0 && boardX < boardSize && boardY < boardSize) {
        const data = {
            position: { x: boardX, y: boardY },
        };


        socketManager.sendMessage({
            type: "click",
            data: data,
        });
    }
});

window.addEventListener('resize', () => {
    gameBoard.resizeCanvas();
});

function loop() {
    requestAnimationFrame(loop);
    gameBoard.drawBoard();
}

loop();