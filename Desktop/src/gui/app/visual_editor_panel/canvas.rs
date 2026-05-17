use openframe::{
    canvas, div, font, px, rgb, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    MouseButton, MouseMoveEvent, ParentElement, StatefulInteractiveElement, Styled, Window,
};

use super::block_render::{render_block, BlockRenderCtx};
use super::block_shape::{Edges, BUMP_H};
use crate::gui::app::ArcadiaRoot;
use crate::gui::assets::MONO_FONT_FAMILY;
use crate::gui::theme;
use arcadia_core::config::visual_editor;
use arcadia_core::modules::visual_editor::parse;

impl ArcadiaRoot {
    /// Free-canvas block editor: top-level stacks are placed at stored (x, y)
    /// positions and can be dragged anywhere. Stack y-order drives the source
    /// order of the canonical `.py`.
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
                    "Empty document — add blocks from the palette to start.",
                ))
                .into_any_element();
        }

        // Lazily load saved positions from the sidecar on first render.
        if self.visual_editor_tabs[idx].block_positions.is_empty() {
            if let Some(path) = self.visual_editor_tabs[idx].file_path.clone() {
                let loaded = visual_editor::load_block_positions(&path);
                if !loaded.is_empty() {
                    self.visual_editor_tabs[idx].block_positions = loaded;
                }
            }
        }

        // Drop targets + per-block bounds are repopulated each frame.
        self.visual_editor_drop_zones.borrow_mut().clear();
        self.visual_editor_block_bounds.borrow_mut().clear();

        let char_width = {
            let font_size = window.rem_size() * 0.75;
            let ts = window.text_system();
            let fid = ts.resolve_font(&font(MONO_FONT_FAMILY));
            ts.ch_advance(fid, font_size).map(f32::from).unwrap_or(7.0)
        };

        let tree = parse::parse(&self.visual_editor_tabs[idx].content);
        let n = tree.root.children.len();

        // Reconcile the positions list with the current top-level block count.
        {
            let positions = &mut self.visual_editor_tabs[idx].block_positions;
            while positions.len() < n {
                let i = positions.len();
                positions.push((40.0, 40.0 + i as f32 * 150.0));
            }
            positions.truncate(n);
        }
        let positions = self.visual_editor_tabs[idx].block_positions.clone();

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

        // Live drop preview from the previous frame's zones / bounds.
        let drag_info = self
            .visual_editor_drag
            .as_ref()
            .filter(|d| d.active)
            .map(|d| (d.cursor, d.path.clone()));
        let drop_preview =
            drag_info.and_then(|(cur, dp)| self.compute_drop(cur, &dp));

        let ctx = BlockRenderCtx {
            is_dark,
            char_width,
            border: p.panel_border,
            text: p.content_title,
            surface: p.panel_bg,
            selected,
            drop_zones: self.visual_editor_drop_zones.clone(),
            block_bounds: self.visual_editor_block_bounds.clone(),
            drop_preview,
        };
        let placed: Vec<(f32, f32, AnyElement)> = tree
            .root
            .children
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let (x, y) = positions[i];
                (x, y, render_block(b, vec![i], Edges::full(), &ctx, cx))
            })
            .collect();

        let inspector = self.visual_editor_inspector(sel_valid, can_up, can_down, window, cx, is_dark);

        // Canvas extent — grow to fit the furthest-placed stack.
        let max_x = positions.iter().map(|(x, _)| *x).fold(0.0_f32, f32::max) + 640.0;
        let max_y = positions.iter().map(|(_, y)| *y).fold(0.0_f32, f32::max) + 520.0;

        let origin_cell = self.visual_editor_canvas_origin.clone();
        let mut area = div()
            .relative()
            .min_w(px(max_x))
            .min_h(px(max_y))
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _, cx| {
                if this.visual_editor_drag.is_some() {
                    this.update_visual_drag(ev.position);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    if this.visual_editor_drag.is_some() {
                        this.finish_visual_drag();
                        cx.notify();
                    }
                }),
            )
            .child(
                canvas(
                    |_, _, _| {},
                    move |bounds, _, _, _| {
                        *origin_cell.borrow_mut() = bounds.origin;
                    },
                )
                .absolute()
                .size_full(),
            );
        for (x, y, el) in placed {
            area = area.child(div().absolute().left(px(x)).top(px(y)).child(el));
        }

        // Drag ghost — a translucent copy of the dragged block following the
        // cursor. Only shown once the press promotes to an actual drag.
        let ghost = self
            .visual_editor_drag
            .as_ref()
            .filter(|d| d.active)
            .map(|d| {
                (
                    d.path.clone(),
                    d.cursor,
                    d.grab_x,
                    d.grab_y,
                    d.followers.clone(),
                )
            });
        if let Some((gpath, gcursor, grab_x, grab_y, followers)) = ghost {
            let origin = *self.visual_editor_canvas_origin.borrow();
            let gx = (f32::from(gcursor.x) - f32::from(origin.x) - grab_x).max(0.0);
            let gy = (f32::from(gcursor.y) - f32::from(origin.y) - grab_y).max(0.0);
            let mut col = div().flex().flex_col();
            let mut any = false;
            if let Some(block) = tree.block_at(&gpath) {
                col = col.child(render_block(block, gpath.clone(), Edges::full(), &ctx, cx));
                any = true;
            }
            for f in &followers {
                let fpath = vec![*f];
                if let Some(block) = tree.block_at(&fpath) {
                    let el = render_block(block, fpath, Edges::full(), &ctx, cx);
                    col = col.child(div().mt(px(-BUMP_H)).child(el));
                    any = true;
                }
            }
            if any {
                area = area.child(
                    div()
                        .absolute()
                        .left(px(gx))
                        .top(px(gy))
                        .opacity(0.7)
                        .child(col),
                );
            }
        }

        div()
            .id("visual-editor-canvas")
            .w_full()
            .h_full()
            .bg(canvas_bg)
            .overflow_x_scroll()
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
                    .child("BLOCKS — drag a stack to move it; vertical order sets code order"),
            )
            .child(inspector)
            .child(area)
            .into_any_element()
    }
}
