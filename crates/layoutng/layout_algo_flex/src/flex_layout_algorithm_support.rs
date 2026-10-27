#![allow(non_snake_case)]

use font_engine::FontBaseline;
use foundation::{
    EBoxAlignment, EBoxPack, LayoutUnit, LogicalToLogical, LogicalToPhysical, PhysicalToLogical,
    WritingDirectionMode,
};
use layoutng_assembly::internal::length_utils::{ResolveColumnGapLength, ResolveRowGapLength};
use layoutng_assembly::logical_box_fragment::LogicalBoxFragment;
use layoutng_geometry::geometry::layout_unit_diffuser::LayoutUnitDiffuser;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{
    ContentDistributionType, ContentPosition, ItemPosition, OverflowAlignment,
};
use layoutng_style::style::style_content_alignment_data::StyleContentAlignmentData;
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;

use crate::flex_line::FlexLine;

// cpp: layoutng_flex/flex_layout_algorithm.cc:53-84
pub struct PhysicalToFlex<Value> {
    logical_: PhysicalToLogical<Value>,
    is_column_: bool,
}

impl<Value: Copy> PhysicalToFlex<Value> {
    pub fn new(
        writing_direction: WritingDirectionMode,
        is_column: bool,
        top: Value,
        right: Value,
        bottom: Value,
        left: Value,
    ) -> Self {
        Self {
            logical_: PhysicalToLogical::new(writing_direction, top, right, bottom, left),
            is_column_: is_column,
        }
    }

    pub fn MainStart(&self) -> Value {
        if self.is_column_ {
            self.logical_.BlockStart()
        } else {
            self.logical_.InlineStart()
        }
    }
    pub fn MainEnd(&self) -> Value {
        if self.is_column_ {
            self.logical_.BlockEnd()
        } else {
            self.logical_.InlineEnd()
        }
    }
    pub fn CrossStart(&self) -> Value {
        if self.is_column_ {
            self.logical_.InlineStart()
        } else {
            self.logical_.BlockStart()
        }
    }
    pub fn CrossEnd(&self) -> Value {
        if self.is_column_ {
            self.logical_.InlineEnd()
        } else {
            self.logical_.BlockEnd()
        }
    }
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:86-157
pub struct BaselineAccumulator {
    font_baseline_: FontBaseline,
    first_major_baseline_: Option<LayoutUnit>,
    first_minor_baseline_: Option<LayoutUnit>,
    first_fallback_baseline_: Option<LayoutUnit>,
    last_major_baseline_: Option<LayoutUnit>,
    last_minor_baseline_: Option<LayoutUnit>,
    last_fallback_baseline_: Option<LayoutUnit>,
}

impl BaselineAccumulator {
    pub fn new(style: &ComputedStyle) -> Self {
        Self {
            font_baseline_: style.GetFontBaseline(),
            first_major_baseline_: None,
            first_minor_baseline_: None,
            first_fallback_baseline_: None,
            last_major_baseline_: None,
            last_minor_baseline_: None,
            last_fallback_baseline_: None,
        }
    }

    pub fn AccumulateItem(
        &mut self,
        fragment: &LogicalBoxFragment<'_>,
        block_offset: LayoutUnit,
        is_first_line: bool,
        is_last_line: bool,
    ) {
        if is_first_line && self.first_fallback_baseline_.is_none() {
            self.first_fallback_baseline_ =
                Some(block_offset + fragment.FirstBaselineOrSynthesize(self.font_baseline_));
        }
        if is_last_line {
            self.last_fallback_baseline_ =
                Some(block_offset + fragment.LastBaselineOrSynthesize(self.font_baseline_));
        }
    }

    pub fn AccumulateLine(&mut self, line: &FlexLine, is_first_line: bool, is_last_line: bool) {
        if is_first_line {
            if line.major_baseline != LayoutUnit::Min() {
                self.first_major_baseline_ = Some(line.cross_axis_offset + line.major_baseline);
            }
            if line.minor_baseline != LayoutUnit::Min() {
                self.first_minor_baseline_ =
                    Some(line.cross_axis_offset + line.line_cross_size - line.minor_baseline);
            }
        }
        if is_last_line {
            if line.major_baseline != LayoutUnit::Min() {
                self.last_major_baseline_ = Some(line.cross_axis_offset + line.major_baseline);
            }
            if line.minor_baseline != LayoutUnit::Min() {
                self.last_minor_baseline_ =
                    Some(line.cross_axis_offset + line.line_cross_size - line.minor_baseline);
            }
        }
    }

    pub fn FirstBaseline(&self) -> Option<LayoutUnit> {
        self.first_major_baseline_
            .or(self.first_minor_baseline_)
            .or(self.first_fallback_baseline_)
    }
    pub fn LastBaseline(&self) -> Option<LayoutUnit> {
        self.last_minor_baseline_
            .or(self.last_major_baseline_)
            .or(self.last_fallback_baseline_)
    }
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:159-163
pub fn RowGap(style: &ComputedStyle, size: LogicalSize) -> LayoutUnit {
    ResolveRowGapLength(style, size.block_size).unwrap_or_default()
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:165-169
pub fn ColumnGap(style: &ComputedStyle, size: LogicalSize) -> LayoutUnit {
    ResolveColumnGapLength(style, size.inline_size).unwrap_or_default()
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:214-263
pub fn ResolvedJustifyContent(
    style: &ComputedStyle,
    writing_direction: WritingDirectionMode,
    is_column: bool,
) -> StyleContentAlignmentData {
    if style.IsDeprecatedFlexbox() {
        let box_pack = style.BoxPack();
        let position = match box_pack {
            EBoxPack::kCenter => ContentPosition::kCenter,
            EBoxPack::kJustify | EBoxPack::kStart => ContentPosition::kFlexStart,
            EBoxPack::kEnd => ContentPosition::kFlexEnd,
        };
        let distribution = if box_pack == EBoxPack::kJustify {
            ContentDistributionType::kSpaceBetween
        } else {
            ContentDistributionType::kDefault
        };
        return StyleContentAlignmentData::new(position, distribution, OverflowAlignment::kDefault);
    }
    let justify = style.JustifyContent();
    let mut position = justify.GetPosition();
    if matches!(position, ContentPosition::kLeft | ContentPosition::kRight) {
        if is_column {
            if writing_direction.IsHorizontal() {
                position = ContentPosition::kStart;
            } else {
                let physical = LogicalToPhysical::new(
                    writing_direction,
                    ContentPosition::kStart,
                    ContentPosition::kEnd,
                    ContentPosition::kStart,
                    ContentPosition::kEnd,
                );
                position = if position == ContentPosition::kLeft {
                    physical.Left()
                } else {
                    physical.Right()
                };
            }
        } else {
            position = if (position == ContentPosition::kLeft) == writing_direction.IsLtr() {
                ContentPosition::kStart
            } else {
                ContentPosition::kEnd
            };
        }
    }
    StyleContentAlignmentData::new(position, justify.Distribution(), justify.Overflow())
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:265-337
pub fn ResolvedAlignSelf(
    style: &ComputedStyle,
    child_style: &ComputedStyle,
    writing_direction: WritingDirectionMode,
    is_column: bool,
    is_out_of_flow: bool,
) -> ItemPosition {
    let is_horizontal_flow = if style.IsHorizontalWritingMode() {
        !is_column
    } else {
        is_column
    };
    if !is_out_of_flow {
        if is_horizontal_flow {
            if child_style.MarginTop().IsAuto() || child_style.MarginBottom().IsAuto() {
                return ItemPosition::kFlexStart;
            }
        } else if child_style.MarginLeft().IsAuto() || child_style.MarginRight().IsAuto() {
            return ItemPosition::kFlexStart;
        }
    }
    if style.IsDeprecatedFlexbox() {
        return match style.BoxAlign() {
            EBoxAlignment::kBaseline => ItemPosition::kBaseline,
            EBoxAlignment::kCenter => ItemPosition::kCenter,
            EBoxAlignment::kStretch => ItemPosition::kStretch,
            EBoxAlignment::kStart => ItemPosition::kFlexStart,
            EBoxAlignment::kEnd => ItemPosition::kFlexEnd,
        };
    }
    let mut align = child_style
        .ResolvedAlignSelf(
            &StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kStretch,
                OverflowAlignment::kDefault,
            ),
            style as *const ComputedStyle,
        )
        .GetPosition();
    assert!(!matches!(
        align,
        ItemPosition::kAuto | ItemPosition::kNormal | ItemPosition::kLeft | ItemPosition::kRight
    ));
    if align == ItemPosition::kStart {
        return ItemPosition::kFlexStart;
    }
    if align == ItemPosition::kEnd {
        return ItemPosition::kFlexEnd;
    }
    let logical = LogicalToLogical::new(
        child_style.GetWritingDirection(),
        writing_direction,
        ItemPosition::kFlexStart,
        ItemPosition::kFlexEnd,
        ItemPosition::kFlexStart,
        ItemPosition::kFlexEnd,
    );
    if align == ItemPosition::kSelfStart {
        return if is_column {
            logical.InlineStart()
        } else {
            logical.BlockStart()
        };
    }
    if align == ItemPosition::kSelfEnd {
        return if is_column {
            logical.InlineEnd()
        } else {
            logical.BlockEnd()
        };
    }
    if style.ResolvedIsFlexWrapReverse() {
        if align == ItemPosition::kFlexStart {
            align = ItemPosition::kFlexEnd;
        } else if align == ItemPosition::kFlexEnd {
            align = ItemPosition::kFlexStart;
        }
    }
    align
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:447-449
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisEdge {
    kStart,
    kCenter,
    kEnd,
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:451-475
pub fn MainAxisStaticPositionEdge(
    justify_content: &StyleContentAlignmentData,
    is_reverse_direction: bool,
) -> AxisEdge {
    let position = justify_content.GetPosition();
    assert!(!matches!(
        position,
        ContentPosition::kLeft | ContentPosition::kRight
    ));
    if position == ContentPosition::kFlexEnd {
        return if is_reverse_direction {
            AxisEdge::kStart
        } else {
            AxisEdge::kEnd
        };
    }
    if position == ContentPosition::kCenter
        || matches!(
            justify_content.Distribution(),
            ContentDistributionType::kSpaceAround | ContentDistributionType::kSpaceEvenly
        )
    {
        return AxisEdge::kCenter;
    }
    if position == ContentPosition::kStart {
        return AxisEdge::kStart;
    }
    if position == ContentPosition::kEnd {
        return AxisEdge::kEnd;
    }
    if is_reverse_direction {
        AxisEdge::kEnd
    } else {
        AxisEdge::kStart
    }
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:477-492
pub fn CrossAxisStaticPositionEdge(alignment: ItemPosition, is_wrap_reverse: bool) -> AxisEdge {
    if is_wrap_reverse && alignment == ItemPosition::kStretch {
        return AxisEdge::kEnd;
    }
    if matches!(
        alignment,
        ItemPosition::kFlexEnd | ItemPosition::kLastBaseline
    ) {
        return AxisEdge::kEnd;
    }
    if alignment == ItemPosition::kCenter {
        return AxisEdge::kCenter;
    }
    AxisEdge::kStart
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:1646-1698
pub fn InitialContentPositionOffset(
    data: &StyleContentAlignmentData,
    free_space: LayoutUnit,
    number_of_items: u32,
    is_reverse: bool,
) -> LayoutUnit {
    match data.Distribution() {
        ContentDistributionType::kDefault => {}
        ContentDistributionType::kSpaceBetween => {
            if free_space > LayoutUnit::default() && number_of_items > 1 {
                return LayoutUnit::default();
            }
            return if is_reverse {
                free_space
            } else {
                LayoutUnit::default()
            };
        }
        ContentDistributionType::kSpaceAround => {
            if free_space > LayoutUnit::default() && number_of_items > 0 {
                return free_space / (2 * number_of_items);
            }
            return (free_space / 2u32).ClampNegativeToZero();
        }
        ContentDistributionType::kSpaceEvenly => {
            if free_space > LayoutUnit::default() && number_of_items > 0 {
                return free_space / (number_of_items + 1);
            }
            return (free_space / 2u32).ClampNegativeToZero();
        }
        ContentDistributionType::kStretch => {
            return if is_reverse {
                free_space
            } else {
                LayoutUnit::default()
            };
        }
    }
    if free_space <= LayoutUnit::default() && data.Overflow() == OverflowAlignment::kSafe {
        return LayoutUnit::default();
    }
    match data.GetPosition() {
        ContentPosition::kCenter => free_space / 2,
        ContentPosition::kStart => LayoutUnit::default(),
        ContentPosition::kEnd => free_space,
        ContentPosition::kFlexEnd => {
            if is_reverse {
                LayoutUnit::default()
            } else {
                free_space
            }
        }
        ContentPosition::kFlexStart
        | ContentPosition::kNormal
        | ContentPosition::kBaseline
        | ContentPosition::kLastBaseline => {
            if is_reverse {
                free_space
            } else {
                LayoutUnit::default()
            }
        }
        ContentPosition::kLeft | ContentPosition::kRight => {
            unreachable!("left/right must be resolved")
        }
    }
}

// cpp: layoutng_flex/flex_layout_algorithm.cc:1700-1720
pub fn ContentDistributionSpace(
    data: &StyleContentAlignmentData,
    free_space: LayoutUnit,
    number_of_items: u32,
) -> LayoutUnitDiffuser {
    if free_space <= LayoutUnit::default() || number_of_items <= 1 {
        return LayoutUnitDiffuser::empty();
    }
    match data.Distribution() {
        ContentDistributionType::kDefault | ContentDistributionType::kStretch => {
            LayoutUnitDiffuser::empty()
        }
        ContentDistributionType::kSpaceBetween => {
            LayoutUnitDiffuser::new(free_space, number_of_items - 1)
        }
        ContentDistributionType::kSpaceEvenly => {
            LayoutUnitDiffuser::new(free_space, number_of_items + 1)
        }
        ContentDistributionType::kSpaceAround => {
            LayoutUnitDiffuser::new(free_space, number_of_items)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{TextDirection, WritingMode};

    #[test]
    fn physical_sides_resolve_in_flex_axes_for_ltr_and_rtl() {
        let ltr = WritingDirectionMode::new(WritingMode::kHorizontalTb, TextDirection::kLtr);
        let rtl = WritingDirectionMode::new(WritingMode::kHorizontalTb, TextDirection::kRtl);
        let row_ltr = PhysicalToFlex::new(ltr, false, 1, 2, 3, 4);
        assert_eq!(
            (
                row_ltr.MainStart(),
                row_ltr.MainEnd(),
                row_ltr.CrossStart(),
                row_ltr.CrossEnd()
            ),
            (4, 2, 1, 3)
        );
        let row_rtl = PhysicalToFlex::new(rtl, false, 1, 2, 3, 4);
        assert_eq!((row_rtl.MainStart(), row_rtl.MainEnd()), (2, 4));
        let column_ltr = PhysicalToFlex::new(ltr, true, 1, 2, 3, 4);
        assert_eq!(
            (
                column_ltr.MainStart(),
                column_ltr.MainEnd(),
                column_ltr.CrossStart(),
                column_ltr.CrossEnd()
            ),
            (1, 3, 4, 2)
        );
    }

    #[test]
    fn static_position_edges_respect_reverse_and_space_around() {
        let start = StyleContentAlignmentData::new(
            ContentPosition::kFlexStart,
            ContentDistributionType::kDefault,
            OverflowAlignment::kDefault,
        );
        assert_eq!(MainAxisStaticPositionEdge(&start, false), AxisEdge::kStart);
        assert_eq!(MainAxisStaticPositionEdge(&start, true), AxisEdge::kEnd);
        let around = StyleContentAlignmentData::new(
            ContentPosition::kNormal,
            ContentDistributionType::kSpaceAround,
            OverflowAlignment::kDefault,
        );
        assert_eq!(
            MainAxisStaticPositionEdge(&around, false),
            AxisEdge::kCenter
        );
        assert_eq!(
            CrossAxisStaticPositionEdge(ItemPosition::kStretch, true),
            AxisEdge::kEnd
        );
    }

    #[test]
    fn flex_content_spacing_falls_back_to_safe_center_on_overflow() {
        let align = StyleContentAlignmentData::new(
            ContentPosition::kCenter,
            ContentDistributionType::kSpaceAround,
            OverflowAlignment::kSafe,
        );
        assert_eq!(
            InitialContentPositionOffset(&align, LayoutUnit::from_signed(-20), 2, false),
            LayoutUnit::default()
        );
        let spread = StyleContentAlignmentData::new(
            ContentPosition::kFlexStart,
            ContentDistributionType::kSpaceBetween,
            OverflowAlignment::kDefault,
        );
        let mut diffuser = ContentDistributionSpace(&spread, LayoutUnit::from_signed(11), 3);
        assert_eq!(
            diffuser.Next() + diffuser.Next(),
            LayoutUnit::from_signed(11)
        );
    }
}
