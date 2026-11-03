use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use super::hyphenation::Hyphenation;
use foundation::AtomicString;

#[allow(non_snake_case)]
pub trait HyphenationResolver {
    fn Resolve(&mut self, locale: &AtomicString) -> *mut Hyphenation;
}

pub type ResolverPointer = NonNull<dyn HyphenationResolver>;

thread_local! {
    static CURRENT_HYPHENATION_RESOLVER: Cell<Option<ResolverPointer>> = const { Cell::new(None) };
}

pub struct HyphenationResolverScope<'a> {
    previous_: Option<ResolverPointer>,
    _resolver_lifetime: PhantomData<&'a mut dyn HyphenationResolver>,
    _same_thread: PhantomData<Rc<()>>,
}

impl<'a> HyphenationResolverScope<'a> {
    pub fn new(resolver: &'a mut dyn HyphenationResolver) -> Self {
        let pointer: NonNull<dyn HyphenationResolver + 'a> = NonNull::from(resolver);
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
        resolver: *mut dyn HyphenationResolver,
    ) -> HyphenationResolverScope<'static> {
        let pointer = NonNull::new(resolver).expect("installed hyphenation resolver");
        let previous_ = CURRENT_HYPHENATION_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(pointer));
            previous
        });
        HyphenationResolverScope {
            previous_,
            _resolver_lifetime: PhantomData,
            _same_thread: PhantomData,
        }
    }
}

impl Drop for HyphenationResolverScope<'_> {
    fn drop(&mut self) {
        CURRENT_HYPHENATION_RESOLVER.with(|current| current.set(self.previous_));
    }
}

#[allow(non_snake_case)]
pub fn CurrentHyphenationResolver() -> Option<ResolverPointer> {
    CURRENT_HYPHENATION_RESOLVER.with(Cell::get)
}
