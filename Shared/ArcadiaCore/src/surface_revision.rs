//! Process-wide counter surfaced in [`crate::modules::surface::SurfaceSnapshot`].
//! Bumped whenever [`crate::config::modules::ModulesConfig`] is persisted so thin clients
//! can detect host-side module changes beyond `surface.patch` alone.

use std::sync::atomic::{AtomicU64, Ordering};

static SURFACE_REVISION: AtomicU64 = AtomicU64::new(1);

#[inline]
pub fn bump_surface_revision() {
    SURFACE_REVISION.fetch_add(1, Ordering::SeqCst);
}

#[inline]
pub fn current_surface_revision() -> u64 {
    SURFACE_REVISION.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_advances_monotonic() {
        let a = current_surface_revision();
        bump_surface_revision();
        assert_eq!(current_surface_revision(), a + 1);
    }
}
