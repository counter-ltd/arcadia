use super::ModuleCommand;

pub const NAME: &str = "ai";

pub fn commands() -> &'static [ModuleCommand] {
    &[]
}

pub struct AiProviderManifest {
    pub module_name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
}

pub const AI_PROVIDER_REGISTRY: &[AiProviderManifest] = &[
    AiProviderManifest {
        module_name: "ai-provider-llama-cpp",
        display_name: "llama.cpp",
        description: "Local inference via llama.cpp. Runs models directly on this machine.",
    },
    AiProviderManifest {
        module_name: "ai-provider-ollama",
        display_name: "Ollama",
        description: "Local inference via Ollama HTTP API.",
    },
    AiProviderManifest {
        module_name: "ai-provider-openai",
        display_name: "OpenAI",
        description: "Cloud inference via OpenAI API.",
    },
    AiProviderManifest {
        module_name: "ai-provider-exec-claude",
        display_name: "Claude (CLI)",
        description: "Claude Pro via installed `claude` CLI. No API key required.",
    },
    AiProviderManifest {
        module_name: "ai-provider-exec-codex",
        display_name: "Codex (CLI)",
        description: "ChatGPT Plus via installed `codex` CLI. No API key required.",
    },
    AiProviderManifest {
        module_name: "ai-provider-exec-gemini",
        display_name: "Gemini (CLI)",
        description: "Gemini Advanced via installed `gemini` CLI. No API key required.",
    },
    AiProviderManifest {
        module_name: "ai-provider-exec-aider",
        display_name: "Aider (CLI)",
        description: "Aider with its configured backend. No API key required.",
    },
    AiProviderManifest {
        module_name: "ai-provider-apfel",
        display_name: "Apple Intelligence",
        description: "On-device inference via macOS Foundation Models. No API key or network required.",
    },
];

pub fn enabled_ai_providers(module_rows: &[(String, bool)]) -> Vec<&'static AiProviderManifest> {
    AI_PROVIDER_REGISTRY
        .iter()
        .filter(|p| {
            module_rows
                .iter()
                .any(|(name, enabled)| name == p.module_name && *enabled)
        })
        .collect()
}

pub fn any_ai_provider_enabled(module_rows: &[(String, bool)]) -> bool {
    AI_PROVIDER_REGISTRY.iter().any(|p| {
        module_rows
            .iter()
            .any(|(name, enabled)| name == p.module_name && *enabled)
    })
}

/// Returns true when at least one AI provider module is enabled.
pub fn is_ai_provider_available(module_rows: &[(String, bool)]) -> bool {
    any_ai_provider_enabled(module_rows)
}

pub fn is_cli_provider(module_name: &str) -> bool {
    matches!(
        module_name,
        "ai-provider-exec-claude"
            | "ai-provider-exec-codex"
            | "ai-provider-exec-gemini"
            | "ai-provider-exec-aider"
    )
}

pub fn cli_binary_for_module(module_name: &str) -> Option<&'static str> {
    match module_name {
        "ai-provider-exec-claude" => Some("claude"),
        "ai-provider-exec-codex"  => Some("codex"),
        "ai-provider-exec-gemini" => Some("gemini"),
        "ai-provider-exec-aider"  => Some("aider"),
        _ => None,
    }
}

pub fn cli_display_name(module_name: &str) -> &'static str {
    match module_name {
        "ai-provider-exec-claude" => "Claude Code",
        "ai-provider-exec-codex"  => "Codex",
        "ai-provider-exec-gemini" => "Gemini",
        "ai-provider-exec-aider"  => "Aider",
        _ => "CLI",
    }
}

pub fn provider_display_name(module_name: &str) -> Option<&'static str> {
    AI_PROVIDER_REGISTRY
        .iter()
        .find(|p| p.module_name == module_name)
        .map(|p| p.display_name)
}
