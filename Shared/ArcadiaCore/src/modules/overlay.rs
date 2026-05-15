//! Single shared HUD overlay window (desktop GUI only).
//!
//! A desktop backend records visibility requests; the GUI foreground pump applies them on the next
//! tick so we never store `AsyncApp` in a `Send` mutex. Quitting the desktop `App` drops every
//! window, including the overlay. Headless / iOS have no backend.
//!
//! ## Stacking tokens (`overlay.set-stacking`)
//!
//! Canonical spellings (stable forever): `normal`, `floating`, `hud`, `system_ui`.
//! `system_ui` additionally requires permission **`overlay.system_ui`** (and **`overlay.hud`**
//! via the command gate).

use std::sync::{Mutex, OnceLock};

use crate::config::modules::OVERLAY_MODULE_NAME;
use crate::config::permissions::{PermissionSubject, PermissionsConfig};
use crate::config::ConfigFile;
use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "overlay";

/// Parsed `overlay.set-stacking` token (no raw OS levels).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayStackingToken {
    Normal,
    Floating,
    Hud,
    SystemUi,
}

impl OverlayStackingToken {
    pub const fn as_str(self) -> &'static str {
        match self {
            OverlayStackingToken::Normal => "normal",
            OverlayStackingToken::Floating => "floating",
            OverlayStackingToken::Hud => "hud",
            OverlayStackingToken::SystemUi => "system_ui",
        }
    }
}

/// Parse a user-supplied stacking token. Aliases are rejected.
pub fn parse_overlay_stacking_token(s: &str) -> Result<OverlayStackingToken, String> {
    match s.trim() {
        "normal" => Ok(OverlayStackingToken::Normal),
        "floating" => Ok(OverlayStackingToken::Floating),
        "hud" => Ok(OverlayStackingToken::Hud),
        "system_ui" => Ok(OverlayStackingToken::SystemUi),
        other => Err(format!(
            "Unknown overlay stacking token '{other}' (expected normal|floating|hud|system_ui)"
        )),
    }
}

fn ensure_overlay_system_ui_permission() -> Result<(), String> {
    let cfg = PermissionsConfig::load_or_create().map_err(|e| e.to_string())?;
    let subject = PermissionSubject::module(OVERLAY_MODULE_NAME.to_string());
    if cfg.effective_allowed(&subject, "overlay.system_ui") {
        Ok(())
    } else {
        Err(
            "Permission denied: overlay.system_ui (subject module:overlay, global or per-module grant missing)"
                .to_string(),
        )
    }
}

pub trait OverlayBackend: Send + Sync {
    fn set_visible(&self, visible: bool);
    fn is_visible(&self) -> bool;
    fn set_stacking(&self, token: OverlayStackingToken) -> Result<(), String>;
}

fn backend_slot() -> &'static Mutex<Option<Box<dyn OverlayBackend>>> {
    static B: OnceLock<Mutex<Option<Box<dyn OverlayBackend>>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(None))
}

/// Last stacking label echoed by `overlay.status` (desktop updates on apply).
fn stacking_status_label() -> &'static Mutex<String> {
    static L: OnceLock<Mutex<String>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(String::from("auto")))
}

/// Install the platform backend (desktop GUI startup).
pub fn set_backend(b: Box<dyn OverlayBackend>) {
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = Some(b);
    }
}

#[cfg(test)]
pub fn clear_backend_for_test() {
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = None;
    }
}

/// Desktop calls after the overlay window is registered so `overlay.status` shows the boot tier.
pub fn set_stacking_status_label(label: impl Into<String>) {
    if let Ok(mut g) = stacking_status_label().lock() {
        *g = label.into();
    }
}

fn cmd_status(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let Ok(guard) = backend_slot().lock() else {
        return "overlay: backend lock poisoned".to_string();
    };
    let Some(backend) = guard.as_ref() else {
        return "overlay: backend not available on this surface".to_string();
    };
    let vis = if backend.is_visible() {
        "visible"
    } else {
        "hidden"
    };
    let stack = stacking_status_label()
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| "<label lock poisoned>".to_string());
    format!("overlay: {vis} stacking={stack}")
}

fn cmd_show(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let Ok(guard) = backend_slot().lock() else {
        return "overlay: backend lock poisoned".to_string();
    };
    let Some(backend) = guard.as_ref() else {
        return "overlay: backend not available on this surface".to_string();
    };
    backend.set_visible(true);
    "overlay: visible".to_string()
}

fn cmd_hide(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let Ok(guard) = backend_slot().lock() else {
        return "overlay: backend lock poisoned".to_string();
    };
    let Some(backend) = guard.as_ref() else {
        return "overlay: backend not available on this surface".to_string();
    };
    backend.set_visible(false);
    "overlay: hidden".to_string()
}

fn cmd_set_stacking(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(raw) = args.first().copied() else {
        return "Usage: overlay.set-stacking <normal|floating|hud|system_ui>".to_string();
    };
    let token = match parse_overlay_stacking_token(raw) {
        Ok(t) => t,
        Err(e) => return e,
    };
    if matches!(token, OverlayStackingToken::SystemUi) {
        if let Err(e) = ensure_overlay_system_ui_permission() {
            return e;
        }
    }
    let Ok(guard) = backend_slot().lock() else {
        return "overlay: backend lock poisoned".to_string();
    };
    let Some(backend) = guard.as_ref() else {
        return "overlay: backend not available on this surface".to_string();
    };
    match backend.set_stacking(token) {
        Ok(()) => format!("overlay: stacking={}", token.as_str()),
        Err(e) => e,
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "status",
            description: "Print overlay window visibility and stacking (overlay.status).",
            required_permissions: &["overlay.hud"],
            run: cmd_status,
        },
        ModuleCommand {
            name: "show",
            description: "Map the HUD overlay window (overlay.show).",
            required_permissions: &["overlay.hud"],
            run: cmd_show,
        },
        ModuleCommand {
            name: "hide",
            description: "Unmap the HUD overlay window (overlay.hide).",
            required_permissions: &["overlay.hud"],
            run: cmd_hide,
        },
        ModuleCommand {
            name: "set-stacking",
            description:
                "Set overlay stacking tier: normal|floating|hud|system_ui (overlay.set-stacking).",
            required_permissions: &["overlay.hud"],
            run: cmd_set_stacking,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_unknown_token() {
        assert!(parse_overlay_stacking_token("system-ui").is_err());
        assert!(parse_overlay_stacking_token("").is_err());
    }

    #[test]
    fn parse_accepts_canonical_tokens() {
        assert_eq!(
            parse_overlay_stacking_token("normal").unwrap(),
            OverlayStackingToken::Normal
        );
        assert_eq!(
            parse_overlay_stacking_token("  hud").unwrap(),
            OverlayStackingToken::Hud
        );
    }
}
