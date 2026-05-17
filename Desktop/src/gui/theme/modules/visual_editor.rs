//! Block colour tokens for the visual Python editor canvas.

use openframe::{rgb, Rgba};

/// Visual category of a block — drives accent, fill and pill colours.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlockCategory {
    /// `def` / `class`.
    Definition,
    /// `if` / `for` / `while` / `with` / `try` / flow keywords.
    Control,
    /// `elif` / `else` / `except` / `finally`.
    Clause,
    /// `return` / `import` / assignment.
    Data,
    /// Bare expression statement (typically a call).
    Call,
    /// Comments and structural trivia.
    Trivia,
    /// Unmodelled syntax rendered as verbatim Python.
    Raw,
}

/// Saturated accent — container edge, header bar, kind pill.
pub fn block_accent(cat: BlockCategory, is_dark: bool) -> Rgba {
    use BlockCategory::*;
    let hex = match (cat, is_dark) {
        (Definition, false) => 0x7c3aed,
        (Definition, true) => 0x8b5cf6,
        (Control, false) => 0x2563eb,
        (Control, true) => 0x3b82f6,
        (Clause, false) => 0x0891b2,
        (Clause, true) => 0x06b6d4,
        (Data, false) => 0x059669,
        (Data, true) => 0x10b981,
        (Call, false) => 0xd97706,
        (Call, true) => 0xf59e0b,
        (Trivia, false) => 0x6b7280,
        (Trivia, true) => 0x9ca3af,
        (Raw, false) => 0xdc2626,
        (Raw, true) => 0xef4444,
    };
    rgb(hex)
}

/// Low-alpha accent tint — header bar background, leaf block fill.
pub fn block_soft(cat: BlockCategory, is_dark: bool) -> Rgba {
    let acc = block_accent(cat, is_dark);
    Rgba {
        a: if is_dark { 0.20 } else { 0.12 },
        ..acc
    }
}

/// Text colour for the kind pill, sitting on a solid [`block_accent`] fill.
pub fn block_pill_text(is_dark: bool) -> Rgba {
    if is_dark {
        rgb(0x141820)
    } else {
        rgb(0xffffff)
    }
}
