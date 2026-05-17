pub const PLATFORM_MACOS: &str = "macos";
pub const PLATFORM_WINDOWS: &str = "windows";
pub const PLATFORM_LINUX: &str = "linux";
pub const PLATFORM_IOS: &str = "ios";

pub trait PlatformInfo {
    fn name(&self) -> &'static str;
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{current, send_system_notification};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{current, send_system_notification};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{current, send_system_notification};

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::{current, send_system_notification};

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "ios"
)))]
mod unknown;
#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_os = "ios"
)))]
pub use unknown::{current, send_system_notification};
