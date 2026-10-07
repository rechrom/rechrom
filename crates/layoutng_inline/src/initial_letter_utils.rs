// C++: layoutng_inline/initial_letter_utils.h/.cc.
#![allow(non_snake_case)]

use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use font_engine::FontHeight;
use foundation::{EFloat, ETextOrientation, LayoutUnit, To};
use layoutng::internal::exclusions::exclusion_area::ExclusionArea;
use layoutng::internal::inline_item::InlineItemType;
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::logical_line_item::LogicalLineItems;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::bfc_rect::BfcRect;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::line_info::LineInfo;
use crate::line_utils::CalculateLeadingSpace;

// cpp: layoutng_inline/initial_letter_utils.cc:29-116
fn ComputeInitialLetterBoxBlockOffset(
    initial_letter_box_fragment: &PhysicalBoxFragment,
    block_size: LayoutUnit,
    initial_letter_box_style: &ComputedStyle,
    paragraph_style: &ComputedStyle,
    initial_letter_block_start_adjust: &mut LayoutUnit,
) -> LayoutUnit {
    let initial_letter = initial_letter_box_style.InitialLetter();
    debug_assert!(!initial_letter.IsNormal());
    let line_height = paragraph_style.ComputedLineHeightAsFixed();
    let size = initial_letter.Size().ceil() as i32;
    let sink = if initial_letter.IsRaise() || initial_letter.IsIntegerSink() {
        initial_letter.Sink()
    } else {
        size
    };

    if size < sink {
        return line_height * sink - block_size;
    }
    *initial_letter_block_start_adjust = line_height * (size - sink);

    if paragraph_style.IsHorizontalTypographicMode()
        || initial_letter_box_style.GetTextOrientation() == ETextOrientation::kSideways
    {
        let baseline = initial_letter_box_fragment
            .FirstBaseline()
            .unwrap_or_default();
        let ascent = if paragraph_style.IsFlippedLinesWritingMode() {
            block_size - baseline
        } else {
            baseline
        };
        let block_offset = LayoutUnit::from_f32(line_height * initial_letter.Size()) - ascent;
        let text_metrics = paragraph_style.GetFontHeightForDefaultBaseline();
        let mut line_metrics = text_metrics;
        let leading_space =
            CalculateLeadingSpace(&paragraph_style.ComputedLineHeightAsFixed(), &line_metrics);
        line_metrics.AddLeading(&leading_space);
        let descent = line_metrics.descent;
        return block_offset - descent;
    }

    (line_height * size - block_size) / 2
}

// cpp: layoutng_inline/initial_letter_utils.cc:118-140
fn ComputeTextInkBounds(
    shape_result: &ShapeResultView,
    style: &ComputedStyle,
    out_baseline: Option<&mut LayoutUnit>,
) -> LogicalRect {
    let text_ink_float_bounds = shape_result.ComputeInkBounds();
    let text_ink_bounds = LogicalRect::EnclosingRect(&text_ink_float_bounds);
    let font = unsafe { &*style.GetFont() };
    let primary_font = unsafe { &*font.PrimaryFont() };
    let ascent = primary_font.GetFontMetrics().Ascent();
    if let Some(baseline) = out_baseline {
        *baseline = LayoutUnit::from_signed(ascent);
    }
    text_ink_bounds + LogicalOffset::new(LayoutUnit::default(), LayoutUnit::from_signed(ascent))
}

// cpp: layoutng_inline/initial_letter_utils.cc:143-196
fn CreateExclusionSpaceForInitialLetterBox(
    float_type: EFloat,
    origin: BfcOffset,
    border_box_offset: &BfcOffset,
    border_box_size: &LogicalSize,
    margins: &BoxStrut,
) -> *const ExclusionArea {
    let local_start_offset = BfcOffset::new(
        border_box_offset.line_offset - margins.inline_start,
        border_box_offset.block_offset - margins.block_start,
    );
    let margin_box_size = LogicalSize::new(
        (border_box_size.inline_size + margins.InlineSum()).ClampNegativeToZero(),
        (border_box_size.block_size + margins.BlockSum()).ClampNegativeToZero(),
    );
    let start_offset = BfcOffset::new(
        if float_type == EFloat::kLeft {
            origin.line_offset + local_start_offset.line_offset
        } else {
            origin.line_offset - margin_box_size.inline_size + margins.inline_end
        },
        origin.block_offset + local_start_offset.block_offset.min(LayoutUnit::default()),
    );
    let end_offset = BfcOffset::new(
        start_offset.line_offset + margin_box_size.inline_size,
        origin.block_offset + local_start_offset.block_offset + margin_box_size.block_size,
    );
    ExclusionArea::CreateForInitialLetterBox(&BfcRect::new(start_offset, end_offset), float_type)
}

// cpp: layoutng_inline/initial_letter_utils.h:18-20
// cpp: layoutng_inline/initial_letter_utils.cc:200-235
pub fn AdjustInitialLetterInTextPosition(
    _line_box_metrics: &FontHeight,
    line_box: &mut LogicalLineItems,
) -> FontHeight {
    let mut font_height = FontHeight::Empty();
    for line_item in line_box.iter_mut() {
        let shape_result = line_item.shape_result.Get();
        let inline_item = line_item.inline_item.Get();
        if shape_result.is_null()
            || inline_item.is_null()
            || unsafe { &*inline_item }.Type() != InlineItemType::kText
        {
            continue;
        }

        let mut baseline = LayoutUnit::default();
        let style = unsafe { &*line_item.Style() };
        let text_ink_bounds =
            ComputeTextInkBounds(unsafe { &*shape_result }, style, Some(&mut baseline));
        line_item.rect.offset.inline_offset -= text_ink_bounds.offset.inline_offset;
        line_item.rect.offset.block_offset = -style.GetFontHeightForDefaultBaseline().ascent;
        line_item.inline_size = text_ink_bounds.size.inline_size;

        if style.IsHorizontalTypographicMode()
            || style.GetTextOrientation() == ETextOrientation::kSideways
        {
            let line_height = text_ink_bounds.size.block_size;
            let ascent = baseline - text_ink_bounds.offset.block_offset;
            font_height.Unite(&FontHeight::new(ascent, line_height - ascent));
            continue;
        }

        let line_height = text_ink_bounds.size.block_size;
        let ascent = LayoutUnit::FromFloatFloor(line_height.ToFloat() / 2.0);
        font_height.Unite(&FontHeight::new(ascent, line_height - ascent));
    }
    font_height
}

// cpp: layoutng_inline/initial_letter_utils.h:23-24
// cpp: layoutng_inline/initial_letter_utils.cc:237-245
pub fn CalculateInitialLetterBoxInlineSize(line_info: &LineInfo) -> LayoutUnit {
    let mut inline_size = line_info.TextIndent();
    for item_result in line_info.Results().iter() {
        let shape_result = item_result.shape_result.Get();
        let item = item_result.item.Get();
        if shape_result.is_null() || unsafe { &*item }.Type() != InlineItemType::kText {
            inline_size += item_result.inline_size;
            continue;
        }
        let style = unsafe { &*unsafe { &*item }.Style() };
        let text_ink_bounds = ComputeTextInkBounds(unsafe { &*shape_result }, style, None);
        inline_size += text_ink_bounds.size.inline_size;
    }
    inline_size
}

// cpp: layoutng_inline/initial_letter_utils.h:28-35
// cpp: layoutng_inline/initial_letter_utils.cc:247-323
pub fn PostPlaceInitialLetterBox(
    line_box_metrics: &FontHeight,
    initial_letter_box_margins: &BoxStrut,
    line_box: &mut LogicalLineItems,
    line_origin: &BfcOffset,
    line_info: &mut LineInfo,
) -> *const ExclusionArea {
    let initial_letter_line_item = line_box
        .iter_mut()
        .find(|line_item| line_item.IsInitialLetterBox())
        .expect("initial-letter line item must exist");
    let initial_letter_box_fragment = unsafe {
        &*To::<PhysicalBoxFragment>(initial_letter_line_item.GetPhysicalFragment() as *const _)
    };
    debug_assert!(initial_letter_box_fragment.IsInitialLetterBox());
    debug_assert!(!initial_letter_box_fragment
        .Style()
        .InitialLetter()
        .IsNormal());

    let line_style = line_info.LineStyle();
    let writing_direction_mode = line_style.GetWritingDirection();
    let line_height = line_style.ComputedLineHeightAsFixed();
    let initial_letter_box_size =
        LogicalFragment::new(writing_direction_mode, initial_letter_box_fragment).Size();

    let mut initial_letter_block_start_adjust = LayoutUnit::default();
    let initial_letter_border_box_block_offset = ComputeInitialLetterBoxBlockOffset(
        initial_letter_box_fragment,
        initial_letter_box_size.block_size,
        unsafe { &*initial_letter_line_item.Style() },
        line_style,
        &mut initial_letter_block_start_adjust,
    ) + initial_letter_box_margins.block_start;
    debug_assert!(initial_letter_block_start_adjust >= LayoutUnit::default());
    line_info.SetInitialLetterBlockStartAdjustment(initial_letter_block_start_adjust);

    let mut adjusted_block_offset = initial_letter_border_box_block_offset;
    adjusted_block_offset -= initial_letter_block_start_adjust;
    if writing_direction_mode.IsFlippedLines() {
        adjusted_block_offset =
            line_height - adjusted_block_offset - initial_letter_box_size.block_size;
    }
    initial_letter_line_item.rect.offset.block_offset =
        adjusted_block_offset - line_box_metrics.ascent;

    let initial_letter_border_box_inline_offset =
        initial_letter_line_item.rect.offset.inline_offset;
    let initial_letter_box_origin = BfcOffset::new(
        if writing_direction_mode.IsLtr() {
            line_origin.line_offset
        } else {
            line_origin.line_offset
                + initial_letter_border_box_inline_offset
                + initial_letter_box_size.inline_size
        },
        line_origin.block_offset,
    );

    let exclusion = CreateExclusionSpaceForInitialLetterBox(
        if writing_direction_mode.IsLtr() {
            EFloat::kLeft
        } else {
            EFloat::kRight
        },
        initial_letter_box_origin,
        &BfcOffset::new(
            initial_letter_border_box_inline_offset,
            initial_letter_border_box_block_offset
                + line_info.ComputeInitialLetterBoxBlockStartAdjustment(),
        ),
        &initial_letter_box_size,
        initial_letter_box_margins,
    );
    line_info.SetInitialLetterBoxBlockSize(unsafe { &*exclusion }.rect.BlockSize());
    exclusion
}
