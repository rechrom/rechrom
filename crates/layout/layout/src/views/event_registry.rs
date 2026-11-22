//! Event-handler registry query view and the `LayoutMutation::EventRegistry`
//! command values.
//!
//! `EventRegistryLayout` resolves the resident LayoutObject identity; the
//! invalidation of that identity crosses boundary 3 as
//! `LayoutMutation::EventRegistry(EventRegistryMutation::Invalidate { .. })`.

pub use layout_base::engine::event_registry::event_registry_source::{
    EventRegistryMutation, NativeEventRegistryLayout,
};
use layout_base::layout_engine::LayoutEngine as CoreLayoutEngine;

use crate::LayoutEngine;

/// Borrowed event-registry queries over the resident LayoutObject tree.
pub struct EventRegistryQueries<'a> {
    engine: &'a CoreLayoutEngine,
}

impl LayoutEngine {
    #[inline]
    pub fn EventRegistry(&self) -> EventRegistryQueries<'_> {
        EventRegistryQueries {
            engine: &self.engine,
        }
    }
}

impl EventRegistryQueries<'_> {
    /// None names Window::ContentLayoutObject (the same native LayoutView).
    #[inline]
    pub fn EventRegistryLayout(&self, node: Option<u64>) -> Option<NativeEventRegistryLayout> {
        self.engine.EventRegistryLayout(node)
    }
}
