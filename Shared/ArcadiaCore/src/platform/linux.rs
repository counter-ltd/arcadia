use super::PlatformInfo;

pub struct LinuxPlatform;

impl PlatformInfo for LinuxPlatform {
    fn name(&self) -> &'static str {
        "linux"
    }
}

pub fn current() -> impl PlatformInfo {
    LinuxPlatform
}

pub fn send_system_notification(title: &str, body: &str) {
    let _ = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .show();
}
