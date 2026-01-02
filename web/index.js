import init, { start } from "./pkg/client.js"

const game_canvas = document.getElementById("game");

async function run() {
    await init();
    start(game_canvas);
}

run().then(() => {}).catch(console.error);