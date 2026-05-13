use crate::modules::ai_types::AiWorkspaceContext;

/// Extract `@path` file mentions from user message text.
pub fn parse_file_mentions(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|tok| tok.starts_with('@'))
        .map(|tok| tok.trim_start_matches('@').to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Read files referenced in `mentions`, gated by workspace permissions.
/// Returns a context block to prepend to the system prompt, or an error string.
pub fn build_file_context(
    mentions: &[String],
    workspace: &AiWorkspaceContext,
) -> Result<String, String> {
    if !workspace.can_read() {
        return Err("workspace.read not granted for this workspace".to_string());
    }
    let mut out = String::new();
    for mention in mentions {
        let path = if std::path::Path::new(mention).is_absolute() {
            mention.clone()
        } else {
            format!("{}/{}", workspace.workspace_path.trim_end_matches('/'), mention)
        };
        if !workspace.is_path_in_scope(&path) {
            return Err(format!("Path out of workspace scope: {path}"));
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Read {path}: {e}"))?;
        out.push_str(&format!("--- @{mention} ---\n{content}\n"));
    }
    Ok(out)
}
