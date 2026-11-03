//! Scoped subsequence cache boundary.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/platform/graphics/paint/subsequence_recorder.h`.

use std::{cell::RefCell, sync::Arc};

use paint_chunk::DisplayItemClientInfo;
use paint_model::PaintResult;

use crate::paint_context::PaintContext;

/// Source: `subsequence_recorder.h:28-53`.
pub struct SubsequenceRecorder<'c, 'o> {
    context_: &'c RefCell<PaintContext<'o>>,
    subsequence_index_: usize,
}

impl<'c, 'o> SubsequenceRecorder<'c, 'o> {
    /// Source: `subsequence_recorder.h:32-35`.
    pub fn UseCachedSubsequenceIfPossible(
        context: &RefCell<PaintContext<'o>>,
        client_id: u64,
        client: DisplayItemClientInfo,
        debug: Option<(i32, Arc<str>)>,
    ) -> Option<PaintResult> {
        context
            .borrow_mut()
            .UseCachedSubsequenceIfPossible(client_id, client, debug)
    }

    pub fn SetPaintResult(&self, result: PaintResult) {
        self.context_
            .borrow_mut()
            .SetSubsequencePaintResult(self.subsequence_index_, result);
    }

    /// Source: `subsequence_recorder.h:37-41`.
    pub fn new(
        context: &'c RefCell<PaintContext<'o>>,
        client_id: u64,
        debug: Option<(i32, Arc<str>)>,
    ) -> Self {
        let subsequence_index_ = context.borrow_mut().BeginSubsequence(client_id, debug);
        Self {
            context_: context,
            subsequence_index_,
        }
    }
}

/// Source: `subsequence_recorder.h:46-48`.
impl Drop for SubsequenceRecorder<'_, '_> {
    fn drop(&mut self) {
        self.context_
            .borrow_mut()
            .EndSubsequence(self.subsequence_index_);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PaintRecording;

    #[test]
    fn empty_scoped_subsequence_is_omitted_on_drop() {
        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        {
            let _recorder = SubsequenceRecorder::new(&context, 7, None);
        }
        let subsequences = context.borrow_mut().Finish();
        assert!(subsequences.GetSubsequenceMarkers(7).is_none());
    }
}
