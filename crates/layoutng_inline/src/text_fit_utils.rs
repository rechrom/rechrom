// C++: layoutng_inline/text_fit_utils.h/.cc. The line and paragraph fitting
// bodies are source-mapped here; shared inline assembly remains pending.
#![allow(non_snake_case)]

use font_engine::fonts::font_description::FontDescription;
use font_engine::fonts::shaping::shape_options::ShapeOptions;
use font_engine::fonts::shaping::shape_result::{OffsetWithSpacing, ShapeResult};
use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use font_engine::fonts::shaping::text_spacing_trim::ShouldTrimStartOfParagraph;
use font_engine::text::native::character::Character;
use font_engine::{Font, HarfBuzzShaper, ShapeResultSpacing};
use foundation::{
    DynamicTo, LayoutUnit, Length, MakeGarbageCollected, Member, RuntimeEnabledFeatures,
    WritingMode,
};
use layoutng::internal::inline_item::{InlineItem, InlineItemType};
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::text_fit_scale::TextFitScale;
use layoutng::internal::text_item_type::TextItemType;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_fragment_tree::inline_items_data::InlineItemsData;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_geometry::geometry::logical_size::ToLogicalSize;
use layoutng_style::style::text_fit::{TextFitMethod, TextFitTarget, TextFitType};

use crate::line_info::LineInfo;

// cpp: layoutng_inline/text_fit_utils.cc:20-58
fn scale_line(
    is_grow: bool,
    mut scale_factor: f32,
    limit: Option<f32>,
    line_info: &mut LineInfo,
) -> bool {
    let mut should_scale_line_height = false;
    let mut inline_size = line_info.TextIndent();
    for item in line_info.MutableResults() {
        let inline_item = unsafe { &*item.item.Get() };
        if inline_item.Type() != InlineItemType::kText
            && inline_item.TextType() != TextItemType::kForcedLineBreak
        {
            inline_size += item.inline_size;
            continue;
        }
        if item.text_fit_scale.Get().is_null() {
            item.text_fit_scale = Member::from_ptr(MakeGarbageCollected(TextFitScale::default()));
        }
        let fit_scale = unsafe { &mut *item.text_fit_scale.Get() };
        if let Some(limit) = limit {
            let max_or_min_scale = limit / unsafe { &*inline_item.Style() }.ComputedFontSize();
            fit_scale.scale = if is_grow {
                scale_factor.min(max_or_min_scale)
            } else {
                scale_factor.max(max_or_min_scale)
            };
        } else {
            fit_scale.scale = scale_factor;
        }
        if fit_scale.scale != 1.0 {
            should_scale_line_height = true;
        }
        inline_size += item.inline_size * fit_scale.scale;
    }
    line_info.SetWidth(line_info.AvailableWidth(), inline_size);
    if let Some(limit) = limit {
        if !is_grow {
            scale_factor = scale_factor.max(limit / line_info.LineStyle().ComputedFontSize());
        }
    }
    line_info.SetTextFitScale(scale_factor);
    should_scale_line_height
}

// cpp: layoutng_inline/text_fit_utils.cc:60-107
fn shape_for_fit(
    item: &InlineItem,
    start_offset: u32,
    end_offset: u32,
    shaper: &HarfBuzzShaper,
    font: &Font,
    segments: *const layoutng::internal::inline_item_segment::InlineItemSegments,
    is_start_of_paragraph: bool,
) -> *mut ShapeResult {
    let mut options = ShapeOptions::default();
    if is_start_of_paragraph {
        options.is_line_start = true;
        let text_content = shaper.GetText();
        options.han_kerning_start = start_offset < text_content.length()
            && ShouldTrimStartOfParagraph(font.GetFontDescription().GetTextSpacingTrim())
            && Character::MaybeHanKerningOpen(i32::from(
                text_content.Span16().unwrap()[start_offset as usize],
            ));
    }
    let result = if !segments.is_null() {
        unsafe { &*segments }.ShapeText(
            shaper,
            font,
            item.Direction(),
            start_offset,
            end_offset,
            item.Index(),
            options,
        )
    } else {
        let mut range = item.CreateRunSegmenterRange();
        range.end = end_offset;
        shaper.ShapeSingleRange(
            font,
            item.Direction(),
            start_offset,
            end_offset,
            range,
            options,
        )
    };

    let original = item.TextShapeResult();
    if !original.is_null() {
        let original = unsafe { &*original };
        original.EnsurePositionData(true);
        let mut offsets = Vec::new();
        let spacing = font.TextAutoSpaceInlineSize();
        for offset in start_offset..end_offset {
            if original.HasAutoSpacingAfter(offset) {
                offsets.push(OffsetWithSpacing {
                    offset: offset + 1,
                    spacing,
                });
            }
        }
        if !offsets.is_empty() {
            unsafe { &mut *result }.ApplyTextAutoSpacing(&offsets);
        }
    }
    result
}

// cpp: layoutng_inline/text_fit_utils.cc:109-115
fn minimum_size(is_grow: bool, node: &InlineNode) -> Option<f32> {
    if !is_grow {
        return node.MinimumFontPhysicalSize();
    }
    None
}

// cpp: layoutng_inline/text_fit_utils.cc:117-127
fn has_fixed_spacing(description: &FontDescription) -> bool {
    let letter_spacing = description.ComputedLetterSpacing();
    if letter_spacing.IsFixed() && !letter_spacing.IsZero() {
        return true;
    }
    let word_spacing = description.ComputedWordSpacing();
    word_spacing.IsFixed() && !word_spacing.IsZero()
}

// cpp: layoutng_inline/text_fit_utils.cc:129-141
fn percentage_spacing_description(description: &FontDescription) -> FontDescription {
    let mut copy = description.clone();
    let letter_spacing = description.ComputedLetterSpacing();
    if letter_spacing.IsFixed() && !letter_spacing.IsZero() {
        copy.SetLetterSpacing(&Length::Fixed(0.0));
    }
    let word_spacing = description.ComputedWordSpacing();
    if word_spacing.IsFixed() && !word_spacing.IsZero() {
        copy.SetWordSpacing(&Length::Fixed(0.0));
    }
    copy
}

// cpp: layoutng_inline/text_fit_utils.cc:143-150
fn restrict_scale(scale: f32, is_grow: bool, limit: Option<f32>) -> f32 {
    let Some(limit) = limit else { return scale };
    if is_grow {
        scale.min(limit.max(1.0))
    } else {
        scale.max(limit.min(1.0))
    }
}

// cpp: layoutng_inline/text_fit_utils.cc:152-171
fn scaled_font_description(
    font: &Font,
    scale_factor: f32,
    limit: Option<f32>,
    restricted: &mut bool,
) -> FontDescription {
    let mut item_scale = scale_factor;
    let original_size = font.GetFontDescription().ComputedSize();
    if let Some(limit) = limit {
        item_scale = if item_scale > 1.0 {
            scale_factor.min(limit / original_size)
        } else {
            scale_factor.max(limit / original_size)
        };
        if item_scale != scale_factor {
            *restricted = true;
        }
    }
    let mut scaled = font.GetFontDescription().clone();
    scaled.SetComputedSize(original_size * item_scale);
    scaled
}

// cpp: layoutng_inline/text_fit_utils.cc:173-223
fn compute_additional_paint_time_scale(
    items_data: &InlineItemsData,
    available_width: LayoutUnit,
    epsilon: LayoutUnit,
    _writing_mode: WritingMode,
    shaper: &HarfBuzzShaper,
    spacing: &mut ShapeResultSpacing,
    line: &InlineCursor,
    mut is_start_of_paragraph: bool,
    scale: f32,
    limit: Option<f32>,
    static_total_size: LayoutUnit,
) -> f32 {
    let mut flexible_total_size = LayoutUnit::default();
    let mut descendants = line.CursorForDescendants();
    while descendants.IsNotNull() {
        let current = descendants.Current();
        if current.IsText() && !current.TextShapeResult().is_null() {
            let font = unsafe { &*current.Style().GetFont() };
            let mut restricted = false;
            let scaled_description = scaled_font_description(font, scale, limit, &mut restricted);
            if restricted {
                return 1.0;
            }
            let scaled_font = MakeGarbageCollected(Font::new_with_selector(
                scaled_description.clone(),
                font.GetFontSelector(),
            ));
            let item_ptr = items_data
                .items
                .iter()
                .find(|item_ptr| {
                    let item = unsafe { &*item_ptr.Get() };
                    item.StartOffset() <= current.TextStartOffset()
                        && current.TextEndOffset() <= item.EndOffset()
                })
                .expect("text fragment must belong to an inline item");
            let shape_result = shape_for_fit(
                unsafe { &*item_ptr.Get() },
                current.TextStartOffset(),
                current.TextEndOffset(),
                shaper,
                unsafe { &*scaled_font },
                items_data.segments.Get(),
                is_start_of_paragraph,
            );
            is_start_of_paragraph = false;
            if spacing.SetSpacingFromDescription(&scaled_description) {
                unsafe { &mut *shape_result }.ApplySpacing(spacing, 0);
            }
            flexible_total_size += unsafe { &*shape_result }
                .SnappedWidth()
                .ClampNegativeToZero();
        }
        descendants.MoveToNextInlineLeaf();
    }
    let remaining_space = available_width - (flexible_total_size + static_total_size);
    if remaining_space.Abs() >= epsilon {
        (flexible_total_size + remaining_space).ToFloat() / flexible_total_size.ToFloat()
    } else {
        1.0
    }
}

// cpp: layoutng_inline/text_fit_utils.cc:226-240
#[unsafe(no_mangle)]
pub extern "Rust" fn ShouldApplyTextFitFromInline(node: &InlineNode) -> bool {
    if !RuntimeEnabledFeatures::CssTextFitEnabled() {
        return false;
    }
    if node.Style().GetTextFit().Type() == TextFitType::kNone {
        return false;
    }
    if node.HasFloats() || node.HasInitialLetterBox() || node.HasRuby() {
        return false;
    }
    true
}

// cpp: layoutng_inline/text_fit_utils.h:16-16
pub fn ShouldApplyTextFit(node: &InlineNode) -> bool {
    ShouldApplyTextFitFromInline(node)
}

// cpp: layoutng_inline/text_fit_utils.h:20-27
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParagraphScale {
    pub scale: f32,
    pub additional_paint_time_scale: f32,
}

impl Default for ParagraphScale {
    fn default() -> Self {
        Self {
            scale: 1.0,
            additional_paint_time_scale: 1.0,
        }
    }
}

// cpp: layoutng_inline/text_fit_utils.h:34-36
// The definition is source-mapped below with the other .cc bodies.
pub fn MeasurePerBlockScale(
    node: &InlineNode,
    fragment: &PhysicalFragment,
    available_width: LayoutUnit,
) -> ParagraphScale {
    MeasurePerBlockScaleFromInline(node, fragment, available_width)
}

// cpp: layoutng_inline/text_fit_utils.cc:242-372
#[unsafe(no_mangle)]
pub extern "Rust" fn MeasurePerBlockScaleFromInline(
    node: &InlineNode,
    fragment: &PhysicalFragment,
    available_width: LayoutUnit,
) -> ParagraphScale {
    let box_fragment = DynamicTo::<PhysicalBoxFragment>(fragment);
    if box_fragment.is_null() {
        return ParagraphScale::default();
    }
    let box_fragment = unsafe { &*box_fragment };
    let items = box_fragment.Items();
    if items.is_null() {
        return ParagraphScale::default();
    }
    let items = unsafe { &*items };

    let pixel_ratio = unsafe { &*node.GetLayoutBox() }.DevicePixelRatioForLayout();
    let epsilon = LayoutUnit::from_f32((2.0 * pixel_ratio) as f32);
    let mut minimum_scale = f32::INFINITY;
    let writing_mode = box_fragment.Style().GetWritingMode();

    let mut is_grow = true;
    let mut cursor = InlineCursor::new_with_items(box_fragment, items);
    while cursor.IsNotNull() {
        if cursor.Current().IsLineBox() {
            let remaining_space =
                available_width - ToLogicalSize(cursor.Current().Size(), writing_mode).inline_size;
            if remaining_space.Abs() >= epsilon && remaining_space < LayoutUnit::default() {
                is_grow = false;
                break;
            }
        }
        cursor.MoveToNextSkippingChildren();
    }

    let text_fit = node.Style().GetTextFit();
    if text_fit.Target() != TextFitTarget::kConsistent {
        return ParagraphScale::default();
    }
    let mut additional_paint_time_scale = 1.0;
    let mut is_next_start_of_paragraph = true;
    let mut cursor = InlineCursor::new_with_items(box_fragment, items);
    while cursor.IsNotNull() {
        if !cursor.Current().IsLineBox() {
            cursor.MoveToNextSkippingChildren();
            continue;
        }
        let mut is_start_of_paragraph = is_next_start_of_paragraph;
        is_next_start_of_paragraph = false;
        let remaining_space =
            available_width - ToLogicalSize(cursor.Current().Size(), writing_mode).inline_size;
        if remaining_space.Abs() < epsilon {
            cursor.MoveToNextSkippingChildren();
            continue;
        }
        let mut flexible_total_size = LayoutUnit::default();
        let mut flexible_including_letter_spacing = LayoutUnit::default();
        let items_data = node.ItemsData(unsafe { &*cursor.CurrentItem() }.UsesFirstLineStyle());
        let shaper = HarfBuzzShaper::new(items_data.text_content.clone());
        let mut spacing = ShapeResultSpacing::new(&items_data.text_content, false);
        let mut did_reshape = false;
        let limit = minimum_size(is_grow, node);
        let mut descendants = cursor.CursorForDescendants();
        while descendants.IsNotNull() {
            let current = descendants.Current();
            if current.IsLineBreak() {
                is_next_start_of_paragraph = true;
                descendants.MoveToNextInlineLeaf();
                continue;
            }
            if !current.IsText() || current.TextShapeResult().is_null() {
                descendants.MoveToNextInlineLeaf();
                continue;
            }
            let style = current.Style();
            if has_fixed_spacing(style.GetFontDescription()) {
                let start = current.TextStartOffset();
                let end = current.TextEndOffset();
                let item_ptr = items_data
                    .items
                    .iter()
                    .find(|item_ptr| {
                        let item = unsafe { &*item_ptr.Get() };
                        item.StartOffset() <= start && end <= item.EndOffset()
                    })
                    .expect("text fragment must belong to an inline item");
                let no_spacing_shape = shape_for_fit(
                    unsafe { &*item_ptr.Get() },
                    start,
                    end,
                    &shaper,
                    unsafe { &*style.GetFont() },
                    items_data.segments.Get(),
                    is_start_of_paragraph,
                );
                is_start_of_paragraph = false;
                did_reshape = true;
                if spacing.SetSpacingFromDescription(&percentage_spacing_description(
                    style.GetFontDescription(),
                )) {
                    unsafe { &mut *no_spacing_shape }.ApplySpacing(&mut spacing, 0);
                }
                flexible_total_size += unsafe { &*no_spacing_shape }
                    .SnappedWidth()
                    .ClampNegativeToZero();
            } else {
                flexible_total_size += ToLogicalSize(current.Size(), writing_mode).inline_size;
            }
            flexible_including_letter_spacing +=
                ToLogicalSize(current.Size(), writing_mode).inline_size;
            descendants.MoveToNextInlineLeaf();
        }
        if flexible_total_size == LayoutUnit::default()
            || remaining_space + flexible_total_size <= LayoutUnit::default()
        {
            cursor.MoveToNextSkippingChildren();
            continue;
        }
        let scale =
            (remaining_space + flexible_total_size).ToFloat() / flexible_total_size.ToFloat();
        if scale < minimum_scale {
            minimum_scale = scale;
            if text_fit.Method() == TextFitMethod::kFontSize || did_reshape {
                let static_total_size =
                    available_width - remaining_space - flexible_including_letter_spacing;
                additional_paint_time_scale = compute_additional_paint_time_scale(
                    items_data,
                    available_width,
                    epsilon,
                    writing_mode,
                    &shaper,
                    &mut spacing,
                    &cursor,
                    is_start_of_paragraph,
                    scale,
                    limit,
                    static_total_size,
                );
            }
        }
        cursor.MoveToNextSkippingChildren();
    }
    if minimum_scale.is_finite() {
        minimum_scale = restrict_scale(
            minimum_scale,
            text_fit.Type() == TextFitType::kGrow,
            text_fit.ScaleFactorLimit(),
        );
        ParagraphScale {
            scale: minimum_scale,
            additional_paint_time_scale,
        }
    } else {
        ParagraphScale {
            scale: 1.0,
            additional_paint_time_scale,
        }
    }
}

// cpp: layoutng_inline/text_fit_utils.h:41-70
// C++ stores references to the line and items data. Raw pointers express the
// same non-owning links while the stack-scoped fitter holds its owned shapers.
pub struct LineFitter {
    node_: InlineNode,
    line_info_: *mut LineInfo,
    items_data_: *const InlineItemsData,
    shaper_: HarfBuzzShaper,
    spacing_: ShapeResultSpacing,
    device_pixel_ratio_: f64,
    epsilon_: LayoutUnit,
}

impl LineFitter {
    // cpp: layoutng_inline/text_fit_utils.h:45-45
    pub fn new(node: &InlineNode, line_info: *mut LineInfo) -> Self {
        LineFitterNewFromInline(node, line_info)
    }

    // cpp: layoutng_inline/text_fit_utils.h:54-55
    pub fn FitLine(&mut self, scale_factor: f32, additional_paint_time_scale: Option<f32>) -> bool {
        LineFitterFitLineFromInline(self, scale_factor, additional_paint_time_scale)
    }

    // cpp: layoutng_inline/text_fit_utils.h:58-58
    pub fn MeasureAndFitLine(&mut self) -> bool {
        LineFitterMeasureAndFitLineFromInline(self)
    }

    // cpp: layoutng_inline/text_fit_utils.h:62-63
    fn MeasureScale(&mut self) -> f32 {
        LineFitterMeasureScaleFromInline(self)
    }
}

// cpp: layoutng_inline/text_fit_utils.cc:374-381
#[unsafe(no_mangle)]
pub extern "Rust" fn LineFitterNewFromInline(
    node: &InlineNode,
    line_info: *mut LineInfo,
) -> LineFitter {
    let items_data = node.ItemsData(unsafe { &*line_info }.UseFirstLineStyle());
    let device_pixel_ratio = unsafe { &*node.GetLayoutBox() }.DevicePixelRatioForLayout();
    LineFitter {
        node_: node.clone(),
        line_info_: line_info,
        items_data_: items_data,
        shaper_: HarfBuzzShaper::new(items_data.text_content.clone()),
        spacing_: ShapeResultSpacing::new(&items_data.text_content, false),
        device_pixel_ratio_: device_pixel_ratio,
        epsilon_: LayoutUnit::from_f32((2.0 * device_pixel_ratio) as f32),
    }
}

// cpp: layoutng_inline/text_fit_utils.cc:383-441
#[unsafe(no_mangle)]
pub extern "Rust" fn LineFitterMeasureScaleFromInline(fitter: &mut LineFitter) -> f32 {
    let line_info = unsafe { &mut *fitter.line_info_ };
    let original_width = line_info.Width();
    let container_width = line_info.AvailableWidth();
    let diff = container_width - original_width;
    if diff.Abs() < fitter.epsilon_ {
        return 1.0;
    }
    let text_fit = fitter.node_.Style().GetTextFit();
    let target = text_fit.Target();
    let apply_text_grow =
        text_fit.Type() == TextFitType::kGrow && target != TextFitTarget::kConsistent;
    let apply_text_shrink =
        text_fit.Type() == TextFitType::kShrink && target != TextFitTarget::kConsistent;
    if (diff > LayoutUnit::default() && !apply_text_grow)
        || (diff < LayoutUnit::default() && !apply_text_shrink)
    {
        return 1.0;
    }
    if target == TextFitTarget::kPerLine && line_info.IsLastLine() {
        return 1.0;
    }

    let mut static_total_size = LayoutUnit::default();
    let mut flexible_total_size = LayoutUnit::default();
    let mut is_first_text = true;
    let is_start_of_paragraph = line_info.IsStartOfParagraph();
    for result in line_info.MutableResults() {
        let item = unsafe { &*result.item.Get() };
        if item.Type() == InlineItemType::kText {
            let style = unsafe { &*item.Style() };
            if has_fixed_spacing(style.GetFontDescription()) {
                let no_spacing_shape = shape_for_fit(
                    item,
                    result.StartOffset(),
                    result.EndOffset(),
                    &fitter.shaper_,
                    unsafe { &*style.GetFont() },
                    unsafe { &*fitter.items_data_ }.segments.Get(),
                    is_start_of_paragraph && is_first_text,
                );
                if fitter
                    .spacing_
                    .SetSpacingFromDescription(&percentage_spacing_description(
                        style.GetFontDescription(),
                    ))
                {
                    unsafe { &mut *no_spacing_shape }.ApplySpacing(&mut fitter.spacing_, 0);
                }
                let size = unsafe { &*no_spacing_shape }
                    .SnappedWidth()
                    .ClampNegativeToZero();
                flexible_total_size += size;
                static_total_size += result.inline_size - size;
            } else {
                flexible_total_size += result.inline_size;
            }
            is_first_text = false;
        } else {
            static_total_size += result.inline_size;
        }
    }
    if flexible_total_size <= LayoutUnit::default() {
        return f32::INFINITY;
    }
    let scale_factor =
        (container_width - static_total_size).ToFloat() / flexible_total_size.ToFloat();
    restrict_scale(
        scale_factor,
        text_fit.Type() == TextFitType::kGrow,
        text_fit.ScaleFactorLimit(),
    )
}

// cpp: layoutng_inline/text_fit_utils.cc:443-546
#[unsafe(no_mangle)]
pub extern "Rust" fn LineFitterFitLineFromInline(
    fitter: &mut LineFitter,
    scale_factor: f32,
    additional_paint_time_scale: Option<f32>,
) -> bool {
    let is_grow = scale_factor > 1.0;
    let text_fit = fitter.node_.Style().GetTextFit();
    let limit = minimum_size(is_grow, &fitter.node_);
    let line_info = unsafe { &mut *fitter.line_info_ };

    let mut contains_fixed_spacing = false;
    for result in line_info.MutableResults() {
        let item = unsafe { &*result.item.Get() };
        if item.Type() == InlineItemType::kText
            && has_fixed_spacing(unsafe { &*item.Style() }.GetFontDescription())
        {
            contains_fixed_spacing = true;
            break;
        }
    }
    if text_fit.Method() == TextFitMethod::kScale && !contains_fixed_spacing {
        return scale_line(is_grow, scale_factor, limit, line_info);
    }

    let mut static_total_size = LayoutUnit::default();
    let mut flexible_total_size = LayoutUnit::default();
    let mut restricted = false;
    let mut is_first_text = true;
    let is_start_of_paragraph = line_info.IsStartOfParagraph();
    for result in line_info.MutableResults() {
        let item = unsafe { &*result.item.Get() };
        if item.Type() != InlineItemType::kText {
            if item.IsForcedLineBreak() {
                let font = unsafe { &*unsafe { &*item.Style() }.GetFont() };
                let scaled_font = MakeGarbageCollected(Font::new_with_selector(
                    scaled_font_description(font, scale_factor, limit, &mut restricted),
                    font.GetFontSelector(),
                ));
                if result.text_fit_scale.Get().is_null() {
                    result.text_fit_scale =
                        Member::from_ptr(MakeGarbageCollected(TextFitScale::default()));
                }
                let fit_scale = unsafe { &mut *result.text_fit_scale.Get() };
                fit_scale.font = Member::from_ptr(scaled_font);
                fit_scale.scale = 1.0;
            }
            static_total_size += result.inline_size;
            continue;
        }
        let font = unsafe { &*unsafe { &*item.Style() }.GetFont() };
        let scaled_description =
            scaled_font_description(font, scale_factor, limit, &mut restricted);
        let scaled_font = MakeGarbageCollected(Font::new_with_selector(
            scaled_description.clone(),
            font.GetFontSelector(),
        ));
        let shape_result = shape_for_fit(
            item,
            result.StartOffset(),
            result.EndOffset(),
            &fitter.shaper_,
            unsafe { &*scaled_font },
            unsafe { &*fitter.items_data_ }.segments.Get(),
            is_start_of_paragraph && is_first_text,
        );
        is_first_text = false;
        let size_without_spacing = unsafe { &*shape_result }
            .SnappedWidth()
            .ClampNegativeToZero();
        if fitter
            .spacing_
            .SetSpacingFromDescription(&scaled_description)
        {
            unsafe { &mut *shape_result }.ApplySpacing(&mut fitter.spacing_, 0);
            result.inline_size = unsafe { &*shape_result }
                .SnappedWidth()
                .ClampNegativeToZero();
            static_total_size += result.inline_size - size_without_spacing;
        } else {
            result.inline_size = size_without_spacing;
        }
        result.shape_result = Member::from_ptr(ShapeResultView::CreateFromResult(shape_result));
        if result.text_fit_scale.Get().is_null() {
            result.text_fit_scale = Member::from_ptr(MakeGarbageCollected(TextFitScale::default()));
        }
        let fit_scale = unsafe { &mut *result.text_fit_scale.Get() };
        fit_scale.font = Member::from_ptr(scaled_font);
        fit_scale.scale = 1.0;
        flexible_total_size += size_without_spacing;
    }

    let mut total_scale = scale_factor;
    if !restricted {
        let paint_scale_limit = text_fit.ScaleFactorLimit().map(|limit| limit / total_scale);
        if let Some(additional_scale) = additional_paint_time_scale {
            if additional_scale != 1.0 {
                let paint_scale = restrict_scale(additional_scale, is_grow, paint_scale_limit);
                scale_line(is_grow, paint_scale, limit, line_info);
                total_scale = paint_scale;
            }
        } else {
            let container_width = line_info.AvailableWidth();
            if (container_width - line_info.ComputeWidth()).Abs() >= fitter.epsilon_ {
                let paint_scale =
                    (container_width - static_total_size).ToFloat() / flexible_total_size.ToFloat();
                let paint_scale = restrict_scale(paint_scale, is_grow, paint_scale_limit);
                scale_line(is_grow, paint_scale, limit, line_info);
                total_scale *= paint_scale;
            }
        }
    }
    let available_width = line_info.AvailableWidth();
    let computed_width = line_info.ComputeWidth();
    line_info.SetWidth(available_width, computed_width);
    if let Some(limit) = limit {
        if !is_grow {
            total_scale = total_scale.max(limit / line_info.LineStyle().ComputedFontSize());
        }
    }
    line_info.SetTextFitScale(total_scale);
    true
}

// cpp: layoutng_inline/text_fit_utils.cc:548-552
#[unsafe(no_mangle)]
pub extern "Rust" fn LineFitterMeasureAndFitLineFromInline(fitter: &mut LineFitter) -> bool {
    let scale_factor = fitter.MeasureScale();
    scale_factor.is_finite() && scale_factor != 1.0 && fitter.FitLine(scale_factor, None)
}
