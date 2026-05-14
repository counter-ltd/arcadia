use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::config_root_dir;

fn chats_dir() -> io::Result<std::path::PathBuf> {
    let mut p = config_root_dir()?;
    p.push("ai-chats");
    std::fs::create_dir_all(&p)?;
    Ok(p)
}

fn session_path(id: &str) -> io::Result<std::path::PathBuf> {
    let mut p = chats_dir()?;
    // Reject ids that would escape the chats dir.
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid session id"));
    }
    p.push(format!("{id}.json"));
    Ok(p)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn new_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn estimate_tokens(text: &str) -> usize {
    // ~1.33 tokens per word — fast offline approximation.
    let words = text.split_whitespace().count();
    words * 4 / 3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub token_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub provider: String,
    pub model_id: String,
    pub messages: Vec<StoredMessage>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub active_rule_ids: Vec<String>,
    #[serde(default)]
    pub active_skill_ids: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl ChatSession {
    pub fn new(provider: &str, model_id: &str) -> Self {
        let now = now_unix();
        ChatSession {
            id: uuid::Uuid::new_v4().to_string(),
            title: "New Chat".to_string(),
            provider: provider.to_string(),
            model_id: model_id.to_string(),
            messages: Vec::new(),
            workspace_id: None,
            active_rule_ids: Vec::new(),
            active_skill_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn push_message(&mut self, role: &str, content: &str) {
        let token_count = estimate_tokens(content);
        if self.title == "New Chat" && role == "user" && !content.is_empty() {
            self.title = content
                .split_whitespace()
                .take(6)
                .collect::<Vec<_>>()
                .join(" ");
        }
        self.messages.push(StoredMessage {
            role: role.to_string(),
            content: content.to_string(),
            token_count,
        });
        self.updated_at = now_unix();
    }

    /// Total estimated token count across all messages (for truncation decisions).
    pub fn total_tokens(&self) -> usize {
        self.messages.iter().map(|m| m.token_count).sum()
    }

    /// Drop oldest user+assistant message pairs from the front until under the token budget.
    /// Preserves at least the last 2 messages regardless of budget.
    pub fn truncate_to_token_budget(&mut self, budget: usize) {
        while self.total_tokens() > budget && self.messages.len() > 2 {
            // Remove user+assistant pair from the front.
            self.messages.remove(0);
            if !self.messages.is_empty() {
                self.messages.remove(0);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub updated_at: u64,
    pub provider: String,
    pub workspace_id: Option<String>,
    /// Last few messages for dashboard preview: (role == "user", truncated content).
    pub last_messages: Vec<(bool, String)>,
}

pub fn save_session(session: &ChatSession) -> io::Result<()> {
    let path = session_path(&session.id)?;
    let json = serde_json::to_string(session).map_err(io::Error::other)?;
    std::fs::write(path, json)
}

pub fn load_session(id: &str) -> io::Result<ChatSession> {
    let path = session_path(id)?;
    let json = std::fs::read_to_string(path)?;
    serde_json::from_str(&json).map_err(io::Error::other)
}

pub fn list_sessions() -> io::Result<Vec<SessionSummary>> {
    let dir = chats_dir()?;
    let mut summaries: Vec<SessionSummary> = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let json = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if let Ok(session) = serde_json::from_str::<ChatSession>(&json) {
            let last_messages = session.messages.iter().rev().take(4)
                .map(|m| (m.role == "user", m.content.chars().take(44).collect::<String>()))
                .collect::<Vec<_>>()
                .into_iter().rev().collect();
            summaries.push(SessionSummary {
                id: session.id,
                title: session.title,
                updated_at: session.updated_at,
                provider: session.provider.clone(),
                workspace_id: session.workspace_id.clone(),
                last_messages,
            });
        }
    }
    // Most recent first.
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(summaries)
}

pub fn delete_session(id: &str) -> io::Result<()> {
    let path = session_path(id)?;
    std::fs::remove_file(path)
}
