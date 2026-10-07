#![allow(non_snake_case)]

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use font_engine::{Font, TextRenderingMode};
use foundation::{
    DowncastFrom, MakeGarbageCollected, Member, String as BlinkString, Traceable,
    UnsupportedLayout, Visitor, WritingMode,
};
use layoutng_assembly::internal::layout_font_resolver::{
    CurrentNativeFontResolver, NativeFontRequest,
};
use layoutng_assembly::internal::layout_input::{
    ComputedStyle as InputComputedStyle, ExtendedStyle, WritingMode as InputWritingMode,
};
use layoutng_assembly::internal::layout_node_metadata::Node;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::layout_pass_scope::LayoutPassScope;
use layoutng_assembly::internal::layout_text::LayoutText;

// cpp: layoutng_svg/svg_layout_objects.cc:22-26
fn NormalizeSvgWhitespace(text: BlinkString) -> BlinkString {
    if text.IsNull() {
        return text;
    }
    if let Some(bytes) = text.Span8() {
        return BlinkString::from_latin1(
            &bytes
                .iter()
                .map(|byte| match byte {
                    b'\t' | b'\n' | b'\r' => b' ',
                    other => *other,
                })
                .collect::<Vec<_>>(),
        );
    }
    BlinkString::from_utf16(
        &text
            .Span16()
            .unwrap_or_default()
            .iter()
            .map(|unit| match unit {
                9 | 10 | 13 => 32,
                other => *other,
            })
            .collect::<Vec<_>>(),
    )
}

// cpp: layoutng_svg/svg_layout_objects.cc:28-39
fn PublicWritingMode(mode: WritingMode) -> InputWritingMode {
    match mode {
        WritingMode::kHorizontalTb => InputWritingMode::kHorizontalTb,
        WritingMode::kVerticalRl | WritingMode::kSidewaysRl => InputWritingMode::kVerticalRl,
        WritingMode::kVerticalLr | WritingMode::kSidewaysLr => InputWritingMode::kVerticalLr,
    }
}

// cpp: layoutng_svg/svg_layout_objects.cc:41-51
fn InputStyleForFont(object: &LayoutObject) -> &InputComputedStyle {
    let node = object.GetNode();
    assert!(!node.is_null());
    let mut ancestor = unsafe { &*node }.parentNode().cast::<Node>();
    while !ancestor.is_null() {
        let ancestor_ref = unsafe { &*ancestor };
        if !ancestor_ref.GetLayoutObject().is_null() {
            return ancestor_ref.InputStyle();
        }
        ancestor = ancestor_ref.parentNode().cast::<Node>();
    }
    unsafe { &*node }.InputStyle()
}

// cpp: layoutng_svg/layout_svg_inline_text.h:10-30
#[repr(C)]
pub struct LayoutSVGInlineText {
    text: LayoutText,
    scaling_factor: f32,
    scaled_font: Cell<Member<Font>>,
}

impl LayoutSVGInlineText {
    // cpp: layoutng_svg/svg_layout_objects.cc:145-146
    pub fn new(node: *mut Node, text: BlinkString) -> Self {
        let text = LayoutText::new(node, NormalizeSvgWhitespace(text));
        text.SetRuntimeClass(LayoutObjectClass::SvgInlineText);
        Self {
            text,
            scaling_factor: 1.0,
            scaled_font: Cell::new(Member::default()),
        }
    }

    // cpp: layoutng_svg/layout_svg_inline_text.h:15-18
    pub fn ScalingFactor(&self) -> f32 {
        self.scaling_factor
    }
    pub fn SvgScalingFactor(&self) -> f32 {
        self.scaling_factor
    }
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGInlineText"
    }
    pub fn IsSVG(&self) -> bool {
        true
    }
    pub fn IsSVGInlineText(&self) -> bool {
        true
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:153-157
    pub fn ScaledFont(&self) -> &Font {
        let mut font = self.scaled_font.get().Get();
        if font.is_null() {
            font = MakeGarbageCollected(unsafe { &*self.StyleRef().GetFont() }.clone());
            self.scaled_font.set(Member::from_ptr(font));
        }
        unsafe { &*font }
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:159-173
    pub fn ComputeFontScale(object: &LayoutObject) -> f32 {
        if object.StyleRef().GetFontDescription().TextRendering()
            == TextRenderingMode::kGeometricPrecision
        {
            return 1.0;
        }
        let algorithms = LayoutPassScope::Algorithms();
        let callback = unsafe { algorithms.as_ref() }
            .and_then(|algorithms| algorithms.svg_support.screen_font_scale)
            .unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "SVG font scaling module is not installed",
                ))
            });
        let factor = callback(object);
        if factor > 0.0 && factor.is_finite() {
            factor
        } else {
            1.0
        }
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:174-197
    pub fn ComputeNewScaledFontForStyle(
        object: &LayoutObject,
        scaling_factor: &mut f32,
    ) -> *const Font {
        *scaling_factor = Self::ComputeFontScale(object);
        let style = object.StyleRef();
        if *scaling_factor == 1.0 {
            return MakeGarbageCollected(unsafe { &*style.GetFont() }.clone());
        }
        let input = InputStyleForFont(object);
        let defaults = ExtendedStyle::default();
        let extra = input.extended.as_ref().unwrap_or(&defaults);
        let description = style.GetFontDescription();
        let factor = *scaling_factor as f64;
        let request = NativeFontRequest::new_auto(
            description.ComputedSize() as f64 * factor,
            description.LetterSpacing() as f64 * factor,
            description.WordSpacing() as f64 * factor,
            PublicWritingMode(style.GetWritingMode()),
            &extra.language,
            &extra.font_families,
            extra.font_weight,
            extra.font_italic,
            description.Orientation(),
        );
        unsafe { &mut *CurrentNativeFontResolver() }.Resolve(&request) as *const Font
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:199-201
    pub fn UpdateScaledFont(&mut self) {
        let mut factor = self.scaling_factor;
        let font = Self::ComputeNewScaledFontForStyle(self, &mut factor);
        self.scaling_factor = factor;
        self.scaled_font.set(Member::from_ptr(font.cast_mut()));
    }
}

impl Deref for LayoutSVGInlineText {
    type Target = LayoutText;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}
impl DerefMut for LayoutSVGInlineText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGInlineText, text) == 0);
impl Traceable for LayoutSVGInlineText {
    // cpp: layoutng_svg/svg_layout_objects.cc:148-151
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.scaled_font.get());
        self.text.Trace(visitor);
    }
}
impl DowncastFrom<LayoutObject> for LayoutSVGInlineText {
    // cpp: layoutng_svg/layout_svg_inline_text.h:32-37
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGInlineText()
    }
}

pub fn SvgInlineTextUpdateScaledFont(object: &mut LayoutObject) {
    unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGInlineText>() }.UpdateScaledFont()
}
pub fn SvgInlineTextScaledFont(object: &LayoutObject) -> *const Font {
    unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGInlineText>() }.ScaledFont()
}
pub fn SvgInlineTextScalingFactor(object: &LayoutObject) -> f32 {
    unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGInlineText>() }.SvgScalingFactor()
}
pub fn ComputeScaledSvgFont(object: &LayoutObject, factor: &mut f32) -> *const Font {
    LayoutSVGInlineText::ComputeNewScaledFontForStyle(object, factor)
}

#[cfg(test)]
mod tests {
    use super::NormalizeSvgWhitespace;
    use foundation::String as BlinkString;

    #[test]
    fn whitespace_normalization_preserves_storage_width_and_utf16_offsets() {
        let latin1 = NormalizeSvgWhitespace(BlinkString::from_latin1(&[b'A', b'\t', 0xe9]));
        assert!(latin1.Is8Bit());
        assert_eq!(latin1.Span8(), Some(&[b'A', b' ', 0xe9][..]));

        let utf16 = NormalizeSvgWhitespace(BlinkString::from_utf16(&[0xd800, 10, 0x4e2d, 13]));
        assert!(!utf16.Is8Bit());
        assert_eq!(utf16.Span16(), Some(&[0xd800, 32, 0x4e2d, 32][..]));
    }
}
