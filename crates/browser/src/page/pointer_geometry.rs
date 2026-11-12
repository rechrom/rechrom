//! Page assembly of call-scoped resident pointer geometry queries.
use super::*;
use foundation::{gfx, LayoutUnit, PhysicalSize, TouchAction};
use interaction::adapter::page_pointer_hit_test::{
    PagePointerGeometry, PointerAdjustmentDOMFacts, PointerGeometryError, PointerHit,
    PreparedPointerHitTest,
};
use interaction::hit_test::{
    core::{
        layout::hit_test_location::HitTestLocation,
        page::touch_adjustment::{self, TouchAdjustmentCandidateType},
    },
    resident::query_services::ResidentQueryViewport,
};

pub(super) struct PagePointerGeometryProvider(pub(super) std::rc::Weak<PageState>);
struct RootViewport;
impl ResidentQueryViewport for RootViewport {
    fn RootFrameToVisualViewport(&self, point: gfx::Point) -> gfx::Point {
        point
    }
}
impl PagePointerGeometryProvider {
    fn Prepare(&self) -> Result<Rc<PageState>, PointerGeometryError> {
        let state = self.0.upgrade().ok_or(PointerGeometryError::Detached)?;
        if !state.layout_engine.borrow().HasCompletedLayoutForHitTest()
            || !state.DocumentLifecycle().IsActive()
        {
            return Err(PointerGeometryError::NoLayout);
        }
        // The same synchronous lifecycle barrier used by native geometry
        // bindings completes before borrowing a source output or running JS.
        state.EnsureLayoutOutput();
        // LocalFrameView's pre-paint lifecycle commits FrameSelection's
        // pending LayoutSelection before context-menu queries read the native
        // per-LayoutText selection bits and endpoint offsets.
        let pending_selection = state
            .layout_editing
            .selections
            .layout_selection
            .borrow()
            .HasPendingSelection();
        if pending_selection {
            let selection = PageMouseFrameGeometry::CurrentSelection(&state);
            let document = state.document.borrow();
            interaction::adapter::page_focus_controller::CommitLayoutSelectionForFocus(
                document.GetDocument(),
                &mut state.layout_engine.borrow_mut(),
                selection,
                false,
            );
        }
        let resident_hit_test_ready = {
            let layout = state.layout_engine.borrow();
            layout
                .GetLayoutResult()
                .and_then(PaintFragmentCursor::TryFromHandle)
                .is_some_and(|fragments| {
                    state
                        .paint_engine
                        .borrow()
                        .ResidentHitTest(fragments)
                        .is_some()
                })
        };
        // LayoutView::HitTest updates the same LocalFrameView lifecycle to
        // PrePaintClean before its resident layer query. This records no
        // display list and publishes no CompositorFrame.
        if state.DocumentLifecycle().GetState()
            < dom::document_lifecycle::LifecycleState::PrePaintClean
            || state.local_frames.get().unwrap().NeedsLifecycle()
            || !resident_hit_test_ready
        {
            let mut layout = state.layout_engine.borrow_mut();
            let retained = crate::persistent_layout::PopulatePaintDataWithEngine(
                &mut layout,
                &mut state.document.borrow_mut(),
                &state.interaction.borrow(),
            )
            .ok_or(PointerGeometryError::NoLayout)?;
            let output = layout.TakePublishedPaintOutput(retained);
            drop(layout);
            let children = state
                .local_frames
                .get()
                .unwrap()
                .PreparePaint(&state.layout_engine.borrow())
                .map_err(|_| PointerGeometryError::NoLayout)?;
            {
                let layout = state.layout_engine.borrow();
                let _layout_paint_scope = layout.PaintScope();
                let child_layouts = children
                    .iter()
                    .map(|child| child.runtime.layout.borrow())
                    .collect::<Vec<_>>();
                let views = super::local_frame::LocalFrameDomain::PaintInputs(
                    &output,
                    &layout,
                    &children,
                    &child_layouts,
                );
                state.SyncVisualViewportPaintState();
                state
                    .paint_engine
                    .borrow_mut()
                    .SetBlockingWheelEventHandlerTargets(
                        super::local_frame_scroll::InteractionScrollOwner::Root(Rc::downgrade(
                            &state,
                        ))
                        .BlockingWheelTargets(),
                    );
                // PopulatePaintDataWithEngine drains the pending typed layout
                // mutations before proving that the resident geometry can be
                // retained. That drain is allowed to schedule layout and
                // rewind the Document from LayoutClean to StyleClean. Only
                // advance the retained-geometry lifecycle after every input
                // refresh has completed and the paint output exists.
                if state.DocumentLifecycle().GetState()
                    == dom::document_lifecycle::LifecycleState::StyleClean
                {
                    state
                        .DocumentLifecycle()
                        .AdvanceTo(dom::document_lifecycle::LifecycleState::LayoutClean);
                }
                state.paint_engine.borrow_mut().PrePaintWithLocalFrameViews(
                    &output,
                    &views,
                    &PagePaintLifecycle(state.DocumentLifecycle()),
                );
            }
            super::local_frame::LocalFrameDomain::ReturnPrePaint(children);
            state
                .layout_engine
                .borrow_mut()
                .ReturnPrePaintOutput(output);
        }
        Ok(state)
    }
    pub(super) fn Query<T>(
        &self,
        read: impl FnOnce(&PreparedPointerHitTest<'_>) -> T,
    ) -> Result<T, PointerGeometryError> {
        self.Prepare()?;
        self.QueryNoLifecycleUpdate(read)
    }
    pub(super) fn QueryNoLifecycleUpdate<T>(
        &self,
        read: impl FnOnce(&PreparedPointerHitTest<'_>) -> T,
    ) -> Result<T, PointerGeometryError> {
        let state = self.0.upgrade().ok_or(PointerGeometryError::Detached)?;
        let document = state.document.Handle();
        let layout = state.layout_engine.borrow();
        let output = layout.Output().ok_or(PointerGeometryError::NoLayout)?;
        let retained = layout
            .GetLayoutResult()
            .ok_or(PointerGeometryError::NoLayout)?;
        let fragments =
            PaintFragmentCursor::TryFromHandle(retained).ok_or(PointerGeometryError::NoLayout)?;
        let paint = state.paint_engine.borrow();
        let text_settings = state.interaction_text_settings.borrow();
        let query = PreparedPointerHitTest {
            document: &document,
            layout: output,
            fragments,
            paint: &paint,
            paths: &paint_skia::path_hit_test::SkiaPathHitTestFactory,
            child_frames: &super::local_frame::LocalFrameQueries {
                domain: state.local_frames.get().unwrap(),
                paint: &paint,
            },
            viewport: &RootViewport,
            text_settings: text_settings.as_ref(),
            native_paths: Arc::new(paint_skia::svg_shape_path::SkiaNativePathFactory),
            editing_positions: Rc::new(
                super::resident_editing_positions::PageResidentEditingPositions(self.0.clone()),
            ),
        };
        Ok(read(&query))
    }
}
impl PagePointerGeometry for PagePointerGeometryProvider {
    fn EventHandlerHitTestNode(
        &self,
        node: u64,
    ) -> Result<
        Option<interaction::hit_test::core::layout::hit_test_result::NodeRef>,
        PointerGeometryError,
    > {
        self.QueryNoLifecycleUpdate(|query| query.EventHandlerHitTestNode(node))
    }
    fn EventHandlerAbsoluteToLocalPoint(
        &self,
        node: u64,
        location: &HitTestLocation,
    ) -> Result<HitTestLocation, PointerGeometryError> {
        self.QueryNoLifecycleUpdate(|query| query.EventHandlerAbsoluteToLocalPoint(node, location))
    }
    fn EventHandlerHitTestNoLifecycleUpdate(
        &self,
        location: HitTestLocation,
        bits: u32,
    ) -> Result<PointerHit, PointerGeometryError> {
        self.QueryNoLifecycleUpdate(|query| query.HitTestInLocalFrame(location, bits))?
            .map_err(PointerGeometryError::Resident)
    }
    fn PrepareLifecycleExceptPaint(&self) -> Result<(), PointerGeometryError> {
        self.Prepare().map(|_| ())
    }
    fn InteractionTextSettings(
        &self,
    ) -> Option<interaction::adapter::page_pointer_hit_test::InteractionTextSettings> {
        let state = self.0.upgrade()?;
        let settings = state.interaction_text_settings.borrow().clone();
        settings
    }
    fn ChildrenOrSiblingsAffectedByActiveOrHover(&self, node: u64) -> bool {
        let Some(state) = self.0.upgrade() else {
            return false;
        };
        let affected = state
            .style_engine
            .borrow()
            .ChildrenOrSiblingsAffectedByActiveOrHover(state.document.borrow().GetDocument(), node);
        affected
    }
    fn HitTest(
        &self,
        location: HitTestLocation,
        bits: u32,
    ) -> Result<PointerHit, PointerGeometryError> {
        self.Query(|query| query.HitTest(location, bits))?
            .map_err(PointerGeometryError::Resident)
    }
    fn HitTestInLocalFrame(
        &self,
        location: HitTestLocation,
        bits: u32,
    ) -> Result<PointerHit, PointerGeometryError> {
        self.Query(|query| query.HitTestInLocalFrame(location, bits))?
            .map_err(PointerGeometryError::Resident)
    }
    fn HitTestAdjustmentRect(
        &self,
        point: gfx::PointF,
        size: PhysicalSize,
    ) -> Result<PointerHit, PointerGeometryError> {
        self.Query(|query| query.HitTestAdjustmentRect(point, size))?
            .map_err(PointerGeometryError::Resident)
    }
    fn BestTouchAdjustment(
        &self,
        kind: TouchAdjustmentCandidateType,
        hit: &PointerHit,
        facts: &dyn PointerAdjustmentDOMFacts,
    ) -> Result<Option<(u64, gfx::Point)>, PointerGeometryError> {
        self.Query(|query| query.BestTouchAdjustment(kind, hit, facts))?
    }
    fn EffectiveTouchAction(&self, node: u64, stylus: bool) -> TouchAction {
        self.Query(|query| query.EffectiveTouchAction(node, stylus))
            .unwrap_or(TouchAction::kAuto)
    }
    fn DeviceScaleFactor(&self) -> f32 {
        self.0.upgrade().map_or(1.0, |state| {
            state.constraints.borrow().device_pixel_ratio as f32
        })
    }
    fn PageScaleFactor(&self) -> f32 {
        self.0
            .upgrade()
            .map_or(1.0, |state| state.web_view_zoom.borrow().Scale())
    }
    fn GetHitTestRectForAdjustment(&self, width: f32, height: f32) -> PhysicalSize {
        touch_adjustment::GetHitTestRectForAdjustment(
            PhysicalSize::new(
                LayoutUnit::from_f64(width as f64),
                LayoutUnit::from_f64(height as f64),
            ),
            self.DeviceScaleFactor(),
            1.0,
            self.PageScaleFactor(),
        )
    }
}
