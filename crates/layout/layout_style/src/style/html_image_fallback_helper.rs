//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: core/html/html_image_fallback_helper.cc:23-210,245-283;
//! core/html/forms/image_input_type.cc:271-281.
#![allow(non_snake_case)]
use super::{
    computed_style::ComputedStyleBuilder, computed_style_constants::EVerticalAlign,
    computed_style_initial_values::ComputedStyleInitialValues,
    style_ua_shadow_host_data::StyleUAShadowHostData,
};
use crate::css::style_color::StyleColor;
use foundation::{
    AtomicString, CSSValueID, Color, EBorderStyle, EBoxSizing, EDisplay, EFloat, EOverflow,
    EPointerEvents, Length, Member, String, TextDirection,
};
pub struct ImageFallbackHostFacts<'a> {
    pub quirks: bool,
    pub has_author_shadow: bool,
    pub has_ua_shadow: bool,
    pub alt_text: &'a String,
    pub alt_attribute: &'a AtomicString,
    pub src_attribute: &'a AtomicString,
}
pub fn TreatImageAsReplaced(quirks: bool, data: &StyleUAShadowHostData) -> bool {
    let dimensions = !data.Width().IsAuto() && !data.Height().IsAuto();
    let aspect_dimensions =
        !data.AspectRatio().IsAuto() && (!data.Width().IsAuto() || !data.Height().IsAuto());
    (dimensions || aspect_dimensions) && (quirks || data.AltAttribute().empty())
}
fn ImageSmallerThanAltImage(width: &Length, height: &Length) -> bool {
    if !width.IsFixed() && !height.IsFixed() {
        return false;
    }
    (height.IsFixed() && height.Pixels() < 18.0) || (width.IsFixed() && width.Pixels() < 18.0)
}
fn ImageRepresentsNothing(data: &StyleUAShadowHostData) -> bool {
    let alt_is_set = !data.AltText().IsNull();
    let alt_is_empty = alt_is_set && data.AltText().empty();
    let src_is_set = !data.SrcAttribute().empty();
    if src_is_set && alt_is_empty {
        return true;
    }
    !src_is_set && (!alt_is_set || alt_is_empty)
}
pub fn AdjustImageInputStyle(
    fallback: bool,
    facts: &ImageFallbackHostFacts<'_>,
    builder: &mut ComputedStyleBuilder,
) {
    if !fallback {
        builder.SetUAShadowHostData(None);
        return;
    }
    AdjustHostStyle(facts, builder);
}
pub fn AdjustHostStyle(facts: &ImageFallbackHostFacts<'_>, builder: &mut ComputedStyleBuilder) {
    if facts.has_author_shadow || !facts.has_ua_shadow {
        builder.SetUAShadowHostData(None);
        return;
    }
    if facts.quirks {
        let width = builder.Width().clone();
        let height = builder.Height().clone();
        if !width.IsAuto() && height.IsAuto() {
            builder.SetHeight(&width);
        } else if !height.IsAuto() && width.IsAuto() {
            builder.SetWidth(&height);
        }
    }
    let width = builder.Width().clone();
    let height = builder.Height().clone();
    let max_width = builder.MaxWidth().clone();
    let max_height = builder.MaxHeight().clone();
    let aspect_ratio = builder.AspectRatio().clone();
    builder.SetUAShadowHostData(Some(Box::new(StyleUAShadowHostData::new(
        &width,
        &height,
        &max_width,
        &max_height,
        &aspect_ratio,
        facts.alt_text,
        facts.alt_attribute,
        facts.src_attribute,
        false,
    ))));
    let data = unsafe { &*builder.UAShadowHostData() };
    if !TreatImageAsReplaced(facts.quirks, data) && builder.Display() == EDisplay::kInline {
        builder.SetWidth(&Length::default());
        builder.SetHeight(&Length::default());
        builder.SetAspectRatio(&ComputedStyleInitialValues::InitialAspectRatio());
    }
}
pub fn AdjustAltTextContainerStyle(quirks: bool, builder: &mut ComputedStyleBuilder) {
    let pointer = builder.UAShadowHostData();
    if pointer.is_null() {
        return;
    }
    // The inherited snapshot remains immutable while the builder COWs fields.
    let data = unsafe { &*pointer };
    if quirks && !data.Width().IsAuto() && !data.Height().IsAuto() {
        builder.SetVerticalAlign(EVerticalAlign::kBaseline);
    }
    if !TreatImageAsReplaced(quirks, data) {
        return;
    }
    builder.SetOverflowX(EOverflow::kHidden);
    builder.SetOverflowY(EOverflow::kHidden);
    builder.SetDisplay(EDisplay::kFlowRoot);
    builder.SetPointerEvents(EPointerEvents::kNone);
    builder.SetMaxHeight(data.MaxHeight());
    builder.SetHeight(data.Height());
    builder.SetWidth(data.Width());
    builder.SetMaxWidth(data.MaxWidth());
    builder.SetBaseTextDecorationData(Member::default());
    if !ImageSmallerThanAltImage(data.Width(), data.Height()) {
        let width = builder.EffectiveZoom() as i32;
        builder.SetBorderTopWidth(&width);
        builder.SetBorderRightWidth(&width);
        builder.SetBorderBottomWidth(&width);
        builder.SetBorderLeftWidth(&width);
        builder.SetBorderTopStyle(EBorderStyle::kSolid);
        builder.SetBorderRightStyle(EBorderStyle::kSolid);
        builder.SetBorderBottomStyle(EBorderStyle::kSolid);
        builder.SetBorderLeftStyle(EBorderStyle::kSolid);
        let color =
            StyleColor::from_color_and_keyword(Color::FromRGB(192, 192, 192), CSSValueID::kSilver);
        builder.SetBorderTopColor(&color);
        builder.SetBorderRightColor(&color);
        builder.SetBorderBottomColor(&color);
        builder.SetBorderLeftColor(&color);
        let padding = Length::Fixed(builder.EffectiveZoom());
        builder.SetPaddingTop(&padding);
        builder.SetPaddingRight(&padding);
        builder.SetPaddingBottom(&padding);
        builder.SetPaddingLeft(&padding);
        builder.SetBoxSizing(EBoxSizing::kBorderBox);
    }
}
pub fn AdjustAltTextImageStyle(quirks: bool, builder: &mut ComputedStyleBuilder) {
    let pointer = builder.UAShadowHostData();
    if pointer.is_null() {
        return;
    }
    let data = unsafe { &*pointer };
    let hide = if TreatImageAsReplaced(quirks, data) {
        ImageSmallerThanAltImage(data.Width(), data.Height())
    } else {
        ImageRepresentsNothing(data)
    };
    if hide {
        builder.SetDisplay(EDisplay::kNone);
    } else if builder.IsInInlinifyingDisplay() {
        builder.SetDisplay(EDisplay::kInline);
        builder.SetFloating(EFloat::kNone);
    } else {
        let direction = builder.Direction();
        builder.SetDisplay(EDisplay::kFlowRoot);
        builder.SetFloating(if direction == TextDirection::kLtr {
            EFloat::kLeft
        } else {
            EFloat::kRight
        });
    }
}
