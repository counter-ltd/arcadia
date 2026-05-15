use arcadia_core::navigation;
use openframe::prelude::FluentBuilder as _;
use openframe::{
    div, px, rgb, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Rgba,
    SharedString, StatefulInteractiveElement, Styled,
};

use crate::gui::app::navigation::NavPageRef;
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, render_icon, GlyphStyleConfig};

// ---------------------------------------------------------------------------
// Glyph-aware color helpers
// ---------------------------------------------------------------------------

fn nav_idle_bg(g: Option<GlyphStyleConfig>, is_dark: bool) -> Rgba {
    g.as_ref().map(|g| g.surface).unwrap_or_else(|| {
        if is_dark {
            rgb(0x171b22)
        } else {
            rgb(0xf6f7fb)
        }
    })
}
fn nav_active_bg(_g: Option<GlyphStyleConfig>, pal_selected: Rgba) -> Rgba {
    pal_selected
}
pub(super) fn nav_item_bg(idle: Rgba, sel: Rgba, active_alpha: f32, hover_alpha: f32) -> Rgba {
    let idle_hov = lerp_color(idle, sel, 0.4);
    let active_hov = lerp_color(
        sel,
        Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        0.05,
    );
    let base = lerp_color(idle, sel, active_alpha);
    let dest = lerp_color(idle_hov, active_hov, active_alpha);
    lerp_color(base, dest, hover_alpha)
}
fn nav_idle_text(g: Option<GlyphStyleConfig>, is_dark: bool) -> Rgba {
    g.as_ref()
        .map(|g| g.dim)
        .unwrap_or_else(|| theme::sidebar_nav_idle_foreground(is_dark))
}
fn nav_active_text(_g: Option<GlyphStyleConfig>, pal_active: Rgba) -> Rgba {
    pal_active
}
fn nav_radius(g: Option<GlyphStyleConfig>) -> f32 {
    g.as_ref().map(|g| g.border_radius).unwrap_or(6.0)
}

pub fn lerp_color(a: Rgba, b: Rgba, t: f32) -> Rgba {
    use arcadia_core::modules::animation::lerp_f32;
    Rgba {
        r: lerp_f32(a.r, b.r, t),
        g: lerp_f32(a.g, b.g, t),
        b: lerp_f32(a.b, b.b, t),
        a: lerp_f32(a.a, b.a, t),
    }
}

fn transparent() -> Rgba {
    Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    }
}

fn active_border(pal_icon_idle: Rgba, active_alpha: f32) -> Rgba {
    lerp_color(transparent(), pal_icon_idle, active_alpha)
}

pub(super) fn nav_item_text(idle: Rgba, active: Rgba, active_alpha: f32, hover_alpha: f32) -> Rgba {
    let white = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let idle_hov = lerp_color(idle, active, 0.4);
    let active_hov = lerp_color(active, white, 0.05);
    let base = lerp_color(idle, active, active_alpha);
    let dest = lerp_color(idle_hov, active_hov, active_alpha);
    lerp_color(base, dest, hover_alpha)
}

fn ai_provider_accent(module_name: &str) -> &'static str {
    use arcadia_core::config::modules::*;
    match module_name {
        AI_EXEC_CLAUDE_MODULE_NAME => "orange",
        AI_EXEC_GEMINI_MODULE_NAME => "sky",
        AI_EXEC_CODEX_MODULE_NAME => "cyan",
        AI_OPENAI_MODULE_NAME => "emerald",
        AI_EXEC_AIDER_MODULE_NAME => "emerald",
        AI_LLAMA_CPP_MODULE_NAME => "amber",
        AI_OLLAMA_MODULE_NAME => "cyan",
        AI_APFEL_MODULE_NAME => "indigo",
        _ => "violet",
    }
}

// ---------------------------------------------------------------------------

impl ArcadiaRoot {
    pub fn sidebar_toggle_button(
        cx: &mut Context<Self>,
        page_glyph: &str,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
    ) -> impl IntoElement {
        let btn_bg = glyph.as_ref().map(|g| g.surface2).unwrap_or_else(|| {
            if is_dark {
                rgb(0x1f2937)
            } else {
                rgb(0xf3f4f6)
            }
        });
        let btn_hover = glyph.as_ref().map(|g| g.border).unwrap_or_else(|| {
            if is_dark {
                rgb(0x243246)
            } else {
                rgb(0xe5e7eb)
            }
        });
        let icon_col = glyph.as_ref().map(|g| g.text).unwrap_or_else(|| {
            if is_dark {
                rgb(0xe5e7eb)
            } else {
                rgb(0x1f2937)
            }
        });
        let radius = nav_radius(glyph);
        div()
            .w_8()
            .h_8()
            .rounded(px(radius))
            .cursor_pointer()
            .bg(btn_bg)
            .text_color(icon_col)
            .hover(move |s| s.bg(btn_hover))
            .flex()
            .items_center()
            .justify_center()
            .child(render_icon(page_glyph).size_4().text_color(icon_col))
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.sidebar_visible = !this.sidebar_visible;
                    cx.notify();
                }),
            )
            .accessibility_label("Toggle sidebar")
    }

    pub fn sidebar_group_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        group_id: String,
        index: usize,
        _is_active: bool,
        is_dark: bool,
        accent: String,
        glyph: Option<GlyphStyleConfig>,
        hover_alpha: f32,
        active_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent.as_str(), is_dark);
        let idle_bg = nav_idle_bg(glyph, is_dark);
        let sel_bg = nav_active_bg(glyph, pal.row_selected);
        let final_bg = nav_item_bg(idle_bg, sel_bg, active_alpha, hover_alpha);
        let icon_col = lerp_color(
            nav_idle_text(glyph, is_dark),
            nav_active_text(glyph, pal.icon_active),
            active_alpha,
        );
        let border_col = active_border(pal.icon_idle, active_alpha);
        let radius = nav_radius(glyph);
        let gid_hover = group_id.clone();
        div()
            .id(SharedString::from(format!("tab-grp-{}", group_id)))
            .w_16()
            .h_16()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(FontWeight::NORMAL)
            .bg(final_bg)
            .border_1()
            .border_color(border_col)
            .text_color(icon_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_tab_hover_anim(gid_hover.clone(), *hovered);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .text_center()
                    .child(
                        render_icon(system_image.as_ref())
                            .size_5()
                            .text_color(icon_col),
                    )
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.start_tab_scroll_anim(index);
                    this.active_group_id = group_id.clone();
                    if let Some(group) = this.effective_group(group_id.as_str()) {
                        if let Some(first_page_id) = group
                            .page_ids()
                            .into_iter()
                            .find(|pid| this.is_page_visible(pid))
                        {
                            this.active_page_id = first_page_id.to_string();
                        }
                    }
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
    }

    pub fn top_bar_global_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        page_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
        glyph: Option<GlyphStyleConfig>,
        pill_expand_alpha: Option<f32>,
        content_alpha: f32,
        preview_bg_alpha: f32,
        shake_offset: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_col = if is_active {
            nav_active_text(glyph, pal.icon_active)
        } else {
            nav_idle_text(glyph, is_dark)
        };
        let idle_bg = glyph
            .as_ref()
            .map(|g| g.surface)
            .unwrap_or_else(|| theme::top_bar_pill_bg(is_dark));
        let active_bg = nav_active_bg(glyph, pal.row_selected);
        let bg = if is_active {
            active_bg
        } else if preview_bg_alpha > 0.0 {
            // Tint toward active bg during preview; border stays transparent (unlike active).
            lerp_color(idle_bg, active_bg, preview_bg_alpha)
        } else {
            idle_bg
        };
        let hover_bg = if is_active {
            pal.row_hover
        } else {
            glyph
                .as_ref()
                .map(|g| g.surface2)
                .unwrap_or_else(|| theme::top_bar_pill_hover_bg(is_dark))
        };
        let radius = nav_radius(glyph);
        let border_col = if is_active {
            pal.icon_active
        } else {
            openframe::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
        };
        // Horizontal padding expands slightly during preview (8→10 px); height unchanged.
        let pill_px = px(8.0 + preview_bg_alpha * 2.0);
        div()
            .px(pill_px)
            .h_8()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(FontWeight::NORMAL)
            .bg(bg)
            .border_1()
            .border_color(border_col)
            .text_color(icon_col)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .gap_1()
                    .items_center()
                    .child(
                        render_icon(system_image.as_ref())
                            .size_4()
                            .text_color(icon_col)
                            .when(shake_offset.abs() > 0.01, |d| d.ml(px(shake_offset))),
                    )
                    .when(!label.is_empty(), |d| {
                        if let Some(alpha) = pill_expand_alpha {
                            if alpha < 0.01 {
                                d
                            } else {
                                d.child(
                                    div()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .max_w(px(alpha * 120.0))
                                        .opacity(alpha * content_alpha)
                                        .child(label),
                                )
                            }
                        } else {
                            d.child(div().opacity(content_alpha).child(label))
                        }
                    }),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id.clone();
                    if page_id == "global.modules" {
                        this.reload_modules();
                    }
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_global_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        page_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
        glyph: Option<GlyphStyleConfig>,
        hover_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent.as_str(), is_dark);
        let idle_bg = nav_idle_bg(glyph, is_dark);
        let sel_bg = nav_active_bg(glyph, pal.row_selected);
        let icon_col = if is_active {
            nav_active_text(glyph, pal.icon_active)
        } else {
            nav_idle_text(glyph, is_dark)
        };
        let final_bg = nav_item_bg(
            idle_bg,
            sel_bg,
            if is_active { 1.0 } else { 0.0 },
            hover_alpha,
        );
        let border_col = active_border(pal.icon_idle, if is_active { 1.0 } else { 0.0 });
        let radius = nav_radius(glyph);
        let item_key = format!("page:{}", page_id);
        div()
            .id(SharedString::from(item_key.clone()))
            .px_3()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_sm()
            .font_weight(FontWeight::NORMAL)
            .bg(final_bg)
            .border_1()
            .border_color(border_col)
            .text_color(icon_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_item_hover_anim(item_key.clone(), *hovered);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        render_icon(system_image.as_ref())
                            .size_4()
                            .text_color(icon_col),
                    )
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id.clone();
                    if page_id == "global.modules" {
                        this.reload_modules();
                    }
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_settings_hub_section(
        &self,
        cx: &mut Context<Self>,
        hub_page: NavPageRef<'_>,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(hub_page.accent(), is_dark);
        let hub_active = self.active_page_id.as_str() == navigation::SETTINGS_HUB_ROOT_PAGE_ID
            || self
                .settings_hub_page_ids_effective()
                .iter()
                .any(|pid| *pid == self.active_page_id.as_str());
        let hub_idle_bg = nav_idle_bg(glyph, is_dark);
        let hub_sel_bg = nav_active_bg(glyph, pal.row_selected);
        let icon_col = if hub_active {
            nav_active_text(glyph, pal.icon_active)
        } else {
            nav_idle_text(glyph, is_dark)
        };
        let hub_ha = *self
            .item_hover_alphas
            .get("page:global.settings")
            .unwrap_or(&0.0);
        let hub_final_bg = nav_item_bg(
            hub_idle_bg,
            hub_sel_bg,
            if hub_active { 1.0 } else { 0.0 },
            hub_ha,
        );
        let hub_border = active_border(pal.icon_idle, if hub_active { 1.0 } else { 0.0 });
        let radius = nav_radius(glyph);
        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .child(
                div()
                    .id(SharedString::from("page:global.settings"))
                    .px_3()
                    .py_1()
                    .rounded(px(radius))
                    .cursor_pointer()
                    .text_sm()
                    .font_weight(FontWeight::NORMAL)
                    .bg(hub_final_bg)
                    .border_1()
                    .border_color(hub_border)
                    .text_color(icon_col)
                    .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                        this.start_item_hover_anim("page:global.settings".to_string(), *hovered);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                render_icon(hub_page.glyph()).size_4().text_color(icon_col),
                            )
                            .child(div().child(hub_page.title().to_string())),
                    )
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.active_page_id = navigation::SETTINGS_HUB_ROOT_PAGE_ID.to_string();
                            this.sync_settings_hub_expanded_from_active_page();
                            cx.notify();
                        }),
                    ),
            )
            .child({
                let visible_count = self.pinned_settings_pages.iter()
                    .filter(|pid| self.is_page_visible(pid))
                    .count();
                let target_h = visible_count as f32 * 28.0;
                let animated_h = self.settings_expand_alpha * target_h;
                let pinned: Vec<String> = self.pinned_settings_pages.clone();
                div()
                    .overflow_hidden()
                    .h(px(animated_h))
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .pl_4()
                    .children(pinned.into_iter().filter_map(|page_id| {
                        if !self.is_page_visible(&page_id) {
                            return None;
                        }
                        let page = self.page_ref(&page_id)?;
                        let page_id_owned = page_id.clone();
                        let page_id_rclick = page_id.clone();
                        let is_active = self.active_page_id == page.id();
                        let sub_pal = theme::nav_accent_palette(page.accent(), is_dark);
                        let sub_ha = *self
                            .item_hover_alphas
                            .get(&format!("page:{}", page.id()))
                            .unwrap_or(&0.0);
                        let sub_icon = nav_item_text(
                            nav_idle_text(glyph, is_dark),
                            nav_active_text(glyph, sub_pal.icon_active),
                            if is_active { 1.0 } else { 0.0 },
                            sub_ha,
                        );
                        let title = page.title().to_string();
                        let glyph_key = page.glyph().to_string();
                        let sub_item_key = format!("page:{}", page.id());
                        Some(
                            div()
                                .id(SharedString::from(sub_item_key.clone()))
                                .px_3()
                                .py_1()
                                .rounded(px(radius))
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::NORMAL)
                                .text_color(sub_icon)
                                .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                                    this.start_item_hover_anim(sub_item_key.clone(), *hovered);
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .items_center()
                                        .child(
                                            render_icon(&glyph_key).size_4().text_color(sub_icon),
                                        )
                                        .child(div().child(title)),
                                )
                                .on_mouse_down(
                                    openframe::MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.active_page_id = page_id_owned.clone();
                                        if page_id_owned == "global.modules" {
                                            this.reload_modules();
                                        }
                                        this.sync_settings_hub_expanded_from_active_page();
                                        cx.notify();
                                    }),
                                )
                                .on_mouse_down(
                                    openframe::MouseButton::Right,
                                    cx.listener(
                                        move |this, event: &openframe::MouseDownEvent, _, cx| {
                                            this.settings_pin_context_menu =
                                                Some((page_id_rclick.clone(), event.position));
                                            cx.notify();
                                        },
                                    ),
                                )
                                .into_any_element(),
                        )
                    }))
            })
    }

    pub fn sidebar_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        page_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
        glyph: Option<GlyphStyleConfig>,
        hover_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent.as_str(), is_dark);
        let idle_bg = nav_idle_bg(glyph, is_dark);
        let sel_bg = nav_active_bg(glyph, pal.row_selected);
        let icon_col = if is_active {
            nav_active_text(glyph, pal.icon_active)
        } else {
            nav_idle_text(glyph, is_dark)
        };
        let final_bg = nav_item_bg(
            idle_bg,
            sel_bg,
            if is_active { 1.0 } else { 0.0 },
            hover_alpha,
        );
        let border_col = active_border(pal.icon_idle, if is_active { 1.0 } else { 0.0 });
        let radius = nav_radius(glyph);
        let _is_shell_page = page_id == "utility.shell";
        let _is_editor_page = page_id == "editor.main";
        let _is_ai_page = page_id == "ai.chat";
        let page_id_left = page_id.clone();
        let item_key = format!("page:{}", page_id);
        div()
            .id(SharedString::from(item_key.clone()))
            .px_3()
            .py_2()
            .rounded(px(radius))
            .cursor_pointer()
            .text_sm()
            .font_weight(FontWeight::NORMAL)
            .bg(final_bg)
            .border_1()
            .border_color(border_col)
            .text_color(icon_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_item_hover_anim(item_key.clone(), *hovered);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        render_icon(system_image.as_ref())
                            .size_4()
                            .text_color(icon_col),
                    )
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id_left.clone();
                    #[cfg(feature = "gui")]
                    if _is_editor_page {
                        this.code_editor_show_dashboard = true;
                    }
                    #[cfg(feature = "gui")]
                    if _is_shell_page && this.terminals.len() > 1 {
                        this.terminal_show_dashboard = true;
                    }
                    if _is_ai_page {
                        this.ai_chat_show_dashboard = true;
                    }
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
            .on_mouse_down(
                openframe::MouseButton::Right,
                cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                    #[cfg(feature = "gui")]
                    if _is_shell_page {
                        this.terminal_context_menu_open = true;
                        this.terminal_kill_menu = None;
                        this.session_route_menu_open = false;
                        this.context_menu_position = event.position;
                        cx.notify();
                    }
                    #[cfg(feature = "gui")]
                    if _is_editor_page {
                        this.code_editor_context_menu_open = true;
                        this.terminal_context_menu_open = false;
                        this.terminal_kill_menu = None;
                        this.session_route_menu_open = false;
                        this.context_menu_position = event.position;
                        cx.notify();
                    }
                    if _is_ai_page {
                        this.ai_context_menu_open = true;
                        #[cfg(feature = "gui")]
                        {
                            this.terminal_context_menu_open = false;
                            this.terminal_kill_menu = None;
                            this.session_route_menu_open = false;
                            this.code_editor_context_menu_open = false;
                            this.context_menu_position = event.position;
                        }
                        cx.notify();
                    }
                    #[cfg(not(feature = "gui"))]
                    let _ = (this, event, cx);
                }),
            )
    }

    #[cfg(feature = "gui")]
    pub fn sidebar_code_editor_sub_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        workspace_label: Option<String>,
        editor_idx: usize,
        is_active: bool,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
        hover_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette("sky", is_dark);
        let text_col = nav_item_text(
            nav_idle_text(glyph, is_dark),
            nav_active_text(glyph, pal.icon_active),
            if is_active { 1.0 } else { 0.0 },
            hover_alpha,
        );
        let meta_col = if is_active {
            pal.icon_active
        } else if is_dark {
            rgb(0x4a5568)
        } else {
            rgb(0x9ca3af)
        };
        let radius = nav_radius(glyph);
        let item_key = format!("editor:{}", editor_idx);
        div()
            .id(SharedString::from(item_key.clone()))
            .ml_7()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(openframe::FontWeight::NORMAL)
            .text_color(text_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_item_hover_anim(item_key.clone(), *hovered);
                cx.notify();
            }))
            .flex()
            .flex_col()
            .gap(px(1.))
            .child(div().child(label))
            .when_some(workspace_label, |d, ws| {
                d.child(div().text_color(meta_col).child(ws))
            })
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if editor_idx < this.code_editor_tabs.len() {
                        this.active_code_editor_tab = editor_idx;
                        this.active_page_id = "editor.main".to_string();
                        this.code_editor_show_dashboard = false;
                        this.sync_settings_hub_expanded_from_active_page();
                    }
                    cx.notify();
                }),
            )
            .on_mouse_down(
                openframe::MouseButton::Right,
                cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                    this.code_editor_tab_menu = Some((editor_idx, event.position));
                    this.code_editor_context_menu_open = false;
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_ai_provider_sub_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        module_name: String,
        is_active: bool,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
        icon_key: &'static str,
        hover_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(ai_provider_accent(&module_name), is_dark);
        let text_col = nav_item_text(
            nav_idle_text(glyph, is_dark),
            nav_active_text(glyph, pal.icon_active),
            if is_active { 1.0 } else { 0.0 },
            hover_alpha,
        );
        let radius = nav_radius(glyph);
        let item_key = format!("aiprov:{}", module_name);
        let module_name_right = module_name.clone();
        div()
            .id(SharedString::from(item_key.clone()))
            .ml_7()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(openframe::FontWeight::NORMAL)
            .text_color(text_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_item_hover_anim(item_key.clone(), *hovered);
                cx.notify();
            }))
            .flex()
            .items_center()
            .gap_1p5()
            .child(render_icon(icon_key).size(px(12.)).text_color(text_col))
            .child(div().child(label))
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_ai_provider_module = module_name.clone();
                    this.active_llama_cpp_model_id = None;
                    this.active_page_id = "ai.models".to_string();
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
            .on_mouse_down(
                openframe::MouseButton::Right,
                cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                    if module_name_right == arcadia_core::config::modules::AI_LLAMA_CPP_MODULE_NAME
                    {
                        this.llama_cpp_provider_menu = Some(event.position);
                        this.ollama_provider_menu = None;
                        this.ai_context_menu_open = false;
                        this.ai_chat_menu = None;
                        #[cfg(feature = "gui")]
                        {
                            this.terminal_context_menu_open = false;
                            this.terminal_kill_menu = None;
                            this.code_editor_context_menu_open = false;
                            this.context_menu_position = event.position;
                        }
                        cx.notify();
                    } else if module_name_right == arcadia_core::config::modules::AI_OLLAMA_MODULE_NAME
                    {
                        this.ollama_provider_menu = Some(event.position);
                        this.llama_cpp_provider_menu = None;
                        this.ai_context_menu_open = false;
                        this.ai_chat_menu = None;
                        #[cfg(feature = "gui")]
                        {
                            this.terminal_context_menu_open = false;
                            this.terminal_kill_menu = None;
                            this.code_editor_context_menu_open = false;
                            this.context_menu_position = event.position;
                        }
                        cx.notify();
                    }
                }),
            )
    }

    pub fn sidebar_llama_cpp_model_sub_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        model_id: String,
        is_active: bool,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
        icon_key: &'static str,
        hover_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(
            ai_provider_accent(arcadia_core::config::modules::AI_LLAMA_CPP_MODULE_NAME),
            is_dark,
        );
        let text_col = nav_item_text(
            nav_idle_text(glyph, is_dark),
            nav_active_text(glyph, pal.icon_active),
            if is_active { 1.0 } else { 0.0 },
            hover_alpha,
        );
        let radius = nav_radius(glyph);
        let item_key = format!("llamamod:{}", model_id);
        div()
            .id(SharedString::from(item_key.clone()))
            .ml_12()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(openframe::FontWeight::NORMAL)
            .text_color(text_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_item_hover_anim(item_key.clone(), *hovered);
                cx.notify();
            }))
            .flex()
            .items_center()
            .gap_1p5()
            .child(render_icon(icon_key).size(px(11.)).text_color(text_col))
            .child(div().child(label))
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_llama_cpp_model_id = Some(model_id.clone());
                    this.active_ai_provider_module =
                        arcadia_core::config::modules::AI_LLAMA_CPP_MODULE_NAME.to_string();
                    this.active_page_id = "ai.models".to_string();
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_cli_provider_sub_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        module_name: String,
        is_active: bool,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
        hover_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(ai_provider_accent(&module_name), is_dark);
        let text_col = nav_item_text(
            nav_idle_text(glyph, is_dark),
            nav_active_text(glyph, pal.icon_active),
            if is_active { 1.0 } else { 0.0 },
            hover_alpha,
        );
        let provider_icon = arcadia_core::config::modules::MODULE_REGISTRY
            .iter()
            .find(|m| m.name == module_name.as_str())
            .map(|m| m.glyph)
            .unwrap_or("modules");
        let radius = nav_radius(glyph);
        let item_key = format!("cliprov:{}", module_name);
        div()
            .id(SharedString::from(item_key.clone()))
            .ml_12()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(openframe::FontWeight::NORMAL)
            .text_color(text_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_item_hover_anim(item_key.clone(), *hovered);
                cx.notify();
            }))
            .flex()
            .items_center()
            .gap_1p5()
            .child(
                render_icon(provider_icon)
                    .size(px(11.))
                    .text_color(text_col),
            )
            .child(div().child(label))
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_ai_provider_module = module_name.clone();
                    this.active_llama_cpp_model_id = None;
                    this.active_page_id = "ai.models".to_string();
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
    }

    pub fn sidebar_ai_chat_sub_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        chat_id: usize,
        is_active: bool,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
        accent: &'static str,
        provider_icon: &'static str,
        workspace_label: Option<String>,
        hover_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette(accent, is_dark);
        let text_col = nav_item_text(
            nav_idle_text(glyph, is_dark),
            nav_active_text(glyph, pal.icon_active),
            if is_active { 1.0 } else { 0.0 },
            hover_alpha,
        );
        let radius = nav_radius(glyph);
        let meta_col = if is_active {
            pal.icon_active
        } else if is_dark {
            rgb(0x4a5568)
        } else {
            rgb(0x9ca3af)
        };
        let item_key = format!("chat:{}", chat_id);
        div()
            .id(SharedString::from(item_key.clone()))
            .ml_7()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(openframe::FontWeight::NORMAL)
            .text_color(text_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_item_hover_anim(item_key.clone(), *hovered);
                cx.notify();
            }))
            .flex()
            .flex_col()
            .gap(px(1.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .items_center()
                    .child(render_icon(provider_icon).size_3().text_color(text_col))
                    .child(div().child(label)),
            )
            .when_some(workspace_label, |d, ws| {
                d.child(div().text_color(meta_col).child(ws))
            })
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_ai_chat_id = chat_id;
                    this.active_page_id = "ai.chat".to_string();
                    this.ai_chat_show_dashboard = false;
                    this.sync_settings_hub_expanded_from_active_page();
                    cx.notify();
                }),
            )
            .on_mouse_down(
                openframe::MouseButton::Right,
                cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                    this.ai_chat_menu = Some((chat_id, event.position));
                    this.ai_context_menu_open = false;
                    cx.notify();
                }),
            )
    }

    #[cfg(feature = "gui")]
    pub fn sidebar_sub_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        terminal_id: usize,
        is_active: bool,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
        hover_alpha: f32,
    ) -> impl IntoElement {
        let pal = theme::nav_accent_palette("emerald", is_dark);
        let text_col = nav_item_text(
            nav_idle_text(glyph, is_dark),
            nav_active_text(glyph, pal.icon_active),
            if is_active { 1.0 } else { 0.0 },
            hover_alpha,
        );
        let radius = nav_radius(glyph);
        let item_key = format!("terminal:{}", terminal_id);
        div()
            .id(SharedString::from(item_key.clone()))
            .ml_7()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(FontWeight::NORMAL)
            .text_color(text_col)
            .on_hover(cx.listener(move |this, hovered: &bool, _, cx| {
                this.start_item_hover_anim(item_key.clone(), *hovered);
                cx.notify();
            }))
            .child(div().child(label))
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    #[cfg(feature = "gui")]
                    if terminal_id < this.terminals.len() {
                        this.active_terminal_id = terminal_id;
                        this.active_page_id = "utility.shell".to_string();
                        this.terminal_show_dashboard = false;
                        this.sync_settings_hub_expanded_from_active_page();
                    }
                    cx.notify();
                }),
            )
            .on_mouse_down(
                openframe::MouseButton::Right,
                cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                    #[cfg(feature = "gui")]
                    {
                        this.terminal_kill_menu = Some(terminal_id);
                        this.terminal_context_menu_open = false;
                        this.session_route_menu_open = false;
                        this.context_menu_position = event.position;
                    }
                    #[cfg(not(feature = "gui"))]
                    let _ = (this, event, terminal_id);
                    cx.notify();
                }),
            )
    }
}
