//! Global cursor position module.
//!
//! Core defines the API + a backend trait; the desktop surface installs a platform-specific
//! implementation at startup. Headless / iOS without a backend → query returns `None`.

use std::sync::{Mutex, OnceLock};

use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "cursor";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

pub trait CursorBackend: Send + Sync {
    fn position(&self) -> Option<CursorPosition>;
    fn primary_screen_size(&self) -> Option<ScreenSize>;
}

fn backend_slot() -> &'static Mutex<Option<Box<dyn CursorBackend>>> {
    static B: OnceLock<Mutex<Option<Box<dyn CursorBackend>>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(None))
}

pub fn set_backend(b: Box<dyn CursorBackend>) {
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = Some(b);
    }
}

pub fn position() -> Option<CursorPosition> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().and_then(|b| b.position()))
}

pub fn primary_screen_size() -> Option<ScreenSize> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().and_then(|b| b.primary_screen_size()))
}

fn cmd_position(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match position() {
        Some(p) => format!("{:.2} {:.2}", p.x, p.y),
        None => "Cursor backend not available on this surface.".to_string(),
    }
}

fn cmd_screen_size(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match primary_screen_size() {
        Some(s) => format!("{} {}", s.width, s.height),
        None => "Cursor backend not available on this surface.".to_string(),
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "position",
            description: "Print the OS-global cursor position as: <x> <y>",
            required_permissions: &["cursor.global_position"],
            run: cmd_position,
        },
        ModuleCommand {
            name: "screen-size",
            description: "Print the primary display size as: <width> <height>",
            required_permissions: &["cursor.global_position"],
            run: cmd_screen_size,
        },
    ]
}
