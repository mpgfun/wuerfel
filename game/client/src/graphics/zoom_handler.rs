use crate::{console_log, graphics::RenderingInfo, log_js_err};

#[inline]
pub fn transform_zoom(info: &RenderingInfo) {
    if let Err(e) = info.ctx.set_transform(
        1.0,
        0.0,
        0.0,
        1.0,
        info.zoom_transform.0,
        info.zoom_transform.1,
    ) {
        log_js_err!(e);
    }

    if let Err(e) = info.ctx.scale(info.camera_zoom, info.camera_zoom) {
        log_js_err!(e);
    }

    if let Err(e) = info.ctx.transform(
        1.0,
        0.0,
        0.0,
        1.0,
        -info.zoom_transform.0,
        -info.zoom_transform.1,
    ) {
        log_js_err!(e);
    }

    if let Err(e) = info.ctx.transform(
        1.0,
        0.0,
        0.0,
        1.0,
        info.camera_position.0,
        info.camera_position.1,
    ) {
        log_js_err!(e);
    }
}
