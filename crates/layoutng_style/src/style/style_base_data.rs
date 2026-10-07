use foundation::{
    CSSBitset, MakeGarbageCollected, Member, ThreadAffinity, ThreadingTrait, Visitor,
};

use super::computed_style::ComputedStyle;

impl foundation::Traceable for StyleBaseData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        StyleBaseData::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/style_base_data.h:21-53
pub struct StyleBaseData {
    computed_style_: Member<ComputedStyle>,
    important_set_: Option<Box<CSSBitset>>,
}

#[allow(non_snake_case)]
impl StyleBaseData {
    // cpp: layoutng_style/style/style_base_data.h:23-26
    pub fn Create(style: *const ComputedStyle, important_set: Option<Box<CSSBitset>>) -> *mut Self {
        MakeGarbageCollected(Self::new(style, important_set))
    }

    // cpp: layoutng_style/style/style_base_data.h:27
    // No definition exists in the supplied C++ tree.
    pub fn new(style: *const ComputedStyle, important_set: Option<Box<CSSBitset>>) -> Self {
        unsafe { StyleBaseDataConstruct(style, important_set) }
    }

    // cpp: layoutng_style/style/style_base_data.h:29-31
    pub fn GetBaseComputedStyle(&self) -> *const ComputedStyle {
        self.computed_style_.Get()
    }

    // cpp: layoutng_style/style/style_base_data.h:32
    pub fn GetBaseImportantSet(&self) -> *const CSSBitset {
        self.important_set_
            .as_deref()
            .map_or(std::ptr::null(), |set| set as *const CSSBitset)
    }

    // cpp: layoutng_style/style/style_base_data.h:34
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.computed_style_);
    }
}

// cpp: layoutng_style/style/style_base_data.h:55-58
#[allow(non_upper_case_globals)]
impl ThreadingTrait for StyleBaseData {
    const kAffinity: ThreadAffinity = ThreadAffinity::kMainThreadOnly;
}

unsafe extern "Rust" {
    fn StyleBaseDataConstruct(
        style: *const ComputedStyle,
        important_set: Option<Box<CSSBitset>>,
    ) -> StyleBaseData;
}
