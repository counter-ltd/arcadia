use serde::{Deserialize, Serialize};

use crate::config::ConfigFile;

const FILE_NAME: &str = "ai-skills.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSkill {
    pub id: String,
    pub name: String,
    pub system_fragment: String,
    /// Empty = all tools permitted.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub max_tokens_override: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSkillsConfig {
    #[serde(default)]
    pub active_skill_ids: Vec<String>,
    #[serde(default)]
    pub skills: Vec<AiSkill>,
}

impl Default for AiSkillsConfig {
    fn default() -> Self {
        Self {
            active_skill_ids: Vec::new(),
            skills: Vec::new(),
        }
    }
}

impl ConfigFile for AiSkillsConfig {
    fn file_name() -> &'static str {
        FILE_NAME
    }
}

pub fn builtin_skills() -> Vec<AiSkill> {
    vec![
        AiSkill {
            id: "code-reviewer".to_string(),
            name: "Code Reviewer".to_string(),
            system_fragment: "You are acting as a code reviewer. For each file or snippet shown, identify bugs, security issues, performance problems, and style violations. Be specific: cite line numbers and explain the risk. Propose concrete fixes.".to_string(),
            allowed_tools: vec!["read_file".to_string(), "list_files".to_string()],
            max_tokens_override: None,
        },
        AiSkill {
            id: "shell-assistant".to_string(),
            name: "Shell Assistant".to_string(),
            system_fragment: "You are a shell command specialist. Before running any command, explain what it does in plain English. Prefer safe, reversible commands. Warn clearly about destructive operations.".to_string(),
            allowed_tools: Vec::new(),
            max_tokens_override: None,
        },
        AiSkill {
            id: "summariser".to_string(),
            name: "Summariser".to_string(),
            system_fragment: "Your job is to compress and summarise. Output bullet points. Strip filler. Lead with the most important point. Keep total output under 200 words unless explicitly asked for more.".to_string(),
            allowed_tools: vec!["read_file".to_string()],
            max_tokens_override: Some(512),
        },
        AiSkill {
            id: "security-auditor".to_string(),
            name: "Security Auditor".to_string(),
            system_fragment: "You are a security auditor. Focus exclusively on security vulnerabilities: injection flaws, authentication bypasses, insecure defaults, exposed secrets, privilege escalation paths, and OWASP Top 10. Classify each finding by severity (Critical / High / Medium / Low). Ignore non-security issues.".to_string(),
            allowed_tools: vec!["read_file".to_string(), "list_files".to_string()],
            max_tokens_override: None,
        },
    ]
}

pub fn all_skills(cfg: &AiSkillsConfig) -> Vec<AiSkill> {
    let mut skills = builtin_skills();
    skills.extend(cfg.skills.iter().cloned());
    skills
}
