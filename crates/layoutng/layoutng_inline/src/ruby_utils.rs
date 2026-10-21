// C++: layoutng_inline/ruby_utils.h/.cc. Ruby indexing, overhang, annotation
// metrics, alignment, and block placement are mapped here. Font-owned Han
// kerning interfaces remain cross-package dependencies.
#![allow(non_snake_case)]

use font_engine::fonts::font_baseline::FontBaseline;
use font_engine::fonts::font_height::FontHeight;
use font_engine::fonts::shaping::han_kerning::HanKerning;
use font_engine::fonts::shaping::shape_result::ShapeResult;
use font_engine::fonts::shaping::shape_result_types::AdjustMidCluster;
use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use font_engine::fonts::shaping::text_spacing_trim::TextSpacingTrim;
use font_engine::text::native::character::Character;
use font_engine::text::native::unicode_category::Category;
use foundation::{
    kNotFound, unicode, ERubyAlign, ERubyOverhang, ETextAlign, HeapVector, IsLtr, LayoutUnit,
    MakeGarbageCollected, RubyPosition, RuntimeEnabledFeatures, String, StringView,
    TextEmphasisMark, To, ToLineWritingMode, Traceable, Visitor, WritingDirectionMode, WtfSizeT,
};
use layoutng::internal::inline_item::{InlineItem, InlineItemType, InlineItems};
use layoutng::internal::inline_item_result::{InlineItemResult, InlineItemResults};
use layoutng::internal::used_font::UsedFont;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::inline_box_state::{InlineBoxState, LogicalRubyColumn};
use crate::inline_item_result_ruby_column::InlineItemResultRubyColumn;
use crate::justification_utils::{ApplyJustification, ComputeRubyBaseInset, JustificationTarget};
use crate::line_info::LineInfo;
use foundation::Member;
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::logical_line_container::LogicalLineContainer;
use layoutng_fragment_tree::logical_line_item::{LogicalLineItem, LogicalLineItems};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style_constants::LineLogicalSide;

const K_HAN_KERNING_HALF: f32 = 0.5;
const K_HAN_KERNING_QUARTER: f32 = 0.25;

// cpp: layoutng_inline/ruby_utils.cc:31-33
fn IsSpaceForRubyOverhang(ch: i32) -> bool {
    Category(ch) == unicode::kSeparator_Space
}

// cpp: layoutng_inline/ruby_utils.cc:35-76
fn AdjustTextOverUnderOffsetsForEmHeight(
    over: LayoutUnit,
    under: LayoutUnit,
    font_baseline: FontBaseline,
    used_font: &UsedFont,
    shape_view: &ShapeResultView,
) -> (LayoutUnit, LayoutUnit) {
    debug_assert!(over <= under);
    if used_font.PrimaryFont().is_null() {
        return (over, under);
    }
    let line_height = under - over;
    let paint_scale = used_font.ScalingFactor();
    let primary_ascent = used_font.FixedAscentForBaseline(font_baseline);
    let primary_descent = line_height - primary_ascent;

    let run_fonts = shape_view.UsedFonts();
    // Rust releases the local GC set on scope exit; no ClearCollectionScope
    // guard is needed for this owning container.
    let no_diff = LayoutUnit::Max();
    let mut over_diff = no_diff;
    let mut under_diff = no_diff;
    for run_font in run_fonts.iter() {
        let mut normalized_height =
            unsafe { &*run_font.Get() }.NormalizedTypoAscentAndDescent(font_baseline);
        normalized_height.ascent *= paint_scale;
        normalized_height.descent *= paint_scale;
        // Preserve the source's floor-before-subtract behavior.
        let current_over_diff = LayoutUnit::from_signed(
            (primary_ascent - normalized_height.ascent)
                .ClampNegativeToZero()
                .Floor(),
        );
        let current_under_diff = LayoutUnit::from_signed(
            (primary_descent - normalized_height.descent)
                .ClampNegativeToZero()
                .Floor(),
        );
        over_diff = over_diff.min(current_over_diff);
        under_diff = under_diff.min(current_under_diff);
    }
    if over_diff == no_diff {
        over_diff = LayoutUnit::default();
    }
    if under_diff == no_diff {
        under_diff = LayoutUnit::default();
    }
    (over + over_diff, under - under_diff)
}

// cpp: layoutng_inline/ruby_utils.cc:78-143
fn ComputeEmHeight(line_item: &LogicalLineItem) -> FontHeight {
    let shape_result_view = line_item.shape_result.Get();
    if !shape_result_view.is_null() {
        let style = unsafe { &*line_item.Style() };
        let primary_font_data = unsafe { &*style.GetFont() }.PrimaryFont();
        if primary_font_data.is_null() {
            return FontHeight::default();
        }
        let font_baseline = style.GetFontBaseline();
        let primary_height = unsafe { &*primary_font_data }
            .GetFontMetrics()
            .GetFloatFontHeight(font_baseline);
        let mut result_height = FontHeight::default();
        let run_fonts = unsafe { &*shape_result_view }.UsedFonts();
        for run_font in run_fonts.iter() {
            result_height
                .Unite(&unsafe { &*run_font.Get() }.NormalizedTypoAscentAndDescent(font_baseline));
        }
        result_height.ascent =
            LayoutUnit::from_signed(result_height.ascent.Ceil()).min(primary_height.ascent);
        result_height.descent =
            LayoutUnit::from_signed(result_height.descent.Ceil()).min(primary_height.descent);
        result_height.Move(line_item.rect.offset.block_offset + primary_height.ascent);
        return result_height;
    }
    let layout_result = line_item.layout_result.Get();
    if !layout_result.is_null() {
        let fragment = unsafe { &*layout_result }.GetPhysicalFragment();
        let style = fragment.Style();
        let inline_size = LogicalFragment::new(style.GetWritingDirection(), fragment)
            .Size()
            .inline_size;
        if inline_size != LayoutUnit::default() && fragment.IsAtomicInline() {
            let direction = WritingDirectionMode::new(
                ToLineWritingMode(style.GetWritingMode()),
                style.Direction(),
            );
            let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(fragment as *const _) };
            let overflow = WritingModeConverter::new(direction, fragment.Size())
                .ToLogicalRect(box_fragment.ScrollableOverflow());
            return FontHeight::new(
                -overflow.offset.block_offset - line_item.BlockOffset(),
                overflow.BlockEndOffset() + line_item.BlockOffset(),
            );
        }
    }
    FontHeight::default()
}

// The foundation String exposes UTF-16 units but not the C++ cursor methods.
// Advancing the local index by a surrogate pair retains the C++ code-unit
// offsets used by ShapeResult and InlineItem.
fn CodePointAtAndNext(text: &StringView, index: &mut u32) -> i32 {
    let ch = text.CodePointAt(*index);
    *index += if ch > 0xffff { 2 } else { 1 };
    ch
}

fn CodePointAtAndPrevious(text: &StringView, start: u32, index: &mut u32) -> i32 {
    debug_assert!(*index > start);
    *index -= 1;
    let units = text.Span16();
    if *index > start
        && (0xdc00..=0xdfff).contains(&units[*index as usize])
        && (0xd800..=0xdbff).contains(&units[(*index - 1) as usize])
    {
        *index -= 1;
    }
    text.CodePointAt(*index)
}

fn CodePointAtOrZero(text: &StringView, index: u32) -> i32 {
    if index < text.length() {
        text.CodePointAt(index)
    } else {
        0
    }
}

// cpp: layoutng_inline/ruby_utils.cc:146-165
fn IsFullWidthGlyph(
    text_shape_result: &ShapeResult,
    font: &font_engine::fonts::simple_font_data::SimpleFontData,
    text_content: &String,
    text_offset: u32,
) -> bool {
    let text_view = StringView::from(text_content);
    let character = CodePointAtOrZero(&text_view, text_offset);
    let code_unit_length = if character > 0xffff { 2 } else { 1 };
    let end_position = text_shape_result.PositionForOffset(
        text_offset + code_unit_length - text_shape_result.StartIndex(),
        AdjustMidCluster::kToEnd,
    );
    let start_position = text_shape_result.PositionForOffset(
        text_offset - text_shape_result.StartIndex(),
        AdjustMidCluster::kToEnd,
    );
    let glyph_width = (end_position - start_position).abs();
    let advance_min = font.PlatformData().size() * 0.9;
    glyph_width > advance_min
}

// cpp: layoutng_inline/ruby_utils.cc:167-186
fn CanTrimHanKerningOpen(
    shape_result: &ShapeResult,
    style: &ComputedStyle,
    text_content: &String,
    text_offset: u32,
) -> bool {
    let text_view = StringView::from(text_content);
    let character = CodePointAtOrZero(&text_view, text_offset);
    if !Character::MaybeHanKerningOpen(character) {
        return false;
    }
    let primary_font = unsafe { &*style.GetFont() }.PrimaryFont();
    if primary_font.is_null()
        || !IsFullWidthGlyph(
            shape_result,
            unsafe { &*primary_font },
            text_content,
            text_offset,
        )
    {
        return false;
    }
    let font_description = style.GetFontDescription();
    // The font_engine owner has not yet connected LocaleOrDefault and
    // SimpleFontData::HanKerningData; preserve both source calls here.
    let font_data = unsafe { &*primary_font }.HanKerningData(
        font_description.LocaleOrDefault(),
        style.IsHorizontalTypographicMode(),
    );
    if text_offset == 0 || font_description.GetTextSpacingTrim() == TextSpacingTrim::kSpaceAll {
        return true;
    }
    let kind = HanKerning::GetCharType(character as u16, &font_data);
    let mut previous_index = text_offset;
    let previous_character = CodePointAtAndPrevious(&text_view, 0, &mut previous_index);
    let previous_type = HanKerning::GetCharType(previous_character as u16, &font_data);
    !HanKerning::ShouldKern(kind, previous_type)
}

// cpp: layoutng_inline/ruby_utils.cc:188-204
fn CanTrimHanKerningClose(
    shape_result: &ShapeResult,
    style: &ComputedStyle,
    text_content: &String,
    text_offset: u32,
) -> bool {
    let text_view = StringView::from(text_content);
    let mut next_index = text_offset;
    let character = CodePointAtAndNext(&text_view, &mut next_index);
    if !Character::MaybeHanKerningClose(character) {
        return false;
    }
    let primary_font = unsafe { &*style.GetFont() }.PrimaryFont();
    if primary_font.is_null()
        || !IsFullWidthGlyph(
            shape_result,
            unsafe { &*primary_font },
            text_content,
            text_offset,
        )
    {
        return false;
    }
    let font_description = style.GetFontDescription();
    let font_data = unsafe { &*primary_font }.HanKerningData(
        font_description.LocaleOrDefault(),
        style.IsHorizontalTypographicMode(),
    );
    if next_index >= text_view.length()
        || font_description.GetTextSpacingTrim() == TextSpacingTrim::kSpaceAll
    {
        return true;
    }
    let kind = HanKerning::GetCharType(character as u16, &font_data);
    let next_character = CodePointAtOrZero(&text_view, next_index);
    let next_type = HanKerning::GetCharType(next_character as u16, &font_data);
    !HanKerning::ShouldKernLast(next_type, kind)
}

// cpp: layoutng_inline/ruby_utils.h:27-36
#[derive(Clone, Copy, Debug)]
pub struct RubyItemIndexes {
    pub column_start: WtfSizeT,
    pub base_end: WtfSizeT,
    pub annotation_start: WtfSizeT,
    pub column_end: WtfSizeT,
}

// cpp: layoutng_inline/ruby_utils.h:44-47
#[derive(Clone, Copy, Debug, Default)]
pub struct AnnotationOverhang {
    pub start: LayoutUnit,
    pub end: LayoutUnit,
}

// cpp: layoutng_inline/ruby_utils.h:100-111
#[derive(Clone, Copy, Debug, Default)]
pub struct AnnotationMetrics {
    pub overflow_over: LayoutUnit,
    pub overflow_under: LayoutUnit,
    pub space_over: LayoutUnit,
    pub space_under: LayoutUnit,
}

// cpp: layoutng_inline/ruby_utils.h:113-119
// cpp: layoutng_inline/ruby_utils.cc:723-866
pub fn ComputeAnnotationOverflow(
    logical_line: &LogicalLineItems,
    line_box_metrics: &FontHeight,
    line_font_size: LayoutUnit,
    annotation_metrics: Option<FontHeight>,
) -> AnnotationMetrics {
    let line_over = LayoutUnit::default();
    let mut content_over = line_over + line_box_metrics.ascent;
    let mut content_under = content_over;
    let mut has_over_annotation = false;
    let mut has_under_annotation = false;
    let mut has_over_emphasis = false;
    let mut has_under_emphasis = false;

    let line_under = line_over + line_box_metrics.LineHeight();
    let mut over_emphasis = LayoutUnit::default();
    let mut under_emphasis = LayoutUnit::default();
    for item in logical_line.iter() {
        if !item.HasInFlowFragment() {
            continue;
        }
        if item.IsControl() || item.IsRubyLinePlaceholder() {
            continue;
        }
        let used_font = item.GetUsedFont();
        let text_box_over = line_box_metrics.ascent + item.BlockOffset();
        let text_box_under = line_box_metrics.ascent + item.BlockEndOffset();
        let mut item_over = text_box_over;
        let mut item_under = text_box_under;
        let shape_result = item.shape_result.Get();
        if !shape_result.is_null() {
            let style = item.Style();
            if !style.is_null() {
                (item_over, item_under) = AdjustTextOverUnderOffsetsForEmHeight(
                    item_over,
                    item_under,
                    unsafe { &*style }.GetFontBaseline(),
                    &used_font,
                    unsafe { &*shape_result },
                );
            }
        } else if item.IsAtomicInline() && !item.IsInitialLetterBox() {
            item_under = ComputeEmHeight(item).LineHeight();
        } else if item.IsInlineBox() {
            continue;
        }

        let style = item.Style();
        if !style.is_null() {
            let style = unsafe { &*style };
            if style.GetTextEmphasisMark() != TextEmphasisMark::kNone {
                if RuntimeEnabledFeatures::TextEmphasisAsRubyEnabled() {
                    let emphasis_mark_height =
                        InlineBoxState::ComputeEmphasisMarkOutsets(style, &used_font).LineHeight();
                    if style.GetTextEmphasisLineLogicalSide() == LineLogicalSide::kOver {
                        item_over =
                            text_box_over - (emphasis_mark_height + item.annotation_metrics.ascent);
                        has_over_emphasis = true;
                    } else {
                        item_under =
                            text_box_under + emphasis_mark_height + item.annotation_metrics.descent;
                        has_under_emphasis = true;
                    }
                } else if RuntimeEnabledFeatures::TextEmphasisWithRubyEnabled() {
                    let emphasis_mark_height =
                        InlineBoxState::ComputeEmphasisMarkOutsets(style, &used_font).LineHeight();
                    if style.GetTextEmphasisLineLogicalSide() == LineLogicalSide::kOver {
                        over_emphasis = over_emphasis.max(emphasis_mark_height);
                    } else {
                        under_emphasis = under_emphasis.max(emphasis_mark_height);
                    }
                } else if style.GetTextEmphasisLineLogicalSide() == LineLogicalSide::kOver {
                    over_emphasis = LayoutUnit::from_signed(1);
                } else {
                    under_emphasis = LayoutUnit::from_signed(1);
                }
            }
        }
        content_over = content_over.min(item_over);
        content_under = content_under.max(item_under);
    }

    if let Some(annotation_metrics) = annotation_metrics {
        if annotation_metrics.ascent != LayoutUnit::default() {
            let mut item_over = line_box_metrics.ascent - annotation_metrics.ascent;
            if RuntimeEnabledFeatures::TextEmphasisWithRubyEnabled() {
                item_over -= over_emphasis;
            }
            content_over = content_over.min(item_over);
            has_over_annotation = true;
        }
        if annotation_metrics.descent != LayoutUnit::default() {
            let mut item_under = line_box_metrics.ascent + annotation_metrics.descent;
            if RuntimeEnabledFeatures::TextEmphasisWithRubyEnabled() {
                item_under += under_emphasis;
            }
            content_under = content_under.max(item_under);
            has_under_annotation = true;
        }
    }

    if content_under - content_over < line_font_size {
        let mut half_leading = (line_box_metrics.LineHeight() - line_font_size) / 2i32;
        half_leading = half_leading.ClampNegativeToZero();
        content_over = line_over + half_leading;
        content_under = line_under - half_leading;
    }
    if !RuntimeEnabledFeatures::TextEmphasisAsRubyEnabled() {
        if over_emphasis > LayoutUnit::default() {
            content_over = content_over.min(line_over);
        }
        if under_emphasis > LayoutUnit::default() {
            content_under = content_under.max(line_under);
        }
    }
    if content_over < line_over && !has_over_annotation && !has_over_emphasis {
        content_over = line_over;
    }
    if content_under > line_under && !has_under_annotation && !has_under_emphasis {
        content_under = line_under;
    }

    AnnotationMetrics {
        overflow_over: (line_over - content_over).ClampNegativeToZero(),
        overflow_under: (content_under - line_under).ClampNegativeToZero(),
        space_over: (content_over - line_over).ClampNegativeToZero(),
        space_under: (line_under - content_under).ClampNegativeToZero(),
    }
}

// cpp: layoutng_inline/ruby_utils.cc:208-227
fn FindPreviousRubyIndex(items: &InlineItemResults, index: WtfSizeT) -> WtfSizeT {
    debug_assert!(index > 0);
    let mut previous_ruby_index = index - 1;
    while !items[previous_ruby_index as usize].IsRubyColumn() {
        let item_result = &items[previous_ruby_index as usize];
        let item_type = unsafe { &*item_result.item.Get() }.Type();
        if item_type != InlineItemType::kOpenTag
            && item_type != InlineItemType::kCloseTag
            && item_type != InlineItemType::kCloseRubyColumn
            && item_type != InlineItemType::kOpenRubyColumn
            && item_type != InlineItemType::kRubyLinePlaceholder
            && item_type != InlineItemType::kText
            && item_type != InlineItemType::kControl
        {
            return kNotFound;
        }
        if previous_ruby_index == 0 {
            return kNotFound;
        }
        previous_ruby_index -= 1;
    }
    previous_ruby_index
}

// cpp: layoutng_inline/ruby_utils.h:38-42
// cpp: layoutng_inline/ruby_utils.cc:231-260
pub fn ParseRubyInInlineItems(items: &InlineItems, start_item_index: WtfSizeT) -> RubyItemIndexes {
    assert!(start_item_index < items.size());
    assert_eq!(
        unsafe { &*items[start_item_index as usize].Get() }.Type(),
        InlineItemType::kOpenRubyColumn
    );
    let mut indexes = RubyItemIndexes {
        column_start: start_item_index,
        base_end: kNotFound,
        annotation_start: kNotFound,
        column_end: kNotFound,
    };
    let mut index = start_item_index + 1;
    while index < items.size() {
        let item = unsafe { &*items[index as usize].Get() };
        if item.Type() == InlineItemType::kCloseRubyColumn {
            if indexes.base_end == kNotFound {
                debug_assert_eq!(indexes.annotation_start, kNotFound);
                indexes.base_end = index;
            } else {
                debug_assert_ne!(indexes.annotation_start, kNotFound);
            }
            indexes.column_end = index;
            return indexes;
        }
        if item.Type() == InlineItemType::kOpenTag
            && unsafe { &*item.GetLayoutObject() }.IsInlineRubyText()
        {
            debug_assert_eq!(indexes.base_end, kNotFound);
            debug_assert_eq!(indexes.annotation_start, kNotFound);
            indexes.base_end = index;
            indexes.annotation_start = index;
        } else if item.Type() == InlineItemType::kOpenRubyColumn {
            let sub_indexes = ParseRubyInInlineItems(items, index);
            index = sub_indexes.column_end;
        }
        index += 1;
    }
    unreachable!("ruby column must have a closing item")
}

// cpp: layoutng_inline/ruby_utils.h:59-68
// cpp: layoutng_inline/ruby_utils.cc:262-438
pub fn GetOverhangForLines(
    ruby_size: LayoutUnit,
    base_line: &LineInfo,
    annotation_line_list: &HeapVector<LineInfo, 1>,
    line_info: &LineInfo,
    ruby_index: WtfSizeT,
) -> AnnotationOverhang {
    let mut overhang = AnnotationOverhang::default();
    let base_line_style = base_line.LineStyle();
    let ruby_align = base_line_style.RubyAlign();
    if ruby_align == ERubyAlign::kSpaceBetween {
        return overhang;
    }

    let mut half_width_of_annotation_font = LayoutUnit::default();
    for annotation_line in annotation_line_list {
        if annotation_line.Width() == ruby_size {
            half_width_of_annotation_font =
                LayoutUnit::from_f32(annotation_line.LineStyle().FontSize() as f32 / 2.0);
            break;
        }
    }
    if half_width_of_annotation_font == LayoutUnit::default() {
        return overhang;
    }
    let space = ruby_size - base_line.Width();
    if space <= LayoutUnit::default() {
        return overhang;
    }

    let ruby_base_inset = ComputeRubyBaseInset(space, base_line);
    if base_line_style.RubyOverhang() != ERubyOverhang::kSpaces {
        if ruby_align == ERubyAlign::kStart {
            overhang.end = space.min(half_width_of_annotation_font);
            return overhang;
        }
        let Some(inset) = ruby_base_inset else {
            return overhang;
        };
        overhang.start = inset.min(half_width_of_annotation_font);
        overhang.end = overhang.start;
        return overhang;
    }

    if ruby_index == 0 {
        let Some(inset) = ruby_base_inset else {
            return overhang;
        };
        overhang.end = if ruby_align == ERubyAlign::kStart {
            space.min(half_width_of_annotation_font)
        } else {
            inset
        };
        return overhang;
    }

    let items = line_info.Results();
    let mut previous_index = ruby_index - 1;
    while previous_index > 0
        && matches!(
            unsafe { &*items[previous_index as usize].item.Get() }.Type(),
            InlineItemType::kOpenTag | InlineItemType::kCloseTag
        )
    {
        previous_index -= 1;
    }

    let mut previous_ruby_overhang_end = LayoutUnit::default();
    let previous_ruby_index = FindPreviousRubyIndex(items, ruby_index);
    if previous_ruby_index != kNotFound {
        let column = items[previous_ruby_index as usize].ruby_column.Get()
            as *const InlineItemResultRubyColumn;
        previous_ruby_overhang_end = unsafe { &*column }.end_overhang;
    }

    let mut space_overhang = LayoutUnit::default();
    let mut previous_item_inline_size_sum = LayoutUnit::default();
    let mut space_start_offset = 0;
    let text_content = &line_info.ItemsData().text_content;
    let text_view = StringView::from(text_content);
    loop {
        let previous_item = &items[previous_index as usize];
        previous_item_inline_size_sum += previous_item.inline_size;
        let previous_item_text_offset = previous_item.TextOffset();
        let item = unsafe { &*previous_item.item.Get() };
        if item.Type() == InlineItemType::kControl {
            space_overhang += previous_item.inline_size;
            space_start_offset = previous_item_text_offset.start;
        } else if item.Type() == InlineItemType::kText {
            space_start_offset = previous_item_text_offset.end;
            while space_start_offset > previous_item_text_offset.start {
                let mut previous_space_start_offset = space_start_offset;
                let previous_character = CodePointAtAndPrevious(
                    &text_view,
                    previous_item_text_offset.start,
                    &mut previous_space_start_offset,
                );
                if !IsSpaceForRubyOverhang(previous_character) {
                    break;
                }
                space_start_offset = previous_space_start_offset;
            }
            if space_start_offset == previous_item_text_offset.end {
                break;
            }
            if space_start_offset == previous_item_text_offset.start {
                space_overhang += previous_item.inline_size;
            } else if !item.TextShapeResult().is_null() {
                let space_shape_result = ShapeResultView::CreateFromResultRange(
                    item.TextShapeResult(),
                    space_start_offset,
                    previous_item_text_offset.end,
                );
                space_overhang += LayoutUnit::from_f32(unsafe { &*space_shape_result }.Width());
                break;
            } else {
                break;
            }
        } else {
            break;
        }

        if previous_index == 0 {
            previous_index = kNotFound;
            break;
        }
        previous_index -= 1;
    }

    let mut kerning_overhang = LayoutUnit::default();
    if previous_index != kNotFound {
        let previous_item = &items[previous_index as usize];
        let item = unsafe { &*previous_item.item.Get() };
        if item.Type() == InlineItemType::kText
            && space_start_offset > previous_item.TextOffset().start
        {
            let previous_item_style = unsafe { &*item.Style() };
            let mut last_non_space_index = space_start_offset;
            let last_non_space_character =
                CodePointAtAndPrevious(&text_view, 0, &mut last_non_space_index);
            let font_size = LayoutUnit::from_f32(previous_item_style.FontSize() as f32);
            if CanTrimHanKerningClose(
                unsafe { &*item.TextShapeResult() },
                previous_item_style,
                text_content,
                last_non_space_index,
            ) {
                kerning_overhang = LayoutUnit::from_f32(font_size.ToFloat() * K_HAN_KERNING_HALF);
            } else if Character::MaybeHanKerningMiddle(last_non_space_character) {
                kerning_overhang =
                    LayoutUnit::from_f32(font_size.ToFloat() * K_HAN_KERNING_QUARTER);
            }
        }
    }

    let Some(inset) = ruby_base_inset else {
        return overhang;
    };
    if ruby_align == ERubyAlign::kStart {
        overhang.end = space.min(inset * 2);
        return overhang;
    }
    overhang.start = inset.min(space_overhang + kerning_overhang);
    overhang.start =
        (previous_item_inline_size_sum - previous_ruby_overhang_end).min(overhang.start);
    overhang.end = inset;
    overhang
}

// cpp: layoutng_inline/ruby_utils.h:51-56
// cpp: layoutng_inline/ruby_utils.cc:440-447
pub fn GetOverhangForColumn(
    item: &InlineItemResult,
    line_info: &LineInfo,
    ruby_index: WtfSizeT,
) -> AnnotationOverhang {
    debug_assert!(item.IsRubyColumn());
    let column = unsafe { &*(item.ruby_column.Get() as *const InlineItemResultRubyColumn) };
    GetOverhangForLines(
        item.inline_size,
        &column.base_line,
        &column.annotation_line_list,
        line_info,
        ruby_index,
    )
}

// cpp: layoutng_inline/ruby_utils.h:70-75
// cpp: layoutng_inline/ruby_utils.cc:449-505
pub fn CanApplyStartOverhang(
    line_info: &LineInfo,
    ruby_index: WtfSizeT,
    ruby_style: &ComputedStyle,
    start_overhang: &mut LayoutUnit,
) -> bool {
    if *start_overhang <= LayoutUnit::default() {
        return false;
    }
    let items = line_info.Results();
    if ruby_index < 1 {
        return false;
    }
    let mut previous_index = ruby_index - 1;
    while previous_index > 0
        && matches!(
            unsafe { &*items[previous_index as usize].item.Get() }.Type(),
            InlineItemType::kOpenTag | InlineItemType::kCloseTag
        )
    {
        previous_index -= 1;
    }
    let previous_item = &items[previous_index as usize];
    let item = unsafe { &*previous_item.item.Get() };
    if ruby_style.RubyOverhang() == ERubyOverhang::kSpaces
        && item.Type() == InlineItemType::kControl
    {
        return true;
    }
    if item.Type() != InlineItemType::kText {
        return false;
    }
    let previous_item_style = unsafe { &*item.Style() };
    if previous_item_style.FontSize() > ruby_style.FontSize() {
        return false;
    }
    if RuntimeEnabledFeatures::TextEmphasisWithRubyEnabled()
        && previous_item_style.GetTextEmphasisMark() != TextEmphasisMark::kNone
        && previous_item_style.GetTextEmphasisLineLogicalSide() == ruby_style.GetRubyPosition()
    {
        return false;
    }
    if ruby_style.RubyOverhang() == ERubyOverhang::kSpaces {
        let text_content = &line_info.ItemsData().text_content;
        let text_view = StringView::from(text_content);
        let mut previous_character_index = previous_item.TextOffset().end;
        let previous_character = CodePointAtAndPrevious(
            &text_view,
            previous_item.TextOffset().start,
            &mut previous_character_index,
        );
        if !IsSpaceForRubyOverhang(previous_character)
            && !CanTrimHanKerningClose(
                unsafe { &*item.TextShapeResult() },
                previous_item_style,
                text_content,
                previous_character_index,
            )
            && !Character::MaybeHanKerningMiddle(previous_character)
        {
            return false;
        }
        return true;
    }
    *start_overhang = (*start_overhang).min(previous_item.inline_size / 2);
    true
}

// cpp: layoutng_inline/ruby_utils.h:77-84
// cpp: layoutng_inline/ruby_utils.cc:507-630
pub fn CommitPendingEndOverhang(
    text_item: &InlineItem,
    shape_result: &ShapeResult,
    line_info: &mut LineInfo,
) -> LayoutUnit {
    let text_content = line_info.ItemsData().text_content.clone();
    let items = line_info.MutableResults();
    if items.size() < 1 {
        return LayoutUnit::default();
    }
    debug_assert!(matches!(
        text_item.Type(),
        InlineItemType::kText | InlineItemType::kControl
    ));
    let mut index = items.size() - 1;
    let mut has_previous_text_or_control = false;
    while !items[index as usize].IsRubyColumn() {
        let item_type = unsafe { &*items[index as usize].item.Get() }.Type();
        if !matches!(
            item_type,
            InlineItemType::kOpenTag
                | InlineItemType::kCloseTag
                | InlineItemType::kCloseRubyColumn
                | InlineItemType::kOpenRubyColumn
                | InlineItemType::kRubyLinePlaceholder
                | InlineItemType::kText
                | InlineItemType::kControl
        ) {
            return LayoutUnit::default();
        }
        if matches!(item_type, InlineItemType::kText | InlineItemType::kControl) {
            has_previous_text_or_control = true;
        }
        if index == 0 {
            return LayoutUnit::default();
        }
        index -= 1;
    }
    let column_item = &mut items[index as usize];
    if column_item.pending_end_overhang <= LayoutUnit::default() {
        return LayoutUnit::default();
    }
    let column =
        unsafe { &mut *(column_item.ruby_column.Get() as *mut InlineItemResultRubyColumn) };
    let column_base_line_style = column.base_line.LineStyle() as *const ComputedStyle;
    let column_base_line_style = unsafe { &*column_base_line_style };
    if column_base_line_style.RubyOverhang() != ERubyOverhang::kSpaces
        && (has_previous_text_or_control || text_item.Type() == InlineItemType::kControl)
    {
        return LayoutUnit::default();
    }
    let text_style = unsafe { &*text_item.Style() };
    if column_base_line_style.FontSize() < text_style.FontSize() {
        return LayoutUnit::default();
    }
    if RuntimeEnabledFeatures::TextEmphasisWithRubyEnabled()
        && text_style.GetTextEmphasisMark() != TextEmphasisMark::kNone
        && text_style.GetTextEmphasisLineLogicalSide() == column_base_line_style.GetRubyPosition()
    {
        return LayoutUnit::default();
    }

    let end_item = column.base_line.MutableResults().last_mut().unwrap();
    if column_base_line_style.RubyOverhang() != ERubyOverhang::kSpaces {
        let text_inline_size = LayoutUnit::from_f32(shape_result.Width());
        let end_overhang = column_item.pending_end_overhang.min(text_inline_size / 2);
        end_item.margins.inline_end -= end_overhang;
        column_item.pending_end_overhang = LayoutUnit::default();
        return end_overhang;
    }

    let item_inline_size = LayoutUnit::from_f32(shape_result.Width());
    let end_overhang;
    let mut is_exhausted = false;
    if text_item.Type() == InlineItemType::kControl {
        end_overhang = item_inline_size.min(column_item.pending_end_overhang);
        column_item.pending_end_overhang -= end_overhang;
    } else {
        debug_assert_eq!(text_item.Type(), InlineItemType::kText);
        let text_view = StringView::from(&text_content);
        let mut space_end = text_item.StartOffset();
        while space_end < text_item.EndOffset() {
            let mut next_space_end = space_end;
            let character = CodePointAtAndNext(&text_view, &mut next_space_end);
            if !IsSpaceForRubyOverhang(character) {
                break;
            }
            space_end = next_space_end;
        }

        let space_overhang = if space_end == text_item.EndOffset() {
            item_inline_size
        } else {
            let space_view = ShapeResultView::CreateFromResultRange(
                shape_result,
                text_item.StartOffset(),
                space_end,
            );
            LayoutUnit::from_f32(unsafe { &*space_view }.Width())
        };
        let mut kerning_overhang = LayoutUnit::default();
        if space_end < text_item.EndOffset() {
            let font_size = text_style.FontSize() as f32;
            if CanTrimHanKerningOpen(shape_result, text_style, &text_content, space_end) {
                kerning_overhang = LayoutUnit::from_f32(font_size * K_HAN_KERNING_HALF);
            } else if Character::MaybeHanKerningMiddle(text_view.CodePointAt(space_end)) {
                kerning_overhang = LayoutUnit::from_f32(font_size * K_HAN_KERNING_QUARTER);
            }
        }
        end_overhang = column_item
            .pending_end_overhang
            .min(space_overhang + kerning_overhang);
        if space_end < text_item.EndOffset() {
            is_exhausted = true;
        } else {
            column_item.pending_end_overhang -= end_overhang;
        }
    }

    column.end_overhang += end_overhang;
    if is_exhausted {
        column_item.pending_end_overhang = LayoutUnit::default();
    }
    end_item.margins.inline_end -= end_overhang;
    end_overhang
}

// cpp: layoutng_inline/ruby_utils.h:89-93
// cpp: layoutng_inline/ruby_utils.cc:632-721
pub fn ApplyRubyAlign(
    available_line_size: LayoutUnit,
    mut on_start_edge: bool,
    mut on_end_edge: bool,
    line_info: &mut LineInfo,
) -> (LayoutUnit, LayoutUnit) {
    debug_assert!(line_info.IsRubyBase() || line_info.IsRubyText());
    let space = available_line_size - line_info.WidthForAlignment();
    if space <= LayoutUnit::default() {
        return (LayoutUnit::default(), LayoutUnit::default());
    }

    let ruby_align = line_info.LineStyle().RubyAlign();
    let mut text_align = line_info.TextAlign();
    match ruby_align {
        ERubyAlign::kSpaceAround => {}
        ERubyAlign::kSpaceBetween => {
            on_start_edge = true;
            on_end_edge = true;
            text_align = ETextAlign::kJustify;
        }
        ERubyAlign::kStart => {
            return if IsLtr(line_info.BaseDirection()) {
                (LayoutUnit::default(), space)
            } else {
                (space, LayoutUnit::default())
            };
        }
        ERubyAlign::kCenter => return (space / 2, space / 2),
    }

    if text_align == ETextAlign::kJustify {
        let target = if on_start_edge && on_end_edge {
            JustificationTarget::kNormal
        } else if line_info.IsRubyBase() {
            JustificationTarget::kRubyBase
        } else {
            debug_assert!(line_info.IsRubyText());
            JustificationTarget::kRubyText
        };
        let inset = ApplyJustification(space, target, line_info);
        if let Some(inset) = inset {
            if on_start_edge && !on_end_edge {
                return (LayoutUnit::default(), inset * 2);
            }
            if !on_start_edge && on_end_edge {
                return (inset * 2, LayoutUnit::default());
            }
            return (inset, inset);
        }
        if on_start_edge && !on_end_edge {
            return (LayoutUnit::default(), space);
        }
        if !on_start_edge && on_end_edge {
            return (space, LayoutUnit::default());
        }
        return (space / 2, space / 2);
    }

    let is_ltr = IsLtr(line_info.BaseDirection());
    if text_align == ETextAlign::kStart {
        text_align = if is_ltr {
            ETextAlign::kLeft
        } else {
            ETextAlign::kRight
        };
    } else if text_align == ETextAlign::kEnd {
        text_align = if is_ltr {
            ETextAlign::kRight
        } else {
            ETextAlign::kLeft
        };
    }
    match text_align {
        ETextAlign::kLeft | ETextAlign::kWebkitLeft => (LayoutUnit::default(), space),
        ETextAlign::kRight | ETextAlign::kWebkitRight => (space, LayoutUnit::default()),
        ETextAlign::kCenter | ETextAlign::kWebkitCenter => (space / 2, space / 2),
        ETextAlign::kStart | ETextAlign::kEnd | ETextAlign::kJustify | ETextAlign::kMatchParent => {
            unreachable!("ruby alignment must resolve text-align")
        }
    }
}

// cpp: layoutng_inline/ruby_utils.h:122-127
// cpp: layoutng_inline/ruby_utils.cc:870-897
pub fn UpdateRubyColumnInlinePositions(
    line_items: &LogicalLineItems,
    inline_size: LayoutUnit,
    column_list: &mut HeapVector<Member<LogicalRubyColumn>>,
) {
    for column_member in column_list {
        let column = unsafe { &mut *column_member.Get() };
        let start_index = column.start_index;
        let inline_offset = if start_index < line_items.size() {
            line_items[start_index as usize].rect.offset.inline_offset
        } else if start_index == line_items.size() {
            if line_items.size() > 0 {
                let last_item = &line_items[(start_index - 1) as usize];
                last_item.rect.offset.inline_offset + last_item.rect.InlineEndOffset()
            } else {
                inline_size
            }
        } else {
            unreachable!(
                "LogicalLineItems::size()={} LogicalRubyColumn::start_index={}",
                line_items.size(),
                start_index
            )
        };
        // The source TODO for overhang remains unresolved.
        let annotation_items = unsafe { &mut *column.annotation_items.Get() };
        annotation_items.MoveInInlineDirection(inline_offset);
        column
            .state_stack
            .MoveBoxDataInInlineDirection(inline_offset);
        UpdateRubyColumnInlinePositions(annotation_items, inline_size, column.RubyColumnList());
    }
}

// cpp: layoutng_inline/ruby_utils.h:129-132
// cpp: layoutng_inline/ruby_utils.cc:899-949
pub fn SetTextEmphasisAnnotationMetrics(
    column_list: &HeapVector<Member<LogicalRubyColumn>>,
    line_box: &mut LogicalLineItems,
) {
    for index in 0..line_box.size() {
        let item = &mut line_box[index as usize];
        if !item.IsItemType(InlineItemType::kText) {
            continue;
        }
        let style = item.Style();
        if style.is_null() || unsafe { &*style }.GetTextEmphasisMark() == TextEmphasisMark::kNone {
            continue;
        }

        let mut matched_column: *const LogicalRubyColumn = std::ptr::null();
        for column_member in column_list {
            let column = unsafe { &*column_member.Get() };
            if column.start_index <= index && index < column.start_index + column.size {
                matched_column = column;
                break;
            }
        }

        let font_baseline = unsafe { &*style }.GetFontBaseline();
        let used_font = item.GetUsedFont();
        let over_initial = -used_font.FixedAscentForBaseline(font_baseline);
        let under_initial = used_font.FixedDescentForBaseline(font_baseline);
        let mut over = over_initial;
        let mut under = under_initial;
        let shape_view = item.shape_result.Get();
        if !shape_view.is_null() {
            (over, under) = AdjustTextOverUnderOffsetsForEmHeight(
                over,
                under,
                font_baseline,
                &used_font,
                unsafe { &*shape_view },
            );
        }
        if !matched_column.is_null() {
            let column = unsafe { &*matched_column };
            if column.layout_annotation_metrics.ascent != LayoutUnit::default() {
                over = -column.layout_annotation_metrics.ascent;
            }
            if column.layout_annotation_metrics.descent != LayoutUnit::default() {
                under = column.layout_annotation_metrics.descent;
            }
        }
        item.annotation_metrics = FontHeight::new(over_initial - over, under - under_initial);
    }

    for column_member in column_list {
        let column = unsafe { &mut *column_member.Get() };
        let annotation_items = column.annotation_items.Get();
        if !annotation_items.is_null() {
            SetTextEmphasisAnnotationMetrics(column.RubyColumnList(), unsafe {
                &mut *annotation_items
            });
        }
    }
}

// cpp: layoutng_inline/ruby_utils.cc:956-962
fn ComputeLogicalLineEmHeight(line_items: &LogicalLineItems) -> FontHeight {
    let mut height = FontHeight::default();
    for item in line_items.iter() {
        height.Unite(&ComputeEmHeight(item));
    }
    height
}

// Rust names the indexed overload separately while preserving its source
// empty-list branch and selected-item traversal.
// cpp: layoutng_inline/ruby_utils.cc:964-975
fn ComputeLogicalLineEmHeightForIndexes(
    line_items: &LogicalLineItems,
    index_list: &[WtfSizeT],
) -> FontHeight {
    if index_list.is_empty() {
        return ComputeLogicalLineEmHeight(line_items);
    }
    let mut height = FontHeight::default();
    for &index in index_list {
        height.Unite(&ComputeEmHeight(&line_items[index as usize]));
    }
    height
}

// cpp: layoutng_inline/ruby_utils.cc:979-994
fn ComputeEmphasisHeights(line_items: &LogicalLineItems) -> FontHeight {
    let mut heights = FontHeight::default();
    for item in line_items.iter() {
        if !item.HasInFlowFragment() {
            continue;
        }
        let style = item.Style();
        if style.is_null() || unsafe { &*style }.GetTextEmphasisMark() == TextEmphasisMark::kNone {
            continue;
        }
        heights.Unite(&InlineBoxState::ComputeEmphasisMarkOutsets(
            unsafe { &*style },
            &item.GetUsedFont(),
        ));
    }
    heights
}

// cpp: layoutng_inline/ruby_utils.cc:998-1019
fn ComputeEmphasisHeightsForIndexes(
    line_items: &LogicalLineItems,
    index_list: &[WtfSizeT],
) -> FontHeight {
    let mut heights = FontHeight::default();
    for &index in index_list {
        if index >= line_items.size() {
            continue;
        }
        let item = &line_items[index as usize];
        if !item.HasInFlowFragment() {
            continue;
        }
        let style = item.Style();
        if style.is_null() || unsafe { &*style }.GetTextEmphasisMark() == TextEmphasisMark::kNone {
            continue;
        }
        heights.Unite(&InlineBoxState::ComputeEmphasisMarkOutsets(
            unsafe { &*style },
            &item.GetUsedFont(),
        ));
    }
    heights
}

// C++ nests RubyLine and AnnotationDepth under RubyBlockPositionCalculator.
// Rust keeps them as module types while preserving their source-owned fields.
pub type RubyLevel = Vec<i32>;

// cpp: layoutng_inline/ruby_utils.h:146-225
pub struct RubyLine {
    level_: RubyLevel,
    column_list_: HeapVector<Member<LogicalRubyColumn>>,
    base_index_list_: Vec<WtfSizeT>,
    metrics_: FontHeight,
    offset_: LayoutUnit,
    over_children_: HeapVector<Member<RubyLine>>,
    under_children_: HeapVector<Member<RubyLine>>,
    relative_offset_: LayoutUnit,
}

#[allow(non_snake_case)]
impl RubyLine {
    // cpp: layoutng_inline/ruby_utils.cc:1488-1489
    pub fn new(level: &RubyLevel) -> Self {
        Self {
            level_: level.clone(),
            column_list_: HeapVector::default(),
            base_index_list_: Vec::new(),
            metrics_: FontHeight::Empty(),
            offset_: LayoutUnit::default(),
            over_children_: HeapVector::default(),
            under_children_: HeapVector::default(),
            relative_offset_: LayoutUnit::default(),
        }
    }

    // cpp: layoutng_inline/ruby_utils.cc:1491-1495
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.column_list_);
        visitor.Trace(&self.over_children_);
        visitor.Trace(&self.under_children_);
    }

    // cpp: layoutng_inline/ruby_utils.h:150-158
    pub fn Level(&self) -> &RubyLevel {
        &self.level_
    }
    pub fn IsBaseLevel(&self) -> bool {
        self.level_.is_empty()
    }
    pub fn IsFirstOverLevel(&self) -> bool {
        self.level_.len() == 1 && self.level_[0] == 1
    }
    pub fn IsFirstUnderLevel(&self) -> bool {
        self.level_.len() == 1 && self.level_[0] == -1
    }
    pub fn BaseIndexList(&self) -> &[WtfSizeT] {
        &self.base_index_list_
    }

    // cpp: layoutng_inline/ruby_utils.cc:1497-1504
    pub fn SortChildren(&mut self) {
        let compare_abs_level =
            |line: &Member<RubyLine>| unsafe { &*line.Get() }.Level().last().unwrap().abs();
        self.over_children_.sort_unstable_by_key(compare_abs_level);
        self.under_children_.sort_unstable_by_key(compare_abs_level);
    }

    // cpp: layoutng_inline/ruby_utils.cc:1506-1517
    pub fn LessThan(&self, another: &RubyLine) -> bool {
        let level1 = self.Level();
        let level2 = another.Level();
        let mut index = 0;
        while index < level1.len() && index < level2.len() && level1[index] == level2[index] {
            index += 1;
        }
        let value1 = if index < level1.len() {
            level1[index]
        } else {
            0
        };
        let value2 = if index < level2.len() {
            level2[index]
        } else {
            0
        };
        value1 < value2
    }

    // cpp: layoutng_inline/ruby_utils.cc:1519-1522
    pub fn Append(&mut self, logical_column: &LogicalRubyColumn) {
        self.column_list_.push_back(Member::from_ptr(
            logical_column as *const _ as *mut LogicalRubyColumn,
        ));
    }

    // cpp: layoutng_inline/ruby_utils.cc:1524-1533
    pub fn MaybeRecordBaseIndexes(&mut self, logical_column: &LogicalRubyColumn) {
        if self.IsFirstOverLevel() || self.IsFirstUnderLevel() {
            self.base_index_list_.reserve(logical_column.size as usize);
            for item_index in logical_column.start_index..logical_column.EndIndex() {
                self.base_index_list_.push(item_index);
            }
        }
    }

    // cpp: layoutng_inline/ruby_utils.cc:1535-1554
    pub fn UpdateMetrics(&mut self) -> FontHeight {
        debug_assert!(self.metrics_.IsEmpty());
        self.metrics_ = FontHeight::default();
        for column_member in &self.column_list_ {
            let column = unsafe { &*column_member.Get() };
            let margins = column.state_stack.AnnotationBoxBlockAxisMargins();
            let annotation_items = unsafe { &*column.annotation_items.Get() };
            if let Some((start_margin, end_margin)) = margins {
                for item in annotation_items.iter() {
                    if item.IsPlaceholder() {
                        self.metrics_.Unite(&FontHeight::new(
                            -item.BlockOffset() + start_margin,
                            item.BlockEndOffset() + end_margin,
                        ));
                        break;
                    }
                }
            } else {
                self.metrics_
                    .Unite(&ComputeLogicalLineEmHeight(annotation_items));
            }
        }
        self.metrics_
    }

    // cpp: layoutng_inline/ruby_utils.cc:1556-1562
    pub fn MoveInBlockDirection(&mut self, offset: LayoutUnit) {
        for column_member in &mut self.column_list_ {
            let column = unsafe { &mut *column_member.Get() };
            unsafe { &mut *column.annotation_items.Get() }.MoveInBlockDirection(offset);
            column.state_stack.MoveBoxDataInBlockDirection(offset);
        }
    }

    // cpp: layoutng_inline/ruby_utils.cc:1564-1572
    pub fn AddLinesTo(&self, line_container: &mut LogicalLineContainer) {
        if self.IsBaseLevel() {
            return;
        }
        for column_member in &self.column_list_ {
            let column = unsafe { &*column_member.Get() };
            line_container.AddAnnotation(self.metrics_, unsafe { &*column.annotation_items.Get() });
        }
    }

    // cpp: layoutng_inline/ruby_utils.cc:1574-1587
    pub fn ComputeLevelEmphasisHeights(&self, base_line_items: &LogicalLineItems) -> FontHeight {
        let mut heights = FontHeight::default();
        if self.IsBaseLevel() {
            return ComputeEmphasisHeights(base_line_items);
        }
        for column_member in &self.column_list_ {
            let column = unsafe { &*column_member.Get() };
            if !column.annotation_items.Get().is_null() {
                heights.Unite(&ComputeEmphasisHeights(unsafe {
                    &*column.annotation_items.Get()
                }));
            }
        }
        heights
    }

    // cpp: layoutng_inline/ruby_utils.cc:1589-1612
    pub fn ComputeParentEmphasisHeightsForChild(
        &self,
        child: &RubyLine,
        base_line_items: &LogicalLineItems,
    ) -> FontHeight {
        let mut heights = FontHeight::default();
        if self.IsBaseLevel() {
            if !child.BaseIndexList().is_empty() {
                return ComputeEmphasisHeightsForIndexes(base_line_items, child.BaseIndexList());
            }
            for column_member in &child.column_list_ {
                let column = unsafe { &*column_member.Get() };
                for index in column.start_index..column.EndIndex() {
                    if index < base_line_items.size() {
                        heights.Unite(&ComputeEmphasisHeightsForIndexes(base_line_items, &[index]));
                    }
                }
            }
        } else {
            for column_member in &self.column_list_ {
                let column = unsafe { &*column_member.Get() };
                if !column.annotation_items.Get().is_null() {
                    heights.Unite(&ComputeEmphasisHeights(unsafe {
                        &*column.annotation_items.Get()
                    }));
                }
            }
        }
        heights
    }

    // cpp: layoutng_inline/ruby_utils.cc:1614-1618
    pub fn ContainsColumn(&self, column: *const LogicalRubyColumn) -> bool {
        self.column_list_
            .iter()
            .any(|candidate| candidate.Get() == column as *mut LogicalRubyColumn)
    }

    // cpp: layoutng_inline/ruby_utils.h:169-206
    pub fn SetOffset(&mut self, offset: LayoutUnit) {
        self.offset_ = offset;
    }
    pub fn Offset(&self) -> LayoutUnit {
        self.offset_
    }
    pub fn Metrics(&self) -> FontHeight {
        self.metrics_
    }
    pub fn ColumnListForTesting(&self) -> &HeapVector<Member<LogicalRubyColumn>> {
        &self.column_list_
    }
    pub fn AddOverChild(&mut self, child: *mut RubyLine) {
        self.over_children_.push_back(Member::from_ptr(child));
    }
    pub fn AddUnderChild(&mut self, child: *mut RubyLine) {
        self.under_children_.push_back(Member::from_ptr(child));
    }
    pub fn OverChildren(&self) -> &HeapVector<Member<RubyLine>> {
        &self.over_children_
    }
    pub fn UnderChildren(&self) -> &HeapVector<Member<RubyLine>> {
        &self.under_children_
    }
    pub fn RelativeOffset(&self) -> LayoutUnit {
        self.relative_offset_
    }
    pub fn SetRelativeOffset(&mut self, offset: LayoutUnit) {
        self.relative_offset_ = offset;
    }
}

impl Traceable for RubyLine {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        RubyLine::Trace(self, visitor);
    }
}

// cpp: layoutng_inline/ruby_utils.h:227-238
#[derive(Clone)]
struct AnnotationDepth {
    column: Member<LogicalRubyColumn>,
    over_depth: i32,
    under_depth: i32,
}

impl AnnotationDepth {
    // cpp: layoutng_inline/ruby_utils.cc:1621-1624
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.column);
    }
}

impl Traceable for AnnotationDepth {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        AnnotationDepth::Trace(self, visitor);
    }
}

// cpp: layoutng_inline/ruby_utils.h:135-298
// STACK_ALLOCATED becomes a stack-owned Rust value with GC Member edges.
pub struct RubyBlockPositionCalculator {
    ruby_lines_: HeapVector<Member<RubyLine>, 2>,
    annotation_metrics_: FontHeight,
}

impl Default for RubyBlockPositionCalculator {
    // cpp: layoutng_inline/ruby_utils.cc:1019
    fn default() -> Self {
        Self {
            ruby_lines_: HeapVector::default(),
            annotation_metrics_: FontHeight::Empty(),
        }
    }
}

#[allow(non_snake_case)]
impl RubyBlockPositionCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    // cpp: layoutng_inline/ruby_utils.cc:1021-1025
    pub fn GroupLines(&mut self, column_list: &HeapVector<Member<LogicalRubyColumn>>) -> &mut Self {
        let root = self.EnsureRubyLine(&RubyLevel::new()) as *mut RubyLine;
        self.HandleRubyLine(unsafe { &*root }, column_list);
        self
    }

    // cpp: layoutng_inline/ruby_utils.cc:1027-1100
    fn HandleRubyLine(
        &mut self,
        current_ruby_line: &RubyLine,
        column_list: &HeapVector<Member<LogicalRubyColumn>>,
    ) -> FontHeight {
        if column_list.empty() {
            return FontHeight::default();
        }

        let create_level_and_update_depth =
            |current: &RubyLevel, current_depth: &AnnotationDepth| {
                let mut depth = current_depth.clone();
                let mut new_level = Vec::with_capacity(current.len() + 1);
                new_level.extend_from_slice(current);
                if unsafe { &*depth.column.Get() }.ruby_position == RubyPosition::kUnder {
                    depth.under_depth -= 1;
                    new_level.push(depth.under_depth);
                } else {
                    depth.over_depth += 1;
                    new_level.push(depth.over_depth);
                }
                (new_level, depth)
            };

        let mut depth_stack = HeapVector::<AnnotationDepth, 1>::default();
        let current_level = current_ruby_line.Level();
        let mut max_annotation_metrics = FontHeight::default();
        for index in 0..column_list.size() {
            depth_stack.push_back(AnnotationDepth {
                column: column_list[index as usize].clone(),
                over_depth: 0,
                under_depth: 0,
            });
            loop {
                if depth_stack.empty() {
                    break;
                }
                let column = unsafe { &*depth_stack.last().unwrap().column.Get() };
                let should_close_column = index + 1 >= column_list.size()
                    || column.EndIndex()
                        <= unsafe { &*column_list[(index + 1) as usize].Get() }.start_index;
                if !should_close_column {
                    break;
                }

                let (annotation_level, closing_depth) =
                    create_level_and_update_depth(current_level, depth_stack.last().unwrap());
                let annotation_line = self.EnsureRubyLine(&annotation_level) as *mut RubyLine;
                let closing_column = unsafe { &mut *closing_depth.column.Get() };
                unsafe { &mut *annotation_line }.Append(closing_column);
                let nested_columns = closing_column.RubyColumnList() as *const _;
                let mut closing_metrics =
                    self.HandleRubyLine(unsafe { &*annotation_line }, unsafe { &*nested_columns });
                unsafe { &mut *annotation_line }.MaybeRecordBaseIndexes(closing_column);

                let mut annotation_height = LayoutUnit::default();
                let annotation_items = closing_column.annotation_items.Get();
                if !annotation_items.is_null() {
                    annotation_height =
                        ComputeLogicalLineEmHeight(unsafe { &*annotation_items }).LineHeight();
                }
                if closing_column.ruby_position == RubyPosition::kOver {
                    closing_metrics.ascent += annotation_height;
                } else {
                    closing_metrics.descent += annotation_height;
                }
                closing_column.annotation_metrics = closing_metrics;
                // The source also assigns a dead local annotation_metrics here.
                max_annotation_metrics.Unite(&closing_metrics);

                depth_stack.pop();
                if let Some(parent_depth) = depth_stack.last_mut() {
                    parent_depth.over_depth = parent_depth.over_depth.max(closing_depth.over_depth);
                    parent_depth.under_depth =
                        parent_depth.under_depth.min(closing_depth.under_depth);
                }
            }
        }
        assert!(depth_stack.empty());
        max_annotation_metrics
    }

    // cpp: layoutng_inline/ruby_utils.cc:1102-1113
    fn EnsureRubyLine(&mut self, level: &RubyLevel) -> &mut RubyLine {
        for line in &self.ruby_lines_ {
            let line = unsafe { &mut *line.Get() };
            if line.Level() == level {
                return line;
            }
        }
        let line = MakeGarbageCollected(RubyLine::new(level));
        self.ruby_lines_.push_back(Member::from_ptr(line));
        unsafe { &mut *line }
    }

    // cpp: layoutng_inline/ruby_utils.cc:1115-1198
    pub fn PlaceLines(
        &mut self,
        base_line_items: &LogicalLineItems,
        line_box_metrics: &FontHeight,
    ) -> &mut Self {
        debug_assert!(!self.ruby_lines_.empty(), "GroupLines must run first");
        self.annotation_metrics_ = FontHeight::default();

        if RuntimeEnabledFeatures::TreeRubyPlacementEnabled() {
            let root = self.BuildTree();
            assert!(!root.is_null());
            let total_subtree_metrics = self.ComputeRelativeOffsets(
                unsafe { &mut *root },
                base_line_items,
                line_box_metrics,
            );
            self.ComputeOffsetsFromBase(unsafe { &mut *root }, LayoutUnit::default());
            if !unsafe { &*root }.OverChildren().empty() {
                self.annotation_metrics_.ascent = total_subtree_metrics.ascent;
            }
            if !unsafe { &*root }.UnderChildren().empty() {
                self.annotation_metrics_.descent = total_subtree_metrics.descent;
            }
            return self;
        }

        self.ruby_lines_.sort_unstable_by(|line1, line2| {
            let line1 = unsafe { &*line1.Get() };
            let line2 = unsafe { &*line2.Get() };
            if line1.LessThan(line2) {
                std::cmp::Ordering::Less
            } else if line2.LessThan(line1) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        let base_index = self
            .ruby_lines_
            .iter()
            .position(|line| unsafe { &*line.Get() }.Level().is_empty())
            .expect("Ruby base line must exist");

        if base_index > 0 {
            let first_under = self.ruby_lines_[..base_index]
                .iter()
                .find(|line| unsafe { &*line.Get() }.IsFirstUnderLevel())
                .expect("first under level must exist");
            let mut em_height = ComputeLogicalLineEmHeightForIndexes(
                base_line_items,
                unsafe { &*first_under.Get() }.BaseIndexList(),
            );
            if em_height.LineHeight() == LayoutUnit::default() {
                em_height = *line_box_metrics;
            }
            let mut offset = em_height.descent;
            for ruby_line in self.ruby_lines_[..base_index].iter().rev() {
                let ruby_line = unsafe { &mut *ruby_line.Get() };
                let metrics = ruby_line.UpdateMetrics();
                offset += metrics.ascent;
                ruby_line.MoveInBlockDirection(offset);
                ruby_line.SetOffset(offset);
                offset += metrics.descent;
            }
            self.annotation_metrics_.descent = offset;
        }

        if base_index + 1 < self.ruby_lines_.len() {
            let first_over = self.ruby_lines_[base_index..]
                .iter()
                .find(|line| unsafe { &*line.Get() }.IsFirstOverLevel())
                .expect("first over level must exist");
            let mut em_height = ComputeLogicalLineEmHeightForIndexes(
                base_line_items,
                unsafe { &*first_over.Get() }.BaseIndexList(),
            );
            if em_height.LineHeight() == LayoutUnit::default() {
                em_height = *line_box_metrics;
            }
            let mut offset = -em_height.ascent;
            for ruby_line in &self.ruby_lines_[base_index + 1..] {
                let ruby_line = unsafe { &mut *ruby_line.Get() };
                let metrics = ruby_line.UpdateMetrics();
                offset -= metrics.descent;
                ruby_line.MoveInBlockDirection(offset);
                ruby_line.SetOffset(offset);
                offset -= metrics.ascent;
            }
            self.annotation_metrics_.ascent = -offset;
        }
        self
    }

    // cpp: layoutng_inline/ruby_utils.cc:1200-1208
    pub fn AddLinesTo(&mut self, line_container: &mut LogicalLineContainer) -> &mut Self {
        debug_assert!(
            !self.annotation_metrics_.IsEmpty(),
            "PlaceLines must run first"
        );
        for ruby_line in &self.ruby_lines_ {
            unsafe { &*ruby_line.Get() }.AddLinesTo(line_container);
        }
        self
    }

    // cpp: layoutng_inline/ruby_utils.cc:1216-1221
    pub fn UpdateColumnLayoutAnnotationMetrics(
        &self,
        column_list: &HeapVector<Member<LogicalRubyColumn>>,
    ) {
        for column_member in column_list {
            self.UpdateColumnLayoutAnnotationMetricsForColumn(
                unsafe { &mut *column_member.Get() },
                LayoutUnit::default(),
            );
        }
    }

    // Rust names the C++ column overload separately.
    // cpp: layoutng_inline/ruby_utils.cc:1223-1258
    fn UpdateColumnLayoutAnnotationMetricsForColumn(
        &self,
        column: &mut LogicalRubyColumn,
        base_offset: LayoutUnit,
    ) {
        let mut associated_line: *const RubyLine = std::ptr::null();
        if !column.annotation_items.Get().is_null() {
            for line_member in &self.ruby_lines_ {
                let line = unsafe { &*line_member.Get() };
                if line.ContainsColumn(column) {
                    associated_line = line;
                    break;
                }
            }
        }
        let mut child_base_offset = base_offset;
        if !associated_line.is_null() {
            child_base_offset = unsafe { &*associated_line }.Offset();
        }
        for sub_column in column.RubyColumnList() {
            self.UpdateColumnLayoutAnnotationMetricsForColumn(
                unsafe { &mut *sub_column.Get() },
                child_base_offset,
            );
        }

        let mut min_offset = LayoutUnit::Max();
        let mut max_offset = LayoutUnit::Min();
        self.AccumulateColumnOffsets(column, &mut min_offset, &mut max_offset);

        let mut new_metrics = FontHeight::default();
        if min_offset != LayoutUnit::Max() {
            new_metrics.ascent = base_offset - min_offset;
        }
        if max_offset != LayoutUnit::Min() {
            new_metrics.descent = max_offset - base_offset;
        }
        column.layout_annotation_metrics = new_metrics;
    }

    // cpp: layoutng_inline/ruby_utils.cc:1260-1294
    fn AccumulateColumnOffsets(
        &self,
        column: &mut LogicalRubyColumn,
        min_offset: &mut LayoutUnit,
        max_offset: &mut LayoutUnit,
    ) {
        let annotation_items = column.annotation_items.Get();
        if !annotation_items.is_null() {
            let mut associated_line: *const RubyLine = std::ptr::null();
            for line_member in &self.ruby_lines_ {
                let line = unsafe { &*line_member.Get() };
                if line.ContainsColumn(column) {
                    associated_line = line;
                    break;
                }
            }
            if !associated_line.is_null() {
                let level = unsafe { &*associated_line }.Level();
                let mut metrics = ComputeLogicalLineEmHeight(unsafe { &*annotation_items });
                let emphasis_metrics = ComputeEmphasisHeights(unsafe { &*annotation_items });
                metrics.ascent += emphasis_metrics.ascent;
                metrics.descent += emphasis_metrics.descent;
                if !level.is_empty() && level[0] > 0 {
                    let start = -metrics.ascent;
                    *min_offset = (*min_offset).min(start);
                } else if !level.is_empty() && level[0] < 0 {
                    let end = metrics.descent;
                    *max_offset = (*max_offset).max(end);
                }
            }
        }
        // C++ const_cast exposes RubyColumnList here. This private Rust helper
        // takes &mut so it can reach the same list without casting away const.
        for sub_column in column.RubyColumnList() {
            self.AccumulateColumnOffsets(unsafe { &mut *sub_column.Get() }, min_offset, max_offset);
        }
    }

    // cpp: layoutng_inline/ruby_utils.cc:1296-1333
    fn BuildTree(&mut self) -> *mut RubyLine {
        let mut root = std::ptr::null_mut();
        for line_member in &self.ruby_lines_ {
            let line = unsafe { &*line_member.Get() };
            if line.IsBaseLevel() {
                root = line_member.Get();
            }
        }

        for line_member in &self.ruby_lines_ {
            let line = unsafe { &*line_member.Get() };
            if line.IsBaseLevel() {
                continue;
            }
            let level = line.Level();
            debug_assert!(!level.is_empty());
            let parent_level: RubyLevel = level[..level.len() - 1].to_vec();
            if let Some(parent_member) = self
                .ruby_lines_
                .iter()
                .find(|candidate| unsafe { &*candidate.Get() }.Level() == &parent_level)
            {
                let parent = unsafe { &mut *parent_member.Get() };
                if *level.last().unwrap() > 0 {
                    parent.AddOverChild(line_member.Get());
                } else {
                    parent.AddUnderChild(line_member.Get());
                }
            }
        }
        for line_member in &self.ruby_lines_ {
            unsafe { &mut *line_member.Get() }.SortChildren();
        }
        root
    }

    // cpp: layoutng_inline/ruby_utils.cc:1335-1469
    fn ComputeRelativeOffsets(
        &mut self,
        node: &mut RubyLine,
        base_line_items: &LogicalLineItems,
        line_box_metrics: &FontHeight,
    ) -> FontHeight {
        let mut node_metrics = FontHeight::default();
        if node.IsBaseLevel() {
            if let Some(first_over) = node.OverChildren().first() {
                node_metrics = ComputeLogicalLineEmHeightForIndexes(
                    base_line_items,
                    unsafe { &*first_over.Get() }.BaseIndexList(),
                );
            } else if let Some(first_under) = node.UnderChildren().first() {
                node_metrics = ComputeLogicalLineEmHeightForIndexes(
                    base_line_items,
                    unsafe { &*first_under.Get() }.BaseIndexList(),
                );
            }
            if node_metrics.LineHeight() == LayoutUnit::default() {
                node_metrics = *line_box_metrics;
            }
        } else {
            node_metrics = node.UpdateMetrics();
        }

        let mut subtree_ascent = node_metrics.ascent;
        let mut subtree_descent = node_metrics.descent;
        let node_emphasis = node.ComputeLevelEmphasisHeights(base_line_items);

        if !node.OverChildren().empty() {
            let mut current_offset = -node_metrics.ascent;
            let mut index = 0;
            while index < node.OverChildren().len() {
                let child = unsafe { &mut *node.OverChildren()[index].Get() };
                if child.Level().len() <= node.Level().len() {
                    break;
                }
                let child_subtree_metrics =
                    self.ComputeRelativeOffsets(child, base_line_items, line_box_metrics);
                let child_relative_offset = current_offset - child_subtree_metrics.descent;
                child.SetRelativeOffset(child_relative_offset);
                current_offset = child_relative_offset - child_subtree_metrics.ascent;
                index += 1;
            }
            subtree_ascent = subtree_ascent.max(-current_offset);

            let mut emphasis_top_with_anno = LayoutUnit::Max();
            if index > 0 {
                let previous = unsafe { &*node.OverChildren()[index - 1].Get() };
                let emp_with_anno =
                    node.ComputeParentEmphasisHeightsForChild(previous, base_line_items);
                emphasis_top_with_anno = current_offset - emp_with_anno.ascent;
            }
            let emphasis_top_without_anno = -node_metrics.ascent - node_emphasis.ascent;
            let emphasis_top = emphasis_top_with_anno.min(emphasis_top_without_anno);
            subtree_ascent = subtree_ascent.max(-emphasis_top);

            while index < node.OverChildren().len() {
                let child = unsafe { &mut *node.OverChildren()[index].Get() };
                let child_subtree_metrics =
                    self.ComputeRelativeOffsets(child, base_line_items, line_box_metrics);
                let parent_emphasis_for_child =
                    node.ComputeParentEmphasisHeightsForChild(child, base_line_items);
                let child_bottom = current_offset - parent_emphasis_for_child.ascent;
                let child_relative_offset = child_bottom - child_subtree_metrics.descent;
                child.SetRelativeOffset(child_relative_offset);
                current_offset = child_relative_offset - child_subtree_metrics.ascent;
                subtree_ascent = subtree_ascent.max(-current_offset);
                index += 1;
            }
        } else {
            subtree_ascent += node_emphasis.ascent;
        }

        if !node.UnderChildren().empty() {
            let mut current_offset = node_metrics.descent;
            let mut index = 0;
            while index < node.UnderChildren().len() {
                let child = unsafe { &mut *node.UnderChildren()[index].Get() };
                if child.Level().len() <= node.Level().len() {
                    break;
                }
                let child_subtree_metrics =
                    self.ComputeRelativeOffsets(child, base_line_items, line_box_metrics);
                let child_relative_offset = current_offset + child_subtree_metrics.ascent;
                child.SetRelativeOffset(child_relative_offset);
                current_offset = child_relative_offset + child_subtree_metrics.descent;
                index += 1;
            }
            subtree_descent = subtree_descent.max(current_offset);

            let mut emphasis_bottom_with_anno = LayoutUnit::Min();
            if index > 0 {
                let previous = unsafe { &*node.UnderChildren()[index - 1].Get() };
                let emp_with_anno =
                    node.ComputeParentEmphasisHeightsForChild(previous, base_line_items);
                emphasis_bottom_with_anno = current_offset + emp_with_anno.descent;
            }
            let emphasis_bottom_without_anno = node_metrics.descent + node_emphasis.descent;
            let emphasis_bottom = emphasis_bottom_with_anno.max(emphasis_bottom_without_anno);
            subtree_descent = subtree_descent.max(emphasis_bottom);

            while index < node.UnderChildren().len() {
                let child = unsafe { &mut *node.UnderChildren()[index].Get() };
                let child_subtree_metrics =
                    self.ComputeRelativeOffsets(child, base_line_items, line_box_metrics);
                let parent_emphasis_for_child =
                    node.ComputeParentEmphasisHeightsForChild(child, base_line_items);
                let child_top = current_offset + parent_emphasis_for_child.descent;
                let child_relative_offset = child_top + child_subtree_metrics.ascent;
                child.SetRelativeOffset(child_relative_offset);
                current_offset = child_relative_offset + child_subtree_metrics.descent;
                subtree_descent = subtree_descent.max(current_offset);
                index += 1;
            }
        } else {
            subtree_descent += node_emphasis.descent;
        }

        FontHeight::new(subtree_ascent, subtree_descent)
    }

    // cpp: layoutng_inline/ruby_utils.cc:1471-1484
    fn ComputeOffsetsFromBase(&mut self, node: &mut RubyLine, parent_offset_from_base: LayoutUnit) {
        let offset_from_base = parent_offset_from_base + node.RelativeOffset();
        node.SetOffset(offset_from_base);
        node.MoveInBlockDirection(offset_from_base);
        for child in node.OverChildren() {
            self.ComputeOffsetsFromBase(unsafe { &mut *child.Get() }, offset_from_base);
        }
        for child in node.UnderChildren() {
            self.ComputeOffsetsFromBase(unsafe { &mut *child.Get() }, offset_from_base);
        }
    }

    // cpp: layoutng_inline/ruby_utils.h:263-269
    pub fn AnnotationMetrics(&self) -> FontHeight {
        debug_assert!(!self.annotation_metrics_.IsEmpty());
        self.annotation_metrics_
    }
    pub fn RubyLineListForTesting(&self) -> &HeapVector<Member<RubyLine>, 2> {
        &self.ruby_lines_
    }
}
