//! Scratch-style block silhouettes painted with the low-level path API.
//!
//! Leaf blocks are rounded rectangles with a puzzle bump on the top edge and a
//! matching notch on the bottom edge. Compound blocks are C-shapes with the
//! same bump/notch plus a mouth cut on the right for nested children.

use openframe::{canvas, point, px, Bounds, IntoElement, PathBuilder, Pixels, Rgba, Styled};

/// Height of the puzzle bump / depth of the notch.
pub const BUMP_H: f32 = 5.0;
/// Left offset of the bump/notch.
const NOTCH_X: f32 = 20.0;
/// Width of the bump/notch.
const NOTCH_W: f32 = 28.0;
/// Corner radius.
const RAD: f32 = 7.0;
/// Fixed header-bar height of a compound block.
pub const HEADER_H: f32 = 30.0;
/// Fixed bottom-lip height of a compound block.
pub const LIP_H: f32 = 14.0;
/// Width of the compound block's left spine.
pub const SPINE_W: f32 = 16.0;

fn trace_leaf(pb: &mut PathBuilder, b: Bounds<Pixels>) {
    let x0 = b.origin.x;
    let y0 = b.origin.y;
    let x1 = x0 + b.size.width;
    let y1 = y0 + b.size.height;
    let top = y0 + px(BUMP_H);
    let nx = x0 + px(NOTCH_X);
    let nx2 = x0 + px(NOTCH_X + NOTCH_W);
    let r = px(RAD);
    let bump = px(BUMP_H);

    pb.move_to(point(x0 + r, top));
    pb.line_to(point(nx, top));
    pb.line_to(point(nx, y0));
    pb.line_to(point(nx2, y0));
    pb.line_to(point(nx2, top));
    pb.line_to(point(x1 - r, top));
    pb.curve_to(point(x1, top + r), point(x1, top));
    pb.line_to(point(x1, y1 - r));
    pb.curve_to(point(x1 - r, y1), point(x1, y1));
    pb.line_to(point(nx2, y1));
    pb.line_to(point(nx2, y1 - bump));
    pb.line_to(point(nx, y1 - bump));
    pb.line_to(point(nx, y1));
    pb.line_to(point(x0 + r, y1));
    pb.curve_to(point(x0, y1 - r), point(x0, y1));
    pb.line_to(point(x0, top + r));
    pb.curve_to(point(x0 + r, top), point(x0, top));
    pb.close();
}

fn trace_c_block(pb: &mut PathBuilder, b: Bounds<Pixels>) {
    let x0 = b.origin.x;
    let y0 = b.origin.y;
    let x1 = x0 + b.size.width;
    let y1 = y0 + b.size.height;
    let top = y0 + px(BUMP_H);
    let hb = top + px(HEADER_H);
    let lt = y1 - px(LIP_H);
    let mx = x0 + px(SPINE_W);
    let nx = x0 + px(NOTCH_X);
    let nx2 = x0 + px(NOTCH_X + NOTCH_W);
    let r = px(RAD);
    let bump = px(BUMP_H);

    pb.move_to(point(x0 + r, top));
    pb.line_to(point(nx, top));
    pb.line_to(point(nx, y0));
    pb.line_to(point(nx2, y0));
    pb.line_to(point(nx2, top));
    pb.line_to(point(x1 - r, top));
    pb.curve_to(point(x1, top + r), point(x1, top));
    pb.line_to(point(x1, hb));
    pb.line_to(point(mx, hb));
    pb.line_to(point(mx, lt));
    pb.line_to(point(x1, lt));
    pb.line_to(point(x1, y1 - r));
    pb.curve_to(point(x1 - r, y1), point(x1, y1));
    pb.line_to(point(nx2, y1));
    pb.line_to(point(nx2, y1 - bump));
    pb.line_to(point(nx, y1 - bump));
    pb.line_to(point(nx, y1));
    pb.line_to(point(x0 + r, y1));
    pb.curve_to(point(x0, y1 - r), point(x0, y1));
    pb.line_to(point(x0, top + r));
    pb.curve_to(point(x0 + r, top), point(x0, top));
    pb.close();
}

/// Mouth-region rectangle of a compound block (filled with the soft tint).
fn trace_mouth(pb: &mut PathBuilder, b: Bounds<Pixels>) {
    let x0 = b.origin.x;
    let y0 = b.origin.y;
    let x1 = x0 + b.size.width;
    let y1 = y0 + b.size.height;
    let hb = y0 + px(BUMP_H + HEADER_H);
    let lt = y1 - px(LIP_H);
    let mx = x0 + px(SPINE_W);
    pb.move_to(point(mx, hb));
    pb.line_to(point(x1, hb));
    pb.line_to(point(x1, lt));
    pb.line_to(point(mx, lt));
    pb.close();
}

/// Absolutely-positioned canvas painting a leaf-block silhouette.
pub fn leaf_bg(accent: Rgba, ring: Option<Rgba>) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let mut fill = PathBuilder::fill();
            trace_leaf(&mut fill, bounds);
            if let Ok(p) = fill.build() {
                window.paint_path(p, accent);
            }
            if let Some(ring) = ring {
                let mut stroke = PathBuilder::stroke(px(2.0));
                trace_leaf(&mut stroke, bounds);
                if let Ok(p) = stroke.build() {
                    window.paint_path(p, ring);
                }
            }
        },
    )
    .absolute()
    .size_full()
}

/// Absolutely-positioned canvas painting a compound C-block silhouette.
pub fn c_block_bg(accent: Rgba, soft: Rgba, ring: Option<Rgba>) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let mut fill = PathBuilder::fill();
            trace_c_block(&mut fill, bounds);
            if let Ok(p) = fill.build() {
                window.paint_path(p, accent);
            }
            let mut mouth = PathBuilder::fill();
            trace_mouth(&mut mouth, bounds);
            if let Ok(p) = mouth.build() {
                window.paint_path(p, soft);
            }
            if let Some(ring) = ring {
                let mut stroke = PathBuilder::stroke(px(2.0));
                trace_c_block(&mut stroke, bounds);
                if let Ok(p) = stroke.build() {
                    window.paint_path(p, ring);
                }
            }
        },
    )
    .absolute()
    .size_full()
}
