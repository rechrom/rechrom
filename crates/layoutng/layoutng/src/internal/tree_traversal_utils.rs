#![allow(non_snake_case, non_camel_case_types)]

use foundation::{DynamicTo, HeapHashSet, Member, PhysicalOffset, To};
use layoutng_fragment_tree::fragment_item::ItemType;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;

use super::fragmentation_utils::BoxFragmentIndex;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_inline::LayoutInline;
use super::layout_object::LayoutObject;

// cpp: layoutng/internal/tree_traversal_utils.h:15-23
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicalFragmentTraversalOptions {
    kFragmentTraversalOptionNone = 0,
    kFragmentTraversalOptionCulledInlines = 1,
}

// cpp: layoutng/internal/tree_traversal_utils.h:27-34
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NextStep {
    kContinue,
    kSkipChildren,
}

// cpp: layoutng/internal/tree_traversal_utils.h:25-49
pub trait PhysicalFragmentTraversalListener {
    // cpp: layoutng/internal/tree_traversal_utils.h:36-41
    fn HandleEntry(
        &mut self,
        _fragment: &PhysicalBoxFragment,
        _offset: PhysicalOffset,
        _is_first_for_node: bool,
    ) -> NextStep {
        NextStep::kContinue
    }

    // cpp: layoutng/internal/tree_traversal_utils.h:43-44
    fn HandleExit(&mut self, _fragment: &PhysicalBoxFragment, _offset: PhysicalOffset) {}

    // cpp: layoutng/internal/tree_traversal_utils.h:46-48
    fn HandleCulledInline(&mut self, _culled_inline: &LayoutInline, _is_first_for_node: bool) {}
}

// cpp: layoutng/internal/tree_traversal_utils.cc:19-32
fn HandleBoxFragment(
    fragment: &PhysicalBoxFragment,
    offset: PhysicalOffset,
    is_first_for_node: bool,
    options: PhysicalFragmentTraversalOptions,
    listener: &mut dyn PhysicalFragmentTraversalListener,
) -> NextStep {
    let next_step = listener.HandleEntry(fragment, offset, is_first_for_node);
    if next_step != NextStep::kSkipChildren {
        ForAllBoxFragmentDescendants(fragment, options, listener);
        listener.HandleExit(fragment, offset);
    }
    next_step
}

// cpp: layoutng/internal/tree_traversal_utils.h:60-62
// cpp: layoutng/internal/tree_traversal_utils.cc:36-106
pub fn ForAllBoxFragmentDescendants(
    fragment: &PhysicalBoxFragment,
    options: PhysicalFragmentTraversalOptions,
    listener: &mut dyn PhysicalFragmentTraversalListener,
) {
    for child in fragment.Children() {
        let child_box_fragment = DynamicTo::<PhysicalBoxFragment>(child.get());
        if !child_box_fragment.is_null() {
            let child_box_fragment = unsafe { &*child_box_fragment };
            HandleBoxFragment(
                child_box_fragment,
                child.offset,
                child_box_fragment.IsFirstForNode(),
                options,
                listener,
            );
        }
    }

    if !fragment.HasItems() {
        return;
    }
    let mut culled_inlines: HeapHashSet<Member<LayoutInline>> = HeapHashSet::default();
    let container = To::<LayoutBlockFlow>(fragment.GetLayoutObject());
    debug_assert!(!container.is_null());
    let mut cursor =
        InlineCursor::from_container_and_items(fragment, unsafe { &*fragment.Items() });
    while cursor.IsNotNull() {
        let item = cursor.Current().Item();
        debug_assert!(!item.is_null());
        if unsafe { &*item }.Type() == ItemType::kLine {
            cursor.MoveToNext();
            continue;
        }

        let child_box_fragment = cursor.Current().BoxFragment();
        if !child_box_fragment.is_null() {
            let child_box_fragment = unsafe { &*child_box_fragment };
            let item = unsafe { &*item };
            let next_step = HandleBoxFragment(
                child_box_fragment,
                *item.OffsetInContainerFragment(),
                item.IsFirstForNode(),
                options,
                listener,
            );
            if next_step == NextStep::kContinue && child_box_fragment.IsInlineBox() {
                cursor.MoveToNext();
            } else {
                cursor.MoveToNextSkippingChildren();
            }
            continue;
        }

        if (options as u32)
            & (PhysicalFragmentTraversalOptions::kFragmentTraversalOptionCulledInlines as u32)
            != 0
        {
            let descendant = cursor.Current().GetLayoutObject();
            if !descendant.is_null() {
                debug_assert_ne!(descendant, container as *const LayoutObject);
                let mut walker = unsafe { &*descendant }.Parent();
                while walker != container as *mut LayoutObject {
                    debug_assert!(!walker.is_null());
                    let layout_inline = DynamicTo::<LayoutInline>(walker);
                    if !layout_inline.is_null() && !unsafe { &*layout_inline }.HasInlineFragments()
                    {
                        let is_new_entry = culled_inlines
                            .insert(Member::from_ptr(layout_inline as *mut LayoutInline));
                        if is_new_entry {
                            let mut culled_cursor =
                                InlineCursor::from_container(unsafe { &*container });
                            culled_cursor.MoveToIncludingCulledInline(unsafe {
                                &*(layout_inline as *const LayoutObject)
                            });
                            let is_first_for_node =
                                BoxFragmentIndex(culled_cursor.ContainerFragment())
                                    == BoxFragmentIndex(fragment);
                            listener
                                .HandleCulledInline(unsafe { &*layout_inline }, is_first_for_node);
                        }
                    }
                    walker = unsafe { &*walker }.Parent();
                }
            }
        }
        cursor.MoveToNextSkippingChildren();
    }
}
