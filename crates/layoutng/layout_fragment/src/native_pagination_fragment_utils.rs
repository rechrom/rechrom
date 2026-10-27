// Page border-box discovery and stitched page geometry are fragment topology
// utilities. They compile through the reciprocal core facade until the native
// type cycle is removed.

use super::*;

// cpp: layoutng/internal/pagination_utils.h:17-18
// cpp: layoutng/internal/pagination_fragment_utils.cc:13-21
#[inline(never)]
pub fn GetPageBorderBoxLink(page_container: &PhysicalBoxFragment) -> &PhysicalFragmentLink {
    debug_assert_eq!(page_container.GetBoxType(), BoxType::kPageContainer);
    for child in page_container.Children() {
        if child.GetBoxType() == BoxType::kPageBorderBox {
            return child;
        }
    }
    panic!("page container has no page border box")
}

// cpp: layoutng/internal/pagination_utils.h:19-20
// cpp: layoutng/internal/pagination_fragment_utils.cc:23-26
#[inline(never)]
pub fn GetPageBorderBox(page_container: &PhysicalBoxFragment) -> &PhysicalBoxFragment {
    let fragment = GetPageBorderBoxLink(page_container).get();
    assert!(!fragment.is_null());
    debug_assert!(unsafe { &*fragment }.IsBox());
    unsafe { &*fragment.cast::<PhysicalBoxFragment>() }
}

// cpp: layoutng/internal/pagination_utils.h:21-22
// cpp: layoutng/internal/pagination_fragment_utils.cc:28-36
#[inline(never)]
pub fn GetPageArea(page_border_box: &PhysicalBoxFragment) -> &PhysicalBoxFragment {
    debug_assert_eq!(page_border_box.GetBoxType(), BoxType::kPageBorderBox);
    debug_assert_eq!(page_border_box.Children().len(), 1);
    let fragment = page_border_box.Children()[0].get();
    assert!(!fragment.is_null());
    assert!(unsafe { &*fragment }.IsBox());
    let page_area = unsafe { &*fragment.cast::<PhysicalBoxFragment>() };
    debug_assert_eq!(page_area.GetBoxType(), BoxType::kPageArea);
    page_area
}

// cpp: layoutng/internal/pagination_utils.h:24-27
// cpp: layoutng/internal/pagination_fragment_utils.cc:38-64
#[inline(never)]
pub fn StitchedPageContentRect(
    page_area: &PhysicalBoxFragment,
    first_page_area: &PhysicalBoxFragment,
    previous_break_token: *const BlockBreakToken,
) -> PhysicalRect {
    debug_assert_eq!(page_area.GetBoxType(), BoxType::kPageArea);
    debug_assert_eq!(first_page_area.GetBoxType(), BoxType::kPageArea);

    let mut physical_page_rect = page_area.LocalRect();
    if previous_break_token.is_null() {
        return physical_page_rect;
    }

    let consumed_block_size = unsafe { &*previous_break_token }.ConsumedBlockSize();
    let block_end = page_area.Style().GetWritingDirection().BlockEnd();
    if block_end == PhysicalDirection::kLeft {
        physical_page_rect.offset.left += first_page_area.Size().width;
        physical_page_rect.offset.left -= consumed_block_size + page_area.Size().width;
    } else if block_end == PhysicalDirection::kRight {
        physical_page_rect.offset.left += consumed_block_size;
    } else {
        assert_eq!(block_end, PhysicalDirection::kDown);
        physical_page_rect.offset.top += consumed_block_size;
    }
    physical_page_rect
}
