use crate::{console_log, graphics::RenderingInfo, log_js_err};

#[inline]
pub fn transform_zoom(info: &RenderingInfo) {
    if let Err(e) = info.ctx.transform(
        info.camera_zoom,
        0.0,
        0.0,
        info.camera_zoom,
        info.camera_position.0,
        info.camera_position.1,
    ) {
        log_js_err!(e);
    }
}

#[inline]
pub fn reset_transformation(info: &RenderingInfo) {
    if let Err(e) = info.ctx.reset_transform() {
        log_js_err!(e);
    }
}
