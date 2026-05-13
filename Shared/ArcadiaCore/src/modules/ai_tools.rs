use crate::modules::ai_sandbox;
use crate::modules::ai_types::{AiToolCall, AiToolDefinition, AiToolResult, AiWorkspaceContext};

pub const TOOL_READ_FILE: AiToolDefinition = AiToolDefinition {
    name: "read_file",
    description: "Read the content of a file in the active workspace.",
    parameters: r#"{"type":"object","properties":{"path":{"type":"string","description":"File path relative to workspace root or absolute."}},"required":["path"]}"#,
};

pub const TOOL_WRITE_FILE: AiToolDefinition = AiToolDefinition {
    name: "write_file",
    description: "Write content to a file in the active workspace. Creates or overwrites.",
    parameters: r#"{"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}"#,
};

pub const TOOL_LIST_FILES: AiToolDefinition = AiToolDefinition {
    name: "list_files",
    description: "List files and directories within the active workspace.",
    parameters: r#"{"type":"object","properties":{"path":{"type":"string","description":"Directory path relative to workspace root. Defaults to workspace root if omitted."}}}"#,
};

pub const TOOL_RUN_COMMAND: AiToolDefinition = AiToolDefinition {
    name: "run_command",
    description: "Run a shell command scoped to the workspace root. Requires workspace.execute permission.",
    parameters: r#"{"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}"#,
};

/// All workspace-aware tools. Populated into requests when a workspace is active.
pub const WORKSPACE_TOOLS: &[AiToolDefinition] =
    &[TOOL_READ_FILE, TOOL_WRITE_FILE, TOOL_LIST_FILES, TOOL_RUN_COMMAND];

/// Dispatch a parsed tool call through the sandbox layer.
/// When `stage_writes` is true, `write_file` reads the original file and returns
/// a STAGED result instead of writing to disk. The caller (ai_runtime) detects the
/// "STAGED\n" prefix and emits a `RuntimeEvent::PendingEdit`.
pub fn execute_tool(
    call: &AiToolCall,
    workspace: Option<&AiWorkspaceContext>,
    stage_writes: bool,
) -> AiToolResult {
    let result: Result<String, String> = match call.name.as_str() {
        "read_file" => {
            match call.arguments.get("path").and_then(|v| v.as_str()).filter(|p| !p.is_empty()) {
                None => Err("Missing or empty 'path' argument".to_string()),
                Some(path) => {
                    let full = resolve_path(path, workspace);
                    ai_sandbox::sandboxed_read(workspace, &full)
                }
            }
        }
        "write_file" => {
            let path = call.arguments.get("path").and_then(|v| v.as_str()).filter(|p| !p.is_empty());
            let content = call.arguments.get("content").and_then(|v| v.as_str());
            match (path, content) {
                (None, _) => Err("Missing or empty 'path' argument".to_string()),
                (_, None) => Err("Missing 'content' argument".to_string()),
                (Some(path), Some(proposed)) => {
                    let full = resolve_path(path, workspace);
                    if stage_writes {
                        // Read original (empty string if file doesn't exist yet).
                        let original = ai_sandbox::sandboxed_read(workspace, &full)
                            .unwrap_or_default();
                        Ok(format!("STAGED\n{original}\n---\n{proposed}"))
                    } else {
                        ai_sandbox::sandboxed_write(workspace, &full, proposed).map(|_| "ok".to_string())
                    }
                }
            }
        }
        "list_files" => {
            let full = match call.arguments.get("path").and_then(|v| v.as_str()).filter(|p| !p.is_empty()) {
                Some(path) => resolve_path(path, workspace),
                None => workspace.map(|w| w.workspace_path.as_str()).unwrap_or(".").to_string(),
            };
            ai_sandbox::sandboxed_list(workspace, &full).map(|entries| entries.join("\n"))
        }
        "run_command" => {
            match call.arguments.get("command").and_then(|v| v.as_str()).filter(|c| !c.is_empty()) {
                None => Err("Missing or empty 'command' argument".to_string()),
                Some(cmd) => ai_sandbox::sandboxed_exec(workspace, cmd),
            }
        }
        name => Err(format!("Unknown tool: {name}")),
    };
    match result {
        Ok(output) => AiToolResult { name: call.name.clone(), output, is_error: false },
        Err(e) => AiToolResult { name: call.name.clone(), output: e, is_error: true },
    }
}

/// Parse tool calls from model text output.
/// Expects a ```json block containing: `{"tool_calls":[{"name":"...","arguments":{...}}]}`
pub fn parse_tool_calls(text: &str) -> Vec<AiToolCall> {
    let Some(start) = text.find("```json") else {
        return vec![];
    };
    let body = &text[start + 7..];
    let Some(end) = body.find("```") else {
        return vec![];
    };
    let json_str = body[..end].trim();
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return vec![];
    };
    val["tool_calls"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|call| {
                    Some(AiToolCall {
                        name: call["name"].as_str()?.to_string(),
                        arguments: call["arguments"].clone(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Render tool definitions as an OpenAI-style JSON array string for system prompt injection.
pub fn render_tool_definitions(tools: &[AiToolDefinition]) -> String {
    let entries: Vec<String> = tools
        .iter()
        .map(|t| {
            format!(
                r#"{{"name":"{}","description":"{}","parameters":{}}}"#,
                t.name, t.description, t.parameters
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}

fn resolve_path(path: &str, workspace: Option<&AiWorkspaceContext>) -> String {
    if std::path::Path::new(path).is_absolute() {
        return path.to_string();
    }
    workspace
        .map(|w| format!("{}/{}", w.workspace_path.trim_end_matches('/'), path))
        .unwrap_or_else(|| path.to_string())
}
