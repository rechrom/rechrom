//! Leaf assembly of native scroll-control queries and source-owned effects.
use super::*;
use foundation::{gfx, Persistent};
use interaction::adapter::page_pointer_scroll_controls::{
    PagePointerScrollControls, ScrollbarTimerKind, ScrollbarTimerTask,
};
use interaction::scroll::core::scroll::scrollbar::{ScrollbarGesture, ScrollbarThumbUse};
use layoutng_assembly::{
    internal::scrollbar::Scrollbar, native_pointer_scroll_controls::NativePointerResizeArea,
};
struct OverlayTimer {
    generation: u64,
    token: foundation::popup_types::PopupTaskId,
    area: NativePointerResizeArea,
}
pub(super) struct PagePointerScrollControlsProvider {
    page: Weak<PageState>,
    child: Option<(
        Weak<super::local_frame::LocalFrame>,
        Weak<super::local_frame::LocalFrameRuntime>,
    )>,
    mouse: Rc<interaction::core::input::mouse_event_manager::MouseEventManagerState>,
    overlay: RefCell<HashMap<usize, OverlayTimer>>,
    generation: Cell<u64>,
}
impl PagePointerScrollControlsProvider {
    pub(super) fn new(
        page: Weak<PageState>,
        mouse: Rc<interaction::core::input::mouse_event_manager::MouseEventManagerState>,
    ) -> Self {
        Self {
            page,
            child: None,
            mouse,
            overlay: RefCell::new(HashMap::new()),
            generation: Cell::new(0),
        }
    }
    pub(super) fn ForChild(
        frame: &Rc<super::local_frame::LocalFrame>,
        runtime: &Rc<super::local_frame::LocalFrameRuntime>,
        mouse: Rc<interaction::core::input::mouse_event_manager::MouseEventManagerState>,
    ) -> Self {
        let mut provider = Self::new(frame.domain.upgrade().unwrap().page.Downgrade(), mouse);
        provider.child = Some((Rc::downgrade(frame), Rc::downgrade(runtime)));
        provider
    }
    fn state(&self) -> PointerFrameState {
        let page = self
            .page
            .upgrade()
            .expect("live pointer scroll-control Page");
        let child = self.child.as_ref().map(|(frame, runtime)| {
            let frame = frame.upgrade().expect("retained pointer Frame");
            let runtime = runtime.upgrade().expect("retained pointer Document");
            assert!(
                frame.IsCurrentRuntime(&runtime),
                "pointer runtime was retired"
            );
            (frame, runtime)
        });
        PointerFrameState { page, child }
    }
    fn TimerTask(&self, task: ScrollbarTimerTask) -> super::popup_tasks::PageInteractionTask {
        if let Some((frame, _)) = &self.child {
            let frame = frame.upgrade().expect("retained pointer Frame");
            super::popup_tasks::PageInteractionTask::ChildScrollbar {
                frame: frame.state.Identity(),
                generation: frame.generation.get(),
                task,
            }
        } else {
            super::popup_tasks::PageInteractionTask::Scrollbar(task)
        }
    }
}
impl PagePointerScrollControlsProvider {
    fn PointFromRoot(&self, point: gfx::Point) -> gfx::Point {
        let Some((frame, _)) = &self.child else {
            return point;
        };
        let frame = frame.upgrade().expect("retained pointer Frame");
        let point = frame
            .domain
            .upgrade()
            .expect("retained FrameTree")
            .RootPointToFrameFloat(
                frame.state.Identity(),
                gfx::PointF::new(point.x() as f32, point.y() as f32),
            )
            .expect("prepared embedded owner coordinates");
        gfx::Point::new(point.x().round() as i32, point.y().round() as i32)
    }
}
impl PagePointerScrollControls for PagePointerScrollControlsProvider {
    fn MouseInScrollbarFrameRect(&self, owner: &Persistent<Scrollbar>) -> bool {
        let point = self.mouse.LastKnownMousePositionInRootFrame();
        let point = self.PointFromRoot(gfx::Point::new(
            point.x().floor() as i32,
            point.y().floor() as i32,
        ));
        let bar = unsafe { &*owner.Get() };
        bar.IsConnectedToScrollableArea() && bar.FrameRect().ContainsPoint(point)
    }
    fn MonotonicTimeMicros(&self) -> i64 {
        self.state()
            .window_performance_timing
            .MonotonicElapsed()
            .as_micros()
            .min(i64::MAX as u128) as i64
    }
    fn ScrollbarInteraction(
        &self,
        owner: &Persistent<Scrollbar>,
        event: layoutng_assembly::native_pointer_scroll_controls::NativeScrollbarInteraction,
    ) -> io::Result<()> {
        let state = self.state();
        // A Browser Page is an ordinary LocalFrame, including browser chrome.
        // It is not Blink's separate PagePopup widget.
        let update = state.Layout().borrow().PointerScrollbarInteraction(
            unsafe { &*owner.Get() },
            event,
            false,
        );
        let Some((area, update)) = update else {
            return Ok(());
        };
        let identity = area.Identity();
        if update.stop_timer || update.schedule_after_us.is_some() {
            if let Some(timer) = self.overlay.borrow_mut().remove(&identity) {
                state.popup_tasks.Cancel(timer.token);
            }
        }
        if let Some(delay) = update.schedule_after_us {
            let generation = self
                .generation
                .get()
                .checked_add(1)
                .expect("overlay timer generation");
            self.generation.set(generation);
            let document = state.page.document.borrow().GetDocument().RootHandle();
            let token = state.popup_tasks.Post(
                document,
                delay as f64 / 1_000_000.,
                self.TimerTask(ScrollbarTimerTask {
                    scrollbar_identity: identity,
                    generation,
                    kind: ScrollbarTimerKind::OverlayFade,
                }),
            );
            self.overlay.borrow_mut().insert(
                identity,
                OverlayTimer {
                    generation,
                    token,
                    area,
                },
            );
        }
        state.dirty.set(true);
        Ok(())
    }
    fn RunOverlayFadeTimer(&self, task: ScrollbarTimerTask) {
        let timer = {
            let mut timers = self.overlay.borrow_mut();
            if !timers
                .get(&task.scrollbar_identity)
                .is_some_and(|timer| timer.generation == task.generation)
            {
                return;
            }
            timers
                .remove(&task.scrollbar_identity)
                .expect("live overlay task")
        };
        let state = self.state();
        state
            .Layout()
            .borrow()
            .PointerScrollbarFadeTimerFired(&timer.area);
        state.dirty.set(true);
    }
    fn PublishMacScrollbarAnimation(
        &self,
        owner: &Persistent<Scrollbar>,
        thumb: f32,
        track: f32,
        width: i32,
        thumb_dirty: bool,
        track_dirty: bool,
        hidden: Option<bool>,
    ) {
        let state = self.state();
        if state.Layout().borrow().PublishPointerMacScrollbarAnimation(
            unsafe { &*owner.Get() },
            thumb,
            track,
            width,
            thumb_dirty,
            track_dirty,
            hidden,
        ) {
            // The native ScrollableArea now owns newer painted alpha/width
            // values. Route that typed source change through the ordinary
            // paint-only lifecycle; `dirty` alone would let Page retain the
            // previous artifact because no DOM style impact accompanies a
            // platform scrollbar animation.
            state.Document().borrow_mut().InvalidatePaint();
            state.dirty.set(true);
        }
    }
    fn ScrollbarPaintArtifactCompositorNeedsUpdate(&self) {
        self.state().dirty.set(true);
    }
    fn TargetThumbGeometry(&self, owner: &Persistent<Scrollbar>) -> (i32, i32) {
        let state = self.state();
        let scrollbar = unsafe { &*owner.Get() };
        let input = scrollbar
            .InputScrollState()
            .expect("connected source scrollbar");
        let target = if let Some((_, runtime)) = &state.child {
            runtime.UserScrollAnimationTarget(input.area_identity)
        } else {
            state
                .scrollbar_scroll_runtime
                .borrow()
                .upgrade()
                .and_then(|runtime| runtime.animators.Get(input.area_identity))
                .and_then(|animator| animator.Snapshot().desired_target_offset)
        }
        .unwrap_or(input.current_offset);
        let geometry = state
            .Layout()
            .borrow()
            .PointerScrollbarTargetThumbGeometry(scrollbar, target);
        geometry
    }
    fn StartScrollbarTimer(
        &self,
        delay: std::time::Duration,
        task: ScrollbarTimerTask,
    ) -> io::Result<u64> {
        let state = self.state();
        let document = state.page.document.borrow().GetDocument().RootHandle();
        Ok(state
            .popup_tasks
            .Post(document, delay.as_secs_f64(), self.TimerTask(task))
            .0)
    }
    fn CancelScrollbarTimer(&self, token: u64) {
        if let Some(state) = self.page.upgrade() {
            state
                .popup_tasks
                .Cancel(foundation::popup_types::PopupTaskId(token));
        }
    }
    fn ScrollbarPointFromRootFrame(
        &self,
        owner: &Persistent<Scrollbar>,
        point: gfx::Point,
    ) -> gfx::Point {
        let state = self.state();
        // Each Page is its own local root. FrameView conversion is identity;
        // the resident scrollbar owner then performs its box-local conversion.
        let point = state.Layout().borrow().PointerScrollbarPointFromRootFrame(
            unsafe { &*owner.Get() },
            self.PointFromRoot(point),
        );
        point
    }
    fn InputEventsScaleFactor(&self) -> f32 {
        // LocalFrameView::InputEventsScaleFactor, cc:2082-2085: the
        // current Page has an inactive VisualViewport (scale 1) and no device
        // emulation transform. CSS/layout zoom is deliberately not this scale.
        1.0
    }
    fn CountScrollbarUse(&self, feature: ScrollbarThumbUse) {
        use dom::use_counter::WebFeature;
        let feature = match feature {
            ScrollbarThumbUse::VerticalMouse => {
                WebFeature::kVerticalScrollbarThumbScrollingWithMouse
            }
            ScrollbarThumbUse::VerticalTouch => {
                WebFeature::kVerticalScrollbarThumbScrollingWithTouch
            }
            ScrollbarThumbUse::HorizontalMouse => {
                WebFeature::kHorizontalScrollbarThumbScrollingWithMouse
            }
            ScrollbarThumbUse::HorizontalTouch => {
                WebFeature::kHorizontalScrollbarThumbScrollingWithTouch
            }
            ScrollbarThumbUse::ReversedButtonDirection => {
                WebFeature::kScrollbarUseScrollbarButtonReversedDirection
            }
            ScrollbarThumbUse::VerticalButton => WebFeature::kScrollbarUseVerticalScrollbarButton,
            ScrollbarThumbUse::HorizontalButton => {
                WebFeature::kScrollbarUseHorizontalScrollbarButton
            }
            ScrollbarThumbUse::VerticalTrack => WebFeature::kScrollbarUseVerticalScrollbarTrack,
            ScrollbarThumbUse::HorizontalTrack => WebFeature::kScrollbarUseHorizontalScrollbarTrack,
        };
        self.state()
            .Document()
            .borrow()
            .GetDocument()
            .CountUse(feature);
    }
    fn InjectScrollbarGesture(&self, request: ScrollbarGesture) -> io::Result<()> {
        self.state().InjectScrollbarGesture(request)
    }
    fn ResizeAreaForNode(&self, node: u64) -> Option<NativePointerResizeArea> {
        let state = self.state();
        state.EnsureLayoutOutput();
        let area = state.Layout().borrow().PointerResizeAreaForNode(node);
        area
    }
    fn IsAbsolutePointInResizeControl(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
    ) -> bool {
        self.state()
            .Layout()
            .borrow()
            .PointerResizeAreaContainsTouchPoint(area, self.PointFromRoot(point))
    }
    fn SetInResizeMode(&self, area: &NativePointerResizeArea, value: bool) {
        self.state()
            .Layout()
            .borrow()
            .SetPointerResizeAreaInResizeMode(area, value);
    }
    fn InResizeMode(&self, area: &NativePointerResizeArea) -> bool {
        self.state()
            .Layout()
            .borrow()
            .PointerResizeAreaInResizeMode(area)
    }
    fn HasLayerAndLayoutBox(&self, area: &NativePointerResizeArea) -> bool {
        self.state()
            .Layout()
            .borrow()
            .PointerResizeAreaHasLayerAndLayoutBox(area)
    }
    fn InitializeResizeTransform(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
    ) -> gfx::Transform {
        self.state()
            .Layout()
            .borrow()
            .InitializePointerResizeTransform(area, self.PointFromRoot(point))
    }
    fn Resize(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
        transform: &gfx::Transform,
    ) -> io::Result<()> {
        let state = self.state();
        let plan =
            state
                .Layout()
                .borrow()
                .PlanPointerResize(area, self.PointFromRoot(point), transform);
        let Some(plan) = plan else {
            return Ok(());
        };
        // Source Element::SetInlineStyleProperty owns declaration mutation.
        // Native Layout supplies only typed pixel values, never CSS text.
        for (property, value) in [
            (foundation::CSSPropertyID::kWidth, plan.width_px),
            (foundation::CSSPropertyID::kHeight, plan.height_px),
        ] {
            let Some(value) = value else {
                continue;
            };
            let result = {
                let owner = state.Document().borrow();
                let document = owner.GetDocument();
                let index = document.FindNodeById(plan.node_id).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "resize element detached")
                })?;
                let value = style::production_css_value::numeric(
                    value as f64,
                    style::css_primitive_value::UnitType::kPixels,
                );
                style::SetInlineCSSPropertyValue(
                    &document.Node(index).InlineStyle(),
                    property,
                    value,
                    document.GetCompatibilityMode()
                        == dom::persistent_document::CompatibilityMode::kNoQuirks,
                )
            };
            let (inline_style, independent) = match result {
                style::CSSInlineDeclarationMutation::Changed(value) => (value, false),
                style::CSSInlineDeclarationMutation::ChangedIndependent(value) => (value, true),
                style::CSSInlineDeclarationMutation::Unchanged => continue,
            };
            state.ApplyDOMMutation(dom::dom_mutation::DOMMutation {
                mutation_type: DOMMutationType::kSetAttribute,
                target_node_id: plan.node_id,
                name: "style".into(),
                value: inline_style.SourceText().to_owned(),
                inline_style: Some(inline_style),
                inline_style_independent: independent,
                ..Default::default()
            });
        }
        // Source cc:2513 Document::UpdateStyleAndLayout(kSizeChange). All
        // Layout borrows above ended before publishing DOM changes or flushing.
        state.EnsureLayoutOutput();
        Ok(())
    }
}

/// The original per-Document owners, borrowed through one pointer-control
/// provider. Shared host timing/task services stay on the containing Page.
struct PointerFrameState {
    page: Rc<PageState>,
    child: Option<(
        Rc<super::local_frame::LocalFrame>,
        Rc<super::local_frame::LocalFrameRuntime>,
    )>,
}
impl std::ops::Deref for PointerFrameState {
    type Target = PageState;
    fn deref(&self) -> &PageState {
        &self.page
    }
}
impl PointerFrameState {
    fn Layout(&self) -> &RefCell<layoutng_assembly::layout_engine::LayoutEngine> {
        self.child
            .as_ref()
            .map_or(&self.page.layout_engine, |(_, runtime)| &runtime.layout)
    }
    fn Document(&self) -> &Rc<DocumentEngine> {
        self.child
            .as_ref()
            .map_or(&self.page.document, |(_, runtime)| &runtime.document)
    }
    fn EnsureLayoutOutput(&self) {
        if let Some((frame, runtime)) = &self.child {
            frame.EnsureFocusLayout(runtime);
        } else {
            self.page.EnsureLayoutOutput();
        }
    }
    fn ApplyDOMMutation(&self, mutation: dom::dom_mutation::DOMMutation) {
        if let Some((frame, runtime)) = &self.child {
            runtime
                .document
                .ApplyDOMMutation(&mutation)
                .unwrap_or_else(|error| std::panic::panic_any(error));
            frame.MarkNavigationDirty();
        } else {
            self.page.ApplyMutation(PageMutation::DOMMutation(mutation));
        }
    }
}
