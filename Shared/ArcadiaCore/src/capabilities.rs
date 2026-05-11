//! Stable user-facing capability strings — keep CLI, LAN hosts, and surfaces aligned.

/// Returned when `shell.execute` runs **locally** on iOS (no subprocess). Routed commands
/// (`ExecutionContext.net_as: lan:…`) never hit this path — they forward from [`crate::modules::execute_command`].
pub const SHELL_EXECUTE_UNAVAILABLE_IOS_LOCAL: &str =
    "shell.execute is not available locally on iOS. Route commands to a host via LAN thin-client settings (ExecutionContext.net_as), or use another surface.";

/// PTY-backed [`crate::modules::shell`] paths (`shell.internal` with an injected executor) when none is registered.
pub const SHELL_INTERNAL_NO_RUNTIME: &str =
    "shell.internal is not available in this runtime (PTY-backed executor not registered).";
