use openframe::Modifiers;

/// Map a GPUI key event to PTY byte sequence.
pub fn key_to_bytes(key: &str, mods: Modifiers) -> Option<Vec<u8>> {
    if mods.control && !mods.alt && !mods.platform {
        return match key {
            "c" | "C" => Some(vec![0x03]),
            "d" | "D" => Some(vec![0x04]),
            "z" | "Z" => Some(vec![0x1a]),
            "l" | "L" => Some(vec![0x0c]),
            "a" | "A" => Some(vec![0x01]),
            "b" | "B" => Some(vec![0x02]),
            "e" | "E" => Some(vec![0x05]),
            "f" | "F" => Some(vec![0x06]),
            "k" | "K" => Some(vec![0x0b]),
            "n" | "N" => Some(vec![0x0e]),
            "p" | "P" => Some(vec![0x10]),
            "r" | "R" => Some(vec![0x12]),
            "u" | "U" => Some(vec![0x15]),
            "w" | "W" => Some(vec![0x17]),
            _ => None,
        };
    }
    match key {
        "enter" => Some(vec![b'\r']),
        "backspace" => Some(vec![0x7f]),
        "escape" => Some(vec![0x1b]),
        "tab" => Some(vec![b'\t']),
        "up" => Some(vec![0x1b, b'[', b'A']),
        "down" => Some(vec![0x1b, b'[', b'B']),
        "right" => Some(vec![0x1b, b'[', b'C']),
        "left" => Some(vec![0x1b, b'[', b'D']),
        "home" => Some(vec![0x1b, b'[', b'H']),
        "end" => Some(vec![0x1b, b'[', b'F']),
        "pageup" | "page_up" => Some(vec![0x1b, b'[', b'5', b'~']),
        "pagedown" | "page_down" => Some(vec![0x1b, b'[', b'6', b'~']),
        "delete" => Some(vec![0x1b, b'[', b'3', b'~']),
        "f1" => Some(vec![0x1b, b'O', b'P']),
        "f2" => Some(vec![0x1b, b'O', b'Q']),
        "f3" => Some(vec![0x1b, b'O', b'R']),
        "f4" => Some(vec![0x1b, b'O', b'S']),
        "f5" => Some(vec![0x1b, b'[', b'1', b'5', b'~']),
        "f6" => Some(vec![0x1b, b'[', b'1', b'7', b'~']),
        "f7" => Some(vec![0x1b, b'[', b'1', b'8', b'~']),
        "f8" => Some(vec![0x1b, b'[', b'1', b'9', b'~']),
        "f9" => Some(vec![0x1b, b'[', b'2', b'0', b'~']),
        "f10" => Some(vec![0x1b, b'[', b'2', b'1', b'~']),
        "f11" => Some(vec![0x1b, b'[', b'2', b'3', b'~']),
        "f12" => Some(vec![0x1b, b'[', b'2', b'4', b'~']),
        _ => None,
    }
}
