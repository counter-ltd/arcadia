use gpui::{
    div, rgb, Context, InteractiveElement, IntoElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled, Window,
};

use arcadia_core::modules::late::{experimental_refresh, experimental_state};
use arcadia_core::config::ConfigFile;
use arcadia_core::config::late::LateConfig;

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

fn section_header(label: &str, is_dark: bool) -> gpui::Div {
    div()
        .px_4()
        .pt_4()
        .pb_1()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme::module_meta_text(is_dark))
        .child(label.to_uppercase())
}

fn pill(text: String, is_dark: bool) -> gpui::Div {
    div()
        .px_2()
        .py_0p5()
        .rounded_md()
        .text_xs()
        .bg(theme::top_bar_pill_bg(is_dark))
        .text_color(theme::top_bar_pill_text(is_dark))
        .child(text)
}

fn row_label(text: String, is_dark: bool) -> gpui::Div {
    div()
        .text_sm()
        .text_color(theme::module_description_text(is_dark))
        .child(text)
}

fn divider(is_dark: bool) -> gpui::Div {
    div()
        .mx_4()
        .my_1()
        .h_px()
        .bg(if is_dark { rgb(0x1e293b) } else { rgb(0xe2e8f0) })
}

pub fn late_experimental_panel(
    _root: &ArcadiaRoot,
    cx: &mut Context<ArcadiaRoot>,
    is_dark: bool,
) -> impl IntoElement {
    let arc = experimental_state();
    let st = arc.lock().unwrap_or_else(|e| e.into_inner());

    let loading = st.loading;
    let last_error = st.last_error.clone();

    // Profile
    let profile_username = st.profile.as_ref().map(|p| p.username.clone()).unwrap_or_default();
    let profile_bio = st.profile.as_ref().map(|p| p.bio.clone()).unwrap_or_default();
    let profile_notify = st.profile.as_ref().map(|p| p.notify_format.clone()).unwrap_or_default();

    // Notifications
    let unread_notif = st.unread_notifications;
    let notif_preview: Vec<String> = st
        .notifications
        .iter()
        .take(5)
        .map(|n| {
            let dot = if n.read { "○" } else { "●" };
            format!("{dot} [{}] {}", n.kind, truncate(&n.body, 60))
        })
        .collect();

    // Articles
    let articles: Vec<(String, String)> = st
        .articles
        .iter()
        .take(6)
        .map(|a| (truncate(&a.title, 60), a.url.clone()))
        .collect();

    // Work profiles
    let work_profiles: Vec<String> = st
        .work_profiles
        .iter()
        .take(8)
        .map(|p| format!("@{} — {} ({})", p.slug, truncate(&p.headline, 40), p.status))
        .collect();

    // RSS
    let rss_unread = st.rss_unread;
    let rss_feeds: Vec<String> = st
        .rss_feeds
        .iter()
        .take(6)
        .map(|f| {
            let status = if f.active { "✓" } else { "✗" };
            format!("{status} {}", truncate(&f.title, 50))
        })
        .collect();
    let rss_entries: Vec<String> = st
        .rss_entries
        .iter()
        .take(5)
        .map(|e| format!("[{}] {}", truncate(&e.feed_title, 20), truncate(&e.title, 50)))
        .collect();

    // Showcase
    let showcase: Vec<String> = st
        .showcase
        .iter()
        .take(6)
        .map(|s| format!("{} — {}", truncate(&s.title, 40), s.tags.join(", ")))
        .collect();

    // Leaderboard
    let leaderboard: Vec<String> = st
        .leaderboard
        .iter()
        .take(10)
        .map(|e| format!("{:>8} {} ({})", e.score, e.username, e.game))
        .collect();

    // Chips
    let chips: Vec<String> = st
        .chips
        .iter()
        .take(12)
        .map(|c| format!("{} ×{}", c.label, c.count))
        .collect();

    // Artboard
    let artboard_label = match st.artboard_size {
        Some((w, h)) => format!("{w}×{h} canvas"),
        None => "no data".to_string(),
    };

    drop(st);

    let refresh_label = if loading { "Loading…" } else { "↻ Refresh" };

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        // Top bar
        .child(
            div()
                .px_4()
                .py_3()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .border_b_1()
                .border_color(if is_dark { rgb(0x1e293b) } else { rgb(0xe2e8f0) })
                .child(
                    div()
                        .text_base()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(if is_dark { rgb(0xf1f5f9) } else { rgb(0x0f172a) })
                        .child("Experimental"),
                )
                .child({
                    let mut d = div().flex_1();
                    if let Some(e) = last_error {
                        d = d.child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xf87171))
                                .child(truncate(&e, 80)),
                        );
                    }
                    d
                })
                .child(
                    div()
                        .cursor_pointer()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .bg(if is_dark { rgb(0x1e293b) } else { rgb(0xe2e8f0) })
                        .text_sm()
                        .text_color(if is_dark { rgb(0x94a3b8) } else { rgb(0x475569) })
                        .hover(move |s| s.bg(if is_dark { rgb(0x334155) } else { rgb(0xcbd5e1) }))
                        .child(refresh_label)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|_this, _, _, _cx| {
                                if let Ok(cfg) = LateConfig::load_or_create() {
                                    experimental_refresh(cfg.server_url, cfg.auth_token);
                                }
                            }),
                        ),
                ),
        )
        // Scrollable content
        .child(
            div()
                .flex_1()
                .min_h_0()
                .id("exp-scroll")
                .overflow_y_scroll()
                .flex()
                .flex_col()
                // ── Profile ────────────────────────────────────────────────
                .child(section_header("Profile", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(row_label(
                            format!("@{profile_username}"),
                            is_dark,
                        ))
                        .child(row_label(
                            if profile_bio.is_empty() {
                                "No bio".to_string()
                            } else {
                                truncate(&profile_bio, 80)
                            },
                            is_dark,
                        ))
                        .child(row_label(
                            format!("notify: {profile_notify}"),
                            is_dark,
                        )),
                )
                .child(divider(is_dark))
                // ── Notifications ──────────────────────────────────────────
                .child(section_header("Notifications", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(pill(format!("{unread_notif} unread"), is_dark))
                        .children(notif_preview.into_iter().map(|n| row_label(n, is_dark))),
                )
                .child(divider(is_dark))
                // ── Articles ───────────────────────────────────────────────
                .child(section_header("Articles", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(articles.into_iter().map(|(title, _url)| row_label(title, is_dark))),
                )
                .child(divider(is_dark))
                // ── Work Profiles ──────────────────────────────────────────
                .child(section_header("Work Profiles", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(if work_profiles.is_empty() {
                            vec![row_label("No profiles yet".to_string(), is_dark)]
                        } else {
                            work_profiles.into_iter().map(|p| row_label(p, is_dark)).collect()
                        }),
                )
                .child(divider(is_dark))
                // ── RSS ────────────────────────────────────────────────────
                .child(section_header("RSS", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(pill(format!("{rss_unread} unread entries"), is_dark))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::module_meta_text(is_dark))
                                .pt_1()
                                .child("Feeds"),
                        )
                        .children(rss_feeds.into_iter().map(|f| row_label(f, is_dark)))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::module_meta_text(is_dark))
                                .pt_1()
                                .child("Latest Entries"),
                        )
                        .children(rss_entries.into_iter().map(|e| row_label(e, is_dark))),
                )
                .child(divider(is_dark))
                // ── Showcase ───────────────────────────────────────────────
                .child(section_header("Showcase", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(if showcase.is_empty() {
                            vec![row_label("No items yet".to_string(), is_dark)]
                        } else {
                            showcase.into_iter().map(|s| row_label(s, is_dark)).collect()
                        }),
                )
                .child(divider(is_dark))
                // ── Leaderboard ────────────────────────────────────────────
                .child(section_header("Game Leaderboard", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(if leaderboard.is_empty() {
                            vec![row_label("No scores yet".to_string(), is_dark)]
                        } else {
                            leaderboard
                                .into_iter()
                                .map(|e| {
                                    div()
                                        .text_xs()
                                        .font_family("monospace")
                                        .text_color(theme::module_description_text(is_dark))
                                        .child(e)
                                })
                                .collect()
                        }),
                )
                .child(divider(is_dark))
                // ── Artboard ───────────────────────────────────────────────
                .child(section_header("Artboard", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .child(pill(artboard_label, is_dark)),
                )
                .child(divider(is_dark))
                // ── Chips ──────────────────────────────────────────────────
                .child(section_header("Chips", is_dark))
                .child(
                    div()
                        .px_4()
                        .pb_4()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap_1()
                        .children(if chips.is_empty() {
                            vec![row_label("No chips".to_string(), is_dark)]
                        } else {
                            chips.into_iter().map(|c| pill(c, is_dark)).collect()
                        }),
                ),
        )
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

impl ArcadiaRoot {
    pub(crate) fn render_late_experimental(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> gpui::Div {
        div()
            .flex_1()
            .h_full()
            .min_h_0()
            .child(late_experimental_panel(self, cx, is_dark))
    }
}
