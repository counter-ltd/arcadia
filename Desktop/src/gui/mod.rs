#[cfg(any(feature = "gui", feature = "ios-gui"))]
pub mod app;
mod assets;
#[cfg(feature = "gui")]
pub mod cursor_backend;
#[cfg(feature = "gui")]
mod overlay_backend;
#[cfg(feature = "gui")]
pub mod platform_backend;
#[cfg(feature = "gui")]
mod overlay_hud;
mod theme;
#[cfg(feature = "gui")]
pub mod tray_backend;
#[cfg(feature = "gui")]
mod tui;

#[cfg(feature = "gui")]
pub use app::run;
