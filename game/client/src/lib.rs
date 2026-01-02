use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

#[wasm_bindgen]
pub fn render(ctx: CanvasRenderingContext2d, width: i32, height: i32) {
    ctx.set_fill_style_str("#ff0000");
    ctx.fill_rect(0.0, 0.0, width as f64, height as f64);
}