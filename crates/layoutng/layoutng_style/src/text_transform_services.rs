use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;

use foundation::{AtomicString, ETextTransform, String, TextOffsetMap};

// cpp: layoutng_style/text_transform_services.h:14-22
#[allow(non_snake_case)]
pub trait TextTransformResolver {
    fn Transform(
        &self,
        transform: ETextTransform,
        locale: &AtomicString,
        text: &String,
        previous_character: u16,
        offset_map: Option<&mut TextOffsetMap>,
    ) -> String;
}

pub type ResolverPointer = NonNull<dyn TextTransformResolver>;

// cpp: layoutng_style/text_transform_services.cc:5-8
thread_local! {
    static CURRENT_TEXT_TRANSFORM_RESOLVER: Cell<Option<ResolverPointer>> = const { Cell::new(None) };
}

// cpp: layoutng_style/text_transform_services.h:24-35
pub struct TextTransformResolverScope<'a> {
    previous_: Option<ResolverPointer>,
    _resolver_lifetime: PhantomData<&'a mut dyn TextTransformResolver>,
    _same_thread: PhantomData<Rc<()>>,
}

impl<'a> TextTransformResolverScope<'a> {
    // cpp: layoutng_style/text_transform_services.h:26
    // cpp: layoutng_style/text_transform_services.cc:10-14
    pub fn new(resolver: &'a mut dyn TextTransformResolver) -> Self {
        let pointer: NonNull<dyn TextTransformResolver + 'a> = NonNull::from(resolver);
        // SAFETY: the scope holds the resolver lifetime and restores the
        // previous pointer on this same thread when dropped.
        let pointer: ResolverPointer = unsafe { std::mem::transmute(pointer) };
        let previous = CURRENT_TEXT_TRANSFORM_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(pointer));
            previous
        });
        Self {
            previous_: previous,
            _resolver_lifetime: PhantomData,
            _same_thread: PhantomData,
        }
    }

    // Rust-only boundary helper for a resolver owned by a stable box in the
    // layout environment. The caller must drop this scope before the box.
    pub unsafe fn new_from_raw(
        resolver: *mut dyn TextTransformResolver,
    ) -> TextTransformResolverScope<'static> {
        let pointer = NonNull::new(resolver).expect("source text transform resolver");
        let previous_ = CURRENT_TEXT_TRANSFORM_RESOLVER.with(|current| {
            let previous = current.get();
            current.set(Some(pointer));
            previous
        });
        TextTransformResolverScope {
            previous_,
            _resolver_lifetime: PhantomData,
            _same_thread: PhantomData,
        }
    }
}

// cpp: layoutng_style/text_transform_services.h:27
// cpp: layoutng_style/text_transform_services.cc:16-18
impl Drop for TextTransformResolverScope<'_> {
    fn drop(&mut self) {
        CURRENT_TEXT_TRANSFORM_RESOLVER.with(|current| current.set(self.previous_));
    }
}

// cpp: layoutng_style/text_transform_services.h:37
// cpp: layoutng_style/text_transform_services.cc:20-22
#[allow(non_snake_case)]
pub fn CurrentTextTransformResolver() -> Option<ResolverPointer> {
    CURRENT_TEXT_TRANSFORM_RESOLVER.with(Cell::get)
}
