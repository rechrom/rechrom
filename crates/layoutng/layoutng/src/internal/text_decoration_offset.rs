//! Layout-owned text-decoration offset computation.

#![allow(non_snake_case)]

use font_engine::{fonts::font_vertical_position_type::IsLineOverSide, FontVerticalPositionType};
use foundation::{FloatValueForLength, LayoutUnit, Length, ResolvedUnderlinePosition};
use layoutng_style::style::computed_style::ComputedStyle;

use super::used_font::UsedFont;

// Source: third_party/blink/renderer/core/layout/text_decoration_offset.cc:16-35.
fn ComputeUnderlineOffsetAuto(
    font: &UsedFont,
    text_underline_offset: f32,
    text_decoration_thickness: f32,
    is_auto: bool,
) -> i32 {
    let gap = if is_auto {
        1.max((text_decoration_thickness / 2.0).ceil() as i32)
    } else {
        0
    };
    let primary_font = font.PrimaryFont();
    debug_assert!(!primary_font.is_null());
    (unsafe { &*primary_font }.GetFontMetrics().Ascent() as f32 * font.ScalingFactor()
        + gap as f32
        + text_underline_offset.round()) as i32
}

// Source: third_party/blink/renderer/core/layout/text_decoration_offset.cc:37-48.
fn ComputeUnderlineOffsetFromFont(font: &UsedFont, text_underline_offset: f32) -> Option<i32> {
    let primary_font = font.PrimaryFont();
    debug_assert!(!primary_font.is_null());
    let metrics = unsafe { &*primary_font }.GetFontMetrics();
    let underline_position = metrics.UnderlinePosition()?;
    Some(
        ((metrics.FloatAscent() + underline_position) * font.ScalingFactor()
            + text_underline_offset)
            .round() as i32,
    )
}

// Source: third_party/blink/renderer/core/layout/text_decoration_offset.h:20-48.
pub struct TextDecorationOffset<'a> {
    text_style_: &'a ComputedStyle,
}

impl<'a> TextDecorationOffset<'a> {
    pub fn new(text_style: &'a ComputedStyle) -> Self {
        Self {
            text_style_: text_style,
        }
    }

    // Source: third_party/blink/renderer/core/layout/text_decoration_offset.cc:52-89.
    pub fn ComputeUnderlineOffsetForUnder(
        &self,
        style_underline_offset: &Length,
        computed_font_size: f32,
        font: &UsedFont,
        text_decoration_thickness: f32,
        position_type: FontVerticalPositionType,
    ) -> i32 {
        let baseline_type = self.text_style_.GetFontBaseline();
        let mut style_underline_offset_pixels =
            LayoutUnit::FromFloatRound(Self::StyleUnderlineOffsetToPixels(
                style_underline_offset,
                computed_font_size,
                font.ScalingFactor(),
            ));
        if IsLineOverSide(position_type) {
            style_underline_offset_pixels = -style_underline_offset_pixels;
        }

        let font_data = font.PrimaryFont();
        if font_data.is_null() {
            return 0;
        }
        let font_data = unsafe { &*font_data };
        let offset = LayoutUnit::FromFloatRound(
            (font_data.GetFontMetrics().FloatAscentFor(baseline_type)
                - font_data
                    .VerticalPosition(position_type, baseline_type)
                    .ToFloat())
                * font.ScalingFactor(),
        ) + style_underline_offset_pixels;
        let offset_int = offset.Floor();
        if position_type == FontVerticalPositionType::TextTop {
            return offset_int - text_decoration_thickness.floor() as i32;
        }
        if !IsLineOverSide(position_type) {
            offset_int + 1
        } else {
            offset_int - 1 - text_decoration_thickness.floor() as i32
        }
    }

    // Source: third_party/blink/renderer/core/layout/text_decoration_offset.cc:91-120.
    pub fn ComputeUnderlineOffset(
        &self,
        underline_position: ResolvedUnderlinePosition,
        computed_font_size: f32,
        font: &UsedFont,
        style_underline_offset: &Length,
        text_decoration_thickness: f32,
    ) -> i32 {
        let style_underline_offset_pixels = Self::StyleUnderlineOffsetToPixels(
            style_underline_offset,
            computed_font_size,
            font.ScalingFactor(),
        );
        match underline_position {
            ResolvedUnderlinePosition::kNearAlphabeticBaselineFromFont => {
                ComputeUnderlineOffsetFromFont(font, style_underline_offset_pixels).unwrap_or_else(
                    || {
                        ComputeUnderlineOffsetAuto(
                            font,
                            style_underline_offset_pixels,
                            text_decoration_thickness,
                            style_underline_offset.IsAuto(),
                        )
                    },
                )
            }
            ResolvedUnderlinePosition::kNearAlphabeticBaselineAuto => ComputeUnderlineOffsetAuto(
                font,
                style_underline_offset_pixels,
                text_decoration_thickness,
                style_underline_offset.IsAuto(),
            ),
            ResolvedUnderlinePosition::kUnder => self.ComputeUnderlineOffsetForUnder(
                style_underline_offset,
                computed_font_size,
                font,
                text_decoration_thickness,
                FontVerticalPositionType::BottomOfEmHeight,
            ),
            ResolvedUnderlinePosition::kOver => {
                unreachable!("kOver is resolved by flipping underline and overline")
            }
        }
    }

    // Source: third_party/blink/renderer/core/layout/text_decoration_offset.cc:122-135.
    fn StyleUnderlineOffsetToPixels(
        style_underline_offset: &Length,
        computed_font_size: f32,
        scaling_factor: f32,
    ) -> f32 {
        if style_underline_offset.IsAuto() {
            return 0.0;
        }
        if style_underline_offset.IsPercent() {
            return FloatValueForLength(
                style_underline_offset,
                computed_font_size * scaling_factor,
            );
        }
        FloatValueForLength(style_underline_offset, computed_font_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_engine::Font;
    use foundation::{LayoutHeapScope, MakeGarbageCollected};

    #[test]
    fn underline_offset_length_uses_scaled_basis_only_for_percentages() {
        assert_eq!(
            TextDecorationOffset::StyleUnderlineOffsetToPixels(&Length::Percent(25.0), 20.0, 2.0,),
            10.0
        );
        assert_eq!(
            TextDecorationOffset::StyleUnderlineOffsetToPixels(&Length::Fixed(5.0), 20.0, 2.0,),
            5.0
        );
        assert_eq!(
            TextDecorationOffset::StyleUnderlineOffsetToPixels(&Length::Auto(), 20.0, 2.0),
            0.0
        );
    }

    #[test]
    fn under_offset_returns_zero_without_primary_font_data() {
        let _heap = LayoutHeapScope::new();
        let style = ComputedStyle::default();
        let font = unsafe { &*MakeGarbageCollected(Font::default()) };
        let used_font = UsedFont::new(font, 1.0);
        let offset = TextDecorationOffset::new(&style).ComputeUnderlineOffsetForUnder(
            &Length::Fixed(3.0),
            16.0,
            &used_font,
            2.0,
            FontVerticalPositionType::BottomOfEmHeight,
        );
        assert_eq!(offset, 0);
    }
}
