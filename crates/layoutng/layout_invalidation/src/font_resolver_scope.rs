//! Per-pass source font resolver scope used while projecting layout input.

#![allow(non_snake_case)]

use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::internal::layout_font_resolver::FontResolver;

// cpp: layoutng/internal/boundary/input.cc:28-31
thread_local! {
    static CURRENT_FONT_RESOLVER: Cell<Option<NonNull<dyn FontResolver>>> = Cell::new(None);
}

// cpp: layoutng/internal/boundary/input.h:25-37
pub struct FontResolverScope<'a> {
    previous_: Option<NonNull<dyn FontResolver>>,
    resolver_lifetime_: PhantomData<&'a mut dyn FontResolver>,
}

impl<'a> FontResolverScope<'a> {
    // cpp: layoutng/internal/boundary/input.cc:33-36
    pub fn new(resolver: &'a mut dyn FontResolver) -> Self {
        // The scope owns the borrow for its whole lifetime. The thread-local
        // slot cannot express that lifetime, so erase it only while the scope
        // is alive and restore the previous pointer on drop.
        let resolver = NonNull::from(resolver);
        let resolver = unsafe {
            std::mem::transmute::<NonNull<dyn FontResolver + 'a>, NonNull<dyn FontResolver>>(
                resolver,
            )
        };
        let previous_ = CURRENT_FONT_RESOLVER.with(|current| current.replace(Some(resolver)));
        Self {
            previous_,
            resolver_lifetime_: PhantomData,
        }
    }

    // Rust-only boundary helper: the standalone tree builder must call
    // PrepareStyle through this same resolver while its TLS scope lives.
    // The caller owns the stable resolver allocation and drops this scope
    // before it can drop or move that allocation.
    pub(crate) unsafe fn new_from_raw(
        resolver: *mut dyn FontResolver,
    ) -> FontResolverScope<'static> {
        let resolver = NonNull::new(resolver).expect("source font resolver");
        let previous_ = CURRENT_FONT_RESOLVER.with(|current| current.replace(Some(resolver)));
        FontResolverScope {
            previous_,
            resolver_lifetime_: PhantomData,
        }
    }
}

impl Drop for FontResolverScope<'_> {
    // cpp: layoutng/internal/boundary/input.cc:38-41
    fn drop(&mut self) {
        CURRENT_FONT_RESOLVER.with(|current| {
            assert!(current.get().is_some(), "current_font_resolver");
            current.set(self.previous_);
        });
    }
}

// cpp: layoutng/internal/boundary/input.cc:43-46
pub fn CurrentFontResolver() -> *mut dyn FontResolver {
    CURRENT_FONT_RESOLVER.with(|current| current.get().expect("current_font_resolver").as_ptr())
}
