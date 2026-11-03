//! LayoutObject font-fallback reuse admission.
//!
//! This module is included at the reciprocal assembly boundary because the
//! standalone reuse crate intentionally does not depend on LayoutObject. The
//! out-of-line entry keeps fallback invalidation visible as reuse work without
//! emitting per-object trace events.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_core_services.cc:374-378
    #[inline(never)]
    pub fn IsFontFallbackValid(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { &*self.StyleRef().GetFont() }.IsFallbackValid()
            && unsafe { &*self.FirstLineStyleRef().GetFont() }.IsFallbackValid()
    }
}
