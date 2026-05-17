//! Free-canvas drag handling. Any block can be dragged: dropped on a compound's
//! mouth it nests there, dropped on bare canvas it becomes a top-level stack.
//! Stack y-position drives source order, re-derived on every drop.

use std::cmp::Ordering;

use openframe::{Pixels, Point};

use crate::gui::app::{ArcadiaRoot, VisualDrag};
use arcadia_core::config::visual_editor;
use arcadia_core::modules::visual_editor::{codegen, parse};

/// True when `zone` is the dragged block itself or one of its descendants —
/// such targets must be rejected (a block cannot be dropped inside itself).
fn within_subtree(zone: &[usize], dragged: &[usize]) -> bool {
    zone.len() >= dragged.len() && &zone[..dragged.len()] == dragged
}

impl ArcadiaRoot {
    fn visual_drag_idx(&self) -> Option<usize> {
        if self.visual_editor_tabs.is_empty() {
            None
        } else {
            Some(
                self.active_visual_editor_tab
                    .min(self.visual_editor_tabs.len() - 1),
            )
        }
    }

    /// Begin dragging the block at `path`; `label` shows in the drag ghost.
    pub(super) fn start_visual_drag(
        &mut self,
        path: Vec<usize>,
        label: String,
        pointer: Point<Pixels>,
    ) {
        self.visual_editor_drag = Some(VisualDrag {
            path,
            label,
            cursor: pointer,
        });
    }

    /// Update the drag ghost to follow `pointer`.
    pub(super) fn update_visual_drag(&mut self, pointer: Point<Pixels>) {
        if let Some(drag) = self.visual_editor_drag.as_mut() {
            drag.cursor = pointer;
        }
    }

    /// Finish a drag: reparent into a compound mouth or reposition on the free
    /// canvas, then re-derive top-level source order from stack y-positions.
    pub(super) fn finish_visual_drag(&mut self) {
        let Some(drag) = self.visual_editor_drag.take() else {
            return;
        };
        let Some(idx) = self.visual_drag_idx() else {
            return;
        };
        let cursor = drag.cursor;
        let path = drag.path;
        let was_top = path.len() == 1;

        // Deepest mouth zone under the cursor that is not the dragged subtree.
        let target: Option<Vec<usize>> = {
            let zones = self.visual_editor_drop_zones.borrow();
            zones
                .iter()
                .filter(|z| z.bounds.contains(&cursor))
                .filter(|z| !within_subtree(&z.path, &path))
                .max_by_key(|z| z.path.len())
                .map(|z| z.path.clone())
        };

        let origin = *self.visual_editor_canvas_origin.borrow();
        let drop_pos = (
            (f32::from(cursor.x) - f32::from(origin.x)).max(0.0),
            (f32::from(cursor.y) - f32::from(origin.y)).max(0.0),
        );

        let content = self.visual_editor_tabs[idx].content.clone();
        let tree = parse::parse(&content);
        let Some(dragged) = tree.block_at(&path) else {
            return;
        };
        let (ds, de) = (dragged.span.start, dragged.span.end);

        match target {
            Some(target_path) => {
                let Some(tb) = tree.block_at(&target_path) else {
                    return;
                };
                let anchor = codegen::line_bounds(&content, tb.span.start, tb.span.end).1;
                let indent = format!("{}    ", codegen::indent_at(&content, tb.span.start));
                let new_content = codegen::move_block(&content, ds, de, anchor, &indent);
                if new_content != content {
                    self.visual_editor_tabs[idx].content = new_content;
                    if was_top {
                        if let Some(k) = path.first().copied() {
                            let positions = &mut self.visual_editor_tabs[idx].block_positions;
                            if k < positions.len() {
                                positions.remove(k);
                            }
                        }
                    }
                    self.visual_editor_selected = None;
                }
            }
            None => {
                if was_top {
                    if let Some(k) = path.first().copied() {
                        if let Some(slot) =
                            self.visual_editor_tabs[idx].block_positions.get_mut(k)
                        {
                            *slot = drop_pos;
                        }
                    }
                } else {
                    // Promote a nested block to a top-level stack.
                    let anchor = content.len();
                    let new_content = codegen::move_block(&content, ds, de, anchor, "");
                    if new_content != content {
                        self.visual_editor_tabs[idx].content = new_content;
                        self.visual_editor_tabs[idx].block_positions.push(drop_pos);
                        self.visual_editor_selected = None;
                    }
                }
            }
        }

        self.resort_visual_top_level(idx);
        self.save_visual_editor_session();
        if let Some(path) = self.visual_editor_tabs[idx].file_path.clone() {
            visual_editor::save_block_positions(
                &path,
                &self.visual_editor_tabs[idx].block_positions,
            );
        }
    }

    /// Reconcile the positions list, then re-emit top-level blocks top-to-bottom
    /// by y so the source order always matches the visual layout.
    fn resort_visual_top_level(&mut self, idx: usize) {
        let content = self.visual_editor_tabs[idx].content.clone();
        let tree = parse::parse(&content);
        let n = tree.root.children.len();
        {
            let positions = &mut self.visual_editor_tabs[idx].block_positions;
            while positions.len() < n {
                let i = positions.len();
                positions.push((40.0, 40.0 + i as f32 * 150.0));
            }
            positions.truncate(n);
        }
        let positions = self.visual_editor_tabs[idx].block_positions.clone();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by(|&a, &b| {
            positions[a]
                .1
                .partial_cmp(&positions[b].1)
                .unwrap_or(Ordering::Equal)
        });
        if order != (0..n).collect::<Vec<_>>() {
            let mut rewritten = String::new();
            for &oi in &order {
                let b = &tree.root.children[oi];
                let s = b.span.start.min(content.len());
                let e = b.span.end.min(content.len());
                rewritten.push_str(&content[s..e]);
                rewritten.push('\n');
            }
            self.visual_editor_tabs[idx].content = rewritten;
            self.visual_editor_tabs[idx].block_positions =
                order.iter().map(|&oi| positions[oi]).collect();
        }
    }
}
