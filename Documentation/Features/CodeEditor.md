# Code Editor

Module name: `code-editor`  
Config file: `code-editor.toml` (via `config/code_editor.rs`)  
Platform: all  
Files: `Desktop/src/gui/app/code_editor_panel.rs`, `code_editor_settings.rs`

---

## Overview

The code editor is an Arcadia-native text editor panel backed by OpenFrame. It integrates with the extension system to support syntax highlighting and decoration providers contributed by Python extensions.

---

## Extension integration

### Syntax highlighting

Python extensions can register a highlight provider:

```python
register_highlight_provider(fn)
```

The provider receives document text and returns a list of `HighlightSpan` values:

| Field | Type | Purpose |
|-------|------|---------|
| `start` | `usize` | Byte offset where this span starts |
| `end` | `usize` | Byte offset where this span ends |
| `token` | `String` | Semantic token name (e.g. `"keyword"`, `"string"`, `"comment"`) |

Token names are mapped to colours via the extension's declared style tokens. This allows per-extension theming of the code editor without hardcoding colours.

### Decorations

Python extensions can register a decoration provider:

```python
register_decoration_provider(fn)
```

The provider receives document text and a line number, and returns a list of `DecorationRect` values:

| Field | Type | Purpose |
|-------|------|---------|
| `col_start` | `usize` | Column index where the decoration begins |
| `col_width` | `usize` | Width in columns |
| `r`, `g`, `b` | `u8` | RGB colour of the decoration rectangle |

Decorations are rendered behind text as translucent rectangles. Used for inline colour previews, coverage indicators, diff gutters, etc.

---

## Theme tokens

`Desktop/src/gui/theme/modules/code_editor.rs` defines the editor's native colour tokens:

- Background and gutter colours
- Current-line highlight
- Selection fill
- Default text and comment colours (overridable by extension syntax tokens)
- Line number colour
- Cursor colour

---

## Config (`code-editor.toml`)

Managed by `CodeEditorConfig` in `config/code_editor.rs`. Stores editor preferences such as:
- Tab width
- Font size
- Soft-wrap toggle
- Any persisted editor state

---

## Reload hook

Python extensions with a registered `reload_fn` are called when the user triggers a reload from the Extensions page. For code editor extensions, this typically re-evaluates the highlight/decoration grammar without requiring a full extension restart.

---

## Dependency chain

`code-editor` has no `required_modules`. It is self-contained and can be enabled independently of other modules.
