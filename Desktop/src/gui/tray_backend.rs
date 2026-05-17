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

use arcadia_core::modules::tray::{TrayBackend, TrayItem, TrayMenuItem};
use arcadia_core::modules::{python_registry, tray};

use display_info::DisplayInfo;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

struct ItemState {
    tray: TrayIcon,
    menu_map: HashMap<MenuId, TrayMenuItem>,
    /// Last menu pushed to the platform tray. When unchanged, skip `set_menu` so rapid
    /// `set_icon` updates (e.g. animated tray icons) do not dismiss an open context menu.
    applied_menu: Vec<TrayMenuItem>,
}

thread_local! {
    static STATE: RefCell<HashMap<String, ItemState>> = RefCell::new(HashMap::new());
}

fn build_menu(source: &[TrayMenuItem]) -> (Menu, HashMap<MenuId, TrayMenuItem>) {
    let menu = Menu::new();
    let mut map: HashMap<MenuId, TrayMenuItem> = HashMap::new();
    for spec in source {
        if spec.command_token.is_empty() {
            if spec.label.is_empty() {
                let _ = menu.append(&PredefinedMenuItem::separator());
            } else {
                let _ = menu.append(&MenuItem::new(spec.label.as_str(), false, None));
            }
            continue;
        }
        let mi = MenuItem::new(spec.label.as_str(), true, None);
        map.insert(mi.id().clone(), spec.clone());
        let _ = menu.append(&mi);
    }
    (menu, map)
}

fn apply_on_main(item: &TrayItem) {
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
            if existing.applied_menu != item.menu {
                let (menu, menu_map) = build_menu(&item.menu);
                let _ = existing.tray.set_menu(Some(Box::new(menu)));
                existing.menu_map = menu_map;
                existing.applied_menu = item.menu.clone();
            }
            return;
        }

        let (menu, menu_map) = build_menu(&item.menu);
        let mut builder = TrayIconBuilder::new()
            .with_id(item.id.clone())
            .with_menu(Box::new(menu))
            .with_tooltip(item.tooltip.as_str());
        if let Some(icon) = icon {
            builder = builder.with_icon(icon);
        }
        match builder.build() {
            Ok(tray) => {
                state.insert(
                    item.id.clone(),
                    ItemState {
                        tray,
                        menu_map,
                        applied_menu: item.menu.clone(),
                    },
                );
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

    fn set_show_menu_on_left_click(&self, tray_item_id: &str, enable: bool) {
        STATE.with(|s| {
            if let Some(st) = s.borrow_mut().get_mut(tray_item_id) {
                st.tray.set_show_menu_on_left_click(enable);
            }
        });
    }
}

/// Install the desktop tray backend with the core `tray` module. Idempotent.
pub fn install() {
    tray::set_backend(Box::new(DesktopTrayBackend));
}

/// Poll `tray-icon` menu + tray-icon click channels. Must be called on the main thread
/// (the same thread that drains `scheduling::run_on_main`); call once per GUI tick from
/// the foreground executor.
pub fn poll_menu_events() {
    let menu_rx = MenuEvent::receiver();
    while let Ok(evt) = menu_rx.try_recv() {
        let lookup = STATE.with(|s| {
            s.borrow()
                .values()
                .find_map(|state| state.menu_map.get(&evt.id).cloned())
        });
        if let Some(spec) = lookup {
            tray::dispatch_menu_command(&spec.command_token, &spec.args);
        }
    }

    let tray_rx = TrayIconEvent::receiver();
    while let Ok(evt) = tray_rx.try_recv() {
        if let TrayIconEvent::Click {
            id,
            button,
            button_state,
            ..
        } = evt
        {
            if button_state != MouseButtonState::Up {
                continue;
            }
            let tray_id = id.as_ref().to_string();
            let button_s = match button {
                MouseButton::Left => "left",
                MouseButton::Right => "right",
                MouseButton::Middle => "middle",
            };
            python_registry::dispatch_tray_icon_click(tray_id, button_s.to_string());
        }
    }
}
