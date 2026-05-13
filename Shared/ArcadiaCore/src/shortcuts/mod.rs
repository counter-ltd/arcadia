//! Registry-driven shortcuts (Arcadia-wide, page-bound, optional OS-global wiring).

mod merge;
mod model;
pub(crate) mod registry;

pub use merge::{
    chord_conflicts, chord_fingerprints, chords_match, merged_shortcuts,
    normalized_chord_fingerprint, pointer_in_hot_corner, touch_hot_corner_matches,
    touch_swipe_edge_matches, validate_static_shortcut_page_ids, ChordConflict,
};
pub use model::{
    ChordFingerprints, EffectiveMergedShortcut, GestureEdge, HotCornerQuadrant, KeyChordSpec,
    ResolvedShortcutFire, ShortcutAction, ShortcutActionStatic, ShortcutDefinition,
    ShortcutRegistrationOwned, ShortcutScope, ShortcutScopeStatic, ShortcutTrigger,
    ShortcutTriggerStatic, ShortcutTriggerStaticChord, ShortcutVisibility,
};
pub use registry::SHORTCUT_DEFINITIONS;
