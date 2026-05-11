//! Root view for the shared HUD overlay window (transparent; compositing for extensions comes later).

use openframe::{Context, IntoElement, Render, Window, div, prelude::*};

pub struct OverlayHudRoot;

impl OverlayHudRoot {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for OverlayHudRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}
