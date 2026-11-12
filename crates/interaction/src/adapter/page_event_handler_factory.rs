//! EventHandler constructor wiring over the Page's original source owners.
//! Chromium: core/input/event_handler.cc:269-302. Construction never allocates
//! a second mouse, pointer, selection, gesture or scroll state.
use std::rc::Rc;

use crate::core::input::event_handler::{
    gesture::EventHandlerGestureTargeting, EventHandlerFactory, EventHandlerGesture,
    EventHandlerHitResult, EventHandlerKeyboard, EventHandlerMouse, EventHandlerPointer,
    EventHandlerRegistry, EventHandlerScrollPort, EventHandlerSelection, EventHandlerTimer,
    EventHandlerWheel,
};

/// Required source services for one synchronous input binding. The assembly
/// root supplies managers bound to the same ResidentCursorOwners and scroll
/// owner. No optional field can turn an absent source path into a handled event.
pub struct PageEventHandlerManagers<'scope, R: EventHandlerHitResult> {
    pub selection: Rc<dyn EventHandlerSelection<R> + 'scope>,
    pub hover_timer: Rc<dyn EventHandlerTimer + 'scope>,
    pub cursor_timer: Rc<dyn EventHandlerTimer + 'scope>,
    pub registry: Rc<dyn EventHandlerRegistry + 'scope>,
    pub scroll: Rc<dyn EventHandlerScrollPort<R> + 'scope>,
    pub mouse: Rc<dyn EventHandlerMouse<R> + 'scope>,
    pub wheel: Rc<dyn EventHandlerWheel + 'scope>,
    pub keyboard: Rc<dyn EventHandlerKeyboard + 'scope>,
    pub pointer: Rc<dyn EventHandlerPointer<'scope, R> + 'scope>,
    pub gesture: Rc<dyn EventHandlerGesture<R> + 'scope>,
    pub active_timer: Rc<dyn EventHandlerTimer + 'scope>,
    pub targeting: Rc<dyn EventHandlerGestureTargeting<'scope, R> + 'scope>,
}

pub struct PageEventHandlerFactory<'scope, R: EventHandlerHitResult> {
    managers: PageEventHandlerManagers<'scope, R>,
    local_root: bool,
}

impl<'scope, R: EventHandlerHitResult> PageEventHandlerFactory<'scope, R> {
    pub fn new(managers: PageEventHandlerManagers<'scope, R>, local_root: bool) -> Self {
        Self {
            managers,
            local_root,
        }
    }
}

impl<'scope, R: EventHandlerHitResult> EventHandlerFactory<'scope, R>
    for PageEventHandlerFactory<'scope, R>
{
    fn SelectionController(&self) -> Rc<dyn EventHandlerSelection<R> + 'scope> {
        self.managers.selection.clone()
    }
    fn HoverTimer(&self) -> Rc<dyn EventHandlerTimer + 'scope> {
        self.managers.hover_timer.clone()
    }
    fn CursorUpdateTimer(&self) -> Rc<dyn EventHandlerTimer + 'scope> {
        self.managers.cursor_timer.clone()
    }
    fn EventHandlerRegistry(&self, local_root: bool) -> Rc<dyn EventHandlerRegistry + 'scope> {
        assert_eq!(local_root, self.local_root);
        self.managers.registry.clone()
    }
    fn ScrollManager(&self) -> Rc<dyn EventHandlerScrollPort<R> + 'scope> {
        self.managers.scroll.clone()
    }
    fn MouseEventManager(
        &self,
        scroll: &Rc<dyn EventHandlerScrollPort<R> + 'scope>,
    ) -> Rc<dyn EventHandlerMouse<R> + 'scope> {
        assert!(Rc::ptr_eq(scroll, &self.managers.scroll));
        self.managers.mouse.clone()
    }
    fn MouseWheelEventManager(
        &self,
        scroll: &Rc<dyn EventHandlerScrollPort<R> + 'scope>,
    ) -> Rc<dyn EventHandlerWheel + 'scope> {
        assert!(Rc::ptr_eq(scroll, &self.managers.scroll));
        self.managers.wheel.clone()
    }
    fn KeyboardEventManager(
        &self,
        scroll: &Rc<dyn EventHandlerScrollPort<R> + 'scope>,
    ) -> Rc<dyn EventHandlerKeyboard + 'scope> {
        assert!(Rc::ptr_eq(scroll, &self.managers.scroll));
        self.managers.keyboard.clone()
    }
    fn PointerEventManager(
        &self,
        mouse: &Rc<dyn EventHandlerMouse<R> + 'scope>,
    ) -> Rc<dyn EventHandlerPointer<'scope, R> + 'scope> {
        assert!(Rc::ptr_eq(mouse, &self.managers.mouse));
        self.managers.pointer.clone()
    }
    fn GestureManager(
        &self,
        scroll: &Rc<dyn EventHandlerScrollPort<R> + 'scope>,
        mouse: &Rc<dyn EventHandlerMouse<R> + 'scope>,
        pointer: &Rc<dyn EventHandlerPointer<'scope, R> + 'scope>,
        selection: &Rc<dyn EventHandlerSelection<R> + 'scope>,
    ) -> Rc<dyn EventHandlerGesture<R> + 'scope> {
        assert!(Rc::ptr_eq(scroll, &self.managers.scroll));
        assert!(Rc::ptr_eq(mouse, &self.managers.mouse));
        assert!(Rc::ptr_eq(pointer, &self.managers.pointer));
        assert!(Rc::ptr_eq(selection, &self.managers.selection));
        self.managers.gesture.clone()
    }
    fn ActiveIntervalTimer(&self) -> Rc<dyn EventHandlerTimer + 'scope> {
        self.managers.active_timer.clone()
    }
    fn GestureTargeting(&self) -> Rc<dyn EventHandlerGestureTargeting<'scope, R> + 'scope> {
        self.managers.targeting.clone()
    }
}
