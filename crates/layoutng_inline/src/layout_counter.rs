// C++: layoutng_inline/layout_counter.h/.cc. The header's declarations with
// no definitions in the supplied C++ checkout remain unresolved below.
#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{
    g_null_atom, keywords, AtomicString, DowncastFrom, DynamicTo, Member, String, Traceable,
    Vector, Visitor,
};
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::content_data::CounterContentData;
use layoutng_style::style::forward::CounterStyle;

// cpp: layoutng_inline/layout_counter.h:38-38,83-84
#[repr(C)]
pub struct LayoutCounter {
    text_: LayoutText,
    counter_: Member<CounterContentData>,
}

const _: () = assert!(std::mem::offset_of!(LayoutCounter, text_) == 0);

impl Deref for LayoutCounter {
    type Target = LayoutText;
    fn deref(&self) -> &Self::Target {
        &self.text_
    }
}
impl DerefMut for LayoutCounter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text_
    }
}

// cpp: layoutng_inline/layout_counter.h:86-91
impl DowncastFrom<LayoutObject> for LayoutCounter {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsCounter()
    }
}

// cpp: layoutng_inline/layout_counter.h:40-41
// The C++ checkout declares but does not define these lifecycle methods.
// Typed calls preserve the missing implementation as a link dependency.
impl Drop for LayoutCounter {
    fn drop(&mut self) {
        unsafe { LayoutCounterDestructFromInline(self) }
    }
}

impl Traceable for LayoutCounter {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        unsafe { LayoutCounterTraceFromInline(self, visitor) }
    }
}

#[allow(non_snake_case)]
impl LayoutCounter {
    // cpp: layoutng_inline/layout_counter.h:43-46
    pub fn Identifier(&self) -> &AtomicString {
        self.CheckIsNotDestroyed();
        unsafe { &*self.counter_.Get() }.Identifier()
    }

    // cpp: layoutng_inline/layout_counter.h:50-54
    pub fn GenerateCounterText(
        counter_values: Vector<i32>,
        counter_style: *const CounterStyle,
        separator: &AtomicString,
    ) -> String {
        unsafe {
            LayoutCounterGenerateCounterTextFromInline(counter_values, counter_style, separator)
        }
    }
    pub fn UpdateCounter(&mut self, counter_values: Vector<i32>) {
        unsafe { LayoutCounterUpdateCounterFromInline(self, counter_values) }
    }

    // cpp: layoutng_inline/layout_counter.cc:12-17
    pub fn IsDirectionalSymbolMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        let name = unsafe { &*self.counter_.Get() }.ListStyle();
        name == &*keywords::kDisclosureOpen || name == &*keywords::kDisclosureClosed
    }

    // cpp: layoutng_inline/layout_counter.cc:19-22
    pub fn Separator(&self) -> &AtomicString {
        self.CheckIsNotDestroyed();
        unsafe { &*self.counter_.Get() }.Separator()
    }

    // cpp: layoutng_inline/layout_counter.cc:24-34
    pub fn ListStyle(object: *const LayoutObject, style: &ComputedStyle) -> AtomicString {
        let counter = DynamicTo::<LayoutCounter>(object);
        if !counter.is_null() {
            return unsafe { &*(&*counter).counter_.Get() }.ListStyle().clone();
        }
        let list_style_type = style.ListStyleType().Get();
        if !list_style_type.is_null() {
            let list_style_type = unsafe { &*list_style_type };
            if list_style_type.IsCounterStyle() {
                return list_style_type.GetCounterStyleName().clone();
            }
        }
        g_null_atom.clone()
    }

    // cpp: layoutng_inline/layout_counter.h:67-70
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutCounter"
    }

    // cpp: layoutng_inline/layout_counter.h:73-73
    pub(crate) fn WillBeDestroyed(&mut self) {
        unsafe { LayoutCounterWillBeDestroyedFromInline(self) }
    }

    // cpp: layoutng_inline/layout_counter.h:76-79
    pub fn IsCounter(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng_inline/layout_counter.h:81-81
    fn NullableCounterStyle(&self) -> *const CounterStyle {
        unsafe { LayoutCounterNullableCounterStyleFromInline(self) }
    }
}

unsafe extern "Rust" {
    fn LayoutCounterDestructFromInline(counter: &mut LayoutCounter);
    fn LayoutCounterTraceFromInline(counter: &LayoutCounter, visitor: &mut Visitor<'_>);
    fn LayoutCounterGenerateCounterTextFromInline(
        counter_values: Vector<i32>,
        counter_style: *const CounterStyle,
        separator: &AtomicString,
    ) -> String;
    fn LayoutCounterUpdateCounterFromInline(counter: &mut LayoutCounter, values: Vector<i32>);
    fn LayoutCounterWillBeDestroyedFromInline(counter: &mut LayoutCounter);
    fn LayoutCounterNullableCounterStyleFromInline(counter: &LayoutCounter) -> *const CounterStyle;
}
