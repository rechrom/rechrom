// LayoutEngine's concrete Paint commit backend is reciprocal assembly code:
// the typed transaction lives in layout_commit, while the engine supplies the
// source clients and resident fragment owner.

use super::*;

pub(super) struct PaintCommit<'engine, 'caret> {
    pub(super) engine: &'engine mut LayoutEngine,
    pub(super) editing: Option<&'caret LayoutEditingState>,
}

impl layout_commit::PaintCommitBackend for PaintCommit<'_, '_> {
    type Layout = LayoutPaintOutput;
    type Published = LayoutResultHandle;

    #[inline(never)]
    fn ValidateAuxiliaryClients(&mut self, commit: &PaintClientCommit) {
        if let Some(editing) = self.editing {
            let caret = editing.caret.borrow();
            let caret_id = caret.display_item_client().Id();
            if commit.UsedClients().contains(&caret_id) {
                caret.ValidateForCommittedPaint(caret_id);
            }
            let drag_caret = editing.drag_caret.borrow();
            let drag_id = drag_caret.display_item_client().Id();
            if commit.UsedClients().contains(&drag_id) {
                drag_caret.ValidateForCommittedPaint(drag_id);
            }
        }
    }

    #[inline(never)]
    fn CommitClients(&mut self, commit: &PaintClientCommit, layout: &mut LayoutPaintOutput) {
        let _scope = self.engine.NativeScrollPassScope();
        if let Some(tree) = &self.engine.tree_ {
            tree.CommitPaintClientState(
                commit.UsedClients(),
                commit.PendingDelayedPaintInvalidations(),
                &mut layout.fragments,
            );
            // LocalFrameView::PaintTree clears only the source dirty paths
            // after completed paint; display-locked subtrees retain theirs.
            if let Some(layer) = unsafe { tree.Root().PaintingLayerDefault().as_mut() } {
                layer.ClearNeedsRepaintRecursively();
            }
        }
        layout
            .fragments
            .PaintState::<LayoutPaintState>()
            .expect("published layout result owns paint state")
            .CommitPublication();
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
