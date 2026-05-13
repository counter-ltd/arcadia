//! Which Python extension is currently on the stack (load body, timer tick, command handler, …).
//! Used by `arcadia.execute` so native commands can honor grants on `python:<id>` when the
//! extension holds the same permission id.

use std::cell::RefCell;

thread_local! {
    static INVOKING_EXTENSION: RefCell<Option<String>> = RefCell::new(None);
}

/// Extension id currently running Python entrypoints for this thread, if any.
pub fn invoking_extension_id() -> Option<String> {
    INVOKING_EXTENSION.with(|c| c.borrow().clone())
}

pub struct PythonExtensionScope {
    previous: Option<String>,
}

impl PythonExtensionScope {
    /// Pushes `extension_id` for the duration of this guard (restore previous on [`Drop`]).
    pub fn enter(extension_id: String) -> Self {
        let previous = INVOKING_EXTENSION.with(|c| {
            let prev = c.borrow().clone();
            *c.borrow_mut() = Some(extension_id);
            prev
        });
        Self { previous }
    }
}

impl Drop for PythonExtensionScope {
    fn drop(&mut self) {
        INVOKING_EXTENSION.with(|c| {
            *c.borrow_mut() = self.previous.take();
        });
    }
}
