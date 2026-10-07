// C++: layoutng_inline/text_combine_style_service.cc.
#![allow(non_snake_case)]

use font_engine::fonts::font_orientation::IsVerticalBaseline;
use font_engine::FontOrientation;
use foundation::{
    EDisplay, ETextAlign, EWordBreak, LayoutUnit, Length, LengthSize, Member, TextDecorationLine,
    TextEmphasisMark, WritingMode,
};
use layoutng::internal::layout_text_combine::LayoutTextCombine;
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layoutng_style::style::computed_style_constants::EVerticalAlign;
use layoutng_style::style::style_intrinsic_length::{
    StyleIntrinsicLength, StyleIntrinsicLengthOptions,
};

// cpp: layoutng_inline/text_combine_style_service.cc:9-29
#[unsafe(no_mangle)]
pub extern "Rust" fn AdjustStyleForCombinedText(builder: &mut ComputedStyleBuilder) {
    builder.ResetTextCombine();
    builder.SetLetterSpacing(&Length::Fixed(0.0));
    builder.SetTextAlign(ETextAlign::kCenter);
    builder.SetTextDecorationLine(TextDecorationLine::kNone);
    builder.SetTextEmphasisMark(TextEmphasisMark::kNone);
    builder.SetVerticalAlign(EVerticalAlign::kMiddle);
    builder.SetWordBreak(EWordBreak::kKeepAll);
    builder.SetWordSpacing(&Length::Fixed(0.0));
    builder.SetWritingMode(WritingMode::kHorizontalTb);
    builder.SetBaseTextDecorationData(Member::default());
    builder.ResetTextIndent();
    builder.UpdateFontOrientation();

    // cpp: layoutng_inline/text_combine_style_service.cc:24-28
    #[cfg(debug_assertions)]
    {
        let font = unsafe { &*builder.GetFont() };
        debug_assert_eq!(
            font.GetFontDescription().Orientation(),
            FontOrientation::kHorizontal
        );
        let style = builder.CloneStyle();
        debug_assert!(!style.is_null());
        LayoutTextCombine::AssertStyleIsValid(unsafe { &*style });
    }
}

// cpp: layoutng_inline/text_combine_style_service.cc:31-46
// The source's release build has an empty body; its assertions are preserved
// in Rust debug builds and exported for LayoutTextCombine's owner forwarding.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutTextCombineAssertStyleIsValid(style: &ComputedStyle) {
    #[cfg(debug_assertions)]
    {
        debug_assert_eq!(style.GetTextDecorationLine(), TextDecorationLine::kNone);
        debug_assert_eq!(style.GetTextEmphasisMark(), TextEmphasisMark::kNone);
        debug_assert_eq!(style.GetWritingMode(), WritingMode::kHorizontalTb);
        debug_assert_eq!(style.LetterSpacing(), 0.0);
        debug_assert!(!style.HasAppliedTextDecorations());
        debug_assert_eq!(style.TextIndent(), &Length::Fixed(0.0));
        let font = unsafe { &*style.GetFont() };
        debug_assert_eq!(
            font.GetFontDescription().Orientation(),
            FontOrientation::kHorizontal
        );
    }
}

// cpp: layoutng_inline/text_combine_style_service.cc:48-65
#[unsafe(no_mangle)]
pub extern "Rust" fn AdjustStyleForTextCombine(builder: &mut ComputedStyleBuilder) {
    debug_assert_eq!(builder.Display(), EDisplay::kInlineBlock);
    let font = unsafe { &*builder.GetFont() };
    debug_assert!(IsVerticalBaseline(font.GetFontDescription().Orientation()));
    let one_em = ComputedStyle::ComputedFontSizeAsFixed(font);
    let line_height = builder.FontHeight();
    let size = LengthSize::new(
        &length_from_layout_unit(line_height),
        &length_from_layout_unit(one_em),
    );
    let intrinsic_width = StyleIntrinsicLength::new(
        &Some(size.Width().clone()),
        StyleIntrinsicLengthOptions::default(),
    );
    let intrinsic_height = StyleIntrinsicLength::new(
        &Some(size.Height().clone()),
        StyleIntrinsicLengthOptions::default(),
    );
    builder.SetContainIntrinsicWidth(&intrinsic_width);
    builder.SetContainIntrinsicHeight(&intrinsic_height);
    builder.SetHeight(size.Height());
    builder.SetLineHeight(size.Height());
    builder.SetMaxHeight(size.Height());
    builder.SetMaxWidth(size.Width());
    builder.SetMinHeight(size.Height());
    builder.SetMinWidth(size.Width());
    builder.SetWidth(size.Width());
    AdjustStyleForCombinedText(builder);
}

// C++ Length::Fixed(LayoutUnit) converts the fixed-point layout value into a
// CSS fixed length. The Rust Length constructor accepts the floating value.
fn length_from_layout_unit(value: LayoutUnit) -> Length {
    Length::Fixed(value.ToFloat())
}
