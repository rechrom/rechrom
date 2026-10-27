//! Specialized native fragment paint payload projection.
//!
//! Table, MathML, frameset, fieldset, column-rule and overflow-clip
//! payloads are grouped by their fragment-output lifecycle rather than
//! retained in the generic native boundary.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:1280-1315
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

// cpp: layoutng/internal/boundary/layout_boundary.cc:1317-1353
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
                style: unsafe { &*source.node }.InputStyle().paint.clone(),
            });
        }
    }
    paint.table = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1355-1412
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

    let native = box_fragment.GetMathMLPaintInfo();
    data.kind = if native.IsRadicalOperator() {
        MathMLPaintKind::kRadical
    } else {
        MathMLPaintKind::kOperator
    };
    data.operator_inline_size = Number(native.operator_inline_size);
    data.operator_ascent = Number(native.operator_ascent);
    data.operator_descent = Number(native.operator_descent);
    data.radical_margin_inline_start = Number(native.radical_base_margins.inline_start);
    data.radical_margin_inline_end = Number(native.radical_base_margins.inline_end);
    data.radical_operator_inline_offset = native.radical_operator_inline_offset.map_or(0.0, Number);
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
        unsafe { &*primary_font }.GetFontMetrics().Ascent() as f64
    };
    ExportShapeResult(
        native.operator_shape_result_view.Get(),
        box_fragment.Style().ComputedFontSize() as f64,
        baseline,
        paint.writing_mode,
        &mut data.operator_glyph_runs,
    );
    paint.mathml = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1414-1437
pub(super) fn ExportFrameSetPaintData(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    if paint.source_kind != NodeKind::kFrameSet {
        return;
    }
    let native = box_fragment.GetFrameSetLayoutData();
    if native.is_null() {
        return;
    }
    let native = unsafe { &*native };
    let mut data = FrameSetPaintData::default();
    data.column_sizes.reserve(native.col_sizes.len());
    for size in &native.col_sizes {
        data.column_sizes.push(Number(*size));
    }
    data.row_sizes.reserve(native.row_sizes.len());
    for size in &native.row_sizes {
        data.row_sizes.push(Number(*size));
    }
    data.column_allows_border
        .extend(native.col_allow_border.iter().copied());
    data.row_allows_border
        .extend(native.row_allow_border.iter().copied());
    data.border_thickness = native.border_thickness as f64;
    data.has_border_color = native.has_border_color;
    paint.frame_set = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1439-1510
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

// cpp: layoutng/internal/boundary/layout_boundary.cc:1512-1551
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
    let native_style = box_fragment.Style();
    let iterator_gap_count = u32::try_from(gap_count).expect("gap count exceeds WTF size");
    let mut width_iterator = GapDataListIterator::new(
        native_style.ColumnRuleWidth().GetGapDataList(),
        iterator_gap_count,
    );
    let mut style_iterator = GapDataListIterator::new(
        native_style.ColumnRuleStyle().GetGapDataList(),
        iterator_gap_count,
    );
    let mut color_iterator = GapDataListIterator::new(
        native_style.ColumnRuleColor().GetGapDataList(),
        iterator_gap_count,
    );
    for index in 0..gap_count {
        if geometry.IsMultiColSpanner(index, GridTrackSizingDirection::kForColumns) {
            continue;
        }
        let width = f64::from(width_iterator.Next());
        let line_style = crate::internal::boundary::native_paint_style::BorderStyle(
            NativeComputedStyle::CollapsedBorderStyle(style_iterator.Next()),
        );
        let color = crate::internal::boundary::native_paint_style::ColorValue(
            native_style.VisitedDependentGapColor(&color_iterator.Next(), true),
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

// cpp: layoutng/internal/boundary/layout_boundary.cc:1553-1562
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
