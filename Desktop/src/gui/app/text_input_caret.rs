//! Simple end-of-string caret for GPUI fields that use `track_focus` + `on_key_down`
//! (same pipe character as the shell prompt field).

pub(crate) const TEXT_INPUT_CARET_CHAR: char = '|';

#[inline]
pub(crate) fn text_with_trailing_caret(text: &str, focused: bool, blink_on: bool) -> String {
    if focused && blink_on {
        let mut s = String::with_capacity(text.len() + TEXT_INPUT_CARET_CHAR.len_utf8());
        s.push_str(text);
        s.push(TEXT_INPUT_CARET_CHAR);
        s
    } else {
        text.to_string()
    }
}
