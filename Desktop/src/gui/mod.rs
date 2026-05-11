#[cfg(any(feature = "gui", feature = "ios-gui"))]
pub mod app;
mod assets;
mod theme;
#[cfg(feature = "gui")]
mod tui;

#[cfg(feature = "gui")]
pub use app::run;
