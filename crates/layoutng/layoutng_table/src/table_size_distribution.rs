#![allow(non_snake_case)]
use foundation::{kIndefiniteSize, LayoutUnit, Vector};
use layoutng_assembly::internal::table_layout_algorithm_types::{
    CellBlockConstraint, ColspanCell, ColspanCells, Column, Columns, Row, Rows, RowspanCell,
    Sections,
};
use layoutng_geometry::geometry::logical_size::LogicalSize;
use std::cmp::{max, min};

// cpp: layoutng_table/table_layout_utils.cc:64-83
pub fn EnsureDistributableColumnExists(start: u32, count: u32, columns: &mut Columns) {
    debug_assert!((start as usize) < columns.data.len());
    debug_assert!(count > 1);
    let count = min(count as usize, columns.data.len() - start as usize);
    let span = &mut columns.data[start as usize..start as usize + count];
    for column in span.iter_mut() {
        if !column.is_collapsed {
            column.is_mergeable = false;
            return;
        }
    }
    span[0].is_mergeable = false;
}
// cpp: layoutng_table/table_layout_utils.cc:87-168
pub fn ApplyCellConstraintsToColumnConstraints(
    cells: &layoutng_assembly::internal::table_layout_algorithm_types::CellInlineConstraints,
    spacing: LayoutUnit,
    fixed: bool,
    colspan: &mut ColspanCells,
    columns: &mut Columns,
) {
    if columns.data.len() < cells.len() {
        let default = Column {
            is_table_fixed: fixed,
            is_mergeable: !fixed,
            ..Column::default()
        };
        columns.data.resize(cells.len(), default);
    } else if columns.data.len() > cells.len() {
        let mut last = columns.data.len() - 1;
        while last.wrapping_add(1) > cells.len() && columns.data[last].is_mergeable {
            last = last.wrapping_sub(1);
        }
        columns.data.truncate(last + 1);
    }
    for cell in colspan.iter() {
        EnsureDistributableColumnExists(cell.start_column, cell.span, columns);
    }
    for (column, cell) in columns.data.iter_mut().zip(cells) {
        column.Encompass(cell);
    }
    // Rust slice sort_by_key is stable, as required by std::stable_sort here.
    colspan.sort_by_key(|cell| (cell.span, cell.start_column));
    DistributeColspanCellsToColumns(colspan, spacing, fixed, columns);
    let mut total = 0f32;
    for column in &mut columns.data {
        if let Some(p) = column.percent {
            if !fixed && p + total > 100.0 {
                column.percent = Some(100.0 - total);
            }
            total += column.percent.unwrap();
        }
        column.min_inline_size.get_or_insert(LayoutUnit::default());
        column.max_inline_size.get_or_insert(LayoutUnit::default());
    }
    if fixed && total > 100.0 {
        for column in &mut columns.data {
            if let Some(p) = column.percent {
                column.percent = Some(p * 100.0 / total);
            }
        }
    }
}

// cpp: layoutng_table/table_layout_utils.cc:1743-1824
pub fn ComputeGridInlineMinMax(
    node: &layoutng_assembly::internal::table_node::TableNode,
    columns: &Columns,
    undistributable: LayoutUnit,
    fixed: bool,
    layout_pass: bool,
) -> layoutng_assembly::internal::min_max_sizes::MinMaxSizes {
    ComputeGridInlineMinMaxWithAllowance(columns, undistributable, fixed, || {
        node.AllowColumnPercentages(layout_pass)
    })
}
// A lazy closure retains the source's short-circuiting native ancestor query.
pub(crate) fn ComputeGridInlineMinMaxWithAllowance(
    columns: &Columns,
    undistributable: LayoutUnit,
    fixed: bool,
    allow_percentages: impl FnOnce() -> bool,
) -> layoutng_assembly::internal::min_max_sizes::MinMaxSizes {
    use layoutng_assembly::internal::{
        min_max_sizes::MinMaxSizes, table_layout_algorithm_types::TableTypes,
    };
    let zero = LayoutUnit::default();
    let mut sizes = MinMaxSizes::default();
    let (mut percent_estimate, mut nonpercent_max) = (zero, zero);
    let mut percent_sum = 0f32;
    for c in &columns.data {
        if let Some(lo) = c.min_inline_size {
            sizes.min_size += if fixed && c.IsFixed() {
                c.max_inline_size.unwrap()
            } else {
                lo
            };
            if c.percent.is_some() && c.percent.unwrap() > 0.0 {
                if c.max_inline_size.unwrap() > zero {
                    let estimate = LayoutUnit::from_f32(
                        100.0 / c.percent.unwrap()
                            * (c.max_inline_size.unwrap() - c.percent_border_padding),
                    );
                    percent_estimate = max(percent_estimate, estimate);
                }
            } else {
                nonpercent_max += c.max_inline_size.unwrap();
            }
        }
        if let Some(hi) = c.max_inline_size {
            sizes.max_size += hi;
        }
        if let Some(p) = c.percent {
            percent_sum += p;
        }
    }
    debug_assert!(percent_sum <= 100.5);
    if 100.0 < percent_sum {
        percent_sum = 100.0;
    }
    if percent_sum > 0.0 && allow_percentages() {
        let mut implied = zero;
        if nonpercent_max != zero {
            implied = if percent_sum == 100.0 {
                TableTypes::kTableMaxInlineSize()
            } else {
                LayoutUnit::from_f32((100.0 / (100.0 - percent_sum)) * nonpercent_max)
            };
        }
        sizes.max_size = max(max(sizes.max_size, implied), percent_estimate);
    }
    sizes.max_size = max(sizes.min_size, sizes.max_size);
    sizes += undistributable;
    sizes
}

// cpp: layoutng_table/table_layout_utils.cc:460-783
pub fn DistributeInlineSizeToComputedInlineSizeAuto(
    mut target: LayoutUnit,
    columns: &[Column],
    constrained: bool,
) -> Vector<LayoutUnit> {
    let zero = LayoutUnit::default();
    let mut counts = [0i32; 3]; // percent, fixed, auto
    let mut guesses = [zero; 4];
    let mut increases = [zero; 4];
    let mut percent_total = 0f32;
    let mut fixed_max = zero;
    let mut auto_max = zero;
    for c in columns {
        debug_assert!(c.min_inline_size.is_some() && c.max_inline_size.is_some());
        if c.is_mergeable {
            continue;
        }
        let lo = c.min_inline_size.unwrap();
        let hi = c.max_inline_size.unwrap();
        guesses[0] += lo;
        if let Some(p) = c.percent {
            counts[0] += 1;
            percent_total += p;
            let resolved = c.ResolvePercentInlineSize(target);
            for g in &mut guesses[1..] {
                *g += resolved;
            }
            increases[1] += resolved - lo;
        } else if c.is_constrained {
            counts[1] += 1;
            fixed_max += hi;
            guesses[1] += lo;
            guesses[2] += hi;
            guesses[3] += hi;
            increases[2] += hi - lo;
        } else {
            counts[2] += 1;
            auto_max += hi;
            guesses[1] += lo;
            guesses[2] += lo;
            guesses[3] += hi;
            increases[3] += hi - lo;
        }
    }
    let mut sizes = vec![zero; columns.len()];
    target = max(target, guesses[0]);
    let guess = guesses.iter().position(|v| *v >= target).unwrap_or(4);
    if guess == 0 {
        for (i, c) in columns.iter().enumerate() {
            if !c.is_mergeable {
                sizes[i] = c.min_inline_size.unwrap_or_default();
            }
        }
    } else if guess < 4 {
        let distributable = target - guesses[guess - 1];
        let exact = guess == 3 && target == guesses[3];
        let mut remaining = if exact { zero } else { distributable };
        let mut last = None;
        for (i, c) in columns.iter().enumerate() {
            if c.is_mergeable {
                continue;
            }
            let lo = c.min_inline_size.unwrap();
            let hi = c.max_inline_size.unwrap();
            let grow = match guess {
                1 => c.percent.is_some(),
                2 => c.percent.is_none() && c.is_constrained,
                3 => c.percent.is_none() && !c.is_constrained && !exact,
                _ => unreachable!(),
            };
            if grow {
                last = Some(i);
                let increase = if guess == 1 {
                    c.ResolvePercentInlineSize(target) - lo
                } else {
                    hi - lo
                };
                let delta = if increases[guess] > zero {
                    distributable.MulDiv(increase, increases[guess])
                } else {
                    distributable / counts[guess - 1]
                };
                remaining -= delta;
                sizes[i] = lo + delta;
            } else if c.percent.is_some() && guess != 1 {
                sizes[i] = c.ResolvePercentInlineSize(target);
            } else if guess == 3 && (c.is_constrained || exact) {
                sizes[i] = hi;
            } else {
                sizes[i] = lo;
            }
        }
        if remaining != zero {
            sizes[last.expect("distribution has no growing column")] += remaining;
        }
    } else {
        let distributable = target - guesses[3];
        let grow_kind = if counts[2] > 0 {
            2
        } else if counts[1] > 0 && constrained {
            1
        } else if counts[0] > 0 {
            0
        } else {
            return sizes;
        };
        let mut remaining = distributable;
        let mut last = None;
        for (i, c) in columns.iter().enumerate() {
            if c.is_mergeable {
                continue;
            }
            if grow_kind == 0 && c.percent.is_none() {
                continue;
            }
            let grow = match grow_kind {
                0 => c.percent.is_some(),
                1 => c.percent.is_none() && c.is_constrained,
                2 => c.percent.is_none() && !c.is_constrained,
                _ => unreachable!(),
            };
            let base = if c.percent.is_some() {
                c.ResolvePercentInlineSize(target)
            } else {
                c.max_inline_size.unwrap()
            };
            if grow {
                last = Some(i);
                let delta = if grow_kind == 0 {
                    if percent_total != 0.0 {
                        LayoutUnit::from_f32(distributable * c.percent.unwrap() / percent_total)
                    } else {
                        distributable / counts[0]
                    }
                } else {
                    let total = if grow_kind == 1 { fixed_max } else { auto_max };
                    if total > zero {
                        distributable.MulDiv(c.max_inline_size.unwrap(), total)
                    } else {
                        distributable / counts[grow_kind]
                    }
                };
                remaining -= delta;
                sizes[i] = base + delta;
            } else {
                sizes[i] = base;
            }
        }
        // The source's percent-only branch tests the index itself, so index 0
        // intentionally does not receive a remaining deficit.
        if remaining != zero && (grow_kind != 0 || last.unwrap_or(usize::MAX) != 0) {
            sizes[last.expect("distribution has no growing column")] += remaining;
        }
    }
    sizes
}

// cpp: layoutng_table/table_layout_utils.cc:785-924
pub fn SynchronizeAssignableTableInlineSizeAndColumnsFixed(
    target: LayoutUnit,
    columns: &Columns,
) -> Vector<LayoutUnit> {
    let zero = LayoutUnit::default();
    let fixed = |c: &Column| c.IsFixed() && c.max_inline_size != Some(zero);
    let zero_constrained = |c: &Column| c.is_constrained && c.max_inline_size == Some(zero);
    let mut counts = [0i32; 4]; // percent, fixed, zero constrained, auto
    let mut percent_size = zero;
    let mut fixed_size = zero;
    for c in &columns.data {
        if c.percent.is_some() {
            counts[0] += 1;
            percent_size += c.ResolvePercentInlineSize(target);
        } else if fixed(c) {
            counts[1] += 1;
            fixed_size += c.max_inline_size.unwrap_or_default();
        } else if zero_constrained(c) {
            counts[2] += 1;
        } else {
            counts[3] += 1;
        }
    }
    let mut sizes = vec![zero; columns.data.len()];
    let mut assigned = zero;
    let mut last = None;
    if counts[1] > 0 {
        let mut scale = 1f32;
        let mut available = true;
        let fixed_target = (target - percent_size).ClampNegativeToZero();
        if (fixed_size < fixed_target && counts[3] == 0) || fixed_size > target {
            if fixed_size != zero {
                scale = fixed_target.ToFloat() / fixed_size;
            } else {
                available = false;
            }
        }
        for (i, c) in columns.data.iter().enumerate() {
            if !fixed(c) {
                continue;
            }
            last = Some(i);
            sizes[i] = if available {
                LayoutUnit::from_f32(scale * c.max_inline_size.unwrap_or_default())
            } else {
                debug_assert_eq!(counts[1] as usize, columns.data.len());
                LayoutUnit::from_f32(target.ToFloat() / counts[1] as f32)
            };
            assigned += sizes[i];
        }
    }
    if assigned >= target {
        return sizes;
    }
    if counts[0] > 0 {
        let mut scale = 1f32;
        let mut available = true;
        if (percent_size < target - assigned && counts[3] == 0) || percent_size > target - assigned
        {
            if percent_size != zero {
                scale = (target - assigned).ToFloat() / percent_size;
            } else {
                available = false;
            }
        }
        for (i, c) in columns.data.iter().enumerate() {
            if c.percent.is_none() {
                continue;
            }
            last = Some(i);
            sizes[i] = if available {
                LayoutUnit::from_f32(scale * c.ResolvePercentInlineSize(target))
            } else {
                LayoutUnit::from_f32((target - assigned).ToFloat() / counts[0] as f32)
            };
            assigned += sizes[i];
        }
    }
    let distributable = target - assigned;
    let distribute_zero = counts[2] as usize == columns.data.len();
    for (i, c) in columns.data.iter().enumerate() {
        if c.percent.is_some() || fixed(c) || (zero_constrained(c) && !distribute_zero) {
            continue;
        }
        last = Some(i);
        sizes[i] = LayoutUnit::from_f32(
            distributable
                / if distribute_zero {
                    counts[2] as f32
                } else {
                    counts[3] as f32
                },
        );
        assigned += sizes[i];
    }
    sizes[last.expect("fixed distribution has no column")] += target - assigned;
    sizes
}

// cpp: layoutng_table/table_layout_utils.cc:926-1005
fn DistributeColspanCellToColumnsFixed(
    cell: &ColspanCell,
    spacing: LayoutUnit,
    columns: &mut Columns,
) {
    let zero = LayoutUnit::default();
    let span =
        &mut columns.data[cell.start_column as usize..(cell.start_column + cell.span) as usize];
    let count = span.iter().filter(|c| !c.is_mergeable).count();
    let mut inner = zero;
    for _ in 1..count {
        inner += spacing;
    }
    let lo = if cell.cell_inline_constraint.is_constrained {
        (cell.cell_inline_constraint.min_inline_size - inner).ClampNegativeToZero()
    } else {
        zero
    };
    let hi = (cell.cell_inline_constraint.max_inline_size - inner).ClampNegativeToZero();
    let new_lo = LayoutUnit::from_f32(lo / count as f32);
    let new_hi = LayoutUnit::from_f32(hi / count as f32);
    let new_percent = cell
        .cell_inline_constraint
        .percent
        .map(|p| p / count as f32);
    let (mut remaining_lo, mut remaining_hi) = (lo, hi);
    let mut last = None;
    for (i, c) in span.iter_mut().enumerate() {
        if c.is_mergeable {
            continue;
        }
        last = Some(i);
        remaining_lo -= new_lo;
        remaining_hi -= new_hi;
        if c.min_inline_size.is_none() {
            c.is_constrained |= cell.cell_inline_constraint.is_constrained;
            c.min_inline_size = Some(new_lo);
        }
        if c.max_inline_size.is_none() {
            c.is_constrained |= cell.cell_inline_constraint.is_constrained;
            c.max_inline_size = Some(new_hi);
        }
        if c.percent.is_none() && !c.is_constrained && new_percent.is_some() {
            c.percent = new_percent;
        }
    }
    let last = &mut span[last.expect("colspan has no distributable column")];
    last.min_inline_size = Some(last.min_inline_size.unwrap() + remaining_lo);
    last.max_inline_size = Some(last.max_inline_size.unwrap() + remaining_hi);
}

// cpp: layoutng_table/table_layout_utils.cc:1007-1121
fn DistributeColspanCellToColumnsAuto(
    cell: &ColspanCell,
    spacing: LayoutUnit,
    columns: &mut Columns,
) {
    if columns.data.is_empty() {
        return;
    }
    let zero = LayoutUnit::default();
    let start = cell.start_column as usize;
    let count = min(cell.span as usize, columns.data.len() - start);
    let span = &mut columns.data[start..start + count];
    let mut inner = zero;
    let active = span.iter().filter(|c| !c.is_mergeable).count();
    for _ in 1..active {
        inner += spacing;
    }
    let lo = (cell.cell_inline_constraint.min_inline_size - inner).ClampNegativeToZero();
    let hi = (cell.cell_inline_constraint.max_inline_size - inner).ClampNegativeToZero();
    if let Some(p) = cell.cell_inline_constraint.percent {
        let mut percent = 0f32;
        let mut nonpercent_count = 0u32;
        let mut nonpercent_max = zero;
        for c in span.iter_mut() {
            c.min_inline_size.get_or_insert(zero);
            c.max_inline_size.get_or_insert(zero);
            if c.is_mergeable {
                continue;
            }
            if let Some(p) = c.percent {
                percent += p;
            } else {
                nonpercent_count += 1;
                nonpercent_max += c.max_inline_size.unwrap();
            }
        }
        let surplus = p - percent;
        if surplus > 0.0 && nonpercent_count > 0 {
            for c in span.iter_mut() {
                if c.is_mergeable || c.percent.is_some() {
                    continue;
                }
                c.percent = Some(if nonpercent_max != zero {
                    surplus * c.max_inline_size.unwrap_or_default() / nonpercent_max
                } else {
                    surplus / nonpercent_count as f32
                });
            }
        }
    }
    for c in span.iter_mut() {
        c.min_inline_size.get_or_insert(zero);
        c.max_inline_size.get_or_insert(zero);
    }
    let sizes = DistributeInlineSizeToComputedInlineSizeAuto(lo, span, true);
    for (c, size) in span.iter_mut().zip(sizes) {
        c.min_inline_size = Some(max(c.min_inline_size.unwrap(), size));
    }
    let sizes = DistributeInlineSizeToComputedInlineSizeAuto(
        hi,
        span,
        cell.cell_inline_constraint.is_constrained,
    );
    for (c, size) in span.iter_mut().zip(sizes) {
        c.max_inline_size = Some(max(
            max(c.min_inline_size.unwrap(), c.max_inline_size.unwrap()),
            size,
        ));
    }
}

// cpp: layoutng_table/table_layout_utils.cc:1127-1329
pub fn DistributeExcessBlockSizeToRows(
    start: u32,
    count: u32,
    desired: LayoutUnit,
    rowspan: bool,
    spacing: LayoutUnit,
    percentage_size: LayoutUnit,
    rows: &mut Rows,
) {
    let zero = LayoutUnit::default();
    debug_assert!(desired >= zero);
    if count == 0 {
        return;
    }
    let end = start + count;
    debug_assert!(end as usize <= rows.len());
    let deficit = |r: &Row| {
        debug_assert_ne!(percentage_size, kIndefiniteSize);
        (LayoutUnit::from_f32(r.percent.unwrap() * percentage_size / 100.0) - r.block_size)
            .ClampNegativeToZero()
    };
    let (mut origins, mut percent, mut auto_nonempty, mut empty, mut nonempty, mut auto_empty) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let mut constrained_nonempty = 0;
    let (mut total, mut percent_deficit, mut auto_size) = (zero, zero, zero);
    for i in start as usize..end as usize {
        let r = &rows[i];
        total += r.block_size;
        if rowspan && i != start as usize && r.has_rowspan_start {
            origins.push(i);
        }
        let mut is_empty = r.block_size == zero;
        if r.percent.is_some() && r.percent != Some(0.0) && percentage_size != kIndefiniteSize {
            let d = deficit(r);
            if d != zero {
                percent.push(i);
                percent_deficit += d;
                is_empty = false;
            }
        }
        let constrained =
            r.is_constrained && (r.percent.is_none() || percentage_size != kIndefiniteSize);
        if is_empty {
            empty.push(i);
            if !constrained {
                auto_empty.push(i);
            }
        } else {
            nonempty.push(i);
            if constrained {
                constrained_nonempty += 1;
            } else {
                auto_nonempty.push(i);
                auto_size += r.block_size;
            }
        }
    }
    let mut distributable = (desired - spacing * (count as i32 - 1)) - total;
    if distributable <= zero {
        return;
    }
    if !percent.is_empty() {
        let size = min(percent_deficit, distributable);
        let mut remaining = size;
        for &i in &percent {
            let d = size.MulDiv(deficit(&rows[i]), percent_deficit);
            rows[i].block_size += d;
            total += d;
            distributable -= d;
            remaining -= d;
        }
        rows[*percent.last().unwrap()].block_size += remaining;
        distributable -= remaining;
        if distributable <= zero {
            return;
        }
    }
    if !origins.is_empty() {
        let mut remaining = distributable;
        for &i in &origins {
            let d = distributable / origins.len() as i32;
            rows[i].block_size += d;
            remaining -= d;
        }
        let last = &mut rows[*origins.last().unwrap()];
        last.block_size = max(last.block_size + remaining, zero);
        return;
    }
    if !auto_nonempty.is_empty() {
        let mut remaining = distributable;
        for &i in &auto_nonempty {
            let d = distributable.MulDiv(rows[i].block_size, auto_size);
            rows[i].block_size += d;
            remaining -= d;
        }
        rows[*auto_nonempty.last().unwrap()].block_size += remaining;
        return;
    }
    if !empty.is_empty() {
        let all_empty = empty.len() == count as usize;
        if rowspan {
            if all_empty {
                rows[*empty.last().unwrap()].block_size += distributable;
                return;
            }
        } else if all_empty || empty.len() + constrained_nonempty == count as usize {
            let grow = if !auto_empty.is_empty() {
                &auto_empty
            } else {
                &empty
            };
            let mut remaining = distributable;
            for &i in grow {
                let d = distributable / grow.len() as i32;
                rows[i].block_size = d;
                remaining -= d;
            }
            rows[*grow.last().unwrap()].block_size += remaining;
            return;
        }
    }
    if !nonempty.is_empty() {
        let mut remaining = distributable;
        for &i in &nonempty {
            let d = distributable.MulDiv(rows[i].block_size, total);
            rows[i].block_size += d;
            remaining -= d;
        }
        rows[*nonempty.last().unwrap()].block_size += remaining;
    }
}

// cpp: layoutng_table/table_layout_utils.h:35-38
pub struct CellBlockSizeData {
    pub block_size: LayoutUnit,
    pub is_initial_block_size_indefinite: bool,
}
// cpp: layoutng_table/table_layout_utils.cc:1333-1361
pub fn ComputeCellBlockSize(
    cell: &CellBlockConstraint,
    rows: &Rows,
    index: u32,
    spacing: &LogicalSize,
    specified: bool,
) -> CellBlockSizeData {
    let mut size = LayoutUnit::default();
    if !rows[index as usize].is_collapsed {
        for i in 0..cell.effective_rowspan {
            let row = &rows[(index + i) as usize];
            if row.is_collapsed {
                continue;
            }
            size += row.block_size;
            if i != 0 {
                size += spacing.block_size;
            }
        }
    }
    CellBlockSizeData {
        block_size: size,
        is_initial_block_size_indefinite: !(cell.is_constrained
            || (size > cell.min_block_size && specified)),
    }
}
// cpp: layoutng_table/table_layout_utils.cc:1826-1845
pub fn DistributeColspanCellsToColumns(
    cells: &ColspanCells,
    spacing: LayoutUnit,
    fixed: bool,
    columns: &mut Columns,
) {
    for cell in cells {
        debug_assert!(cell.span > 1);
        if fixed {
            DistributeColspanCellToColumnsFixed(cell, spacing, columns);
        } else {
            DistributeColspanCellToColumnsAuto(cell, spacing, columns);
        }
    }
}
// cpp: layoutng_table/table_layout_utils.cc:1850-1864
pub fn SynchronizeAssignableTableInlineSizeAndColumns(
    size: LayoutUnit,
    fixed: bool,
    columns: &Columns,
) -> Vector<LayoutUnit> {
    if columns.data.is_empty() {
        Vector::new()
    } else if fixed {
        SynchronizeAssignableTableInlineSizeAndColumnsFixed(size, columns)
    } else {
        DistributeInlineSizeToComputedInlineSizeAuto(size, &columns.data, true)
    }
}
// cpp: layoutng_table/table_layout_utils.cc:1866-1875
pub fn DistributeRowspanCellToRows(cell: &RowspanCell, spacing: LayoutUnit, rows: &mut Rows) {
    debug_assert!(cell.effective_rowspan > 1);
    DistributeExcessBlockSizeToRows(
        cell.start_row,
        cell.effective_rowspan,
        cell.min_block_size,
        true,
        spacing,
        kIndefiniteSize,
        rows,
    );
}
// cpp: layoutng_table/table_layout_utils.cc:1878-1889
pub fn DistributeSectionFixedBlockSizeToRows(
    start: u32,
    count: u32,
    size: LayoutUnit,
    spacing: LayoutUnit,
    percentage_size: LayoutUnit,
    rows: &mut Rows,
) {
    DistributeExcessBlockSizeToRows(start, count, size, false, spacing, percentage_size, rows);
}
// cpp: layoutng_table/table_layout_utils.cc:1891-2081
pub fn DistributeTableBlockSizeToSections(
    spacing: LayoutUnit,
    size: LayoutUnit,
    sections: &mut Sections,
    rows: &mut Rows,
) {
    if sections.is_empty() {
        return;
    }
    let zero = LayoutUnit::default();
    let distributable = (size - spacing * (sections.len() as i32 + 1)).ClampNegativeToZero();
    let percentage = |s: &layoutng_assembly::internal::table_layout_algorithm_types::Section| {
        max(
            s.block_size,
            LayoutUnit::from_f32(s.percent.unwrap() * distributable / 100.0),
        )
    };
    let mut minimum = zero;
    let mut percent_guess = zero;
    let mut has_body = false;
    let mut groups: [Vec<usize>; 6] = std::array::from_fn(|_| Vec::new()); // auto/fixed/percent, body variants
    let mut totals = [zero; 6];
    for (i, s) in sections.iter().enumerate() {
        minimum += s.block_size;
        percent_guess += if s.percent.is_some() {
            percentage(s)
        } else {
            s.block_size
        };
        has_body |= s.is_tbody;
        let kind = if s.percent.is_some() {
            2
        } else if s.is_constrained {
            1
        } else {
            0
        };
        groups[kind].push(i);
        if kind != 2 {
            totals[kind] += s.block_size;
        }
        if s.is_tbody {
            groups[kind + 3].push(i);
            if kind != 2 {
                totals[kind + 3] += s.block_size;
            }
        }
    }
    if distributable <= minimum {
        return;
    }
    if !groups[2].is_empty() && percent_guess > minimum {
        let available = min(percent_guess, distributable) - minimum;
        let difference = percent_guess - minimum;
        let mut remaining = available;
        for &i in &groups[2] {
            let s = &mut sections[i];
            let d = available.MulDiv(percentage(s) - s.block_size, difference);
            s.block_size += d;
            s.needs_redistribution = true;
            remaining -= d;
            minimum += d;
            totals[2] += s.block_size;
            if s.is_tbody {
                totals[5] += s.block_size;
            }
        }
        let s = &mut sections[*groups[2].last().unwrap()];
        s.block_size += remaining;
        totals[2] += remaining;
        minimum += remaining;
        if s.is_tbody {
            totals[5] += remaining;
        }
    }
    let base = if has_body { 3 } else { 0 };
    let kind = if !groups[base].is_empty() {
        base
    } else if !groups[base + 1].is_empty() {
        base + 1
    } else {
        base + 2
    };
    let grow = &groups[kind];
    debug_assert!(!grow.is_empty());
    let available = distributable - minimum;
    if available > zero {
        let mut remaining = available;
        for &i in grow {
            let s = &mut sections[i];
            let d = if totals[kind] > zero {
                available.MulDiv(s.block_size, totals[kind])
            } else {
                available / grow.len() as i32
            };
            s.block_size += d;
            s.needs_redistribution = true;
            remaining -= d;
        }
        sections[*grow.last().unwrap()].block_size += remaining;
    }
    for s in sections {
        if s.needs_redistribution {
            DistributeExcessBlockSizeToRows(
                s.start_row,
                s.row_count,
                s.block_size,
                false,
                spacing,
                s.block_size,
                rows,
            );
        }
    }
}
