#![allow(dead_code)]
use openframe::{
    div, px, Context, InteractiveElement, IntoElement, MouseButton, ParentElement,
    StatefulInteractiveElement, Styled, Window,
};

use arcadia_core::modules::late::{experimental_refresh, experimental_state};
use arcadia_core::config::ConfigFile;
use arcadia_core::config::late::LateConfig;

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;

fn section_header(label: &str, meta_c: openframe::Rgba) -> openframe::Div {
    div()
        .px_4()
        .pt_4()
        .pb_1()
        .text_xs()
        .font_weight(openframe::FontWeight::SEMIBOLD)
        .text_color(meta_c)
        .child(label.to_uppercase())
}

fn pill(text: String, bg: openframe::Rgba, tc: openframe::Rgba) -> openframe::Div {
    div()
        .px_2()
        .py_0p5()
        .rounded_md()
        .text_xs()
        .bg(bg)
        .text_color(tc)
        .child(text)
}

fn row_label(text: String, desc_c: openframe::Rgba) -> openframe::Div {
    div()
        .text_sm()
        .text_color(desc_c)
        .child(text)
}

fn divider(line_c: openframe::Rgba) -> openframe::Div {
    div()
        .mx_4()
        .my_1()
        .h_px()
        .bg(line_c)
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

    let exp_border = theme::ui_border(cx, is_dark);
    let exp_surface2 = theme::ui_surface2(cx, is_dark);
    let exp_text = theme::ui_text(cx, is_dark);
    let exp_subtext = theme::ui_subtext(cx, is_dark);
    let exp_radius = theme::ui_radius(cx);
    let exp_section_meta = theme::content_muted_text(cx, is_dark);
    let exp_row_desc = theme::content_subdued_text(cx, is_dark);
    let exp_pill_bg = theme::action_pill_bg(cx, is_dark);
    let exp_pill_tc = theme::action_pill_text(cx, is_dark);
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
                .border_color(exp_border)
                .child(
                    div()
                        .text_base()
                        .font_weight(openframe::FontWeight::SEMIBOLD)
                        .text_color(exp_text)
                        .child("Experimental"),
                )
                .child({
                    let mut d = div().flex_1();
                    if let Some(e) = last_error {
                        d = d.child(
                            div()
                                .text_xs()
                                .text_color(theme::ui_danger(cx, is_dark))
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
                        .rounded(px(exp_radius))
                        .bg(exp_surface2)
                        .text_sm()
                        .text_color(exp_subtext)
                        .hover(move |s| s.bg(exp_surface2))
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
                .child(section_header("Profile", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(row_label(
                            format!("@{profile_username}"),
                            exp_row_desc,
                        ))
                        .child(row_label(
                            if profile_bio.is_empty() {
                                "No bio".to_string()
                            } else {
                                truncate(&profile_bio, 80)
                            },
                            exp_row_desc,
                        ))
                        .child(row_label(
                            format!("notify: {profile_notify}"),
                            exp_row_desc,
                        )),
                )
                .child(divider(exp_border))
                // ── Notifications ──────────────────────────────────────────
                .child(section_header("Notifications", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(pill(
                            format!("{unread_notif} unread"),
                            exp_pill_bg,
                            exp_pill_tc,
                        ))
                        .children(
                            notif_preview
                                .into_iter()
                                .map(|n| row_label(n, exp_row_desc)),
                        ),
                )
                .child(divider(exp_border))
                // ── Articles ───────────────────────────────────────────────
                .child(section_header("Articles", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(
                            articles
                                .into_iter()
                                .map(|(title, _url)| row_label(title, exp_row_desc)),
                        ),
                )
                .child(divider(exp_border))
                // ── Work Profiles ──────────────────────────────────────────
                .child(section_header("Work Profiles", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(if work_profiles.is_empty() {
                            vec![row_label("No profiles yet".to_string(), exp_row_desc)]
                        } else {
                            work_profiles
                                .into_iter()
                                .map(|p| row_label(p, exp_row_desc))
                                .collect()
                        }),
                )
                .child(divider(exp_border))
                // ── RSS ────────────────────────────────────────────────────
                .child(section_header("RSS", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(pill(
                            format!("{rss_unread} unread entries"),
                            exp_pill_bg,
                            exp_pill_tc,
                        ))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(openframe::FontWeight::SEMIBOLD)
                                .text_color(exp_section_meta)
                                .pt_1()
                                .child("Feeds"),
                        )
                        .children(rss_feeds.into_iter().map(|f| row_label(f, exp_row_desc)))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(openframe::FontWeight::SEMIBOLD)
                                .text_color(exp_section_meta)
                                .pt_1()
                                .child("Latest Entries"),
                        )
                        .children(rss_entries.into_iter().map(|e| row_label(e, exp_row_desc))),
                )
                .child(divider(exp_border))
                // ── Showcase ───────────────────────────────────────────────
                .child(section_header("Showcase", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(if showcase.is_empty() {
                            vec![row_label("No items yet".to_string(), exp_row_desc)]
                        } else {
                            showcase
                                .into_iter()
                                .map(|s| row_label(s, exp_row_desc))
                                .collect()
                        }),
                )
                .child(divider(exp_border))
                // ── Leaderboard ────────────────────────────────────────────
                .child(section_header("Game Leaderboard", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(if leaderboard.is_empty() {
                            vec![row_label("No scores yet".to_string(), exp_row_desc)]
                        } else {
                            leaderboard
                                .into_iter()
                                .map(|e| {
                                    div()
                                        .text_xs()
                                        .font_family("monospace")
                                        .text_color(exp_row_desc)
                                        .child(e)
                                })
                                .collect()
                        }),
                )
                .child(divider(exp_border))
                // ── Artboard ───────────────────────────────────────────────
                .child(section_header("Artboard", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        .child(pill(artboard_label, exp_pill_bg, exp_pill_tc)),
                )
                .child(divider(exp_border))
                // ── Chips ──────────────────────────────────────────────────
                .child(section_header("Chips", exp_section_meta))
                .child(
                    div()
                        .px_4()
                        .pb_4()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap_1()
                        .children(if chips.is_empty() {
                            vec![row_label("No chips".to_string(), exp_row_desc)]
                        } else {
                            chips
                                .into_iter()
                                .map(|c| pill(c, exp_pill_bg, exp_pill_tc))
                                .collect()
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
    ) -> openframe::Div {
        div()
            .flex_1()
            .h_full()
            .min_h_0()
            .child(late_experimental_panel(self, cx, is_dark))
    }
}
