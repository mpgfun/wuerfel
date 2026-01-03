use std::{cell::RefCell, rc::Rc};

use futures::{
    StreamExt,
    channel::mpsc::{self, UnboundedSender},
};
use shared::net::packets::join_response::JoinResponseS2CPacketData;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};

use crate::{
    com::MpscMessage,
    graphics::{RenderingInfo, render},
    net::{create_ws, handle_mpsc_message},
};

mod com;
mod graphics;
mod net;

const FPS: i32 = 60;

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

struct ClientGame {
    #[allow(unused)]
    pub data: JoinResponseS2CPacketData,
}

impl ClientGame {
    pub fn new(data: JoinResponseS2CPacketData) -> Self {
        Self { data }
    }
}

struct ClientState {
    should_close: bool,
    game: Option<ClientGame>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            should_close: false,
            game: None,
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
    let rendering_info: RenderingInfo = RenderingInfo {
        ctx,
        canvas: canvas.clone(),
        width: canvas.width(),
        height: canvas.height(),
    };
    let (tx, mut rx) = mpsc::unbounded::<MpscMessage>();
    let state = Rc::new(RefCell::new(ClientState::new()));
    let ws = create_ws();
    let mut cloned_ws = ws.clone();
    let cloned_state = state.clone();
    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            handle_mpsc_message(msg, &mut cloned_ws, cloned_state.clone());
        }
    });

    start_render_loop(rendering_info, state, tx);
}

fn start_render_loop(
    mut rendering_info: RenderingInfo,
    state: Rc<RefCell<ClientState>>,
    tx: UnboundedSender<MpscMessage>,
) {
    tx.unbounded_send(MpscMessage::CreateOnMessage).unwrap();
    let interval_callback = Closure::wrap(Box::new(move || {
        render(&mut rendering_info, state.clone());
    }) as Box<dyn FnMut()>);
    let _timeout = window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            1000 / FPS,
        )
        .unwrap();
    interval_callback.forget();
}
