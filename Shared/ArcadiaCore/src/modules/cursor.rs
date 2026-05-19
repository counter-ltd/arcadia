//! Global cursor position module — cursor input only.
//!
//! Core defines the API + a backend trait; the desktop surface installs a platform-specific
//! implementation at startup. Headless / iOS without a backend → query returns `None`.
//!
//! Display geometry (screen size, scale factor) and platform metadata (menu bar height) live
//! in [`crate::modules::platform`] so this module stays cursor-input-only.

use std::sync::{Mutex, OnceLock};

use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "cursor";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}

/// Global cursor position plus primary mouse buttons when the active cursor backend supports it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorSnapshot {
    pub x: f64,
    pub y: f64,
    pub left_button: bool,
    pub right_button: bool,
}

pub trait CursorBackend: Send + Sync {
    fn position(&self) -> Option<CursorPosition>;
    /// OS-global mouse sample including left/right button down state, or `None` if unavailable.
    fn snapshot(&self) -> Option<CursorSnapshot> {
        None
    }
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

pub fn snapshot() -> Option<CursorSnapshot> {
    backend_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().and_then(|b| b.snapshot()))
}

fn cmd_position(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match position() {
        Some(p) => format!("{:.2} {:.2}", p.x, p.y),
        None => "Cursor backend not available on this surface.".to_string(),
    }
}

fn cmd_screen_size(_args: &[&str], _ctx: &ExecutionContext) -> String {
    match crate::modules::platform::primary_screen_size() {
        Some(s) => format!("{} {}", s.width, s.height),
        None => "Platform backend not available on this surface.".to_string(),
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

#[derive(Default)]
pub struct CursorExtension;

impl crate::extension::Extension for CursorExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "cursor".to_string(),
            version: "0.1.0".to_string(),
            description:
                "OS-global cursor position and primary display size for extensions that track input."
                    .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: vec!["cursor.global_position".to_string()],
            workspace_permissions: Vec::new(),
            supported_platforms: vec![
                crate::platform::PLATFORM_MACOS.to_string(),
                crate::platform::PLATFORM_WINDOWS.to_string(),
                crate::platform::PLATFORM_LINUX.to_string(),
            ],
            api_exports: Vec::new(),
        }
    }

    fn commands(&self) -> Vec<crate::extension::OwnedModuleCommand> {
        commands()
            .iter()
            .map(crate::extension::OwnedModuleCommand::from_static)
            .collect()
    }

    fn permissions(&self) -> &'static [crate::config::permissions::PermissionDefinition] {
        use crate::config::permissions::{PermissionDefinition, SystemGrant};
        static PERMS: &[PermissionDefinition] = &[
            PermissionDefinition {
                id: "cursor.global_position",
                title: "Global cursor position",
                description: "Read the OS-global mouse cursor position even when Arcadia is not focused (macOS: Accessibility; same device-query gate as global mouse buttons).",
                default_global: false,
                system_grant: Some(SystemGrant::Accessibility),
            },
            PermissionDefinition {
                id: "cursor.global_mouse_buttons",
                title: "Global mouse buttons",
                description: "Read whether the primary mouse buttons are pressed anywhere on the OS, even when Arcadia is not focused (macOS: Accessibility; same gate as global cursor position).",
                default_global: false,
                system_grant: Some(SystemGrant::Accessibility),
            },
        ];
        PERMS
    }
}

crate::register_extension!(CursorExtension);
