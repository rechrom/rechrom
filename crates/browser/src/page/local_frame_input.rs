//! LocalFrame EventHandler assembly. Chromium event_handler.cc forwards mouse
//! input to the hit local child; KeyboardEventManager uses the focused Frame.
//! Document/Interaction/Layout owners remain resident in the same Page domain.
use super::local_frame::{LocalFrame, LocalFrameDomain, LocalFrameRuntime};
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

pub(super) struct ChildInputGeometry {
    frame: Weak<LocalFrame>,
    runtime: Weak<LocalFrameRuntime>,
}
impl ChildInputGeometry {
    pub(super) fn new(frame: &Rc<LocalFrame>, runtime: &Rc<LocalFrameRuntime>) -> Rc<Self> {
        Rc::new(Self {
            frame: Rc::downgrade(frame),
            runtime: Rc::downgrade(runtime),
        })
    }
    fn Owners(
        &self,
    ) -> Result<(Rc<LocalFrame>, Rc<LocalFrameRuntime>, Rc<PageState>), PointerGeometryError> {
        let frame = self.frame.upgrade().ok_or(PointerGeometryError::Detached)?;
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PointerGeometryError::Detached)?;
        if !frame.IsCurrentRuntime(&runtime) {
            return Err(PointerGeometryError::Detached);
        }
        let page = frame
            .domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
            .ok_or(PointerGeometryError::Detached)?;
        Ok((frame, runtime, page))
    }
    fn Query<T>(
        &self,
        read: impl FnOnce(&PreparedPointerHitTest<'_>) -> T,
    ) -> Result<T, PointerGeometryError> {
        let (_, _, page) = self.Owners()?;
        // Prepare the original local-root/child PrePaint owners as one lifecycle
        // barrier, then release all loans before a listener can run.
        super::pointer_geometry::PagePointerGeometryProvider(Rc::downgrade(&page))
            .PrepareLifecycleExceptPaint()?;
        self.QueryNoLifecycleUpdate(read)
    }
    fn QueryNoLifecycleUpdate<T>(
        &self,
        read: impl FnOnce(&PreparedPointerHitTest<'_>) -> T,
    ) -> Result<T, PointerGeometryError> {
        let (frame, runtime, page) = self.Owners()?;
        let layout = runtime.layout.borrow();
        let output = layout.Output().ok_or(PointerGeometryError::NoLayout)?;
        let retained = layout
            .GetLayoutResult()
            .ok_or(PointerGeometryError::NoLayout)?;
        let fragments =
            PaintFragmentCursor::TryFromHandle(retained).ok_or(PointerGeometryError::NoLayout)?;
        let paint = page.paint_engine.borrow();
        let domain = frame
            .domain
            .upgrade()
            .ok_or(PointerGeometryError::Detached)?;
        let document = runtime.document.Handle();
        let settings = page.interaction_text_settings.borrow();
        Ok(read(&PreparedPointerHitTest {
            document: &document,
            layout: output,
            fragments,
            paint: &paint,
            paths: &paint_skia::path_hit_test::SkiaPathHitTestFactory,
            child_frames: &super::local_frame::LocalFrameQueries {
                domain: &domain,
                paint: &paint,
            },
            viewport: self,
            text_settings: settings.as_ref(),
            native_paths: Arc::new(paint_skia::svg_shape_path::SkiaNativePathFactory),
            editing_positions: runtime
                .positions
                .get()
                .expect("child editing query owner")
                .clone(),
        }))
    }
}
impl ResidentQueryViewport for ChildInputGeometry {
    fn RootFrameToVisualViewport(&self, point: gfx::Point) -> gfx::Point {
        point
    }
    fn LocalFramePointToRootFrame(&self, point: gfx::Point) -> gfx::Point {
        interaction::event_dispatcher::MouseFrameGeometry::NonLocatedPointToRootFrame(self, point)
            .expect("prepared child FrameView")
    }
    fn LocalFrameRectToRootFrame(&self, rect: gfx::Rect) -> gfx::Rect {
        interaction::event_dispatcher::MouseFrameGeometry::NonLocatedRectToRootFrame(self, rect)
            .expect("prepared child FrameView")
    }
}
impl PagePointerGeometry for ChildInputGeometry {
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
        self.Query(|_| ())
    }
    fn InteractionTextSettings(
        &self,
    ) -> Option<interaction::adapter::page_pointer_hit_test::InteractionTextSettings> {
        let (_, _, page) = self.Owners().ok()?;
        let settings = page.interaction_text_settings.borrow().clone();
        settings
    }
    fn ChildrenOrSiblingsAffectedByActiveOrHover(&self, node: u64) -> bool {
        let Ok((_, runtime, _)) = self.Owners() else {
            return false;
        };
        let result = runtime
            .style
            .borrow()
            .ChildrenOrSiblingsAffectedByActiveOrHover(
                runtime.document.borrow().GetDocument(),
                node,
            );
        result
    }
    fn HitTest(
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
        self.Owners().map_or(1.0, |(_, runtime, _)| {
            runtime.constraints.borrow().device_pixel_ratio as f32
        })
    }
    fn PageScaleFactor(&self) -> f32 {
        self.Owners()
            .map_or(1.0, |(_, _, page)| page.web_view_zoom.borrow().Scale())
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

impl interaction::event_dispatcher::MouseFrameGeometry for ChildInputGeometry {
    fn AcceptsLoadDrops(&self) -> bool {
        self.Owners()
            .is_ok_and(|(_, _, page)| page.client.borrow().AcceptsLoadDrops())
    }
    fn LoadDroppedURL(&self, url: &str, foreground: bool, user_activation: bool) {
        let Ok((_, _, page)) = self.Owners() else {
            return;
        };
        // DragController::PerformDrag navigates the local root, opening the
        // first dropped URL in the foreground and later URLs in background.
        interaction::event_dispatcher::MouseFrameGeometry::LoadDroppedURL(
            &PageMouseFrameGeometry(Rc::downgrade(&page)),
            url,
            foreground,
            user_activation,
        );
    }
    fn DragTargetStateChanged(&self) {
        if let Ok((frame, _, _)) = self.Owners() {
            // DOM's upload button already owns the source active-state style
            // mutation. Schedule that original child Document's lifecycle.
            frame.MarkNavigationDirty();
        }
    }
    fn ScheduleAutoscrollAnimation(&self) {
        if let Ok((_, _, page)) = self.Owners() {
            page.animation_begin_frame_requested.set(true);
        }
    }
    fn AutoscrollLayoutForNode(
        &self,
        node: u64,
    ) -> Option<Rc<dyn interaction::default::core::page::autoscroll_controller::AutoscrollLayout>>
    {
        let (frame, runtime, _) = self.Owners().ok()?;
        super::mouse_autoscroll::LayoutForChildNode(&frame, &runtime, node)
    }
    fn DragFrameToViewport(&self, point: gfx::Point) -> Option<gfx::Point> {
        let point = self.NonLocatedPointToRootFrame(point)?;
        let (_, _, page) = self.Owners().ok()?;
        let point = page
            .web_view_zoom
            .borrow()
            .RootFrameToViewport(gfx::PointF::new(point.x() as f32, point.y() as f32));
        Some(gfx::Point::new(
            point.x().round() as i32,
            point.y().round() as i32,
        ))
    }
    fn DragLayout(
        &self,
        node: u64,
    ) -> Option<layoutng_assembly::native_mouse_coordinates::NativeMouseCoordinateLayout> {
        let (frame, runtime, _) = self.Owners().ok()?;
        frame.EnsureFocusLayout(&runtime);
        let result = runtime.layout.borrow().MouseCoordinateLayout(node);
        result
    }
    fn DragBoundingBox(&self, node: u64) -> Option<gfx::Rect> {
        let (_, runtime, _) = self.Owners().ok()?;
        let rect = {
            let layout = runtime.layout.borrow();
            layout
                .MouseCoordinateLayout(node)?
                .AbsoluteBoundingBoxIncludingDescendants(&layout)
        };
        self.NonLocatedRectToRootFrame(rect)
    }
    fn SimulatedClickRootFrameCenter(
        &self,
        node: u64,
    ) -> Option<(foundation::PhysicalOffset, f32)> {
        let (frame, runtime, _) = self.Owners().ok()?;
        let (point, zoom) = {
            let layout = runtime.layout.borrow();
            layout
                .MouseCoordinateLayout(node)?
                .SimulatedClickRootFrameCenter(&layout)?
        };
        let point = frame.domain.upgrade()?.MapChildPoint(
            &frame,
            gfx::PointF::new(point.left.ToFloat(), point.top.ToFloat()),
        );
        Some((
            foundation::PhysicalOffset::new(
                LayoutUnit::FromFloatRound(point.x()),
                LayoutUnit::FromFloatRound(point.y()),
            ),
            zoom,
        ))
    }
    fn HasFrameView(&self) -> bool {
        self.Owners()
            .is_ok_and(|(_, runtime, _)| runtime.layout.borrow().Output().is_some())
    }
    fn HasContentLayout(&self) -> bool {
        self.HasFrameView()
    }
    fn LayoutZoomFactor(&self) -> Option<f32> {
        Some(
            self.Owners()
                .ok()?
                .1
                .constraints
                .borrow()
                .layout_zoom_factor as f32,
        )
    }
    fn DevicePixelRatio(&self) -> Option<f64> {
        Some(
            self.Owners()
                .ok()?
                .1
                .constraints
                .borrow()
                .device_pixel_ratio,
        )
    }
    fn LayoutViewportScrollOffset(&self) -> Option<(i32, i32)> {
        self.Owners()
            .ok()?
            .1
            .layout
            .borrow()
            .MouseLayoutViewportScrollOffsetInt()
    }
    fn HasLayoutObject(&self, node: u64) -> bool {
        self.Owners().is_ok_and(|(_, runtime, _)| {
            runtime
                .layout
                .borrow()
                .EventRegistryLayout(Some(node))
                .is_some()
        })
    }
    fn IsUserScrollable(&self, node: u64) -> bool {
        self.Owners().is_ok_and(|(_, runtime, _)| {
            runtime
                .layout
                .borrow()
                .Output()
                .and_then(|output| output.Node(node))
                .is_some_and(|node| node.FocusNavigationFacts().is_user_scrollable)
        })
    }
    fn IsCanvasLayoutObject(&self, node: u64) -> bool {
        self.Owners().is_ok_and(|(_, runtime, _)| {
            runtime
                .layout
                .borrow()
                .Output()
                .and_then(|output| output.Node(node))
                .is_some_and(|node| node.FocusNavigationFacts().is_canvas)
        })
    }
    fn ConvertFromRootFrame(&self, point: gfx::PointF) -> Option<gfx::PointF> {
        let (frame, _, _) = self.Owners().ok()?;
        frame
            .domain
            .upgrade()?
            .RootPointToFrameFloat(frame.state.Identity(), point)
    }
    fn RootFrameToViewport(&self, point: gfx::PointF) -> Option<gfx::PointF> {
        Some(
            self.Owners()
                .ok()?
                .2
                .web_view_zoom
                .borrow()
                .RootFrameToViewport(point),
        )
    }
    fn PointerLockPosition(&self) -> Option<(gfx::PointF, gfx::PointF)> {
        None
    }
    fn NonLocatedContextMenuCoordinates(&self) -> Option<interaction::adapter::page_nonlocated_context_menu_coordinates::NativeContextMenuFrameCoordinates>{
        let (_, _, page) = self.Owners().ok()?;
        let mut result =
            interaction::event_dispatcher::MouseFrameGeometry::NonLocatedContextMenuCoordinates(
                &PageMouseFrameGeometry(Rc::downgrade(&page)),
            )?;
        result.outermost_main_frame = false;
        Some(result)
    }
    fn NonLocatedRectFromRootFrame(&self, rect: gfx::Rect) -> Option<gfx::Rect> {
        use interaction::event_dispatcher::MouseFrameGeometry;
        let points = [
            (rect.x(), rect.y()),
            (rect.right(), rect.y()),
            (rect.x(), rect.bottom()),
            (rect.right(), rect.bottom()),
        ]
        .map(|(x, y)| self.ConvertFromRootFrame(gfx::PointF::new(x as f32, y as f32)))
        .into_iter()
        .collect::<Option<Vec<_>>>()?;
        let left = points
            .iter()
            .map(|point| point.x())
            .fold(f32::INFINITY, f32::min)
            .floor();
        let top = points
            .iter()
            .map(|point| point.y())
            .fold(f32::INFINITY, f32::min)
            .floor();
        let right = points
            .iter()
            .map(|point| point.x())
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil();
        let bottom = points
            .iter()
            .map(|point| point.y())
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil();
        Some(gfx::Rect::new(
            gfx::Point::new(left as i32, top as i32),
            gfx::Size::new((right - left) as i32, (bottom - top) as i32),
        ))
    }
    fn NonLocatedPointToRootFrame(&self, point: gfx::Point) -> Option<gfx::Point> {
        let (frame, _, _) = self.Owners().ok()?;
        let result = frame
            .domain
            .upgrade()?
            .MapChildPoint(&frame, gfx::PointF::new(point.x() as f32, point.y() as f32));
        Some(gfx::Point::new(
            result.x().round() as i32,
            result.y().round() as i32,
        ))
    }
    fn NonLocatedRectToRootFrame(&self, rect: gfx::Rect) -> Option<gfx::Rect> {
        let (frame, _, _) = self.Owners().ok()?;
        let rect = frame.domain.upgrade()?.MapChildRect(
            &frame,
            gfx::RectF::new(
                gfx::PointF::new(rect.x() as f32, rect.y() as f32),
                gfx::SizeF::new(rect.width() as f32, rect.height() as f32),
            ),
        );
        Some(gfx::Rect::new(
            gfx::Point::new(rect.x().floor() as i32, rect.y().floor() as i32),
            gfx::Size::new(rect.width().ceil() as i32, rect.height().ceil() as i32),
        ))
    }
}

impl LocalFrameRuntime {
    pub(super) fn BindNativeInput(
        self: &Rc<Self>,
        frame: &Rc<LocalFrame>,
        engine: &interaction::Interaction<'static>,
    ) {
        super::select_control_host::ResidentSelectControlHost::Child(frame, self, engine);
        let geometry = ChildInputGeometry::new(frame, self);
        engine.SetMouseFrameGeometry(Some(geometry.clone()));
        engine.BindPointerInput(
            geometry,
            Rc::new(
                super::pointer_scroll_controls::PagePointerScrollControlsProvider::ForChild(
                    frame,
                    self,
                    engine.SourceInputOwners().mouse.clone(),
                ),
            ),
        );
    }
    pub(super) fn CompleteNoScriptFormSubmissions(
        self: &Rc<Self>,
        frame: &Rc<LocalFrame>,
        engine: &interaction::Interaction<'_>,
    ) {
        if frame
            .navigation
            .borrow()
            .window
            .as_ref()
            .is_some_and(|window| Rc::ptr_eq(&window.document, &self.document))
        {
            return;
        }
        let Some(owner) = self.frame_interaction.get() else {
            return;
        };
        loop {
            let Some(submission) = owner.pending_submissions.borrow_mut().pop_front() else {
                break;
            };
            if !frame.IsCurrentRuntime(self) {
                break;
            }
            frame.EnsureFocusLayout(self);
            let document = self.document.Handle();
            let prepared = {
                let layout = self.layout.borrow();
                layout.Output().map(|output| {
                    engine.ResumeFormSubmission(
                        &document,
                        submission.token,
                        interaction::adapter::page_editing_context::PreparedEditingGeometry::new(
                            output,
                        ),
                    )
                })
            };
            match prepared {
                Some(Ok(Some(interaction::input_type::PrepareFormEntryListResult::Ready(
                    pending,
                )))) => engine.FinishFormSubmission(&document, pending, submission.parameters),
                Some(Ok(None)) => {}
                Some(Err(error)) => frame.ApplyFrameInteractionEffect(
                    self,
                    &owner.pending_submissions,
                    interaction::InteractionEffect::FormSubmissionError(error),
                ),
                _ => {
                    if let Some(page) = frame
                        .domain
                        .upgrade()
                        .and_then(|domain| domain.page.upgrade())
                    {
                        page.client
                            .borrow_mut()
                            .DidFail("form submission has no prepared child layout output");
                    }
                }
            }
        }
    }
}

impl LocalFrame {
    pub(super) fn ApplyFrameInteractionEffect(
        &self,
        runtime: &Rc<LocalFrameRuntime>,
        pending: &Rc<RefCell<std::collections::VecDeque<interaction::PreparedFormSubmission>>>,
        effect: interaction::InteractionEffect,
    ) {
        let Some(domain) = self.domain.upgrade() else {
            return;
        };
        let Some(page) = domain.page.upgrade() else {
            return;
        };
        match effect {
            interaction::InteractionEffect::MouseClickActivation { target_node_id } => {
                let document = runtime
                    .document
                    .borrow()
                    .GetDocument()
                    .RootHandle()
                    .Identity();
                page.client
                    .borrow_mut()
                    .DidDispatchClick(document, target_node_id);
            }
            interaction::InteractionEffect::SignalCloseWatchers => {
                domain.WithInputListeners(self.state.Identity(), |engine, document| {
                    engine
                        .SignalCloseWatchers(document)
                        .unwrap_or_else(|error| std::panic::panic_any(error))
                });
            }
            interaction::InteractionEffect::NotifyHistoryUserActivation(now) => {
                domain.NotifyHistoryUserActivation(self.state.Identity(), now)
            }
            interaction::InteractionEffect::ConsumeHistoryUserActivation => {
                domain.ConsumeHistoryUserActivation()
            }
            interaction::InteractionEffect::PrepareFormSubmission(submission) => {
                pending.borrow_mut().push_back(submission)
            }
            interaction::InteractionEffect::SubmitForm {
                parameters,
                entries,
            } => {
                let url = runtime
                    .document
                    .borrow()
                    .GetDocument()
                    .DocumentURL()
                    .to_owned();
                match crate::form_submission::BuildRequest(&url, &parameters, &entries) {
                    Ok(Some(request)) => domain.RouteTargetNavigation(
                        self.state.Identity(),
                        request,
                        parameters.scheduling.clone(),
                    ),
                    Ok(None) => {}
                    Err(error) => page.client.borrow_mut().DidFail(&error.to_string()),
                }
            }
            interaction::InteractionEffect::FormSubmissionError(error) => {
                if !matches!(
                    error,
                    interaction::input_type::FormDataAdmission::StaleSubmission { .. }
                ) {
                    page.client
                        .borrow_mut()
                        .DidFail(&format!("form submission: {error:?}"));
                }
            }
            interaction::InteractionEffect::CaptureMouseEventsToWidget(capture) => {
                page.client.borrow_mut().CaptureMouseEventsToWidget(capture)
            }
        }
    }
}

impl LocalFrameDomain {
    /// The host close-listener endpoint is registered exactly while at least
    /// one watcher is enabled. Clone the endpoint before synchronous dispatch.
    pub(super) fn SignalCloseWatchers(&self, token: u64) -> bool {
        let Some(document) = self.Document(token) else {
            return false;
        };
        let listener = document
            .borrow()
            .GetDocument()
            .PopupLifecycleState()
            .close_watcher_stack
            .close_listener
            .clone();
        if let Some(listener) = listener {
            listener();
            true
        } else {
            false
        }
    }
    // Frame::NotifyUserActivationInFrameTree: ancestors first, then every
    // accessible same-origin local frame. FrameTree remains the sole owner.
    pub(super) fn NotifyHistoryUserActivation(
        &self,
        source: u64,
        now: interaction::input::base::time::time::TimeTicks,
    ) {
        let notify = |token| {
            if let (Some(engine), Some(document)) =
                (self.FocusInteraction(token), self.Document(token))
            {
                engine.NotifyHistoryUserActivationForFrame(&document.Handle(), now);
            }
        };
        let mut ancestor = Some(source);
        while let Some(token) = ancestor {
            notify(token);
            ancestor = if token == self.root {
                None
            } else {
                self.Frame(token).map(|frame| frame.parent)
            };
        }
        let Some(source_document) = self.Document(source) else {
            return;
        };
        let source_origin = source_document
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .clone();
        let tokens: Vec<_> = std::iter::once(self.root)
            .chain(self.frames.borrow().keys().copied())
            .collect();
        for token in tokens {
            if let Some(document) = self.Document(token) {
                let accessible = foundation::security::CanAccessSecurityOrigin(
                    source_origin.as_ref(),
                    document
                        .borrow()
                        .GetDocument()
                        .SecurityContext()
                        .Origin()
                        .as_ref(),
                );
                if accessible {
                    notify(token);
                }
            }
        }
    }
    // LocalFrame::ConsumeHistoryUserActivation consumes each live local frame,
    // preserving the separate UserActivationState and per-window stack owner.
    pub(super) fn ConsumeHistoryUserActivation(&self) {
        let tokens: Vec<_> = std::iter::once(self.root)
            .chain(self.frames.borrow().keys().copied())
            .collect();
        for token in tokens {
            if let Some(interaction) = self.FocusInteraction(token) {
                interaction.ConsumeHistoryUserActivationForFrame();
            }
        }
    }
    pub(super) fn WithInputListeners<T>(
        &self,
        token: u64,
        run: impl FnOnce(&interaction::Interaction<'_>, &interaction::InteractionDocument) -> T,
    ) -> Option<T> {
        let engine = self.FocusInteraction(token)?;
        let document = self.Document(token)?.Handle();
        let window = self
            .Frame(token)
            .and_then(|frame| frame.navigation.borrow().window.clone())
            .filter(|window| Rc::ptr_eq(&window.document.Handle(), &document));
        let owner = if token == self.root {
            self.script_factory
                .get()
                .and_then(|factory| factory.RootInteraction())
        } else {
            window.as_ref().map(|window| window.interaction.clone())
        };
        let Some(owner) = owner else {
            return Some(run(&engine, &document));
        };
        let script = self.script_factory.get()?.script.upgrade()?;
        let realm = window
            .as_ref()
            .map(|window| window.realm.clone())
            .unwrap_or_else(|| script.borrow().Realm().clone());
        let mut script = script.borrow_mut();
        let mut run = Some(run);
        let mut result = None;
        let invocation = script.RuntimeMut().WithHostRuntime(&realm, &mut |runtime| {
            result = Some(owner.WithScriptRuntime(runtime, |scoped| {
                run.take().expect("one child input invocation")(scoped, &document)
            }));
        });
        invocation.unwrap_or_else(|error| std::panic::panic_any(error));
        result
    }
    /// Complete explicit document effects after the root EventHandler has
    /// synchronously routed input. This lends the originating realm only;
    /// target selection and event/default processing stay in Interaction.
    pub(super) fn CompleteChildInputEffects(&self) {
        let frames = self.frames.borrow().values().cloned().collect::<Vec<_>>();
        for frame in frames {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            let owner = runtime.frame_interaction.get().expect("child input owner");
            if owner.pending_submissions.borrow().is_empty() {
                continue;
            }
            let window = frame
                .navigation
                .borrow()
                .window
                .clone()
                .filter(|window| Rc::ptr_eq(&window.document, &runtime.document));
            if let Some(window) = window {
                self.WithInputListeners(frame.state.Identity(), |engine, _| {
                    window.interaction.CompleteFormSubmissions(engine)
                });
            } else {
                runtime.CompleteNoScriptFormSubmissions(&frame, &owner.engine);
            }
        }
    }
    pub(super) fn RootPointToFrameFloat(
        &self,
        token: u64,
        point: gfx::PointF,
    ) -> Option<gfx::PointF> {
        if token == self.root {
            return Some(point);
        }
        let frame = self.Frame(token)?;
        let point = self.RootPointToFrameFloat(frame.parent, point)?;
        self.WithParentLayout(&frame, |layout| {
            layout
                .Output()?
                .Node(frame.state.Owner()?.Id())?
                .DocumentPointToEmbeddedContent(point)
        })
    }
    pub(super) fn HitInputTarget(
        &self,
        point: Offset,
    ) -> io::Result<Option<(Rc<LocalFrame>, u64)>> {
        let page = self
            .page
            .upgrade()
            .ok_or_else(|| io::Error::other("detached local root"))?;
        let zoom = page.constraints.borrow().layout_zoom_factor;
        let location = HitTestLocation::FromPhysicalOffset(foundation::PhysicalOffset::new(
            LayoutUnit::from_f64(point.x * zoom),
            LayoutUnit::from_f64(point.y * zoom),
        ));
        let hit = super::pointer_geometry::PagePointerGeometryProvider(Rc::downgrade(&page))
            .HitTest(
                location,
                interaction::hit_test::core::layout::hit_test_request::RequestType::kReadOnly
                    as u32,
            )
            .map_err(|error| io::Error::other(format!("local frame input hit: {error:?}")))?;
        let Some(node) = hit.result.InnerNode() else {
            return Ok(None);
        };
        let Some(id) = node.DOMNodeId() else {
            return Ok(None);
        };
        let identity = node.DOMIdentity();
        Ok(self
            .frames
            .borrow()
            .values()
            .find(|frame| {
                frame.runtime.borrow().as_ref().is_some_and(|runtime| {
                    let owner = runtime.document.borrow();
                    let document = owner.GetDocument();
                    document
                        .FindNodeById(id)
                        .is_some_and(|node| document.NodeHandle(node).Identity() == identity)
                })
            })
            .cloned()
            .map(|frame| (frame, id)))
    }
    pub(super) fn TakeChildUnbufferedInputRequest(&self) -> bool {
        let mut requested = false;
        for frame in self.frames.borrow().values() {
            if let Some(engine) = self.FocusInteraction(frame.state.Identity()) {
                requested |= engine.TakeUnbufferedInputRequest();
            }
        }
        requested
    }
}
