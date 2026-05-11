//! Shortcut definitions used by static registry and merged runtime shortcut lists.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum ShortcutVisibility {
    /// Listed only under owning module/extension settings UI (typically UI-only shortcuts).
    ModuleSettingsOnly,
    /// Listed only in the global shortcuts preferences panel.
    GlobalPrefsOnly,
    /// Listed in both.
    Both,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ShortcutScope {
    ArcadiaWide,
    PageBound(Vec<String>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShortcutScopeStatic {
    ArcadiaWide,
    Pages(&'static [&'static str]),
}

/// Modifier chord aligned with OpenFrame [`Modifiers`] naming (`platform` = Cmd / Win / Super).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct KeyChordSpec {
    pub key: String,
    #[serde(default)]
    pub control: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub platform: bool,
    #[serde(default)]
    pub function: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum GestureEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum HotCornerQuadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Trigger variants; chord matching is implemented first; pointer-driven triggers share the same fire path.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ShortcutTrigger {
    Chord(KeyChordSpec),
    /// Leader sequence; matched when steps align within timeout (handled in GUI).
    Sequence(Vec<KeyChordSpec>),
    EdgeSwipe {
        edge: GestureEdge,
        /// Minimum drag distance in px (approximate; GUI converts pointer deltas).
        min_delta_px: f32,
    },
    HotCorner {
        quadrant: HotCornerQuadrant,
        /// Margin from edges as fraction of view width/height (e.g. 0.08).
        margin_fraction: f32,
        dwell_frames: u32,
    },
}

/// Compile-time trigger row for [`ShortcutDefinition`].
#[derive(Clone, Copy, Debug)]
pub enum ShortcutTriggerStatic {
    Chord {
        key: &'static str,
        control: bool,
        alt: bool,
        shift: bool,
        platform: bool,
        function: bool,
    },
    Sequence(&'static [ShortcutTriggerStaticChord]),
    EdgeSwipe {
        edge: GestureEdge,
        min_delta_px: f32,
    },
    HotCorner {
        quadrant: HotCornerQuadrant,
        margin_fraction: f32,
        dwell_frames: u32,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct ShortcutTriggerStaticChord {
    pub key: &'static str,
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub platform: bool,
    pub function: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ShortcutAction {
    ExecuteCommand {
        token: String,
        #[serde(default)]
        args: Vec<String>,
    },
    Navigate {
        page_id: String,
    },
    UiControl {
        control_id: String,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ShortcutActionStatic {
    ExecuteCommand {
        token: &'static str,
        args: &'static [&'static str],
    },
    Navigate {
        page_id: &'static str,
    },
    UiControl {
        control_id: &'static str,
    },
}

/// Static compile-time shortcut definition (Rust modules / core).
#[derive(Clone, Debug)]
pub struct ShortcutDefinition {
    pub id: &'static str,
    pub label: &'static str,
    /// Owning namespace — `MODULE_REGISTRY` module key or `"arcadia"` for core shell shortcuts.
    pub owner: &'static str,
    /// When set, owning module must be enabled (`ModulesConfig`) before shortcut applies.
    pub required_registry_module: Option<&'static str>,
    pub scope: ShortcutScopeStatic,
    pub visibility: ShortcutVisibility,
    pub priority: i16,
    pub consumes: bool,
    /// When true, shortcut may fire while text/TUI has focus (used sparingly).
    pub bypass_text_focus: bool,
    /// Desktop hook registers OS-global listener when supported (macOS first).
    pub system_wide: bool,
    pub triggers: &'static [ShortcutTriggerStatic],
    pub actions: &'static [ShortcutActionStatic],
}

fn trigger_static_to_owned(t: ShortcutTriggerStatic) -> ShortcutTrigger {
    match t {
        ShortcutTriggerStatic::Chord {
            key,
            control,
            alt,
            shift,
            platform,
            function,
        } => ShortcutTrigger::Chord(KeyChordSpec {
            key: key.to_string(),
            control,
            alt,
            shift,
            platform,
            function,
        }),
        ShortcutTriggerStatic::Sequence(steps) => ShortcutTrigger::Sequence(
            steps
                .iter()
                .map(|s| KeyChordSpec {
                    key: s.key.to_string(),
                    control: s.control,
                    alt: s.alt,
                    shift: s.shift,
                    platform: s.platform,
                    function: s.function,
                })
                .collect(),
        ),
        ShortcutTriggerStatic::EdgeSwipe { edge, min_delta_px } => {
            ShortcutTrigger::EdgeSwipe { edge, min_delta_px }
        }
        ShortcutTriggerStatic::HotCorner {
            quadrant,
            margin_fraction,
            dwell_frames,
        } => ShortcutTrigger::HotCorner {
            quadrant,
            margin_fraction,
            dwell_frames,
        },
    }
}

fn action_static_to_owned(a: ShortcutActionStatic) -> ShortcutAction {
    match a {
        ShortcutActionStatic::ExecuteCommand { token, args } => ShortcutAction::ExecuteCommand {
            token: token.to_string(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
        },
        ShortcutActionStatic::Navigate { page_id } => ShortcutAction::Navigate {
            page_id: page_id.to_string(),
        },
        ShortcutActionStatic::UiControl { control_id } => ShortcutAction::UiControl {
            control_id: control_id.to_string(),
        },
    }
}

impl ShortcutDefinition {
    pub fn owned_scope(&self) -> ShortcutScope {
        match self.scope {
            ShortcutScopeStatic::ArcadiaWide => ShortcutScope::ArcadiaWide,
            ShortcutScopeStatic::Pages(ids) => {
                ShortcutScope::PageBound(ids.iter().map(|s| (*s).to_string()).collect())
            }
        }
    }

    pub fn owned_triggers(&self) -> Vec<ShortcutTrigger> {
        self.triggers
            .iter()
            .copied()
            .map(trigger_static_to_owned)
            .collect()
    }

    pub fn owned_actions(&self) -> Vec<ShortcutAction> {
        self.actions
            .iter()
            .copied()
            .map(action_static_to_owned)
            .collect()
    }
}

/// Actions produced after resolving user overrides (strings fully owned).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolvedShortcutFire {
    ExecuteCommand { token: String, args: Vec<String> },
    Navigate { page_id: String },
    UiControl { owner: String, control_id: String },
}

#[derive(Clone, Debug, Default)]
pub struct ChordFingerprints {
    pub chords: Vec<(String, String)>,
}

/// Runtime shortcut row after merging static defs, Python registrations, and config overrides.
#[derive(Clone, Debug)]
pub struct EffectiveMergedShortcut {
    pub id: String,
    pub label: String,
    pub owner: String,
    pub required_registry_module: Option<String>,
    pub scope: ShortcutScope,
    pub visibility: ShortcutVisibility,
    pub priority: i16,
    pub consumes: bool,
    pub bypass_text_focus: bool,
    pub system_wide: bool,
    pub triggers: Vec<ShortcutTrigger>,
    pub actions: Vec<ShortcutAction>,
    /// When `Some`, shortcut originated from that Python extension (`register_shortcut`).
    pub source_extension_id: Option<String>,
}

/// JSON/dict registration shape for Python extensions (validated before acceptance).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortcutRegistrationOwned {
    pub id: String,
    pub label: String,
    pub owner: String,
    #[serde(default)]
    pub required_registry_module: Option<String>,
    pub scope: ShortcutScope,
    pub visibility: ShortcutVisibility,
    pub priority: i16,
    #[serde(default = "default_true")]
    pub consumes: bool,
    #[serde(default)]
    pub bypass_text_focus: bool,
    #[serde(default)]
    pub system_wide: bool,
    pub triggers: Vec<ShortcutTrigger>,
    pub actions: Vec<ShortcutAction>,
}

fn default_true() -> bool {
    true
}

impl ShortcutRegistrationOwned {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("shortcut id cannot be empty".into());
        }
        if let ShortcutScope::PageBound(pages) = &self.scope {
            for pid in pages {
                if crate::navigation::page_by_id(pid).is_none() {
                    return Err(format!(
                        "shortcut {} references unknown navigation page '{pid}'",
                        self.id
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn into_effective(self, extension_id: &str) -> Result<EffectiveMergedShortcut, String> {
        self.validate()?;
        Ok(EffectiveMergedShortcut {
            id: self.id,
            label: self.label,
            owner: self.owner,
            required_registry_module: self.required_registry_module,
            scope: self.scope,
            visibility: self.visibility,
            priority: self.priority,
            consumes: self.consumes,
            bypass_text_focus: self.bypass_text_focus,
            system_wide: self.system_wide,
            triggers: self.triggers,
            actions: self.actions,
            source_extension_id: Some(extension_id.to_string()),
        })
    }
}
