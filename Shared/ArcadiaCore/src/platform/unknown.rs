use super::PlatformInfo;

pub struct UnknownPlatform;

impl PlatformInfo for UnknownPlatform {
    fn name(&self) -> &'static str {
        "unknown"
    }
}

pub fn current() -> impl PlatformInfo {
    UnknownPlatform
}

pub fn send_system_notification(_title: &str, _body: &str) {}
