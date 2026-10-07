// C++: font_engine/text/native/hyphenation_services.h/.cc
use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use super::hyphenation::Hyphenation;
use foundation::AtomicString;

// cpp: font_engine/text/native/hyphenation_services.h:12-16
#[allow(non_snake_case)]
pub trait NativeHyphenationResolver {
    fn Resolve(&mut self, locale: &AtomicString) -> *mut Hyphenation;
}

pub type ResolverPointer = NonNull<dyn NativeHyphenationResolver>;

// cpp: font_engine/text/native/hyphenation_services.cc:7
thread_local! {
    static CURRENT_HYPHENATION_RESOLVER: Cell<Option<ResolverPointer>> = const { Cell::new(None) };
}

// cpp: font_engine/text/native/hyphenation_services.h:18-28
pub struct NativeHyphenationResolverScope<'a> {
    previous_: Option<ResolverPointer>,
    _resolver_lifetime: PhantomData<&'a mut dyn NativeHyphenationResolver>,
    _same_thread: PhantomData<Rc<()>>,
}

impl<'a> NativeHyphenationResolverScope<'a> {
    // cpp: font_engine/text/native/hyphenation_services.cc:10-14
    pub fn new(resolver: &'a mut dyn NativeHyphenationResolver) -> Self {
        let pointer: NonNull<dyn NativeHyphenationResolver + 'a> = NonNull::from(resolver);
        // SAFETY: this scope retains the resolver lifetime and is !Send.
        let pointer: ResolverPointer = unsafe { std::mem::transmute(pointer) };
        let previous_ = CURRENT_HYPHENATION_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(pointer));
            previous
        });
        Self {
            previous_,
            _resolver_lifetime: PhantomData,
            _same_thread: PhantomData,
        }
    }

    // Rust-only boundary helper: the caller retains the boxed resolver and
    // drops this scope before that box, matching the C++ stack lifetime.
    pub unsafe fn new_from_raw(
        resolver: *mut dyn NativeHyphenationResolver,
    ) -> NativeHyphenationResolverScope<'static> {
        let pointer = NonNull::new(resolver).expect("native hyphenation resolver");
        let previous_ = CURRENT_HYPHENATION_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(pointer));
            previous
        });
        NativeHyphenationResolverScope {
            previous_,
            _resolver_lifetime: PhantomData,
            _same_thread: PhantomData,
        }
    }
}

// cpp: font_engine/text/native/hyphenation_services.cc:16-18
impl Drop for NativeHyphenationResolverScope<'_> {
    fn drop(&mut self) {
        CURRENT_HYPHENATION_RESOLVER.with(|current| current.set(self.previous_));
    }
}

// cpp: font_engine/text/native/hyphenation_services.cc:20-22
#[allow(non_snake_case)]
pub fn CurrentNativeHyphenationResolver() -> Option<ResolverPointer> {
    CURRENT_HYPHENATION_RESOLVER.with(Cell::get)
}
