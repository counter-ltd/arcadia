use openframe::{AppContext, Application, TitlebarOptions, WindowOptions};

use super::super::assets::EmbeddedAssets;
use super::ArcadiaRoot;

use crate::cli;

pub fn run() {
    use std::process;
    use std::thread;

    cli::print_startup("gui");

    thread::spawn(|| {
        cli::start_loop(|| process::exit(0));
    });

    Application::new().with_assets(EmbeddedAssets).run(|app| {
        app.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, app| app.new(|cx| ArcadiaRoot::new(cx)),
        )
        .expect("failed to open GPUI window");
    });
}
