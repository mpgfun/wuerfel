use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, js_sys::Number, window};

use crate::{ClientGame, ClientState};

pub struct RenderingInfo {
    pub ctx: CanvasRenderingContext2d,
    pub canvas: HtmlCanvasElement,
    pub width: u32,
    pub height: u32,
}

pub fn render(info: &mut RenderingInfo, state: Rc<RefCell<ClientState>>) {
    info.canvas.set_width(
        window()
            .unwrap()
            .inner_width()
            .unwrap()
            .dyn_into::<Number>()
            .unwrap()
            .value_of() as u32,
    );
    info.canvas.set_height(
        window()
            .unwrap()
            .inner_height()
            .unwrap()
            .dyn_into::<Number>()
            .unwrap()
            .value_of() as u32,
    );

    info.width = info.canvas.width();
    info.height = info.canvas.height();

    let state_ref = state.borrow();
    match &state_ref.game {
        None => draw_loading_screen(info),
        Some(game) => draw_game(info, game, state.clone()),
    }
}

fn background(info: &mut RenderingInfo, color: &str) {
    info.ctx.set_fill_style_str(color);
    info.ctx
        .fill_rect(0.0, 0.0, info.width as f64, info.height as f64);
}

fn draw_loading_screen(info: &mut RenderingInfo) {
    background(info, "#000000");
    info.ctx.set_fill_style_str("#ffffff");
    info.ctx.set_font("30px Arial");
    info.ctx.set_text_baseline("middle");
    info.ctx.set_text_align("center");
    info.ctx
        .fill_text(
            "Loading...",
            (info.width / 2) as f64,
            (info.height / 2) as f64,
        )
        .unwrap();
}

fn draw_game(info: &mut RenderingInfo, game: &ClientGame, _state: Rc<RefCell<ClientState>>) {
    background(info, "#ffffff");
    let square_size_x = info.width / game.data.map_config.size_x;
    let square_size_y = info.height / game.data.map_config.size_y;
    for (pos, square) in &game.data.snapshot.squares {
        let color = match game.data.snapshot.players.get(&square.owner) {
            Some(player_data) => player_data.1.clone(),
            None => String::from("#000000"),
        };
        info.ctx.set_fill_style_str(color.as_str());
        info.ctx.fill_rect(
            (square_size_x * pos.x) as f64,
            (square_size_y * pos.y) as f64,
            square_size_x as f64,
            square_size_y as f64,
        );
    }
}
