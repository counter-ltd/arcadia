use std::time::Duration;

use openframe::{canvas, div, Context, Div, ParentElement, Styled, Timer, Window};

use crate::gui::app::ArcadiaRoot;

use super::draw_arch::splash_draw_arch;
use super::draw_bg::splash_draw_bg;
use super::draw_hills::splash_draw_hills;
use super::draw_horizon::splash_draw_horizon_glow;
use super::draw_stars::splash_draw_stars;
use super::draw_sun::splash_draw_sun;
use super::math::{ease_out_cubic, splash_phase};
use super::SPLASH_TOTAL_MS;

impl ArcadiaRoot {
    pub(crate) fn ensure_splash_tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.splash_tick_started {
            return;
        }
        self.splash_tick_started = true;
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>, cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_millis(16)).await;
                        let done = cx
                            .update(|_, app| {
                                view.update(app, |this, cx| {
                                    this.splash_elapsed_ms += 16.0;
                                    cx.notify();
                                    this.splash_elapsed_ms >= SPLASH_TOTAL_MS
                                })
                                .unwrap_or(true)
                            })
                            .unwrap_or(true);
                        if done {
                            break;
                        }
                    }
                }
            },
        )
        .detach();
    }

    pub(crate) fn render_splash(&self) -> Div {
        let t = self.splash_elapsed_ms;

        let hills_t = ease_out_cubic(splash_phase(t, 200.0, 900.0));
        let sun_t = ease_out_cubic(splash_phase(t, 700.0, 1100.0));
        let arch_t = ease_out_cubic(splash_phase(t, 1300.0, 1200.0));
        let stars_t = splash_phase(t, 2000.0, 800.0);
        let master_alpha = if t < 3600.0 {
            splash_phase(t, 0.0, 400.0)
        } else {
            1.0 - splash_phase(t, 3600.0, 700.0)
        };

        div().size_full().opacity(master_alpha).child(
            canvas(
                |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    splash_draw_bg(bounds, window);
                    splash_draw_horizon_glow(bounds, sun_t, window);
                    splash_draw_arch(bounds, arch_t, window);
                    splash_draw_stars(bounds, stars_t, window);
                    splash_draw_sun(bounds, sun_t, window);
                    splash_draw_hills(bounds, hills_t, window);
                },
            )
            .size_full(),
        )
    }
}
