//! PointerManager leaves over canonical Scrollbar/ScrollableArea owners.
//! Scrollbar algorithms stay in InteractionScroll; Layout owns resize used
//! values and geometry. Page supplies only scoped native calls and effects.
use crate::core::input::pointer_event_manager::{
    PointerResizeArea, PointerResizeTransform, PointerScrollbar,
};
use foundation::{gfx, Persistent};
use interaction_input::public::common::input::{
    web_gesture_event::WebGestureEvent, web_input_event::Type, web_mouse_event::WebMouseEvent,
    web_pointer_event::WebPointerEvent,
};
use interaction_scroll::core::scroll::scrollbar::{
    self, ScrollOffsets, ScrollbarGesture, ScrollbarGestureSource, ScrollbarPointerSource,
    ScrollbarThumbUse, ThumbGeometry,
};
use interaction_scroll::core::scroll::{scrollbar_animation as animation, scrollbar_theme};
use layoutng_assembly::{
    internal::{
        scrollbar::Scrollbar, scrollbar_orientation::ScrollbarOrientation,
        scrollbar_part::ScrollbarPart,
    },
    native_pointer_scroll_controls::NativePointerResizeArea,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    rc::Rc,
    time::Duration,
};
#[derive(Clone, Copy)]
pub struct ScrollbarTimerTask {
    pub scrollbar_identity: usize,
    pub generation: u64,
    pub kind: ScrollbarTimerKind,
}
#[derive(Clone, Copy)]
pub enum ScrollbarTimerKind {
    Autoscroll,
    Mac(animation::Timer),
    OverlayFade,
}
/// Each method enters the existing native Layout scope or publishes an effect;
/// no method owns an input algorithm or a projected layout/scroll tree.
pub trait PagePointerScrollControls {
    fn MouseInScrollbarFrameRect(&self, scrollbar: &Persistent<Scrollbar>) -> bool;
    fn MonotonicTimeMicros(&self) -> i64;
    fn ScrollbarInteraction(
        &self,
        scrollbar: &Persistent<Scrollbar>,
        event: layoutng_assembly::native_pointer_scroll_controls::NativeScrollbarInteraction,
    ) -> io::Result<()>;
    fn RunOverlayFadeTimer(&self, task: ScrollbarTimerTask);
    fn PublishMacScrollbarAnimation(
        &self,
        scrollbar: &Persistent<Scrollbar>,
        thumb: f32,
        track: f32,
        width: i32,
        thumb_dirty: bool,
        track_dirty: bool,
        hidden: Option<bool>,
    );
    fn ScrollbarPaintArtifactCompositorNeedsUpdate(&self);
    fn TargetThumbGeometry(&self, scrollbar: &Persistent<Scrollbar>) -> (i32, i32);
    fn StartScrollbarTimer(&self, delay: Duration, task: ScrollbarTimerTask) -> io::Result<u64>;
    fn CancelScrollbarTimer(&self, token: u64);
    fn ScrollbarPointFromRootFrame(
        &self,
        scrollbar: &Persistent<Scrollbar>,
        point: gfx::Point,
    ) -> gfx::Point;
    fn InputEventsScaleFactor(&self) -> f32;
    fn CountScrollbarUse(&self, feature: ScrollbarThumbUse);
    fn InjectScrollbarGesture(&self, request: ScrollbarGesture) -> io::Result<()>;
    fn ResizeAreaForNode(&self, node: u64) -> Option<NativePointerResizeArea>;
    fn IsAbsolutePointInResizeControl(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
    ) -> bool;
    fn SetInResizeMode(&self, area: &NativePointerResizeArea, value: bool);
    fn InResizeMode(&self, area: &NativePointerResizeArea) -> bool;
    fn HasLayerAndLayoutBox(&self, area: &NativePointerResizeArea) -> bool;
    fn InitializeResizeTransform(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
    ) -> gfx::Transform;
    fn Resize(
        &self,
        area: &NativePointerResizeArea,
        point: gfx::Point,
        transform: &gfx::Transform,
    ) -> io::Result<()>;
}
/// Strong retention represents the original native object's wrapper identity.
/// PointerManager's capture members are Weak; per-hit wrappers would otherwise
/// expire between down and move. Clear this registry at source Clear/dispose.
pub struct PagePointerScrollControlOwners {
    controls: Rc<dyn PagePointerScrollControls>,
    scrollbars: RefCell<HashMap<usize, Rc<NativePointerScrollbar>>>,
    areas: RefCell<HashMap<usize, Rc<NativePointerResizeOwner>>>,
}
impl PagePointerScrollControlOwners {
    pub fn new(controls: Rc<dyn PagePointerScrollControls>) -> Self {
        Self {
            controls,
            scrollbars: Default::default(),
            areas: Default::default(),
        }
    }
    pub fn Clear(&self) {
        for scrollbar in self.scrollbars.borrow().values() {
            scrollbar::StopTimerIfNeeded(scrollbar.as_ref());
            for (_, (_, token)) in scrollbar.mac_tokens.borrow_mut().drain() {
                self.controls.CancelScrollbarTimer(token);
            }
        }
        self.scrollbars.borrow_mut().clear();
        self.areas.borrow_mut().clear();
    }
    pub fn Scrollbar(&self, owner: Persistent<Scrollbar>) -> Rc<dyn PointerScrollbar> {
        self.NativeScrollbar(owner)
    }
    fn NativeScrollbar(&self, owner: Persistent<Scrollbar>) -> Rc<NativePointerScrollbar> {
        self.scrollbars
            .borrow_mut()
            .entry(owner.Get() as usize)
            .or_insert_with(|| {
                Rc::new(NativePointerScrollbar {
                    owner,
                    controls: self.controls.clone(),
                    error: RefCell::new(None),
                    timer_token: Cell::new(None),
                    mac_animation: RefCell::new(None),
                    mac_tokens: RefCell::new(HashMap::new()),
                })
            })
            .clone()
    }
    pub fn HandleGestureTapOrPress(
        &self,
        owner: Persistent<Scrollbar>,
        event: &WebGestureEvent,
    ) -> io::Result<bool> {
        let owner = self.NativeScrollbar(owner);
        let handled = scrollbar::HandleGestureTapOrPress(owner.as_ref(), event);
        let error = owner.error.borrow_mut().take();
        match error {
            Some(error) => Err(error),
            None => Ok(handled),
        }
    }
    pub fn EventHandlerScrollbar(
        &self,
        owner: Persistent<Scrollbar>,
    ) -> Rc<dyn crate::core::input::event_handler::EventHandlerScrollbar> {
        self.NativeScrollbar(owner)
    }
    /// Captured EventHandler results reuse the original overflow-control owner;
    /// converting back must never manufacture a different Scrollbar identity.
    pub(crate) fn HitTestScrollbar(
        &self,
        scrollbar: &Rc<dyn crate::core::input::event_handler::EventHandlerScrollbar>,
    ) -> Option<interaction_hit_test::core::layout::hit_test_result::ScrollbarRef> {
        let native = self.scrollbars.borrow().values().find_map(|native| {
            let candidate: Rc<dyn crate::core::input::event_handler::EventHandlerScrollbar> =
                native.clone();
            Rc::ptr_eq(&candidate, scrollbar).then(|| native.owner.clone())
        })?;
        Some(Rc::new(EventHandlerHitScrollbar(native)))
    }
    pub fn HandleNativeActivities(
        &self,
        activities: Vec<
            layoutng_assembly::internal::layout_scrollable_area::NativeScrollbarActivity,
        >,
    ) -> io::Result<()> {
        use layoutng_assembly::internal::layout_scrollable_area::NativeScrollbarActivityKind as Kind;
        for activity in activities {
            let owner = self.NativeScrollbar(activity.scrollbar);
            if matches!(activity.kind, Kind::Removed) {
                for (_, (_, token)) in owner.mac_tokens.borrow_mut().drain() {
                    self.controls.CancelScrollbarTimer(token);
                }
                owner.mac_animation.borrow_mut().take();
                continue;
            }
            if !owner.native().IsOverlayScrollbar() {
                continue;
            }
            if owner.mac_animation.borrow().is_none() {
                let (expanded, unexpanded) = match owner.native().CSSScrollbarWidth() {
                    foundation::EScrollbarWidth::kNone => (0, 0),
                    foundation::EScrollbarWidth::kThin => (14, 10),
                    _ => (16, 12),
                };
                *owner.mac_animation.borrow_mut() = Some(animation::MacScrollbarAnimation::new(expanded, unexpanded, foundation::scrollbar_theme_settings::ScrollbarThemeSettings::ScrollbarAnimationsEnabled()));
            }
            let activity = match activity.kind {
                Kind::Added | Kind::Removed => continue,
                Kind::Scrolled => animation::Activity::Scrolled(
                    self.controls.MouseInScrollbarFrameRect(&owner.owner),
                ),
                Kind::FadeInDeferred => animation::Activity::FadeInDeferred(
                    self.controls.MouseInScrollbarFrameRect(&owner.owner),
                ),
                Kind::FadeOutIfNeeded => animation::Activity::FadeOutIfNeeded,
                Kind::MouseEntered => animation::Activity::Entered,
                Kind::MouseExited => animation::Activity::Exited,
            };
            let now = animation::TimeTicks::FromMicroseconds(self.controls.MonotonicTimeMicros());
            let effects = owner
                .mac_animation
                .borrow_mut()
                .as_mut()
                .expect("registered Mac animator")
                .Activity(activity, now);
            owner.ApplyMacEffects(effects)?;
        }
        Ok(())
    }
    pub fn RunTimer(&self, task: ScrollbarTimerTask) -> io::Result<()> {
        match task.kind {
            ScrollbarTimerKind::Autoscroll => self.RunAutoscrollTimer(task),
            ScrollbarTimerKind::OverlayFade => {
                self.controls.RunOverlayFadeTimer(task);
                Ok(())
            }
            ScrollbarTimerKind::Mac(timer) => {
                let Some(owner) = self
                    .scrollbars
                    .borrow()
                    .get(&task.scrollbar_identity)
                    .cloned()
                else {
                    return Ok(());
                };
                if !owner
                    .mac_tokens
                    .borrow()
                    .get(&timer)
                    .is_some_and(|(generation, _)| *generation == task.generation)
                {
                    return Ok(());
                }
                owner.mac_tokens.borrow_mut().remove(&timer);
                let now =
                    animation::TimeTicks::FromMicroseconds(self.controls.MonotonicTimeMicros());
                let effects = owner
                    .mac_animation
                    .borrow_mut()
                    .as_mut()
                    .map(|animation| animation.TimerFired(timer, task.generation, now))
                    .unwrap_or_default();
                owner.ApplyMacEffects(effects)
            }
        }
    }
    pub fn RunAutoscrollTimer(&self, task: ScrollbarTimerTask) -> io::Result<()> {
        let Some(owner) = self
            .scrollbars
            .borrow()
            .get(&task.scrollbar_identity)
            .cloned()
        else {
            return Ok(());
        };
        let state = owner.native().InputState();
        if !state.autoscroll_timer_active.get()
            || state.autoscroll_timer_generation.get() != task.generation
        {
            return Ok(());
        }
        state.autoscroll_timer_active.set(false);
        owner.timer_token.set(None);
        scrollbar::AutoscrollTimerFired(owner.as_ref());
        let error = owner.error.borrow_mut().take();
        error.map_or(Ok(()), Err)
    }
    pub fn AutoscrollPressedPart(
        &self,
        owner: Persistent<Scrollbar>,
        delay: Duration,
    ) -> io::Result<()> {
        let owner = self.NativeScrollbar(owner);
        scrollbar::AutoscrollPressedPart(owner.as_ref(), delay);
        let error = owner.error.borrow_mut().take();
        error.map_or(Ok(()), Err)
    }
    pub fn AutoscrollPressedPartWithInitialDelay(
        &self,
        owner: Persistent<Scrollbar>,
    ) -> io::Result<()> {
        let native = unsafe { &*owner.Get() };
        let delay = if !native.IsCustomScrollbar()
            && native.ThemeMetrics().uses_macos_painted_scrollbars
        {
            foundation::scrollbar_theme_settings::ScrollbarThemeSettings::MacInitialAutoscrollTimerDelay()
        } else {
            Duration::from_millis(250)
        };
        self.AutoscrollPressedPart(owner, delay)
    }
    pub fn StopAutoscrollTimer(&self, owner: Persistent<Scrollbar>) {
        let owner = self.NativeScrollbar(owner);
        scrollbar::StopTimerIfNeeded(owner.as_ref());
    }
    pub fn EnclosingLayerScrollableArea(&self, node: u64) -> Option<Rc<dyn PointerResizeArea>> {
        let area = self.controls.ResizeAreaForNode(node)?;
        Some(
            self.areas
                .borrow_mut()
                .entry(area.Identity())
                .or_insert_with(|| {
                    Rc::new(NativePointerResizeOwner {
                        area,
                        controls: self.controls.clone(),
                    })
                })
                .clone(),
        )
    }
}
struct EventHandlerHitScrollbar(Persistent<Scrollbar>);
impl interaction_hit_test::core::layout::hit_test_result::HitTestScrollbar
    for EventHandlerHitScrollbar
{
    fn NativeScrollbarOwner(&self) -> Option<&Persistent<Scrollbar>> {
        Some(&self.0)
    }
}
struct NativePointerScrollbar {
    owner: Persistent<Scrollbar>,
    controls: Rc<dyn PagePointerScrollControls>,
    error: RefCell<Option<io::Error>>,
    timer_token: Cell<Option<u64>>,
    mac_animation: RefCell<Option<animation::MacScrollbarAnimation>>,
    mac_tokens: RefCell<HashMap<animation::Timer, (u64, u64)>>,
}
impl NativePointerScrollbar {
    fn MouseTheme(&self) -> scrollbar_theme::MouseTheme {
        if self.native().IsCustomScrollbar() {
            scrollbar_theme::MouseTheme::Base
        } else if self.native().ThemeMetrics().uses_macos_painted_scrollbars {
            scrollbar_theme::MouseTheme::Mac
        } else if !self.native().IsOverlayScrollbar()
            || foundation::scrollbar_theme_settings::ScrollbarThemeSettings::FluentScrollbarsEnabled(
            )
        {
            scrollbar_theme::MouseTheme::Aura
        } else {
            scrollbar_theme::MouseTheme::Base
        }
    }
    fn Interaction(
        &self,
        event: layoutng_assembly::native_pointer_scroll_controls::NativeScrollbarInteraction,
    ) {
        if let Err(error) = self.controls.ScrollbarInteraction(&self.owner, event) {
            self.error.borrow_mut().get_or_insert(error);
        }
    }
    fn native(&self) -> &Scrollbar {
        unsafe { &*self.owner.Get() }
    }
    fn ApplyMacEffects(&self, effects: Vec<animation::Effect>) -> io::Result<()> {
        for effect in effects {
            match effect {
                animation::Effect::Cancel(timer) => {
                    if let Some((_, token)) = self.mac_tokens.borrow_mut().remove(&timer) {
                        self.controls.CancelScrollbarTimer(token);
                    }
                }
                animation::Effect::Schedule {
                    timer,
                    generation,
                    delay_us,
                } => {
                    let token = self.controls.StartScrollbarTimer(
                        Duration::from_micros(delay_us),
                        ScrollbarTimerTask {
                            scrollbar_identity: self.owner.Get() as usize,
                            generation,
                            kind: ScrollbarTimerKind::Mac(timer),
                        },
                    )?;
                    self.mac_tokens
                        .borrow_mut()
                        .insert(timer, (generation, token));
                }
                animation::Effect::Publish {
                    thumb,
                    track,
                    width,
                    thumb_dirty,
                    track_dirty,
                    hidden,
                } => self.controls.PublishMacScrollbarAnimation(
                    &self.owner,
                    thumb,
                    track,
                    width,
                    thumb_dirty,
                    track_dirty,
                    hidden,
                ),
            }
        }
        Ok(())
    }
    fn CheckError(&self) {
        if let Some(error) = self.error.borrow_mut().take() {
            std::panic::panic_any(error);
        }
    }
}
impl crate::core::input::event_handler::EventHandlerScrollbar for NativePointerScrollbar {
    fn HandleGestureTapOrPress(&self, event: &WebGestureEvent) -> bool {
        let handled = scrollbar::HandleGestureTapOrPress(self, event);
        self.CheckError();
        handled
    }
    fn Pressed(&self) -> bool {
        self.native().PressedPart() != ScrollbarPart::kNoPart
    }
    fn ThumbPressed(&self) -> bool {
        self.native().PressedPart() == ScrollbarPart::kThumbPart
    }
    fn Enabled(&self) -> bool {
        self.native().Enabled()
    }
    fn MouseDown(&self, event: &WebMouseEvent) {
        scrollbar::MouseDown(self, event);
        self.CheckError();
    }
    fn MouseUp(&self, event: &WebMouseEvent) {
        scrollbar::MouseUp(self, event);
        self.CheckError();
    }
    fn MouseMoved(&self, event: &WebMouseEvent) {
        scrollbar::MouseMoved(self, event);
        self.CheckError();
    }
    fn MouseEntered(&self) {
        scrollbar::MouseEntered(self);
        self.CheckError();
    }
    fn MouseExited(&self) {
        scrollbar::MouseExited(self);
        self.CheckError();
    }
}
impl scrollbar::ScrollbarMouseSource for NativePointerScrollbar {
    fn HoveredPart(&self) -> ScrollbarPart {
        self.native().HoveredPart()
    }
    fn CurrentPosition(&self) -> f32 {
        self.native().CurrentPos()
    }
    fn TrackPosition(&self) -> i32 {
        self.native()
            .ThemeGeometryAt(self.native().CurrentPos())
            .track_position
    }
    fn MiddleClickAutoscrollEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::MiddleClickAutoscrollEnabled()
    }
    fn ShouldCenterOnThumb(&self, event: &WebMouseEvent) -> bool {
        scrollbar_theme::ShouldCenterOnThumb(self.MouseTheme(), event)
    }
    fn ShouldDragDocumentInsteadOfThumb(&self, event: &WebMouseEvent) -> bool {
        scrollbar_theme::ShouldDragDocumentInsteadOfThumb(self.MouseTheme(), event)
    }
    fn ShouldSnapBackToDragOrigin(&self, event: &WebMouseEvent) -> bool {
        let point = event.PositionInRootFrame();
        let local = self.ConvertFromRootFrame(gfx::Point::new(
            point.x().floor() as i32,
            point.y().floor() as i32,
        ));
        let track = self
            .native()
            .ThemeGeometryAt(self.native().CurrentPos())
            .track_rect;
        scrollbar_theme::ShouldSnapBackToDragOrigin(
            self.MouseTheme(),
            self.Horizontal(),
            if self.Horizontal() {
                track.height()
            } else {
                track.width()
            },
            local,
        )
    }
    fn InitialAutoscrollTimerDelay(&self) -> Duration {
        if matches!(self.MouseTheme(), scrollbar_theme::MouseTheme::Mac) {
            foundation::scrollbar_theme_settings::ScrollbarThemeSettings::MacInitialAutoscrollTimerDelay()
        } else {
            Duration::from_millis(250)
        }
    }
    fn MouseEnteredScrollbar(&self) {
        self.Interaction(
            layoutng_assembly::native_pointer_scroll_controls::NativeScrollbarInteraction::Entered,
        );
    }
    fn MouseExitedScrollbar(&self) {
        self.Interaction(
            layoutng_assembly::native_pointer_scroll_controls::NativeScrollbarInteraction::Exited,
        );
    }
    fn MouseCapturedScrollbar(&self) {
        self.Interaction(
            layoutng_assembly::native_pointer_scroll_controls::NativeScrollbarInteraction::Captured,
        );
    }
    fn MouseReleasedScrollbar(&self) {
        self.Interaction(
            layoutng_assembly::native_pointer_scroll_controls::NativeScrollbarInteraction::Released,
        );
    }
    fn UsesFluentOverlayScrollbars(&self) -> bool {
        !self.native().IsCustomScrollbar()
            && self.native().IsOverlayScrollbar()
            && foundation::scrollbar_theme_settings::ScrollbarThemeSettings::FluentScrollbarsEnabled(
            )
    }
    fn InvalidateThumb(&self) {
        self.native().InvalidatePaintForInput();
    }
    fn SetPaintArtifactCompositorNeedsUpdate(&self) {
        self.controls.ScrollbarPaintArtifactCompositorNeedsUpdate();
    }
}
impl PointerScrollbar for NativePointerScrollbar {
    fn HandlePointerEvent(&self, event: &WebPointerEvent) -> bool {
        let handled = scrollbar::HandlePointerEvent(self, event);
        if let Some(error) = self.error.borrow_mut().take() {
            std::panic::panic_any(error);
        }
        handled
    }
}
impl ScrollbarPointerSource for NativePointerScrollbar {
    fn InputState(&self) -> &foundation::scrollbar_input_state::ScrollbarInputState {
        self.native().InputState()
    }
    fn Connected(&self) -> bool {
        self.native().IsConnectedToScrollableArea()
    }
    fn Horizontal(&self) -> bool {
        self.native().Orientation() == ScrollbarOrientation::kHorizontalScrollbar
    }
    fn ThumbPressed(&self) -> bool {
        self.native().PressedPart() == ScrollbarPart::kThumbPart
    }
    fn SetThumbPressed(&self, pressed: bool, event_type: Type) {
        self.SetPressedPart(
            if pressed {
                ScrollbarPart::kThumbPart
            } else {
                ScrollbarPart::kNoPart
            },
            event_type,
        );
    }
    fn IsThumbAtRootFramePoint(&self, point: gfx::Point) -> bool {
        self.native()
            .HitTestScrollbarPoint(self.ConvertFromRootFrame(point))
            == ScrollbarPart::kThumbPart
    }
    fn ConvertFromRootFrame(&self, point: gfx::Point) -> gfx::Point {
        self.controls
            .ScrollbarPointFromRootFrame(&self.owner, point)
    }
    fn ThumbGeometry(&self) -> ThumbGeometry {
        let value = self.native().ThemeGeometryAt(self.native().CurrentPos());
        ThumbGeometry {
            position: value.thumb_position,
            length: value.thumb_length,
            track_length: value.track_length,
        }
    }
    fn ScrollOffsets(&self) -> ScrollOffsets {
        let value = self
            .native()
            .InputScrollState()
            .expect("connected scrollbar area");
        ScrollOffsets {
            current: value.current_offset,
            minimum: value.minimum_offset,
            maximum: value.maximum_offset,
        }
    }
    fn ScrollElementId(&self) -> foundation::CompositorElementId {
        self.native()
            .InputScrollState()
            .expect("connected scrollbar area")
            .scroll_element_id
    }
    fn InputEventsScaleFactor(&self) -> f32 {
        self.controls.InputEventsScaleFactor()
    }
    fn InjectScrollbarGesture(&self, request: ScrollbarGesture) {
        if self.error.borrow().is_some() {
            return;
        }
        if let Err(error) = self.controls.InjectScrollbarGesture(request) {
            *self.error.borrow_mut() = Some(error);
        }
    }
}
impl ScrollbarGestureSource for NativePointerScrollbar {
    fn PressedPart(&self) -> ScrollbarPart {
        self.native().PressedPart()
    }
    fn SetPressedPart(&self, part: ScrollbarPart, event_type: Type) {
        if self.Connected() {
            for feature in scrollbar::PressedPartUseFeatures(part, self.Horizontal(), event_type)
                .into_iter()
                .flatten()
            {
                self.controls.CountScrollbarUse(feature);
            }
        }
        if self.native().SetPressedPart(part) {
            self.controls.ScrollbarPaintArtifactCompositorNeedsUpdate();
        }
    }
    fn HitTestRootFramePosition(&self, point: gfx::Point) -> ScrollbarPart {
        self.native()
            .HitTestScrollbarPoint(self.ConvertFromRootFrame(point))
    }
}
impl scrollbar::ScrollbarAutoscrollSource for NativePointerScrollbar {
    fn TargetThumbGeometry(&self) -> (i32, i32) {
        self.controls.TargetThumbGeometry(&self.owner)
    }
    fn SetHoveredPart(&self, part: ScrollbarPart) {
        if self.native().SetHoveredPart(part) {
            self.controls.ScrollbarPaintArtifactCompositorNeedsUpdate();
        }
    }
    fn StartAutoscrollTimer(&self, delay: Duration) {
        self.StopAutoscrollTimer();
        let state = self.InputState();
        let generation = state
            .autoscroll_timer_generation
            .get()
            .checked_add(1)
            .expect("scrollbar timer generation exhausted");
        state.autoscroll_timer_generation.set(generation);
        state.autoscroll_timer_active.set(true);
        match self.controls.StartScrollbarTimer(
            delay,
            ScrollbarTimerTask {
                scrollbar_identity: self.owner.Get() as usize,
                generation,
                kind: ScrollbarTimerKind::Autoscroll,
            },
        ) {
            Ok(token) => self.timer_token.set(Some(token)),
            Err(error) => {
                state.autoscroll_timer_active.set(false);
                *self.error.borrow_mut() = Some(error);
            }
        }
    }
    fn StopAutoscrollTimer(&self) {
        self.InputState().autoscroll_timer_active.set(false);
        if let Some(token) = self.timer_token.take() {
            self.controls.CancelScrollbarTimer(token);
        }
    }
    fn AutoscrollTimerDelay(&self) -> Duration {
        // CustomScrollbarTheme inherits the base delay; native Mac theme
        // reads the same NSDefaults-updated source global as its initial delay.
        if !self.native().IsCustomScrollbar()
            && self.native().ThemeMetrics().uses_macos_painted_scrollbars
        {
            foundation::scrollbar_theme_settings::ScrollbarThemeSettings::MacAutoscrollTimerDelay()
        } else {
            Duration::from_micros(50_000)
        }
    }
}
struct ResizeTransform(gfx::Transform);
impl PointerResizeTransform for ResizeTransform {
    fn Transform(&self) -> &gfx::Transform {
        &self.0
    }
}
struct NativePointerResizeOwner {
    area: NativePointerResizeArea,
    controls: Rc<dyn PagePointerScrollControls>,
}
fn Point(point: gfx::PointF) -> gfx::Point {
    gfx::Point::new(point.x() as i32, point.y() as i32)
}
impl PointerResizeArea for NativePointerResizeOwner {
    fn IsAbsolutePointInResizeControl(&self, point: gfx::PointF) -> bool {
        self.controls
            .IsAbsolutePointInResizeControl(&self.area, Point(point))
    }
    fn SetInResizeMode(&self, value: bool) {
        self.controls.SetInResizeMode(&self.area, value);
    }
    fn InResizeMode(&self) -> bool {
        self.controls.InResizeMode(&self.area)
    }
    fn HasLayerAndLayoutBox(&self) -> bool {
        self.controls.HasLayerAndLayoutBox(&self.area)
    }
    fn InitializeResizeTransform(&self, point: gfx::PointF) -> Rc<dyn PointerResizeTransform> {
        Rc::new(ResizeTransform(
            self.controls
                .InitializeResizeTransform(&self.area, Point(point)),
        ))
    }
    fn Resize(&self, point: gfx::PointF, transform: &dyn PointerResizeTransform) {
        self.controls
            .Resize(&self.area, Point(point), transform.Transform())
            .expect("pointer resize lifecycle");
    }
}
