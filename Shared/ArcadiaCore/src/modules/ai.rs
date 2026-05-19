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
        description:
            "On-device inference via macOS Foundation Models. No API key or network required.",
    },
];

pub fn enabled_ai_providers(module_rows: &[(String, bool)]) -> Vec<&'static AiProviderManifest> {
    let mut providers: Vec<&'static AiProviderManifest> = AI_PROVIDER_REGISTRY
        .iter()
        .filter(|p| {
            module_rows
                .iter()
                .any(|(name, enabled)| name == p.module_name && *enabled)
        })
        .collect();
    providers.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
    providers
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
        "ai-provider-exec-codex" => Some("codex"),
        "ai-provider-exec-gemini" => Some("gemini"),
        "ai-provider-exec-aider" => Some("aider"),
        _ => None,
    }
}

pub fn cli_display_name(module_name: &str) -> &'static str {
    match module_name {
        "ai-provider-exec-claude" => "Claude Code",
        "ai-provider-exec-codex" => "Codex",
        "ai-provider-exec-gemini" => "Gemini",
        "ai-provider-exec-aider" => "Aider",
        _ => "CLI",
    }
}

pub fn provider_display_name(module_name: &str) -> Option<&'static str> {
    AI_PROVIDER_REGISTRY
        .iter()
        .find(|p| p.module_name == module_name)
        .map(|p| p.display_name)
}

#[derive(Default)]
pub struct AiExtension;

impl crate::extension::Extension for AiExtension {
    fn manifest(&self) -> crate::extension::OwnedModuleManifest {
        use crate::extension::OwnedWorkspacePermissionDef;
        crate::extension::OwnedModuleManifest {
            name: NAME.to_string(),
            glyph: "ai-provider".to_string(),
            version: "0.1.0".to_string(),
            description: "AI chat interface. Requires an AI provider module to be enabled."
                .to_string(),
            accent: String::new(),
            required_modules: Vec::new(),
            required_permissions: Vec::new(),
            workspace_permissions: vec![
                OwnedWorkspacePermissionDef {
                    id: "workspace.ai_read".to_string(),
                    title: "File read (AI)".to_string(),
                    description:
                        "Allow the AI to read files via @mention and read_file tool."
                            .to_string(),
                    default_granted: false,
                },
                OwnedWorkspacePermissionDef {
                    id: "workspace.ai_write".to_string(),
                    title: "File write (AI)".to_string(),
                    description: "Allow the AI to create and modify files via write_file tool."
                        .to_string(),
                    default_granted: false,
                },
                OwnedWorkspacePermissionDef {
                    id: "workspace.ai_execute".to_string(),
                    title: "Command execution (AI)".to_string(),
                    description: "Allow the AI to run allowlisted commands via run_command tool."
                        .to_string(),
                    default_granted: false,
                },
            ],
            supported_platforms: Vec::new(),
            api_exports: Vec::new(),
        }
    }
}

crate::register_extension!(AiExtension);
