// Copyright (C) 2006-2010 Apple Inc.; BSD-style license, see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: spin_button_element.h:39-116; .cc:46-264.
use foundation::{
    gfx::PointF, spin_button_geometry::SpinButtonGeometry, LayoutUnit, PhysicalDirection,
};
use interaction_dispatch::core::{
    dom::events::event::EventRef, events::pointer_event_factory::PointerEventFactory,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UpDownState {
    Down,
    Up,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EventDispatch {
    Allowed,
    Disallowed,
}
pub trait SpinButtonOwner {
    fn FocusAndSelectSpinButtonOwner(&self);
    fn ShouldSpinButtonRespondToMouseEvents(&self) -> bool;
    fn ShouldSpinButtonRespondToWheelEvents(&self) -> bool;
    fn SpinButtonDidReleaseMouseCapture(&self, dispatch: EventDispatch);
    fn SpinButtonStepDown(&self);
    fn SpinButtonStepUp(&self);
}
pub trait SpinButtonDOM {
    fn SetShadowPseudoIdAndId(&self, pseudo: &'static str, id: &'static str);
    fn HasLayoutBox(&self) -> bool;
    fn HasLayoutObject(&self) -> bool;
    fn HasLocalFrame(&self) -> bool;
    fn HasPage(&self) -> bool;
    fn OwnerShadowHostDisabled(&self) -> Option<bool>;
    fn OwnerShadowHostReadOnly(&self) -> Option<bool>;
    fn OwnerShadowHostReadWrite(&self) -> Option<bool>;
    fn SetMousePointerCapture(&self, pointer: i32);
    fn ReleaseMousePointerCapture(&self, pointer: i32);
    fn RegisterPopupOpeningObserver(&self);
    fn UnregisterPopupOpeningObserver(&self);
    fn HTMLDivDefaultEventHandler(&self, event: &EventRef);
    fn HTMLDivWillRespondToMouseMoveEvents(&self) -> bool;
    fn HTMLDivWillRespondToMouseClickEvents(&self) -> bool;
    fn DetachHTMLDivLayoutTree(&self, performing_reattach: bool);
}
pub trait SpinButtonGeometryOwner {
    fn Geometry(&self, absolute: PointF) -> Option<SpinButtonGeometry>;
    fn SetShouldDoFullPaintInvalidation(&self);
}
pub trait SpinButtonTimer {
    fn Start(&self, initial: Duration, repeat: Duration);
    fn Stop(&self);
}
pub trait SpinButtonTheme {
    fn InitialAutoscrollTimerDelay(&self) -> Duration;
    fn AutoscrollTimerDelay(&self) -> Duration;
}
pub struct SpinButtonElement {
    owner: RefCell<Option<Rc<dyn SpinButtonOwner>>>,
    dom: Rc<dyn SpinButtonDOM>,
    geometry: Rc<dyn SpinButtonGeometryOwner>,
    timer: Rc<dyn SpinButtonTimer>,
    theme: Rc<dyn SpinButtonTheme>,
    capturing: Cell<bool>,
    up_down_state: Cell<UpDownState>,
    press_starting_state: Cell<UpDownState>,
    should_recalc_up_down_state: Cell<bool>,
}
impl SpinButtonElement {
    pub fn new(
        owner: Rc<dyn SpinButtonOwner>,
        dom: Rc<dyn SpinButtonDOM>,
        geometry: Rc<dyn SpinButtonGeometryOwner>,
        timer: Rc<dyn SpinButtonTimer>,
        theme: Rc<dyn SpinButtonTheme>,
    ) -> Rc<Self> {
        dom.SetShadowPseudoIdAndId("-webkit-inner-spin-button", "spin");
        Rc::new(Self {
            owner: RefCell::new(Some(owner)),
            dom,
            geometry,
            timer,
            theme,
            capturing: Cell::new(false),
            up_down_state: Cell::new(UpDownState::Down),
            press_starting_state: Cell::new(UpDownState::Down),
            should_recalc_up_down_state: Cell::new(false),
        })
    }
    fn Owner(&self) -> Option<Rc<dyn SpinButtonOwner>> {
        self.owner.borrow().clone()
    }
    // Source: h:84-99; cc:183-189. These are native subclass virtuals.
    pub fn IsSpinButtonElement(&self) -> bool {
        true
    }
    pub fn IsDisabledFormControl(&self) -> bool {
        self.dom.OwnerShadowHostDisabled().unwrap_or(false)
    }
    pub fn MatchesReadOnlyPseudoClass(&self) -> bool {
        self.dom
            .OwnerShadowHostReadOnly()
            .expect("spin has its shadow host")
    }
    pub fn MatchesReadWritePseudoClass(&self) -> bool {
        self.dom
            .OwnerShadowHostReadWrite()
            .expect("spin has its shadow host")
    }
    pub fn IsFocusableState(&self) -> crate::core::dom::element::FocusableState {
        crate::core::dom::element::FocusableState::kNotFocusable
    }
    pub fn GetUpDownState(&self) -> UpDownState {
        self.up_down_state.get()
    }
    pub fn RemoveSpinButtonOwner(&self) {
        self.owner.borrow_mut().take();
    }
    pub fn DetachLayoutTree(&self, performing_reattach: bool) {
        self.ReleaseCapture(EventDispatch::Disallowed);
        self.dom.DetachHTMLDivLayoutTree(performing_reattach);
    }
    pub fn DefaultEventHandler(&self, event: &EventRef) {
        let mouse = {
            let event = event.borrow();
            event
                .AsMouseEvent()
                .map(|mouse| (event.BaseEvent().r#type().to_owned(), mouse.RawButton()))
        };
        let Some((kind, button)) = mouse else {
            self.InheritedDefaultIfNeeded(event);
            return;
        };
        if !self.dom.HasLayoutBox() || !self.ShouldRespondToMouseEvents() {
            self.InheritedDefaultIfNeeded(event);
            return;
        }
        if kind == "mousedown" && button == 0 {
            if let Some(owner) = self.Owner() {
                owner.FocusAndSelectSpinButtonOwner();
            }
            if self.dom.HasLayoutObject() {
                // Timer starts before stepping, so synchronous listeners may cancel it.
                self.press_starting_state.set(self.up_down_state.get());
                self.timer.Start(
                    self.theme.InitialAutoscrollTimerDelay(),
                    self.theme.AutoscrollTimerDelay(),
                );
                if self.should_recalc_up_down_state.replace(false) {
                    self.CalculateUpDownStateByMouseLocation(event);
                }
                self.DoStepAction(if self.up_down_state.get() == UpDownState::Up {
                    1
                } else {
                    -1
                });
            }
            if self.dom.HasLayoutObject() && !self.capturing.get() && self.dom.HasLocalFrame() {
                self.dom
                    .SetMousePointerCapture(PointerEventFactory::kMouseId);
                self.capturing.set(true);
                if self.dom.HasPage() {
                    self.dom.RegisterPopupOpeningObserver();
                }
            }
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        } else if kind == "mouseup" && button == 0 {
            self.ReleaseCapture(EventDispatch::Allowed);
        } else if kind == "mousemove" {
            self.CalculateUpDownStateByMouseLocation(event);
        }
        self.InheritedDefaultIfNeeded(event);
    }
    fn InheritedDefaultIfNeeded(&self, event: &EventRef) {
        if !event.borrow().BaseEvent().DefaultHandled() {
            self.dom.HTMLDivDefaultEventHandler(event);
        }
    }
    pub fn WillOpenPopup(&self) {
        self.ReleaseCapture(EventDispatch::Allowed);
    }
    pub fn ForwardEvent(&self, event: &EventRef) {
        if !self.dom.HasLayoutBox() {
            return;
        }
        if event.borrow().BaseEvent().r#type() == "focus" {
            self.should_recalc_up_down_state.set(true);
        }
        let amount = {
            let event = event.borrow();
            event.AsWheelEvent().map(|wheel| wheel.wheelDeltaY())
        };
        let Some(amount) = amount else { return };
        let Some(owner) = self.Owner() else { return };
        if !owner.ShouldSpinButtonRespondToWheelEvents() {
            return;
        }
        self.DoStepAction(amount);
        event.borrow_mut().BaseEventMut().SetDefaultHandled();
    }
    pub fn WillRespondToMouseMoveEvents(&self) -> bool {
        (self.dom.HasLayoutBox() && self.ShouldRespondToMouseEvents())
            || self.dom.HTMLDivWillRespondToMouseMoveEvents()
    }
    pub fn WillRespondToMouseClickEvents(&self) -> bool {
        (self.dom.HasLayoutBox() && self.ShouldRespondToMouseEvents())
            || self.dom.HTMLDivWillRespondToMouseClickEvents()
    }
    fn DoStepAction(&self, amount: i32) {
        if let Some(owner) = self.Owner() {
            if amount > 0 {
                owner.SpinButtonStepUp()
            } else if amount < 0 {
                owner.SpinButtonStepDown()
            }
        }
    }
    pub fn ReleaseCapture(&self, dispatch: EventDispatch) {
        self.timer.Stop();
        if !self.capturing.get() {
            return;
        }
        if self.dom.HasLocalFrame() {
            self.dom
                .ReleaseMousePointerCapture(PointerEventFactory::kMouseId);
            self.capturing.set(false);
            if self.dom.HasPage() {
                self.dom.UnregisterPopupOpeningObserver();
            }
        }
        if let Some(owner) = self.Owner() {
            owner.SpinButtonDidReleaseMouseCapture(dispatch);
        }
    }
    pub fn Step(&self, amount: i32) {
        if self.ShouldRespondToMouseEvents() {
            self.DoStepAction(amount);
        }
    }
    pub fn RepeatingTimerFired(&self) {
        self.Step(if self.up_down_state.get() == UpDownState::Up {
            1
        } else {
            -1
        });
    }
    fn ShouldRespondToMouseEvents(&self) -> bool {
        self.Owner()
            .is_none_or(|owner| owner.ShouldSpinButtonRespondToMouseEvents())
    }
    fn CalculateUpDownStateByMouseLocation(&self, event: &EventRef) {
        let absolute = {
            let event = event.borrow();
            event.AsMouseEvent().map(|mouse| *mouse.AbsoluteLocation())
        };
        let Some(geometry) = absolute.and_then(|point| self.geometry.Geometry(point)) else {
            return;
        };
        let old = self.up_down_state.get();
        let up = match geometry.line_over {
            PhysicalDirection::kUp => {
                LayoutUnit::from_signed(geometry.local.y()) < geometry.size.height / 2
            }
            PhysicalDirection::kLeft => {
                LayoutUnit::from_signed(geometry.local.x()) < geometry.size.width / 2
            }
            PhysicalDirection::kRight => {
                LayoutUnit::from_signed(geometry.local.x()) >= geometry.size.width / 2
            }
            PhysicalDirection::kDown => unreachable!("source NOTREACHED: line-over is never down"),
        };
        let new = if up {
            UpDownState::Up
        } else {
            UpDownState::Down
        };
        self.up_down_state.set(new);
        if old != new {
            self.geometry.SetShouldDoFullPaintInvalidation();
        }
    }
}
