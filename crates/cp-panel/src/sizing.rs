const EDGE: f64 = 8.0;

pub fn room_of(area: crate::placing::Area, scale: f64) -> (f64, f64) {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    (
        (area.right - area.left) / scale,
        (area.bottom - area.top) / scale,
    )
}

pub fn fitted(wanted: (f32, f32), zoom: f32, room: (f64, f64)) -> (f32, f32) {
    let zoom = if zoom > 0.0 { f64::from(zoom) } else { 1.0 };
    let (room_width, room_height) = (
        (room.0 - 2.0 * EDGE).max(1.0),
        (room.1 - 2.0 * EDGE).max(1.0),
    );
    let width = (f64::from(wanted.0) * zoom).min(room_width);
    let height = if wanted.1 > 0.0 {
        (f64::from(wanted.1) * zoom).min(room_height)
    } else {
        room_height
    };
    #[allow(clippy::cast_possible_truncation)]
    ((width / zoom) as f32, (height / zoom) as f32)
}

#[cfg(test)]
#[path = "sizing_test.rs"]
mod tests;
