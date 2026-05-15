//! OS-global hotkey registration (`global-hotkey` crate). Not used on iOS.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use arcadia_core::config::shortcuts::has_system_wide_consent;
use arcadia_core::shortcuts::{self, KeyChordSpec, ShortcutTrigger};
use global_hotkey::hotkey::{Code, HotKey, Modifiers as GhModifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

struct OsHotkeyState {
    manager: Option<GlobalHotKeyManager>,
    /// HotKey platform id → Arcadia shortcut id
    id_map: HashMap<u32, String>,
    registered: Vec<HotKey>,
}

impl Default for OsHotkeyState {
    fn default() -> Self {
        Self {
            manager: None,
            id_map: HashMap::new(),
            registered: Vec::new(),
        }
    }
}

fn state() -> &'static Mutex<OsHotkeyState> {
    static S: OnceLock<Mutex<OsHotkeyState>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(OsHotkeyState::default()))
}

fn openframe_key_to_code(key: &str) -> Option<Code> {
    match key {
        "escape" => Some(Code::Escape),
        "tab" => Some(Code::Tab),
        "space" => Some(Code::Space),
        "enter" => Some(Code::Enter),
        "backspace" => Some(Code::Backspace),
        "delete" => Some(Code::Delete),
        "up" => Some(Code::ArrowUp),
        "down" => Some(Code::ArrowDown),
        "left" => Some(Code::ArrowLeft),
        "right" => Some(Code::ArrowRight),
        "home" => Some(Code::Home),
        "end" => Some(Code::End),
        "pageup" => Some(Code::PageUp),
        "pagedown" => Some(Code::PageDown),
        k if k.len() == 1 => {
            let c = k.chars().next()?;
            if c.is_ascii_alphabetic() {
                let u = c.to_ascii_uppercase();
                match u {
                    'A' => Some(Code::KeyA),
                    'B' => Some(Code::KeyB),
                    'C' => Some(Code::KeyC),
                    'D' => Some(Code::KeyD),
                    'E' => Some(Code::KeyE),
                    'F' => Some(Code::KeyF),
                    'G' => Some(Code::KeyG),
                    'H' => Some(Code::KeyH),
                    'I' => Some(Code::KeyI),
                    'J' => Some(Code::KeyJ),
                    'K' => Some(Code::KeyK),
                    'L' => Some(Code::KeyL),
                    'M' => Some(Code::KeyM),
                    'N' => Some(Code::KeyN),
                    'O' => Some(Code::KeyO),
                    'P' => Some(Code::KeyP),
                    'Q' => Some(Code::KeyQ),
                    'R' => Some(Code::KeyR),
                    'S' => Some(Code::KeyS),
                    'T' => Some(Code::KeyT),
                    'U' => Some(Code::KeyU),
                    'V' => Some(Code::KeyV),
                    'W' => Some(Code::KeyW),
                    'X' => Some(Code::KeyX),
                    'Y' => Some(Code::KeyY),
                    'Z' => Some(Code::KeyZ),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ if key.starts_with('f') => match key {
            "f1" => Some(Code::F1),
            "f2" => Some(Code::F2),
            "f3" => Some(Code::F3),
            "f4" => Some(Code::F4),
            "f5" => Some(Code::F5),
            "f6" => Some(Code::F6),
            "f7" => Some(Code::F7),
            "f8" => Some(Code::F8),
            "f9" => Some(Code::F9),
            "f10" => Some(Code::F10),
            "f11" => Some(Code::F11),
            "f12" => Some(Code::F12),
            _ => None,
        },
        _ => None,
    }
}

fn chord_to_hotkey(chord: &KeyChordSpec) -> Option<HotKey> {
    if chord.function {
        return None;
    }
    let key = openframe_key_to_code(chord.key.as_str())?;
    let mut m = GhModifiers::empty();
    if chord.shift {
        m |= GhModifiers::SHIFT;
    }
    if chord.control {
        m |= GhModifiers::CONTROL;
    }
    if chord.alt {
        m |= GhModifiers::ALT;
    }
    if chord.platform {
        m |= GhModifiers::SUPER;
    }
    if chord.function {
        m |= GhModifiers::FN;
    }
    let hk = HotKey::new(Some(m), key);
    Some(hk)
}

/// Rebuild OS-global registrations from merged shortcuts + consent flags.
pub fn sync_os_global_hotkeys() {
    let Ok(mut st) = state().lock() else {
        return;
    };
    if st.manager.is_none() {
        match GlobalHotKeyManager::new() {
            Ok(m) => st.manager = Some(m),
            Err(e) => {
                eprintln!("global-hotkey: manager init failed: {e}");
                return;
            }
        }
    }
    if let Some(ref mgr) = st.manager {
        for hk in &st.registered {
            let _ = mgr.unregister(*hk);
        }
    }
    st.registered.clear();
    st.id_map.clear();

    let mgr = st.manager.as_ref().unwrap();
    let mut new_registered = Vec::new();
    let mut new_map = HashMap::new();
    for sc in shortcuts::merged_shortcuts() {
        if !sc.system_wide || !has_system_wide_consent(&sc.id) {
            continue;
        }
        let Some(t) = sc.triggers.first() else {
            continue;
        };
        let ShortcutTrigger::Chord(ch) = t else {
            continue;
        };
        let Some(hk) = chord_to_hotkey(ch) else {
            continue;
        };
        if mgr.register(hk).is_ok() {
            new_map.insert(hk.id(), sc.id.clone());
            new_registered.push(hk);
        }
    }
    st.id_map = new_map;
    st.registered = new_registered;
}

pub fn poll_global_hotkey_events(
    this: &mut super::ArcadiaRoot,
    window: &mut openframe::Window,
    cx: &mut openframe::Context<super::ArcadiaRoot>,
) {
    while let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv() {
        if ev.state != HotKeyState::Pressed {
            continue;
        }
        let sid = {
            let Ok(st) = state().lock() else {
                return;
            };
            st.id_map.get(&ev.id).cloned()
        };
        let Some(sid) = sid else {
            continue;
        };
        let list = shortcuts::merged_shortcuts();
        if let Some(sc) = list.into_iter().find(|s| s.id == sid) {
            this.fire_shortcut_actions(&sc, window, cx);
        }
    }
}
