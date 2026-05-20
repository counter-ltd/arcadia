use crate::config::workspace::WorkspaceEntry;

/// Provider-agnostic model kind — used by all providers (llama.cpp, Ollama, OpenAI, …).
#[derive(Clone, Debug, PartialEq)]
pub enum AiModelKind {
    TextGeneration,
    ImageGeneration,
    Vision,
    Embedding,
}

// ── Workspace context ─────────────────────────────────────────────────────────

/// Carries the active workspace scope into AI inference requests.
/// Context-only — no sandboxing enforced yet. Inference thread may use
/// can_read() / can_write() as enforcement gates in a follow-up.
#[derive(Clone, Debug)]
pub struct AiWorkspaceContext {
    pub workspace_id: String,
    pub workspace_label: String,
    pub workspace_path: String,
    pub granted_permissions: Vec<String>,
}

impl AiWorkspaceContext {
    pub fn from_workspace_entry(entry: &WorkspaceEntry) -> Self {
        Self {
            workspace_id: entry.id.clone(),
            workspace_label: entry.label.clone(),
            workspace_path: entry.path.clone(),
            granted_permissions: entry.granted_permissions.clone(),
        }
    }

    pub fn can_read(&self) -> bool {
        self.granted_permissions
            .iter()
            .any(|p| p == "workspace.ai_read")
    }

    pub fn can_write(&self) -> bool {
        self.granted_permissions
            .iter()
            .any(|p| p == "workspace.ai_write")
    }

    pub fn can_execute(&self) -> bool {
        self.granted_permissions
            .iter()
            .any(|p| p == "workspace.ai_execute")
    }

    pub fn is_path_in_scope(&self, path: &str) -> bool {
        let Ok(canon_ws) = std::fs::canonicalize(&self.workspace_path) else {
            return false;
        };
        // Canonicalize the target path. If it doesn't exist yet (new file write),
        // canonicalize its parent directory instead so traversal attacks still fail.
        let canon = if let Ok(c) = std::fs::canonicalize(path) {
            c
        } else {
            let p = std::path::Path::new(path);
            let parent = p.parent().unwrap_or(p);
            let Ok(cp) = std::fs::canonicalize(parent) else {
                return false;
            };
            cp.join(p.file_name().unwrap_or_default())
        };
        canon.starts_with(&canon_ws)
    }
}

// ── Tool types ────────────────────────────────────────────────────────────────

/// Definition injected into the system prompt so the model knows what tools exist.
#[derive(Clone, Debug)]
pub struct AiToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: &'static str,
}

/// A parsed tool call from model output.
#[derive(Clone, Debug)]
pub struct AiToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Result fed back to the model after tool execution.
#[derive(Clone, Debug)]
pub struct AiToolResult {
    pub name: String,
    pub output: String,
    pub is_error: bool,
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct TextGenerationRequest {
    pub system: String,
    pub messages: Vec<(String, String)>,
    pub max_tokens: i32,
    pub workspace_context: Option<AiWorkspaceContext>,
    /// Tool definitions available to the model. Empty = no tool use.
    pub tools: Vec<AiToolDefinition>,
    /// Active rule IDs — merged into system prompt by prepare_system().
    pub active_rule_ids: Vec<String>,
    /// Active skill IDs — merged into system prompt by prepare_system().
    pub active_skill_ids: Vec<String>,
    /// When true, write_file stages edits instead of writing directly to disk.
    pub stage_writes: bool,
}

#[derive(Clone, Debug)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: u32,
    pub height: u32,
    pub workspace_context: Option<AiWorkspaceContext>,
    /// Provider-specific style token (e.g. `"animation"`, `"illustration"`, `"sketch"`
    /// for ImagePlayground). `None` lets the provider pick its default.
    pub style: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EmbeddingRequest {
    pub input: String,
    pub workspace_context: Option<AiWorkspaceContext>,
}

#[derive(Clone, Debug)]
pub struct VisionRequest {
    pub system: String,
    pub messages: Vec<(String, String)>,
    pub image_data: Vec<u8>,
    pub workspace_context: Option<AiWorkspaceContext>,
}

// ── Response types ────────────────────────────────────────────────────────────

/// Single streamed text token.
#[derive(Clone, Debug)]
pub struct TextGenerationToken(pub String);

/// Final output from an image generation request.
#[derive(Clone, Debug)]
pub struct ImageGenerationOutput {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Dense embedding vector.
#[derive(Clone, Debug)]
pub struct EmbeddingOutput {
    pub vector: Vec<f32>,
}
