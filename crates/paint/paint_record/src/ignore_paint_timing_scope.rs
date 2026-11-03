//! Scoped state used to suppress Paint Timing observation.
//!
//! Source entity: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/
//! ignore_paint_timing_scope.{h,cc}`.

use std::cell::Cell;

thread_local! {
    // Source: ignore_paint_timing_scope.cc:9-10. Blink Paint runs on its
    // rendering thread; thread-local storage preserves that ownership without
    // allowing one concurrently painted document to reset another's scope.
    static IGNORE_DEPTH: Cell<i32> = const { Cell::new(0) };
    static IS_DOCUMENT_ELEMENT_INVISIBLE: Cell<bool> = const { Cell::new(false) };
}

/// Source: `ignore_paint_timing_scope.h:19-51`.
pub struct IgnorePaintTimingScope {
    previous_ignore_depth_: i32,
    previous_is_document_element_invisible_: bool,
}

impl IgnorePaintTimingScope {
    /// Source: `ignore_paint_timing_scope.h:23-27`.
    pub fn new() -> Self {
        Self {
            previous_ignore_depth_: Self::IgnoreDepth(),
            previous_is_document_element_invisible_: Self::IsDocumentElementInvisible(),
        }
    }

    pub fn SetIsDocumentElementInvisible(is_invisible: bool) {
        IS_DOCUMENT_ELEMENT_INVISIBLE.set(is_invisible);
    }

    pub fn IsDocumentElementInvisible() -> bool {
        IS_DOCUMENT_ELEMENT_INVISIBLE.get()
    }

    pub fn IncrementIgnoreDepth() {
        IGNORE_DEPTH.set(Self::IgnoreDepth() + 1);
    }

    pub fn IgnoreDepth() -> i32 {
        IGNORE_DEPTH.get()
    }

    /// Source: `ignore_paint_timing_scope.h:38-44`.
    pub fn ShouldIgnore() -> bool {
        (!Self::IsDocumentElementInvisible() && Self::IgnoreDepth() != 0) || Self::IgnoreDepth() > 1
    }
}

impl Default for IgnorePaintTimingScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for IgnorePaintTimingScope {
    fn drop(&mut self) {
        IGNORE_DEPTH.set(self.previous_ignore_depth_);
        IS_DOCUMENT_ELEMENT_INVISIBLE.set(self.previous_is_document_element_invisible_);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_scope_restores_both_source_globals_and_ignore_policy() {
        assert_eq!(IgnorePaintTimingScope::IgnoreDepth(), 0);
        assert!(!IgnorePaintTimingScope::IsDocumentElementInvisible());
        {
            let _outer = IgnorePaintTimingScope::new();
            IgnorePaintTimingScope::SetIsDocumentElementInvisible(true);
            IgnorePaintTimingScope::IncrementIgnoreDepth();
            assert!(!IgnorePaintTimingScope::ShouldIgnore());
            {
                let _inner = IgnorePaintTimingScope::new();
                IgnorePaintTimingScope::IncrementIgnoreDepth();
                assert!(IgnorePaintTimingScope::ShouldIgnore());
            }
            assert_eq!(IgnorePaintTimingScope::IgnoreDepth(), 1);
            assert!(IgnorePaintTimingScope::IsDocumentElementInvisible());
        }
        assert_eq!(IgnorePaintTimingScope::IgnoreDepth(), 0);
        assert!(!IgnorePaintTimingScope::IsDocumentElementInvisible());
    }
}
