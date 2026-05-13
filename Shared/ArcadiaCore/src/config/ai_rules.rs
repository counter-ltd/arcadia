use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const FILE_NAME: &str = "ai-rules.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRule {
    pub id: String,
    pub name: String,
    pub system_fragment: String,
    #[serde(default)]
    pub forbidden_tools: Vec<String>,
    #[serde(default)]
    pub max_tokens_override: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRulesConfig {
    #[serde(default)]
    pub active_rule_ids: Vec<String>,
    #[serde(default)]
    pub custom_rules: Vec<AiRule>,
}

impl Default for AiRulesConfig {
    fn default() -> Self {
        Self {
            active_rule_ids: Vec::new(),
            custom_rules: Vec::new(),
        }
    }
}

impl ConfigFile for AiRulesConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }
}

pub fn builtin_rules() -> Vec<AiRule> {
    vec![
        AiRule {
            id: "safe-exec".to_string(),
            name: "Safe Exec".to_string(),
            system_fragment: "Before any run_command call, briefly explain what the command does and confirm it is safe to run.".to_string(),
            forbidden_tools: Vec::new(),
            max_tokens_override: None,
        },
        AiRule {
            id: "concise-output".to_string(),
            name: "Concise Output".to_string(),
            system_fragment: "Prefer short, direct responses. Omit preamble, filler, and unnecessary explanation.".to_string(),
            forbidden_tools: Vec::new(),
            max_tokens_override: Some(1024),
        },
        AiRule {
            id: "diff-over-full-file".to_string(),
            name: "Diff Over Full File".to_string(),
            system_fragment: "When editing files, emit only the changed portion using a diff or snippet — do not rewrite the entire file.".to_string(),
            forbidden_tools: Vec::new(),
            max_tokens_override: None,
        },
        AiRule {
            id: "no-exec".to_string(),
            name: "No Exec".to_string(),
            system_fragment: "You may not run shell commands in this session. Use read_file, list_files, and write_file only.".to_string(),
            forbidden_tools: vec!["run_command".to_string()],
            max_tokens_override: None,
        },
    ]
}

pub fn all_rules(cfg: &AiRulesConfig) -> Vec<AiRule> {
    let mut rules = builtin_rules();
    rules.extend(cfg.custom_rules.iter().cloned());
    rules
}
