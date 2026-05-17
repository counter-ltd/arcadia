use openframe::{
    div, font, rgb, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, StatefulInteractiveElement, Styled, Window,
};

use super::block_render::{render_block, BlockRenderCtx};
use crate::gui::app::ArcadiaRoot;
use crate::gui::assets::MONO_FONT_FAMILY;
use crate::gui::theme;
use arcadia_core::modules::visual_editor::parse;

impl ArcadiaRoot {
    /// Phase 4: parse the tab's Python source into a block tree, render it as
    /// nested Scratch-style blocks, and host the inspector bar for editing the
    /// selected block. Block edits splice the canonical buffer, which re-parses
    /// on the next frame — the bidirectional loop.
    pub(super) fn visual_editor_canvas(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let p = theme::theme_palette(cx, is_dark);
        let idx = self
            .active_visual_editor_tab
            .min(self.visual_editor_tabs.len().saturating_sub(1));

        let canvas_bg = if is_dark { rgb(0x1a1f29) } else { rgb(0xfafafa) };

        if self.visual_editor_tabs[idx].content.trim().is_empty() {
            return div()
                .id("visual-editor-canvas")
                .w_full()
                .h_full()
                .bg(canvas_bg)
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .child(
                    theme::render_icon("blocks")
                        .size_8()
                        .text_color(p.content_meta),
                )
                .child(div().text_sm().text_color(p.content_meta).child(
                    "Empty document — drag-and-drop block authoring lands in a later phase.",
                ))
                .into_any_element();
        }

        // Monospace cell width — Raw blocks render on the editor's char grid.
        let char_width = {
            let font_size = window.rem_size() * 0.75;
            let ts = window.text_system();
            let fid = ts.resolve_font(&font(MONO_FONT_FAMILY));
            ts.ch_advance(fid, font_size).map(f32::from).unwrap_or(7.0)
        };

        let tree = parse::parse(&self.visual_editor_tabs[idx].content);

        // Resolve selection state for the inspector.
        let selected = self.visual_editor_selected.clone();
        let sel_valid = selected
            .as_ref()
            .map(|path| tree.block_at(path).is_some())
            .unwrap_or(false);
        let (can_up, can_down) = match selected.as_ref().filter(|_| sel_valid) {
            Some(path) => match (tree.siblings_at(path), path.last()) {
                (Some(sibs), Some(&pos)) => (pos > 0, pos + 1 < sibs.len()),
                _ => (false, false),
            },
            None => (false, false),
        };

        let ctx = BlockRenderCtx {
            is_dark,
            char_width,
            border: p.panel_border,
            text: p.content_title,
            surface: p.panel_bg,
            selected,
        };
        let block_els: Vec<AnyElement> = tree
            .root
            .children
            .iter()
            .enumerate()
            .map(|(i, b)| render_block(b, vec![i], &ctx, cx))
            .collect();

        let inspector = self.visual_editor_inspector(sel_valid, can_up, can_down, window, cx, is_dark);

        div()
            .id("visual-editor-canvas")
            .w_full()
            .h_full()
            .bg(canvas_bg)
            .overflow_y_scroll()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.content_meta)
                    .child("BLOCKS"),
            )
            .child(inspector)
            .child(div().flex().flex_col().gap_0().children(block_els))
            .into_any_element()
    }
}
