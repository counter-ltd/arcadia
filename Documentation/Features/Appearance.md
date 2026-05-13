# Appearance

Config file: `appearance.toml`  
Theme files: `Desktop/src/gui/theme/`  
Platform: all (theme tokens used by OpenFrame on desktop and iOS)

---

## Overview

Arcadia's appearance system has three layers:

1. **Native theme tokens** — Rust constants in `Desktop/src/gui/theme/` used directly in OpenFrame view code.
2. **Active style** — a string ID (`appearance.toml`) selecting which extension or style is active.
3. **Extension tokens** — Python extensions declaring `StyleTokenSpec` overrides that the GUI renders as configurable sliders, color pickers, and toggles.

---

## Config (`appearance.toml`)

| Field | Default | Purpose |
|-------|---------|---------|
| `active_style` | `"default"` | ID of the active style extension or `"default"` for the built-in theme |

### Legacy migrations

`AppearanceConfig::merge_defaults()` collapses old style IDs:

| Old value | Maps to |
|-----------|---------|
| `"flux"` | `"terminal"` |
| `"shell"` | `"terminal"` |

---

## Native theme system

### Directory layout

```
Desktop/src/gui/theme/
  mod.rs             — top-level token exports, SURFACE_BG, accent helpers
  palette.rs         — base colour palette constants
  icons.rs           — icon_path() mapping glyph key → asset path
  splash_colors.rs   — splash screen gradient colours
  modules/
    code_editor.rs   — code editor-specific tokens
    panel.rs         — panel/card surface tokens
    row_surface.rs   — list row tokens
    typography.rs    — font size/weight constants
    mod.rs
  nav_accents/
    amber.rs cyan.rs emerald.rs fuchsia.rs indigo.rs
    orange.rs sky.rs teal.rs violet.rs
    palette.rs       — AccentPalette struct: bg, fg, selected, border, icon tints
    mod.rs           — accent_palette(key) dispatcher
```

### Invariant

Never use `rgb(0x...)` inline in view code. All colours must be named constants from `theme/` or resolved via `accent_color(accent)`. This keeps the entire visual identity centralised and swappable.

### Accent palettes

Each navigation page and group declares an `accent` key (e.g. `"emerald"`, `"violet"`). `accent_palette(key)` returns an `AccentPalette`:

| Field | Purpose |
|-------|---------|
| `bg` | Background fill for selected state |
| `fg` | Foreground/text colour for selected state |
| `selected` | Selection highlight colour |
| `border` | Border/separator colour in this accent |
| `icon_tint` | Icon tint for the page glyph |

Available accent keys: `amber`, `cyan`, `emerald`, `fuchsia`, `indigo`, `orange`, `sky`, `teal`, `violet`, and any key resolving to a default.

### Icons

`icon_path(glyph_key)` in `theme/icons.rs` maps page/group glyph strings to asset paths under `Desktop/assets/icons/`. Adding a new icon:
1. Add SVG to `Desktop/assets/icons/`.
2. Add a match arm in `icon_path()`.
3. Use the key in `NavigationPageDefinition.glyph`.

---

## Extension style tokens

Python extensions can declare `StyleTokenSpec` values to expose configurable appearance parameters. The token system (`modules/style_tokens.rs`) defines:

### Token kinds

| Kind | UI control | Config value |
|------|-----------|--------------|
| `Color` | Colour picker (hex) | `#rrggbb` string |
| `Numeric` | Slider or number field | `f64` within declared bounds |
| `Enum` | Dropdown / segmented control | String from declared variants |
| `Bool` | Toggle | Boolean |

### `StyleTokenNumericBounds`

Defines `min`, `max`, and an optional `StyleTokenNumericGranularity`:

| Granularity | Meaning |
|-------------|---------|
| `Integer` | Snap to whole numbers |
| `Step(f64)` | Snap to multiples of step |
| `Float` | Free float within bounds |

Helper functions:
- `snap_slider_value(value, granularity)` — round to nearest valid step
- `clamp_numeric_display_for_spec(value, spec)` — clamp to `[min, max]`
- `format_slider_value(value, granularity)` — format for display (integer: no decimal; float: 2dp)
- `resolve_slider_numeric(raw, bounds)` — parse stored string to f64

### Visibility rules (`StyleTokenVisibility`)

Tokens can be conditionally shown based on another token's value:

```rust
StyleTokenVisibility::When {
    key: "some_other_token",
    op: StyleTokenCompareOp::Eq,
    value: "enabled",
}
```

`style_token_row_visible(token, all_tokens)` evaluates the visibility rule against current token values.

### Extension token settings UI

`Desktop/src/gui/app/extension_token_settings.rs` and `appearance/extension_tokens.rs` render per-extension token panels. Each active extension with declared tokens gets a section showing its token controls.

---

## Appearance settings page

Page: `global.appearance` (always visible)  
File: `Desktop/src/gui/app/appearance/`

| Sub-file | Purpose |
|----------|---------|
| `panel.rs` | Root appearance panel — active style selector, extension token sections |
| `extension_tokens.rs` | Renders per-extension token controls |
| `color_picker_modal.rs` | Modal colour picker for `Color` token kind |
| `mod.rs` | Module exports |

The panel reads `AppearanceConfig` to show/highlight the active style. Style switching writes `active_style` to `appearance.toml`.
