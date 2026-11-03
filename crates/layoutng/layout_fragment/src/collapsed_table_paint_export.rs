//! Chromium collapsed-table border paint projection.
//!
//! Kept separate from ordinary table payload export because collapsed edge
//! and fragmented-section reconstruction has its own performance profile.

#![allow(non_snake_case)]

use super::*;

fn ExportBorderStyle(style: EBorderStyle) -> BorderLineStyle {
    match style {
        EBorderStyle::kHidden => BorderLineStyle::kHidden,
        EBorderStyle::kDotted => BorderLineStyle::kDotted,
        EBorderStyle::kDashed => BorderLineStyle::kDashed,
        EBorderStyle::kSolid => BorderLineStyle::kSolid,
        EBorderStyle::kDouble => BorderLineStyle::kDouble,
        EBorderStyle::kGroove => BorderLineStyle::kGroove,
        EBorderStyle::kRidge => BorderLineStyle::kRidge,
        EBorderStyle::kInset => BorderLineStyle::kInset,
        EBorderStyle::kOutset => BorderLineStyle::kOutset,
        _ => BorderLineStyle::kNone,
    }
}

fn IsStartTableRowFragmented(section: &PhysicalBoxFragment) -> bool {
    for child in section.Children() {
        if child.IsTableRow() {
            let row = To::<PhysicalBoxFragment>(child.get());
            return IsBreakInside(FindPreviousBreakToken(unsafe { &*row }));
        }
    }
    false
}

fn IsEndTableRowFragmented(section: &PhysicalBoxFragment) -> bool {
    for child in section.Children().iter().rev() {
        if child.IsTableRow() {
            let token = To::<BlockBreakToken>(child.GetBreakToken());
            return IsBreakInside(token) && !unsafe { &*token }.IsAtBlockEnd();
        }
    }
    false
}

#[inline(never)]
pub(super) fn ExportCollapsedTablePaintData(
    box_fragment: &PhysicalBoxFragment,
    _style_sources: &StyleSourceMap,
    paint: &mut PaintProperties,
) {
    if !box_fragment.IsTable() {
        return;
    }
    let borders = box_fragment.TableCollapsedBorders();
    if borders.is_null() || !unsafe { &*borders }.IsCollapsed() {
        return;
    }
    let borders = unsafe { &*borders };
    let geometry = box_fragment.TableCollapsedBordersGeometry();
    if geometry.is_null() {
        return;
    }
    let geometry = unsafe { &*geometry };

    let mut data = CollapsedTablePaintData {
        edges_per_row: borders.EdgesPerRow() as usize,
        ..CollapsedTablePaintData::default()
    };
    data.edges.reserve(borders.EdgeCount() as usize);
    for (edge_index, border_edge) in borders.edges_.iter().enumerate() {
        let mut edge = CollapsedTableEdge {
            exists: !border_edge.style.Get().is_null(),
            box_order: border_edge.box_order as usize,
            ..CollapsedTableEdge::default()
        };
        if edge.exists {
            let style = borders.BorderStyle(edge_index as u32);
            edge.can_paint = borders.CanPaint(edge_index as u32);
            edge.width = Number(borders.BorderWidth(edge_index as u32));
            edge.style = ExportBorderStyle(style);
            edge.style_rank = style as u8;
            if border_edge.edge_side != EdgeSide::kDoNotFill {
                let style = unsafe { &*border_edge.style.Get() };
                let color = match border_edge.edge_side {
                    EdgeSide::kTop => style.BorderTopColor(),
                    EdgeSide::kRight => style.BorderRightColor(),
                    EdgeSide::kBottom => style.BorderBottomColor(),
                    EdgeSide::kLeft => style.BorderLeftColor(),
                    EdgeSide::kDoNotFill => unreachable!(),
                };
                edge.color =
                    crate::internal::boundary::paint_style_export::StyleColor(style, color);
            }
        }
        data.edges.push(edge);
    }
    data.columns.reserve(geometry.columns.len());
    for column in &geometry.columns {
        data.columns.push(Number(*column));
    }

    for child in box_fragment.Children() {
        if !child.IsTableSection() {
            continue;
        }
        let section = To::<PhysicalBoxFragment>(child.get());
        let section = unsafe { &*section };
        let Some(start_row) = section.TableSectionStartRowIndex() else {
            continue;
        };
        let Some(row_offsets) = section.TableSectionRowOffsets() else {
            continue;
        };
        let mut output = CollapsedTableSection {
            offset: Offset {
                x: Number(child.Offset().left),
                y: Number(child.Offset().top),
            },
            size: Size {
                width: Number(section.Size().width),
                height: Number(section.Size().height),
            },
            start_row: start_row as usize,
            ..CollapsedTableSection::default()
        };
        output.row_offsets.reserve(row_offsets.len());
        for row_offset in row_offsets {
            output.row_offsets.push(Number(*row_offset));
        }
        output.start_row_fragmented = IsStartTableRowFragmented(section);
        output.end_row_fragmented = IsEndTableRowFragmented(section);
        data.sections.push(output);
    }
    paint.collapsed_table = Some(Arc::new(data));
}
