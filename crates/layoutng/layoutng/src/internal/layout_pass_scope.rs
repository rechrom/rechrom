use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::layout_algorithm_set::{FloatLayoutSupport, LayoutAlgorithmSet};
use super::layout_object_factory_set::LayoutObjectFactorySet;
use foundation::UnsupportedLayout;

// cpp: layoutng/internal/layout_pass_scope.h:28-31
// cpp: layoutng/internal/layout_pass_scope.cc:14-15
thread_local! {
    static OBJECTS: Cell<*const LayoutObjectFactorySet> = const { Cell::new(std::ptr::null()) };
}

// cpp: layoutng/internal/layout_pass_scope.h:17-33
pub struct LayoutObjectFactoryScope {
    previous_objects_: *const LayoutObjectFactorySet,
    _thread_bound: PhantomData<Rc<()>>,
}

impl Default for LayoutObjectFactoryScope {
    fn default() -> Self {
        Self::new(std::ptr::null())
    }
}

#[allow(non_snake_case)]
impl LayoutObjectFactoryScope {
    // cpp: layoutng/internal/layout_pass_scope.cc:17-22
    pub fn new(objects: *const LayoutObjectFactorySet) -> Self {
        let previous_objects = OBJECTS.with(Cell::get);
        if !objects.is_null() {
            OBJECTS.with(|current| current.set(objects));
        }
        Self {
            previous_objects_: previous_objects,
            _thread_bound: PhantomData,
        }
    }

    // cpp: layoutng/internal/layout_pass_scope.h:27-27
    pub fn Objects() -> *const LayoutObjectFactorySet {
        OBJECTS.with(Cell::get)
    }
}

// cpp: layoutng/internal/layout_pass_scope.cc:24-26
impl Drop for LayoutObjectFactoryScope {
    fn drop(&mut self) {
        OBJECTS.with(|current| current.set(self.previous_objects_));
    }
}

// cpp: layoutng/internal/layout_pass_scope.h:55-57
// cpp: layoutng/internal/layout_pass_scope.cc:12-13
thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
    static ALGORITHMS: Cell<*const LayoutAlgorithmSet> = const { Cell::new(std::ptr::null()) };
}

// cpp: layoutng/internal/layout_pass_scope.h:38-61
pub struct LayoutPassScope {
    previous_algorithms_: *const LayoutAlgorithmSet,
    object_factory_scope_: LayoutObjectFactoryScope,
}

impl Default for LayoutPassScope {
    fn default() -> Self {
        Self::new(std::ptr::null(), std::ptr::null())
    }
}

#[allow(non_snake_case)]
impl LayoutPassScope {
    pub fn with_algorithms(algorithms: *const LayoutAlgorithmSet) -> Self {
        Self::new(algorithms, std::ptr::null())
    }

    // cpp: layoutng/internal/layout_pass_scope.cc:28-36
    pub fn new(
        algorithms: *const LayoutAlgorithmSet,
        objects: *const LayoutObjectFactorySet,
    ) -> Self {
        let previous_algorithms = ALGORITHMS.with(Cell::get);
        let object_factory_scope = LayoutObjectFactoryScope::new(objects);
        DEPTH.with(|depth| depth.set(depth.get().wrapping_add(1)));
        if !algorithms.is_null() {
            ALGORITHMS.with(|current| current.set(algorithms));
        }
        Self {
            previous_algorithms_: previous_algorithms,
            object_factory_scope_: object_factory_scope,
        }
    }

    // cpp: layoutng/internal/layout_pass_scope.h:48-53
    pub fn IsActive() -> bool {
        DEPTH.with(|depth| depth.get() != 0)
    }

    pub fn Algorithms() -> *const LayoutAlgorithmSet {
        ALGORITHMS.with(Cell::get)
    }

    pub fn Objects() -> *const LayoutObjectFactorySet {
        LayoutObjectFactoryScope::Objects()
    }

    // cpp: layoutng/internal/layout_pass_scope.cc:44-51
    pub fn RequireFloatSupport() -> *const FloatLayoutSupport {
        let algorithms = Self::Algorithms();
        if algorithms.is_null()
            || unsafe { &*algorithms }
                .float_support
                .margin_box_inline_size
                .is_none()
            || unsafe { &*algorithms }.float_support.position.is_none()
            || unsafe { &*algorithms }.float_support.create_shape.is_none()
        {
            std::panic::panic_any(UnsupportedLayout::new(
                "float layout module is not installed",
            ));
        }
        unsafe { std::ptr::addr_of!((*algorithms).float_support) }
    }
}

// cpp: layoutng/internal/layout_pass_scope.cc:38-42
impl Drop for LayoutPassScope {
    fn drop(&mut self) {
        DEPTH.with(|depth| {
            debug_assert!(depth.get() != 0);
            depth.set(depth.get().wrapping_sub(1));
        });
        ALGORITHMS.with(|current| current.set(self.previous_algorithms_));
        // object_factory_scope_ drops after this body and restores OBJECTS.
    }
}
