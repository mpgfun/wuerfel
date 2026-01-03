use futures::channel::mpsc::{self, UnboundedSender};
use shared::net::packets::{
    C2SPacket, join::JoinC2SPacket, join_response::JoinResponseS2CPacketData,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, WheelEvent, window};

use crate::{
    com::{MpscMessage, mpsc_receiver_loop},
    graphics::RenderingInfo,
    net::create_ws,
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

#[macro_export]
macro_rules! log_js_err {
    ($err:expr) => {
        console_log!(
            "{}",
            match $err.as_string() {
                Some(s) => s,
                None => String::from("<no error information available>"),
            }
        )
    };
}

struct ClientGame {
    pub data: JoinResponseS2CPacketData,
}

impl ClientGame {
    pub fn new(data: JoinResponseS2CPacketData) -> Self {
        Self { data }
    }
}

struct ClientState {
    game: Option<ClientGame>,
}

impl ClientState {
    pub fn new() -> Self {
        Self { game: None }
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
        camera_zoom: 1.0,
        camera_position: (0.0, 0.0),
    };
    let (tx, rx) = mpsc::unbounded::<MpscMessage>();
    register_event_handlers(canvas, tx.clone());
    let state = ClientState::new();
    let ws = create_ws(tx.clone());
    let Some(ws) = ws else {
        console_log!("Failed to create websocket");
        return;
    };
    start_render_loop(tx.clone());
    let _ = tx.unbounded_send(MpscMessage::SendPacket(C2SPacket::Join(JoinC2SPacket {
        lobby_id: 0,
    })));
    spawn_local(mpsc_receiver_loop(rx, tx, ws, rendering_info, state));
}

fn start_render_loop(tx: UnboundedSender<MpscMessage>) {
    let interval_callback = Closure::wrap(Box::new(move || {
        let _ = tx.unbounded_send(MpscMessage::Draw);
    }) as Box<dyn FnMut()>);
    let _timeout = match window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            interval_callback.as_ref().unchecked_ref(),
            1000 / FPS,
        ) {
        Ok(timeout) => timeout,
        Err(e) => {
            log_js_err!(e);
            return;
        }
    };
    interval_callback.forget();
}

fn register_event_handlers(canvas: HtmlCanvasElement, tx: UnboundedSender<MpscMessage>) {
    let onwheel = Closure::wrap(Box::new(move |e: WheelEvent| {
        let _ = tx.unbounded_send(MpscMessage::Scrolling(e.delta_y()));
    }) as Box<dyn FnMut(WheelEvent)>);
    canvas.set_onwheel(Some(onwheel.as_ref().unchecked_ref()));
    onwheel.forget();
}
