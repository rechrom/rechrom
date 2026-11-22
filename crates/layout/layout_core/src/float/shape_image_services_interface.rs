#![allow(non_snake_case)]

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::internal::layout_input::PaintImage;

// cpp: layoutng/internal/shapes/shape_image_services.h:16-20
pub trait ShapeImageResolver {
    fn Resolve(&self, resource_id: u64) -> *const PaintImage;
}

pub type ResolverPtr = *const (dyn ShapeImageResolver + 'static);

// cpp: layoutng/internal/boundary/shape_image_resolver_scope.cc:7-9
thread_local! {
    static CURRENT_SHAPE_IMAGE_RESOLVER: Cell<Option<ResolverPtr>> = const { Cell::new(None) };
}

// cpp: layoutng/internal/shapes/shape_image_services.h:22-32
pub struct ShapeImageResolverScope<'a> {
    previous_: Option<ResolverPtr>,
    _resolver: PhantomData<&'a dyn ShapeImageResolver>,
    _thread_bound: PhantomData<Rc<()>>,
}

impl<'a> ShapeImageResolverScope<'a> {
    // cpp: layoutng/internal/shapes/shape_image_services.h:24-24
    // cpp: layoutng/internal/boundary/shape_image_resolver_scope.cc:11-15
    pub fn new(resolver: &'a dyn ShapeImageResolver) -> Self {
        // The TLS slot stores a raw pointer while this scope is alive; its
        // lifetime is erased only for storage and no safe dereference escapes.
        let pointer: *const (dyn ShapeImageResolver + 'a) = resolver;
        let pointer: ResolverPtr = unsafe { std::mem::transmute(pointer) };
        let previous_ = CURRENT_SHAPE_IMAGE_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(pointer));
            previous
        });
        Self {
            previous_,
            _resolver: PhantomData,
            _thread_bound: PhantomData,
        }
    }

    // Rust-only boundary helper for a resolver owned by a stable box in the
    // layout environment. The caller must drop this scope before the box.
    pub(crate) unsafe fn new_from_raw(
        resolver: *const dyn ShapeImageResolver,
    ) -> ShapeImageResolverScope<'static> {
        assert!(!resolver.is_null(), "source shape image resolver");
        let previous_ = CURRENT_SHAPE_IMAGE_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(resolver));
            previous
        });
        ShapeImageResolverScope {
            previous_,
            _resolver: PhantomData,
            _thread_bound: PhantomData,
        }
    }
}

// cpp: layoutng/internal/shapes/shape_image_services.h:25-25
// cpp: layoutng/internal/boundary/shape_image_resolver_scope.cc:17-19
impl Drop for ShapeImageResolverScope<'_> {
    fn drop(&mut self) {
        CURRENT_SHAPE_IMAGE_RESOLVER.with(|current| current.set(self.previous_));
    }
}

// cpp: layoutng/internal/shapes/shape_image_services.h:34-34
// cpp: layoutng/internal/boundary/shape_image_resolver_scope.cc:21-23
pub fn CurrentShapeImageResolver() -> Option<ResolverPtr> {
    CURRENT_SHAPE_IMAGE_RESOLVER.with(Cell::get)
}

// cpp: layoutng/internal/shapes/shape_image_services.h:35-35
// Defined in //src/layoutng_float/shape_image_services.cc, outside the
// selected packages. Its non-null result remains an external contract.
unsafe extern "Rust" {
    pub fn CurrentShapeImage(resource_id: u64) -> *const PaintImage;
}
