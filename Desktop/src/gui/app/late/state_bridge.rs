use std::time::Duration;

use openframe::{Context, Timer, Window};

use crate::gui::app::ArcadiaRoot;

const LATE_PAGES: &[&str] = &["late.now_playing", "late.experimental"];

impl ArcadiaRoot {
    pub fn ensure_late_poll_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.late_poll_task_started {
            return;
        }
        if !LATE_PAGES.contains(&self.active_page_id.as_str()) {
            return;
        }
        self.late_poll_task_started = true;
        cx.spawn_in(
            window,
            move |view: openframe::WeakEntity<ArcadiaRoot>,
                  cx: &mut openframe::AsyncWindowContext| {
                let mut cx = cx.clone();
                async move {
                    loop {
                        Timer::after(Duration::from_millis(250)).await;
                        let should_stop = cx
                            .update(|_, app| {
                                view.update(app, |this, cx| {
                                    if !LATE_PAGES.contains(&this.active_page_id.as_str()) {
                                        this.late_poll_task_started = false;
                                        return true;
                                    }
                                    let arc = arcadia_core::modules::late::state();
                                    let rev =
                                        arc.lock().unwrap_or_else(|e| e.into_inner()).revision;
                                    if rev != this.late_last_revision {
                                        this.late_last_revision = rev;
                                        cx.notify();
                                    }
                                    false
                                })
                                .unwrap_or(true)
                            })
                            .unwrap_or(true);
                        if should_stop {
                            break;
                        }
                    }
                }
            },
        )
        .detach();
    }
}
