use super::*;

// cpp: layoutng/internal/layout_box_fragment_data.cc:678-763
#[inline(never)]
pub(super) fn LayoutBoxSetScrollableOverflowFromLayoutResults(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    box_.ClearSelfNeedsScrollableOverflowRecalc();
    box_.ClearChildNeedsScrollableOverflowRecalc();
    let overflow = box_.overflow_.Get();
    if !overflow.is_null() {
        unsafe { &mut *overflow }.scrollable_overflow = None;
    }
    if box_.IsLayoutReplaced() {
        return;
    }
    let writing_mode = box_.StyleRef().GetWritingMode();
    let mut scrollable_overflow: Option<PhysicalRect> = None;
    let mut consumed_block_size = LayoutUnit::default();
    let mut fragment_width_sum = LayoutUnit::default();
    for layout_result in box_.layout_results_.iter() {
        let fragment = To::<PhysicalBoxFragment>(
            unsafe { &*layout_result.Get() }.GetPhysicalFragment() as *const _,
        );
        let fragment = unsafe { &*fragment };
        let offset_adjust = match writing_mode {
            WritingMode::kHorizontalTb => {
                PhysicalOffset::new(LayoutUnit::default(), consumed_block_size)
            }
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
                fragment_width_sum += fragment.Size().width;
                PhysicalOffset::new(
                    -fragment.Size().width - consumed_block_size,
                    LayoutUnit::default(),
                )
            }
            WritingMode::kVerticalLr | WritingMode::kSidewaysLr => {
                PhysicalOffset::new(consumed_block_size, LayoutUnit::default())
            }
        };
        let mut fragment_scrollable_overflow = fragment.ScrollableOverflow();
        fragment_scrollable_overflow.offset += offset_adjust;
        if let Some(overflow) = scrollable_overflow.as_mut() {
            overflow.UniteEvenIfEmpty(&fragment_scrollable_overflow);
        } else {
            scrollable_overflow = Some(fragment_scrollable_overflow);
        }
        let break_token = fragment.GetBreakToken();
        if !break_token.is_null() {
            if unsafe { &*break_token }.IsRepeated() {
                break;
            }
            consumed_block_size = unsafe { &*break_token }.ConsumedBlockSize();
        }
    }
    let Some(mut scrollable_overflow) = scrollable_overflow else {
        return;
    };
    if box_.StyleRef().IsFlippedBlocksWritingMode() {
        scrollable_overflow.offset.left += fragment_width_sum;
    }
    if scrollable_overflow.IsEmpty()
        || box_
            .PhysicalPaddingBoxRect()
            .ContainsRect(&scrollable_overflow)
    {
        return;
    }
    debug_assert!(!box_.ScrollableOverflowIsSet());
    if box_.overflow_.Get().is_null() {
        box_.overflow_ = Member::from_ptr(MakeGarbageCollected(BoxOverflowModel::default()));
    }
    unsafe { &mut *box_.overflow_.Get() }.scrollable_overflow =
        Some(BoxScrollableOverflowModel::new(&scrollable_overflow));
}
