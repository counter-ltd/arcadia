//! Shared extension token editor (persisted under `extension_tokens/`).

use crate::gui::app::text_input_caret::text_with_trailing_caret;
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;
use arcadia_core::modules::python_registry::{
    format_slider_value, resolve_slider_numeric, snap_slider_value, style_token_row_visible,
    StyleTokenKind, StyleTokenNumericGranularity, StyleTokenSpec,
};
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, relative, rgb, AnyElement, AppContext, Bounds, ClickEvent, Context, DragMoveEvent,
    FontWeight, Hitbox, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    ParentElement, Pixels, Point, Render, Rgba, SharedString, StatefulInteractiveElement, Styled,
    Window,
};

// ── Gradient helpers ──────────────────────────────────────────────────────────

const GRAD_BAR_W: f32 = 280.0;
const GRAD_BAR_H: f32 = 16.0;
const GRAD_HANDLE_D: f32 = 18.0;
const GRAD_HANDLE_R: f32 = GRAD_HANDLE_D / 2.0;
// Container tall enough for the handle to sit centred on the bar.
const GRAD_CONT_H: f32 = GRAD_HANDLE_D + 6.0;
const GRAD_BAR_Y: f32 = (GRAD_CONT_H - GRAD_BAR_H) / 2.0;
const GRAD_HANDLE_Y: f32 = (GRAD_CONT_H - GRAD_HANDLE_D) / 2.0;
const GRAD_SEGMENTS: usize = 32;
// Bar is inset by one handle-radius on each side so end-handles sit centred
// on the rounded caps rather than hanging off the pixel edge.
const GRAD_BAR_INSET: f32 = GRAD_HANDLE_R;
const GRAD_BAR_INNER_W: f32 = GRAD_BAR_W - 2.0 * GRAD_BAR_INSET;

/// Remap a raw t-value (0–1 across the full container width) to a gradient
/// position (0–1 across the inset bar area).
#[inline]
fn remap_t_to_pos(raw_t: f32) -> f32 {
    ((raw_t * GRAD_BAR_W - GRAD_BAR_INSET) / GRAD_BAR_INNER_W).clamp(0.0, 1.0)
}

#[derive(Clone, Debug)]
pub struct GradientStop {
    pub pos: f32,
    pub color: String,
}

pub fn parse_gradient_stops(s: &str) -> Vec<GradientStop> {
    let s = s.trim();
    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(s) {
        let mut stops: Vec<GradientStop> = arr
            .iter()
            .filter_map(|v| {
                let pos = v.get("pos")?.as_f64()? as f32;
                let color = v.get("color")?.as_str()?.to_string();
                Some(GradientStop {
                    pos: pos.clamp(0., 1.),
                    color,
                })
            })
            .collect();
        if stops.len() >= 2 {
            stops.sort_by(|a, b| {
                a.pos
                    .partial_cmp(&b.pos)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            return stops;
        }
    }
    vec![
        GradientStop {
            pos: 0.0,
            color: "#1a1a2e".to_string(),
        },
        GradientStop {
            pos: 1.0,
            color: "#16213e".to_string(),
        },
    ]
}

pub fn stops_to_json(stops: &[GradientStop]) -> String {
    let segs: Vec<String> = stops
        .iter()
        .map(|s| format!(r#"{{"pos":{},"color":"{}"}}"#, s.pos, s.color))
        .collect();
    format!("[{}]", segs.join(","))
}

fn lerp_rgba(a: Rgba, b: Rgba, t: f32) -> Rgba {
    Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: 1.0,
    }
}

fn sample_gradient(stops: &[GradientStop], t: f32) -> Rgba {
    if stops.is_empty() {
        return rgb(0x000000);
    }
    let first_rgba = crate::gui::app::lifecycle::parse_hex_color(&stops[0].color)
        .unwrap_or_else(|| rgb(0x000000));
    let last_rgba = crate::gui::app::lifecycle::parse_hex_color(&stops[stops.len() - 1].color)
        .unwrap_or_else(|| rgb(0x000000));
    if t <= stops[0].pos {
        return first_rgba;
    }
    if t >= stops[stops.len() - 1].pos {
        return last_rgba;
    }
    for i in 0..stops.len().saturating_sub(1) {
        let a = &stops[i];
        let b = &stops[i + 1];
        if t >= a.pos && t <= b.pos {
            let span = b.pos - a.pos;
            let local_t = if span > 1e-6 { (t - a.pos) / span } else { 0.0 };
            let ca = crate::gui::app::lifecycle::parse_hex_color(&a.color)
                .unwrap_or_else(|| rgb(0x000000));
            let cb = crate::gui::app::lifecycle::parse_hex_color(&b.color)
                .unwrap_or_else(|| rgb(0x000000));
            return lerp_rgba(ca, cb, local_t);
        }
    }
    last_rgba
}

// ── Gradient drag payload ─────────────────────────────────────────────────────

#[derive(Clone)]
struct GradientStopDrag {
    module: String,
    key: String,
    stop_index: usize,
}

fn token_edit_rgba(display: &str, default_s: &str) -> Rgba {
    crate::gui::app::lifecycle::parse_hex_color(display.trim())
        .or_else(|| crate::gui::app::lifecycle::parse_hex_color(default_s.trim()))
        .unwrap_or_else(|| rgb(0x00cc88))
}

fn token_kind_label(kind: StyleTokenKind) -> &'static str {
    match kind {
        StyleTokenKind::Color => "color",
        StyleTokenKind::Float => "float",
        StyleTokenKind::String => "string",
        StyleTokenKind::Bool => "bool",
        StyleTokenKind::Int => "int",
        StyleTokenKind::Gradient => "gradient",
    }
}

#[derive(Clone)]
struct ExtensionTokenSliderDrag {
    module: String,
    key: String,
    kind: StyleTokenKind,
    lo: f64,
    hi: f64,
    step: f64,
    granularity: StyleTokenNumericGranularity,
}

struct SliderDragGhost;

impl Render for SliderDragGhost {
    fn render(&mut self, _: &mut Window, _: &mut Context<'_, Self>) -> impl IntoElement {
        div().w(px(1.)).h(px(1.))
    }
}

fn bool_from_display(s: &str) -> bool {
    matches!(
        s.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
}

fn slider_t_from_hit(bounds: &Bounds<Pixels>, position: &Point<Pixels>) -> f64 {
    let w = bounds.size.width.to_f64();
    if w <= f64::EPSILON {
        return 0.;
    }
    let x = position.x.to_f64() - bounds.origin.x.to_f64();
    (x / w).clamp(0., 1.)
}

impl ArcadiaRoot {
    pub(crate) fn extension_tokens_settings_card(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
        module_id: &str,
        specs: &[StyleTokenSpec],
        title: String,
        help: String,
        extension_label: Option<String>,
    ) -> openframe::Div {
        let (panel_bg, panel_stroke, panel_radius, header_color, subtext_color, is_glyph) = {
            let g = theme::active_glyph(cx);
            (
                g.as_ref()
                    .map(|g| g.surface)
                    .unwrap_or_else(|| theme::module_panel_bg(is_dark)),
                g.as_ref()
                    .map(|g| g.border)
                    .unwrap_or_else(|| theme::module_panel_stroke(is_dark)),
                g.as_ref().map(|g| g.border_radius).unwrap_or(8.0),
                g.as_ref()
                    .map(|g| g.text)
                    .unwrap_or_else(|| theme::module_title_text(is_dark)),
                g.as_ref()
                    .map(|g| g.dim)
                    .unwrap_or_else(|| theme::module_meta_text(is_dark)),
                g.is_some(),
            )
        };
        let p = theme::theme_palette(cx, is_dark);
        let ext_token_focus = self.extension_token_focus.clone();
        let input_bg = theme::glyph_snapshot(cx)
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::ui_surface(cx, is_dark));
        let input_border = theme::glyph_snapshot(cx)
            .map(|g| g.border)
            .unwrap_or_else(|| theme::ui_border(cx, is_dark));

        let module_owned = module_id.to_string();
        let rows = specs
            .iter()
            .filter(|spec| {
                style_token_row_visible(
                    &spec.visibility,
                    module_id,
                    specs,
                    &self.extension_token_values,
                )
            })
            .map(|spec| {
            let key = spec.key.clone();
            let label = spec.label.clone();
            let kind = spec.kind;
            let pair = (module_owned.clone(), key.clone());
            let display_val = self
                .extension_token_values
                .get(&pair)
                .cloned()
                .unwrap_or_else(|| spec.default_value.clone());
            let is_editing = self
                .extension_token_editing
                .as_ref()
                .is_some_and(|(m, k)| m == &module_owned && k == &key);

            let row_module = module_owned.clone();
            let row_key = key.clone();
            let row_default = spec.default_value.clone();

            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(1.))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(header_color)
                                        .child(label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(subtext_color)
                                        .child(format!(
                                            "{} · {}",
                                            key,
                                            token_kind_label(kind)
                                        )),
                                ),
                        ),
                )
                .child({
                    let edit_cell: AnyElement = if kind == StyleTokenKind::Gradient {
                        // ── Gradient editor ──────────────────────────────────
                        let stops = parse_gradient_stops(&display_val);
                        let n_stops = stops.len();

                        // Gradient bar: GRAD_SEGMENTS solid-colour segments.
                        let segment_colors: Vec<Rgba> = (0..GRAD_SEGMENTS)
                            .map(|i| {
                                let t = (i as f32 + 0.5) / GRAD_SEGMENTS as f32;
                                sample_gradient(&stops, t)
                            })
                            .collect();

                        // Normal-flow bar inset by GRAD_BAR_INSET on each side so the
                        // end handles sit centred on the rounded caps.
                        let gradient_bar = div()
                            .mt(px(GRAD_BAR_Y))
                            .ml(px(GRAD_BAR_INSET))
                            .w(px(GRAD_BAR_INNER_W))
                            .h(px(GRAD_BAR_H))
                            .rounded(px(GRAD_BAR_H / 2.0))
                            .overflow_hidden()
                            .flex()
                            .flex_row()
                            .children(segment_colors.into_iter().map(|c| {
                                div().flex_1().h_full().bg(c)
                            }));

                        // Click-to-add overlay: absolute over bar, transparent hit target.
                        let bar_hit = {
                            let m = row_module.clone();
                            let k = row_key.clone();
                            let stops_for_click = stops.clone();
                            let weak = cx.weak_entity();
                            div()
                                .absolute()
                                .top(px(GRAD_BAR_Y))
                                .left_0()
                                .right_0()
                                .h(px(GRAD_BAR_H))
                                .rounded(px(GRAD_BAR_H / 2.0))
                                .cursor_pointer()
                                .on_mouse_down_with_hitbox(
                                    MouseButton::Left,
                                    move |ev: &MouseDownEvent, hb: &Hitbox, _, cx| {
                                        let raw_t = slider_t_from_hit(&hb.bounds, &ev.position) as f32;
                                        let t = remap_t_to_pos(raw_t);
                                        // Near an existing handle → let handle's own handler take it.
                                        let threshold = GRAD_HANDLE_D / GRAD_BAR_INNER_W;
                                        let near_existing = stops_for_click
                                            .iter()
                                            .any(|s| (s.pos - t).abs() < threshold);
                                        if near_existing {
                                            return;
                                        }
                                        let _ = weak.update(cx, |this, cx| {
                                            let pair = (m.clone(), k.clone());
                                            let mut new_stops = stops_for_click.clone();
                                            let sampled = sample_gradient(&stops_for_click, t);
                                            let r = (sampled.r * 255.) as u8;
                                            let g = (sampled.g * 255.) as u8;
                                            let b = (sampled.b * 255.) as u8;
                                            let hex = format!("#{r:02x}{g:02x}{b:02x}");
                                            new_stops.push(GradientStop { pos: t, color: hex });
                                            new_stops.sort_by(|a, b| {
                                                a.pos
                                                    .partial_cmp(&b.pos)
                                                    .unwrap_or(std::cmp::Ordering::Equal)
                                            });
                                            let json = stops_to_json(&new_stops);
                                            this.extension_token_values.insert(pair, json);
                                            this.flush_extension_token_edit(m.clone(), k.clone(), cx);
                                            cx.notify();
                                        });
                                    },
                                )
                        };

                        // Per-stop handles (absolutely positioned on the container).
                        let handles: Vec<_> = stops
                            .iter()
                            .enumerate()
                            .map(|(i, stop)| {
                                // Handle centre sits at INSET + pos*INNER_W; subtract
                                // HANDLE_R to get the left-edge pixel offset.
                                let handle_x = stop.pos * GRAD_BAR_INNER_W;
                                let stop_color = crate::gui::app::lifecycle::parse_hex_color(
                                    &stop.color,
                                )
                                .unwrap_or_else(|| rgb(0x888888));

                                let m = row_module.clone();
                                let k = row_key.clone();
                                let stops_clone = stops.clone();
                                let stop_hex = stop.color.clone();

                                div()
                                    .absolute()
                                    .top(px(GRAD_HANDLE_Y))
                                    .left(px(handle_x))
                                    .w(px(GRAD_HANDLE_D))
                                    .h(px(GRAD_HANDLE_D))
                                    .rounded_full()
                                    .bg(stop_color)
                                    .border_2()
                                    .border_color(openframe::white())
                                    .shadow_sm()
                                    .cursor_pointer()
                                    .id((
                                        SharedString::from(format!(
                                            "grad-stop-{}-{}-{}",
                                            row_module, row_key, i
                                        )),
                                        0usize,
                                    ))
                                    // Drag to reposition.
                                    .on_drag(
                                        GradientStopDrag {
                                            module: m.clone(),
                                            key: k.clone(),
                                            stop_index: i,
                                        },
                                        |_, _, _, cx| cx.new(|_| SliderDragGhost),
                                    )
                                    // Click (not drag) to open colour picker.
                                    .on_click({
                                        let weak = cx.weak_entity();
                                        let m2 = m.clone();
                                        let k2 = k.clone();
                                        let hex = stop_hex.clone();
                                        let default_hex = stops_clone
                                            .first()
                                            .map(|s| s.color.clone())
                                            .unwrap_or_default();
                                        move |_: &ClickEvent, _, cx| {
                                            let _ = weak.update(cx, |this, cx| {
                                                if let Some((ref em, ref ek)) =
                                                    this.extension_token_editing.clone()
                                                {
                                                    this.flush_extension_token_edit(
                                                        em.clone(),
                                                        ek.clone(),
                                                        cx,
                                                    );
                                                }
                                                this.extension_token_editing = None;
                                                this.gradient_stop_editing_index = Some(i);
                                                this.color_picker_modal = Some((
                                                    m2.clone(),
                                                    k2.clone(),
                                                    hex.clone(),
                                                    default_hex.clone(),
                                                ));
                                                cx.notify();
                                            });
                                        }
                                    })
                                    // Right-click to delete (if more than 2 stops).
                                    .when(n_stops > 2, |d| {
                                        d.on_mouse_down(
                                            MouseButton::Right,
                                            cx.listener({
                                                let m = m.clone();
                                                let k = k.clone();
                                                let stops_del = stops.clone();
                                                move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    let mut new_stops = stops_del.clone();
                                                    if new_stops.len() > 2 {
                                                        new_stops.remove(i);
                                                    }
                                                    let pair = (m.clone(), k.clone());
                                                    let json = stops_to_json(&new_stops);
                                                    this.extension_token_values.insert(pair, json);
                                                    this.flush_extension_token_edit(
                                                        m.clone(),
                                                        k.clone(),
                                                        cx,
                                                    );
                                                    cx.notify();
                                                }
                                            }),
                                        )
                                    })
                                    .into_any_element()
                            })
                            .collect();

                        // Drag-move handler on the outer container updates stop position.
                        let container = {
                            let m = row_module.clone();
                            let k = row_key.clone();
                            let stops_drag = stops.clone();
                            div()
                                .relative()
                                .w(px(GRAD_BAR_W))
                                .h(px(GRAD_CONT_H))
                                .child(gradient_bar)
                                .child(bar_hit)
                                .children(handles)
                                .on_drag_move(cx.listener(
                                    move |this,
                                          ev: &DragMoveEvent<GradientStopDrag>,
                                          _,
                                          cx| {
                                        let (drag_m, drag_k, drag_idx) = {
                                            let pl = ev.drag(cx);
                                            if pl.module != m || pl.key != k {
                                                return;
                                            }
                                            (pl.module.clone(), pl.key.clone(), pl.stop_index)
                                        };
                                        let t = remap_t_to_pos(
                                            slider_t_from_hit(&ev.bounds, &ev.event.position)
                                                as f32,
                                        );
                                        let pair = (drag_m.clone(), drag_k.clone());
                                        let current = this
                                            .extension_token_values
                                            .get(&pair)
                                            .cloned()
                                            .unwrap_or_else(|| stops_to_json(&stops_drag));
                                        let mut cur_stops = parse_gradient_stops(&current);
                                        if drag_idx < cur_stops.len() {
                                            // Clamp so this stop never crosses its neighbours.
                                            let lo = if drag_idx > 0 {
                                                cur_stops[drag_idx - 1].pos + 0.01
                                            } else {
                                                0.0
                                            };
                                            let hi = if drag_idx + 1 < cur_stops.len() {
                                                cur_stops[drag_idx + 1].pos - 0.01
                                            } else {
                                                1.0
                                            };
                                            cur_stops[drag_idx].pos = t.clamp(lo, hi);
                                            let new_json = stops_to_json(&cur_stops);
                                            let prev = this
                                                .extension_token_values
                                                .get(&pair)
                                                .cloned();
                                            if prev.as_deref() == Some(new_json.as_str()) {
                                                return;
                                            }
                                            this.extension_token_values
                                                .insert(pair, new_json);
                                            this.flush_extension_token_edit(
                                                drag_m, drag_k, cx,
                                            );
                                            cx.notify();
                                        }
                                    },
                                ))
                        };

                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(container)
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(subtext_color)
                                    .child("click bar to add stop · drag handles · right-click handle to remove"),
                            )
                            .into_any_element()
                    } else if kind == StyleTokenKind::Color {
                        let preview = token_edit_rgba(&display_val, &row_default);
                        div()
                            .px_3()
                            .py_2()
                            .rounded(px(panel_radius))
                            .bg(input_bg)
                            .border_1()
                            .border_color(input_border)
                            .text_sm()
                            .text_color(header_color)
                            .cursor_pointer()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w(px(14.))
                                            .h(px(14.))
                                            .rounded(px(3.))
                                            .border_1()
                                            .border_color(input_border)
                                            .bg(preview),
                                    )
                                    .child(div().child(display_val.clone())),
                            )
                            .on_mouse_down(MouseButton::Left, cx.listener({
                                let m = row_module.clone();
                                let k = row_key.clone();
                                let d = row_default.clone();
                                let current = display_val.clone();
                                move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    if let Some((ref em, ref ek)) = this.extension_token_editing.clone() {
                                        this.flush_extension_token_edit(em.clone(), ek.clone(), cx);
                                    }
                                    this.extension_token_editing = None;
                                    this.color_picker_modal = Some((m.clone(), k.clone(), current.clone(), d.clone()));
                                    cx.notify();
                                }
                            }))
                            .into_any_element()
                    } else if kind == StyleTokenKind::Bool {
                        let enabled = bool_from_display(display_val.as_str());
                        let r_track = panel_radius.min(8.0_f32).max(0.0);
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .cursor_pointer()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if enabled {
                                        p.accent
                                    } else {
                                        p.ui_subtext
                                    })
                                    .child(if enabled { "ON" } else { "OFF" }),
                            )
                            .child(if enabled {
                                div()
                                    .w_10()
                                    .h_6()
                                    .px_0p5()
                                    .when(!is_glyph, |d| d.rounded_full())
                                    .rounded(px(r_track))
                                    .border_1()
                                    .border_color(p.border)
                                    .bg(p.accent)
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .child(
                                        div()
                                            .w_4()
                                            .h_4()
                                            .when(!is_glyph, |d| d.rounded_full())
                                            .rounded(px(r_track))
                                            .bg(p.on_accent),
                                    )
                            } else {
                                div()
                                    .w_10()
                                    .h_6()
                                    .px_0p5()
                                    .when(!is_glyph, |d| d.rounded_full())
                                    .rounded(px(r_track))
                                    .border_1()
                                    .border_color(p.border)
                                    .bg(p.surface_elevated)
                                    .flex()
                                    .items_center()
                                    .justify_start()
                                    .child(
                                        div()
                                            .w_4()
                                            .h_4()
                                            .when(!is_glyph, |d| d.rounded_full())
                                            .rounded(px(r_track))
                                            .bg(p.toggle_knob_off),
                                    )
                            })
                            .on_mouse_down(MouseButton::Left, cx.listener({
                                let m = row_module.clone();
                                let k = row_key.clone();
                                move |this, _, _, cx| {
                                    if let Some((ref em, ref ek)) = this.extension_token_editing.clone() {
                                        if em != &m || ek != &k {
                                            this.flush_extension_token_edit(em.clone(), ek.clone(), cx);
                                        }
                                    }
                                    this.extension_token_editing = None;
                                    let pair = (m.clone(), k.clone());
                                    let cur = this
                                        .extension_token_values
                                        .get(&pair)
                                        .map(String::as_str)
                                        .unwrap_or("false");
                                    let next = (!bool_from_display(cur)).to_string();
                                    this.extension_token_values.insert(pair, next);
                                    this.flush_extension_token_edit(m.clone(), k.clone(), cx);
                                    cx.notify();
                                }
                            }))
                            .into_any_element()
                    } else if matches!(kind, StyleTokenKind::Int | StyleTokenKind::Float) {
                        match resolve_slider_numeric(spec, display_val.as_str()) {
                            Some(res) => {
                        let lo = res.lo;
                        let hi = res.hi.max(lo + 1e-6);
                        let step = res.step;
                        let gran = res.granularity;
                        let cur_raw = display_val.trim().parse::<f64>().unwrap_or(lo);
                        let cur = snap_slider_value(cur_raw, lo, hi, step, kind);
                        let fill_t = (((cur - lo) / (hi - lo)) as f32).clamp(0., 1.);
                        let fill_basis = fill_t.max(0.0001);
                        let weak = cx.weak_entity();
                        let drag_payload = ExtensionTokenSliderDrag {
                            module: row_module.clone(),
                            key: row_key.clone(),
                            kind,
                            lo,
                            hi,
                            step,
                            granularity: gran,
                        };
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .flex_1()
                                            .min_w(px(120.))
                                            .h(px(12.))
                                            .rounded(px(6.))
                                            .overflow_hidden()
                                            .border_1()
                                            .border_color(input_border)
                                            .cursor_pointer()
                                            .on_mouse_down_with_hitbox(
                                                MouseButton::Left,
                                                {
                                                    let m = row_module.clone();
                                                    let k = row_key.clone();
                                                    let weak = weak.clone();
                                                    move |ev: &MouseDownEvent, hb: &Hitbox, _, cx| {
                                                        let _ = weak.update(cx, |this, cx| {
                                                            if let Some((ref em, ref ek)) =
                                                                this.extension_token_editing.clone()
                                                            {
                                                                if em != &m || ek != &k {
                                                                    this.flush_extension_token_edit(
                                                                        em.clone(),
                                                                        ek.clone(),
                                                                        cx,
                                                                    );
                                                                }
                                                            }
                                                            this.extension_token_editing = None;
                                                            let t = slider_t_from_hit(&hb.bounds, &ev.position);
                                                            let v = lo + t * (hi - lo);
                                                            let v = snap_slider_value(v, lo, hi, step, kind);
                                                            let s =
                                                                format_slider_value(v, kind, gran);
                                                            this.extension_token_values
                                                                .insert((m.clone(), k.clone()), s);
                                                            this.flush_extension_token_edit(m.clone(), k.clone(), cx);
                                                            cx.notify();
                                                        });
                                                    }
                                                }
                                            )
                                            .id((
                                                SharedString::from(format!(
                                                    "ext-tok-slider-{}-{}",
                                                    module_owned, row_key
                                                )),
                                                0usize,
                                            ))
                                            .on_drag(drag_payload.clone(), |_, _, _, cx| {
                                                cx.new(|_| SliderDragGhost)
                                            })
                                            .on_drag_move(cx.listener({
                                                let row_module = row_module.clone();
                                                let row_key = row_key.clone();
                                                move |this, ev: &DragMoveEvent<ExtensionTokenSliderDrag>, _, cx| {
                                                    let (module, key, lo, hi, kind, step, gran) = {
                                                        let pl = ev.drag(&*cx);
                                                        if pl.module != row_module || pl.key != row_key {
                                                            return;
                                                        }
                                                        (
                                                            pl.module.clone(),
                                                            pl.key.clone(),
                                                            pl.lo,
                                                            pl.hi,
                                                            pl.kind,
                                                            pl.step,
                                                            pl.granularity,
                                                        )
                                                    };
                                                    let t = slider_t_from_hit(&ev.bounds, &ev.event.position);
                                                    let v = lo + t * (hi - lo);
                                                    let v = snap_slider_value(v, lo, hi, step, kind);
                                                    let s =
                                                        format_slider_value(v, kind, gran);
                                                    if let Some((ref em, ref ek)) =
                                                        this.extension_token_editing.clone()
                                                    {
                                                        if em != &module || ek != &key {
                                                            this.flush_extension_token_edit(
                                                                em.clone(),
                                                                ek.clone(),
                                                                cx,
                                                            );
                                                        }
                                                    }
                                                    this.extension_token_editing = None;
                                                    let pair = (module.clone(), key.clone());
                                                    let prev = this
                                                        .extension_token_values
                                                        .get(&pair)
                                                        .cloned();
                                                    if prev.as_deref() == Some(s.as_str()) {
                                                        return;
                                                    }
                                                    this.extension_token_values
                                                        .insert(pair, s);
                                                    this.flush_extension_token_edit(module, key, cx);
                                                    cx.notify();
                                                }
                                            }))
                                            .child(
                                                div()
                                                    .h_full()
                                                    .flex_none()
                                                    .flex_basis(relative(fill_basis))
                                                    .bg(p.accent),
                                            )
                                            .child(div().h_full().flex_1().bg(input_bg)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(subtext_color)
                                            .min_w(px(40.))
                                            .child(display_val.clone()),
                                    ),
                            )
                            .into_any_element()
                            }
                            None => div()
                                .text_xs()
                                .text_color(subtext_color)
                                .child("(invalid numeric default)")
                                .into_any_element(),
                        }
                    } else if !spec.options.is_empty() {
                        let options = spec.options.clone();
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap_2()
                            .children(options.into_iter().map(|opt| {
                                let selected = display_val.trim() == opt.as_str();
                                let m = row_module.clone();
                                let k = row_key.clone();
                                let opt_val = opt.clone();
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(panel_radius))
                                    .border_1()
                                    .border_color(if selected { p.accent } else { input_border })
                                    .bg(if selected { p.accent } else { input_bg })
                                    .text_sm()
                                    .font_weight(if selected { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                    .text_color(if selected { p.on_accent } else { header_color })
                                    .cursor_pointer()
                                    .child(opt.clone())
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _, cx| {
                                        if let Some((ref em, ref ek)) = this.extension_token_editing.clone() {
                                            this.flush_extension_token_edit(em.clone(), ek.clone(), cx);
                                        }
                                        this.extension_token_editing = None;
                                        let pair = (m.clone(), k.clone());
                                        this.extension_token_values.insert(pair, opt_val.clone());
                                        this.flush_extension_token_edit(m.clone(), k.clone(), cx);
                                        cx.notify();
                                    }))
                            }))
                            .into_any_element()
                    } else if is_editing {
                        let show_caret = ext_token_focus.is_focused(window) && is_editing;
                        let blink = self.text_caret_blink_visible;
                        div()
                            .px_3()
                            .py_2()
                            .rounded(px(panel_radius))
                            .bg(input_bg)
                            .border_1()
                            .border_color(input_border)
                            .text_sm()
                            .text_color(header_color)
                            .track_focus(&ext_token_focus)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, window, _| {
                                    this.extension_token_focus.focus(window);
                                }),
                            )
                            .child(div().child(text_with_trailing_caret(
                                display_val.as_str(),
                                show_caret,
                                blink,
                            )))
                            .on_key_down(cx.listener({
                                let m = row_module.clone();
                                let k = row_key.clone();
                                let default_for_esc = row_default.clone();
                                move |this, event: &KeyDownEvent, _, cx| {
                                    let key_ev = event.keystroke.key.as_str();
                                    let mods = event.keystroke.modifiers;
                                    if key_ev == "enter" || key_ev == "return" {
                                        this.flush_extension_token_edit(m.clone(), k.clone(), cx);
                                        this.extension_token_editing = None;
                                        cx.notify();
                                        return;
                                    }
                                    if key_ev == "escape" {
                                        let file = arcadia_core::config::extension_tokens::load_module_tokens(&m)
                                            .unwrap_or_default();
                                        let v = arcadia_core::config::extension_tokens::merged_display_for_key(
                                            &k,
                                            &default_for_esc,
                                            &file,
                                        );
                                        this.extension_token_values.insert((m.clone(), k.clone()), v);
                                        this.extension_token_editing = None;
                                        cx.notify();
                                        return;
                                    }
                                    let pair = (m.clone(), k.clone());
                                    let entry = this.extension_token_values.entry(pair.clone()).or_insert_with(String::new);
                                    if key_ev == "backspace" {
                                        entry.pop();
                                        cx.notify();
                                    } else if key_ev == "space" {
                                        entry.push(' ');
                                        cx.notify();
                                    } else if !mods.control
                                        && !mods.alt
                                        && !mods.platform
                                        && !mods.function
                                    {
                                        if let Some(ch) = &event.keystroke.key_char {
                                            entry.push_str(ch);
                                            cx.notify();
                                        }
                                    }
                                }
                            }))
                            .into_any_element()
                    } else {
                        div()
                            .px_3()
                            .py_2()
                            .rounded(px(panel_radius))
                            .bg(input_bg)
                            .border_1()
                            .border_color(input_border)
                            .text_sm()
                            .text_color(header_color)
                            .cursor_pointer()
                            .child(div().child(display_val.clone()))
                            .on_mouse_down(MouseButton::Left, cx.listener({
                                let m = row_module.clone();
                                let k = row_key.clone();
                                move |this, _, window, cx| {
                                    if let Some((ref em, ref ek)) = this.extension_token_editing.clone() {
                                        if em != &m || ek != &k {
                                            this.flush_extension_token_edit(em.clone(), ek.clone(), cx);
                                        }
                                    }
                                    this.extension_token_editing = Some((m.clone(), k.clone()));
                                    this.extension_token_focus.focus(window);
                                    cx.notify();
                                }
                            }))
                            .into_any_element()
                    };
                    edit_cell
                })
            });

        let mut outer = div()
            .w_full()
            .p_4()
            .rounded(px(panel_radius))
            .bg(panel_bg)
            .border_1()
            .border_color(panel_stroke)
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(header_color)
                            .child(title.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(subtext_color)
                            .child(help.clone()),
                    ),
            );
        if let Some(lbl) = extension_label {
            outer = outer.child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(subtext_color)
                    .child(format!("Extension: {lbl}")),
            );
        }
        outer.children(rows)
    }
}
