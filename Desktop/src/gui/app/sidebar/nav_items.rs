use arcadia_core::navigation;
use openframe::{div, px, rgb, Context, FontWeight, InteractiveElement, IntoElement, ParentElement, Rgba, ScrollHandle, SharedString, StatefulInteractiveElement, Styled};
use openframe::prelude::FluentBuilder as _;

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

fn lerp_color(a: Rgba, b: Rgba, t: f32) -> Rgba {
    use arcadia_core::modules::animation::lerp_f32;
    Rgba { r: lerp_f32(a.r, b.r, t), g: lerp_f32(a.g, b.g, t), b: lerp_f32(a.b, b.b, t), a: lerp_f32(a.a, b.a, t) }
}

fn ai_provider_accent(module_name: &str) -> &'static str {
    use arcadia_core::config::modules::*;
    match module_name {
        AI_EXEC_CLAUDE_MODULE_NAME              => "orange",
        AI_EXEC_GEMINI_MODULE_NAME              => "sky",
        AI_EXEC_CODEX_MODULE_NAME | AI_OPENAI_MODULE_NAME => "emerald",
        AI_EXEC_AIDER_MODULE_NAME               => "teal",
        AI_LLAMA_CPP_MODULE_NAME                => "amber",
        AI_OLLAMA_MODULE_NAME                   => "cyan",
        AI_APFEL_MODULE_NAME                    => "indigo",
        _                                       => "violet",
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
            .accessibility_label("Toggle sidebar")
    }

    pub fn sidebar_group_item(
        cx: &mut Context<Self>,
        label: openframe::SharedString,
        system_image: openframe::SharedString,
        group_id: String,
        index: usize,
        is_active: bool,
        is_dark: bool,
        accent: String,
        glyph: Option<GlyphStyleConfig>,
        hover_alpha: f32,
        active_alpha: f32,
    ) -> impl IntoElement {
        let pal        = theme::nav_accent_palette(accent.as_str(), is_dark);
        let idle_bg       = nav_idle_bg(glyph, is_dark);
        let sel_bg        = nav_active_bg(glyph, pal.row_selected);
        // Hover target sits 40% toward active — always dimmer than the fully active state.
        let hover_target  = lerp_color(idle_bg, sel_bg, 0.4);
        let base_bg       = lerp_color(idle_bg, sel_bg, active_alpha);
        let final_bg      = lerp_color(base_bg, hover_target, hover_alpha);
        let icon_col   = lerp_color(nav_idle_text(glyph, is_dark), nav_active_text(glyph, pal.icon_active), active_alpha);
        let radius     = nav_radius(glyph);
        let gid_hover  = group_id.clone();
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
            .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
            .bg(final_bg)
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
                    .child(render_icon(system_image.as_ref()).size_5().text_color(icon_col))
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
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
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
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        };
        let chevron_col   = nav_idle_text(glyph, is_dark);
        let chevron       = if self.settings_hub_expanded { "▼" } else { "▶" };
        let radius        = nav_radius(glyph);
        let has_pinned = self.pinned_settings_pages.iter()
            .any(|pid| self.is_page_visible(pid));
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
                            .when(has_pinned, |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(chevron_col)
                                        .child(chevron),
                                )
                            }),
                    )
                    .on_mouse_down(
                        openframe::MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.active_page_id =
                                navigation::SETTINGS_HUB_ROOT_PAGE_ID.to_string();
                            if has_pinned {
                                this.settings_hub_expanded = !this.settings_hub_expanded;
                            }
                            cx.notify();
                        }),
                    ),
            )
            .child(if self.settings_hub_expanded {
                let pinned: Vec<String> = self.pinned_settings_pages.clone();
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .pl_4()
                    .children(
                        pinned
                            .into_iter()
                            .filter_map(|page_id| {
                                if !self.is_page_visible(&page_id) {
                                    return None;
                                }
                                let page = self.page_ref(&page_id)?;
                                let page_id_owned = page_id.clone();
                                let page_id_rclick = page_id.clone();
                                let is_active = self.active_page_id == page.id();
                                let sub_pal   = theme::nav_accent_palette(page.accent(), is_dark);
                                let sub_icon  = if is_active { nav_active_text(glyph, sub_pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
                                let sub_bg    = if is_active { nav_active_bg(glyph, sub_pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
                                let sub_hover = nav_hover_bg_raw(glyph, is_dark, sub_pal.row_hover);
                                let title = page.title().to_string();
                                let glyph_key = page.glyph().to_string();
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
                                                .child(render_icon(&glyph_key).size_4().text_color(sub_icon))
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
                                            cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                                                this.settings_pin_context_menu = Some((page_id_rclick.clone(), event.position));
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
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        };
        let radius   = nav_radius(glyph);
        let _is_shell_page  = page_id == "utility.shell";
        let _is_editor_page = page_id == "editor.main";
        let _is_ai_page     = page_id == "ai.chat";
        let page_id_left    = page_id.clone();
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
    ) -> impl IntoElement {
        let pal      = theme::nav_accent_palette("sky", is_dark);
        let text_col = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let meta_col = if is_active { pal.icon_active } else if is_dark { rgb(0x4a5568) } else { rgb(0x9ca3af) };
        let bg       = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x1e2530) } else { rgb(0xe0f2fe) })
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
            .font_weight(if is_active { openframe::FontWeight::MEDIUM } else { openframe::FontWeight::NORMAL })
            .bg(bg)
            .text_color(text_col)
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .flex_col()
            .gap(px(1.))
            .child(div().child(label))
            .when_some(workspace_label, |d, ws| {
                d.child(
                    div()
                        .text_color(meta_col)
                        .child(ws),
                )
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
    ) -> impl IntoElement {
        let pal      = theme::nav_accent_palette(ai_provider_accent(&module_name), is_dark);
        let text_col = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg       = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x1e1e2e) } else { rgb(0xede9fe) })
        };
        let radius   = nav_radius(glyph);
        let module_name_right = module_name.clone();
        div()
            .ml_7()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active { openframe::FontWeight::MEDIUM } else { openframe::FontWeight::NORMAL })
            .bg(bg)
            .text_color(text_col)
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .items_center()
            .gap_1p5()
            .child(
                render_icon(icon_key)
                    .size(px(12.))
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
            .on_mouse_down(
                openframe::MouseButton::Right,
                cx.listener(move |this, event: &openframe::MouseDownEvent, _, cx| {
                    if module_name_right == arcadia_core::config::modules::AI_LLAMA_CPP_MODULE_NAME {
                        this.llama_cpp_provider_menu = Some(event.position);
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
    ) -> impl IntoElement {
        let pal      = theme::nav_accent_palette(ai_provider_accent(arcadia_core::config::modules::AI_LLAMA_CPP_MODULE_NAME), is_dark);
        let text_col = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg       = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x1a1a2a) } else { rgb(0xf5f3ff) })
        };
        let radius   = nav_radius(glyph);
        div()
            .ml_12()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active { openframe::FontWeight::MEDIUM } else { openframe::FontWeight::NORMAL })
            .bg(bg)
            .text_color(text_col)
            .hover(move |s| s.bg(hover_bg))
            .flex()
            .items_center()
            .gap_1p5()
            .child(
                render_icon(icon_key)
                    .size(px(11.))
                    .text_color(text_col),
            )
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
    ) -> impl IntoElement {
        let pal      = theme::nav_accent_palette(ai_provider_accent(&module_name), is_dark);
        let text_col = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg       = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg = if is_active {
            nav_hover_bg_raw(glyph, is_dark, pal.row_hover)
        } else {
            nav_hover_bg_raw(glyph, is_dark, if is_dark { rgb(0x1a1a2a) } else { rgb(0xf5f3ff) })
        };
        let provider_icon = arcadia_core::config::modules::MODULE_REGISTRY.iter()
            .find(|m| m.name == module_name.as_str())
            .map(|m| m.glyph)
            .unwrap_or("modules");
        let radius   = nav_radius(glyph);
        div()
            .ml_12()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active { openframe::FontWeight::MEDIUM } else { openframe::FontWeight::NORMAL })
            .bg(bg)
            .text_color(text_col)
            .hover(move |s| s.bg(hover_bg))
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
    ) -> impl IntoElement {
        let pal      = theme::nav_accent_palette(accent, is_dark);
        let text_col = if is_active { nav_active_text(glyph, pal.icon_active) } else { nav_idle_text(glyph, is_dark) };
        let bg       = if is_active { nav_active_bg(glyph, pal.row_selected) } else { nav_idle_bg(glyph, is_dark) };
        let hover_bg = nav_hover_bg_raw(glyph, is_dark, pal.row_hover);
        let radius   = nav_radius(glyph);
        let meta_col = if is_active { pal.icon_active } else if is_dark { rgb(0x4a5568) } else { rgb(0x9ca3af) };
        div()
            .ml_7()
            .pl_2()
            .pr_2()
            .py_1()
            .rounded(px(radius))
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active { openframe::FontWeight::MEDIUM } else { openframe::FontWeight::NORMAL })
            .bg(bg)
            .text_color(text_col)
            .hover(move |s| s.bg(hover_bg))
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
