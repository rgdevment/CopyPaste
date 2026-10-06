#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

const GAP: f64 = 12.0;
const EDGE: f64 = 8.0;

pub fn beside(pointer: (f64, f64), area: Area, size: (f64, f64), unit: f64) -> (f64, f64) {
    let (width, height) = size;
    let (gap, edge) = (GAP * unit, EDGE * unit);
    let x = if pointer.0 + gap + width <= area.right {
        pointer.0 + gap
    } else if pointer.0 - gap - width >= area.left {
        pointer.0 - gap - width
    } else {
        area.right - width - gap
    };
    let y = (pointer.1 - height / 2.0)
        .max(area.top + edge)
        .min(area.bottom - edge - height);
    (
        x.min(area.right - width).max(area.left),
        y.min(area.bottom - height).max(area.top),
    )
}

#[cfg(test)]
#[path = "placing_test.rs"]
mod tests;
