use super::PlatformInfo;

pub struct WindowsPlatform;

impl PlatformInfo for WindowsPlatform {
    fn name(&self) -> &'static str {
        "windows"
    }
}

pub fn current() -> impl PlatformInfo {
    WindowsPlatform
}

pub fn send_system_notification(title: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .app_name("Arcadia")
        .show();
}
