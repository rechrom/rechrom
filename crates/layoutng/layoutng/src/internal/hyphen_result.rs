#![allow(non_snake_case)]

use font_engine::ShapeResult;
use foundation::{LayoutUnit, Member, String, Visitor};
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng/internal/hyphen_result.h:17-39
#[derive(Clone)]
pub struct HyphenResult {
    pub text_: String,
    pub shape_result_: Member<ShapeResult>,
}

impl Default for HyphenResult {
    // cpp: layoutng/internal/hyphen_result.h:21-21
    fn default() -> Self {
        Self {
            text_: String::default(),
            shape_result_: Member::default(),
        }
    }
}

impl HyphenResult {
    // cpp: layoutng/internal/hyphen_result.h:22-22
    pub fn new(style: &ComputedStyle) -> Self {
        let mut result = Self::default();
        result.Shape(style);
        result
    }

    // cpp: layoutng/internal/hyphen_result.h:24-24
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.shape_result_);
    }

    // cpp: layoutng/internal/hyphen_result.h:26-26
    pub fn IsPresent(&self) -> bool {
        !self.text_.IsNull()
    }

    // cpp: layoutng/internal/hyphen_result.h:28-28
    pub fn Text(&self) -> &String {
        &self.text_
    }

    // cpp: layoutng/internal/hyphen_result.h:29-29
    pub fn GetShapeResult(&self) -> &ShapeResult {
        unsafe { &*self.shape_result_.Get() }
    }

    // cpp: layoutng/internal/hyphen_result.h:30-32
    pub fn InlineSize(&self) -> LayoutUnit {
        self.GetShapeResult().SnappedWidth().ClampNegativeToZero()
    }

    // cpp: layoutng/internal/hyphen_result.h:34-34
    // Defined in //src/layoutng_inline/hyphen_result.cc. The selected
    // package retains this call but does not copy the inline package's body.
    pub fn Shape(&mut self, style: &ComputedStyle) {
        unsafe { ShapeHyphenResult(self, style) }
    }
}

unsafe extern "Rust" {
    fn ShapeHyphenResult(result: &mut HyphenResult, style: &ComputedStyle);
}
