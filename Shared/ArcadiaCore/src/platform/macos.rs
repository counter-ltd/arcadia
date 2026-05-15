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
    let _ = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show();
}
