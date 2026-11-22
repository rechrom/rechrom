//! Document::IncDOMTreeVersion/StyleVersion source ownership.
//! Source document.h:1506-1512; document.cc:2494-2520. Mutation producers
//! advance the canonical counter independently of observer/script interest.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static GLOBAL_DOM_TREE_VERSION: AtomicU64 = AtomicU64::new(0);
impl PersistentDocument {
    pub fn IncDOMTreeVersion(&self) {
        self.dom_tree_version.set(
            GLOBAL_DOM_TREE_VERSION
                .fetch_add(1, Ordering::Relaxed)
                .wrapping_add(1),
        );
    }
    pub fn DomTreeVersion(&self) -> u64 {
        self.dom_tree_version.get()
    }
    pub fn StyleVersion(&self) -> u64 {
        self.lifecycle.StyleVersion()
    }
}
