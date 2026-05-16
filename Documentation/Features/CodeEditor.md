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

Managed by `CodeEditorConfig` in `config/code_editor.rs`:
- `show_indentation_marks` — draw indent guide lines
- `char_width_override` — manual monospace advance width (`None` = auto-measure)
- `auto_indent` — new lines inherit the previous line's leading whitespace
- `auto_close_brackets` — insert the matching closer for `([{"'`
- `undo_enabled` — per-tab undo/redo history
- `line_commands` — enable the editor line commands
- `cursor_style` — `CursorStyle`: `block` / `bar` / `underline`

All settings are exposed on the **Editor** settings page (`editor.settings`).

---

## Commands & shortcuts

Editor commands are registered as static shortcuts (`shortcuts/registry.rs`,
ids `editor:*`), page-scoped to `editor.main` with `bypass_text_focus: true`.
They fire `UiControl { control_id: "editor.*" }`, handled by
`fire_shortcut_ui_control` → `ArcadiaRoot::editor_run_command` in
`code_editor_panel/commands.rs`.

| Command | Default chord | `control_id` |
|---------|---------------|--------------|
| Save file | Cmd+S | `editor.save` |
| Select line | Cmd+L | `editor.select_line` |
| Duplicate line | Cmd+Shift+D | `editor.duplicate_line` |
| Delete line | Cmd+Shift+K | `editor.delete_line` |
| Move line up/down | Alt+Up / Alt+Down | `editor.move_line_up` / `_down` |
| Indent / outdent | Cmd+] / Cmd+[ | `editor.indent` / `editor.outdent` |
| Toggle line comment | Cmd+/ | `editor.toggle_comment` |
| Undo / redo | Cmd+Z / Cmd+Shift+Z | `editor.undo` / `editor.redo` |

Raw text input (typing, arrows, backspace, enter, tab) stays in
`code_editor_panel/edit.rs::apply_key` — too high-frequency for the shortcut
registry. `apply_key` also owns auto-indent, auto-close and undo recording.
Undo history lives in `ArcadiaRoot.code_editor_undo` (`EditorUndoMap`, keyed by
tab id), with a coalescing flag so a run of typing folds into one undo step.

---

## Reload hook

Python extensions with a registered `reload_fn` are called when the user triggers a reload from the Extensions page. For code editor extensions, this typically re-evaluates the highlight/decoration grammar without requiring a full extension restart.

---

## Dependency chain

`code-editor` has no `required_modules`. It is self-contained and can be enabled independently of other modules.
