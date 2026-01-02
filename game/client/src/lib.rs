use std::{cell::RefCell, ops::ControlFlow, rc::Rc};

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MessageEvent, WebSocket, js_sys};

use crate::net::{create_ws, decode_and_apply_packet};

mod net;

#[wasm_bindgen]
unsafe extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        $crate::log(&format_args!($($t)*).to_string())
    };
}

struct ClientGameState {
    ctx: CanvasRenderingContext2d,
    should_disconnect: bool,
}

impl ClientGameState {
    pub fn render(&self) {
        todo!()
    }

    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        Self {
            ctx,
            should_disconnect: false,
        }
    }
}

#[wasm_bindgen]
pub fn start(canvas: HtmlCanvasElement) {
    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    let state = Rc::new(RefCell::new(ClientGameState::new(ctx)));
    let socket = create_ws();

    start_render_loop(state, socket);
}

fn start_render_loop(
    // mut rx: UnboundedReceiver<Vec<u8>>,
    state: Rc<RefCell<ClientGameState>>,
    socket: WebSocket,
) {
    let mut socket_cloned = socket.clone();
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: web_sys::MessageEvent| {
        if let Ok(buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
            let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
            match decode_and_apply_packet(state.borrow_mut(), bytes, &mut socket_cloned) {
                ControlFlow::Break(_) => {
                    console_log!("controlflow break");
                }
                ControlFlow::Continue(_) => {
                    console_log!("controlflow continue");
                }
            }
        }
    });
    socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}
