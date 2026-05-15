//! Extension (and shared) UI token specs: kinds, declarative row visibility.
//!
//! Visibility is evaluated only in settings UI from **current** effective values
//! (defaults + TOML + in-memory edits). Extensions/modules declare conditions next
//! to each token — no host hardcoding per extension.

use std::collections::HashMap;

/// Declared UI/config token (persisted under `extension_tokens/` when used by an extension).
#[derive(Clone, Debug, PartialEq)]
pub struct StyleTokenSpec {
    pub key: String,
    pub label: String,
    pub kind: StyleTokenKind,
    /// Serialized default (same representation saved in TOML).
    pub default_value: String,
    /// When `None`, row always shown. Otherwise all [`StyleTokenVisibility`] rules must hold.
    pub visibility: Option<StyleTokenVisibility>,
    /// Optional int/float slider bounds. `None` = use built-in defaults + heuristic max at UI time.
    pub numeric: Option<StyleTokenNumericBounds>,
    /// When non-empty, the UI renders a segmented chip selector instead of a free-text input.
    pub options: Vec<String>,
}

/// Step / decimal policy for float sliders; int tokens always use whole steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleTokenNumericGranularity {
    Whole,
    Tenth,
    Thousandth,
}

impl StyleTokenNumericGranularity {
    pub fn step_f64(self) -> f64 {
        match self {
            StyleTokenNumericGranularity::Whole => 1.0,
            StyleTokenNumericGranularity::Tenth => 0.1,
            StyleTokenNumericGranularity::Thousandth => 0.001,
        }
    }

    pub fn float_decimal_places(self) -> u32 {
        match self {
            StyleTokenNumericGranularity::Whole => 0,
            StyleTokenNumericGranularity::Tenth => 1,
            StyleTokenNumericGranularity::Thousandth => 3,
        }
    }
}

/// Merged numeric bounds for int/float tokens (all fields set).
#[derive(Clone, Debug, PartialEq)]
pub struct StyleTokenNumericBounds {
    pub min: String,
    pub max: String,
    pub granularity: StyleTokenNumericGranularity,
}

/// Optional fields from Python `register_tokens` before merge with defaults.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleTokenNumericPartial {
    pub min: Option<String>,
    pub max: Option<String>,
    pub granularity: Option<StyleTokenNumericGranularity>,
}

impl StyleTokenNumericPartial {
    pub fn is_empty(&self) -> bool {
        self.min.is_none() && self.max.is_none() && self.granularity.is_none()
    }
}

/// Heuristic int slider upper bound (legacy UI behavior).
pub fn heuristic_int_hi(default_i: i64, cur_i: i64) -> i64 {
    let m = default_i.max(cur_i).max(1);
    if m <= 15 {
        (m * 8).max(20).min(100)
    } else {
        m.saturating_mul(15).max(m.saturating_add(1000)).max(100)
    }
}

/// Heuristic float slider `(lo, hi)` (legacy UI behavior).
pub fn heuristic_float_lo_hi(default_f: f64, cur_f: f64) -> (f64, f64) {
    let v = default_f.max(cur_f);
    let lo = 0_f64;
    let hi = if v <= 1.0 {
        2.0_f64
    } else if v <= 24.0 {
        (v * 2.5 + 8.0).max(v + 1.0)
    } else {
        v * 2.0 + 32.0
    };
    (lo, hi)
}

pub fn parse_granularity_str(s: &str) -> Result<StyleTokenNumericGranularity, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "whole" => Ok(StyleTokenNumericGranularity::Whole),
        "tenth" => Ok(StyleTokenNumericGranularity::Tenth),
        "thousandth" | "thousandths" => Ok(StyleTokenNumericGranularity::Thousandth),
        other => Err(format!("unknown granularity: {other}")),
    }
}

/// Merge optional Python fields with defaults; `Ok(None)` if partial is empty (caller stores `numeric: None`).
pub fn merge_token_numeric_bounds(
    kind: StyleTokenKind,
    default_value: &str,
    partial: StyleTokenNumericPartial,
) -> Result<Option<StyleTokenNumericBounds>, String> {
    if partial.is_empty() {
        return Ok(None);
    }
    if !matches!(kind, StyleTokenKind::Int | StyleTokenKind::Float) {
        return Err("min/max/granularity only apply to int or float tokens".into());
    }

    let default_trim = default_value.trim();
    let mut gran = partial
        .granularity
        .unwrap_or(if matches!(kind, StyleTokenKind::Int) {
            StyleTokenNumericGranularity::Whole
        } else {
            StyleTokenNumericGranularity::Tenth
        });
    if matches!(kind, StyleTokenKind::Int)
        && matches!(
            gran,
            StyleTokenNumericGranularity::Tenth | StyleTokenNumericGranularity::Thousandth
        )
    {
        gran = StyleTokenNumericGranularity::Whole;
    }

    let (min_s, max_s) = match kind {
        StyleTokenKind::Int => {
            let d = default_trim
                .parse::<i64>()
                .map_err(|_| format!("invalid int default: {default_trim}"))?;
            let min_i = if let Some(ref m) = partial.min {
                m.trim()
                    .parse::<i64>()
                    .map_err(|_| format!("invalid int min: {m}"))?
            } else {
                0_i64
            };
            let max_i = if let Some(ref m) = partial.max {
                m.trim()
                    .parse::<i64>()
                    .map_err(|_| format!("invalid int max: {m}"))?
            } else {
                heuristic_int_hi(d, d)
            };
            if min_i > max_i {
                return Err(format!("min ({min_i}) must be <= max ({max_i})"));
            }
            if d < min_i || d > max_i {
                return Err(format!(
                    "default int {d} must lie within [{min_i}, {max_i}]"
                ));
            }
            (min_i.to_string(), max_i.to_string())
        }
        StyleTokenKind::Float => {
            let d = default_trim
                .parse::<f64>()
                .map_err(|_| format!("invalid float default: {default_trim}"))?;
            let min_f = if let Some(ref m) = partial.min {
                m.trim()
                    .parse::<f64>()
                    .map_err(|_| format!("invalid float min: {m}"))?
            } else {
                0_f64
            };
            let max_f = if let Some(ref m) = partial.max {
                m.trim()
                    .parse::<f64>()
                    .map_err(|_| format!("invalid float max: {m}"))?
            } else {
                heuristic_float_lo_hi(d, d).1
            };
            if min_f > max_f || !min_f.is_finite() || !max_f.is_finite() {
                return Err(format!("invalid float range [{min_f}, {max_f}]"));
            }
            if d < min_f || d > max_f {
                return Err(format!(
                    "default float {d} must lie within [{min_f}, {max_f}]"
                ));
            }
            (storage_float(min_f), storage_float(max_f))
        }
        _ => unreachable!(),
    };

    Ok(Some(StyleTokenNumericBounds {
        min: min_s,
        max: max_s,
        granularity: gran,
    }))
}

fn storage_float(v: f64) -> String {
    if !v.is_finite() {
        return "0".into();
    }
    if (v - v.round()).abs() < 1e-9 {
        return format!("{}", v.round() as i64);
    }
    let mut s = format!("{v:.6}");
    while s.contains('.') && (s.ends_with('0') || s.ends_with('.')) {
        s.pop();
    }
    s
}

/// Resolved track for int/float sliders at UI time.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSliderNumeric {
    pub lo: f64,
    pub hi: f64,
    pub step: f64,
    pub float_decimal_places: u32,
    pub granularity: StyleTokenNumericGranularity,
}

/// Slider geometry + snapping for int/float tokens.
pub fn resolve_slider_numeric(
    spec: &StyleTokenSpec,
    current_display: &str,
) -> Option<ResolvedSliderNumeric> {
    let kind = match spec.kind {
        StyleTokenKind::Int | StyleTokenKind::Float => spec.kind,
        _ => return None,
    };
    let d = parse_numeric_default(kind, &spec.default_value)?;
    let c = parse_numeric_current(kind, current_display).unwrap_or(d);
    let (lo, hi, gran) = match (&spec.numeric, kind) {
        (None, StyleTokenKind::Int) => {
            let lo = 0_f64;
            let hi = heuristic_int_hi(d as i64, c as i64) as f64;
            let gran = StyleTokenNumericGranularity::Whole;
            (lo, hi, gran)
        }
        (None, StyleTokenKind::Float) => {
            let (lo, hi) = heuristic_float_lo_hi(d, c);
            let gran = StyleTokenNumericGranularity::Tenth;
            (lo, hi, gran)
        }
        (Some(b), StyleTokenKind::Int) => {
            let lo = b.min.trim().parse::<f64>().ok()?;
            let hi = b.max.trim().parse::<f64>().ok()?;
            let gran = StyleTokenNumericGranularity::Whole;
            (lo, hi, gran)
        }
        (Some(b), StyleTokenKind::Float) => {
            let lo = b.min.trim().parse::<f64>().ok()?;
            let hi = b.max.trim().parse::<f64>().ok()?;
            (lo, hi, b.granularity)
        }
        _ => return None,
    };
    let hi = hi.max(lo + 1e-9);
    let step = match kind {
        StyleTokenKind::Int => 1.0,
        StyleTokenKind::Float => gran.step_f64(),
        _ => 1.0,
    };
    let float_decimal_places = match kind {
        StyleTokenKind::Float => gran.float_decimal_places(),
        _ => 0,
    };
    Some(ResolvedSliderNumeric {
        lo,
        hi,
        step,
        float_decimal_places,
        granularity: gran,
    })
}

fn parse_numeric_default(kind: StyleTokenKind, s: &str) -> Option<f64> {
    match kind {
        StyleTokenKind::Int => s.trim().parse::<i64>().ok().map(|i| i as f64),
        StyleTokenKind::Float => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn parse_numeric_current(kind: StyleTokenKind, s: &str) -> Option<f64> {
    parse_numeric_default(kind, s)
}

/// `(lo, hi)` for clamping a numeric value on save (same policy as slider track).
pub fn numeric_clamp_lo_hi(
    spec: &StyleTokenSpec,
    value_for_heuristic_max: f64,
) -> Option<(f64, f64)> {
    let kind = match spec.kind {
        StyleTokenKind::Int | StyleTokenKind::Float => spec.kind,
        _ => return None,
    };
    let d = parse_numeric_default(kind, &spec.default_value)?;
    let v = if value_for_heuristic_max.is_finite() {
        value_for_heuristic_max
    } else {
        d
    };
    let (lo, hi) = match &spec.numeric {
        None => match kind {
            StyleTokenKind::Int => (0_f64, heuristic_int_hi(d as i64, v as i64) as f64),
            StyleTokenKind::Float => heuristic_float_lo_hi(d, v),
            _ => return None,
        },
        Some(b) => {
            let lo = b.min.trim().parse::<f64>().ok()?;
            let hi = b.max.trim().parse::<f64>().ok()?;
            (lo, hi)
        }
    };
    Some((lo, hi.max(lo + 1e-9)))
}

/// Snap raw slider position then format for token display.
pub fn format_slider_value(
    v: f64,
    kind: StyleTokenKind,
    gran: StyleTokenNumericGranularity,
) -> String {
    if !v.is_finite() {
        return match kind {
            StyleTokenKind::Int => "0".into(),
            StyleTokenKind::Float => "0".into(),
            _ => "0".into(),
        };
    }
    match kind {
        StyleTokenKind::Int => {
            let i = v.round().clamp(i64::MIN as f64, i64::MAX as f64) as i64;
            i.to_string()
        }
        StyleTokenKind::Float => {
            let places = gran.float_decimal_places();
            let mut s = if places == 0 {
                format!("{:.0}", v)
            } else if places == 1 {
                format!("{:.1}", v)
            } else {
                format!("{:.3}", v)
            };
            if places > 0 {
                while s.contains('.') && (s.ends_with('0') || s.ends_with('.')) {
                    s.pop();
                }
            }
            s
        }
        _ => v.to_string(),
    }
}

pub fn snap_slider_value(v: f64, lo: f64, hi: f64, step: f64, kind: StyleTokenKind) -> f64 {
    let v = v.clamp(lo, hi);
    if !step.is_finite() || step <= 0.0 {
        return v;
    }
    let snapped = if matches!(kind, StyleTokenKind::Int) {
        v.round()
    } else {
        ((v - lo) / step).round() * step + lo
    };
    snapped.clamp(lo, hi)
}

/// Clamp a display string for int/float to slider bounds after parse.
pub fn clamp_numeric_display_for_spec(spec: &StyleTokenSpec, display: &str) -> Option<String> {
    let kind = match spec.kind {
        StyleTokenKind::Int | StyleTokenKind::Float => spec.kind,
        _ => return None,
    };
    let parsed = parse_numeric_current(kind, display)?;
    let (lo, hi) = numeric_clamp_lo_hi(spec, parsed)?;
    let gran = match (&spec.numeric, kind) {
        (Some(b), StyleTokenKind::Float) => b.granularity,
        (_, StyleTokenKind::Float) => StyleTokenNumericGranularity::Tenth,
        _ => StyleTokenNumericGranularity::Whole,
    };
    let step = match kind {
        StyleTokenKind::Int => 1.0,
        StyleTokenKind::Float => gran.step_f64(),
        _ => 1.0,
    };
    let snapped = snap_slider_value(parsed, lo, hi, step, kind);
    Some(format_slider_value(snapped, kind, gran))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleTokenKind {
    Color,
    Float,
    String,
    Bool,
    Int,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleTokenCompareOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

/// Declarative visibility for one token row. Serialized from Python `visible_when=…`.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleTokenVisibility {
    /// Every child must be true.
    All(Vec<StyleTokenVisibility>),
    /// At least one child must be true.
    Any(Vec<StyleTokenVisibility>),
    /// Compare another token's effective display string to a literal (typed per that token's `kind`).
    Compare {
        token: String,
        op: StyleTokenCompareOp,
        literal: String,
    },
}

/// Effective display string for `key` in this module (`memory` wins, else spec default).
pub fn effective_token_display(
    module: &str,
    key: &str,
    specs: &[StyleTokenSpec],
    memory: &HashMap<(String, String), String>,
) -> String {
    let pair = (module.to_string(), key.to_string());
    memory.get(&pair).cloned().unwrap_or_else(|| {
        specs
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.default_value.clone())
            .unwrap_or_default()
    })
}

fn kind_for_token(specs: &[StyleTokenSpec], key: &str) -> Option<StyleTokenKind> {
    specs.iter().find(|s| s.key == key).map(|s| s.kind)
}

fn parse_bool_loose(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" | "" => Some(false),
        _ => None,
    }
}

fn compare_values(
    kind: StyleTokenKind,
    current: &str,
    literal: &str,
    op: StyleTokenCompareOp,
) -> bool {
    match kind {
        StyleTokenKind::Int => {
            let Ok(a) = current.trim().parse::<i64>() else {
                return false;
            };
            let Ok(b) = literal.trim().parse::<i64>() else {
                return false;
            };
            match op {
                StyleTokenCompareOp::Eq => a == b,
                StyleTokenCompareOp::Ne => a != b,
                StyleTokenCompareOp::Gt => a > b,
                StyleTokenCompareOp::Ge => a >= b,
                StyleTokenCompareOp::Lt => a < b,
                StyleTokenCompareOp::Le => a <= b,
            }
        }
        StyleTokenKind::Float => {
            let Ok(a) = current.trim().parse::<f64>() else {
                return false;
            };
            let Ok(b) = literal.trim().parse::<f64>() else {
                return false;
            };
            match op {
                StyleTokenCompareOp::Eq => a == b,
                StyleTokenCompareOp::Ne => a != b,
                StyleTokenCompareOp::Gt => a > b,
                StyleTokenCompareOp::Ge => a >= b,
                StyleTokenCompareOp::Lt => a < b,
                StyleTokenCompareOp::Le => a <= b,
            }
        }
        StyleTokenKind::Bool => {
            let Some(a) = parse_bool_loose(current) else {
                return false;
            };
            let Some(b) = parse_bool_loose(literal) else {
                return false;
            };
            match op {
                StyleTokenCompareOp::Eq => a == b,
                StyleTokenCompareOp::Ne => a != b,
                _ => false,
            }
        }
        StyleTokenKind::String | StyleTokenKind::Color => match op {
            StyleTokenCompareOp::Eq => current.trim() == literal.trim(),
            StyleTokenCompareOp::Ne => current.trim() != literal.trim(),
            _ => false,
        },
    }
}

/// Whether a token settings row should render for the given module state.
pub fn style_token_row_visible(
    visibility: &Option<StyleTokenVisibility>,
    module: &str,
    specs: &[StyleTokenSpec],
    memory: &HashMap<(String, String), String>,
) -> bool {
    let Some(v) = visibility else {
        return true;
    };
    eval_visibility(v, module, specs, memory)
}

fn eval_visibility(
    v: &StyleTokenVisibility,
    module: &str,
    specs: &[StyleTokenSpec],
    memory: &HashMap<(String, String), String>,
) -> bool {
    match v {
        StyleTokenVisibility::All(children) => {
            if children.is_empty() {
                return true;
            }
            children
                .iter()
                .all(|c| eval_visibility(c, module, specs, memory))
        }
        StyleTokenVisibility::Any(children) => {
            if children.is_empty() {
                return false;
            }
            children
                .iter()
                .any(|c| eval_visibility(c, module, specs, memory))
        }
        StyleTokenVisibility::Compare { token, op, literal } => {
            let Some(kind) = kind_for_token(specs, token.as_str()) else {
                return false;
            };
            let cur = effective_token_display(module, token.as_str(), specs, memory);
            compare_values(kind, &cur, literal, *op)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec_int(key: &str, default: i32) -> StyleTokenSpec {
        StyleTokenSpec {
            key: key.to_string(),
            label: key.to_string(),
            kind: StyleTokenKind::Int,
            default_value: default.to_string(),
            visibility: None,
            numeric: None,
            options: Vec::new(),
        }
    }

    fn mem(m: &str, k: &str, v: &str) -> HashMap<(String, String), String> {
        let mut h = HashMap::new();
        h.insert((m.to_string(), k.to_string()), v.to_string());
        h
    }

    #[test]
    fn gte_hides_when_eye_count_low() {
        let specs = vec![
            spec_int("eye_count", 2),
            StyleTokenSpec {
                key: "relative_blink_on_click".into(),
                label: "rel".into(),
                kind: StyleTokenKind::Bool,
                default_value: "false".into(),
                visibility: Some(StyleTokenVisibility::Compare {
                    token: "eye_count".into(),
                    op: StyleTokenCompareOp::Ge,
                    literal: "2".into(),
                }),
                numeric: None,
                options: Vec::new(),
            },
        ];
        let module = "googly-eyes";
        let v = &specs[1].visibility;
        assert!(style_token_row_visible(
            v,
            module,
            &specs,
            &mem(module, "eye_count", "2")
        ));
        assert!(style_token_row_visible(
            v,
            module,
            &specs,
            &mem(module, "eye_count", "3")
        ));
        assert!(!style_token_row_visible(
            v,
            module,
            &specs,
            &mem(module, "eye_count", "1")
        ));
        assert!(style_token_row_visible(v, module, &specs, &HashMap::new()));
    }

    #[test]
    fn all_requires_both() {
        let specs = vec![
            spec_int("eye_count", 2),
            StyleTokenSpec {
                key: "x".into(),
                label: "x".into(),
                kind: StyleTokenKind::Bool,
                default_value: "false".into(),
                visibility: Some(StyleTokenVisibility::All(vec![
                    StyleTokenVisibility::Compare {
                        token: "eye_count".into(),
                        op: StyleTokenCompareOp::Ge,
                        literal: "2".into(),
                    },
                    StyleTokenVisibility::Compare {
                        token: "eye_count".into(),
                        op: StyleTokenCompareOp::Le,
                        literal: "3".into(),
                    },
                ])),
                numeric: None,
                options: Vec::new(),
            },
        ];
        let module = "m";
        let vis = &specs[1].visibility;
        let h = mem(module, "eye_count", "2");
        assert!(style_token_row_visible(vis, module, &specs, &h));
        let h2 = mem(module, "eye_count", "4");
        assert!(!style_token_row_visible(vis, module, &specs, &h2));
    }

    #[test]
    fn merge_partial_fills_max_for_int() {
        let p = StyleTokenNumericPartial {
            min: Some("1".into()),
            max: None,
            granularity: None,
        };
        let out = merge_token_numeric_bounds(StyleTokenKind::Int, "2", p)
            .unwrap()
            .unwrap();
        assert_eq!(out.min, "1");
        assert!(out.max.parse::<i64>().unwrap() >= 2);
        assert_eq!(out.granularity, StyleTokenNumericGranularity::Whole);
    }

    #[test]
    fn merge_empty_returns_none() {
        assert!(merge_token_numeric_bounds(
            StyleTokenKind::Int,
            "2",
            StyleTokenNumericPartial::default()
        )
        .unwrap()
        .is_none());
    }

    #[test]
    fn resolve_none_int_matches_heuristic() {
        let spec = spec_int("n", 2);
        let r = resolve_slider_numeric(&spec, "2").unwrap();
        assert!((r.lo - 0.).abs() < 1e-6);
        assert!((r.hi - heuristic_int_hi(2, 2) as f64).abs() < 1e-6);
        assert_eq!(r.step, 1.0);
        assert_eq!(r.granularity, StyleTokenNumericGranularity::Whole);
    }

    #[test]
    fn int_granularity_tenth_coerces_to_whole_in_merge() {
        let p = StyleTokenNumericPartial {
            min: Some("0".into()),
            max: Some("10".into()),
            granularity: Some(StyleTokenNumericGranularity::Thousandth),
        };
        let out = merge_token_numeric_bounds(StyleTokenKind::Int, "5", p)
            .unwrap()
            .unwrap();
        assert_eq!(out.granularity, StyleTokenNumericGranularity::Whole);
    }

    #[test]
    fn snap_float_tenth() {
        let v = snap_slider_value(0.37, 0., 1., 0.1, StyleTokenKind::Float);
        assert!((v - 0.4).abs() < 1e-6 || (v - 0.4).abs() < 0.05);
    }
}
