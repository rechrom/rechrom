#![allow(non_snake_case)]

use foundation::{HeapVector, LayoutUnit, WtfSizeT};

use crate::flex_item::FlexItem;

// cpp: layoutng_flex/flex_line_breaker.h:17-28
pub struct InitialFlexLine {
    pub count: WtfSizeT,
    pub sum_hypothetical_main_size: LayoutUnit,
}

impl InitialFlexLine {
    pub fn new(count: WtfSizeT, sum_hypothetical_main_size: LayoutUnit) -> Self {
        Self {
            count,
            sum_hypothetical_main_size,
        }
    }
}

// cpp: layoutng_flex/flex_line_breaker.h:30-39
pub struct FlexLineBreakerResult {
    pub flex_lines: HeapVector<InitialFlexLine, 1>,
    pub max_sum_hypothetical_main_size: LayoutUnit,
}

impl Default for FlexLineBreakerResult {
    fn default() -> Self {
        Self {
            flex_lines: HeapVector::default(),
            max_sum_hypothetical_main_size: LayoutUnit::default(),
        }
    }
}

// cpp: layoutng_flex/flex_line_breaker.h:49-69
pub fn BreakFlexItemsIntoLines(
    all_items: &[FlexItem],
    line_break_size: LayoutUnit,
    gap_between_items: LayoutUnit,
    is_multi_line: bool,
    balance_min_line_count: Option<WtfSizeT>,
) -> FlexLineBreakerResult {
    if all_items.is_empty() {
        return FlexLineBreakerResult::default();
    }
    if let Some(min_line_count) = balance_min_line_count {
        return BalanceBreakFlexItemsIntoLines(
            all_items,
            line_break_size,
            gap_between_items,
            min_line_count,
        );
    }
    GreedyBreakFlexItemsIntoLines(all_items, line_break_size, gap_between_items, is_multi_line)
}

// C++ ClampedNumeric<uint64_t> saturates every scoring arithmetic operation.
// cpp: layoutng_flex/flex_line_breaker.cc:16-18
type ScoreUnit = u64;
const INFINITY: ScoreUnit = u64::MAX;
const NOT_FOUND: usize = usize::MAX;

// cpp: layoutng_flex/flex_line_breaker.cc:65-71
#[derive(Clone, Copy)]
struct ScoreData {
    limit: ScoreUnit,
    best_score: ScoreUnit,
    best_break: usize,
    start: usize,
    initial_start: usize,
}

impl Default for ScoreData {
    fn default() -> Self {
        Self {
            limit: INFINITY,
            best_score: INFINITY,
            best_break: NOT_FOUND,
            start: NOT_FOUND,
            initial_start: NOT_FOUND,
        }
    }
}

// cpp: layoutng_flex/flex_line_breaker.cc:73-79
struct ScoreContext {
    gap_between_items: ScoreUnit,
    line_break_size: ScoreUnit,
    sums: Vec<ScoreUnit>,
    scores: Vec<ScoreData>,
}

// cpp: layoutng_flex/flex_line_breaker.cc:81-145
fn Score(end: usize, limit: ScoreUnit, ctx: &mut ScoreContext) -> ScoreUnit {
    let data = ctx.scores[end];
    let mut best_score = INFINITY;
    let mut best_break = NOT_FOUND;
    let mut start = data.initial_start;
    if data.limit == INFINITY {
        if data.best_score != INFINITY {
            return data.best_score;
        }
    } else if limit <= data.limit {
        return data.best_score;
    } else {
        best_score = data.best_score;
        best_break = data.best_break;
        start = data.start;
    }

    for start_index in start..=end {
        let length = ctx.sums[end]
            .saturating_sub(if start_index == 0 {
                0
            } else {
                ctx.sums[start_index - 1]
            })
            .saturating_sub(ctx.gap_between_items);
        let line_score = if length > ctx.line_break_size {
            0
        } else {
            let free_space = ctx.line_break_size - length;
            free_space.saturating_mul(free_space)
        };
        if line_score > best_score {
            break;
        }
        if line_score > limit {
            let entry = &mut ctx.scores[end];
            entry.best_score = best_score;
            entry.best_break = best_break;
            entry.limit = limit;
            entry.start = start_index;
            return best_score;
        }
        let next_limit = if best_score == INFINITY {
            INFINITY
        } else {
            best_score.saturating_sub(line_score)
        };
        let previous_score = if start_index == 0 {
            0
        } else {
            Score(start_index - 1, next_limit, ctx)
        };
        let score = line_score.saturating_add(previous_score);
        if score <= best_score {
            best_score = score;
            best_break = if start_index == 0 {
                NOT_FOUND
            } else {
                start_index - 1
            };
        }
    }
    let entry = &mut ctx.scores[end];
    entry.best_score = best_score;
    entry.best_break = best_break;
    entry.limit = INFINITY;
    entry.start = NOT_FOUND;
    best_score
}

// cpp: layoutng_flex/flex_line_breaker.cc:148-176
fn GreedyLineCount(ctx: &ScoreContext, line_break_size: ScoreUnit) -> usize {
    let mut index = 0;
    let mut previous_sum: ScoreUnit = 0;
    let mut line_count = 1;
    loop {
        let bound = previous_sum
            .saturating_add(line_break_size)
            .saturating_add(ctx.gap_between_items);
        let next = index + ctx.sums[index..].partition_point(|sum| *sum <= bound);
        if next == ctx.sums.len() {
            break;
        }
        let is_single_item = usize::from(line_count == 1) + next - index <= 1;
        index = if is_single_item { next } else { next - 1 };
        previous_sum = ctx.sums[index];
        if index + 1 != ctx.sums.len() {
            line_count += 1;
        }
    }
    line_count
}

// cpp: layoutng_flex/flex_line_breaker.cc:178-269
fn ApplyMinLineCount(min_line_count: usize, ctx: &mut ScoreContext) -> usize {
    let mut low: ScoreUnit = 0;
    let mut high = ctx.line_break_size.min(
        ctx.sums
            .last()
            .copied()
            .unwrap()
            .saturating_sub(ctx.gap_between_items),
    );
    while low < high {
        let midpoint = low + (high - low) / 2;
        if GreedyLineCount(ctx, midpoint) > min_line_count {
            low = midpoint + 1;
        } else {
            high = midpoint;
        }
    }
    ctx.line_break_size = high;
    let mut line_count = GreedyLineCount(ctx, ctx.line_break_size);
    if line_count >= min_line_count {
        return line_count;
    }

    let mut perfect_fit_indices = Vec::new();
    let mut previous: ScoreUnit = 0;
    let mut line_has_content = false;
    for index in 0..ctx.sums.len() {
        let line_size = ctx.sums[index]
            .saturating_sub(previous)
            .saturating_sub(ctx.gap_between_items);
        if line_size == ctx.line_break_size {
            perfect_fit_indices.push(index);
        }
        if line_size > ctx.line_break_size {
            previous = if line_has_content {
                ctx.sums[index - 1]
            } else {
                ctx.sums[index]
            };
            line_has_content = false;
            continue;
        }
        line_has_content = true;
    }
    for index in perfect_fit_indices.into_iter().rev() {
        for sum in ctx.sums[index..].iter_mut() {
            *sum = sum.saturating_add(1);
        }
        line_count = GreedyLineCount(ctx, ctx.line_break_size);
        if line_count >= min_line_count {
            break;
        }
    }
    line_count
}

// cpp: layoutng_flex/flex_line_breaker.cc:271-307
fn BreakIntoLines(
    all_items: &[FlexItem],
    gap_between_items: LayoutUnit,
    mut should_break: impl FnMut(usize, LayoutUnit) -> bool,
) -> FlexLineBreakerResult {
    let mut result = FlexLineBreakerResult::default();
    let mut offset = 0;
    while offset < all_items.len() {
        let mut sum = LayoutUnit::default();
        let mut count = 0;
        for item in &all_items[offset..] {
            if should_break(count, sum + item.HypotheticalMainAxisMarginBoxSize()) {
                break;
            }
            sum += item.HypotheticalMainAxisMarginBoxSize() + gap_between_items;
            count += 1;
        }
        assert!(count > 0, "flex line must contain an item");
        sum -= gap_between_items;
        result
            .flex_lines
            .push(InitialFlexLine::new(count as WtfSizeT, sum));
        result.max_sum_hypothetical_main_size = result.max_sum_hypothetical_main_size.max(sum);
        offset += count;
    }
    result
}

// cpp: layoutng_flex/flex_line_breaker.cc:311-409
pub fn BalanceBreakFlexItemsIntoLines(
    all_items: &[FlexItem],
    line_break_size: LayoutUnit,
    gap_between_items: LayoutUnit,
    min_line_count: WtfSizeT,
) -> FlexLineBreakerResult {
    let count = all_items.len();
    let min_line_count = (min_line_count as usize).min(count);
    debug_assert!(min_line_count >= 1);
    let mut ctx = ScoreContext {
        gap_between_items: gap_between_items.RawValue() as u64,
        line_break_size: line_break_size.RawValue() as u64,
        sums: vec![0; count],
        scores: vec![ScoreData::default(); count],
    };
    for index in 0..count {
        let item_size = all_items[index]
            .HypotheticalMainAxisMarginBoxSize()
            .ClampNegativeToZero();
        let previous = if index == 0 { 0 } else { ctx.sums[index - 1] };
        ctx.sums[index] =
            previous.saturating_add((item_size + gap_between_items).RawValue() as u64);
    }
    let mut line_count = GreedyLineCount(&ctx, ctx.line_break_size);
    if line_count < min_line_count {
        line_count = ApplyMinLineCount(min_line_count, &mut ctx);
    }
    let mut first = 0;
    for end in 0..count {
        debug_assert!(first <= end);
        let length = |start: usize, end: usize, ctx: &ScoreContext| {
            ctx.sums[end]
                .saturating_sub(if start == 0 { 0 } else { ctx.sums[start - 1] })
                .saturating_sub(ctx.gap_between_items)
        };
        while first < end && length(first, end, &ctx) > ctx.line_break_size {
            first += 1;
        }
        ctx.scores[end].initial_start = first;
        if first == end && length(first, end, &ctx) > ctx.line_break_size {
            first += 1;
        }
    }
    Score(count - 1, INFINITY, &mut ctx);
    let mut item_counts = Vec::with_capacity(line_count);
    let mut previous_index = count - 1;
    let mut index = ctx.scores[previous_index].best_break;
    while index != NOT_FOUND {
        item_counts.push(previous_index - index);
        previous_index = index;
        index = ctx.scores[index].best_break;
    }
    item_counts.push(previous_index + 1);
    debug_assert_eq!(line_count, item_counts.len());
    item_counts.reverse();
    let mut line_index = 0;
    BreakIntoLines(all_items, gap_between_items, |count, _| {
        if count == item_counts[line_index] {
            line_index += 1;
            true
        } else {
            false
        }
    })
}

// cpp: layoutng_flex/flex_line_breaker.cc:411-419
pub fn GreedyBreakFlexItemsIntoLines(
    all_items: &[FlexItem],
    line_break_size: LayoutUnit,
    gap_between_items: LayoutUnit,
    is_multi_line: bool,
) -> FlexLineBreakerResult {
    BreakIntoLines(all_items, gap_between_items, |count, line_size| {
        is_multi_line && count != 0 && line_size > line_break_size
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::WritingMode;
    use layoutng_assembly::internal::baseline_utils::BaselineGroup;
    use layoutng_assembly::internal::block_node::BlockNode;
    use layoutng_assembly::internal::min_max_sizes::MinMaxSizes;
    use layoutng_geometry::geometry::box_strut::{BoxStrut, PhysicalBoxStrut};
    use layoutng_style::style::computed_style_constants::ItemPosition;

    fn item(index: u32, width: i32) -> FlexItem {
        FlexItem::new(
            BlockNode::null(),
            index,
            0.0,
            1.0,
            LayoutUnit::from_signed(width),
            MinMaxSizes {
                min_size: LayoutUnit::default(),
                max_size: LayoutUnit::from_signed(1000),
            },
            LayoutUnit::default(),
            None,
            PhysicalBoxStrut::default(),
            BoxStrut::default(),
            0,
            ItemPosition::kNormal,
            WritingMode::kHorizontalTb,
            BaselineGroup::kMajor,
            false,
            false,
            false,
            true,
        )
    }

    #[test]
    fn greedy_wraps_at_gap_inclusive_boundary() {
        let items = [item(0, 30), item(1, 30), item(2, 30), item(3, 30)];
        let lines = BreakFlexItemsIntoLines(
            &items,
            LayoutUnit::from_signed(70),
            LayoutUnit::from_signed(10),
            true,
            None,
        );
        assert_eq!(lines.flex_lines.len(), 2);
        assert_eq!(lines.flex_lines[0].count, 2);
        assert_eq!(lines.flex_lines[1].count, 2);
        assert_eq!(
            lines.max_sum_hypothetical_main_size,
            LayoutUnit::from_signed(70)
        );
    }

    #[test]
    fn balanced_break_honors_minimum_line_count() {
        let items = [item(0, 20), item(1, 20), item(2, 20), item(3, 20)];
        let lines = BreakFlexItemsIntoLines(
            &items,
            LayoutUnit::from_signed(40),
            LayoutUnit::default(),
            true,
            Some(3),
        );
        assert_eq!(lines.flex_lines.len(), 3);
        assert_eq!(
            lines.flex_lines.iter().map(|line| line.count).sum::<u32>(),
            4
        );
    }
}
