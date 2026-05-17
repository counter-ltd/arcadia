//! Renders one [`Block`] as a nested, Scratch-style shape. Compound blocks are
//! containers whose body holds child blocks; nesting is expressed by actual
//! element containment, not indentation margins. Clicking a block selects it
//! (by child-index path) for editing in the canvas inspector.

use openframe::{
    div, px, rgb, AnyElement, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, Rgba, Styled,
};

use super::block_shape::{c_block_bg, leaf_bg, BUMP_H, HEADER_H, LIP_H, SPINE_W};
use crate::gui::app::code_editor_panel::{code_line_content, LineStyle};
use crate::gui::app::ArcadiaRoot;
use crate::gui::assets::MONO_FONT_FAMILY;
use crate::gui::theme::{block_accent, block_pill_text, block_soft, BlockCategory};
use arcadia_core::config::code_editor::CursorStyle;
use arcadia_core::modules::python_registry;
use arcadia_core::modules::visual_editor::blocks::{Block, BlockKind};
use arcadia_core::modules::visual_editor::codegen;

/// Shared colours / metrics / selection state for one canvas render pass.
pub(super) struct BlockRenderCtx {
    pub is_dark: bool,
    pub char_width: f32,
    pub border: Rgba,
    pub text: Rgba,
    pub surface: Rgba,
    /// Child-index path of the currently selected block, if any.
    pub selected: Option<Vec<usize>>,
}

fn classify(kind: &BlockKind) -> (&'static str, BlockCategory) {
    match kind {
        BlockKind::Module => ("module", BlockCategory::Trivia),
        BlockKind::FunctionDef { .. } => ("def", BlockCategory::Definition),
        BlockKind::ClassDef { .. } => ("class", BlockCategory::Definition),
        BlockKind::If { .. } => ("if", BlockCategory::Control),
        BlockKind::Elif { .. } => ("elif", BlockCategory::Clause),
        BlockKind::Else => ("else", BlockCategory::Clause),
        BlockKind::For { .. } => ("for", BlockCategory::Control),
        BlockKind::While { .. } => ("while", BlockCategory::Control),
        BlockKind::With { .. } => ("with", BlockCategory::Control),
        BlockKind::Try => ("try", BlockCategory::Control),
        BlockKind::Except { .. } => ("except", BlockCategory::Clause),
        BlockKind::Finally => ("finally", BlockCategory::Clause),
        BlockKind::Return { .. } => ("return", BlockCategory::Data),
        BlockKind::Import { .. } => ("import", BlockCategory::Data),
        BlockKind::Assign { .. } => ("set", BlockCategory::Data),
        BlockKind::Expr { .. } => ("call", BlockCategory::Call),
        BlockKind::Comment { .. } => ("note", BlockCategory::Trivia),
        BlockKind::Keyword { .. } => ("flow", BlockCategory::Control),
        BlockKind::Raw { .. } => ("py", BlockCategory::Raw),
    }
}

/// Header line for compounds, statement text for leaves, empty for unit kinds.
fn detail(kind: &BlockKind) -> &str {
    match kind {
        BlockKind::FunctionDef { header }
        | BlockKind::ClassDef { header }
        | BlockKind::If { header }
        | BlockKind::Elif { header }
        | BlockKind::For { header }
        | BlockKind::While { header }
        | BlockKind::With { header }
        | BlockKind::Except { header } => header,
        BlockKind::Return { text }
        | BlockKind::Import { text }
        | BlockKind::Assign { text }
        | BlockKind::Expr { text }
        | BlockKind::Comment { text }
        | BlockKind::Keyword { text }
        | BlockKind::Raw { text } => text,
        BlockKind::Module | BlockKind::Else | BlockKind::Try | BlockKind::Finally => "",
    }
}

/// Selection-ring colour painted around the selected block silhouette.
fn ring_color(is_dark: bool) -> Rgba {
    if is_dark {
        rgb(0xffffff)
    } else {
        rgb(0x1f2937)
    }
}

/// Inverted pill — sits on a solid-accent block surface.
fn pill_on_accent(label: &str, cat: BlockCategory, is_dark: bool) -> AnyElement {
    div()
        .flex_shrink_0()
        .px_1p5()
        .py_0p5()
        .rounded(px(4.))
        .bg(block_pill_text(is_dark))
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(block_accent(cat, is_dark))
        .child(label.to_string())
        .into_any_element()
}

/// Update selection + load the block's source line into the inspector draft.
fn select(this: &mut ArcadiaRoot, path: Vec<usize>, span_start: usize, window: &mut openframe::Window) {
    let idx = this
        .active_visual_editor_tab
        .min(this.visual_editor_tabs.len().saturating_sub(1));
    let line = this
        .visual_editor_tabs
        .get(idx)
        .map(|t| {
            let s = span_start.min(t.content.len());
            let e = codegen::line_end(&t.content, s);
            t.content[s..e].to_string()
        })
        .unwrap_or_default();
    this.visual_editor_edit_caret = line.len();
    this.visual_editor_edit_draft = line;
    this.visual_editor_selected = Some(path);
    this.visual_editor_input_focus.focus(window);
}

/// Render `block` and its subtree. `path` is the block's child-index path from
/// the module root.
pub(super) fn render_block(
    block: &Block,
    path: Vec<usize>,
    ctx: &BlockRenderCtx,
    cx: &mut Context<ArcadiaRoot>,
) -> AnyElement {
    let (label, cat) = classify(&block.kind);
    let selected = ctx.selected.as_deref() == Some(path.as_slice());
    match &block.kind {
        BlockKind::Raw { text } => {
            render_raw(label, text, path, block.span.start, selected, ctx, cx)
        }
        kind if kind.is_compound() => render_compound(
            label,
            cat,
            kind,
            &block.children,
            path,
            block.span.start,
            selected,
            ctx,
            cx,
        ),
        kind => render_leaf(label, cat, kind, path, block.span.start, selected, ctx, cx),
    }
}

fn render_leaf(
    label: &str,
    cat: BlockCategory,
    kind: &BlockKind,
    path: Vec<usize>,
    span_start: usize,
    selected: bool,
    ctx: &BlockRenderCtx,
    cx: &mut Context<ArcadiaRoot>,
) -> AnyElement {
    let d = detail(kind);
    let first = d.lines().next().unwrap_or("");
    let suffix = if d.lines().count() > 1 { " …" } else { "" };
    let accent = block_accent(cat, ctx.is_dark);
    let ring = if selected {
        Some(ring_color(ctx.is_dark))
    } else {
        None
    };
    div()
        .relative()
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _ev: &MouseDownEvent, window, cx| {
                select(this, path.clone(), span_start, window);
                cx.stop_propagation();
                cx.notify();
            }),
        )
        .child(leaf_bg(accent, ring))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .pt(px(BUMP_H + 5.))
                .pb(px(6.))
                .px(px(10.))
                .child(pill_on_accent(label, cat, ctx.is_dark))
                .child(
                    div()
                        .flex_1()
                        .font_family(MONO_FONT_FAMILY)
                        .text_xs()
                        .text_color(block_pill_text(ctx.is_dark))
                        .child(format!("{first}{suffix}")),
                ),
        )
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_compound(
    label: &str,
    cat: BlockCategory,
    kind: &BlockKind,
    children: &[Block],
    path: Vec<usize>,
    span_start: usize,
    selected: bool,
    ctx: &BlockRenderCtx,
    cx: &mut Context<ArcadiaRoot>,
) -> AnyElement {
    let accent = block_accent(cat, ctx.is_dark);
    let soft = block_soft(cat, ctx.is_dark);
    let ring = if selected {
        Some(ring_color(ctx.is_dark))
    } else {
        None
    };
    let header_text = detail(kind);
    let header_path = path.clone();

    // Top of the C: the painted header bar carries the pill + header line.
    let mut header = div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(HEADER_H))
        .px(px(10.))
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _ev: &MouseDownEvent, window, cx| {
                select(this, header_path.clone(), span_start, window);
                cx.stop_propagation();
                cx.notify();
            }),
        )
        .child(pill_on_accent(label, cat, ctx.is_dark));
    if !header_text.is_empty() {
        header = header.child(
            div()
                .flex_1()
                .font_family(MONO_FONT_FAMILY)
                .text_xs()
                .text_color(block_pill_text(ctx.is_dark))
                .child(header_text.to_string()),
        );
    }

    let kids: Vec<AnyElement> = children
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut cp = path.clone();
            cp.push(i);
            render_block(c, cp, ctx, cx)
        })
        .collect();

    // Children sit in the C's mouth — clear of the painted left spine.
    let mouth = div()
        .flex()
        .flex_col()
        .gap_0()
        .pl(px(SPINE_W + 6.))
        .pr(px(8.))
        .py(px(6.))
        .children(kids);

    div()
        .relative()
        .child(c_block_bg(accent, soft, ring))
        .child(
            div()
                .flex()
                .flex_col()
                .pt(px(BUMP_H))
                .child(header)
                .child(mouth)
                .child(div().h(px(LIP_H))),
        )
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn render_raw(
    label: &str,
    text: &str,
    path: Vec<usize>,
    span_start: usize,
    selected: bool,
    ctx: &BlockRenderCtx,
    cx: &mut Context<ArcadiaRoot>,
) -> AnyElement {
    let accent = block_accent(BlockCategory::Raw, ctx.is_dark);
    let ring = if selected {
        Some(ring_color(ctx.is_dark))
    } else {
        None
    };
    let style = LineStyle {
        char_width: ctx.char_width,
        line_fg: ctx.text,
        sel_bg: ctx.text,
        cursor_bg: ctx.text,
        cursor_fg: ctx.surface,
        indent_guide_color: ctx.border,
        show_indent_guides: false,
        small: true,
        cursor_style: CursorStyle::Block,
    };
    // Highlight via the registered Python provider, if any extension supplies
    // one. With no provider the lines render as plain monospace text.
    let hl = python_registry::call_highlight_provider("python", text);

    let mut line_start = 0usize;
    let mut lines: Vec<AnyElement> = Vec::new();
    for line in text.split('\n') {
        lines.push(code_line_content(line, line_start, &hl, &[], None, None, style));
        line_start += line.len() + 1;
    }

    div()
        .relative()
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _ev: &MouseDownEvent, window, cx| {
                select(this, path.clone(), span_start, window);
                cx.stop_propagation();
                cx.notify();
            }),
        )
        .child(leaf_bg(accent, ring))
        .child(
            div()
                .flex()
                .flex_col()
                .pt(px(BUMP_H))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .h(px(HEADER_H))
                        .px(px(10.))
                        .child(pill_on_accent(label, BlockCategory::Raw, ctx.is_dark))
                        .child(
                            div()
                                .text_xs()
                                .text_color(block_pill_text(ctx.is_dark))
                                .child("verbatim Python"),
                        ),
                )
                .child(
                    // Code body on an inset panel so highlighted text stays
                    // readable over the solid accent silhouette.
                    div()
                        .mx(px(8.))
                        .mb(px(8.))
                        .rounded(px(6.))
                        .bg(ctx.surface)
                        .px_2()
                        .py_1()
                        .flex()
                        .flex_col()
                        .children(lines),
                ),
        )
        .into_any_element()
}
