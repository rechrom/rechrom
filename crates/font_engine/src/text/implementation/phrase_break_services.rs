#![allow(non_snake_case)]

use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use foundation::{AtomicString, String};

pub trait PhraseBreakResolver {
    fn NextBreak(
        &self,
        locale: &AtomicString,
        text: &String,
        from_index: u32,
        range_end: u32,
    ) -> u32;
}

type ResolverPointer = NonNull<dyn PhraseBreakResolver>;

thread_local! {
    static CURRENT_PHRASE_BREAK_RESOLVER: Cell<Option<ResolverPointer>> = const { Cell::new(None) };
}

pub struct PhraseBreakResolverScope<'a> {
    previous_: Option<ResolverPointer>,
    _resolver_lifetime: PhantomData<&'a mut dyn PhraseBreakResolver>,
    _same_thread: PhantomData<Rc<()>>,
}

impl<'a> PhraseBreakResolverScope<'a> {
    pub fn new(resolver: &'a mut dyn PhraseBreakResolver) -> Self {
        let pointer: NonNull<dyn PhraseBreakResolver + 'a> = NonNull::from(resolver);
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
        resolver: *mut dyn PhraseBreakResolver,
    ) -> PhraseBreakResolverScope<'static> {
        let pointer = NonNull::new(resolver).expect("installed phrase-break resolver");
        let previous_ = CURRENT_PHRASE_BREAK_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(pointer));
            previous
        });
        PhraseBreakResolverScope {
            previous_,
            _resolver_lifetime: PhantomData,
            _same_thread: PhantomData,
        }
    }
}

impl Drop for PhraseBreakResolverScope<'_> {
    fn drop(&mut self) {
        CURRENT_PHRASE_BREAK_RESOLVER.with(|current| current.set(self.previous_));
    }
}

pub fn CurrentPhraseBreakResolver() -> Option<NonNull<dyn PhraseBreakResolver>> {
    CURRENT_PHRASE_BREAK_RESOLVER.with(Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedBreak(u32);
    impl PhraseBreakResolver for FixedBreak {
        fn NextBreak(&self, _: &AtomicString, _: &String, _: u32, _: u32) -> u32 {
            self.0
        }
    }

    #[test]
    fn nested_scopes_restore_the_prior_resolver() {
        let mut outer = FixedBreak(2);
        let mut inner = FixedBreak(4);
        let _outer = PhraseBreakResolverScope::new(&mut outer);
        let first = CurrentPhraseBreakResolver().unwrap();
        {
            let _inner = PhraseBreakResolverScope::new(&mut inner);
            let current = CurrentPhraseBreakResolver().unwrap();
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
        assert_eq!(CurrentPhraseBreakResolver(), Some(first));
    }
}
