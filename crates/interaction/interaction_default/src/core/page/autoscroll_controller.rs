/*
 * Copyright (C) 2013 Google Inc. All rights reserved.
 * Copyright (C) 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@webkit.org)
 * Copyright (C) 2012 Digia Plc. and/or its subsidiary(-ies)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/page/autoscroll_controller.h:46-55,113-137;
// core/page/autoscroll_controller.cc:46-95,107-173,175-228,254-534.
// Scope: selection/drag main-thread and middle-click compositor autoscroll.
// iOS selection-to-point is outside this desktop source slice.
use interaction_input::{
    base::time::time::TimeTicks, public::common::input::web_pointer_properties::PointF,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum AutoscrollType {
    #[default]
    None,
    DragAndDrop,
    Selection,
    MiddleClick,
}
/// The same resident LayoutObject/Box identity. Implementations retain the
/// native owner, not an event-only layout tree. Removed owners become detached.
pub trait AutoscrollLayout {
    fn Identity(&self) -> usize;
    fn FrameIdentity(&self) -> Option<usize>;
    fn HasPage(&self) -> bool;
    fn IsBox(&self) -> bool;
    fn HasNode(&self) -> bool;
    fn NodeId(&self) -> Option<u64> {
        None
    }
    fn IsAttachedOutermostMainFrame(&self) -> bool;
    fn FindAutoscrollable(&self, middle_click: bool) -> Option<Rc<dyn AutoscrollLayout>>;
    fn NodeAutoscrollBox(&self) -> Option<Rc<dyn AutoscrollLayout>>;
    fn StopNodeAutoscroll(&self);
    fn CalculateAutoscrollDirection(&self, point: PointF) -> (f64, f64);
    fn Autoscroll(&self, point: (f64, f64)) -> bool;
    fn ParentCrossingDocumentBoundary(&self) -> Option<Rc<dyn AutoscrollLayout>>;
    fn HasScrollableOverflow(&self, horizontal: bool) -> bool;
    fn CanPropagate(&self, horizontal: bool) -> bool;
    fn IsUserScrollable(&self) -> bool;
}
/// Frame input and embedding scheduling. These are enqueue/notification leaves;
/// the source controller decides state, ordering and continuation.
/// Selection and drag-and-drop need no platform panning/impl-scroll service.
/// Keeping these leaves separate lets BeginFrame borrow the live selection
/// transaction without fabricating middle-click host capabilities.
pub trait SelectionAutoscrollFrame: AutoscrollStopServices {
    fn LastKnownMousePositionInRootFrame(&self, layout: &dyn AutoscrollLayout) -> PointF;
    fn MousePressed(&self, layout: &dyn AutoscrollLayout) -> bool;
    fn UpdateSelectionForMouseDrag(&self, layout: &dyn AutoscrollLayout);
    fn AssertScrollLifecyclePrepared(&self, layout: &dyn AutoscrollLayout);
    fn ScheduleMainThreadAnimation(&self, layout: &dyn AutoscrollLayout);
}
pub trait AutoscrollFrame: AutoscrollStopServices {
    fn VisualViewportMaximumScrollOffset(&self) -> (f64, f64);
    fn DevicePixelRatio(&self) -> f32;
    fn IsLocalRoot(&self) -> bool;
    fn HasView(&self) -> bool;
    fn SetPanningCursor(&self, cursor: PanningCursor);
    fn SystemAutoscrollStart(&self, position: PointF);
    fn SystemAutoscrollFling(&self, velocity: (f32, f32));
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PanningCursor {
    NorthEast,
    NorthWest,
    North,
    SouthEast,
    SouthWest,
    South,
    East,
    West,
    Vertical,
    Horizontal,
    Middle,
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum MiddleClickMode {
    #[default]
    Initial,
    Holding,
    Toggled,
}
fn MiddleCursor(velocity: (f32, f32), vertical: bool, horizontal: bool) -> PanningCursor {
    if velocity.1 > 0.0 && vertical {
        if horizontal && velocity.0 < 0.0 {
            return PanningCursor::NorthEast;
        }
        if horizontal && velocity.0 > 0.0 {
            return PanningCursor::NorthWest;
        }
        return PanningCursor::North;
    }
    if velocity.1 < 0.0 && vertical {
        if horizontal && velocity.0 < 0.0 {
            return PanningCursor::SouthEast;
        }
        if horizontal && velocity.0 > 0.0 {
            return PanningCursor::SouthWest;
        }
        return PanningCursor::South;
    }
    if velocity.0 < 0.0 && horizontal {
        return PanningCursor::East;
    }
    if velocity.0 > 0.0 && horizontal {
        return PanningCursor::West;
    }
    if vertical && !horizontal {
        PanningCursor::Vertical
    } else if horizontal && !vertical {
        PanningCursor::Horizontal
    } else {
        PanningCursor::Middle
    }
}
/// EventHandler retains this state for the LocalFrame lifetime. Frame services
/// are borrowed for each call, so a selection callback never retains a listener
/// dispatcher or a Layout loan from an earlier input transaction.
pub struct AutoscrollControllerState {
    pub autoscroll_type: Cell<AutoscrollType>,
    layout: RefCell<Option<Rc<dyn AutoscrollLayout>>>,
    pressed: RefCell<Option<Rc<dyn AutoscrollLayout>>>,
    drag_start: Cell<TimeTicks>,
    drag_reference: Cell<(f64, f64)>,
    scroll_result: Cell<bool>,
    horizontal: RefCell<Option<Rc<dyn AutoscrollLayout>>>,
    vertical: RefCell<Option<Rc<dyn AutoscrollLayout>>>,
    middle_start: Cell<PointF>,
    last_velocity: Cell<(f32, f32)>,
    middle_mode: Cell<MiddleClickMode>,
}
impl Default for AutoscrollControllerState {
    fn default() -> Self {
        Self {
            autoscroll_type: Cell::new(AutoscrollType::None),
            layout: RefCell::new(None),
            pressed: RefCell::new(None),
            drag_start: Cell::new(TimeTicks::new()),
            drag_reference: Cell::new((0.0, 0.0)),
            scroll_result: Cell::new(true),
            horizontal: RefCell::new(None),
            vertical: RefCell::new(None),
            middle_start: Cell::new(PointF::default()),
            last_velocity: Cell::new((0.0, 0.0)),
            middle_mode: Cell::new(MiddleClickMode::Initial),
        }
    }
}
/// Stop's source leaves are also used by ScrollManager and focus. These paths
/// do not require animation or selection-update services.
pub trait AutoscrollStopServices {
    fn StopNodeAutoscroll(&self, layout: &dyn AutoscrollLayout) {
        layout.StopNodeAutoscroll();
    }
    fn NotifyAutoscrollForSelectionInMainFrame(&self, active: bool);
    fn SystemAutoscrollEnd(&self);
    fn SetCursorOverridden(&self, overridden: bool);
    fn UpdateCursor(&self);
}
impl AutoscrollControllerState {
    pub fn SelectionAutoscrollInProgress(&self) -> bool {
        self.autoscroll_type.get() == AutoscrollType::Selection
    }
    pub fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.autoscroll_type.get() == AutoscrollType::MiddleClick
    }
    pub fn MiddleClickAutoscrollUsesFrame(&self, identity: usize) -> bool {
        self.MiddleClickAutoscrollInProgress()
            && [
                self.horizontal.borrow().clone(),
                self.vertical.borrow().clone(),
            ]
            .into_iter()
            .flatten()
            .any(|layout| layout.FrameIdentity() == Some(identity))
    }
    // Source: autoscroll_controller.cc:141-150,523-535.
    pub fn StopAutoscroll(&self, services: &dyn AutoscrollStopServices) {
        let pressed = self.pressed.borrow_mut().take();
        if let Some(pressed) = pressed.filter(|layout| layout.HasNode()) {
            services.StopNodeAutoscroll(pressed.as_ref());
        }
        let notify = self.layout.borrow().as_ref().is_some_and(|owner| {
            owner.FrameIdentity().is_some() && owner.IsAttachedOutermostMainFrame()
        });
        if notify {
            services.NotifyAutoscrollForSelectionInMainFrame(false);
        }
        self.layout.borrow_mut().take();
        self.autoscroll_type.set(AutoscrollType::None);
    }
    // Source: autoscroll_controller.cc:357-372.
    pub fn StopMiddleClickAutoscroll(&self, services: &dyn AutoscrollStopServices) {
        if !self.MiddleClickAutoscrollInProgress() {
            return;
        }
        services.SystemAutoscrollEnd();
        self.autoscroll_type.set(AutoscrollType::None);
        services.SetCursorOverridden(false);
        services.UpdateCursor();
        self.horizontal.borrow_mut().take();
        self.vertical.borrow_mut().take();
    }
}
pub struct AutoscrollController<'a, F: AutoscrollStopServices + ?Sized> {
    frame: &'a F,
    state: &'a AutoscrollControllerState,
}
impl<F: AutoscrollStopServices + ?Sized> std::ops::Deref for AutoscrollController<'_, F> {
    type Target = AutoscrollControllerState;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}
impl<F: AutoscrollStopServices + ?Sized> AutoscrollStopServices for AutoscrollController<'_, F> {
    fn StopNodeAutoscroll(&self, layout: &dyn AutoscrollLayout) {
        self.frame.StopNodeAutoscroll(layout);
    }
    fn NotifyAutoscrollForSelectionInMainFrame(&self, active: bool) {
        self.frame.NotifyAutoscrollForSelectionInMainFrame(active);
    }
    fn SystemAutoscrollEnd(&self) {
        self.frame.SystemAutoscrollEnd();
    }
    fn SetCursorOverridden(&self, overridden: bool) {
        self.frame.SetCursorOverridden(overridden);
    }
    fn UpdateCursor(&self) {
        self.frame.UpdateCursor();
    }
}
impl<'a, F: AutoscrollStopServices + ?Sized> AutoscrollController<'a, F> {
    pub fn WithState(frame: &'a F, state: &'a AutoscrollControllerState) -> Self {
        Self { frame, state }
    }
}
impl<F: SelectionAutoscrollFrame + ?Sized> AutoscrollController<'_, F> {
    pub fn SelectionAutoscrollInProgress(&self) -> bool {
        self.autoscroll_type.get() == AutoscrollType::Selection
    }
    pub fn AutoscrollInProgress(&self) -> bool {
        self.layout.borrow().is_some()
    }
    pub fn IsAutoscrolling(&self) -> bool {
        self.autoscroll_type.get() != AutoscrollType::None
    }
    pub fn AutoscrollInProgressFor(&self, layout: &dyn AutoscrollLayout) -> bool {
        self.layout
            .borrow()
            .as_ref()
            .is_some_and(|owner| owner.Identity() == layout.Identity())
    }
    pub fn StartAutoscrollForSelection(&self, layout: Rc<dyn AutoscrollLayout>) {
        if self.autoscroll_type.get() != AutoscrollType::None {
            return;
        }
        let scrollable = layout.FindAutoscrollable(false).or_else(|| {
            if layout.HasNode() {
                layout.NodeAutoscrollBox()
            } else {
                None
            }
        });
        let Some(scrollable) = scrollable else {
            return;
        };
        *self.pressed.borrow_mut() = layout.IsBox().then_some(layout);
        self.autoscroll_type.set(AutoscrollType::Selection);
        *self.layout.borrow_mut() = Some(scrollable);
        self.UpdateCachedSelectionState(true);
        self.ScheduleAnimation();
    }
    pub fn StopAutoscroll(&self) {
        self.state.StopAutoscroll(self);
    }
    pub fn StopAutoscrollIfNeeded(&self, layout: &dyn AutoscrollLayout) {
        if self
            .pressed
            .borrow()
            .as_ref()
            .is_some_and(|owner| owner.Identity() == layout.Identity())
        {
            self.pressed.borrow_mut().take();
        }
        if self
            .horizontal
            .borrow()
            .as_ref()
            .is_some_and(|owner| owner.Identity() == layout.Identity())
        {
            self.horizontal.borrow_mut().take();
        }
        if self
            .vertical
            .borrow()
            .as_ref()
            .is_some_and(|owner| owner.Identity() == layout.Identity())
        {
            self.vertical.borrow_mut().take();
        }
        if self.MiddleClickAutoscrollInProgress()
            && self.horizontal.borrow().is_none()
            && self.vertical.borrow().is_none()
        {
            self.frame.SystemAutoscrollEnd();
            self.autoscroll_type.set(AutoscrollType::None);
        }
        if !self.AutoscrollInProgressFor(layout) {
            return;
        }
        self.UpdateCachedSelectionState(false);
        self.layout.borrow_mut().take();
        self.autoscroll_type.set(AutoscrollType::None);
    }
    pub fn UpdateDragAndDrop(
        &self,
        target: Option<Rc<dyn AutoscrollLayout>>,
        position: PointF,
        time: TimeTicks,
    ) {
        let Some(target) = target else {
            self.StopAutoscroll();
            return;
        };
        let current = self.layout.borrow().clone();
        if current
            .as_ref()
            .is_some_and(|owner| owner.FrameIdentity() != target.FrameIdentity())
        {
            return;
        }
        self.frame.AssertScrollLifecyclePrepared(target.as_ref());
        let Some(scrollable) = target.FindAutoscrollable(false) else {
            self.StopAutoscroll();
            return;
        };
        if !scrollable.HasPage() {
            self.StopAutoscroll();
            return;
        }
        let offset = scrollable.CalculateAutoscrollDirection(position);
        if offset == (0.0, 0.0) {
            self.StopAutoscroll();
            return;
        }
        self.drag_reference.set((
            position.x().round() as f64 + offset.0,
            position.y().round() as f64 + offset.1,
        ));
        if self.autoscroll_type.get() == AutoscrollType::None {
            self.autoscroll_type.set(AutoscrollType::DragAndDrop);
            *self.layout.borrow_mut() = Some(scrollable);
            self.drag_start.set(time);
            self.ScheduleAnimation();
        } else if current
            .as_ref()
            .is_none_or(|owner| owner.Identity() != scrollable.Identity())
        {
            self.drag_start.set(time);
            *self.layout.borrow_mut() = Some(scrollable);
        }
    }
    pub fn Animate(&self, now: TimeTicks) {
        if self.autoscroll_type.get() == AutoscrollType::MiddleClick {
            return;
        }
        let layout = self.layout.borrow().clone();
        let Some(layout) = layout.filter(|owner| owner.FrameIdentity().is_some()) else {
            self.StopAutoscroll();
            return;
        };
        let mouse = self
            .frame
            .LastKnownMousePositionInRootFrame(layout.as_ref());
        let offset = layout.CalculateAutoscrollDirection(mouse);
        let point = (
            mouse.x().round() as f64 + offset.0,
            mouse.y().round() as f64 + offset.1,
        );
        match self.autoscroll_type.get() {
            AutoscrollType::DragAndDrop => {
                self.ScheduleAnimation();
                if now
                    .ToInternalValue()
                    .saturating_sub(self.drag_start.get().ToInternalValue())
                    > 200_000
                {
                    layout.Autoscroll(self.drag_reference.get());
                }
            }
            AutoscrollType::Selection => {
                if !self.frame.MousePressed(layout.as_ref()) {
                    self.StopAutoscroll();
                    return;
                }
                if self.scroll_result.get() {
                    self.frame.UpdateSelectionForMouseDrag(layout.as_ref());
                }
                if self.autoscroll_type.get() != AutoscrollType::None {
                    self.ScheduleAnimation();
                    let current = self
                        .layout
                        .borrow()
                        .clone()
                        .expect("active autoscroll retains layout");
                    self.scroll_result.set(current.Autoscroll(point));
                }
            }
            AutoscrollType::None | AutoscrollType::MiddleClick => {}
        }
    }
    fn ScheduleAnimation(&self) {
        let layout = self.layout.borrow().clone();
        if let Some(layout) = layout {
            self.frame.ScheduleMainThreadAnimation(layout.as_ref());
        }
    }
    fn UpdateCachedSelectionState(&self, active: bool) {
        let layout = self.layout.borrow().clone();
        if layout.as_ref().is_some_and(|owner| {
            owner.FrameIdentity().is_some() && owner.IsAttachedOutermostMainFrame()
        }) {
            self.frame.NotifyAutoscrollForSelectionInMainFrame(active);
        }
    }
}

impl<F: AutoscrollFrame + ?Sized> AutoscrollController<'_, F> {
    fn CanScrollDirection(&self, layout: &dyn AutoscrollLayout, horizontal: bool) -> bool {
        let viewport = self.frame.VisualViewportMaximumScrollOffset();
        layout.HasScrollableOverflow(horizontal)
            || (layout.HasPage()
                && if horizontal {
                    viewport.0 > 0.0
                } else {
                    viewport.1 > 0.0
                })
    }
    pub fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.autoscroll_type.get() == AutoscrollType::MiddleClick
    }
    pub fn StopMiddleClickAutoscroll(&self) {
        self.state.StopMiddleClickAutoscroll(self);
    }
    pub fn StartMiddleClickAutoscroll(
        &self,
        scrollable: Rc<dyn AutoscrollLayout>,
        position: PointF,
        global: PointF,
    ) {
        if self.autoscroll_type.get() != AutoscrollType::None {
            return;
        }
        self.autoscroll_type.set(AutoscrollType::MiddleClick);
        self.middle_mode.set(MiddleClickMode::Initial);
        self.middle_start.set(global);
        let (mut can_vertical, mut can_horizontal) = (false, false);
        let (mut propagate_vertical, mut propagate_horizontal) = (true, true);
        let mut layout = Some(scrollable);
        while let Some(current) = layout {
            if can_horizontal && can_vertical {
                break;
            }
            if current.IsBox() {
                if propagate_vertical
                    && self.CanScrollDirection(current.as_ref(), false)
                    && self.vertical.borrow().is_none()
                {
                    *self.vertical.borrow_mut() = Some(current.clone());
                    can_vertical = true;
                }
                if propagate_horizontal
                    && self.CanScrollDirection(current.as_ref(), true)
                    && self.horizontal.borrow().is_none()
                {
                    *self.horizontal.borrow_mut() = Some(current.clone());
                    can_horizontal = true;
                }
                propagate_vertical = current.CanPropagate(false);
                propagate_horizontal = current.CanPropagate(true);
            }
            if (!propagate_vertical && !propagate_horizontal) || (can_horizontal && can_vertical) {
                break;
            }
            layout = current.ParentCrossingDocumentBoundary();
        }
        self.last_velocity.set((0.0, 0.0));
        if self.frame.HasView() {
            self.frame
                .SetPanningCursor(MiddleCursor((0.0, 0.0), can_vertical, can_horizontal));
        }
        self.frame.SetCursorOverridden(true);
        let dpr = self.frame.DevicePixelRatio();
        self.frame
            .SystemAutoscrollStart(PointF::new(position.x() / dpr, position.y() / dpr));
    }
    pub fn HandleMouseMoveForMiddleClickAutoscroll(&self, global: PointF) {
        if !self.MiddleClickAutoscrollInProgress() {
            return;
        }
        let horizontal = self
            .horizontal
            .borrow()
            .clone()
            .filter(|layout| layout.HasNode());
        let vertical = self
            .vertical
            .borrow()
            .clone()
            .filter(|layout| layout.HasNode());
        if horizontal
            .as_ref()
            .is_some_and(|owner| !owner.IsUserScrollable())
            && vertical
                .as_ref()
                .is_some_and(|owner| !owner.IsUserScrollable())
        {
            self.StopMiddleClickAutoscroll();
            return;
        }
        if !self.frame.HasView() {
            return;
        }
        let dpr = self.frame.DevicePixelRatio();
        let start = self.middle_start.get();
        let mut distance = (
            (global.x() - start.x()) / dpr,
            (global.y() - start.y()) / dpr,
        );
        if distance.0.abs() <= 15.0 {
            distance.0 = 0.0;
        }
        if distance.1.abs() <= 15.0 {
            distance.1 = 0.0;
        }
        let component = |distance: f32| -> f32 {
            (f64::from(distance.abs()).powf(2.2)
                * -0.000008
                * f64::from(if distance < 0.0 {
                    -1.0
                } else if distance > 0.0 {
                    1.0
                } else {
                    0.0
                })) as f32
        };
        let velocity = (component(distance.0), component(distance.1));
        let can_vertical = vertical
            .as_ref()
            .is_some_and(|owner| self.CanScrollDirection(owner.as_ref(), false));
        let can_horizontal = horizontal
            .as_ref()
            .is_some_and(|owner| self.CanScrollDirection(owner.as_ref(), true));
        if velocity != self.last_velocity.get() {
            self.last_velocity.set(velocity);
            if self.middle_mode.get() == MiddleClickMode::Initial {
                self.middle_mode.set(MiddleClickMode::Holding);
            }
            self.frame.SetCursorOverridden(false);
            self.frame
                .SetPanningCursor(MiddleCursor(velocity, can_vertical, can_horizontal));
            self.frame.SetCursorOverridden(true);
            self.frame.SystemAutoscrollFling(velocity);
        }
    }
    pub fn HandleMouseReleaseForMiddleClickAutoscroll(&self, is_middle_button: bool) {
        if !self.MiddleClickAutoscrollInProgress() || !self.frame.IsLocalRoot() {
            return;
        }
        if self.middle_mode.get() == MiddleClickMode::Initial && is_middle_button {
            self.middle_mode.set(MiddleClickMode::Toggled);
        } else if self.middle_mode.get() == MiddleClickMode::Holding {
            self.StopMiddleClickAutoscroll();
        }
    }
}
