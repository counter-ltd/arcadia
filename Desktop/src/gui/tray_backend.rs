//! Desktop tray backend wiring `arcadia_core::modules::tray` to the cross-platform
//! `tray-icon` crate.
//!
//! `tray_icon::TrayIcon` is intentionally `!Send` (platform handles live on the main
//! thread), so we keep the platform state in a `thread_local` on the main thread.
//! Backend trait methods are always invoked via `scheduling::run_on_main`, which is
//! drained from the GUI foreground executor — i.e. on the main thread — so the
//! thread_local is always the right one.
//!
//! The `TrayBackend` impl itself is a zero-sized unit struct so it satisfies the
//! `Send + Sync` bound the core trait requires.

use std::cell::RefCell;
use std::collections::HashMap;

use arcadia_core::modules::tray::{self, TrayBackend, TrayItem, TrayMenuItem};

use display_info::DisplayInfo;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

struct ItemState {
    tray: TrayIcon,
    menu_map: HashMap<MenuId, TrayMenuItem>,
}

thread_local! {
    static STATE: RefCell<HashMap<String, ItemState>> = RefCell::new(HashMap::new());
}

fn build_menu(source: &[TrayMenuItem]) -> (Menu, HashMap<MenuId, TrayMenuItem>) {
    let menu = Menu::new();
    let mut map: HashMap<MenuId, TrayMenuItem> = HashMap::new();
    for spec in source {
        if spec.command_token.is_empty() && spec.label.is_empty() {
            let _ = menu.append(&PredefinedMenuItem::separator());
            continue;
        }
        let mi = MenuItem::new(spec.label.as_str(), true, None);
        map.insert(mi.id().clone(), spec.clone());
        let _ = menu.append(&mi);
    }
    (menu, map)
}

fn apply_on_main(item: &TrayItem) {
    let (menu, menu_map) = build_menu(&item.menu);

    let icon = if let Some(image) = item.image.as_ref() {
        Icon::from_rgba(image.rgba.clone(), image.width, image.height).ok()
    } else {
        // 16x16 fully transparent placeholder so the tray slot is reserved immediately.
        Icon::from_rgba(vec![0u8; 16 * 16 * 4], 16, 16).ok()
    };

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(existing) = state.get_mut(&item.id) {
            if let Some(icon) = icon.clone() {
                let _ = existing.tray.set_icon(Some(icon));
            }
            let _ = existing.tray.set_tooltip(Some(item.tooltip.as_str()));
            let _ = existing.tray.set_menu(Some(Box::new(menu)));
            existing.menu_map = menu_map;
            return;
        }

        let mut builder = TrayIconBuilder::new()
            .with_id(item.id.clone())
            .with_menu(Box::new(menu))
            .with_tooltip(item.tooltip.as_str());
        if let Some(icon) = icon {
            builder = builder.with_icon(icon);
        }
        match builder.build() {
            Ok(tray) => {
                state.insert(item.id.clone(), ItemState { tray, menu_map });
            }
            Err(e) => eprintln!("tray-icon build failed for {}: {e}", item.id),
        }
    });
}

fn remove_on_main(id: &str) {
    STATE.with(|s| {
        s.borrow_mut().remove(id);
    });
}

/// Zero-sized backend handle. All state lives in the main-thread `STATE` thread_local.
#[derive(Default)]
pub struct DesktopTrayBackend;

impl TrayBackend for DesktopTrayBackend {
    fn apply(&self, item: &TrayItem) {
        apply_on_main(item);
    }

    fn remove(&self, id: &str) {
        remove_on_main(id);
    }

    fn icon_screen_bounds(&self, id: &str) -> Option<(f64, f64, f64, f64, f64)> {
        let infos = DisplayInfo::all().ok()?;
        let primary = infos
            .iter()
            .find(|d| d.is_primary)
            .or_else(|| infos.first())?;
        let pixels_per_point = f64::from(primary.scale_factor);
        let mut out = None;
        STATE.with(|s| {
            if let Some(st) = s.borrow().get(id) {
                if let Some(r) = st.tray.rect() {
                    out = Some((
                        r.position.x,
                        r.position.y,
                        r.size.width as f64,
                        r.size.height as f64,
                        pixels_per_point,
                    ));
                }
            }
        });
        out
    }
}

/// Install the desktop tray backend with the core `tray` module. Idempotent.
pub fn install() {
    tray::set_backend(Box::new(DesktopTrayBackend));
}

/// Poll `tray-icon` `MenuEvent` channel and forward clicks into the Arcadia command
/// dispatcher. Must be called on the main thread (the same thread that drains
/// `scheduling::run_on_main`); call once per GUI tick from the foreground executor.
pub fn poll_menu_events() {
    let rx = MenuEvent::receiver();
    while let Ok(evt) = rx.try_recv() {
        let lookup = STATE.with(|s| {
            s.borrow()
                .values()
                .find_map(|state| state.menu_map.get(&evt.id).cloned())
        });
        if let Some(spec) = lookup {
            tray::dispatch_menu_command(&spec.command_token, &spec.args);
        }
    }
}
