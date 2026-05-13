#[cfg(any(feature = "gui", feature = "ios-gui"))]
pub mod app;
mod assets;
#[cfg(feature = "gui")]
pub mod cursor_backend;
mod theme;
#[cfg(feature = "gui")]
pub mod tray_backend;
#[cfg(feature = "gui")]
mod overlay_backend;
#[cfg(feature = "gui")]
mod overlay_hud;
#[cfg(feature = "gui")]
mod tui;

#[cfg(feature = "gui")]
pub use app::run;
