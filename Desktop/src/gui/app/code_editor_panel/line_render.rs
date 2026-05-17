use openframe::{div, px, rgb, rgba, AnyElement, IntoElement, ParentElement, Rgba, Styled};

use arcadia_core::config::code_editor::CursorStyle;
use arcadia_core::modules::python_registry::{DecorationRect, HighlightSpan};

use super::segments::{line_segments, SegKind};
use super::text::{char_cols, expand_tabs};
use crate::gui::assets::MONO_FONT_FAMILY;

/// Visual style for one rendered code line. Shared by the editor view and the
/// dashboard preview so both render syntax / decoration / indent output from
/// the same providers — no hard-coded, per-language styling anywhere.
#[derive(Clone, Copy)]
pub(crate) struct LineStyle {
    pub char_width: f32,
    pub line_fg: Rgba,
    pub sel_bg: Rgba,
    pub cursor_bg: Rgba,
    pub cursor_fg: Rgba,
    pub indent_guide_color: Rgba,
    pub show_indent_guides: bool,
    /// `text_xs` (preview) when true, `text_sm` (editor) when false.
    pub small: bool,
    pub cursor_style: CursorStyle,
}

/// Build the content portion of a code line: decoration backgrounds, indent
/// guides and per-character cells. The caller supplies provider output
/// (`hl_spans`, `decorations`) so extension-defined highlighting and
/// decorations render identically in the editor and in dashboard previews.
pub(crate) fn code_line_content(
    line: &str,
    line_start_byte: usize,
    hl_spans: &[HighlightSpan],
    decorations: &[DecorationRect],
    cursor_in_line: Option<usize>,
    sel_in_line: Option<(usize, usize)>,
    st: LineStyle,
) -> AnyElement {
    let cw = st.char_width;
    let display_line = expand_tabs(line);
    let segs = line_segments(line, line_start_byte, cursor_in_line, sel_in_line, hl_spans);

    let indent_levels = if st.show_indent_guides {
        display_line.chars().take_while(|&c| c == ' ').count() / 4
    } else {
        0
    };

    let mut base = div()
        .flex_1()
        .px_1()
        .py_0p5()
        .font_family(MONO_FONT_FAMILY)
        .text_color(st.line_fg)
        .relative()
        .flex()
        .flex_row()
        .items_start();
    base = if st.small { base.text_xs() } else { base.text_sm() };

    // Decoration backgrounds: absolute rects on the char grid, behind the text.
    base = decorations.iter().fold(base, |d, r| {
        let color = ((r.r as u32) << 24)
            | ((r.g as u32) << 16)
            | ((r.b as u32) << 8)
            | r.a as u32;
        d.child(
            div()
                .absolute()
                .left(px(r.col_start as f32 * cw))
                .top(px(0.))
                .bottom(px(0.))
                .w(px(r.col_width as f32 * cw))
                .bg(rgba(color)),
        )
    });

    // Indent guides: 1px lines at each 4-col boundary from the first indent
    // through the text start. Level 0 (x = 0) is omitted — the gutter border
    // marks that edge instead.
    let guide_count = if indent_levels == 0 {
        0
    } else {
        indent_levels + 1
    };
    for level in 1..guide_count {
        base = base.child(
            div()
                .absolute()
                .left(px(level as f32 * 4.0 * cw))
                .top(px(0.))
                .bottom(px(0.))
                .w(px(1.))
                .bg(st.indent_guide_color),
        );
    }

    base.children(segs.into_iter().flat_map(move |(text, kind, hl_color)| {
        let text_color = match (kind, hl_color) {
            (SegKind::Normal, Some(c)) => rgb(c),
            _ => st.line_fg,
        };
        expand_tabs(&text)
            .chars()
            .collect::<Vec<char>>()
            .into_iter()
            .map(move |ch| {
                let mut cell = div()
                    .flex_shrink_0()
                    .w(px(char_cols(ch) as f32 * cw))
                    .overflow_hidden()
                    .font_family(MONO_FONT_FAMILY)
                    .text_color(text_color);
                cell = if st.small { cell.text_xs() } else { cell.text_sm() };
                let cell = cell.child(ch.to_string());
                match kind {
                    SegKind::Normal => cell.into_any_element(),
                    SegKind::Selected => cell.bg(st.sel_bg).into_any_element(),
                    SegKind::Cursor => match st.cursor_style {
                        CursorStyle::Block => cell
                            .bg(st.cursor_bg)
                            .text_color(st.cursor_fg)
                            .into_any_element(),
                        // Bar/underline are absolute overlays so the caret never
                        // consumes layout space and shifts the glyph.
                        CursorStyle::Bar => cell
                            .relative()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(0.))
                                    .top(px(0.))
                                    .bottom(px(0.))
                                    .w(px(2.))
                                    .bg(st.cursor_bg),
                            )
                            .into_any_element(),
                        CursorStyle::Underline => cell
                            .relative()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(0.))
                                    .right(px(0.))
                                    .bottom(px(0.))
                                    .h(px(2.))
                                    .bg(st.cursor_bg),
                            )
                            .into_any_element(),
                    },
                }
            })
            .collect::<Vec<_>>()
    }))
    .into_any_element()
}
