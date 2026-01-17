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
    public boardSize: number;
    public squareSize: number;
    public offsetX: number;
    public offsetY: number;
    public scale: number;
    public squares: {
        pos: Position,
        square: Square,
    }[];
    public players: {
        id: number,
        color: [number, number, number],
    }[];
    private isDragging: boolean;
    private dragStartX: number | undefined;
    private dragStartY: number | undefined;
    public myID: number | undefined;

    constructor(canvas: HTMLCanvasElement) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d')!;
        this.boardSize = 0;
        this.squareSize = 0;
        this.offsetX = 0;
        this.offsetY = 0;
        this.scale = 10;
        this.dragStartX = undefined;
        this.dragStartY = undefined;
        this.isDragging = false;
        this.squares = [];
        this.players = [];
        this.myID = undefined;

        this.setupEventListeners();
    }

    private setupEventListeners() {
        this.canvas.addEventListener('mousedown', this.onMouseDown.bind(this));
        this.canvas.addEventListener('mousemove', this.onMouseMove.bind(this));
        this.canvas.addEventListener('mouseup', this.onMouseUp.bind(this));
        this.canvas.addEventListener('mouseleave', this.onMouseUp.bind(this));
        this.canvas.addEventListener('wheel', this.onWheel.bind(this));
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

        let zoom = event.deltaY < 0 ? 1 + zoomIntensity : 1 - zoomIntensity;

        const newScale = this.scale * zoom;

        if (newScale < 5 || newScale > 10) return;

        this.offsetX = mouseX - (mouseX - this.offsetX) * zoom;
        this.offsetY = mouseY - (mouseY - this.offsetY) * zoom;

        this.scale = newScale;
        this.drawBoard();
    }

    public drawBoard() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.save();
        this.ctx.scale(this.scale, this.scale);
        this.ctx.translate(-this.offsetX, -this.offsetY);


        let borders = 1 / this.scale;

        for (let boardX = 0; boardX < this.boardSize; boardX++) {
            for (let boardY = 0; boardY < this.boardSize; boardY++) {
                const x = boardX * this.squareSize;
                const y = boardY * this.squareSize;

                const squareMatch = this.squares.filter((value) => value.pos.x === boardX && value.pos.y === boardY);
                let square: {pos: Position, square: Square} | undefined;
                if (squareMatch.length === 0) {
                    square = undefined;
                } else {
                    square = squareMatch[0];
                }
                if (square) {
                    console.log("drawing square");
                    let owner = square.square.owner;
                    let ownerColor = this.players.find(player => player.id === owner)?.color;
                    if (ownerColor === undefined) {
                        console.error("Out of sync!");
                        continue;
                    }
                    this.ctx.fillStyle = `rgb(${ownerColor[0]}, ${ownerColor[1]}, ${ownerColor[2]})`;// '#e74c3c';
                    this.ctx.fillRect(x + borders, y + borders, this.squareSize - borders, this.squareSize - borders);

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
                    this.ctx.fillRect(x + borders, y + borders, this.squareSize - borders, this.squareSize - borders);
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
        console.log("players:")
        console.log(this.players);
    }
}