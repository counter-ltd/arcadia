use super::PlatformInfo;

pub struct IosPlatform;

impl PlatformInfo for IosPlatform {
    fn name(&self) -> &'static str {
        "ios"
    }
}

pub fn current() -> impl PlatformInfo {
    IosPlatform
}

pub fn send_system_notification(_title: &str, _body: &str) {}
