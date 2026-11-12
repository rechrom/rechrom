// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source core/editing/selection_editor.cc:106-115,123-137,597-648.
//! SelectionEditor's retained derived bounds. The FrameSelection owner keeps
//! this cache beside its existing canonical selection; it stores no positions.
use super::frame_selection::SelectionValue;
use foundation::gfx::Rect;
use std::cell::Cell;

pub trait SelectionAbsoluteBoundsSource<S: SelectionValue> {
    fn UpdateStyleAndLayoutForSelection(&self);
    fn ComputeVisibleSelectionInDOMTree(&self, selection: &S) -> S;
    fn IsCaret(&self, selection: &S) -> bool;
    fn IsAnchorFirst(&self, selection: &S) -> bool;
    fn AbsoluteCaretBounds(&self, selection: &S) -> Rect;
    /// NormalizeSelectionRange + FirstRectForRange of each collapsed endpoint.
    fn NormalizedEndpointBounds(&self, selection: &S) -> Option<(Rect, Rect)>;
}
pub struct SelectionEditorAbsoluteBounds {
    dirty: Cell<bool>,
    has_bounds: Cell<bool>,
    anchor: Cell<Rect>,
    focus: Cell<Rect>,
}
impl Default for SelectionEditorAbsoluteBounds {
    fn default() -> Self {
        Self {
            dirty: Cell::new(true),
            has_bounds: Cell::new(false),
            anchor: Cell::new(Rect::default()),
            focus: Cell::new(Rect::default()),
        }
    }
}
impl SelectionEditorAbsoluteBounds {
    pub fn MarkCacheDirty(&self) {
        if !self.dirty.replace(true) {
            self.has_bounds.set(false);
            self.anchor.set(Rect::default());
            self.focus.set(Rect::default());
        }
    }
    pub fn ComputeAbsoluteBounds<S: SelectionValue>(
        &self,
        selection: &S,
        source: &dyn SelectionAbsoluteBoundsSource<S>,
    ) -> Option<(Rect, Rect)> {
        if self.dirty.replace(false) {
            let (mut anchor, mut focus) = if source.IsCaret(selection) {
                let caret = source.AbsoluteCaretBounds(selection);
                (caret, caret)
            } else {
                let Some(bounds) = source.NormalizedEndpointBounds(selection) else {
                    self.has_bounds.set(false);
                    return None;
                };
                bounds
            };
            if !source.IsAnchorFirst(selection) {
                std::mem::swap(&mut anchor, &mut focus);
            }
            self.anchor.set(anchor);
            self.focus.set(focus);
            self.has_bounds.set(true);
        }
        self.has_bounds
            .get()
            .then(|| (self.anchor.get(), self.focus.get()))
    }
}
