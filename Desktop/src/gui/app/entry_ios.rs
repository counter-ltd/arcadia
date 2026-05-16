use openframe::{AppContext, Application, WindowOptions};

use super::super::assets::{register_bundled_fonts, EmbeddedAssets};
use super::ArcadiaRoot;

pub fn run(metal_layer_ptr: usize) {
    Application::new()
        .with_assets(EmbeddedAssets)
        .run(move |app| {
            register_bundled_fonts(&app.text_system());

            app.open_window(
                WindowOptions {
                    titlebar: None,
                    ..Default::default()
                },
                |_win, app| app.new(|cx| ArcadiaRoot::new(cx)),
            )
            .expect("failed to open iOS window");

            #[cfg(target_os = "ios")]
            openframe::ios_set_metal_layer_ptr(metal_layer_ptr);
            #[cfg(not(target_os = "ios"))]
            let _ = metal_layer_ptr;
        });
}

pub fn inject_touch(x: f32, y: f32, phase: u8) {
    #[cfg(target_os = "ios")]
    openframe::ios_inject_touch(x, y, phase);
    #[cfg(not(target_os = "ios"))]
    let _ = (x, y, phase);
}
