#![allow(non_snake_case)]

use foundation::{DynamicTo, HeapVector, Member, PhysicalOffset, Visitor};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_fragment_tree::physical_fragment::BoxType;

use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::pagination_utils::{GetPageArea, GetPageBorderBox, StitchedPageContentRect};
use super::tree_traversal_utils::{
    ForAllBoxFragmentDescendants, NextStep, PhysicalFragmentTraversalListener,
    PhysicalFragmentTraversalOptions,
};

// cpp: layoutng/internal/layout_box_utils.cc:22-53
struct OutOfFlowDescendant {
    box_: Member<LayoutBox>,
    offset_from_root_fragmentation_context_: PhysicalOffset,
}

impl OutOfFlowDescendant {
    // cpp: layoutng/internal/layout_box_utils.cc:26-32
    fn new(box_: &LayoutBox, offset: PhysicalOffset) -> Self {
        debug_assert!(box_.IsOutOfFlowPositioned());
        Self {
            box_: Member::from_ptr(box_ as *const LayoutBox as *mut LayoutBox),
            offset_from_root_fragmentation_context_: offset,
        }
    }

    // cpp: layoutng/internal/layout_box_utils.cc:34-44
    fn UpdateLocation(&mut self) {
        let box_ = unsafe { &mut *self.box_.Get() };
        let containing_block = box_.ContainingBlock();
        let first_fragment = unsafe { &*(*containing_block).GetPhysicalFragment(0) };
        box_.SetLocation(
            self.offset_from_root_fragmentation_context_
                - first_fragment.OffsetFromRootFragmentationContext(),
        );
    }

    // cpp: layoutng/internal/layout_box_utils.cc:46
    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.box_);
    }
}

type OutOfFlowDescendants = HeapVector<OutOfFlowDescendant>;

// cpp: layoutng/internal/layout_box_utils.cc:55-120
fn SetChildLocation(
    parent: &PhysicalBoxFragment,
    child: &PhysicalBoxFragment,
    child_offset: PhysicalOffset,
    fragmentainer: *const PhysicalBoxFragment,
    fragmentainer_offset: PhysicalOffset,
    out_of_flow: *mut OutOfFlowDescendants,
) {
    if child.IsLayoutObjectDestroyedOrMoved() {
        return;
    }
    if !child.IsFirstForNode() {
        return;
    }
    let child_box = DynamicTo::<LayoutBox>(child.GetMutableLayoutObject());
    assert!(!child_box.is_null());
    if !fragmentainer.is_null() && child.IsOutOfFlowPositioned() {
        assert!(!out_of_flow.is_null());
        unsafe { &mut *out_of_flow }.push_back(OutOfFlowDescendant::new(
            unsafe { &*child_box },
            unsafe { &*fragmentainer }.OffsetFromRootFragmentationContext() + child_offset,
        ));
        return;
    }
    let mut offset = fragmentainer_offset + child_offset;
    if !parent.IsFirstForNode() {
        let parent_box = DynamicTo::<LayoutBox>(parent.GetMutableLayoutObject());
        if !parent_box.is_null() {
            let first = unsafe { &*(*parent_box).GetPhysicalFragment(0) };
            offset += parent.OffsetFromRootFragmentationContext()
                - first.OffsetFromRootFragmentationContext();
        }
    }
    unsafe { &mut *child_box }.SetLocation(offset);
    if !fragmentainer.is_null() {
        unsafe { &mut *child_box }.SetShouldCheckForPaintInvalidation();
    }
}

// cpp: layoutng/internal/layout_box_utils.cc:122-183
fn UpdateBoxChildLocations(parent: &PhysicalBoxFragment, out_of_flow: *mut OutOfFlowDescendants) {
    let items = parent.Items();
    if !items.is_null() {
        let mut cursor = InlineCursor::from_container_and_items(parent, unsafe { &*items });
        while cursor.IsNotNull() {
            let child = cursor.Current().BoxFragment();
            if child.is_null() {
                cursor.MoveToNext();
                continue;
            }
            let model =
                DynamicTo::<LayoutBoxModelObject>(unsafe { &*child }.GetMutableLayoutObject());
            if model.is_null() {
                cursor.MoveToNext();
                continue;
            }
            if !DynamicTo::<LayoutBox>(model).is_null() {
                SetChildLocation(
                    parent,
                    unsafe { &*child },
                    cursor.Current().OffsetInContainerFragment(),
                    std::ptr::null(),
                    PhysicalOffset::default(),
                    out_of_flow,
                );
            }
            cursor.MoveToNext();
        }
    }
    for child in parent.Children() {
        let child_box = DynamicTo::<PhysicalBoxFragment>(child.get());
        if child_box.is_null() {
            continue;
        }
        let child_box = unsafe { &*child_box };
        if child_box.IsColumnBox() {
            for grandchild in child_box.Children() {
                let grandchild_box = DynamicTo::<PhysicalBoxFragment>(grandchild.get());
                assert!(!grandchild_box.is_null());
                SetChildLocation(
                    parent,
                    unsafe { &*grandchild_box },
                    grandchild.offset,
                    child_box,
                    child.offset,
                    out_of_flow,
                );
            }
        } else {
            debug_assert!(!child_box.IsFragmentainerBox());
            let mut page_area = std::ptr::null();
            if parent.IsFragmentainerBox() {
                debug_assert_eq!(parent.GetBoxType(), BoxType::kPageArea);
                page_area = parent;
            }
            SetChildLocation(
                parent,
                child_box,
                child.offset,
                page_area,
                PhysicalOffset::default(),
                out_of_flow,
            );
        }
    }
}

// cpp: layoutng/internal/layout_box_utils.cc:185-195
// cpp: layoutng/internal/layout_box_utils.cc:274-287
struct TraversalListener {
    state_stack_: Vec<State>,
    out_of_flow_: *mut OutOfFlowDescendants,
}

// cpp: layoutng/internal/layout_box_utils.cc:274-283
struct State {
    accumulated_offset: PhysicalOffset,
    force_subtree_update: bool,
}

impl TraversalListener {
    // cpp: layoutng/internal/layout_box_utils.cc:189-195
    fn new(
        root: &PhysicalBoxFragment,
        offset: PhysicalOffset,
        out_of_flow: *mut OutOfFlowDescendants,
    ) -> Self {
        let mut stack = Vec::with_capacity(256);
        stack.push(State {
            accumulated_offset: offset,
            force_subtree_update: Self::ShouldThisForceEntireSubtreeUpdate(root),
        });
        Self {
            state_stack_: stack,
            out_of_flow_: out_of_flow,
        }
    }

    // cpp: layoutng/internal/layout_box_utils.cc:204-216
    fn ShouldThisForceEntireSubtreeUpdate(fragment: &PhysicalBoxFragment) -> bool {
        let box_ = DynamicTo::<LayoutBox>(fragment.GetMutableLayoutObject());
        if !box_.is_null()
            && unsafe { &*box_ }.ShouldCheckForPaintInvalidation()
            && unsafe { &*box_ }.IsFragmentationContextRoot()
        {
            return true;
        }
        false
    }

    // cpp: layoutng/internal/layout_box_utils.cc:266-272
    fn AccumulatedOffset(&self) -> PhysicalOffset {
        self.state_stack_
            .last()
            .expect("root state")
            .accumulated_offset
    }

    fn ShouldForceEntireSubtreeUpdate(&self) -> bool {
        self.state_stack_
            .last()
            .expect("root state")
            .force_subtree_update
    }
}

// cpp: layoutng/internal/layout_box_utils.cc:197-202
impl Drop for TraversalListener {
    fn drop(&mut self) {
        debug_assert_eq!(self.state_stack_.len(), 1);
    }
}

impl PhysicalFragmentTraversalListener for TraversalListener {
    // cpp: layoutng/internal/layout_box_utils.cc:219-250
    fn HandleEntry(
        &mut self,
        fragment: &PhysicalBoxFragment,
        offset: PhysicalOffset,
        _first: bool,
    ) -> NextStep {
        if fragment.IsMonolithic() {
            return NextStep::kSkipChildren;
        }
        let accumulated = self.AccumulatedOffset() + offset;
        fragment
            .GetMutableForContainerLayout()
            .SetOffsetFromRootFragmentationContext(accumulated);
        let object = fragment.GetLayoutObject();
        if !object.is_null() && unsafe { &*object }.ChildLayoutBlockedByDisplayLock() {
            return NextStep::kSkipChildren;
        }
        let force = self.ShouldForceEntireSubtreeUpdate()
            || Self::ShouldThisForceEntireSubtreeUpdate(fragment);
        let update_children = force
            || fragment.IsFragmentainerBox()
            || unsafe { &*object }.ShouldCheckForPaintInvalidation();
        if !update_children {
            return NextStep::kSkipChildren;
        }
        self.state_stack_.push(State {
            accumulated_offset: accumulated,
            force_subtree_update: force,
        });
        NextStep::kContinue
    }

    // cpp: layoutng/internal/layout_box_utils.cc:252-264
    fn HandleExit(&mut self, fragment: &PhysicalBoxFragment, _offset: PhysicalOffset) {
        if !fragment.IsFragmentainerBox() {
            UpdateBoxChildLocations(fragment, self.out_of_flow_);
        }
        self.state_stack_.pop();
    }
}

// cpp: layoutng/internal/layout_box_utils.cc:289-308
fn UpdateOffsetsFromRootFragmentationContext(
    fragment: &PhysicalBoxFragment,
    offset: PhysicalOffset,
) {
    let mut out_of_flow = OutOfFlowDescendants::default();
    let mut listener = TraversalListener::new(fragment, offset, &mut out_of_flow);
    ForAllBoxFragmentDescendants(
        fragment,
        PhysicalFragmentTraversalOptions::kFragmentTraversalOptionNone,
        &mut listener,
    );
    UpdateBoxChildLocations(fragment, &mut out_of_flow);
    for descendant in out_of_flow.iter_mut() {
        descendant.UpdateLocation();
    }
}

// cpp: layoutng/internal/layout_box_utils.h:19
// cpp: layoutng/internal/layout_box_utils.cc:312-365
pub fn UpdateChildLayoutBoxLocations(fragment: &PhysicalBoxFragment) {
    if unsafe { &*fragment.GetLayoutObject() }.ChildLayoutBlockedByDisplayLock() {
        return;
    }
    debug_assert!(fragment.IsOnlyForNode());
    if !fragment.IsFragmentationContextRoot() {
        debug_assert!(fragment.IsOnlyForNode());
        UpdateBoxChildLocations(fragment, std::ptr::null_mut());
        return;
    }
    if !fragment.IsPaginatedRoot() {
        UpdateOffsetsFromRootFragmentationContext(fragment, PhysicalOffset::default());
    } else {
        let mut first_page_area: *const PhysicalBoxFragment = std::ptr::null();
        let mut previous_break_token: *const BlockBreakToken = std::ptr::null();
        for link in fragment.Children() {
            let page_container = unsafe { &*link.get().cast::<PhysicalBoxFragment>() };
            let page_area = GetPageArea(GetPageBorderBox(page_container));
            if first_page_area.is_null() {
                first_page_area = page_area;
            }
            let rect = StitchedPageContentRect(
                page_area,
                unsafe { &*first_page_area },
                previous_break_token,
            );
            page_area
                .GetMutableForContainerLayout()
                .SetOffsetFromRootFragmentationContext(rect.offset);
            UpdateOffsetsFromRootFragmentationContext(page_area, rect.offset);
            previous_break_token = page_area.GetBreakToken();
        }
    }
}
