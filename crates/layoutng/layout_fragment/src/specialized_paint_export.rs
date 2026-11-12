//! Specialized source fragment paint payload projection.
//!
//! Table, MathML, frameset, fieldset, column-rule, gap-rule and overflow-clip
//! payloads are grouped by their fragment-output lifecycle rather than
//! retained in the generic source boundary.

#![allow(non_snake_case)]

use super::*;
use crate::internal::fragmentation_utils::GetFirstFragmentBreakTokenData;
use crate::internal::gap::gap_decoration_utils::CSSGapDecorationUtils;
use crate::internal::gap::gap_geometry::{BlockedStatus, GapGeometry};
use crate::internal::gap::gap_intersection::GapIntersection;
use foundation::Vector;
use layoutng_style::style::gap_data_list::GapDataListValueAccessor;

fn ComputeTableColumnsRect(fragment: &PhysicalBoxFragment) -> PhysicalRect {
    let writing_direction = fragment.Style().GetWritingDirection();
    let mut columns_rect = LogicalRect::default();
    let mut stitched_block_size = LayoutUnit::default();
    let mut fragment_block_offset = LayoutUnit::default();
    let mut has_section = false;
    let layout_box = To::<LayoutBox>(fragment.GetLayoutObject());
    for table_fragment in unsafe { &*layout_box }.PhysicalFragments() {
        if std::ptr::eq(table_fragment, fragment) {
            fragment_block_offset = stitched_block_size;
        }
        let converter = WritingModeConverter::new(writing_direction, table_fragment.Size());
        for child in table_fragment.Children() {
            if !child.IsTableSection() {
                continue;
            }
            let mut section_rect =
                converter.ToLogicalRect(PhysicalRect::new(child.Offset(), child.Size()));
            section_rect.offset.block_offset += stitched_block_size;
            if !has_section {
                columns_rect = section_rect;
                has_section = true;
            } else {
                columns_rect.UniteEvenIfEmpty(&section_rect);
            }
        }
        stitched_block_size += LogicalFragment::new(writing_direction, table_fragment).BlockSize();
    }
    if !has_section {
        return PhysicalRect::default();
    }
    columns_rect.offset.block_offset -= fragment_block_offset;
    WritingModeConverter::new(writing_direction, fragment.Size()).ToPhysicalRect(columns_rect)
}

#[inline(never)]
pub(super) fn ExportTablePaintData(
    box_fragment: &PhysicalBoxFragment,
    sources: &SourceMap,
    paint: &mut PaintProperties,
) {
    if !box_fragment.IsTable() {
        return;
    }
    let mut data = TablePaintData::default();
    let converter = WritingModeConverter::new(
        box_fragment.Style().GetWritingDirection(),
        box_fragment.Size(),
    );
    let grid = converter.ToPhysicalRect(box_fragment.TableGridRect());
    data.grid_offset = Offset {
        x: Number(grid.offset.left),
        y: Number(grid.offset.top),
    };
    data.grid_size = Size {
        width: Number(grid.size.width),
        height: Number(grid.size.height),
    };
    let columns = ComputeTableColumnsRect(box_fragment);
    data.columns_offset = Offset {
        x: Number(columns.offset.left),
        y: Number(columns.offset.top),
    };
    data.columns_size = Size {
        width: Number(columns.size.width),
        height: Number(columns.size.height),
    };
    let geometries = box_fragment.TableColumnGeometries();
    if !geometries.is_null() {
        let geometries = unsafe { &*geometries };
        data.columns.reserve(geometries.len());
        for geometry in geometries.iter() {
            let column_box = geometry.node.GetLayoutBox();
            if column_box.is_null() {
                continue;
            }
            let Some(source) = sources.get(&(column_box as *const LayoutObject)) else {
                continue;
            };
            data.columns.push(TablePaintColumn {
                start_column: geometry.start_column as usize,
                span: geometry.span as usize,
                inline_offset: Number(geometry.inline_offset),
                inline_size: Number(geometry.inline_size),
                node_id: unsafe { &*source.node }.InputId(),
                style: crate::internal::boundary::paint_style_export::ExportStyleWithInputResources(
                    unsafe { &*column_box }.StyleRef(),
                    unsafe { &*source.node }.InputStyleResources(),
                ),
            });
        }
    }
    paint.table = Some(Arc::new(data));
}

pub(super) fn ExportMathMLPaintData(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    if let Some(baseline) = box_fragment.FirstBaseline() {
        paint.first_baseline = Some(Number(baseline));
    }
    if !box_fragment.HasExtraMathMLPainting() {
        return;
    }

    let mut data = MathMLPaintData::default();
    let algorithms = LayoutPassScope::Algorithms();
    let math = if algorithms.is_null() {
        None
    } else {
        Some(&unsafe { &*algorithms }.mathml_support)
    };
    if box_fragment.IsMathMLFraction() {
        let Some((axis_height, fraction_rule_thickness)) =
            math.and_then(|support| Some((support.axis_height?, support.fraction_rule_thickness?)))
        else {
            std::panic::panic_any(UnsupportedLayout::new(
                "MathML fraction paint module is not installed",
            ));
        };
        data.kind = MathMLPaintKind::kFraction;
        data.axis_height = Number(axis_height(box_fragment.Style()));
        data.rule_thickness = Number(fraction_rule_thickness(box_fragment.Style()));
        paint.mathml = Some(Arc::new(data));
        return;
    }

    let source = box_fragment.GetMathMLPaintInfo();
    data.kind = if source.IsRadicalOperator() {
        MathMLPaintKind::kRadical
    } else {
        MathMLPaintKind::kOperator
    };
    data.operator_inline_size = Number(source.operator_inline_size);
    data.operator_ascent = Number(source.operator_ascent);
    data.operator_descent = Number(source.operator_descent);
    data.radical_margin_inline_start = Number(source.radical_base_margins.inline_start);
    data.radical_margin_inline_end = Number(source.radical_base_margins.inline_end);
    data.radical_operator_inline_offset = source.radical_operator_inline_offset.map_or(0.0, Number);
    if data.kind == MathMLPaintKind::kRadical {
        let Some(radical_metrics) = math.and_then(|support| support.radical_metrics) else {
            std::panic::panic_any(UnsupportedLayout::new(
                "MathML radical paint module is not installed",
            ));
        };
        let has_index = paint.source_kind == NodeKind::kMathRoot;
        let mut vertical_gap = LayoutUnit::default();
        let mut rule_thickness = LayoutUnit::default();
        radical_metrics(
            box_fragment.Style(),
            has_index,
            &mut vertical_gap,
            &mut rule_thickness,
        );
        data.vertical_gap = Number(vertical_gap);
        data.rule_thickness = Number(rule_thickness);
    }
    let font = box_fragment.Style().GetFont();
    let primary_font = unsafe { &*font }.PrimaryFont();
    let baseline = if primary_font.is_null() {
        0.0
    } else {
        Number(LayoutUnit::from_f32(
            unsafe { &*primary_font }.GetFontMetrics().FloatAscent(),
        ))
    };
    ExportShapeResult(
        source.operator_shape_result_view.Get(),
        box_fragment.Style().ComputedFontSize() as f64,
        baseline,
        paint.writing_mode,
        &mut data.operator_glyph_runs,
    );
    paint.mathml = Some(Arc::new(data));
}

pub(super) fn ExportFrameSetPaintData(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    if paint.source_kind != NodeKind::kFrameSet {
        return;
    }
    let source = box_fragment.GetFrameSetLayoutData();
    if source.is_null() {
        return;
    }
    let source = unsafe { &*source };
    let mut data = FrameSetPaintData::default();
    data.column_sizes.reserve(source.col_sizes.len());
    for size in &source.col_sizes {
        data.column_sizes.push(Number(*size));
    }
    data.row_sizes.reserve(source.row_sizes.len());
    for size in &source.row_sizes {
        data.row_sizes.push(Number(*size));
    }
    data.column_allows_border
        .extend(source.col_allow_border.iter().copied());
    data.row_allows_border
        .extend(source.row_allow_border.iter().copied());
    data.border_thickness = source.border_thickness as f64;
    data.has_border_color = source.has_border_color;
    paint.frame_set = Some(Arc::new(data));
}

pub(super) fn ExportFieldsetPaintData(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    if paint.source_kind != NodeKind::kFieldset {
        return;
    }
    let mut data = FieldsetPaintData::default();
    let mut legend_border_box = PhysicalRect::default();
    if let Some(legend) = box_fragment.Children().first() {
        if legend.IsRenderedLegend() {
            data.has_legend = true;
            legend_border_box.size = legend.Size();
            let writing_direction = box_fragment.Style().GetWritingDirection();
            let borders = box_fragment.Borders();
            let padding = box_fragment.Padding();
            let physical_content_size = box_fragment.Size()
                - PhysicalSize::new(
                    borders.HorizontalSum() + padding.HorizontalSum(),
                    borders.VerticalSum() + padding.VerticalSum(),
                );
            let logical_content_size =
                ToLogicalSize(physical_content_size, writing_direction.GetWritingMode());
            let relative_offset =
                ComputeRelativeOffset(legend.Style(), writing_direction, &logical_content_size);
            let logical_offset = WritingModeConverter::new(writing_direction, box_fragment.Size())
                .ToLogicalOffset(legend.Offset(), legend.Size())
                - relative_offset;
            legend_border_box.offset =
                layoutng_geometry::geometry::logical_offset::LogicalOffset::from(logical_offset)
                    .ConvertToPhysical(
                        writing_direction,
                        box_fragment.Size(),
                        legend_border_box.size,
                    );
        }
    }

    let borders = box_fragment.Borders();
    let mut cutout;
    let mut outsets = layoutng_geometry::geometry::box_strut::PhysicalBoxStrut::default();
    if box_fragment.Style().IsHorizontalWritingMode() {
        let legend_size = legend_border_box.size.height;
        let border_size = borders.top;
        let excess = legend_size - border_size;
        if excess > LayoutUnit::default() {
            outsets.top = excess / 2;
        }
        cutout = PhysicalRect::from_units(
            legend_border_box.X(),
            LayoutUnit::default(),
            legend_border_box.Width(),
            legend_size.max(border_size),
        );
    } else {
        let legend_size = legend_border_box.Width();
        let flipped = box_fragment.Style().IsFlippedBlocksWritingMode();
        let border_size = if flipped { borders.right } else { borders.left };
        let excess = legend_size - border_size;
        if excess > LayoutUnit::default() {
            if flipped {
                outsets.right = excess / 2;
            } else {
                outsets.left = excess / 2;
            }
        }
        let total = legend_size.max(border_size);
        cutout = PhysicalRect::from_units(
            LayoutUnit::default(),
            legend_border_box.offset.top,
            total,
            legend_border_box.size.height,
        );
        if flipped {
            cutout.offset.left += box_fragment.Size().width - total;
        }
    }
    data.border_outsets = Edges {
        top: Number(outsets.top),
        right: Number(outsets.right),
        bottom: Number(outsets.bottom),
        left: Number(outsets.left),
    };
    data.legend_cutout_offset = Offset {
        x: Number(cutout.offset.left),
        y: Number(cutout.offset.top),
    };
    data.legend_cutout_size = Size {
        width: Number(cutout.size.width),
        height: Number(cutout.size.height),
    };
    paint.fieldset = Some(Arc::new(data));
}

#[inline(never)]
pub(super) fn ExportColumnRulePaintData(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    let geometry = box_fragment.GetGapGeometry();
    if geometry.is_null() || unsafe { &*geometry }.GetContainerType() != ContainerType::kMultiColumn
    {
        return;
    }
    let geometry = unsafe { &*geometry };
    // The modern gap-decoration painter owns multicolumn rules whenever the
    // style generated gap geometry.  Keep this legacy straight-line payload as
    // a fallback only for fragments that do not use that pipeline.
    if box_fragment.Style().HasGapRule() {
        return;
    }
    let mut data = ColumnRulePaintData::default();
    let converter = WritingModeConverter::new(
        box_fragment.Style().GetWritingDirection(),
        box_fragment.Size(),
    );
    let block_start = geometry.GetContentBlockStart();
    let block_size = geometry.GetContentBlockEnd() - block_start;
    let columns_are_main = geometry.IsMainDirection(GridTrackSizingDirection::kForColumns);
    let gap_count = if columns_are_main {
        geometry.MainGapCount()
    } else {
        geometry.CrossGapCount()
    };
    let style = box_fragment.Style();
    let iterator_gap_count = u32::try_from(gap_count).expect("gap count exceeds WTF size");
    let mut width_iterator =
        GapDataListIterator::new(style.ColumnRuleWidth().GetGapDataList(), iterator_gap_count);
    let mut style_iterator =
        GapDataListIterator::new(style.ColumnRuleStyle().GetGapDataList(), iterator_gap_count);
    let mut color_iterator =
        GapDataListIterator::new(style.ColumnRuleColor().GetGapDataList(), iterator_gap_count);
    for index in 0..gap_count {
        if geometry.IsMultiColSpanner(index, GridTrackSizingDirection::kForColumns) {
            continue;
        }
        let width = f64::from(width_iterator.Next());
        let line_style = crate::internal::boundary::paint_style_export::BorderStyle(
            StyleComputedStyle::CollapsedBorderStyle(style_iterator.Next()),
        );
        let color = crate::internal::boundary::paint_style_export::ColorValue(
            style.VisitedDependentGapColor(&color_iterator.Next(), true),
        );
        if width <= 0.0 || line_style == BorderLineStyle::kNone || color.alpha <= 0.0 {
            continue;
        }
        let thickness = LayoutUnit::from_f64(width);
        let center = geometry.GetGapCenterOffset(GridTrackSizingDirection::kForColumns, index);
        let logical =
            LogicalRect::from_units(center - thickness / 2, block_start, thickness, block_size);
        let physical = converter.ToPhysicalRect(logical);
        data.rules.push(ColumnRule {
            offset: Offset {
                x: Number(physical.offset.left),
                y: Number(physical.offset.top),
            },
            size: Size {
                width: Number(physical.size.width),
                height: Number(physical.size.height),
            },
            color,
            line_style,
            stroke_width: width,
        });
    }
    if !data.rules.is_empty() {
        paint.column_rules = Some(Arc::new(data));
    }
}

fn ShouldMoveGapRuleStartForward(
    start: u32,
    rule_break: foundation::RuleBreak,
    rule_visibility: foundation::RuleVisibilityItems,
    geometry: &GapGeometry,
    intersections: &Vector<GapIntersection>,
) -> bool {
    let visible = CSSGapDecorationUtils::IsRuleSegmentVisible(
        *intersections[start as usize].SegmentState(),
        rule_visibility,
    );
    if geometry.GetContainerType() == ContainerType::kFlex {
        if intersections[start as usize].IsOverlapWindowOpen() {
            return true;
        }
        if intersections[start as usize].IsOverlapWindowClose() {
            return false;
        }
    }
    if rule_break == foundation::RuleBreak::kNone {
        return !visible;
    }
    let blocked = GapGeometry::BlockedStatusFromGapStates(intersections, start);
    blocked.HasBlockedStatus(BlockedStatus::kBlockedAfter) || !visible
}

fn ShouldMoveGapRuleEndForward(
    direction: GridTrackSizingDirection,
    gap_index: u32,
    end: u32,
    rule_break: foundation::RuleBreak,
    rule_visibility: foundation::RuleVisibilityItems,
    geometry: &GapGeometry,
    intersections: &Vector<GapIntersection>,
) -> bool {
    if !CSSGapDecorationUtils::IsRuleSegmentVisible(
        *intersections[end as usize].SegmentState(),
        rule_visibility,
    ) {
        return false;
    }
    if rule_break == foundation::RuleBreak::kNone {
        return true;
    }
    let blocked = GapGeometry::BlockedStatusFromGapStates(intersections, end);
    if rule_break == foundation::RuleBreak::kNormal {
        return !blocked.HasBlockedStatus(BlockedStatus::kBlockedAfter);
    }
    debug_assert_eq!(rule_break, foundation::RuleBreak::kIntersection);
    if geometry.GetContainerType() == ContainerType::kFlex {
        if intersections[end as usize].IsOverlapWindowClose() {
            return true;
        }
        if intersections[end as usize].IsOverlapWindowOpen() {
            return false;
        }
        return false;
    }
    if blocked.HasBlockedStatus(BlockedStatus::kBlockedAfter) {
        return false;
    }
    if geometry.GetContainerType() != ContainerType::kGrid {
        return false;
    }
    let cross_direction = if direction == GridTrackSizingDirection::kForColumns {
        GridTrackSizingDirection::kForRows
    } else {
        GridTrackSizingDirection::kForColumns
    };
    let cross_blocked = geometry.GetIntersectionBlockedStatus(
        cross_direction,
        end - 1,
        gap_index + 1,
        intersections,
    );
    cross_blocked.HasBlockedStatus(BlockedStatus::kBlockedAfter)
        && cross_blocked.HasBlockedStatus(BlockedStatus::kBlockedBefore)
}

fn AdjustGapRuleIntersectionPair(
    direction: GridTrackSizingDirection,
    start: &mut u32,
    end: &mut u32,
    gap_index: u32,
    rule_break: foundation::RuleBreak,
    rule_visibility: foundation::RuleVisibilityItems,
    geometry: &GapGeometry,
    intersections: &Vector<GapIntersection>,
) {
    let last = intersections.len() as u32 - 1;
    while *start < last
        && ShouldMoveGapRuleStartForward(
            *start,
            rule_break,
            rule_visibility,
            geometry,
            intersections,
        )
    {
        *start += 1;
    }
    if *start == last {
        return;
    }
    *end = *start + 1;
    while *end < last
        && ShouldMoveGapRuleEndForward(
            direction,
            gap_index,
            *end,
            rule_break,
            rule_visibility,
            geometry,
            intersections,
        )
    {
        *end += 1;
    }
}

fn AdvanceCrossGapOwnerCursor(geometry: &GapGeometry, gap_index: u32, owner: &mut u32) {
    while (*owner as usize) < geometry.GetMainGaps().len() {
        let main_gap = &geometry.GetMainGaps()[*owner as usize];
        if main_gap.HasCrossGapsBefore() && gap_index <= main_gap.GetCrossGapBeforeEnd() {
            return;
        }
        *owner += 1;
    }
}

fn ExpandedGapRuleWidths(
    values: &layoutng_style::style::gap_data_list::GapDataList<i32>,
    count: u32,
) -> Vector<i32> {
    let mut iterator = GapDataListIterator::new(values.GetGapDataList(), count);
    let mut result = Vector::with_capacity(count as usize);
    while iterator.HasNext() {
        result.push(iterator.Next());
    }
    result
}

fn ExportGapRuleDirection(
    box_fragment: &PhysicalBoxFragment,
    geometry: &GapGeometry,
    direction: GridTrackSizingDirection,
    data: &mut GapRulePaintData,
) -> bool {
    let is_main = geometry.IsMainDirection(direction);
    let style = box_fragment.Style();
    let is_column_gap = direction == GridTrackSizingDirection::kForColumns;
    let gap_count = if is_main {
        geometry.MainGapCount()
    } else {
        geometry.CrossGapCount()
    };
    if gap_count == 0 {
        return true;
    }
    let has_row_gap_fragmentation = geometry.HasRowGapFragmentation(box_fragment, is_main);
    let total_gap_count = if has_row_gap_fragmentation {
        let first_fragment_data = GetFirstFragmentBreakTokenData(box_fragment);
        assert!(!first_fragment_data.is_null());
        unsafe { &*first_fragment_data }.GetTotalRowGapCount()
    } else {
        gap_count
    };

    let rule_break =
        CSSGapDecorationUtils::ResolveRuleBreakValue(style, direction, geometry.GetContainerType());
    let rule_visibility = CSSGapDecorationUtils::ResolveRuleVisibilityItemsValue(
        style,
        geometry.GetContainerType(),
        direction,
    );
    let cross_direction = if is_column_gap {
        GridTrackSizingDirection::kForRows
    } else {
        GridTrackSizingDirection::kForColumns
    };
    let cross_rule_visibility = CSSGapDecorationUtils::ResolveRuleVisibilityItemsValue(
        style,
        geometry.GetContainerType(),
        cross_direction,
    );
    let (rule_widths, rule_styles, rule_colors) = if is_column_gap {
        (
            style.ColumnRuleWidth(),
            style.ColumnRuleStyle(),
            style.ColumnRuleColor(),
        )
    } else {
        (
            style.RowRuleWidth(),
            style.RowRuleStyle(),
            style.RowRuleColor(),
        )
    };
    let has_overlap_join = CSSGapDecorationUtils::HasOverlapJoin(style, is_column_gap);
    let cross_widths = if has_overlap_join {
        let count = if is_main {
            geometry.CrossGapCount()
        } else {
            geometry.MainGapCount()
        };
        if is_column_gap {
            ExpandedGapRuleWidths(style.RowRuleWidth(), count)
        } else {
            ExpandedGapRuleWidths(style.ColumnRuleWidth(), count)
        }
    } else {
        Vector::default()
    };

    let all_single = rule_widths.HasSingleValue()
        && rule_styles.HasSingleValue()
        && rule_colors.HasSingleValue();
    let reversed =
        gap_count > 1 && !all_single && geometry.HasNonIdentityDecorationOrder(direction);
    let width_accessor =
        reversed.then(|| GapDataListValueAccessor::new(rule_widths.GetGapDataList(), gap_count));
    let style_accessor =
        reversed.then(|| GapDataListValueAccessor::new(rule_styles.GetGapDataList(), gap_count));
    let color_accessor =
        reversed.then(|| GapDataListValueAccessor::new(rule_colors.GetGapDataList(), gap_count));
    let mut width_iterator = (!reversed)
        .then(|| GapDataListIterator::new(rule_widths.GetGapDataList(), total_gap_count));
    let mut style_iterator = (!reversed)
        .then(|| GapDataListIterator::new(rule_styles.GetGapDataList(), total_gap_count));
    let mut color_iterator = (!reversed)
        .then(|| GapDataListIterator::new(rule_colors.GetGapDataList(), total_gap_count));

    let converter = WritingModeConverter::new(style.GetWritingDirection(), box_fragment.Size());
    let physical_sides = foundation::PhysicalToLogical::new(
        style.GetWritingDirection(),
        GapRuleSide::Top,
        GapRuleSide::Right,
        GapRuleSide::Bottom,
        GapRuleSide::Left,
    );
    let rule_side = if is_column_gap {
        physical_sides.InlineStart()
    } else {
        physical_sides.BlockStart()
    };
    let mut intersections = Vector::<GapIntersection>::default();
    geometry.InitPaintState();
    let mut owner = (!is_main
        && matches!(
            geometry.GetContainerType(),
            ContainerType::kFlex | ContainerType::kGridLanes
        ))
    .then_some(0u32);

    for gap_index in 0..gap_count {
        if geometry.IsMultiColSpanner(gap_index, direction) {
            continue;
        }
        if let Some(owner) = owner.as_mut() {
            AdvanceCrossGapOwnerCursor(geometry, gap_index, owner);
        }
        let geometric_gap_index = if has_row_gap_fragmentation {
            assert_ne!(geometry.GetContainerType(), ContainerType::kGridLanes);
            let object = box_fragment.GetLayoutObject();
            assert!(!object.is_null());
            u32::try_from(unsafe { &*object }.StitchedRowGapIndex(
                box_fragment,
                gap_index as usize,
                owner.map(|index| index as usize),
            ))
            .expect("stitched row gap index exceeds wtf_size_t")
        } else {
            gap_index
        };
        let decoration_index = if reversed {
            geometry.DecorationIndexForGap(direction, gap_index, owner, gap_count)
        } else {
            gap_index
        };
        let width_value = if let Some(accessor) = &width_accessor {
            accessor.ValueAt(decoration_index)
        } else {
            if has_row_gap_fragmentation {
                width_iterator
                    .as_mut()
                    .unwrap()
                    .AdvanceUpTo(geometric_gap_index);
            }
            width_iterator.as_mut().unwrap().Next()
        };
        let style_value = if let Some(accessor) = &style_accessor {
            accessor.ValueAt(decoration_index)
        } else {
            if has_row_gap_fragmentation {
                style_iterator
                    .as_mut()
                    .unwrap()
                    .AdvanceUpTo(geometric_gap_index);
            }
            style_iterator.as_mut().unwrap().Next()
        };
        let color_value = if let Some(accessor) = &color_accessor {
            accessor.ValueAt(decoration_index)
        } else {
            if has_row_gap_fragmentation {
                color_iterator
                    .as_mut()
                    .unwrap()
                    .AdvanceUpTo(geometric_gap_index);
            }
            color_iterator.as_mut().unwrap().Next()
        };
        let line_style = crate::internal::boundary::paint_style_export::BorderStyle(
            StyleComputedStyle::CollapsedBorderStyle(style_value),
        );
        let color = crate::internal::boundary::paint_style_export::ColorValue(
            style.VisitedDependentGapColor(&color_value, is_column_gap),
        );
        if width_value <= 0
            || matches!(
                line_style,
                BorderLineStyle::kNone | BorderLineStyle::kHidden
            )
            || color.alpha <= 0.0
        {
            continue;
        }

        let thickness = LayoutUnit::from_signed(width_value);
        let center = geometry.GetGapCenterOffset(direction, gap_index);
        geometry.GenerateIntersectionListForGap(direction, gap_index, &mut intersections, owner);
        assert!(intersections.len() >= 2);
        let last = intersections.len() as u32 - 1;
        let mut start = 0u32;
        while start < last {
            let mut end = start;
            AdjustGapRuleIntersectionPair(
                direction,
                &mut start,
                &mut end,
                gap_index,
                rule_break,
                rule_visibility,
                geometry,
                &intersections,
            );
            if start >= end {
                break;
            }
            let start_is_cap = geometry.IsCapIntersection(
                cross_direction,
                gap_index,
                start,
                is_main,
                rule_visibility,
                cross_rule_visibility,
                &intersections,
            );
            let end_is_cap = geometry.IsCapIntersection(
                cross_direction,
                gap_index,
                end,
                is_main,
                rule_visibility,
                cross_rule_visibility,
                &intersections,
            );
            let start_max_inset =
                geometry.GetMaxInsetWidth(direction, gap_index, start, is_main, &intersections);
            let end_max_inset =
                geometry.GetMaxInsetWidth(direction, gap_index, end, is_main, &intersections);
            let start_cross_decoration = if has_overlap_join {
                geometry.GetCrossDecorationWidthForIntersection(
                    gap_index,
                    start,
                    is_main,
                    &intersections,
                    start_is_cap,
                    &cross_widths,
                )
            } else {
                LayoutUnit::default()
            };
            let end_cross_decoration = if has_overlap_join {
                geometry.GetCrossDecorationWidthForIntersection(
                    gap_index,
                    end,
                    is_main,
                    &intersections,
                    end_is_cap,
                    &cross_widths,
                )
            } else {
                LayoutUnit::default()
            };
            let start_inset = geometry.ComputeInsetStart(
                style,
                gap_index,
                start,
                &intersections,
                start_is_cap,
                is_column_gap,
                is_main,
                has_overlap_join && !start_is_cap,
                start_max_inset,
                start_cross_decoration,
            );
            let end_inset = geometry.ComputeInsetEnd(
                style,
                gap_index,
                end,
                &intersections,
                end_is_cap,
                is_column_gap,
                is_main,
                has_overlap_join && !end_is_cap,
                end_max_inset,
                end_cross_decoration,
            );
            let start_cross_width = geometry.GetCrossWidthForIntersection(
                direction,
                gap_index,
                start,
                is_main,
                &intersections,
            );
            let end_cross_width = geometry.GetCrossWidthForIntersection(
                direction,
                gap_index,
                end,
                is_main,
                &intersections,
            );
            let secondary_start =
                intersections[start as usize].GetOffset() + start_cross_width / 2 + start_inset;
            let secondary_size = intersections[end as usize].GetOffset()
                - secondary_start
                - end_cross_width / 2
                - end_inset;
            let logical = if is_column_gap {
                LogicalRect::from_units(
                    center - thickness / 2,
                    secondary_start,
                    thickness,
                    secondary_size,
                )
            } else {
                LogicalRect::from_units(
                    secondary_start,
                    center - thickness / 2,
                    secondary_size,
                    thickness,
                )
            };
            let physical = converter.ToPhysicalRect(logical);
            data.rules.push(GapRule {
                offset: Offset {
                    x: Number(physical.offset.left),
                    y: Number(physical.offset.top),
                },
                size: Size {
                    width: Number(physical.size.width),
                    height: Number(physical.size.height),
                },
                color,
                line_style,
                side: rule_side,
            });
            start = end;
        }
    }
    true
}

#[inline(never)]
pub(super) fn ExportGapRulePaintData(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    let geometry = box_fragment.GetGapGeometry();
    if geometry.is_null() || !box_fragment.Style().HasGapRule() {
        return;
    }
    let geometry = unsafe { &*geometry };
    let mut data = GapRulePaintData::default();
    let directions =
        if box_fragment.Style().RuleOverlap() == foundation::ERuleOverlap::kColumnOverRow {
            [
                GridTrackSizingDirection::kForRows,
                GridTrackSizingDirection::kForColumns,
            ]
        } else {
            [
                GridTrackSizingDirection::kForColumns,
                GridTrackSizingDirection::kForRows,
            ]
        };
    let mut has_supported_axis = false;
    for direction in directions {
        has_supported_axis |= ExportGapRuleDirection(box_fragment, geometry, direction, &mut data);
    }
    if has_supported_axis {
        paint.gap_rules = Some(Arc::new(data));
    }
}

pub(super) fn ExportOverflowClipMargin(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    if !box_fragment.ShouldApplyOverflowClipMargin() {
        return;
    }
    let outsets = box_fragment.OverflowClipMarginOutsets();
    paint.overflow_clip_margin_outsets = Some(Edges {
        top: Number(outsets.top),
        right: Number(outsets.right),
        bottom: Number(outsets.bottom),
        left: Number(outsets.left),
    });
}

#[cfg(test)]
mod gap_rule_tests {
    use super::*;
    use crate::internal::gap::gap_utils::GapSegmentState;

    #[test]
    fn intersection_pairing_respects_between_visibility() {
        let geometry = GapGeometry::new(ContainerType::kGrid);
        let intersections = vec![
            GapIntersection::with_segment_state(
                LayoutUnit::from_signed(0),
                GapSegmentState::new(GapSegmentState::kNone as u32),
            ),
            GapIntersection::with_segment_state(
                LayoutUnit::from_signed(10),
                GapSegmentState::default(),
            ),
            GapIntersection::with_segment_state(
                LayoutUnit::from_signed(20),
                GapSegmentState::new(GapSegmentState::kNone as u32),
            ),
        ];
        let mut start = 0;
        let mut end = 0;
        AdjustGapRuleIntersectionPair(
            GridTrackSizingDirection::kForColumns,
            &mut start,
            &mut end,
            0,
            foundation::RuleBreak::kNone,
            foundation::RuleVisibilityItems::kBetween,
            &geometry,
            &intersections,
        );
        assert_eq!((start, end), (0, 1));

        start = end;
        AdjustGapRuleIntersectionPair(
            GridTrackSizingDirection::kForColumns,
            &mut start,
            &mut end,
            0,
            foundation::RuleBreak::kNone,
            foundation::RuleVisibilityItems::kBetween,
            &geometry,
            &intersections,
        );
        assert_eq!(start, 2);
    }
}
