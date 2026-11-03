//! Scoped propagation of PaintChunk's effectively-invisible state.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/scoped_effectively_invisible.h`.

use std::cell::RefCell;

use crate::paint_context::PaintContext;

pub struct ScopedEffectivelyInvisible<'a, 'o> {
    context: &'a RefCell<PaintContext<'o>>,
    previous_effectively_invisible: bool,
}

impl<'a, 'o> ScopedEffectivelyInvisible<'a, 'o> {
    pub fn new(context: &'a RefCell<PaintContext<'o>>) -> Self {
        let previous_effectively_invisible = context.borrow().CurrentEffectivelyInvisible();
        context.borrow_mut().SetCurrentEffectivelyInvisible(true);
        Self {
            context,
            previous_effectively_invisible,
        }
    }
}

impl Drop for ScopedEffectivelyInvisible<'_, '_> {
    fn drop(&mut self) {
        self.context
            .borrow_mut()
            .SetCurrentEffectivelyInvisible(self.previous_effectively_invisible);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PaintRecording;

    #[test]
    fn restores_the_previous_chunk_state() {
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        assert!(!context.borrow().CurrentEffectivelyInvisible());
        {
            let _scope = ScopedEffectivelyInvisible::new(&context);
            assert!(context.borrow().CurrentEffectivelyInvisible());
        }
        assert!(!context.borrow().CurrentEffectivelyInvisible());
    }
}
