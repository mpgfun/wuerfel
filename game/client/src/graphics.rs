use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, js_sys::Number, window};

use crate::{ClientGame, ClientState, graphics::zoom_handler::transform_zoom};

mod zoom_handler;

const MIN_ZOOM: f64 = 0.5;
const MAX_ZOOM: f64 = 3.0;
const ZOOM_STEP: f64 = 0.1;

#[derive(Clone)]
pub struct RenderingInfo {
    pub ctx: CanvasRenderingContext2d,
    pub canvas: HtmlCanvasElement,
    pub width: u32,
    pub height: u32,
    pub camera_zoom: f64,
    pub zoom_transform: (f64, f64),
    pub camera_position: (f64, f64),
    pub last_mouse_pos: Option<(f64, f64)>,
}

pub fn render(info: &RenderingInfo, state: &ClientState) {
    match &state.game {
        None => draw_loading_screen(info),
        Some(game) => draw_game(info, game, state),
    }
}

pub fn resize(info: &mut RenderingInfo) {
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
}

fn background(info: &RenderingInfo, color: &str) {
    info.ctx.set_fill_style_str(color);
    info.ctx
        .fill_rect(0.0, 0.0, info.width as f64, info.width as f64);
}

fn draw_loading_screen(info: &RenderingInfo) {
    background(&info, "#000000");
    info.ctx.set_fill_style_str("#ffffff");
    info.ctx.set_font("30px Arial");
    info.ctx.set_text_baseline("middle");
    info.ctx.set_text_align("center");
    let _ = info.ctx.fill_text(
        "Loading...",
        (info.width / 2) as f64,
        (info.height / 2) as f64,
    );
}

fn draw_game(info: &RenderingInfo, game: &ClientGame, _state: &ClientState) {
    transform_zoom(&info);
    background(&info, "#ffffff");
    let square_size = info.width / game.data.map_config.size_x;
    for (pos, square) in &game.data.snapshot.squares {
        let color = match game.data.snapshot.players.get(&square.owner) {
            Some(player_data) => player_data.1.clone(),
            None => String::from("#000000"),
        };
        info.ctx.set_fill_style_str(color.as_str());
        info.ctx.fill_rect(
            (square_size * pos.x) as f64,
            (square_size * pos.y) as f64,
            square_size as f64,
            square_size as f64,
        );
    }
    // reset_transformation(&info);
}

pub fn update_zoom(info: &mut RenderingInfo, delta: f64) {
    if delta < 0.0 {
        if info.camera_zoom <= MIN_ZOOM {
            return;
        }
        info.camera_zoom -= ZOOM_STEP;
    } else if delta > 0.0 {
        if info.camera_zoom >= MAX_ZOOM {
            return;
        }
        info.camera_zoom += ZOOM_STEP;
    }
    if let Some(last_mouse_pos) = info.last_mouse_pos {
        info.zoom_transform = last_mouse_pos;
    }
}
