//! Per-extension persisted token values for styling extensions (`~/Arcadia/Configuration/extension_tokens/`).

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use toml::Value;

use super::config_root_dir;

use crate::modules::python_registry::{GlyphParams, StyleTokenKind};

/// Sanitize extension module id for use as a filename segment.
pub fn sanitize_module_id(module: &str) -> String {
    module
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn tokens_dir() -> io::Result<PathBuf> {
    let mut p = config_root_dir()?;
    p.push("extension_tokens");
    fs::create_dir_all(&p)?;
    Ok(p)
}

pub fn tokens_file_for_module(module: &str) -> io::Result<PathBuf> {
    let mut p = tokens_dir()?;
    let safe = sanitize_module_id(module);
    if safe.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty extension module id",
        ));
    }
    p.push(format!("{safe}.toml"));
    Ok(p)
}

#[derive(Default, Serialize, Deserialize)]
pub struct ExtensionTokensFile {
    #[serde(default)]
    pub tokens: HashMap<String, Value>,
}

pub fn load_module_tokens(module: &str) -> io::Result<HashMap<String, Value>> {
    let path = tokens_file_for_module(module)?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let raw = fs::read_to_string(&path)?;
    let file: ExtensionTokensFile =
        toml::from_str(&raw).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(file.tokens)
}

pub fn save_module_tokens(module: &str, tokens: &HashMap<String, Value>) -> io::Result<()> {
    tokens_dir()?;
    let path = tokens_file_for_module(module)?;
    let file = ExtensionTokensFile {
        tokens: tokens.clone(),
    };
    let body =
        toml::to_string_pretty(&file).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, body)
}

pub fn merged_display_for_key(
    key: &str,
    default: &str,
    file: &HashMap<String, Value>,
) -> String {
    if let Some(v) = file.get(key) {
        value_to_display_string(v)
    } else {
        default.to_string()
    }
}

pub fn parse_input_to_value(kind: StyleTokenKind, input: &str) -> Result<Value, String> {
    let t = input.trim();
    match kind {
        StyleTokenKind::Color | StyleTokenKind::String => Ok(Value::String(t.to_string())),
        StyleTokenKind::Float => t
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| format!("invalid float: {t}")),
        StyleTokenKind::Int => t
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| format!("invalid integer: {t}")),
        StyleTokenKind::Bool => match t.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(Value::Boolean(true)),
            "false" | "0" | "no" => Ok(Value::Boolean(false)),
            "" => Err("empty boolean".into()),
            _ => Err(format!("expected true/false, got {t}")),
        },
    }
}

pub fn value_to_display_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => format!("{f}"),
        Value::Boolean(b) => b.to_string(),
        Value::Datetime(d) => d.to_string(),
        _ => v.to_string(),
    }
}

pub fn value_as_f32(v: &Value) -> Option<f32> {
    match v {
        Value::Float(f) => Some(*f as f32),
        Value::Integer(i) => Some(*i as f32),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn value_as_glyph_color_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

/// Apply saved token file entries onto glyph params. Only keys that map to [`GlyphParams`] are used;
/// other keys are for custom / future use and stay in the file.
pub fn apply_file_tokens_to_glyph(g: &mut GlyphParams, file: &HashMap<String, Value>) {
    for (k, v) in file {
        match k.as_str() {
            "bg" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.bg = Some(s);
                }
            }
            "surface" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.surface = Some(s);
                }
            }
            "surface2" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.surface2 = Some(s);
                }
            }
            "text" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.text = Some(s);
                }
            }
            "dim" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.dim = Some(s);
                }
            }
            "border" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.border = Some(s);
                }
            }
            "accent" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.accent = Some(s);
                }
            }
            "border_chars" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.border_chars = Some(s);
                }
            }
            "border_horizontal_pattern" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.border_horizontal_pattern = Some(s);
                }
            }
            "border_vertical_pattern" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.border_vertical_pattern = Some(s);
                }
            }
            "border_font_family" => {
                if let Some(s) = value_as_glyph_color_string(v) {
                    g.border_font_family = Some(s);
                }
            }
            "border_radius" => {
                if let Some(f) = value_as_f32(v) {
                    g.border_radius = f;
                }
            }
            "border_font_size_rems" => {
                if let Some(f) = value_as_f32(v) {
                    g.border_font_size_rems = Some(f);
                }
            }
            "border_side_rail_px" => {
                if let Some(f) = value_as_f32(v) {
                    g.border_side_rail_px = Some(f);
                }
            }
            _ => {}
        }
    }
}
