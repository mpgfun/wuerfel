import { SquareChange } from "./WebSocketManager";

export interface Position {
    x: number,
    y: number,
}

export interface Square {
    owner: number,
    number: number,
}

export class GameBoard {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;
    private boardSize: number;
    private squareSize: number;
    public offsetX: number;
    public offsetY: number;
    public scale: number;
    private clickedSquares: Set<string>;
    public squares: {
        pos: Position,
        square: Square,
    }[];
    private isDragging: boolean;
    private dragStartX: number | undefined;
    private dragStartY: number | undefined;

    constructor(canvas: HTMLCanvasElement, boardSize: number, squareSize: number) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d')!;
        this.boardSize = boardSize;
        this.squareSize = squareSize;
        this.offsetX = 0;
        this.offsetY = 0;
        this.scale = 1;
        this.clickedSquares = new Set();
        this.dragStartX = undefined;
        this.dragStartY = undefined;
        this.isDragging = false;
        this.squares = [];

        this.setupEventListeners();
    }

    private setupEventListeners() {
        this.canvas.addEventListener('mousedown', this.onMouseDown.bind(this));
        this.canvas.addEventListener('mousemove', this.onMouseMove.bind(this));
        this.canvas.addEventListener('mouseup', this.onMouseUp.bind(this));
        this.canvas.addEventListener('mouseleave', this.onMouseUp.bind(this));
        this.canvas.addEventListener('wheel', this.onWheel.bind(this));
        this.canvas.addEventListener('click', this.onClick.bind(this));
    }

    private onMouseDown(event: MouseEvent) {
        this.isDragging = true;
        this.dragStartX = event.clientX;
        this.dragStartY = event.clientY;
    }

    private onMouseMove(event: MouseEvent) {
        if (this.isDragging) {
            this.dragStartX ||= event.clientX;
            this.dragStartY ||= event.clientY;
            this.offsetX -= (event.clientX - this.dragStartX) / this.scale;
            this.offsetY -= (event.clientY - this.dragStartY) / this.scale;
            this.dragStartX = event.clientX;
            this.dragStartY = event.clientY;
            this.drawBoard();
        }
    }

    private onMouseUp() {
        this.isDragging = false;
    }

    private onWheel(event: WheelEvent) {
        event.preventDefault();

        const zoomIntensity = 0.1;
        const rect = this.canvas.getBoundingClientRect();
        const mouseX = (event.clientX - rect.left) / this.scale + this.offsetX;
        const mouseY = (event.clientY - rect.top) / this.scale + this.offsetY;

        const zoom = event.deltaY < 0 ? 1 + zoomIntensity : 1 - zoomIntensity;

        const newScale = this.scale * zoom;

        if (newScale < 0.5 || newScale > 3) return;

        this.offsetX = mouseX - (mouseX - this.offsetX) * zoom;
        this.offsetY = mouseY - (mouseY - this.offsetY) * zoom;

        this.scale = newScale;
        this.drawBoard();
    }

    private onClick(event: MouseEvent) {
        const rect = this.canvas.getBoundingClientRect();
        const clickX = (event.clientX - rect.left) / this.scale + this.offsetX;
        const clickY = (event.clientY - rect.top) / this.scale + this.offsetY;

        const boardX = Math.floor(clickX / this.squareSize);
        const boardY = Math.floor(clickY / this.squareSize);

        if (boardX >= 0 && boardY >= 0 && boardX < this.boardSize / this.squareSize && boardY < this.boardSize / this.squareSize) {
            const squareKey = `${boardX},${boardY}`;

            if (this.clickedSquares.has(squareKey)) {
                this.clickedSquares.delete(squareKey);
            } else {
                this.clickedSquares.add(squareKey);
            }

            this.drawBoard();
        }
    }

    public drawBoard() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.save();
        this.ctx.scale(this.scale, this.scale);
        this.ctx.translate(-this.offsetX, -this.offsetY);

        for (let x = 0; x < this.boardSize; x += this.squareSize) {
            for (let y = 0; y < this.boardSize; y += this.squareSize) {
                const boardX = x / this.squareSize;
                const boardY = y / this.squareSize;

                const squareMatch = this.squares.filter((value) => value.pos.x === boardX && value.pos.y === boardY);
                let square: {pos: Position, square: Square} | undefined;
                if (squareMatch.length === 0) {
                    square = undefined;
                } else {
                    square = squareMatch[0];
                }
                if (square) {
                    this.ctx.fillStyle = '#e74c3c';
                    this.ctx.fillRect(x, y, this.squareSize - 2, this.squareSize - 2);

                    // Draw the number on top of the square
                    this.ctx.fillStyle = '#ffffff';
                    this.ctx.font = `${this.squareSize / 2}px Arial`;
                    this.ctx.textAlign = 'center';
                    this.ctx.textBaseline = 'middle';
                    this.ctx.fillText(
                        square.square.number.toString(),
                        x + this.squareSize / 2,
                        y + this.squareSize / 2
                    );
                } else {
                    this.ctx.fillStyle = '#3498db';
                    this.ctx.fillRect(x, y, this.squareSize - 2, this.squareSize - 2);
                }
            }
        }

        this.ctx.restore();
    }

    public onChanges(changes: SquareChange[]) {
        console.log("changes received: " + JSON.stringify(changes));
        for (const change of changes) {
            const pos = change[0];
            let index = this.squares.findIndex(value => value.pos.x == pos.x && value.pos.y == pos.y);
            if (change[1].id !== null) {
                if (index === -1) {
                    this.squares.push({
                        pos: pos,
                        square: {
                            owner: change[1].id,
                            number: change[1].number,
                        },
                    });
                } else {
                    this.squares[index] = {
                        pos: pos,
                        square: {
                            owner: change[1].id,
                            number: change[1].number,
                        },
                    };
                }
            } else {
                if (index !== -1) {
                    this.squares.splice(index, 1);
                }
            }
        }
        console.log("squares:");
        console.log(this.squares);
    }
}