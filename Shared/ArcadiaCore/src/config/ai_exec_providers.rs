use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

/// A single configured CLI exec provider entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecCliEntry {
    /// Unique identifier (e.g. "claude", "codex", "my-custom-cli").
    pub id: String,
    /// Binary name or absolute path. Must be in EXEC_ALLOWLIST.
    pub binary: String,
    /// Display label shown in provider selector.
    pub label: String,
    /// Flag used to select a specific model (e.g. `"--model"`). None = use CLI default.
    #[serde(default)]
    pub model_flag: Option<String>,
    /// Model value passed after model_flag (e.g. `"claude-opus-4-7"`). Empty = CLI default.
    #[serde(default)]
    pub model_value: String,
    /// Static extra args prepended before the prompt (e.g. `["--print"]` for claude).
    /// These are literal flag strings — never user-supplied content.
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// Whether this entry is active in the provider selector.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

/// Persisted config for CLI exec providers.
/// Auto-detected providers are written here on first detection so the user
/// can adjust them (model_flag, model_value, extra_args) without editing code.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AiExecProvidersConfig {
    #[serde(default)]
    pub entries: Vec<ExecCliEntry>,
}

impl ConfigFile for AiExecProvidersConfig {
    fn file_name() -> &'static str { "ai-exec-providers.toml" }
}
