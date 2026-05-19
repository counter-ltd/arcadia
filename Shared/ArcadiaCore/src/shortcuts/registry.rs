//! The compiled-in shortcut catalog — built from the extension collector.
//!
//! Each module declares the shortcuts it owns via its `Extension` impl; the
//! app shell owns the Arcadia-wide shortcuts. The former hand-written array is
//! gone.

use super::model::ShortcutDefinition;

/// All keyboard / pointer shortcuts, contributed by every extension's
/// `shortcuts()`.
pub static SHORTCUT_DEFINITIONS: std::sync::LazyLock<Vec<ShortcutDefinition>> =
    std::sync::LazyLock::new(|| {
        let providers = crate::extension::provider::default_providers();
        let collected = crate::extension::collector::collect(&providers)
            .expect("extension collector must produce a valid shortcut set");
        collected.shortcuts().iter().map(|s| (*s).clone()).collect()
    });
