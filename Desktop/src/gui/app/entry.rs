use openframe::{
    AppContext, Application, TitlebarOptions, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions, WindowStacking,
};

static QUIT_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

use super::super::assets::{register_bundled_fonts, EmbeddedAssets};
use super::super::overlay_hud::OverlayHudRoot;
use super::ArcadiaRoot;

use crate::cli;
use crate::gui::{cursor_backend, overlay_backend, platform_backend, tray_backend};
use arcadia_core::modules::overlay::OverlayStackingToken;
use arcadia_core::scheduling;

pub fn run() {
    use std::sync::atomic::Ordering;
    use std::thread;

    cli::print_startup("gui");

    thread::spawn(|| {
        cli::start_loop(|| {
            QUIT_REQUESTED.store(true, Ordering::Release);
        });
    });

    // Mark that we will drain the main-thread queue; subsystems may now route work via
    // `scheduling::run_on_main` instead of running inline.
    scheduling::register_main_thread_drainer();

    Application::new().with_assets(EmbeddedAssets).run(|app| {
        // Register the bundled monospace font before any window renders so the
        // code editor and TUI get uniform glyph advances instead of an OS fallback.
        register_bundled_fonts(&app.text_system());

        // Install desktop backends BEFORE the window opens so items registered by Python
        // extensions during ArcadiaRoot::new() flush onto the platform tray on first frame.
        // tray-icon requires main-thread creation on macOS — the run callback runs there.
        overlay_backend::init_overlay_module();
        tray_backend::install();
        cursor_backend::install();
        platform_backend::install();

        app.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_, app| {
                let overlay_opts_hud = hud_overlay_window_options(app, WindowStacking::Hud);
                if let Ok(handle) = app.open_window(overlay_opts_hud, |_ow, app2| {
                    app2.new(|cx| OverlayHudRoot::new(cx, OverlayStackingToken::Hud))
                }) {
                    overlay_backend::register_overlay_window(handle, OverlayStackingToken::Hud);
                } else {
                    eprintln!("arcadia: failed to open HUD overlay window");
                }

                let overlay_opts_bmb = hud_overlay_window_options(app, WindowStacking::BelowMenuBar);
                if let Ok(handle) = app.open_window(overlay_opts_bmb, |_ow, app2| {
                    app2.new(|cx| OverlayHudRoot::new(cx, OverlayStackingToken::BelowMenuBar))
                }) {
                    overlay_backend::register_overlay_window(handle, OverlayStackingToken::BelowMenuBar);
                } else {
                    eprintln!("arcadia: failed to open below-menu-bar overlay window");
                }

                app.new(|cx| ArcadiaRoot::new(cx))
            },
        )
        .expect("failed to open GPUI window");
        // Foreground the app and key window (e.g. when started from a terminal, the previous
        // app would otherwise stay active despite WindowOptions::focus defaulting to true).
        app.activate(true);

        // Main-thread pump: drains the scheduling queue (tray image/menu updates marshal here)
        // and forwards tray-icon menu clicks + tray surface clicks (Python handlers).
        spawn_main_thread_pump(app);
    });
}

fn spawn_main_thread_pump(app: &mut openframe::App) {
    use openframe::Timer;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    app.spawn(async |cx| loop {
        Timer::after(Duration::from_millis(50)).await;
        if QUIT_REQUESTED.load(Ordering::Acquire) {
            cx.update(|app| app.quit()).ok();
            return;
        }
        scheduling::drain_main_queue();
        tray_backend::poll_menu_events();
        overlay_backend::poll_overlay(cx);
    })
    .detach();
}

fn hud_overlay_window_options(app: &openframe::App, stacking: WindowStacking) -> WindowOptions {
    use openframe::{point, px, size};

    let bounds = app
        .primary_display()
        .map(|d| WindowBounds::Windowed(d.bounds()))
        .unwrap_or_else(|| {
            WindowBounds::Windowed(openframe::Bounds::new(
                point(px(0.), px(0.)),
                size(px(800.), px(600.)),
            ))
        });
    WindowOptions {
        window_bounds: Some(bounds),
        titlebar: None,
        focus: false,
        show: false,
        kind: WindowKind::PopUp,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        mouse_passthrough: true,
        stacking,
        ..Default::default()
    }
}
