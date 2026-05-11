//! Tray / menu-bar icon module.
//!
//! Core holds the canonical tray-item state (image, tooltip, menu); a desktop-side backend
//! provides the platform implementation (`tray-icon` crate on mac/win/linux). Backend
//! methods are always invoked from the main thread via [`crate::scheduling::run_on_main`].
//!
//! Items are identified by an auto-generated string id. Extensions hold the id and use it
//! to update their tray entry.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use crate::modules::{ExecutionContext, ModuleCommand};
use crate::scheduling;

pub const NAME: &str = "tray";

#[derive(Clone, Debug)]
pub struct TrayMenuItem {
    pub label: String,
    /// Empty token = separator (label ignored).
    pub command_token: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TrayImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct TrayItem {
    pub id: String,
    pub owner: String,
    pub label: String,
    pub tooltip: String,
    pub image: Option<TrayImage>,
    pub menu: Vec<TrayMenuItem>,
}

/// Backend installed by the desktop surface during startup. `apply` is the create-or-update
/// hook; `remove` tears the platform icon down. Calls always happen on the main thread.
pub trait TrayBackend: Send + Sync {
    fn apply(&self, item: &TrayItem);
    fn remove(&self, id: &str);

    /// Global on-screen bounds for `tray_item_id` when the platform can report them.
    ///
    /// Returns `(x, y, width, height, pixels_per_point)` where `x/y/width/height` are **physical
    /// pixels** in global desktop space with origin top-left and Y increasing downward (same as
    /// `tray-icon`'s `TrayIcon::rect` physical convention), and `pixels_per_point` is the primary display backing
    /// scale (e.g. `2.0` on Retina) for mapping Quartz cursor points into that space the same way
    /// tray hit-testing does. Default: `None` (unknown / unsupported).
    fn icon_screen_bounds(&self, tray_item_id: &str) -> Option<(f64, f64, f64, f64, f64)> {
        let _ = tray_item_id;
        None
    }
}

struct State {
    items: Mutex<BTreeMap<String, TrayItem>>,
    next_id: AtomicU64,
}

fn state() -> &'static State {
    static S: OnceLock<State> = OnceLock::new();
    S.get_or_init(|| State {
        items: Mutex::new(BTreeMap::new()),
        next_id: AtomicU64::new(1),
    })
}

fn backend_slot() -> &'static Mutex<Option<Box<dyn TrayBackend>>> {
    static B: OnceLock<Mutex<Option<Box<dyn TrayBackend>>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(None))
}

/// Install the platform backend (called from Desktop GUI startup). Replays any items
/// already registered so the new backend syncs to current state.
pub fn set_backend(b: Box<dyn TrayBackend>) {
    let replay: Vec<TrayItem> = state()
        .items
        .lock()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();
    if let Ok(mut slot) = backend_slot().lock() {
        *slot = Some(b);
    }
    for item in replay {
        notify_apply(item);
    }
}

fn notify_apply(item: TrayItem) {
    scheduling::run_on_main(move || {
        if let Ok(slot) = backend_slot().lock() {
            if let Some(backend) = slot.as_ref() {
                backend.apply(&item);
            }
        }
    });
}

fn notify_remove(id: String) {
    scheduling::run_on_main(move || {
        if let Ok(slot) = backend_slot().lock() {
            if let Some(backend) = slot.as_ref() {
                backend.remove(&id);
            }
        }
    });
}

/// Register a new tray item and return its generated id.
///
/// If an item with the same `owner` already exists (e.g. extension body re-run), returns that
/// id so callers do not accumulate duplicate menu-bar slots for one logical owner.
pub fn register_item(owner: impl Into<String>, label: impl Into<String>) -> String {
    let owner = owner.into();
    let label = label.into();

    let existing_id = if let Ok(items) = state().items.lock() {
        items
            .iter()
            .find(|(_, it)| it.owner == owner)
            .map(|(id, _)| id.clone())
    } else {
        None
    };

    if let Some(id) = existing_id {
        let mut updated: Option<TrayItem> = None;
        if let Ok(mut items) = state().items.lock() {
            if let Some(it) = items.get_mut(&id) {
                if it.label != label {
                    it.label = label.clone();
                    updated = Some(it.clone());
                }
            }
        }
        if let Some(item) = updated {
            notify_apply(item);
        }
        return id;
    }

    let seq = state().next_id.fetch_add(1, Ordering::Relaxed);
    let id = format!("tray-{seq}");
    let item = TrayItem {
        id: id.clone(),
        owner,
        label,
        tooltip: String::new(),
        image: None,
        menu: Vec::new(),
    };
    if let Ok(mut items) = state().items.lock() {
        items.insert(id.clone(), item.clone());
    }
    notify_apply(item);
    id
}

fn mutate<F: FnOnce(&mut TrayItem)>(id: &str, f: F) -> Result<TrayItem, String> {
    let mut items = state()
        .items
        .lock()
        .map_err(|_| "tray state poisoned".to_string())?;
    let item = items
        .get_mut(id)
        .ok_or_else(|| format!("unknown tray item: {id}"))?;
    f(item);
    Ok(item.clone())
}

pub fn set_image(id: &str, rgba: Vec<u8>, width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("tray image width/height must be > 0".into());
    }
    let expected = (width as usize) * (height as usize) * 4;
    if rgba.len() != expected {
        return Err(format!(
            "tray image bytes length {} does not match {}x{} RGBA ({} bytes expected)",
            rgba.len(),
            width,
            height,
            expected
        ));
    }
    let updated = mutate(id, |it| {
        it.image = Some(TrayImage {
            rgba,
            width,
            height,
        });
    })?;
    notify_apply(updated);
    Ok(())
}

pub fn set_tooltip(id: &str, tooltip: impl Into<String>) -> Result<(), String> {
    let tooltip = tooltip.into();
    let updated = mutate(id, |it| it.tooltip = tooltip)?;
    notify_apply(updated);
    Ok(())
}

pub fn set_menu(id: &str, menu: Vec<TrayMenuItem>) -> Result<(), String> {
    let updated = mutate(id, |it| it.menu = menu)?;
    notify_apply(updated);
    Ok(())
}

pub fn remove_item(id: &str) -> Result<(), String> {
    {
        let mut items = state()
            .items
            .lock()
            .map_err(|_| "tray state poisoned".to_string())?;
        if items.remove(id).is_none() {
            return Err(format!("unknown tray item: {id}"));
        }
    }
    notify_remove(id.to_string());
    Ok(())
}

pub fn list_items() -> Vec<TrayItem> {
    state()
        .items
        .lock()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default()
}

/// Query the tray icon's screen bounds + primary display `pixels_per_point`.
///
/// Must run through the UI main queue (see [`scheduling::main_queue_active`]); returns `None`
/// when there is no drainer, no backend, or the platform cannot report bounds.
pub fn icon_screen_bounds(tray_item_id: &str) -> Option<(f64, f64, f64, f64, f64)> {
    if !scheduling::main_queue_active() {
        return None;
    }
    let (tx, rx) = mpsc::channel();
    let id = tray_item_id.to_string();
    scheduling::run_on_main(move || {
        let out = backend_slot()
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().and_then(|b| b.icon_screen_bounds(&id)));
        let _ = tx.send(out);
    });
    rx.recv_timeout(Duration::from_millis(100)).ok().flatten()
}

/// Remove every tray item registered under `owner` (e.g. `python:googly-eyes`). Called when a
/// Python extension is disabled so its menu-bar icons disappear immediately instead of lingering
/// until the next app launch.
pub fn remove_items_for_owner(owner: &str) -> usize {
    let removed: Vec<String> = {
        let mut items = match state().items.lock() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        let ids: Vec<String> = items
            .iter()
            .filter_map(|(id, it)| (it.owner == owner).then(|| id.clone()))
            .collect();
        for id in &ids {
            items.remove(id);
        }
        ids
    };
    for id in &removed {
        notify_remove(id.clone());
    }
    removed.len()
}

/// Backend invokes this when a menu item is clicked. Core dispatches the command token.
/// Errors are surfaced through `eprintln!` for now — POC; later we can route to a log channel.
pub fn dispatch_menu_command(token: &str, args: &[String]) {
    if token.is_empty() {
        return;
    }
    let token = token.to_string();
    let args = args.to_vec();
    scheduling::spawn(move || {
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        match crate::modules::execute_command(&token, &arg_refs, &ExecutionContext::default()) {
            Ok(Some(_)) => {}
            Ok(None) => eprintln!("tray menu: unknown command {token}"),
            Err(e) => eprintln!("tray menu: {token}: {e}"),
        }
    });
}

fn cmd_list(_args: &[&str], _ctx: &ExecutionContext) -> String {
    let items = list_items();
    if items.is_empty() {
        return "No tray items registered.".to_string();
    }
    let mut out = String::new();
    for it in items {
        let (w, h) = it
            .image
            .as_ref()
            .map(|im| (im.width, im.height))
            .unwrap_or((0, 0));
        out.push_str(&format!(
            "{} ({}) — label: {:?} tooltip: {:?} image: {}x{} menu: {}\n",
            it.id,
            it.owner,
            it.label,
            it.tooltip,
            w,
            h,
            it.menu.len(),
        ));
    }
    out.trim_end().to_string()
}

fn cmd_remove(args: &[&str], _ctx: &ExecutionContext) -> String {
    let Some(id) = args.first() else {
        return "Usage: tray.remove <item-id>".to_string();
    };
    match remove_item(id) {
        Ok(()) => format!("Removed tray item {id}."),
        Err(e) => e,
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[
        ModuleCommand {
            name: "list",
            description: "List active tray items.",
            required_permissions: &[],
            run: cmd_list,
        },
        ModuleCommand {
            name: "remove",
            description: "Remove a tray item by id: tray.remove <item-id>",
            required_permissions: &["tray.create"],
            run: cmd_remove,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_then_remove() {
        let id = register_item("tray-test-register-then-remove", "hello");
        assert!(list_items().iter().any(|it| it.id == id));
        set_tooltip(&id, "tip").unwrap();
        let item = list_items().into_iter().find(|i| i.id == id).unwrap();
        assert_eq!(item.tooltip, "tip");
        remove_item(&id).unwrap();
        assert!(list_items().iter().all(|it| it.id != id));
    }

    #[test]
    fn register_item_reuses_owner() {
        let a = register_item("python:test-owner", "a");
        let b = register_item("python:test-owner", "b");
        assert_eq!(a, b);
        remove_item(&a).unwrap();
    }

    #[test]
    fn set_image_rejects_size_mismatch() {
        let id = register_item("tray-test-set-image-mismatch", "img");
        let err = set_image(&id, vec![0; 9], 2, 2).unwrap_err();
        assert!(err.contains("RGBA"), "{err}");
        remove_item(&id).unwrap();
    }

    #[test]
    fn set_image_accepts_correct_size() {
        let id = register_item("tray-test-set-image-ok", "img2");
        let pixels = vec![255u8; 2 * 2 * 4];
        set_image(&id, pixels, 2, 2).unwrap();
        let item = list_items().into_iter().find(|i| i.id == id).unwrap();
        let img = item.image.unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.rgba.len(), 16);
        remove_item(&id).unwrap();
    }
}
