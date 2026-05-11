use arcadia_core::navigation;
use openframe::{div, px, rgb, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Rgba, Styled};

use crate::gui::app::navigation::NavPageRef;
use crate::gui::app::ArcadiaRoot;
use crate::gui::theme::{self, render_icon, GlyphStyleConfig};

// ---------------------------------------------------------------------------
// Glyph-aware color helpers
// ---------------------------------------------------------------------------

fn nav_idle_bg(g: Option<GlyphStyleConfig>, is_dark: bool) -> Rgba {
    g.as_ref().map(|g| g.surface).unwrap_or_else(|| if is_dark { rgb(0x171b22) } else { rgb(0xf6f7fb) })
}
fn nav_active_bg(_g: Option<GlyphStyleConfig>, pal_selected: Rgba) -> Rgba {
    pal_selected
}
fn nav_hover_bg_raw(_g: Option<GlyphStyleConfig>, _is_dark: bool, fallback: Rgba) -> Rgba {
    fallback
}
fn nav_idle_text(g: Option<GlyphStyleConfig>, is_dark: bool) -> Rgba {
    g.as_ref().map(|g| g.dim).unwrap_or_else(|| theme::sidebar_nav_idle_foreground(is_dark))
}
fn nav_active_text(_g: Option<GlyphStyleConfig>, pal_active: Rgba) -> Rgba {
    pal_active
}
fn nav_radius(g: Option<GlyphStyleConfig>) -> f32 {
    g.as_ref().map(|g| g.border_radius).unwrap_or(6.0)
}

// ---------------------------------------------------------------------------

impl ArcadiaRoot {
    pub fn sidebar_toggle_button(
        cx: &mut Context<Self>,
        page_glyph: &str,
        is_dark: bool,
        glyph: Option<GlyphStyleConfig>,
    ) -> impl IntoElement {
        let btn_bg    = glyph.as_ref().map(|g| g.surface2).unwrap_or_else(|| if is_dark { rgb(0x1f2937) } else { rgb(0xf3f4f6) });
        let btn_hover = glyph.as_ref().map(|g| g.border).unwrap_or_else(|| if is_dark { rgb(0x243246) } else { rgb(0xe5e7eb) });
        let icon_col  = glyph.as_ref().map(|g| g.text).unwrap_or_else(|| if is_dark { rgb(0xe5e7eb) } else { rgb(0x1f2937) });
        let radius    = nav_radius(glyph);
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
    }

    pub fn sidebar_group_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        group_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
        glyph: Option<GlyphStyleConfig>,
    ) -> impl IntoElement {
        let pal        = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_col   = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let label_col  = icon_col;
        let bg         = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg   = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x243246) } else { rgb(0xeef2ff) })
        };
        let radius     = nav_radius(glyph);
        div()
            .w_16()
            .h_16()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
            .bg(bg)
            .text_color(label_col)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_1()
                    .text_center()
                    .child(render_icon(system_image.as_ref()).size_5().text_color(icon_col))
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
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
    ) -> impl IntoElement {
        let pal       = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_col  = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg        = if is_active {
            nav_active_bg(glyph, pal.row_selected)
        } else {
            glyph.as_ref().map(|g| g.surface).unwrap_or_else(|| theme::top_bar_pill_bg(is_dark))
        };
        let hover_bg  = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            glyph.as_ref().map(|g| g.surface2).unwrap_or_else(|| theme::top_bar_pill_hover_bg(is_dark))
        };
        let radius    = nav_radius(glyph);
        div()
            .px_2()
            .py_0p5()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
            .bg(bg)
            .text_color(icon_col)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .gap_1()
                    .items_center()
                    .child(render_icon(system_image.as_ref()).size_4().text_color(icon_col))
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

    pub fn sidebar_global_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        page_id: String,
        is_active: bool,
        is_dark: bool,
        accent: String,
        glyph: Option<GlyphStyleConfig>,
    ) -> impl IntoElement {
        let pal       = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_col  = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg        = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg  = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x243246) } else { rgb(0xeef2ff) })
        };
        let radius    = nav_radius(glyph);
        div()
            .px_3()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_sm()
            .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
            .bg(bg)
            .text_color(icon_col)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(render_icon(system_image.as_ref()).size_4().text_color(icon_col))
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
        let icon_col  = if hub_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg        = if hub_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg  = if hub_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x243246) } else { rgb(0xeef2ff) })
        };
        let expand_border = glyph.as_ref().map(|g| g.border).unwrap_or_else(|| if is_dark { rgb(0x374151) } else { rgb(0xe5e7eb) });
        let chevron_col   = nav_idle_text(glyph, is_dark);
        let chevron       = if self.settings_hub_expanded { "▼" } else { "▶" };
        let radius        = nav_radius(glyph);
        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded(px(radius))
                    .cursor_pointer()
                    .text_sm()
                    .font_weight(if hub_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                    .bg(bg)
                    .text_color(icon_col)
                    .hover(move |s| s.bg(hover_bg))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(render_icon(hub_page.glyph()).size_4().text_color(icon_col))
                                    .child(div().child(hub_page.title().to_string())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(chevron_col)
                                    .child(chevron),
                            ),
                    )
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.active_page_id =
                                navigation::SETTINGS_HUB_ROOT_PAGE_ID.to_string();
                            this.settings_hub_expanded = !this.settings_hub_expanded;
                            cx.notify();
                        }),
                    ),
            )
            .child(if self.settings_hub_expanded {
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .pl_4()
                    .border_l_2()
                    .border_color(expand_border)
                    .children(
                        self.settings_hub_page_ids_effective()
                            .into_iter()
                            .filter_map(|page_id| {
                                if !self.is_page_visible(page_id) {
                                    return None;
                                }
                                let page = self.page_ref(page_id)?;
                                let page_id_owned = page_id.to_string();
                                let is_active = self.active_page_id == page.id();
                                let sub_pal   = theme::nav_accent_palette(page.accent(), is_dark);
                                let sub_icon  = if is_active { nav_active_text(glyph, sub_pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
                                let sub_bg    = if is_active { nav_active_bg(glyph, sub_pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
                                let sub_hover = if is_active {
                                    nav_hover_bg_raw(glyph, is_dark, sub_pal.row_hover)
                                } else {
                                    nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x243246) } else { rgb(0xeef2ff) })
                                };
                                Some(
                                    div()
                                        .px_3()
                                        .py_1()
                                        .rounded(px(radius))
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .bg(sub_bg)
                                        .text_color(sub_icon)
                                        .hover(move |s| s.bg(sub_hover))
                                        .child(
                                            div()
                                                .flex()
                                                .gap_2()
                                                .items_center()
                                                .child(render_icon(page.glyph()).size_4().text_color(sub_icon))
                                                .child(div().child(page.title().to_string())),
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
                                        .into_any_element(),
                                )
                            }),
                    )
            } else {
                div().hidden()
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
    ) -> impl IntoElement {
        let pal      = theme::nav_accent_palette(accent.as_str(), is_dark);
        let icon_col = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg       = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x243246) } else { rgb(0xeef2ff) })
        };
        let radius   = nav_radius(glyph);
        let _is_shell_page = page_id == "utility.shell";
        let page_id_left   = page_id.clone();
        div()
            .px_3()
            .py_2()
            .rounded(px(radius))
            .cursor_pointer()
            .text_sm()
            .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
            .bg(bg)
            .text_color(icon_col)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(render_icon(system_image.as_ref()).size_4().text_color(icon_col))
                    .child(div().child(label)),
            )
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.active_page_id = page_id_left.clone();
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
                    #[cfg(not(feature = "gui"))]
                    let _ = (this, event, cx);
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
    ) -> impl IntoElement {
        let pal      = theme::nav_accent_palette("emerald", is_dark);
        let text_col = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg       = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x1e2530) } else { rgb(0xeef2ff) })
        };
        let radius   = nav_radius(glyph);
        div()
            .ml_7()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active { FontWeight::MEDIUM } else { FontWeight::NORMAL })
            .bg(bg)
            .text_color(text_col)
            .hover(move |s| s.bg(hover_bg))
            .child(div().child(label))
            .on_mouse_down(
                openframe::MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    #[cfg(feature = "gui")]
                    if terminal_id < this.terminals.len() {
                        this.active_terminal_id = terminal_id;
                        this.active_page_id = "utility.shell".to_string();
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
