use std::process::Command;

use crate::modules::ai_types::TextGenerationRequest;

/// A CLI AI provider detected on PATH at startup.
#[derive(Clone, Debug)]
pub struct DetectedCliProvider {
    pub id: String,
    pub binary: String,
    pub label: String,
    pub version: String,
    /// Flag used to select a model, e.g. `"--model"`.
    pub model_flag: Option<String>,
    /// Static args that must be present for non-interactive use (e.g. `["--print"]` for claude).
    pub extra_args: Vec<String>,
}

/// (module_name, binary_name, display_label, model_flag, extra_args_for_non_interactive_mode)
static KNOWN_CLIS: &[(&str, &str, &str, Option<&str>, &[&str])] = &[
    ("ai-provider-exec-claude", "claude", "Claude (CLI)", Some("--model"), &["--print"]),
    ("ai-provider-exec-codex",  "codex",  "Codex (CLI)",  Some("--model"), &[]),
    ("ai-provider-exec-gemini", "gemini", "Gemini (CLI)", Some("--model"), &[]),
    ("ai-provider-exec-aider",  "aider",  "Aider (CLI)",  Some("--model"), &["--message"]),
];

/// Given a module name (e.g. `"ai-provider-exec-claude"`), return the binary name and model flag.
/// Returns `None` for unknown module names.
pub fn cli_for_module(module_name: &str) -> Option<(&'static str, Option<&'static str>)> {
    KNOWN_CLIS.iter()
        .find(|&&(m, _, _, _, _)| m == module_name)
        .map(|&(_, bin, _, model_flag, _)| (bin, model_flag))
}

/// Scan PATH for known AI CLIs. Each is probed with `--version`; timeout is OS default
/// for process spawn (kept short by the 2-second kill via SIGTERM if needed).
pub fn scan_for_cli_providers() -> Vec<DetectedCliProvider> {
    KNOWN_CLIS.iter().filter_map(|&(module_name, bin, label, model_flag, extra_args)| {
        probe_cli(bin).map(|version| DetectedCliProvider {
            id: module_name.to_string(),
            binary: bin.to_string(),
            label: label.to_string(),
            version,
            model_flag: model_flag.map(str::to_string),
            extra_args: extra_args.iter().map(|s| s.to_string()).collect(),
        })
    }).collect()
}

fn probe_cli(binary: &str) -> Option<String> {
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .ok()?;
    // Accept any binary that either exits 0 or writes something to stdout.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_line = stdout.lines().next()
        .or_else(|| stderr.lines().next())
        .unwrap_or("detected")
        .trim()
        .to_string();
    if output.status.success() || !stdout.is_empty() {
        Some(if first_line.is_empty() { "detected".to_string() } else { first_line })
    } else {
        None
    }
}

/// Format the full conversation history as a plain-text prompt for non-interactive CLI use.
/// Sent via stdin pipe — never interpolated into a shell command.
pub fn build_cli_prompt(request: &TextGenerationRequest) -> String {
    let mut out = String::with_capacity(1024);
    if !request.system.is_empty() {
        out.push_str("System: ");
        out.push_str(&request.system);
        out.push_str("\n\n");
    }
    for (role, content) in &request.messages {
        let label = if role == "user" { "User" } else { "Assistant" };
        out.push_str(label);
        out.push_str(": ");
        out.push_str(content);
        out.push_str("\n\n");
    }
    out.push_str("Assistant:");
    out
}
