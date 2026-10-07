#![allow(non_snake_case)]
use foundation::{
    kIndefiniteSize, EBoxSizing, EDisplay, EVisibility, HeapVector, IsHorizontalWritingMode,
    IsParallelWritingMode, LayoutUnit, To, Visitor, WritingDirectionMode,
};
use layoutng_assembly::internal::{
    block_node::BlockNode,
    constraint_space::LayoutResultCacheSlot,
    constraint_space_builder::ConstraintSpaceBuilder,
    layout_input_node::{LayoutInputNode, MinMaxSizesFloatInput},
    layout_node_metadata::Element,
    length_utils::SizeType,
    min_max_sizes::MinMaxSizes,
    table_layout_algorithm_types::*,
};
use layoutng_geometry::geometry::{box_strut::BoxStrut, logical_size::LogicalSize};
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_table/table_layout_algorithm_types.cc:25-80
fn InlineSizesFromStyle(
    style: &ComputedStyle,
    inline_border_padding: LayoutUnit,
    is_parallel: bool,
) -> (
    Option<LayoutUnit>,
    Option<LayoutUnit>,
    Option<LayoutUnit>,
    Option<f32>,
) {
    let length = if is_parallel {
        style.LogicalWidth()
    } else {
        style.LogicalHeight()
    };
    let min_length = if is_parallel {
        style.LogicalMinWidth()
    } else {
        style.LogicalMinHeight()
    };
    let max_length = if is_parallel {
        style.LogicalMaxWidth()
    } else {
        style.LogicalMaxHeight()
    };
    let convert = |v: f32| {
        let v = LayoutUnit::from_f32(v);
        if style.BoxSizing() == EBoxSizing::kContentBox {
            v + inline_border_padding
        } else {
            v.max(inline_border_padding)
        }
    };
    let inline_size = length.IsFixed().then(|| convert(length.Pixels()));
    let min_inline_size = min_length.IsFixed().then(|| convert(min_length.Pixels()));
    let mut max_inline_size = max_length.IsFixed().then(|| convert(max_length.Pixels()));
    if let (Some(min), Some(max)) = (min_inline_size, max_inline_size) {
        max_inline_size = Some(max.max(min));
    }
    let mut percentage_inline_size = None;
    if length.IsPercent() {
        percentage_inline_size = Some(length.PercentValue());
    } else if length.IsCalculated() && !length.GetCalculationValue().IsExpression() {
        let v = length.GetPixelsAndPercent();
        if v.pixels == 0.0 {
            percentage_inline_size = Some(v.percent);
        }
    }
    if let Some(percent) = percentage_inline_size {
        if max_length.IsPercent() {
            let max = max_length.PercentValue();
            percentage_inline_size = Some(if max < percent { max } else { percent });
        }
    }
    if let (Some(min), Some(max)) = (min_inline_size, max_inline_size) {
        debug_assert!(max >= min);
    }
    (
        inline_size,
        min_inline_size,
        max_inline_size,
        percentage_inline_size,
    )
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:86-122
#[unsafe(no_mangle)]
pub extern "Rust" fn TableTypesCreateColumnFromTable(
    style: &ComputedStyle,
    default_inline_size: Option<LayoutUnit>,
    is_table_fixed: bool,
) -> Column {
    let (inline_size, min_inline_size, _, mut percent) =
        InlineSizesFromStyle(style, LayoutUnit::default(), true);
    let mut inline_size = inline_size.or(default_inline_size);
    if let (Some(min), Some(size)) = (min_inline_size, inline_size) {
        inline_size = Some(size.max(min));
    }
    let is_constrained = inline_size.is_some();
    if percent == Some(0.0) {
        percent = None;
    }
    let is_collapsed = style.Visibility() == EVisibility::kCollapse;
    let is_mergeable = !is_table_fixed
        && inline_size.unwrap_or_default() == LayoutUnit::default()
        && percent.unwrap_or(0.0) == 0.0;
    Column::new(
        Some(min_inline_size.unwrap_or_default()),
        inline_size,
        percent,
        LayoutUnit::default(),
        is_constrained,
        is_collapsed,
        is_table_fixed,
        is_mergeable,
    )
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:128-213
#[unsafe(no_mangle)]
pub extern "Rust" fn TableTypesCreateCellInlineConstraintFromTable(
    node: &BlockNode,
    table_writing_direction: WritingDirectionMode,
    is_fixed_layout: bool,
    cell_border: &BoxStrut,
    cell_padding: &BoxStrut,
) -> CellInlineConstraint {
    let style = node.Style();
    let table_writing_mode = table_writing_direction.GetWritingMode();
    let is_parallel = IsParallelWritingMode(table_writing_mode, style.GetWritingMode());
    let mut cached_min_max_sizes: Option<MinMaxSizes> = None;
    let mut min_max_sizes = || {
        if cached_min_max_sizes.is_none() {
            let cell_writing_direction = style.GetWritingDirection();
            let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
                table_writing_mode,
                cell_writing_direction,
                true,
                true,
                false,
            );
            builder.SetTableCellBorders(
                cell_border,
                cell_writing_direction,
                table_writing_direction,
            );
            builder.SetIsTableCell(true);
            builder.SetCacheSlot(LayoutResultCacheSlot::kMeasure);
            if !is_parallel {
                let icb = node.InitialContainingBlockSize();
                builder.SetOrthogonalFallbackInlineSize(
                    if IsHorizontalWritingMode(table_writing_mode) {
                        icb.height
                    } else {
                        icb.width
                    },
                );
            }
            builder.SetAvailableSize(LogicalSize::new(kIndefiniteSize, kIndefiniteSize));
            cached_min_max_sizes = Some(
                node.ComputeMinMaxSizes(
                    table_writing_mode,
                    SizeType::kIntrinsic,
                    &builder.ToConstraintSpace(),
                    MinMaxSizesFloatInput::default(),
                )
                .sizes,
            );
        }
        cached_min_max_sizes.unwrap()
    };
    let border_padding = (*cell_border + *cell_padding).InlineSum();
    let (css_size, css_min, css_max, css_percent) =
        InlineSizesFromStyle(style, border_padding, is_parallel);
    let mut resolved_min = LayoutUnit::default();
    if !is_fixed_layout {
        resolved_min = min_max_sizes().min_size.max(css_min.unwrap_or_default());
        let dom = node.GetDOMNode();
        let has_nowrap =
            !dom.is_null() && unsafe { &*To::<Element>(dom) }.HasNoWrapAttributeForLayout();
        if let Some(size) = css_size {
            if unsafe { &*node.GetLayoutBox() }.InQuirksModeForLayout() && has_nowrap {
                resolved_min = resolved_min.max(size);
            }
        }
    }
    // C++ value_or evaluates its argument even when css_size is present.
    let mut content_max = css_size.unwrap_or(min_max_sizes().max_size);
    if let Some(max) = css_max {
        content_max = content_max.min(max);
        resolved_min = resolved_min.min(max);
    }
    let resolved_max = resolved_min.max(content_max);
    let percent_border_padding =
        if is_fixed_layout && css_percent.is_some() && style.BoxSizing() == EBoxSizing::kContentBox
        {
            border_padding
        } else {
            LayoutUnit::default()
        };
    debug_assert!(resolved_min <= resolved_max);
    debug_assert!(resolved_max >= percent_border_padding);
    CellInlineConstraint {
        min_inline_size: resolved_min,
        max_inline_size: resolved_max,
        percent: css_percent,
        percent_border_padding,
        is_constrained: css_size.is_some(),
    }
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:215-236
#[unsafe(no_mangle)]
pub extern "Rust" fn TableTypesCreateSectionFromTable(
    node: &LayoutInputNode,
    start_row: u32,
    row_count: u32,
    block_size: LayoutUnit,
    treat_as_tbody: bool,
) -> Section {
    let size = node.Style().LogicalHeight();
    Section {
        start_row,
        row_count,
        block_size,
        percent: size.IsPercent().then(|| size.PercentValue()),
        is_constrained: size.IsFixed() || size.IsPercent(),
        is_tbody: treat_as_tbody,
        needs_redistribution: false,
    }
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:238-262
#[unsafe(no_mangle)]
pub extern "Rust" fn TableCellInlineConstraintEncompassFromTable(
    constraint: &mut CellInlineConstraint,
    other: &CellInlineConstraint,
) {
    constraint.min_inline_size = constraint.min_inline_size.max(other.min_inline_size);
    constraint.max_inline_size = if constraint.is_constrained == other.is_constrained {
        constraint.max_inline_size.max(other.max_inline_size)
    } else if constraint.is_constrained {
        constraint.max_inline_size.max(other.min_inline_size)
    } else {
        debug_assert!(other.is_constrained);
        constraint.min_inline_size.max(other.max_inline_size)
    };
    constraint.is_constrained |= other.is_constrained;
    if other.percent > constraint.percent {
        constraint.percent = other.percent;
        constraint.percent_border_padding = other.percent_border_padding;
    }
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:264-306
#[unsafe(no_mangle)]
pub extern "Rust" fn TableColumnEncompassFromTable(
    column: &mut Column,
    cell: &Option<CellInlineConstraint>,
) {
    let Some(cell) = cell else {
        return;
    };
    if column.is_constrained && column.is_table_fixed {
        return;
    }
    if !column.is_table_fixed {
        column.is_mergeable = false;
    }
    if let Some(min) = column.min_inline_size {
        if min < cell.min_inline_size {
            column.min_inline_size = Some(cell.min_inline_size);
        }
        column.max_inline_size = Some(if column.is_constrained {
            column
                .max_inline_size
                .expect("constrained column maximum")
                .max(if cell.is_constrained {
                    cell.max_inline_size
                } else {
                    cell.min_inline_size
                })
        } else {
            column
                .max_inline_size
                .unwrap_or_default()
                .max(cell.max_inline_size)
        });
    } else {
        column.min_inline_size = Some(cell.min_inline_size);
        column.max_inline_size = Some(cell.max_inline_size);
    }
    if let (Some(min), Some(max)) = (column.min_inline_size, column.max_inline_size) {
        column.max_inline_size = Some(min.max(max));
    }
    if cell.percent > column.percent {
        column.percent = cell.percent;
        column.percent_border_padding = cell.percent_border_padding;
    }
    column.is_constrained |= cell.is_constrained;
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:308-343
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenConstructFromTable(
    table: &BlockNode,
) -> TableGroupedChildren {
    let mut grouped = TableGroupedChildren {
        captions: HeapVector::new(),
        columns: HeapVector::new(),
        header: BlockNode::null(),
        bodies: HeapVector::new(),
        footer: BlockNode::null(),
    };
    let mut child = table.FirstChild();
    while child.is_non_null() {
        let block_child = BlockNode::from(child.clone());
        if block_child.IsTableCaption() {
            grouped.captions.push(block_child);
        } else {
            match child.Style().Display() {
                EDisplay::kTableColumn | EDisplay::kTableColumnGroup => {
                    grouped.columns.push(block_child)
                }
                EDisplay::kTableHeaderGroup => {
                    if !grouped.header.is_non_null() {
                        grouped.header = block_child;
                    } else {
                        grouped.bodies.push(block_child);
                    }
                }
                EDisplay::kTableRowGroup => grouped.bodies.push(block_child),
                EDisplay::kTableFooterGroup => {
                    if !grouped.footer.is_non_null() {
                        grouped.footer = block_child;
                    } else {
                        grouped.bodies.push(block_child);
                    }
                }
                _ => panic!("unexpected table child"),
            }
        }
        child = block_child_next(&child);
    }
    grouped
}
fn block_child_next(child: &LayoutInputNode) -> LayoutInputNode {
    BlockNode::from(child.clone()).NextSibling()
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:345-351
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenTraceFromTable(
    children: &TableGroupedChildren,
    visitor: &mut Visitor,
) {
    for node in &children.captions {
        node.Trace(visitor);
    }
    for node in &children.columns {
        node.Trace(visitor);
    }
    children.header.Trace(visitor);
    for node in &children.bodies {
        node.Trace(visitor);
    }
    children.footer.Trace(visitor);
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:361-371
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenIteratorConstructFromTable(
    grouped_children: &TableGroupedChildren,
    is_end: bool,
) -> TableGroupedChildrenIterator {
    let mut result = TableGroupedChildrenIterator {
        grouped_children_: grouped_children,
        current_section_: if is_end {
            CurrentSection::kEnd
        } else {
            CurrentSection::kNone
        },
        body_vector_: std::ptr::null(),
        position_: 0,
    };
    if !is_end {
        TableGroupedChildrenIteratorAdvanceForwardFromTable(&mut result);
    }
    result
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:373-393
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenIteratorIncrementFromTable(
    iterator: &mut TableGroupedChildrenIterator,
) {
    match iterator.current_section_ {
        CurrentSection::kHead | CurrentSection::kFoot => {
            TableGroupedChildrenIteratorAdvanceForwardFromTable(iterator)
        }
        CurrentSection::kBody => {
            iterator.position_ += 1;
            if iterator.position_ == unsafe { &*iterator.grouped_children_ }.bodies.size() {
                TableGroupedChildrenIteratorAdvanceForwardFromTable(iterator);
            }
        }
        CurrentSection::kEnd => {}
        CurrentSection::kNone => panic!("invalid table iterator increment"),
    }
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:395-414
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenIteratorDecrementFromTable(
    iterator: &mut TableGroupedChildrenIterator,
) {
    match iterator.current_section_ {
        CurrentSection::kHead | CurrentSection::kFoot | CurrentSection::kEnd => {
            TableGroupedChildrenIteratorAdvanceBackwardFromTable(iterator)
        }
        CurrentSection::kBody => {
            if iterator.position_ == 0 {
                TableGroupedChildrenIteratorAdvanceBackwardFromTable(iterator);
            } else {
                iterator.position_ -= 1;
            }
        }
        CurrentSection::kNone => panic!("invalid table iterator decrement"),
    }
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:416-428
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenIteratorDereferenceFromTable(
    iterator: &TableGroupedChildrenIterator,
) -> BlockNode {
    let grouped = unsafe { &*iterator.grouped_children_ };
    match iterator.current_section_ {
        CurrentSection::kHead => grouped.header.clone(),
        CurrentSection::kFoot => grouped.footer.clone(),
        CurrentSection::kBody => unsafe { &*iterator.body_vector_ }
            .at(iterator.position_)
            .clone(),
        _ => panic!("invalid table iterator dereference"),
    }
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:430-437
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenIteratorEqualFromTable(
    left: &TableGroupedChildrenIterator,
    right: &TableGroupedChildrenIterator,
) -> bool {
    left.current_section_ == right.current_section_
        && (left.current_section_ != CurrentSection::kBody
            || (left.body_vector_ == right.body_vector_ && left.position_ == right.position_))
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:439-460
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenIteratorAdvanceForwardFromTable(
    iterator: &mut TableGroupedChildrenIterator,
) {
    let grouped = unsafe { &*iterator.grouped_children_ };
    match iterator.current_section_ {
        CurrentSection::kNone => {
            iterator.current_section_ = CurrentSection::kHead;
            if !grouped.header.is_non_null() {
                TableGroupedChildrenIteratorAdvanceForwardFromTable(iterator);
            }
        }
        CurrentSection::kHead => {
            iterator.current_section_ = CurrentSection::kBody;
            iterator.body_vector_ = &grouped.bodies;
            iterator.position_ = 0;
            if grouped.bodies.is_empty() {
                TableGroupedChildrenIteratorAdvanceForwardFromTable(iterator);
            }
        }
        CurrentSection::kBody => {
            iterator.current_section_ = CurrentSection::kFoot;
            if !grouped.footer.is_non_null() {
                TableGroupedChildrenIteratorAdvanceForwardFromTable(iterator);
            }
        }
        CurrentSection::kFoot => iterator.current_section_ = CurrentSection::kEnd,
        CurrentSection::kEnd => panic!("invalid table iterator forward advance"),
    }
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:462-478
#[unsafe(no_mangle)]
pub extern "Rust" fn TableGroupedChildrenIteratorAdvanceBackwardFromTable(
    iterator: &mut TableGroupedChildrenIterator,
) {
    let grouped = unsafe { &*iterator.grouped_children_ };
    match iterator.current_section_ {
        CurrentSection::kNone => panic!("invalid table iterator backward advance"),
        CurrentSection::kHead => iterator.current_section_ = CurrentSection::kNone,
        CurrentSection::kBody => {
            iterator.current_section_ = CurrentSection::kHead;
            if !grouped.header.is_non_null() {
                TableGroupedChildrenIteratorAdvanceBackwardFromTable(iterator);
            }
        }
        CurrentSection::kFoot => {
            iterator.current_section_ = CurrentSection::kBody;
            iterator.body_vector_ = &grouped.bodies;
            if grouped.bodies.is_empty() {
                TableGroupedChildrenIteratorAdvanceBackwardFromTable(iterator);
            } else {
                iterator.position_ = grouped.bodies.size() - 1;
            }
        }
        CurrentSection::kEnd => {
            iterator.current_section_ = CurrentSection::kFoot;
            if !grouped.footer.is_non_null() {
                TableGroupedChildrenIteratorAdvanceBackwardFromTable(iterator);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{
        CalculationValue, LayoutHeapScope, Length, LengthValueRange, PixelsAndPercent, WritingMode,
    };
    use layoutng_style::style::computed_style::ComputedStyleBuilder;
    fn unit(s: &str) -> Option<LayoutUnit> {
        if s == "none" {
            None
        } else {
            Some(LayoutUnit::from_f64(s.parse().unwrap()))
        }
    }
    fn percent(s: &str) -> Option<f32> {
        if s == "none" {
            None
        } else {
            Some(s.parse().unwrap())
        }
    }
    fn length(s: &str) -> Length {
        if s == "auto" {
            return Length::default();
        }
        if s == "none" {
            return Length::None();
        }
        let mut fields = s.split(':');
        match fields.next().unwrap() {
            "f" => Length::Fixed(fields.next().unwrap().parse::<f64>().unwrap()),
            "p" => Length::Percent(fields.next().unwrap().parse::<f64>().unwrap()),
            "c" => Length::from_calculation_value(CalculationValue::new(
                PixelsAndPercent::new(
                    fields.next().unwrap().parse().unwrap(),
                    fields.next().unwrap().parse().unwrap(),
                    true,
                    true,
                ),
                LengthValueRange::kAll,
            )),
            _ => panic!("unknown frozen length"),
        }
    }
    fn dump_column(c: &Column) -> Vec<Option<f64>> {
        vec![
            c.min_inline_size.map(|v| v.ToDouble()),
            c.max_inline_size.map(|v| v.ToDouble()),
            c.percent.map(f64::from),
            Some(c.percent_border_padding.ToDouble()),
            Some(c.is_constrained as u8 as f64),
            Some(c.is_collapsed as u8 as f64),
            Some(c.is_table_fixed as u8 as f64),
            Some(c.is_mergeable as u8 as f64),
        ]
    }
    #[test]
    fn native_table_column_and_cell_constraint_algorithms_match_cpp() {
        crate::native_test_thread::run(column_constraint_body);
    }
    fn column_constraint_body() {
        let _scope = LayoutHeapScope::new();
        let inputs =
            include_str!("../../../artifacts/cpp-reference/table-column-constraints-input.tsv")
                .lines()
                .collect::<Vec<_>>();
        let results =
            include_str!("../../../artifacts/cpp-reference/table-column-constraints-results.tsv")
                .lines()
                .collect::<Vec<_>>();
        assert_eq!(inputs.len(), 4416);
        assert_eq!(results.len(), inputs.len());
        for (index, (line, result)) in inputs.iter().zip(results).enumerate() {
            let v = line.split('\t').collect::<Vec<_>>();
            let actual = match v[0] {
                "column" => {
                    let mut builder = ComputedStyleBuilder::from_style(unsafe {
                        &*ComputedStyle::GetInitialStyleSingleton()
                    });
                    let vertical = v[7] == "1";
                    builder.SetWritingMode(if vertical {
                        WritingMode::kVerticalRl
                    } else {
                        WritingMode::kHorizontalTb
                    });
                    builder.SetVisibility(if v[6] == "1" {
                        EVisibility::kCollapse
                    } else {
                        EVisibility::kVisible
                    });
                    if vertical {
                        builder.SetHeight(&length(v[1]));
                        builder.SetMinHeight(&length(v[2]));
                        builder.SetMaxHeight(&length(v[3]));
                    } else {
                        builder.SetWidth(&length(v[1]));
                        builder.SetMinWidth(&length(v[2]));
                        builder.SetMaxWidth(&length(v[3]));
                    }
                    dump_column(&TableTypes::CreateColumn(
                        unsafe { &*builder.TakeStyle() },
                        unit(v[4]),
                        v[5] == "1",
                    ))
                }
                "cell" => {
                    let mut c = CellInlineConstraint {
                        min_inline_size: unit(v[1]).unwrap(),
                        max_inline_size: unit(v[2]).unwrap(),
                        percent: percent(v[3]),
                        percent_border_padding: unit(v[4]).unwrap(),
                        is_constrained: v[5] == "1",
                    };
                    c.Encompass(&CellInlineConstraint {
                        min_inline_size: unit(v[6]).unwrap(),
                        max_inline_size: unit(v[7]).unwrap(),
                        percent: percent(v[8]),
                        percent_border_padding: unit(v[9]).unwrap(),
                        is_constrained: v[10] == "1",
                    });
                    vec![
                        Some(c.min_inline_size.ToDouble()),
                        Some(c.max_inline_size.ToDouble()),
                        c.percent.map(f64::from),
                        Some(c.percent_border_padding.ToDouble()),
                        Some(c.is_constrained as u8 as f64),
                    ]
                }
                "merge" => {
                    let mut c = Column::new(
                        unit(v[1]),
                        unit(v[2]),
                        percent(v[3]),
                        unit(v[4]).unwrap(),
                        v[5] == "1",
                        false,
                        v[6] == "1",
                        v[7] == "1",
                    );
                    c.Encompass(&Some(CellInlineConstraint {
                        min_inline_size: unit(v[8]).unwrap(),
                        max_inline_size: unit(v[9]).unwrap(),
                        percent: percent(v[10]),
                        percent_border_padding: unit(v[11]).unwrap(),
                        is_constrained: v[12] == "1",
                    }));
                    dump_column(&c)
                }
                _ => panic!("unknown source constraint case"),
            };
            let mut expected = result.split('\t');
            assert_eq!(expected.next().unwrap().parse::<usize>().unwrap(), index);
            let expected = expected
                .map(|v| {
                    if v == "none" {
                        None
                    } else {
                        Some(v.parse::<f64>().unwrap())
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "source constraint case {index}: {line}");
        }
    }
}
