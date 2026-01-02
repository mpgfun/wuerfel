import init, { render } from "./pkg/client.js"

const game_canvas = document.getElementById("game");
const ctx = game_canvas.getContext("2d");

function renderLoop() {
    requestAnimationFrame(renderLoop);

    game_canvas.width = window.innerWidth;
    game_canvas.height = window.innerHeight;

    // call WASM
    render(ctx, game_canvas.width, game_canvas.height);
}

async function run() {
    await init();
    renderLoop();
}

run().then(() => {}).catch(console.error);