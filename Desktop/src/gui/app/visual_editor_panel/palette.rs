//! Block palette panel — a right rail listing insertable blocks. Clicking an
//! entry splices its snippet into the canonical buffer at the selected block
//! (into a compound's body, or after a leaf) or at end of file.

use openframe::{
    div, px, AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, StatefulInteractiveElement, Styled,
};

use crate::gui::app::ArcadiaRoot;
use crate::gui::theme;
use arcadia_core::modules::python_registry;
use arcadia_core::modules::visual_editor::codegen;
use arcadia_core::modules::visual_editor::palette::{
    builtin_arcadia_blocks, builtin_python_blocks, BlockDef,
};
use arcadia_core::modules::visual_editor::parse;

impl ArcadiaRoot {
    /// Splice `snippet` into the active tab. Insertion target: the body of the
    /// selected compound block, after the selected leaf, or end of file.
    pub(super) fn visual_editor_insert_block(&mut self, snippet: &str) {
        if self.visual_editor.tabs.is_empty() {
            return;
        }
        let idx = self
            .visual_editor.active_tab
            .min(self.visual_editor.tabs.len() - 1);
        let content = self.visual_editor.tabs[idx].content.clone();

        let (anchor, indent) = match &self.visual_editor.selected {
            Some(path) => {
                let tree = parse::parse(&content);
                match tree.block_at(path) {
                    Some(b) if b.kind.is_compound() => {
                        let header_indent = codegen::indent_at(&content, b.span.start);
                        (b.span.end, format!("{header_indent}    "))
                    }
                    Some(b) => (b.span.start, codegen::indent_at(&content, b.span.start)),
                    None => (content.len(), String::new()),
                }
            }
            None => (content.len(), String::new()),
        };

        let updated = codegen::insert_block(&content, anchor, &indent, snippet);
        self.visual_editor.tabs[idx].content = updated;
        self.save_visual_editor_session();
    }

    /// The block palette right rail.
    pub(crate) fn visual_editor_palette_panel(
        &mut self,
        cx: &mut Context<Self>,
        is_dark: bool,
    ) -> AnyElement {
        let bg = theme::explorer_sidebar_bg(is_dark);
        let border = theme::explorer_border(is_dark);
        let text = theme::explorer_text(is_dark);
        let dim = theme::explorer_dim(is_dark);
        let hover = theme::explorer_hover_bg(is_dark);

        // Merge the three palette sources, preserving category grouping order.
        let mut groups: Vec<(String, Vec<BlockDef>)> = Vec::new();
        let mut add = |defs: Vec<BlockDef>| {
            for d in defs {
                match groups.iter_mut().find(|(c, _)| *c == d.category) {
                    Some(g) => g.1.push(d),
                    None => groups.push((d.category.clone(), vec![d])),
                }
            }
        };
        add(builtin_python_blocks());
        add(builtin_arcadia_blocks());
        add(python_registry::list_blocks());

        let mut panel = div()
            .id("visual-editor-palette")
            .w(px(232.))
            .h_full()
            .flex_shrink_0()
            .bg(bg)
            .border_l_1()
            .border_color(border)
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(text)
                    .border_b_1()
                    .border_color(border)
                    .child("Palette"),
            );

        for (category, defs) in groups {
            panel = panel.child(
                div()
                    .px_3()
                    .pt_2()
                    .pb_1()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(dim)
                    .child(category),
            );
            for def in defs {
                let snippet = def.snippet.clone();
                panel = panel.child(
                    div()
                        .id(openframe::SharedString::from(format!("palette:{}", def.id)))
                        .mx_2()
                        .mb_0p5()
                        .px_2()
                        .py_1()
                        .rounded(px(6.))
                        .text_xs()
                        .text_color(text)
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover))
                        .child(def.label.clone())
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.visual_editor_insert_block(&snippet);
                                cx.notify();
                            }),
                        ),
                );
            }
        }

        panel.into_any_element()
    }
}
