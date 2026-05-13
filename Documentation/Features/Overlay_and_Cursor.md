# Overlay & Cursor

Overlay module: `overlay`  
Cursor module: `cursor`  
Platform: macOS, Windows, Linux (both modules; iOS and headless have no backend)

---

## Overlay

### Overview

The `overlay` module provides a single always-on-top transparent HUD window. Its primary consumer is Python extensions that declare `overlay.hud` in their required permissions. The window supports pointer pass-through — input events fall through to windows beneath it.

### Stacking levels

The overlay window supports four canonical stacking tokens (stable, never change their string values):

| Token | Meaning |
|-------|---------|
| `normal` | Standard window level — appears in normal window stack |
| `floating` | Floats above normal windows |
| `hud` | HUD level — above floating, below system UI; requires `overlay.hud` permission |
| `system_ui` | System UI level — highest; requires both `overlay.hud` and `overlay.system_ui` permissions |

### Commands

| Command | Usage | Description |
|---------|-------|-------------|
| `overlay.show` | `overlay.show` | Make the overlay window visible. Requires `overlay.hud` permission. |
| `overlay.hide` | `overlay.hide` | Hide the overlay window. |
| `overlay.set-stacking` | `overlay.set-stacking <token>` | Set stacking level: `normal`, `floating`, `hud`, `system_ui`. `system_ui` requires additional `overlay.system_ui` permission. |

### Backend architecture

The overlay module uses a backend pattern:
- `overlay` module in `arcadia-core` defines the interface and a pending-request queue.
- The desktop GUI foreground pump installs the concrete `OverlayBackend` at startup.
- Visibility requests are queued by the module; the GUI tick applies them on the next frame cycle so `AsyncApp` is never stored in a `Send` mutex.
- On desktop app quit, all windows (including the overlay) are dropped by the GPUI runtime.
- Headless and iOS have no backend — overlay commands return errors.

### Permissions

| Permission | Required for |
|------------|-------------|
| `overlay.hud` | `overlay.show`, `overlay.set-stacking` (any token) |
| `overlay.system_ui` | `overlay.set-stacking system_ui` (in addition to `overlay.hud`) |

The permission subject is `PermissionSubject::module("overlay")`.

### Companion auto-enable

Python extensions declaring `overlay.hud` in `required_permissions` trigger `ensure_native_companions_for_loaded_extension`. This automatically:
1. Grants `overlay.hud` to the `overlay` module subject.
2. Enables the `overlay` module (with requirements).

Users do not need to manually enable `overlay` for HUD extensions.

### HUD sprites

`modules/overlay_hud_sprite.rs` defines sprite data types for content rendered inside the HUD overlay. Sprites are drawn by the overlay backend's render loop.

---

## Cursor

### Overview

The `cursor` module provides OS-global cursor position and primary display size for extensions that track or respond to mouse input. Examples: overlay extensions that follow the cursor, gesture extensions, screen-region tools.

### Backend interface

```rust
pub trait CursorBackend: Send + Sync {
    fn position(&self) -> Option<CursorPosition>;
    fn primary_screen_size(&self) -> Option<ScreenSize>;
    fn snapshot(&self) -> Option<CursorSnapshot> { None }
}
```

`set_backend(b)` installs the platform implementation at startup. Without a backend, all queries return `None`.

### Types

| Type | Fields |
|------|--------|
| `CursorPosition` | `x: f64`, `y: f64` |
| `ScreenSize` | `width: u32`, `height: u32` |
| `CursorSnapshot` | `x: f64`, `y: f64`, `left_button: bool`, `right_button: bool` |

### Commands

| Command | Usage | Description |
|---------|-------|-------------|
| `cursor.position` | `cursor.position` | Returns `{"x": …, "y": …}` or error if no backend |
| `cursor.screen-size` | `cursor.screen-size` | Returns `{"width": …, "height": …}` or error |
| `cursor.snapshot` | `cursor.snapshot` | Returns `{"x":…,"y":…,"left_button":…,"right_button":…}` including button state if supported |

### Platform notes

| Platform | Backend |
|----------|---------|
| macOS | `NSEvent.mouseLocation` + `NSScreen.main.frame` |
| Windows | `GetCursorPos` + `GetSystemMetrics` |
| Linux | X11 or Wayland cursor query |
| iOS | No backend — all queries return `None` |
| Headless | No backend — all queries return `None` |

---

## Typical extension pattern

```python
# Extension declaring overlay.hud
register_module(
    name="my-hud",
    version="1.0.0",
    description="Cursor-following HUD overlay",
    required_permissions=["overlay.hud"],
)

def on_tick():
    pos = execute("cursor.snapshot")  # get cursor position + buttons
    execute("overlay.show")           # ensure HUD visible
    # draw HUD content at cursor position
```

Enabling this extension automatically enables `overlay` and grants `overlay.hud` — the user only enables the extension.
