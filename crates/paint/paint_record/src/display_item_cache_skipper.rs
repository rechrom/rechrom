//! Scoped display-item cache suppression.
//!
//! Source entity: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/
//! display_item_cache_skipper.h`.

use std::cell::RefCell;

use crate::paint_context::PaintContext;

/// Source: `display_item_cache_skipper.h:14-30`.
///
/// The guard changes cache admission only; recorded paint operations and
/// package boundaries remain unchanged. Nested guards retain the source
/// counter semantics and the final guard restores ordinary cache admission.
pub(crate) struct DisplayItemCacheSkipper<'c, 'o> {
    context_: &'c RefCell<PaintContext<'o>>,
}

impl<'c, 'o> DisplayItemCacheSkipper<'c, 'o> {
    /// Source: `display_item_cache_skipper.h:18-21`.
    pub(crate) fn new(context: &'c RefCell<PaintContext<'o>>) -> Self {
        context.borrow_mut().BeginSkippingCache();
        Self { context_: context }
    }
}

/// Source: `display_item_cache_skipper.h:24-26`.
impl Drop for DisplayItemCacheSkipper<'_, '_> {
    fn drop(&mut self) {
        self.context_.borrow_mut().EndSkippingCache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PaintRecording;

    #[test]
    fn nested_source_scopes_restore_cache_admission_on_the_last_drop() {
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        assert!(!context.borrow().IsSkippingCache());
        let outer = DisplayItemCacheSkipper::new(&context);
        assert!(context.borrow().IsSkippingCache());
        {
            let _inner = DisplayItemCacheSkipper::new(&context);
            assert!(context.borrow().IsSkippingCache());
        }
        assert!(context.borrow().IsSkippingCache());
        drop(outer);
        assert!(!context.borrow().IsSkippingCache());
    }
}
