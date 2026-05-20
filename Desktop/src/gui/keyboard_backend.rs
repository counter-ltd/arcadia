//! Desktop keyboard backend: OS-global key / mouse / scroll events via
//! `NSEvent.addGlobalMonitorForEventsMatchingMask:handler:` (Apple's stable API).
//!
//! Why not rdev/CGEventTap: rdev 0.5 has non-exhaustive match patterns over real
//! macOS keycodes (dead keys, vendor function keys), which panic from inside its
//! CGEventTap C callback and SIGTRAP the process. NSEvent global monitor is
//! exception-safe — Apple-maintained, dispatches via the AppKit run-loop, gated
//! by the existing Accessibility (Arcadia `keyboard.global_events` /
//! `SystemGrant::Accessibility`) grant. Unknown event types are simply ignored.
//!
//! Threading: the handler block fires on the main AppKit run-loop. Our forward
//! fn does no blocking work — it converts to a `KeyboardEvent` and calls
//! `keyboard::dispatch_event`, which fans out under a non-blocking lock.
//!
//! We refuse to install the monitor until the user flips the Arcadia
//! `keyboard.global_events` global toggle on — without that we never touch the
//! OS API and macOS never shows its TCC prompt.

use std::sync::Mutex;
use std::time::SystemTime;

use arcadia_core::config::permissions::PermissionsConfig;
use arcadia_core::config::ConfigFile;
use arcadia_core::modules::keyboard::{self, KeyboardBackend, KeyboardEvent};

use block2::RcBlock;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSEvent, NSEventMask, NSEventType};

const KEYBOARD_GLOBAL_EVENTS_PERMISSION: &str = "keyboard.global_events";

fn global_events_enabled() -> bool {
    PermissionsConfig::load_or_create()
        .ok()
        .map(|cfg| cfg.global_allowed(KEYBOARD_GLOBAL_EVENTS_PERMISSION))
        .unwrap_or(false)
}

pub struct DesktopKeyboardBackend {
    /// Two NSEvent monitor tokens: the global monitor fires when Arcadia is NOT
    /// the active app; the local monitor fires when it IS. Without both, typing
    /// in Arcadia itself produces no events.
    global_monitor: Mutex<Option<Retained<AnyObject>>>,
    local_monitor: Mutex<Option<Retained<AnyObject>>>,
}

// NSEvent monitor token is an opaque NSObject. Apple's docs allow it to be sent
// to `+[NSEvent removeMonitor:]` from any thread, but we only ever touch it
// from the install/uninstall path on the main thread.
unsafe impl Send for DesktopKeyboardBackend {}
unsafe impl Sync for DesktopKeyboardBackend {}

impl DesktopKeyboardBackend {
    pub fn new() -> Self {
        Self {
            global_monitor: Mutex::new(None),
            local_monitor: Mutex::new(None),
        }
    }
}

impl KeyboardBackend for DesktopKeyboardBackend {
    fn start(&self) -> Result<(), String> {
        let mut g = self.global_monitor.lock().map_err(|e| e.to_string())?;
        let mut l = self.local_monitor.lock().map_err(|e| e.to_string())?;
        if g.is_some() && l.is_some() {
            return Ok(());
        }
        if !global_events_enabled() {
            return Err(
                "Permission denied: keyboard.global_events (toggle on in Permissions settings)"
                    .into(),
            );
        }
        let mask = NSEventMask::KeyDown
            | NSEventMask::KeyUp
            | NSEventMask::LeftMouseDown
            | NSEventMask::LeftMouseUp
            | NSEventMask::RightMouseDown
            | NSEventMask::RightMouseUp
            | NSEventMask::OtherMouseDown
            | NSEventMask::OtherMouseUp
            | NSEventMask::ScrollWheel;

        // Global monitor: fires when Arcadia is not the active app.
        if g.is_none() {
            let handler = RcBlock::new(move |ev: std::ptr::NonNull<NSEvent>| {
                let ev_ref: &NSEvent = unsafe { ev.as_ref() };
                forward_ns_event(ev_ref);
            });
            let token: *mut AnyObject = unsafe {
                msg_send![
                    objc2::class!(NSEvent),
                    addGlobalMonitorForEventsMatchingMask: mask,
                    handler: &*handler
                ]
            };
            if token.is_null() {
                return Err("addGlobalMonitorForEventsMatchingMask returned nil".into());
            }
            *g = Some(unsafe { Retained::retain(token).ok_or("retain global monitor failed")? });
        }

        // Local monitor: fires when Arcadia is the active app. The handler returns
        // the event back so normal in-app keyboard handling still works.
        if l.is_none() {
            let local_handler =
                RcBlock::new(move |ev: std::ptr::NonNull<NSEvent>| -> *mut NSEvent {
                    let ev_ref: &NSEvent = unsafe { ev.as_ref() };
                    forward_ns_event(ev_ref);
                    ev.as_ptr()
                });
            let token: *mut AnyObject = unsafe {
                msg_send![
                    objc2::class!(NSEvent),
                    addLocalMonitorForEventsMatchingMask: mask,
                    handler: &*local_handler
                ]
            };
            if !token.is_null() {
                *l = Some(unsafe {
                    Retained::retain(token).ok_or("retain local monitor failed")?
                });
            }
        }
        Ok(())
    }

    fn stop(&self) {
        for slot in [&self.global_monitor, &self.local_monitor] {
            if let Ok(mut s) = slot.lock() {
                if let Some(token) = s.take() {
                    unsafe {
                        let _: () = msg_send![objc2::class!(NSEvent), removeMonitor: &*token];
                    }
                }
            }
        }
    }

    fn is_active(&self) -> bool {
        self.global_monitor
            .lock()
            .map(|s| s.is_some())
            .unwrap_or(false)
    }

    fn permission_granted(&self) -> Option<bool> {
        Some(global_events_enabled())
    }
}

fn now_ns() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

fn forward_ns_event(ev: &NSEvent) {
    // Even though NSEvent never panics on unknown keycodes, the downstream
    // dispatcher invokes Python handlers — catch any panic to keep AppKit's
    // run-loop alive.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ts = now_ns();
        let kind = ev.r#type();
        let out: Option<KeyboardEvent> = match kind {
            NSEventType::KeyDown => Some(KeyboardEvent::KeyDown {
                keycode: ev.keyCode() as u32,
                modifiers: ev.modifierFlags().0 as u32,
                repeat: ev.isARepeat(),
                timestamp_ns: ts,
            }),
            NSEventType::KeyUp => Some(KeyboardEvent::KeyUp {
                keycode: ev.keyCode() as u32,
                modifiers: ev.modifierFlags().0 as u32,
                timestamp_ns: ts,
            }),
            NSEventType::LeftMouseDown => Some(KeyboardEvent::MouseDown {
                button: 0,
                timestamp_ns: ts,
            }),
            NSEventType::LeftMouseUp => Some(KeyboardEvent::MouseUp {
                button: 0,
                timestamp_ns: ts,
            }),
            NSEventType::RightMouseDown => Some(KeyboardEvent::MouseDown {
                button: 1,
                timestamp_ns: ts,
            }),
            NSEventType::RightMouseUp => Some(KeyboardEvent::MouseUp {
                button: 1,
                timestamp_ns: ts,
            }),
            NSEventType::OtherMouseDown => Some(KeyboardEvent::MouseDown {
                button: ev.buttonNumber().clamp(0, 255) as u8,
                timestamp_ns: ts,
            }),
            NSEventType::OtherMouseUp => Some(KeyboardEvent::MouseUp {
                button: ev.buttonNumber().clamp(0, 255) as u8,
                timestamp_ns: ts,
            }),
            NSEventType::ScrollWheel => Some(KeyboardEvent::ScrollWheel {
                dx: ev.deltaX() as f32,
                dy: ev.deltaY() as f32,
                timestamp_ns: ts,
            }),
            _ => None,
        };
        if let Some(e) = out {
            keyboard::dispatch_event(e);
        }
    }));
}

pub fn install() {
    keyboard::set_backend(Box::new(DesktopKeyboardBackend::new()));
}
