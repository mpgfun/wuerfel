use std::{cell::RefCell, rc::Rc};

use web_sys::CanvasRenderingContext2d;

use crate::ClientGameState;

pub struct RenderingInfo {
    pub ctx: CanvasRenderingContext2d,
    pub width: u32,
    pub height: u32,
}

pub fn render(info: &mut RenderingInfo, state: Rc<RefCell<ClientGameState>>) {
    // TODO..

    info.ctx.set_fill_style_str(match state.borrow().color {
        0 => "#ff0000",
        1 => "#00ff00",
        2 => "#0000ff",
        _ => "#000000",
    });
    info.ctx
        .fill_rect(0.0, 0.0, info.width as f64, info.height as f64);

    state.borrow_mut().color += 1;
    if state.borrow().color == 3 {
        state.borrow_mut().color = 0;
    }
}
