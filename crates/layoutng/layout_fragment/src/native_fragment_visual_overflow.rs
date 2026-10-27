use super::*;

// cpp: layoutng/internal/layout_box_visual_overflow.cc:166-239
#[inline(never)]
pub(super) fn LayoutBoxCopyVisualOverflowFromFragmentsWithoutInvalidations(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    debug_assert!(box_.CanUseFragmentsForVisualOverflow());
    let fragment_count = box_.PhysicalFragmentCount();
    if fragment_count == 0 {
        debug_assert!(box_.IsLayoutTableCol());
        box_.ClearVisualOverflow();
        return;
    }
    if fragment_count == 1 {
        let fragment = unsafe { &*box_.GetPhysicalFragment(0) };
        debug_assert!(fragment.CanUseFragmentsForInkOverflow());
        if !fragment.HasInkOverflow() {
            box_.ClearVisualOverflow();
            return;
        }
        box_.SetVisualOverflow(
            &fragment.SelfInkOverflowRect(),
            &fragment.ContentsInkOverflowRect(),
        );
        return;
    }

    let containing_block = box_.ContainingBlock();
    debug_assert!(!containing_block.is_null());
    let writing_mode = unsafe { &*containing_block }.StyleRef().GetWritingMode();
    let mut has_overflow = false;
    let mut self_rect = PhysicalRect::default();
    let mut contents_rect = PhysicalRect::default();
    let mut last_fragment: *const PhysicalBoxFragment = std::ptr::null();
    for index in 0..fragment_count {
        let fragment = unsafe { &*box_.GetPhysicalFragment(index) };
        debug_assert!(fragment.CanUseFragmentsForInkOverflow());
        if !fragment.HasInkOverflow() {
            last_fragment = fragment;
            continue;
        }
        has_overflow = true;
        let mut fragment_self_rect = fragment.SelfInkOverflowRect();
        let mut fragment_contents_rect = fragment.ContentsInkOverflowRect();
        if !last_fragment.is_null() {
            let break_token = unsafe { &*last_fragment }.GetBreakToken();
            debug_assert!(!break_token.is_null());
            let block_offset = unsafe { &*break_token }.ConsumedBlockSize();
            if IsHorizontalWritingMode(writing_mode) {
                fragment_self_rect.offset.top += block_offset;
                fragment_contents_rect.offset.top += block_offset;
            } else {
                fragment_self_rect.offset.left += block_offset;
                fragment_contents_rect.offset.left += block_offset;
            }
        }
        last_fragment = fragment;
        self_rect.Unite(&fragment_self_rect);
        contents_rect.Unite(&fragment_contents_rect);
        let break_token = fragment.GetBreakToken();
        if !break_token.is_null() && unsafe { &*break_token }.IsRepeated() {
            break;
        }
    }
    if !has_overflow {
        box_.ClearVisualOverflow();
        return;
    }
    box_.SetVisualOverflow(&self_rect, &contents_rect);
}
