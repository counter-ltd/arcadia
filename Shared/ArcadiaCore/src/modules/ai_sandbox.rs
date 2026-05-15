use crate::modules::ai_types::AiWorkspaceContext;

pub enum SandboxOp<'a> {
    Read(&'a str),
    Write(&'a str),
    Execute(&'a str),
    List(&'a str),
}

fn require_ctx(ctx: Option<&AiWorkspaceContext>) -> Result<&AiWorkspaceContext, String> {
    ctx.ok_or_else(|| {
        "No workspace context — select a workspace in the AI chat panel to enable file access"
            .to_string()
    })
}

pub fn check(ctx: Option<&AiWorkspaceContext>, op: &SandboxOp) -> Result<(), String> {
    let ctx = require_ctx(ctx)?;
    let (path, permitted, perm_name) = match op {
        SandboxOp::Read(p) | SandboxOp::List(p) => (*p, ctx.can_read(), "workspace.ai_read"),
        SandboxOp::Write(p) => (*p, ctx.can_write(), "workspace.ai_write"),
        SandboxOp::Execute(p) => (*p, ctx.can_execute(), "workspace.ai_execute"),
    };
    if !ctx.is_path_in_scope(path) {
        return Err(format!("Path out of workspace scope: {path}"));
    }
    if !permitted {
        return Err(format!(
            "Permission denied: {perm_name} not granted for this workspace"
        ));
    }
    Ok(())
}

pub fn sandboxed_read(ctx: Option<&AiWorkspaceContext>, path: &str) -> Result<String, String> {
    check(ctx, &SandboxOp::Read(path))?;
    std::fs::read_to_string(path).map_err(|e| format!("Read {path}: {e}"))
}

pub fn sandboxed_write(
    ctx: Option<&AiWorkspaceContext>,
    path: &str,
    content: &str,
) -> Result<(), String> {
    check(ctx, &SandboxOp::Write(path))?;
    std::fs::write(path, content).map_err(|e| format!("Write {path}: {e}"))
}

pub fn sandboxed_list(ctx: Option<&AiWorkspaceContext>, path: &str) -> Result<Vec<String>, String> {
    check(ctx, &SandboxOp::List(path))?;
    let entries = std::fs::read_dir(path).map_err(|e| format!("List {path}: {e}"))?;
    Ok(entries
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_string_lossy().to_string())
        .collect())
}

/// Binaries allowed by the AI executor. All others are rejected regardless of
/// the workspace.execute grant. This is a defence-in-depth measure — the GUI
/// layer should additionally present a per-command confirmation prompt before
/// calling this function.
///
/// SECURITY: This checks only the leading binary name. The full command is passed
/// to `sh -c`, so allowed binaries can still chain via `&&`, `;`, or pipes. This
/// is acceptable because the allowlist excludes shells (`sh`, `bash`, `zsh`) and
/// destructive tools (`rm`, `curl`, `dd`). Do NOT add shells or network tools to
/// this list without also adding a secondary confirmation gate at the call site.
const EXEC_ALLOWLIST: &[&str] = &[
    "cargo",
    "rustc",
    "rustfmt",
    "clippy-driver",
    "npm",
    "npx",
    "node",
    "yarn",
    "pnpm",
    "python",
    "python3",
    "pip",
    "pip3",
    "uv",
    "git",
    "gh",
    "make",
    "cmake",
    "ninja",
    "ls",
    "find",
    "grep",
    "rg",
    "cat",
    "head",
    "tail",
    "wc",
    "echo",
    "printf",
    "mkdir",
    "cp",
    "mv",
    "swift",
    "swiftc",
    "go",
    "java",
    "javac",
    "mvn",
    "gradle",
    "ruby",
    "gem",
    "bundle",
    // AI CLI exec providers (subscription-based; no API key)
    "claude",
    "codex",
    "gemini",
    "aider",
];

fn exec_binary(cmd: &str) -> Option<&str> {
    cmd.trim().split_ascii_whitespace().next()
}

fn is_exec_allowed(cmd: &str) -> bool {
    match exec_binary(cmd) {
        Some(bin) => EXEC_ALLOWLIST.contains(&bin),
        None => false,
    }
}

/// Check whether a standalone binary name is in the exec allowlist.
/// Used by `run_exec_cli` before spawning a CLI provider process.
pub fn is_binary_allowed(binary: &str) -> bool {
    EXEC_ALLOWLIST.contains(&binary)
}

pub fn sandboxed_exec(ctx: Option<&AiWorkspaceContext>, cmd: &str) -> Result<String, String> {
    let ctx = require_ctx(ctx)?;
    check(Some(ctx), &SandboxOp::Execute(&ctx.workspace_path.clone()))?;
    if !is_exec_allowed(cmd) {
        let bin = exec_binary(cmd).unwrap_or("<empty>");
        return Err(format!(
            "Exec denied: '{bin}' is not in the allowed command list. \
             Only development toolchain commands are permitted."
        ));
    }
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(&ctx.workspace_path)
        .output()
        .map_err(|e| format!("Exec: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    Ok(format!("{stdout}{stderr}").trim().to_string())
}
