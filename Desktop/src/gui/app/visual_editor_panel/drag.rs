//! Free-canvas drag handling. Any block can be dragged: dropped on a compound's
//! mouth it nests there, dropped on bare canvas it becomes a top-level stack.
//! Stack y-position drives source order, re-derived on every drop.

use std::cmp::Ordering;

use openframe::{Pixels, Point};

use super::block_shape::BUMP_H;
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
        if self.visual_editor.tabs.is_empty() {
            None
        } else {
            Some(
                self.visual_editor.active_tab
                    .min(self.visual_editor.tabs.len() - 1),
            )
        }
    }

    /// Pixels the pointer must move before a press becomes a drag.
    const DRAG_THRESHOLD: f32 = 4.0;

    /// Window-space height of the top-level stack `k` from last frame's bounds.
    fn stack_height(&self, k: usize) -> f32 {
        self.visual_editor.block_bounds
            .borrow()
            .iter()
            .find(|(p, _)| p.as_slice() == [k])
            .map(|(_, b)| f32::from(b.size.height))
            .unwrap_or(64.0)
    }

    /// Top-level stacks snapped flush below `lead`, walked downward — the run
    /// that shift-drag carries along with the lead block.
    fn attached_chain_below(&self, idx: usize, lead: usize) -> Vec<usize> {
        const TOL: f32 = 10.0;
        let positions = self.visual_editor.tabs[idx].block_positions.clone();
        let mut chain = Vec::new();
        let mut cur = lead;
        while let Some(&(cx, cy)) = positions.get(cur) {
            let target_y = cy + self.stack_height(cur) - BUMP_H;
            let next = positions.iter().enumerate().find(|(o, (ox, oy))| {
                *o != lead
                    && !chain.contains(o)
                    && (ox - cx).abs() < TOL
                    && (oy - target_y).abs() < TOL
            });
            match next {
                Some((o, _)) => {
                    chain.push(o);
                    cur = o;
                }
                None => break,
            }
        }
        chain
    }

    /// Arm a potential drag of the block at `path`. Stays inactive (a click)
    /// until the pointer moves past [`DRAG_THRESHOLD`]. With `shift`, a
    /// top-level block also carries the stacks snapped below it.
    pub(super) fn start_visual_drag(
        &mut self,
        path: Vec<usize>,
        pointer: Point<Pixels>,
        shift: bool,
    ) {
        // Pointer offset within the grabbed block, from last frame's bounds.
        let (grab_x, grab_y) = self
            .visual_editor.block_bounds
            .borrow()
            .iter()
            .find(|(p, _)| p == &path)
            .map(|(_, b)| {
                (
                    f32::from(pointer.x) - f32::from(b.origin.x),
                    f32::from(pointer.y) - f32::from(b.origin.y),
                )
            })
            .unwrap_or((18.0, 10.0));
        let followers = match (shift && path.len() == 1, self.visual_drag_idx(), path.first()) {
            (true, Some(idx), Some(&k)) => self.attached_chain_below(idx, k),
            _ => Vec::new(),
        };
        self.visual_editor.drag = Some(VisualDrag {
            path,
            origin: pointer,
            cursor: pointer,
            grab_x,
            grab_y,
            active: false,
            followers,
        });
    }

    /// Track the pointer; promote a pending press to an active drag once it
    /// moves past the threshold.
    pub(super) fn update_visual_drag(&mut self, pointer: Point<Pixels>) {
        if let Some(drag) = self.visual_editor.drag.as_mut() {
            drag.cursor = pointer;
            if !drag.active {
                let dx = f32::from(pointer.x) - f32::from(drag.origin.x);
                let dy = f32::from(pointer.y) - f32::from(drag.origin.y);
                if dx.abs() > Self::DRAG_THRESHOLD || dy.abs() > Self::DRAG_THRESHOLD {
                    drag.active = true;
                }
            }
        }
    }

    /// Resolve the drop target under `cursor`: the deepest compound mouth
    /// containing it (excluding the dragged block's own subtree) and the
    /// insertion index among that compound's children, picked by cursor y.
    pub(super) fn compute_drop(
        &self,
        cursor: Point<Pixels>,
        dragged: &[usize],
    ) -> Option<(Vec<usize>, usize)> {
        let target = {
            let zones = self.visual_editor.drop_zones.borrow();
            zones
                .iter()
                .filter(|z| z.bounds.contains(&cursor))
                .filter(|z| !within_subtree(&z.path, dragged))
                .max_by_key(|z| z.path.len())
                .map(|z| z.path.clone())?
        };
        let bounds = self.visual_editor.block_bounds.borrow();
        let mut ci = 0usize;
        loop {
            let mut child = target.clone();
            child.push(ci);
            let Some((_, b)) = bounds.iter().find(|(p, _)| p == &child) else {
                break;
            };
            let mid = f32::from(b.origin.y) + f32::from(b.size.height) / 2.0;
            if f32::from(cursor.y) < mid {
                return Some((target, ci));
            }
            ci += 1;
        }
        Some((target, ci))
    }

    /// Finish a drag: reparent into a compound mouth or reposition on the free
    /// canvas, then re-derive top-level source order from stack y-positions.
    pub(super) fn finish_visual_drag(&mut self) {
        let Some(drag) = self.visual_editor.drag.take() else {
            return;
        };
        // A press that never moved is a click — selection already happened on
        // mouse-down, so there is nothing to commit.
        if !drag.active {
            return;
        }
        let Some(idx) = self.visual_drag_idx() else {
            return;
        };
        let cursor = drag.cursor;
        let path = drag.path;
        let (grab_x, grab_y) = (drag.grab_x, drag.grab_y);
        let followers = drag.followers;
        let was_top = path.len() == 1;

        // A shift-drag of an attached stack only repositions — never reparents.
        let target = if followers.is_empty() {
            self.compute_drop(cursor, &path)
        } else {
            None
        };

        let origin = *self.visual_editor.canvas_origin.borrow();
        let drop_pos = (
            (f32::from(cursor.x) - f32::from(origin.x) - grab_x).max(0.0),
            (f32::from(cursor.y) - f32::from(origin.y) - grab_y).max(0.0),
        );

        let content = self.visual_editor.tabs[idx].content.clone();
        let tree = parse::parse(&content);
        let Some(dragged) = tree.block_at(&path) else {
            return;
        };
        let (ds, de) = (dragged.span.start, dragged.span.end);

        match target {
            Some((target_path, insert_idx)) => {
                let Some(tb) = tree.block_at(&target_path) else {
                    return;
                };
                let anchor = if insert_idx == 0 {
                    tb.span.start
                } else {
                    let mut prev = target_path.clone();
                    prev.push(insert_idx - 1);
                    match tree.block_at(&prev) {
                        Some(b) => b.span.end,
                        None => codegen::line_bounds(&content, tb.span.start, tb.span.end).1,
                    }
                };
                let indent = format!("{}    ", codegen::indent_at(&content, tb.span.start));
                let new_content = codegen::move_block(&content, ds, de, anchor, &indent);
                if new_content != content {
                    self.visual_editor.tabs[idx].content = new_content;
                    if was_top {
                        if let Some(k) = path.first().copied() {
                            let positions = &mut self.visual_editor.tabs[idx].block_positions;
                            if k < positions.len() {
                                positions.remove(k);
                            }
                        }
                    }
                    self.visual_editor.selected = None;
                }
            }
            None => {
                if was_top {
                    if let Some(k) = path.first().copied() {
                        // The dragged set: lead block plus any shift-drag followers.
                        let mut group = vec![k];
                        group.extend(followers.iter().copied());
                        match self.find_snap(idx, &group, k, drop_pos) {
                            Some((sibling, below)) => {
                                self.reflow_stack(idx, &group, sibling, below);
                            }
                            None => {
                                self.place_group(idx, k, &followers, drop_pos);
                            }
                        }
                        self.visual_editor.selected = None;
                    }
                } else {
                    // Promote a nested block to a top-level stack.
                    let anchor = content.len();
                    let new_content = codegen::move_block(&content, ds, de, anchor, "");
                    if new_content != content {
                        self.visual_editor.tabs[idx].content = new_content;
                        self.visual_editor.tabs[idx].block_positions.push(drop_pos);
                        self.visual_editor.selected = None;
                    }
                }
            }
        }

        self.resort_visual_top_level(idx);
        self.save_visual_editor_session();
        if let Some(path) = self.visual_editor.tabs[idx].file_path.clone() {
            visual_editor::save_block_positions(
                &path,
                &self.visual_editor.tabs[idx].block_positions,
            );
        }
    }

    /// Find a sibling stack whose top or bottom edge the dropped lead block
    /// lands near. Returns `(sibling index, dropped below it)`.
    fn find_snap(
        &self,
        idx: usize,
        exclude: &[usize],
        lead: usize,
        pos: (f32, f32),
    ) -> Option<(usize, bool)> {
        const SNAP: f32 = 28.0;
        let dragged_h = self.stack_height(lead);
        let positions = self.visual_editor.tabs[idx].block_positions.clone();
        let mut best: Option<(usize, bool, f32)> = None;
        for (other, opos) in positions.iter().enumerate() {
            if exclude.contains(&other) || (pos.0 - opos.0).abs() >= SNAP {
                continue;
            }
            let oh = self.stack_height(other);
            let d_below = (pos.1 - (opos.1 + oh)).abs();
            if d_below < SNAP && best.map(|(_, _, d)| d_below < d).unwrap_or(true) {
                best = Some((other, true, d_below));
            }
            let d_above = ((pos.1 + dragged_h) - opos.1).abs();
            if d_above < SNAP && best.map(|(_, _, d)| d_above < d).unwrap_or(true) {
                best = Some((other, false, d_above));
            }
        }
        best.map(|(s, b, _)| (s, b))
    }

    /// Walk up to the top of the flush stack containing top-level block `k`.
    fn stack_top(&self, idx: usize, k: usize) -> usize {
        const TOL: f32 = 10.0;
        let positions = self.visual_editor.tabs[idx].block_positions.clone();
        let mut cur = k;
        loop {
            let Some(&(cx, cy)) = positions.get(cur) else {
                break;
            };
            let pred = positions.iter().enumerate().find(|(o, (ox, oy))| {
                *o != cur
                    && (ox - cx).abs() < TOL
                    && ((oy + self.stack_height(*o) - BUMP_H) - cy).abs() < TOL
            });
            match pred {
                Some((o, _)) => cur = o,
                None => break,
            }
        }
        cur
    }

    /// The full flush stack containing `k`, ordered top to bottom.
    fn stack_of(&self, idx: usize, k: usize) -> Vec<usize> {
        let top = self.stack_top(idx, k);
        let mut stack = vec![top];
        stack.extend(self.attached_chain_below(idx, top));
        stack
    }

    /// Insert the dragged `group` into `sibling`'s flush stack and cascade
    /// every member's position, pushing the rest of the stack down to make
    /// room — an "insert here" reflow.
    fn reflow_stack(&mut self, idx: usize, group: &[usize], sibling: usize, below: bool) {
        let mut members: Vec<usize> = self
            .stack_of(idx, sibling)
            .into_iter()
            .filter(|m| !group.contains(m))
            .collect();
        let Some(sib_idx) = members.iter().position(|&m| m == sibling) else {
            return;
        };
        let insert_at = if below { sib_idx + 1 } else { sib_idx };
        for (i, &g) in group.iter().enumerate() {
            let at = (insert_at + i).min(members.len());
            members.insert(at, g);
        }

        let positions = self.visual_editor.tabs[idx].block_positions.clone();
        let anchor_x = positions.get(sibling).map(|p| p.0).unwrap_or(40.0);
        let anchor_y = members
            .iter()
            .filter(|m| !group.contains(m))
            .filter_map(|&m| positions.get(m).map(|p| p.1))
            .fold(f32::MAX, f32::min);
        let anchor_y = if anchor_y == f32::MAX {
            positions.get(sibling).map(|p| p.1).unwrap_or(40.0)
        } else {
            anchor_y
        };

        let heights: Vec<f32> = members.iter().map(|&m| self.stack_height(m)).collect();
        let positions = &mut self.visual_editor.tabs[idx].block_positions;
        let mut y = anchor_y;
        for (i, &m) in members.iter().enumerate() {
            if m < positions.len() {
                positions[m] = (anchor_x, y);
            }
            y += heights[i] - BUMP_H;
        }
    }

    /// Place a shift-dragged group: the lead at `lead_pos`, each follower
    /// stacked flush below it (overlapped by the bump height).
    fn place_group(
        &mut self,
        idx: usize,
        lead: usize,
        followers: &[usize],
        lead_pos: (f32, f32),
    ) {
        let lead_h = self.stack_height(lead);
        let follower_h: Vec<f32> = followers.iter().map(|&f| self.stack_height(f)).collect();
        let positions = &mut self.visual_editor.tabs[idx].block_positions;
        if lead < positions.len() {
            positions[lead] = lead_pos;
        }
        let mut y = lead_pos.1 + lead_h - BUMP_H;
        for (i, &f) in followers.iter().enumerate() {
            if f < positions.len() {
                positions[f] = (lead_pos.0, y);
            }
            y += follower_h[i] - BUMP_H;
        }
    }

    /// Reconcile the positions list, then re-emit top-level blocks top-to-bottom
    /// by y so the source order always matches the visual layout.
    fn resort_visual_top_level(&mut self, idx: usize) {
        let content = self.visual_editor.tabs[idx].content.clone();
        let tree = parse::parse(&content);
        let n = tree.root.children.len();
        {
            let positions = &mut self.visual_editor.tabs[idx].block_positions;
            while positions.len() < n {
                let i = positions.len();
                positions.push((40.0, 40.0 + i as f32 * 150.0));
            }
            positions.truncate(n);
        }
        let positions = self.visual_editor.tabs[idx].block_positions.clone();
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
            self.visual_editor.tabs[idx].content = rewritten;
            self.visual_editor.tabs[idx].block_positions =
                order.iter().map(|&oi| positions[oi]).collect();
        }
    }
}
