use super::PlatformInfo;

pub struct MacOsPlatform;

impl PlatformInfo for MacOsPlatform {
    fn name(&self) -> &'static str {
        "macos"
    }
}

pub fn current() -> impl PlatformInfo {
    MacOsPlatform
}

pub fn send_system_notification(title: &str, body: &str) {
    fn osascript_str(s: &str) -> String {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    }
    let script = format!(
        "display notification {} with title {}",
        osascript_str(body),
        osascript_str(title),
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output();
}
