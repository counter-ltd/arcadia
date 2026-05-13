//! iOS static library entry point. Compiled only with `--features ios-gui`.
//! The Swift host calls the `extern "C"` functions below to start OpenFrame and
//! forward UITouch events into the Rust render loop.

#[cfg(feature = "ios-gui")]
mod gui;

#[cfg(feature = "ios-gui")]
pub use ios_ffi::*;

#[cfg(feature = "ios-gui")]
mod ios_ffi {
    use std::os::raw::{c_char, c_void};
    use std::path::PathBuf;

    /// Called once from the Swift host after UIApplication starts.
    /// `metal_layer_ptr` is the `Unmanaged.passUnretained(caMetalLayer).toOpaque()` value.
    /// `config_root` is the null-terminated UTF-8 app Documents directory path.
    #[no_mangle]
    pub extern "C" fn arcadia_ios_start(metal_layer_ptr: usize, config_root: *const c_char) {
        let config_root = unsafe { std::ffi::CStr::from_ptr(config_root) }
            .to_string_lossy()
            .to_string();
        arcadia_core::config::set_config_root(PathBuf::from(config_root));
        arcadia_core::modules::load_all();
        crate::gui::app::entry_ios::run(metal_layer_ptr);
    }

    /// Forward a UITouch event into the OpenFrame event loop.
    /// `phase`: 0 = began, 1 = moved, 2 = ended, 3 = cancelled.
    #[no_mangle]
    pub extern "C" fn arcadia_ios_inject_touch(x: f32, y: f32, phase: u8) {
        crate::gui::app::entry_ios::inject_touch(x, y, phase);
    }

    /// Populates `*out_count` with accessibility snapshot length after latest drawn frame.
    #[no_mangle]
    pub unsafe extern "C" fn arcadia_ios_accessibility_node_count(out_count: *mut usize) {
        if out_count.is_null() {
            return;
        }
        *out_count = openframe::snapshot_node_count();
    }

    /// Writes hit rectangle (`out_rect4` = x, y, w, h), traits, and NUL-terminated UTF-8 strings.
    #[no_mangle]
    pub unsafe extern "C" fn arcadia_ios_accessibility_query_node(
        index: usize,
        out_rect4: *mut f32,
        out_traits: *mut u64,
        label_buf: *mut c_void,
        label_cap: usize,
        hint_buf: *mut c_void,
        hint_cap: usize,
        out_label_len: *mut usize,
        out_hint_len: *mut usize,
    ) {
        openframe::snapshot_query_node(
            index,
            out_rect4,
            out_traits,
            label_buf.cast::<u8>(),
            label_cap,
            hint_buf.cast::<u8>(),
            hint_cap,
            out_label_len,
            out_hint_len,
        );
    }
}
