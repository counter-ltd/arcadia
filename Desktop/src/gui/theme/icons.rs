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
        "extensions" => "icons/python.svg",
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
        _ => "icons/terminal.svg",
    }
}

pub fn render_icon(glyph_key: &str) -> Svg {
    svg().path(icon_path(glyph_key))
}
