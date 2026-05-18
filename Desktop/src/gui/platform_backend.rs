//! Desktop platform backend: primary display size + macOS UI metadata (menu bar height,
//! notch section widths).
//!
//! Uses `display-info` for cross-platform display geometry (no Accessibility permission needed)
//! and `NSScreen` / `NSStatusBar` on macOS for menu bar metadata.
//!
//! `menu_bar_content_widths` returns `(left_frac, right_frac)` — fractions of logical screen
//! width in [0, 1].  Callers multiply by their own logical screen width to obtain pixel
//! coordinates in whatever coordinate space they use.  Two detection sources:
//!
//! 1. **AX (Accessibility API)**: queries the focused application's `AXMenuBar` for the left
//!    boundary and ControlCenter / SystemUIServer's `AXExtrasMenuBar` for the right boundary.
//!    Requires Accessibility permission.
//!
//! 2. **Notch geometry** (notch Macs, no permission needed): `NSScreen.auxiliaryTopLeft/RightArea`
//!    sampled once at install time.  Values are normalised against the NSScreen width captured at
//!    that moment so they stay valid even when display-info and NSScreen disagree on logical width
//!    (e.g. "More Space" display mode on Apple Silicon Macs).
//!
//! Wired into `arcadia_core::modules::platform` from desktop startup via [`install`].

use arcadia_core::config::permissions::SystemGrant;
use arcadia_core::modules::platform::{self, PlatformBackend, ScreenSize};

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

pub struct DesktopPlatformBackend {
    /// `(left_end, right_start)` in NSScreen logical points on notch Macs; `None` on non-notch.
    notch_section_widths: Option<(f32, f32)>,
    /// `NSScreen.mainScreen().frame().size.width` in points, captured at install on the main
    /// thread. This is the coordinate space AX `AXPosition` values live in — used as the
    /// denominator when normalising AX menu/status boundaries to fractions. display-info's
    /// width can disagree (scaled "More Space" modes), so AX math must not use it.
    ns_screen_w: f32,
    /// PID of ControlCenter (macOS 11+) or SystemUIServer (older), captured on the main thread
    /// at install time. Avoids calling NSRunningApplication from background threads.
    #[cfg(target_os = "macos")]
    status_bar_pid: Option<i32>,
}

impl PlatformBackend for DesktopPlatformBackend {
    fn primary_screen_size(&self) -> Option<ScreenSize> {
        let infos = display_info::DisplayInfo::all().ok()?;
        let primary = infos
            .iter()
            .find(|d| d.is_primary)
            .or_else(|| infos.first())?;
        Some(ScreenSize {
            width: primary.width,
            height: primary.height,
            scale_factor: primary.scale_factor,
        })
    }

    fn menu_bar_height(&self) -> Option<f32> {
        #[cfg(target_os = "macos")]
        {
            use objc2_app_kit::NSStatusBar;
            let h = NSStatusBar::systemStatusBar().thickness() as f32;
            if h > 0.0 { Some(h) } else { None }
        }
        #[cfg(not(target_os = "macos"))]
        { None }
    }

    fn menu_bar_notch_widths(&self) -> Option<(f32, f32)> {
        self.notch_section_widths
    }


    /// Returns `(left_frac, right_frac)` in [0, 1] representing the fraction of screen width
    /// where app menus end and status items begin.  Multiply by your logical screen width.
    fn menu_bar_content_widths(&self) -> Option<(f32, f32)> {
        #[cfg(target_os = "macos")]
        {
            // AX `AXPosition` values are in NSScreen point space — use the NSScreen width
            // captured at install, not display-info (which disagrees on scaled display modes).
            let screen_w = if self.ns_screen_w > 0.0 {
                self.ns_screen_w as f64
            } else {
                self.primary_screen_size()
                    .map(|s| s.width as f64 / s.scale_factor as f64)
                    .unwrap_or(1280.0)
            };

            // 1. AX-based detection (requires Accessibility permission).
            if unsafe { AXIsProcessTrusted() } {
                let left  = ax_left_boundary();
                let right = self.status_bar_pid.and_then(|pid| ax_right_boundary(pid, screen_w));
                let l = left.unwrap_or(0.0);
                let r = right.unwrap_or(screen_w);
                // Right boundary is the authoritative check — ControlCenter always has items.
                // l=0.0 is valid when the focused app has no menu items (e.g. a bare GPUI app);
                // callers handle that by substituting a pct-based default for the left section.
                if r < screen_w && l < r {
                    return Some(((l / screen_w) as f32, (r / screen_w) as f32));
                }
            }

            // Notch geometry is intentionally NOT a fallback here — notch widths represent the
            // narrow areas adjacent to the physical notch, not actual menu/status-item boundaries.
            // Callers wanting notch geometry should use `menu_bar_notch_widths()` directly.
            None
        }
        #[cfg(not(target_os = "macos"))]
        { None }
    }

    fn prompt_system_grant(&self, grant: SystemGrant) -> bool {
        #[cfg(target_os = "macos")]
        {
            match grant {
                // Accessibility additionally shows the system consent dialog.
                SystemGrant::Accessibility => prompt_accessibility(),
                other => open_privacy_pane(privacy_anchor(other)),
            }
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = grant;
            false
        }
    }

    fn is_system_grant_active(&self, grant: SystemGrant) -> bool {
        #[cfg(target_os = "macos")]
        {
            // Only the cheap, dependency-free checks are wired. The rest report `false`
            // (the GUI then shows "Grant"); a real check needs the per-capability API
            // (AVCaptureDevice, CLLocationManager, …) and should be added when a feature
            // actually consumes that permission.
            match grant {
                SystemGrant::Accessibility   => unsafe { AXIsProcessTrusted() },
                SystemGrant::ScreenRecording => unsafe { CGPreflightScreenCaptureAccess() },
                _ => false,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = grant;
            false
        }
    }
}

/// macOS Privacy & Security settings-pane anchor for each system grant.
#[cfg(target_os = "macos")]
fn privacy_anchor(grant: SystemGrant) -> &'static str {
    match grant {
        SystemGrant::Accessibility   => "Privacy_Accessibility",
        SystemGrant::ScreenRecording => "Privacy_ScreenCapture",
        SystemGrant::Camera          => "Privacy_Camera",
        SystemGrant::Microphone      => "Privacy_Microphone",
        SystemGrant::InputMonitoring => "Privacy_ListenEvent",
        SystemGrant::Location        => "Privacy_LocationServices",
        SystemGrant::Automation      => "Privacy_Automation",
        SystemGrant::FullDiskAccess  => "Privacy_AllFiles",
        SystemGrant::Contacts        => "Privacy_Contacts",
        SystemGrant::Calendars       => "Privacy_Calendars",
        SystemGrant::Photos          => "Privacy_Photos",
        SystemGrant::Reminders       => "Privacy_Reminders",
        SystemGrant::Bluetooth       => "Privacy_Bluetooth",
    }
}

/// Open a macOS Privacy & Security settings pane so the user can flip the OS grant.
#[cfg(target_os = "macos")]
fn open_privacy_pane(anchor: &str) {
    let _ = std::process::Command::new("open")
        .arg(format!(
            "x-apple.systempreferences:com.apple.preference.security?{anchor}"
        ))
        .spawn();
}

/// macOS Accessibility settings flow. `AXIsProcessTrustedWithOptions` with the prompt option
/// shows the system consent dialog the first time (no-op once trusted); the Privacy →
/// Accessibility settings pane is always opened so the user can flip the grant either way.
#[cfg(target_os = "macos")]
fn prompt_accessibility() {
    use std::ffi::c_void;
    type Ref = *mut c_void;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: Ref) -> bool;
        static kAXTrustedCheckOptionPrompt: Ref;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFBooleanTrue: Ref;
        static kCFTypeDictionaryKeyCallBacks: c_void;
        static kCFTypeDictionaryValueCallBacks: c_void;
        fn CFDictionaryCreate(
            allocator: Ref,
            keys: *const Ref,
            values: *const Ref,
            num_values: isize,
            key_cbs: *const c_void,
            value_cbs: *const c_void,
        ) -> Ref;
        fn CFRelease(cf: Ref);
    }

    unsafe {
        let keys: [Ref; 1] = [kAXTrustedCheckOptionPrompt];
        let values: [Ref; 1] = [kCFBooleanTrue];
        let opts = CFDictionaryCreate(
            std::ptr::null_mut(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );
        if !opts.is_null() {
            AXIsProcessTrustedWithOptions(opts);
            CFRelease(opts);
        }
    }

    open_privacy_pane("Privacy_Accessibility");
}

// ── AX boundary helpers (macOS only) ─────────────────────────────────────────

/// Right edge of the rightmost app-menu item via the focused application's AXMenuBar.
/// Returns the x coordinate in display-info logical points, or `None`.
#[cfg(target_os = "macos")]
fn ax_left_boundary() -> Option<f64> {
    use std::ffi::c_void;
    use std::os::raw::c_char;
    use std::ptr;

    type Ref    = *mut c_void;
    type AXErr  = i32;
    const CF_UTF8:     u32   = 0x0800_0100;
    const VAL_CGPOINT: u32   = 1;
    const VAL_CGSIZE:  u32   = 2;
    const AX_OK:       AXErr = 0;

    #[repr(C)] struct CgPoint { x: f64, y: f64 }
    #[repr(C)] struct CgSize  { w: f64, h: f64 }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateSystemWide() -> Ref;
        fn AXUIElementCopyAttributeValue(el: Ref, attr: Ref, out: *mut Ref) -> AXErr;
        fn AXValueGetValue(val: Ref, ty: u32, out: *mut c_void) -> bool;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(alloc: Ref, s: *const c_char, enc: u32) -> Ref;
        fn CFArrayGetCount(arr: Ref) -> isize;
        fn CFArrayGetValueAtIndex(arr: Ref, i: isize) -> Ref;
        fn CFGetTypeID(cf: Ref) -> usize;
        fn CFArrayGetTypeID() -> usize;
        fn CFRelease(cf: Ref);
    }

    macro_rules! cfstr {
        ($s:expr) => {
            CFStringCreateWithCString(ptr::null_mut(), concat!($s, "\0").as_ptr() as *const c_char, CF_UTF8)
        };
    }

    unsafe {
        let system = AXUIElementCreateSystemWide();
        if system.is_null() { return None; }

        let k_focused = cfstr!("AXFocusedApplication");
        let mut focused: Ref = ptr::null_mut();
        let err = AXUIElementCopyAttributeValue(system, k_focused, &mut focused);
        CFRelease(system); CFRelease(k_focused);
        if err != AX_OK || focused.is_null() { return None; }

        let k_mb = cfstr!("AXMenuBar");
        let mut menu_bar: Ref = ptr::null_mut();
        let err = AXUIElementCopyAttributeValue(focused, k_mb, &mut menu_bar);
        CFRelease(focused); CFRelease(k_mb);
        if err != AX_OK || menu_bar.is_null() { return None; }

        let k_ch = cfstr!("AXChildren");
        let mut children: Ref = ptr::null_mut();
        let err = AXUIElementCopyAttributeValue(menu_bar, k_ch, &mut children);
        CFRelease(menu_bar); CFRelease(k_ch);
        if err != AX_OK || children.is_null() { return None; }
        if CFGetTypeID(children) != CFArrayGetTypeID() { CFRelease(children); return None; }

        let count  = CFArrayGetCount(children);
        let k_pos  = cfstr!("AXPosition");
        let k_size = cfstr!("AXSize");
        let mut left_edge: f64 = 0.0;

        for i in 0..count {
            let child = CFArrayGetValueAtIndex(children, i);
            if child.is_null() { continue; }

            let mut pv: Ref = ptr::null_mut();
            if AXUIElementCopyAttributeValue(child, k_pos, &mut pv) != AX_OK || pv.is_null() { continue; }
            let mut pt = CgPoint { x: 0.0, y: 0.0 };
            let ok = AXValueGetValue(pv, VAL_CGPOINT, &mut pt as *mut _ as *mut c_void);
            CFRelease(pv);
            if !ok { continue; }

            let mut sv: Ref = ptr::null_mut();
            if AXUIElementCopyAttributeValue(child, k_size, &mut sv) != AX_OK || sv.is_null() { continue; }
            let mut sz = CgSize { w: 0.0, h: 0.0 };
            let ok = AXValueGetValue(sv, VAL_CGSIZE, &mut sz as *mut _ as *mut c_void);
            CFRelease(sv);
            if !ok { continue; }

            let right = pt.x + sz.w;
            if right > left_edge { left_edge = right; }
        }

        CFRelease(children); CFRelease(k_pos); CFRelease(k_size);
        if left_edge > 0.0 { Some(left_edge) } else { None }
    }
}

/// Left x-coordinate of the leftmost item in ControlCenter's / SystemUIServer's
/// `AXExtrasMenuBar` — the menu-bar status-item area (clock, Wi-Fi, Control Center, …).
/// Status items live under `AXExtrasMenuBar`, NOT `AXMenuBar`; agent apps like ControlCenter
/// have no `AXMenuBar` at all.
/// `pid` is the pre-cached process ID (captured on the main thread at install time).
/// Returns the x coordinate in display-info logical points, or `None`.
#[cfg(target_os = "macos")]
fn ax_right_boundary(pid: i32, screen_w: f64) -> Option<f64> {
    use std::ffi::c_void;
    use std::os::raw::c_char;
    use std::ptr;

    type Ref    = *mut c_void;
    type AXErr  = i32;
    const CF_UTF8:     u32   = 0x0800_0100;
    const VAL_CGPOINT: u32   = 1;
    const VAL_CGSIZE:  u32   = 2;
    const AX_OK:       AXErr = 0;

    #[repr(C)] struct CgPoint { x: f64, y: f64 }
    #[repr(C)] struct CgSize  { w: f64, h: f64 }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> Ref;
        fn AXUIElementCopyAttributeValue(el: Ref, attr: Ref, out: *mut Ref) -> AXErr;
        fn AXValueGetValue(val: Ref, ty: u32, out: *mut c_void) -> bool;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(alloc: Ref, s: *const c_char, enc: u32) -> Ref;
        fn CFArrayGetCount(arr: Ref) -> isize;
        fn CFArrayGetValueAtIndex(arr: Ref, i: isize) -> Ref;
        fn CFGetTypeID(cf: Ref) -> usize;
        fn CFArrayGetTypeID() -> usize;
        fn CFRelease(cf: Ref);
    }

    macro_rules! cfstr {
        ($s:expr) => {
            CFStringCreateWithCString(ptr::null_mut(), concat!($s, "\0").as_ptr() as *const c_char, CF_UTF8)
        };
    }

    unsafe {
        let app_elem = AXUIElementCreateApplication(pid);
        if app_elem.is_null() { return None; }

        let k_mb = cfstr!("AXExtrasMenuBar");
        let mut menu_bar: Ref = ptr::null_mut();
        let err = AXUIElementCopyAttributeValue(app_elem, k_mb, &mut menu_bar);
        CFRelease(app_elem); CFRelease(k_mb);
        if err != AX_OK || menu_bar.is_null() { return None; }

        let k_ch = cfstr!("AXChildren");
        let mut children: Ref = ptr::null_mut();
        let err = AXUIElementCopyAttributeValue(menu_bar, k_ch, &mut children);
        CFRelease(menu_bar); CFRelease(k_ch);
        if err != AX_OK || children.is_null() { return None; }
        if CFGetTypeID(children) != CFArrayGetTypeID() { CFRelease(children); return None; }

        let count  = CFArrayGetCount(children);
        let k_pos  = cfstr!("AXPosition");
        let k_size = cfstr!("AXSize");
        let mut min_x: f64 = screen_w; // sentinel

        for i in 0..count {
            let child = CFArrayGetValueAtIndex(children, i);
            if child.is_null() { continue; }

            let mut pv: Ref = ptr::null_mut();
            if AXUIElementCopyAttributeValue(child, k_pos, &mut pv) != AX_OK || pv.is_null() { continue; }
            let mut pt = CgPoint { x: 0.0, y: 0.0 };
            let ok = AXValueGetValue(pv, VAL_CGPOINT, &mut pt as *mut _ as *mut c_void);
            CFRelease(pv);
            if !ok { continue; }

            // Skip zero-width children: AXExtrasMenuBar exposes hidden / inactive status
            // items (positioned left of the visible cluster) that report a 0-width size.
            let mut sv: Ref = ptr::null_mut();
            if AXUIElementCopyAttributeValue(child, k_size, &mut sv) != AX_OK || sv.is_null() { continue; }
            let mut sz = CgSize { w: 0.0, h: 0.0 };
            let ok = AXValueGetValue(sv, VAL_CGSIZE, &mut sz as *mut _ as *mut c_void);
            CFRelease(sv);
            if !ok || sz.w <= 0.0 { continue; }

            // Status items always sit in the right half; ignore left-half / x=0 elements.
            if pt.x > screen_w / 2.0 && pt.x < min_x { min_x = pt.x; }
        }

        CFRelease(children); CFRelease(k_pos); CFRelease(k_size);
        if min_x < screen_w { Some(min_x) } else { None }
    }
}

// ── CGS SPI ───────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn cgs_space_type() -> u32 {
    type CGSConnectionID = u32;
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGSMainConnectionID() -> CGSConnectionID;
        fn CGSGetActiveSpace(conn: CGSConnectionID) -> u64;
        fn CGSSpaceGetType(conn: CGSConnectionID, space_id: u64) -> u32;
    }
    unsafe {
        let conn = CGSMainConnectionID();
        let space = CGSGetActiveSpace(conn);
        CGSSpaceGetType(conn, space)
    }
}

// ── Space / Mission Control observer ─────────────────────────────────────────
//
// Spawns a dedicated thread with its own CFRunLoop. Observes
// `com.apple.spaces.changed` which fires on every active-space transition,
// including Mission Control open and close (both are space transitions).
//
// MC state is determined by CGS space type == 4 (kCGSSpaceTypeSystem /
// Exposé). The notification fires after CGS has committed the new space, so
// the check is not racy. The AtomicBool in platform::MC_ACTIVE is updated on
// every transition so `is_mission_control_active()` stays lock-free.

#[cfg(target_os = "macos")]
fn spawn_space_observer() {
    use std::ffi::c_void;
    use std::os::raw::c_char;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFNotificationCenterGetDistributedCenter() -> *mut c_void;
        fn CFNotificationCenterAddObserver(
            center: *mut c_void,
            observer: *const c_void,
            callback: unsafe extern "C" fn(
                *mut c_void, *const c_void, *mut c_void,
                *const c_void, *mut c_void,
            ),
            name: *mut c_void,
            object: *const c_void,
            behavior: i32,
        );
        fn CFStringCreateWithCString(
            alloc: *mut c_void, s: *const c_char, enc: u32,
        ) -> *mut c_void;
        fn CFRunLoopRun();
    }

    unsafe extern "C" fn on_space_change(
        _: *mut c_void, _: *const c_void, _: *mut c_void,
        _: *const c_void, _: *mut c_void,
    ) {
        let mc = cgs_space_type() == 4;
        arcadia_core::modules::platform::set_mission_control_active(mc);
        arcadia_core::modules::platform::fire_space_change_handlers();
    }

    std::thread::Builder::new()
        .name("arcadia-space-observer".into())
        .spawn(|| unsafe {
            const CF_UTF8: u32 = 0x0800_0100;
            let center = CFNotificationCenterGetDistributedCenter();
            let name = CFStringCreateWithCString(
                std::ptr::null_mut(),
                b"com.apple.spaces.changed\0".as_ptr() as *const c_char,
                CF_UTF8,
            );
            CFNotificationCenterAddObserver(
                center, std::ptr::null(), on_space_change, name,
                std::ptr::null(),
                3, // CFNotificationSuspensionBehaviorCoalesce
            );
            CFRunLoopRun();
        })
        .ok();
}

// ── Install ───────────────────────────────────────────────────────────────────

pub fn install() {
    #[cfg(target_os = "macos")]
    let (notch_section_widths, ns_screen_w): (Option<(f32, f32)>, f32) = {
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSScreen;
        // SAFETY: install() is always called from the Application::run() callback,
        // which executes on the main thread.
        unsafe {
            let mtm = MainThreadMarker::new_unchecked();
            match NSScreen::mainScreen(mtm) {
                None => (None, 0.0),
                Some(screen) => {
                    let left  = screen.auxiliaryTopLeftArea();
                    let right = screen.auxiliaryTopRightArea();
                    let sw    = screen.frame().size.width as f32;
                    let notch = if left.size.width > 0.0 && right.size.width > 0.0 {
                        Some((left.size.width as f32, sw - right.size.width as f32))
                    } else {
                        None
                    };
                    (notch, sw)
                }
            }
        }
    };
    #[cfg(not(target_os = "macos"))]
    let (notch_section_widths, ns_screen_w): (Option<(f32, f32)>, f32) = (None, 0.0);

    // Capture the status-bar process PID on the main thread so AX right-boundary queries
    // can be made from any thread without touching NSRunningApplication again.
    #[cfg(target_os = "macos")]
    let status_bar_pid: Option<i32> = {
        use objc2_app_kit::NSRunningApplication;
        use objc2_foundation::NSString;
        let candidates = ["com.apple.controlcenter", "com.apple.systemuiserver"];
        let mut found: Option<i32> = None;
        'pid: for &bid_str in &candidates {
            let bid  = NSString::from_str(bid_str);
            let apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&bid);
            if let Some(app) = apps.firstObject() {
                let p = app.processIdentifier() as i32;
                if p > 0 { found = Some(p); break 'pid; }
            }
        }
        found
    };

    platform::set_backend(Box::new(DesktopPlatformBackend {
        notch_section_widths,
        ns_screen_w,
        #[cfg(target_os = "macos")]
        status_bar_pid,
    }));

    #[cfg(target_os = "macos")]
    spawn_space_observer();
}
