use super::*;

// cpp: layoutng/internal/layout_box_fragment_data.cc:578-592
#[inline(never)]
pub(super) fn LayoutBoxLocationContainer(box_: &LayoutBox) -> *mut LayoutBox {
    box_.CheckIsNotDestroyed();
    if box_.IsSVGChild() {
        return std::ptr::null_mut();
    }
    let mut container = box_.Container();
    while !container.is_null() && !unsafe { &*container }.IsBox() {
        container = unsafe { &*container }.Container();
    }
    To::<LayoutBox>(container)
}

// cpp: layoutng/internal/fragmentation_utils.cc:1404-1434
#[inline(never)]
pub(super) fn LayoutBoxFindPreviousBreakToken(
    fragment: &PhysicalBoxFragment,
) -> *const BlockBreakToken {
    let box_ = To::<LayoutBox>(fragment.GetLayoutObject());
    debug_assert!(!box_.is_null());
    let box_ = unsafe { &*box_ };
    debug_assert!(box_.PhysicalFragmentCount() >= 1);
    if fragment.IsFirstForNode() {
        return std::ptr::null();
    }
    debug_assert!(box_.PhysicalFragmentCount() > 1);
    let break_token = fragment.GetBreakToken();
    let previous_fragment = if !break_token.is_null() {
        let sequence_number = unsafe { &*break_token }.SequenceNumber() as usize;
        debug_assert!(sequence_number >= 1);
        box_.GetPhysicalFragment(sequence_number - 1)
    } else {
        box_.GetPhysicalFragment(box_.PhysicalFragmentCount() - 2)
    };
    unsafe { &*previous_fragment }.GetBreakToken()
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:69-90
fn fragment_for_edge(box_: &LayoutBox, edges: PhysicalBoxSides) -> *const PhysicalBoxFragment {
    debug_assert!(box_.PhysicalFragmentCount() > 1);
    debug_assert_eq!(
        u8::from(edges.top) + u8::from(edges.right) + u8::from(edges.bottom) + u8::from(edges.left),
        1
    );
    let logical_edges = edges.ToLogical(box_.StyleRef().GetWritingDirection());
    if logical_edges.block_end {
        for index in (1..box_.PhysicalFragmentCount()).rev() {
            let fragment = box_.GetPhysicalFragment(index);
            let previous_token = LayoutBoxFindPreviousBreakToken(unsafe { &*fragment });
            if !unsafe { &*previous_token }.IsAtBlockEnd() {
                return fragment;
            }
        }
    }
    box_.GetPhysicalFragment(0)
}

fn fragment_for_left_edge(box_: &LayoutBox) -> *const PhysicalBoxFragment {
    fragment_for_edge(box_, PhysicalBoxSides::new(false, false, false, true))
}

fn fragment_for_right_edge(box_: &LayoutBox) -> *const PhysicalBoxFragment {
    fragment_for_edge(box_, PhysicalBoxSides::new(false, true, false, false))
}

fn fragment_for_top_edge(box_: &LayoutBox) -> *const PhysicalBoxFragment {
    fragment_for_edge(box_, PhysicalBoxSides::new(true, false, false, false))
}

fn fragment_for_bottom_edge(box_: &LayoutBox) -> *const PhysicalBoxFragment {
    fragment_for_edge(box_, PhysicalBoxSides::new(false, false, true, false))
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:482-521
#[inline(never)]
pub(super) fn LayoutBoxStitchedBlockSize(box_: &LayoutBox) -> LayoutUnit {
    box_.CheckIsNotDestroyed();
    if box_.PhysicalFragmentCount() == 0 {
        return LayoutUnit::default();
    }
    let writing_direction = box_.StyleRef().GetWritingDirection();
    let first_fragment = unsafe { &*box_.GetPhysicalFragment(0) };
    let first_break_token = first_fragment.GetBreakToken();
    if first_fragment.IsOnlyForNode()
        || (!first_break_token.is_null() && unsafe { &*first_break_token }.IsRepeated())
    {
        return LogicalFragment::new(writing_direction, first_fragment).BlockSize();
    }
    let mut index = box_.PhysicalFragmentCount() - 1;
    debug_assert!(index >= 1);
    let mut last_content_fragment_size = unsafe { &*box_.GetPhysicalFragment(index) }.Size();
    let mut previously_consumed_block_size = LayoutUnit::default();
    while index != 0 {
        index -= 1;
        let break_token = unsafe { &*box_.GetPhysicalFragment(index) }.GetBreakToken();
        if !unsafe { &*break_token }.IsAtBlockEnd() {
            previously_consumed_block_size = unsafe { &*break_token }.ConsumedBlockSize();
            break;
        }
        last_content_fragment_size = unsafe { &*box_.GetPhysicalFragment(index) }.Size();
    }
    let logical_size = ToLogicalSize(last_content_fragment_size, box_.StyleRef().GetWritingMode());
    previously_consumed_block_size + logical_size.block_size
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:593-640
#[inline(never)]
pub(super) fn LayoutBoxPhysicalContractedBoxRect(
    box_: &LayoutBox,
    edge: ContractionEdge,
) -> PhysicalRect {
    box_.CheckIsNotDestroyed();
    let mut rect = PhysicalRect::new(PhysicalOffset::default(), box_.StitchedSize());
    let mut inset = PhysicalBoxStrut::default();
    if box_.PhysicalFragmentCount() == 1 {
        let fragment = unsafe { &*box_.GetPhysicalFragment(0) };
        if fragment.HasBorders() {
            inset += fragment.Borders();
        }
        if fragment.HasScrollbar() {
            inset += fragment.Scrollbar();
        }
        if edge == ContractionEdge::kContractToContentEdge && fragment.HasPadding() {
            inset += fragment.Padding();
        }
    } else if box_.PhysicalFragmentCount() != 0 {
        let top_fragment = unsafe { &*fragment_for_top_edge(box_) };
        let right_fragment = unsafe { &*fragment_for_right_edge(box_) };
        let bottom_fragment = unsafe { &*fragment_for_bottom_edge(box_) };
        let left_fragment = unsafe { &*fragment_for_left_edge(box_) };
        inset.top += top_fragment.Borders().top + top_fragment.Scrollbar().top;
        inset.right += right_fragment.Borders().right + right_fragment.Scrollbar().right;
        inset.bottom += bottom_fragment.Borders().bottom + bottom_fragment.Scrollbar().bottom;
        inset.left += left_fragment.Borders().left + left_fragment.Scrollbar().left;
        if edge == ContractionEdge::kContractToContentEdge {
            inset.top += top_fragment.Padding().top;
            inset.right += right_fragment.Padding().right;
            inset.bottom += bottom_fragment.Padding().bottom;
            inset.left += left_fragment.Padding().left;
        }
    }
    rect.Contract(&inset);
    rect.size.width = rect.size.width.ClampNegativeToZero();
    rect.size.height = rect.size.height.ClampNegativeToZero();
    rect
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:533-576
#[inline(never)]
pub(super) fn LayoutBoxComputeSize(box_: &LayoutBox) -> PhysicalSize {
    box_.CheckIsNotDestroyed();
    let results = box_.GetLayoutResults();
    let Some(first_result) = results.iter().next() else {
        return PhysicalSize::default();
    };
    let first_fragment = unsafe { &*first_result.Get() }.GetPhysicalFragment();
    if results.len() == 1 {
        return first_fragment.Size();
    }
    let converter =
        WritingModeConverter::without_outer_size(first_fragment.Style().GetWritingDirection());
    let mut previous_break_token: *const BlockBreakToken = std::ptr::null();
    let mut size = LogicalSize::default();
    for result in results {
        let fragment = unsafe { &*result.Get() }.GetPhysicalFragment();
        debug_assert!(fragment.IsBox());
        let physical_fragment = unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) };
        let fragment_logical_size = converter.ToLogicalSize(physical_fragment.Size());
        if physical_fragment.IsFirstForNode() {
            size = fragment_logical_size;
        } else {
            debug_assert!(!previous_break_token.is_null());
            size.block_size = fragment_logical_size.block_size
                + unsafe { &*previous_break_token }.ConsumedBlockSize();
        }
        previous_break_token = physical_fragment.GetBreakToken();
        if previous_break_token.is_null()
            || unsafe { &*previous_break_token }.IsRepeated()
            || unsafe { &*previous_break_token }.IsAtBlockEnd()
        {
            break;
        }
    }
    converter.ToPhysicalSize(size)
}

// cpp: layoutng/internal/layout_box_geometry.cc:576-601
#[inline(never)]
pub(super) fn LayoutBoxBoundingBoxRelativeToFirstFragment(box_: &LayoutBox) -> PhysicalRect {
    box_.CheckIsNotDestroyed();
    let mut bounding_rect = PhysicalRect::default();
    let mut first_fragment: *const PhysicalBoxFragment = std::ptr::null();
    for fragment in box_.PhysicalFragments() {
        let offset = if first_fragment.is_null() {
            first_fragment = fragment;
            PhysicalOffset::default()
        } else {
            fragment.OffsetFromRootFragmentationContext()
                - unsafe { &*first_fragment }.OffsetFromRootFragmentationContext()
        };
        let fragment_rect = PhysicalRect::new(offset, fragment.Size());
        bounding_rect.UniteEvenIfEmpty(&fragment_rect);
        let break_token = fragment.GetBreakToken();
        if !break_token.is_null() && unsafe { &*break_token }.IsAtBlockEnd() {
            break;
        }
    }
    bounding_rect
}

// cpp: layoutng/internal/layout_box_geometry.cc:603-623
#[inline(never)]
pub(super) fn LayoutBoxQuadsInAncestorInternal(
    box_: &LayoutBox,
    quads: &mut Vector<gfx::QuadF>,
    ancestor: *const LayoutBoxModelObject,
    mode: MapCoordinatesFlags,
    box_type: BoxQuadType,
) {
    box_.CheckIsNotDestroyed();
    // The LayoutBox storage used by SVG containers must not turn a resource
    // definition into a CSS geometry box.
    if box_.IsSVGHiddenContainer() {
        return;
    }
    let mut first_fragment: Option<&PhysicalBoxFragment> = None;
    for fragment in box_.PhysicalFragments() {
        let offset = if let Some(first) = first_fragment {
            fragment.OffsetFromRootFragmentationContext()
                - first.OffsetFromRootFragmentationContext()
        } else {
            first_fragment = Some(fragment);
            PhysicalOffset::default()
        };
        let mut rect = LocalRectForBoxQuad(fragment, box_type);
        rect.offset += offset;
        quads.push(box_.LocalRectToAncestorQuad(&rect, ancestor, mode));
    }
}

// cpp: layoutng/internal/layout_box_geometry.cc:665-674
#[inline(never)]
pub(super) fn LayoutBoxResolvedDirection(box_: &LayoutBox) -> TextDirection {
    box_.CheckIsNotDestroyed();
    if box_.IsInLayoutNGInlineFormattingContext() && box_.IsInline() {
        let mut cursor = InlineCursor::default();
        cursor.MoveToLayoutObject(box_);
        if !cursor.IsNull() {
            return cursor.Current().ResolvedDirection();
        }
    }
    box_.StyleRef().Direction()
}
