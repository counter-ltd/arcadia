//! Popup color picker modal with preview — appears as an overlay without displacing content.

use openframe::{
    div, px, rgb, Context, InteractiveElement, IntoElement, MouseButton, ParentElement, Rgba,
    Styled,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

fn parse_hex_to_rgba(hex: &str) -> Option<Rgba> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Rgba {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: 1.,
        })
    } else {
        None
    }
}

fn rgba_to_hex(r: Rgba) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (r.r * 255.) as u8,
        (r.g * 255.) as u8,
        (r.b * 255.) as u8,
    )
}

impl ArcadiaRoot {
    /// Renders the color picker modal overlay if active.
    pub fn color_picker_modal(
        &mut self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> impl IntoElement {
        let Some((ref module, ref key, ref current_hex, ref default_hex)) = self.color_picker_modal
        else {
            return div().into_any_element();
        };
        let module = module.clone();
        let key = key.clone();
        let current_hex = current_hex.clone();
        let default_hex = default_hex.clone();

        let surface = theme::ui_surface(cx, is_dark);
        let surface2 = theme::ui_surface2(cx, is_dark);
        let border = theme::ui_border(cx, is_dark);
        let text = theme::ui_text(cx, is_dark);
        let subtext = theme::ui_subtext(cx, is_dark);
        let accent = theme::ui_accent(cx);
        let accent_fg = theme::ui_accent_fg(cx);
        let radius = theme::ui_radius(cx);

        // Parse current color or fall back to default
        let rgba = parse_hex_to_rgba(&current_hex)
            .or_else(|| parse_hex_to_rgba(&default_hex))
            .unwrap_or_else(|| rgb(0x00cc88));

        // Color preview square
        let preview_color = rgba;

        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .child(
                // Backdrop to close on click
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .bg(rgb(0x000000))
                    .opacity(0.25)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.color_picker_modal = None;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                // Modal container — centered
                div()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .child(
                        div()
                            .p_5()
                            .rounded(px(radius))
                            .bg(surface)
                            .border_1()
                            .border_color(border)
                            .flex()
                            .flex_col()
                            .gap_4()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_, _, _, cx| {
                                    cx.stop_propagation();
                                }),
                            )
                            .child(
                                // Header
                                div()
                                    .text_lg()
                                    .font_weight(openframe::FontWeight::BOLD)
                                    .text_color(text)
                                    .child("Pick a color"),
                            )
                            .child(
                                // Color preview row
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_3()
                                    .items_center()
                                    .child(
                                        div()
                                            .w(px(48.))
                                            .h(px(48.))
                                            .rounded(px(radius))
                                            .border_2()
                                            .border_color(border)
                                            .bg(preview_color),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(openframe::FontWeight::SEMIBOLD)
                                                    .text_color(text)
                                                    .child("Preview"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(subtext)
                                                    .child(current_hex.clone()),
                                            ),
                                    ),
                            )
                            .child(
                                // Color picker
                                openframe::color_picker_with_weak(
                                    cx.weak_entity(),
                                    rgba,
                                    {
                                        let m = module.clone();
                                        let k = key.clone();
                                        move |this, picked: Rgba, _cx| {
                                            let hex = rgba_to_hex(picked);
                                            // If editing a specific gradient stop, patch that stop's
                                            // color in the gradient JSON rather than replacing the
                                            // whole token value.
                                            if let Some(idx) = this.gradient_stop_editing_index {
                                                use openframe::{
                                                    parse_gradient_stops, stops_to_json,
                                                };
                                                let pair = (m.clone(), k.clone());
                                                let current = this
                                                    .extension_token_values
                                                    .get(&pair)
                                                    .cloned()
                                                    .unwrap_or_default();
                                                let mut stops = parse_gradient_stops(&current);
                                                if idx < stops.len() {
                                                    stops[idx].color = hex.clone();
                                                }
                                                let new_json = stops_to_json(&stops);
                                                this.extension_token_values.insert(pair, new_json.clone());
                                                this.color_picker_modal = Some((
                                                    m.clone(),
                                                    k.clone(),
                                                    hex,
                                                    this.color_picker_modal
                                                        .as_ref()
                                                        .map(|(_, _, _, d)| d.clone())
                                                        .unwrap_or_default(),
                                                ));
                                            } else {
                                                this.extension_token_values
                                                    .insert((m.clone(), k.clone()), hex.clone());
                                                this.color_picker_modal = Some((
                                                    m.clone(),
                                                    k.clone(),
                                                    hex,
                                                    this.color_picker_modal
                                                        .as_ref()
                                                        .map(|(_, _, _, d)| d.clone())
                                                        .unwrap_or_default(),
                                                ));
                                            }
                                        }
                                    },
                                ),
                            )
                            .child(
                                // Action buttons
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .justify_end()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(radius))
                                            .cursor_pointer()
                                            .bg(surface2)
                                            .text_color(text)
                                            .child("Cancel")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener({
                                                    let m = module.clone();
                                                    let k = key.clone();
                                                    let d = default_hex.clone();
                                                    move |this, _, _, cx| {
                                                        let file = arcadia_core::config::extension_tokens::load_module_tokens(&m)
                                                            .unwrap_or_default();
                                                        let reverted = arcadia_core::config::extension_tokens::merged_display_for_key(
                                                            &k,
                                                            &d,
                                                            &file,
                                                        );
                                                        this.extension_token_values
                                                            .insert((m.clone(), k.clone()), reverted);
                                                        this.color_picker_modal = None;
                                                        this.gradient_stop_editing_index = None;
                                                        cx.notify();
                                                    }
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded(px(radius))
                                            .cursor_pointer()
                                            .bg(accent)
                                            .text_color(accent_fg)
                                            .child("Apply")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener({
                                                    let m = module.clone();
                                                    let k = key.clone();
                                                    move |this, _, _, cx| {
                                                        this.flush_extension_token_edit(
                                                            m.clone(),
                                                            k.clone(),
                                                            cx,
                                                        );
                                                        this.color_picker_modal = None;
                                                        this.gradient_stop_editing_index = None;
                                                        cx.notify();
                                                    }
                                                }),
                                            ),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
}
