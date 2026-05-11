//! Merge static definitions, Python extensions, config overrides, and module enablement checks.

use std::collections::HashMap;

use crate::config::modules::ModulesConfig;
use crate::config::shortcuts::{ShortcutOverride, ShortcutsConfig};
use crate::config::ConfigFile;
use crate::modules::python_registry;
use crate::navigation;

use super::model::{
    EffectiveMergedShortcut, GestureEdge, HotCornerQuadrant, KeyChordSpec, ShortcutScopeStatic,
    ShortcutTrigger,
};
use super::registry::SHORTCUT_DEFINITIONS;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChordConflict {
    pub fingerprint: String,
    pub shortcut_ids: Vec<String>,
}

fn module_enabled(cfg: &ModulesConfig, name: &str) -> bool {
    cfg.modules.get(name).copied().unwrap_or(false)
}

fn apply_chord_override(
    triggers: &[ShortcutTrigger],
    chord: &KeyChordSpec,
) -> Vec<ShortcutTrigger> {
    let mut out: Vec<ShortcutTrigger> = triggers
        .iter()
        .filter(|t| !matches!(t, ShortcutTrigger::Chord(_)))
        .cloned()
        .collect();
    out.insert(0, ShortcutTrigger::Chord(chord.clone()));
    out
}

fn gather_module_gated_static(cfg: &ModulesConfig) -> Vec<EffectiveMergedShortcut> {
    let mut out = Vec::new();
    for def in SHORTCUT_DEFINITIONS {
        if let Some(m) = def.required_registry_module {
            if !module_enabled(cfg, m) {
                continue;
            }
        }
        out.push(EffectiveMergedShortcut {
            id: def.id.to_string(),
            label: def.label.to_string(),
            owner: def.owner.to_string(),
            required_registry_module: def.required_registry_module.map(str::to_string),
            scope: def.owned_scope(),
            visibility: def.visibility,
            priority: def.priority,
            consumes: def.consumes,
            bypass_text_focus: def.bypass_text_focus,
            system_wide: def.system_wide,
            triggers: def.owned_triggers(),
            actions: def.owned_actions(),
            source_extension_id: None,
        });
    }
    out
}

fn gather_python_shortcuts(cfg: &ModulesConfig) -> Vec<EffectiveMergedShortcut> {
    python_registry::list_extension_shortcuts()
        .into_iter()
        .filter(|s| {
            if let Some(ext) = &s.source_extension_id {
                python_registry::extension_enabled(ext)
                    && python_registry::extension_supported_at_runtime(ext)
            } else {
                true
            }
        })
        .filter(|s| {
            if let Some(m) = &s.required_registry_module {
                module_enabled(cfg, m)
            } else {
                true
            }
        })
        .collect()
}

fn apply_overrides(
    mut list: Vec<EffectiveMergedShortcut>,
    overrides: &std::collections::BTreeMap<String, ShortcutOverride>,
) -> Vec<EffectiveMergedShortcut> {
    for item in &mut list {
        let Some(o) = overrides.get(&item.id) else {
            continue;
        };
        if o.disabled == Some(true) {
            item.triggers.clear();
            continue;
        }
        if let Some(ref pr) = o.priority {
            item.priority = *pr;
        }
        if let Some(ref ch) = o.chord {
            item.triggers = apply_chord_override(&item.triggers, ch);
        }
    }
    list.into_iter()
        .filter(|s| !s.triggers.is_empty())
        .collect()
}

/// Full merged list honoring module/extension enablement and user overrides.
pub fn merged_shortcuts() -> Vec<EffectiveMergedShortcut> {
    let Ok(cfg) = ModulesConfig::load_or_create() else {
        return Vec::new();
    };
    let Ok(sc) = ShortcutsConfig::load_or_create() else {
        return Vec::new();
    };

    let mut list = gather_module_gated_static(&cfg);
    list.extend(gather_python_shortcuts(&cfg));
    apply_overrides(list, &sc.overrides)
}

pub fn normalized_chord_fingerprint(chord: &KeyChordSpec) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        chord.control as u8,
        chord.alt as u8,
        chord.shift as u8,
        chord.platform as u8,
        chord.function as u8,
        chord.key.to_ascii_lowercase(),
    )
}

pub fn chord_fingerprints(shortcut: &EffectiveMergedShortcut) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for t in &shortcut.triggers {
        match t {
            ShortcutTrigger::Chord(c) => {
                pairs.push((shortcut.id.clone(), normalized_chord_fingerprint(c)))
            }
            ShortcutTrigger::Sequence(seq) => {
                if let Some(first) = seq.first() {
                    pairs.push((shortcut.id.clone(), normalized_chord_fingerprint(first)));
                }
            }
            ShortcutTrigger::EdgeSwipe { .. } | ShortcutTrigger::HotCorner { .. } => {}
        }
    }
    pairs
}

pub fn chord_conflicts() -> Vec<ChordConflict> {
    let shortcuts = merged_shortcuts();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for s in &shortcuts {
        for (id, fp) in chord_fingerprints(s) {
            map.entry(fp).or_default().push(id);
        }
    }
    let mut out: Vec<ChordConflict> = map
        .into_iter()
        .filter(|(_, ids)| {
            let uniq: std::collections::HashSet<_> = ids.iter().collect();
            uniq.len() > 1
        })
        .map(|(fingerprint, shortcut_ids)| ChordConflict {
            fingerprint,
            shortcut_ids,
        })
        .collect();
    out.sort_by(|a, b| a.fingerprint.cmp(&b.fingerprint));
    out
}

pub fn chords_match(spec: &KeyChordSpec, incoming: &KeyChordSpec) -> bool {
    spec.control == incoming.control
        && spec.alt == incoming.alt
        && spec.shift == incoming.shift
        && spec.platform == incoming.platform
        && spec.function == incoming.function
        && spec.key.eq_ignore_ascii_case(&incoming.key)
}

pub fn pointer_in_hot_corner(
    quadrant: HotCornerQuadrant,
    margin_fraction: f32,
    pos_x: f32,
    pos_y: f32,
    vw: f32,
    vh: f32,
) -> bool {
    if vw <= 0. || vh <= 0. {
        return false;
    }
    let mx = margin_fraction.max(0.01) * vw;
    let my = margin_fraction.max(0.01) * vh;
    match quadrant {
        HotCornerQuadrant::TopLeft => pos_x <= mx && pos_y <= my,
        HotCornerQuadrant::TopRight => pos_x >= vw - mx && pos_y <= my,
        HotCornerQuadrant::BottomLeft => pos_x <= mx && pos_y >= vh - my,
        HotCornerQuadrant::BottomRight => pos_x >= vw - mx && pos_y >= vh - my,
    }
}

pub fn touch_hot_corner_matches(
    trigger: &ShortcutTrigger,
    pos_x: f32,
    pos_y: f32,
    vw: f32,
    vh: f32,
) -> bool {
    let ShortcutTrigger::HotCorner {
        quadrant,
        margin_fraction,
        ..
    } = trigger
    else {
        return false;
    };
    pointer_in_hot_corner(*quadrant, *margin_fraction, pos_x, pos_y, vw, vh)
}

pub fn touch_swipe_edge_matches(
    trigger: &ShortcutTrigger,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    vw: f32,
    vh: f32,
    margin_fraction: f32,
) -> bool {
    let ShortcutTrigger::EdgeSwipe { edge, min_delta_px } = trigger else {
        return false;
    };
    if vw <= 0. || vh <= 0. {
        return false;
    }
    let mx = margin_fraction.max(0.02) * vw;
    let my = margin_fraction.max(0.02) * vh;
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    match edge {
        GestureEdge::Left => start_x <= mx && dx >= *min_delta_px,
        GestureEdge::Right => start_x >= vw - mx && dx <= -*min_delta_px,
        GestureEdge::Top => start_y <= my && dy >= *min_delta_px,
        GestureEdge::Bottom => start_y >= vh - my && dy <= -*min_delta_px,
    }
}

/// Validates every static shortcut page id exists (call from tests / CI).
pub fn validate_static_shortcut_page_ids() -> Result<(), String> {
    for def in SHORTCUT_DEFINITIONS {
        if let ShortcutScopeStatic::Pages(ids) = def.scope {
            for page_id in ids {
                if navigation::page_by_id(page_id).is_none() {
                    return Err(format!(
                        "SHORTCUT_DEFINITIONS entry '{}' references unknown page '{page_id}'",
                        def.id
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_static_shortcut_page_ids;

    #[test]
    fn static_shortcut_page_ids_exist() {
        validate_static_shortcut_page_ids().expect("all static shortcut page ids must exist");
    }
}
