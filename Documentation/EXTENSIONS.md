# Extension Development Guide

Python extensions live in `~/Arcadia/Extensions/`. Arcadia scans this directory on startup, discovers them, and lets users opt in via **Extensions settings** (`python.settings`). No Rust changes required for new extensions unless you're adding a new capability to the Python API itself.

---

## Directory Layout

```
~/Arcadia/Extensions/
  my_ext/
    main.py          ← required entry point
    Assets/
      icon.svg       ← optional: sidebar icon (24×24 SVG, currentColor stroke)
      *.py           ← optional: local modules imported from main.py
  loose.py           ← single-file alternative (Assets/ resolved to Extensions/Assets/)
```

Local imports from `main.py`:

```python
# Extensions/my_ext/main.py
import sys, os
_dir = os.path.dirname(os.path.abspath(__file__))
if _dir not in sys.path:
    sys.path.insert(0, _dir)
import helpers  # loads Extensions/my_ext/helpers.py
```

---

## Minimal Extension

```python
import arcadia

arcadia.register_module(
    "my-ext",
    "1.0.0",
    "Does something useful.",
)

arcadia.register_command(
    "my-ext.hello",
    "Returns a greeting.",
    lambda args: f"Hello, {args[0] if args else 'world'}!",
)
```

---

## Full API Reference

### Registration

```python
arcadia.register_module(
    name: str,
    version: str,
    description: str,
    permissions: list[str] = [],   # declared before first-enable; user approves
    platforms: list[str] = [],     # [] = all; or ["macos", "windows", "linux", "ios"]
)
```

```python
arcadia.register_command(
    token: str,                        # "ext-id.verb" format required
    description: str,
    handler: Callable[[list[str]], str],
    permissions: list[str] = [],
)
```

```python
arcadia.register_nav_page(
    extension_id: str,
    group_id: str,                     # "utilities" | "network" | "social" | "code" | "ai"
    *,
    title: str,
    description: str,
    glyph: str = "extensions",        # icon key or "extension-icon/<ext-id>" (auto-resolves Assets/icon.svg)
    system_image: str = "square.grid.2x2",  # SF Symbol for iOS
    accent: str = "amber",            # palette key: amber emerald sky violet rose teal fuchsia indigo cyan
    status_command: str | None = None, # called on render: must return JSON {text: str, active: bool}
    actions: list[dict] | None = None, # see Action dict below
)
```

**Action dict:**
```python
{"label": str, "command": str, "args": list[str], "style": "primary"|"secondary"|"destructive"}
```
`style` defaults to `"secondary"`. The generic panel renders all actions as buttons; clicking dispatches the command.

**Status command contract:** return JSON with at minimum `{"text": "...", "active": bool}`. `text` is displayed as a badge; `active` controls badge color (info = active, muted = inactive).

```python
arcadia.register_style(
    name: str, label: str, description: str,
    *, module: str | None = None,
    bg=None, surface=None, surface2=None, text=None, dim=None,
    ui_font_family=None, border=None, accent=None,
    border_chars=None, border_horizontal_pattern=None,
    border_vertical_pattern=None, border_font_family=None,
    border_font_size_rems=None, border_side_rail_px=None,
    border_radius=0.0,
    light_bg=None, ...  # light_ prefix for all color fields
)
```

```python
arcadia.register_tokens(
    module_name: str,
    token_specs: list[dict],
)
# token spec keys: key, label, kind (color/float/string/bool/int),
#                  default, min, max, granularity, visible_when
```

```python
arcadia.register_editor_token_module(extension_id: str)
# marks extension tokens as Editor-scoped (appear in Editor settings, not standalone page)
```

```python
arcadia.register_shortcut_json(extension_id: str, json_spec: str)
```

```python
arcadia.register_highlight_provider(
    extension_id: str,
    language: str,
    handler: Callable[[str], list[tuple[int, int, str]]],
    # handler(text) → [(byte_start, byte_end, token_name), ...]
)
```

```python
arcadia.register_decoration_provider(
    extension_id: str,
    handler: Callable[[str, int], list[dict]],
    # handler(line, line_idx) → [{"col_start":int,"col_width":int,"r":int,"g":int,"b":int,"a":int}, ...]
)
```

---

### Execution

```python
arcadia.execute(token: str, args: list[str]) -> str
# Dispatches any registered command (core or extension). Permission checks apply.
```

```python
arcadia.read_tokens(module_id: str) -> dict[str, str]
# Returns current merged token values for an extension.
```

---

### Tray Icons

Requires `permissions=["tray.create"]` in `register_module`.

```python
tray_id = arcadia.tray_register(extension_id, label, tooltip=None, instance=None)
arcadia.tray_set_image(extension_id, tray_id, rgba_bytes: bytes, width: int, height: int)
arcadia.tray_set_tooltip(extension_id, tray_id, text: str)
arcadia.tray_set_menu(extension_id, tray_id, items: list[dict])
# item dict: {"label": str, "command": str, "args": list[str]} or {"separator": True}

arcadia.tray_set_show_menu_on_left_click(extension_id, tray_id, enable: bool)
arcadia.register_tray_icon_click_handler(extension_id, handler)
# handler(tray_id: str, button: str) -> None   button = "left" | "right"

arcadia.tray_icon_screen_bounds(extension_id, tray_id)
# → (x, y, width, height, pixels_per_point) | None

arcadia.tray_remove(extension_id, tray_id)
```

---

### Timers & Animation

```python
task_id = arcadia.set_timer(extension_id, interval_ms: int, callback)
# callback() fires every interval_ms. Returns task_id.
arcadia.cancel_timer(task_id)

tween_id = arcadia.animate(extension_id, duration_ms: int, easing: str, callback)
# easing: "linear" | "ease_out_cubic" | "ease_in_cubic" | "ease_in_out_cubic" | "ease_out_elastic"
# callback(t: float)  t ∈ [0.0, 1.0]
arcadia.cancel_animation(tween_id)
```

---

### Cursor & Screen

Requires `permissions=["cursor.global_position"]`.

```python
arcadia.cursor_position(extension_id) -> tuple[float, float] | None
arcadia.cursor_snapshot(extension_id) -> tuple[float, float, bool, bool] | None
# → (x, y, left_down, right_down)
arcadia.screen_size(extension_id) -> tuple[int, int] | None
```

---

### Overlay HUD

Requires `permissions=["overlay.hud"]`. macOS/Windows/Linux only.

```python
arcadia.overlay_display_size(extension_id) -> tuple[int, int] | None
arcadia.overlay_hud_set_sprite(
    extension_id, rgba_bytes: bytes, width: int, height: int,
    pad_right=None, pad_bottom=None, display_width=None, display_height=None,
)
arcadia.overlay_hud_clear_sprite(extension_id)
```

---

### Assets

```python
arcadia.extension_assets_path(extension_id) -> str | None
# → absolute path to <bundle>/Assets/

arcadia.read_extension_asset(extension_id, relative: str) -> bytes
# relative must not contain ".." or absolute paths
```

---

## Permissions Reference

| Permission ID | Grants |
|---|---|
| `tray.create` | Register/modify system tray icons and menus |
| `cursor.global_position` | Read global cursor position and screen size |
| `cursor.global_mouse_buttons` | Read mouse button down state |
| `overlay.hud` | Control the always-on-top overlay HUD sprite |
| `python.host` | Trigger full extension reload |
| `python.extension_toggle` | Enable/disable extensions programmatically |

---

## Nav Page Pattern (Utilities / Network / etc.)

For any extension that wants a page in a sidebar group:

```python
arcadia.register_nav_page(
    "my-ext", "utilities",
    title="My Tool",
    description="Short description shown as subtitle.",
    glyph="extension-icon/my-ext",    # resolves Assets/icon.svg automatically
    system_image="wrench",
    accent="sky",
    status_command="my-ext.status",   # optional; return {"text": "...", "active": bool}
    actions=[
        {"label": "Run",  "command": "my-ext.run",  "args": [],       "style": "primary"},
        {"label": "Reset","command": "my-ext.reset", "args": [],       "style": "destructive"},
    ],
)
```

The page ID is always `python.nav_page|{extension_id}`. The generic panel (`extension_nav_panel.rs`) renders it — no per-extension Rust panel needed. The sidebar entry appears automatically once the extension is enabled.

**Status command example:**
```python
def handle_status(args):
    return json.dumps({"text": "Active — 2h 30m", "active": True})
    # or
    return json.dumps({"text": "Idle", "active": False})
```

---

## Settings Page (Token Overrides)

Extensions that call `register_tokens` without owning an Appearance style automatically get a standalone Settings hub page at `python.extension_tokens|{ext_id}`. No extra registration needed.

```python
arcadia.register_tokens("my-ext", [
    {"key": "refresh_ms", "label": "Refresh rate (ms)", "kind": "int",
     "default": "500", "min": "100", "max": "5000"},
    {"key": "color",      "label": "Accent color",      "kind": "color",
     "default": "#3b82f6"},
])
# Read values at runtime:
vals = arcadia.read_tokens("my-ext")
color = vals.get("color", "#3b82f6")
```

---

## Lifecycle

| Phase | What happens |
|---|---|
| Startup | Loader scans `~/Arcadia/Extensions/`, pre-parses permissions/platforms, creates disabled stubs |
| First enable | User toggles ON in Extensions settings; body (`main.py`) executes; registration calls run |
| Runtime | Commands dispatched via `execute_command`; timers fire; tray items live |
| Disable | All registrations cleared (commands, styles, tokens, tray, timers, nav pages) |
| Reload | Full clear + re-scan + re-execute all enabled extensions |

Extensions are **disabled by default**. The user must explicitly opt in.

---

## Example Extensions

| Extension | What it demonstrates |
|---|---|
| `Extensions/hello/` | Minimal: 3 commands, `arcadia.execute()` to call core |
| `Extensions/taurine/` | Nav page + tray + subprocess + countdown timer + status command |
| `Extensions/overlay_pet/` | Overlay HUD sprite, timer loop, style tokens, local module imports |
| `Extensions/rainbow_indent/` | Decoration provider, `read_tokens()` for live color customisation |
| `Extensions/rust_syntax/` | Highlight provider, regex tokenisation |
| `Extensions/default_theme/` | `register_style` with dark/light mode glyphs and 30 color tokens |

---

## Icon

Drop a 24×24 SVG at `Assets/icon.svg`. Use `stroke="currentColor"` — Arcadia applies theme tinting. Pass `glyph="extension-icon/<ext-id>"` to `register_nav_page` and the system resolves the file automatically.

If no icon is provided, the sidebar falls back to the generic extensions glyph.
