//! Manifest-only AI sub-modules: rule/skill configuration and the
//! subscription-CLI / on-device inference providers. These have no commands of
//! their own — they extend the `ai` module's UI and provider set — so each is a
//! pure [`Extension`] manifest registered via `inventory`.

use crate::config::modules::AI_MODULE_NAME;
use crate::extension::{Extension, OwnedModuleManifest};
use crate::platform::PLATFORM_MACOS;

/// Build a manifest for an AI sub-module. All require the `ai` module.
fn ai_submodule_manifest(
    name: &str,
    glyph: &str,
    description: &str,
    accent: &str,
    supported_platforms: Vec<String>,
) -> OwnedModuleManifest {
    OwnedModuleManifest {
        name: name.to_string(),
        glyph: glyph.to_string(),
        version: "0.1.0".to_string(),
        description: description.to_string(),
        accent: accent.to_string(),
        required_modules: vec![AI_MODULE_NAME.to_string()],
        required_permissions: Vec::new(),
        workspace_permissions: Vec::new(),
        supported_platforms,
        api_exports: Vec::new(),
    }
}

macro_rules! ai_submodule {
    ($struct:ident, $name:path, $glyph:literal, $accent:literal, $platforms:expr, $desc:literal) => {
        #[derive(Default)]
        pub struct $struct;

        impl Extension for $struct {
            fn manifest(&self) -> OwnedModuleManifest {
                ai_submodule_manifest($name, $glyph, $desc, $accent, $platforms)
            }
        }

        crate::register_extension!($struct);
    };
}

ai_submodule!(
    AiRulesExtension,
    crate::config::modules::AI_RULES_MODULE_NAME,
    "ai-rule",
    "",
    Vec::new(),
    "AI Rules — per-chat constraints: forbidden tools, response format, persona. Adds a configuration page to the AI sidebar."
);

ai_submodule!(
    AiSkillsExtension,
    crate::config::modules::AI_SKILLS_MODULE_NAME,
    "ai-skill",
    "",
    Vec::new(),
    "AI Skills — named behaviours: system prompt fragments, tool allowlists, parameter overrides. Adds a configuration page to the AI sidebar."
);

ai_submodule!(
    AiExecClaudeExtension,
    crate::config::modules::AI_EXEC_CLAUDE_MODULE_NAME,
    "claude",
    "orange",
    Vec::new(),
    "Claude CLI provider — uses the installed `claude` binary with a Claude Pro subscription. No API key required."
);

ai_submodule!(
    AiExecCodexExtension,
    crate::config::modules::AI_EXEC_CODEX_MODULE_NAME,
    "codex",
    "cyan",
    Vec::new(),
    "Codex CLI provider — uses the installed `codex` binary with a ChatGPT Plus subscription. No API key required."
);

ai_submodule!(
    AiExecGeminiExtension,
    crate::config::modules::AI_EXEC_GEMINI_MODULE_NAME,
    "gemini",
    "sky",
    Vec::new(),
    "Gemini CLI provider — uses the installed `gemini` binary with a Gemini Advanced subscription. No API key required."
);

ai_submodule!(
    AiExecAiderExtension,
    crate::config::modules::AI_EXEC_AIDER_MODULE_NAME,
    "aider",
    "emerald",
    Vec::new(),
    "Aider CLI provider — uses the installed `aider` binary with its own configured backend."
);

ai_submodule!(
    AiApfelExtension,
    crate::config::modules::AI_APFEL_MODULE_NAME,
    "apfel",
    "indigo",
    vec![PLATFORM_MACOS.to_string()],
    "Apple Intelligence provider — on-device inference via the macOS Foundation Models framework. No API key, no network required."
);
