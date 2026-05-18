use openframe::{svg, Svg};

pub fn icon_path(glyph_key: &str) -> &'static str {
    match glyph_key {
        "terminal" => "icons/terminal.svg",
        "home" => "icons/home.svg",
        "logs" => "icons/logs.svg",
        "log-out" => "icons/log-out.svg",
        "x" => "icons/x.svg",
        "settings" => "icons/settings.svg",
        "appearance" => "icons/appearance.svg",
        "extensions" => "icons/puzzle.svg",
        "python" => "icons/python.svg",
        "permissions" => "icons/permissions.svg",
        "nodes" => "icons/nodes.svg",
        "tools" => "icons/tools.svg",
        "services" => "icons/services.svg",
        "network" => "icons/network.svg",
        "chat" => "icons/chat.svg",
        "music" => "icons/music.svg",
        "flask" => "icons/flask.svg",
        "coffee" => "icons/coffee.svg",
        "workspaces" => "icons/workspaces.svg",
        "folder" => "icons/folder.svg",
        "folder-open" => "icons/folder-open.svg",
        "file" => "icons/file.svg",
        "file-code" => "icons/file-code.svg",
        "blocks" => "icons/blocks.svg",
        "file-text" => "icons/file-text.svg",
        "chevron-left" => "icons/chevron-left.svg",
        "chevron-right" => "icons/chevron-right.svg",
        "chevron-down" => "icons/chevron-down.svg",
        "pin" => "icons/pin.svg",
        "pin-fill" => "icons/pin-fill.svg",
        "code" => "icons/modules.svg",
        "modules" => "icons/modules.svg",
        "goto" => "icons/goto.svg",
        "shortcuts" => "icons/shortcuts.svg",
        "message" => "icons/message.svg",
        "animation" => "icons/animation.svg",
        "surface" => "icons/surface.svg",
        "tray" => "icons/tray.svg",
        "cursor" => "icons/cursor.svg",
        "overlay" => "icons/overlay.svg",
        "ai-provider" => "icons/ai-provider.svg",
        "ai-model" => "icons/ai-model.svg",
        "ai-rule" => "icons/ai-rule.svg",
        "ai-skill" => "icons/ai-skill.svg",
        "claude" => "icons/claude.svg",
        "openai" => "icons/openai.svg",
        "ollama" => "icons/ollama.svg",
        "llama-cpp" => "icons/llama-cpp.svg",
        "codex" => "icons/codex.svg",
        "gemini" => "icons/gemini.svg",
        "aider" => "icons/aider.svg",
        "apfel" => "icons/apfel.svg",
        "notification" => "icons/notification.svg",
        "notification-on" => "icons/notification-on.svg",
        "type-text" => "icons/type-text.svg",
        "type-image" => "icons/type-image.svg",
        "type-vision" => "icons/type-vision.svg",
        "type-embedding" => "icons/type-embedding.svg",
        _ => "icons/terminal.svg",
    }
}

pub fn render_icon(glyph_key: &str) -> Svg {
    if glyph_key.starts_with("extension-icon/") {
        svg().path(openframe::SharedString::from(glyph_key.to_string()))
    } else {
        svg().path(icon_path(glyph_key))
    }
}
