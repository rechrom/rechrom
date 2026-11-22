//! Scroll-source query view (wave E family `scroll`).
//!
//! [`ScrollQueries`] borrows the facade for one call chain and forwards the
//! native scroll, user-scroll and autoscroll source constructors plus the
//! read-only scroll layout queries to the core engine. It owns no state and
//! adds no indirection: every method is `#[inline]` and allocation-free
//! beyond what the core entry point itself returns.
//!
//! `LayoutEngine::Scroll()` lends a shared view for the read-only queries;
//! `LayoutEngine::ScrollMut()` lends an exclusive view that additionally
//! builds the borrowed scroll sources and commits compositor scroll offsets.
//! The scrollbar-hidden command is `LayoutMutation::Scroll`.

use std::ops::Deref;

use foundation::CompositorElementId;
use layout_assembly::internal::layout_input::Offset;
use layout_assembly::internal::layout_scrollable_area::NativeScrollbarActivity;
use layout_assembly::layout_engine::SuppressScrollAnchorScope;
use layout_assembly::engine::scroll::autoscroll_source::{
    NativeAutoscrollLayout, NativeAutoscrollSource, NativeLogicalScrollArea,
};
use layout_assembly::engine::scroll::scroll_source::{
    NativeCompositorScrollUpdate, NativeScrollFrameGeometry, NativeScrollSource,
};
use layout_assembly::engine::scroll::user_scroll_source::NativeUserScrollSource;

use crate::LayoutEngine;

/// Borrowed scroll query view over the facade. `E` is `&LayoutEngine` for
/// [`LayoutEngine::Scroll`] and `&mut LayoutEngine` for
/// [`LayoutEngine::ScrollMut`].
pub struct ScrollQueries<E> {
    engine: E,
}

impl LayoutEngine {
    /// Read-only scroll layout queries.
    #[inline]
    pub fn Scroll(&self) -> ScrollQueries<&'_ LayoutEngine> {
        ScrollQueries { engine: self }
    }

    /// Scroll sources and compositor scroll commits, which need the engine
    /// exclusively; also offers every read-only query of [`Self::Scroll`].
    #[inline]
    pub fn ScrollMut(&mut self) -> ScrollQueries<&'_ mut LayoutEngine> {
        ScrollQueries { engine: self }
    }
}

impl<E: Deref<Target = LayoutEngine>> ScrollQueries<E> {
    #[inline]
    pub fn AutoscrollLayoutFor(&self, node_id: u64) -> Option<NativeAutoscrollLayout> {
        self.engine.engine.AutoscrollLayoutFor(node_id)
    }

    #[inline]
    pub fn LogicalScrollLayoutFor(&self, node_id: u64) -> Option<NativeAutoscrollLayout> {
        self.engine.engine.LogicalScrollLayoutFor(node_id)
    }

    #[inline]
    pub fn LogicalScrollLayoutView(&self) -> Option<NativeAutoscrollLayout> {
        self.engine.engine.LogicalScrollLayoutView()
    }

    #[inline]
    pub fn SuppressScrollAnchoringForNode(
        &self,
        anchor_node: Option<u64>,
    ) -> SuppressScrollAnchorScope {
        self.engine.engine.SuppressScrollAnchoringForNode(anchor_node)
    }
}

impl<'a> ScrollQueries<&'a mut LayoutEngine> {
    #[inline]
    pub fn NativeScrollSource(self, id: u64) -> Option<NativeScrollSource<'a>> {
        let ScrollQueries { engine } = self;
        engine.engine.NativeScrollSource(id)
    }

    #[inline]
    pub fn NativeUserScrollSource<'b>(
        self,
        native: &'b NativeLogicalScrollArea,
    ) -> Option<NativeUserScrollSource<'b>>
    where
        'a: 'b,
    {
        let ScrollQueries { engine } = self;
        engine.engine.NativeUserScrollSource(native)
    }

    #[inline]
    pub fn NativeAutoscrollSource<'b>(
        self,
        object: &'b NativeAutoscrollLayout,
        frame: NativeScrollFrameGeometry,
    ) -> Option<NativeAutoscrollSource<'b>>
    where
        'a: 'b,
    {
        let ScrollQueries { engine } = self;
        engine.engine.NativeAutoscrollSource(object, frame)
    }

    #[inline]
    pub fn DidCompositorScroll(
        self,
        expected: CompositorElementId,
        position: Offset,
        source_type: scroll_snap::scroll_source_type::ScrollSourceType,
    ) -> Option<NativeCompositorScrollUpdate> {
        let ScrollQueries { engine } = self;
        engine
            .engine
            .DidCompositorScroll(expected, position, source_type)
    }

    #[inline]
    pub fn TakeNativeScrollbarActivities(self) -> Vec<NativeScrollbarActivity> {
        let ScrollQueries { engine } = self;
        engine.engine.TakeNativeScrollbarActivities()
    }
}
