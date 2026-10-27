// LayoutEngine's concrete Paint commit backend is reciprocal assembly code:
// the typed transaction lives in layout_commit, while the engine supplies the
// native clients and resident fragment owner.

use super::*;

pub(super) struct NativePaintCommit<'engine, 'caret> {
    pub(super) engine: &'engine mut LayoutEngine,
    pub(super) caret: Option<&'caret FrameCaret>,
}

impl layout_commit::PaintCommitBackend for NativePaintCommit<'_, '_> {
    type Layout = LayoutPaintOutput;
    type Published = LayoutResultHandle;

    #[inline(never)]
    fn ValidateAuxiliaryClients(&mut self, commit: &PaintClientCommit) {
        if let Some(caret) = self.caret {
            let caret_id = caret.display_item_client().Id();
            if commit.UsedClients().contains(&caret_id) {
                caret.ValidateForCommittedPaint(caret_id);
            }
        }
    }

    #[inline(never)]
    fn CommitNativeClients(&mut self, commit: &PaintClientCommit, layout: &mut LayoutPaintOutput) {
        if let Some(tree) = &self.engine.tree_ {
            tree.CommitPaintClientState(commit.UsedClients(), &mut layout.fragments);
        }
    }

    #[inline(never)]
    fn CommitPrePaintInputs(&mut self, commit: &PaintClientCommit) {
        self.engine
            .pre_paint_inputs_
            .CommitClients(commit.UsedClients().iter().copied());
    }

    #[inline(never)]
    fn PublishLayout(&mut self, layout: LayoutPaintOutput) -> LayoutResultHandle {
        self.engine.output_kind = LayoutUpdateKind::Paint;
        self.engine.invalidated_fragments_ = None;
        self.engine.fragments = Some(layout.fragments.clone());
        layout.fragments
    }
}
