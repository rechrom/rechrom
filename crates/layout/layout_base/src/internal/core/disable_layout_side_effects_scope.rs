use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

// cpp: layoutng/internal/disable_layout_side_effects_scope.h:29-30
// cpp: layoutng/internal/layout_pass_scope.cc:11-11
thread_local! {
    static COUNT: Cell<u32> = const { Cell::new(0) };
}

// cpp: layoutng/internal/disable_layout_side_effects_scope.h:13-31
pub struct DisableLayoutSideEffectsScope {
    // The C++ scope is stack allocated and tied to its constructing thread.
    _thread_bound: PhantomData<Rc<()>>,
}

#[allow(non_snake_case)]
impl DisableLayoutSideEffectsScope {
    // cpp: layoutng/internal/disable_layout_side_effects_scope.h:21-21
    pub fn new() -> Self {
        COUNT.with(|count| count.set(count.get().wrapping_add(1)));
        Self {
            _thread_bound: PhantomData,
        }
    }

    // cpp: layoutng/internal/disable_layout_side_effects_scope.h:27-27
    pub fn IsDisabled() -> bool {
        COUNT.with(|count| count.get() != 0)
    }
}

// cpp: layoutng/internal/disable_layout_side_effects_scope.h:22-25
impl Drop for DisableLayoutSideEffectsScope {
    fn drop(&mut self) {
        COUNT.with(|count| {
            debug_assert!(count.get() != 0);
            count.set(count.get().wrapping_sub(1));
        });
    }
}
