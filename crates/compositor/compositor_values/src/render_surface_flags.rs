//! Render-surface property change flags that the property trees report
//! (cc/trees/property_tree.cc effect updates). Moved from
//! compositor_draw::render_surface_policy, which re-exports it.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderSurfacePropertyChangedFlags {
    pub self_changed: bool,
    pub ancestor_changed: bool,
}

impl RenderSurfacePropertyChangedFlags {
    pub const fn new(self_changed: bool, ancestor_changed: bool) -> Self {
        Self {
            self_changed,
            ancestor_changed,
        }
    }

    pub const fn self_changed(&self) -> bool {
        self.self_changed
    }

    pub const fn ancestor_changed(&self) -> bool {
        self.ancestor_changed
    }
}
