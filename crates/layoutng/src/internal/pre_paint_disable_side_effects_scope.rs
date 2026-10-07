use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

// cpp: layoutng/internal/pre_paint_disable_side_effects_scope.h:31-32
static COUNT: AtomicU32 = AtomicU32::new(0);

// cpp: layoutng/internal/pre_paint_disable_side_effects_scope.h:13-33
pub struct PrePaintDisableSideEffectsScope {
    _thread_bound: PhantomData<Rc<()>>,
}

#[allow(non_snake_case)]
impl PrePaintDisableSideEffectsScope {
    // cpp: layoutng/internal/pre_paint_disable_side_effects_scope.h:23-23
    pub fn new() -> Self {
        COUNT.fetch_add(1, Ordering::Relaxed);
        Self {
            _thread_bound: PhantomData,
        }
    }

    // cpp: layoutng/internal/pre_paint_disable_side_effects_scope.h:29-29
    pub fn IsDisabled() -> bool {
        COUNT.load(Ordering::Relaxed) != 0
    }
}

// cpp: layoutng/internal/pre_paint_disable_side_effects_scope.h:24-27
impl Drop for PrePaintDisableSideEffectsScope {
    fn drop(&mut self) {
        let previous = COUNT.fetch_sub(1, Ordering::Relaxed);
        debug_assert!(previous != 0);
    }
}
