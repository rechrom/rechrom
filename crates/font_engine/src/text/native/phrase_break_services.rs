#![allow(non_snake_case)]

use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use foundation::{AtomicString, String};

// cpp: font_engine/text/native/phrase_break_services.h:10-17
pub trait NativePhraseBreakResolver {
    fn NextBreak(
        &self,
        locale: &AtomicString,
        text: &String,
        from_index: u32,
        range_end: u32,
    ) -> u32;
}

type ResolverPointer = NonNull<dyn NativePhraseBreakResolver>;

// cpp: font_engine/text/native/phrase_break_services.cc:7-8
thread_local! {
    static CURRENT_PHRASE_BREAK_RESOLVER: Cell<Option<ResolverPointer>> = const { Cell::new(None) };
}

// cpp: font_engine/text/native/phrase_break_services.h:19-30
pub struct NativePhraseBreakResolverScope<'a> {
    previous_: Option<ResolverPointer>,
    _resolver_lifetime: PhantomData<&'a mut dyn NativePhraseBreakResolver>,
    _same_thread: PhantomData<Rc<()>>,
}

impl<'a> NativePhraseBreakResolverScope<'a> {
    // cpp: font_engine/text/native/phrase_break_services.cc:10-14
    pub fn new(resolver: &'a mut dyn NativePhraseBreakResolver) -> Self {
        let pointer: NonNull<dyn NativePhraseBreakResolver + 'a> = NonNull::from(resolver);
        // SAFETY: this scope retains the resolver lifetime and is !Send.
        let pointer: ResolverPointer = unsafe { std::mem::transmute(pointer) };
        let previous_ = CURRENT_PHRASE_BREAK_RESOLVER.with(|current| {
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

    // The layout boundary owns the boxed resolver through this scope.
    pub unsafe fn new_from_raw(
        resolver: *mut dyn NativePhraseBreakResolver,
    ) -> NativePhraseBreakResolverScope<'static> {
        let pointer = NonNull::new(resolver).expect("native phrase-break resolver");
        let previous_ = CURRENT_PHRASE_BREAK_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(pointer));
            previous
        });
        NativePhraseBreakResolverScope {
            previous_,
            _resolver_lifetime: PhantomData,
            _same_thread: PhantomData,
        }
    }
}

// cpp: font_engine/text/native/phrase_break_services.cc:16-18
impl Drop for NativePhraseBreakResolverScope<'_> {
    fn drop(&mut self) {
        CURRENT_PHRASE_BREAK_RESOLVER.with(|current| current.set(self.previous_));
    }
}

// cpp: font_engine/text/native/phrase_break_services.cc:20-25
pub fn CurrentNativePhraseBreakResolver() -> Option<NonNull<dyn NativePhraseBreakResolver>> {
    CURRENT_PHRASE_BREAK_RESOLVER.with(Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedBreak(u32);
    impl NativePhraseBreakResolver for FixedBreak {
        fn NextBreak(&self, _: &AtomicString, _: &String, _: u32, _: u32) -> u32 {
            self.0
        }
    }

    #[test]
    fn nested_scopes_restore_the_prior_resolver() {
        let mut outer = FixedBreak(2);
        let mut inner = FixedBreak(4);
        let _outer = NativePhraseBreakResolverScope::new(&mut outer);
        let first = CurrentNativePhraseBreakResolver().unwrap();
        {
            let _inner = NativePhraseBreakResolverScope::new(&mut inner);
            let current = CurrentNativePhraseBreakResolver().unwrap();
            assert_eq!(
                unsafe { current.as_ref() }.NextBreak(
                    &AtomicString::default(),
                    &String::new(),
                    0,
                    0
                ),
                4
            );
        }
        assert_eq!(CurrentNativePhraseBreakResolver(), Some(first));
    }
}
