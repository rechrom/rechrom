#![allow(non_snake_case)]

use foundation::{
    kIndefiniteSize, DynamicTo, InfiniteIntRect, LayoutUnit, PhysicalRect, ScopedCSSName,
    UnsupportedLayout,
};
use layoutng_fragment_tree::inline_cursor::InlineCursor;
#[cfg(debug_assertions)]
use layoutng_fragment_tree::physical_box_fragment::AllowPostLayoutScope;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::overflow_clip_axes::{
    kNoOverflowClip, kOverflowClipBothAxis, kOverflowClipX, kOverflowClipY, OverflowClipAxes,
};
use layoutng_style::style::appearance::AppearanceValue;

use super::anchor_map::AnchorMap;
use super::anchor_scope::ToAnchorScopedName;
use super::layout_block::LayoutBlock;
use super::layout_box::LayoutBox;
use super::layout_inline::LayoutInline;
use super::layout_node_metadata::Element;
use super::layout_object::{LayoutObject, RecalcScrollableOverflowResult};
use super::layout_pass_scope::LayoutPassScope;
use super::scrollable_overflow_calculator::ScrollableOverflowCalculator;

// This implementation file supplies methods of the pending LayoutBox and
// LayoutObject headers. Their owning structs can forward to these functions.

// cpp: layoutng/internal/layout_box_core_services.cc:23-35
pub fn ApplyVisibleOverflowToClipRect(
    overflow_clip: OverflowClipAxes,
    clip_rect: &mut PhysicalRect,
) {
    debug_assert_ne!(overflow_clip, kOverflowClipBothAxis);
    let infinite_rect = InfiniteIntRect();
    if overflow_clip & kOverflowClipX == kNoOverflowClip {
        clip_rect.offset.left = LayoutUnit::from_signed(infinite_rect.x());
        clip_rect.size.width = LayoutUnit::from_signed(infinite_rect.width());
    }
    if overflow_clip & kOverflowClipY == kNoOverflowClip {
        clip_rect.offset.top = LayoutUnit::from_signed(infinite_rect.y());
        clip_rect.size.height = LayoutUnit::from_signed(infinite_rect.height());
    }
}

// cpp: layoutng/internal/layout_box_core_services.cc:39-52
fn RecalcFragmentChildren(
    result: &mut RecalcScrollableOverflowResult,
    fragment: &PhysicalBoxFragment,
) {
    for child in fragment.PostLayoutChildren().iter() {
        let child_fragment = child.get();
        if !unsafe { &*child_fragment }.GetLayoutObject().is_null() {
            let box_fragment = DynamicTo::<PhysicalBoxFragment>(child_fragment);
            if !box_fragment.is_null() {
                let owner = unsafe { &*box_fragment }.MutableOwnerLayoutBox();
                if !owner.is_null() {
                    let child_result = unsafe { &mut *owner }.RecalcScrollableOverflow();
                    result.Unite(&child_result);
                }
            }
        } else {
            let fragmentainer = DynamicTo::<PhysicalBoxFragment>(child_fragment);
            if !fragmentainer.is_null() {
                RecalcFragmentChildren(result, unsafe { &*fragmentainer });
            }
        }
    }
}

// cpp: layoutng/internal/layout_box_core_services.cc:54-79
fn ForEachAnchorMapOnContainer(box_: &LayoutBox, mut function: impl FnMut(&AnchorMap)) {
    let container = box_.Container();
    if container.is_null() {
        return;
    }
    if unsafe { &*container }.IsLayoutBlock() {
        let block = DynamicTo::<LayoutBlock>(container);
        assert!(!block.is_null());
        for fragment in unsafe { &*block }.PhysicalFragments() {
            let map = fragment.GetAnchorMap();
            if !map.is_null() {
                function(unsafe { &*map });
            }
        }
        return;
    }
    assert!(unsafe { &*container }.IsLayoutInline());
    let inline = DynamicTo::<LayoutInline>(container);
    assert!(!inline.is_null());
    if !unsafe { &*inline }.HasInlineFragments() {
        return;
    }
    let mut cursor = InlineCursor::default();
    cursor.MoveToLayoutObject(unsafe { &*container });
    while cursor.IsNotNull() {
        let fragment = cursor.Current().BoxFragment();
        if !fragment.is_null() {
            let map = unsafe { &*fragment }.GetAnchorMap();
            if !map.is_null() {
                function(unsafe { &*map });
            }
        }
        cursor.MoveToNextForSameLayoutObject();
    }
}

// cpp: layoutng/internal/layout_box_core_services.cc:83-94
pub fn LayoutBoxDefaultIntrinsicContentInlineSize(box_: &LayoutBox) -> LayoutUnit {
    box_.CheckIsNotDestroyed();
    let algorithms = LayoutPassScope::Algorithms();
    if !algorithms.is_null() {
        if let Some(intrinsic_size) = unsafe { &*algorithms }.forms_support.intrinsic_inline_size {
            return intrinsic_size(box_);
        }
    }
    if matches!(
        box_.StyleRef().EffectiveAppearance(),
        AppearanceValue::kCheckbox | AppearanceValue::kRadio
    ) {
        std::panic::panic_any(UnsupportedLayout::new(
            "forms layout module is not installed",
        ));
    }
    kIndefiniteSize
}

// cpp: layoutng/internal/layout_box_core_services.cc:96-109
pub fn LayoutBoxDefaultIntrinsicContentBlockSize(
    box_: &LayoutBox,
    children_have_geometry: bool,
) -> LayoutUnit {
    box_.CheckIsNotDestroyed();
    let algorithms = LayoutPassScope::Algorithms();
    if !algorithms.is_null() {
        if let Some(intrinsic_size) = unsafe { &*algorithms }.forms_support.intrinsic_block_size {
            return intrinsic_size(box_, children_have_geometry);
        }
    }
    if matches!(
        box_.StyleRef().EffectiveAppearance(),
        AppearanceValue::kCheckbox | AppearanceValue::kRadio
    ) {
        std::panic::panic_any(UnsupportedLayout::new(
            "forms layout module is not installed",
        ));
    }
    kIndefiniteSize
}

// cpp: layoutng/internal/layout_box_core_services.cc:111-115
pub fn LayoutBoxGetScrollMarkerGroup(box_: &mut LayoutBox) -> *mut LayoutBlock {
    box_.CheckIsNotDestroyed();
    std::ptr::null_mut()
}

// cpp: layoutng/internal/layout_box_core_services.cc:117-134
pub fn LayoutBoxFindTargetAnchor(box_: &LayoutBox, name: &ScopedCSSName) -> *const LayoutObject {
    box_.CheckIsNotDestroyed();
    if !box_.IsOutOfFlowPositioned() {
        return std::ptr::null();
    }
    let scoped_name = ToAnchorScopedName(name, unsafe {
        &*(box_ as *const LayoutBox as *const LayoutObject)
    });
    let mut anchor: *const LayoutObject = std::ptr::null();
    ForEachAnchorMapOnContainer(box_, |map| {
        let candidate = map.AnchorLayoutObject(box_, scoped_name);
        if !candidate.is_null()
            && (anchor.is_null()
                || (anchor != candidate
                    && unsafe { &*anchor }.IsBeforeInPreOrderDefault(unsafe { &*candidate })))
        {
            anchor = candidate;
        }
    });
    anchor
}

// cpp: layoutng/internal/layout_box_core_services.cc:136-151
pub fn LayoutBoxAcceptableImplicitAnchor(box_: &LayoutBox) -> *const LayoutObject {
    box_.CheckIsNotDestroyed();
    if !box_.IsOutOfFlowPositioned() {
        return std::ptr::null();
    }
    let element = DynamicTo::<Element>(box_.GetNode());
    let anchor_element = if element.is_null() {
        std::ptr::null_mut()
    } else {
        unsafe { &*element }.container.node.ImplicitAnchorElement()
    };
    let anchor = if anchor_element.is_null() {
        std::ptr::null_mut()
    } else {
        unsafe { &*anchor_element }.container.node.GetLayoutObject()
    };
    if anchor.is_null() {
        return std::ptr::null();
    }
    let mut acceptable = false;
    ForEachAnchorMapOnContainer(box_, |map| {
        acceptable |= !map.AnchorLayoutObject(box_, anchor_element).is_null();
    });
    if acceptable {
        anchor
    } else {
        std::ptr::null()
    }
}

// cpp: layoutng/internal/layout_box_core_services.cc:153-189
pub fn LayoutBoxRecalcScrollableOverflowNG(box_: &mut LayoutBox) -> RecalcScrollableOverflowResult {
    box_.CheckIsNotDestroyed();
    if box_.NeedsLayout() {
        return RecalcScrollableOverflowResult::default();
    }
    let mut child_result = RecalcScrollableOverflowResult::default();
    if box_.ChildNeedsScrollableOverflowRecalc() {
        child_result = box_.RecalcChildScrollableOverflowNG();
    }
    let recalculate =
        box_.SelfNeedsScrollableOverflowRecalc() || child_result.scrollable_overflow_changed;
    let mut rebuild = child_result.rebuild_fragment_tree;
    let mut changed = false;
    if rebuild || recalculate {
        for result in box_.GetLayoutResults() {
            let result = unsafe { &*result.Get() };
            let fragment = result.GetPhysicalFragment();
            let fragment = unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) };
            if recalculate {
                let old_overflow = fragment.ScrollableOverflow();
                let fragmented = result
                    .GetConstraintSpaceForCaching()
                    .HasBlockFragmentation();
                let new_overflow =
                    ScrollableOverflowCalculator::RecalculateScrollableOverflowForFragment(
                        fragment, fragmented,
                    );
                if old_overflow != new_overflow {
                    fragment
                        .GetMutableForStyleRecalc()
                        .SetScrollableOverflow(new_overflow);
                    changed = true;
                    rebuild = true;
                }
            }
        }
        box_.SetScrollableOverflowFromLayoutResults();
    }
    changed =
        changed && !box_.ShouldApplyLayoutContainment() && !box_.ShouldClipOverflowAlongBothAxis();
    RecalcScrollableOverflowResult {
        scrollable_overflow_changed: changed,
        rebuild_fragment_tree: rebuild,
    }
}

// cpp: layoutng/internal/layout_box_core_services.cc:191-215
pub fn LayoutBoxRecalcChildScrollableOverflowNG(
    box_: &mut LayoutBox,
) -> RecalcScrollableOverflowResult {
    box_.CheckIsNotDestroyed();
    debug_assert!(box_.ChildNeedsScrollableOverflowRecalc());
    box_.ClearChildNeedsScrollableOverflowRecalc();
    #[cfg(debug_assertions)]
    let _scope = AllowPostLayoutScope::new();
    let mut result = RecalcScrollableOverflowResult::default();
    for layout_result in box_.GetLayoutResults() {
        let layout_result = unsafe { &*layout_result.Get() };
        let fragment = layout_result.GetPhysicalFragment();
        let fragment = unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) };
        if fragment.HasItems() {
            let mut cursor = InlineCursor::from_fragment(fragment);
            while cursor.IsNotNull() {
                let item = cursor.Current().Item();
                let child = unsafe { &*item }.PostLayoutBoxFragment();
                if !child.is_null() && unsafe { &*(*child).GetLayoutObject() }.IsBox() {
                    let owner = unsafe { &*child }.MutableOwnerLayoutBox();
                    let child_result = unsafe { &mut *owner }.RecalcScrollableOverflow();
                    result.Unite(&child_result);
                }
                cursor.MoveToNext();
            }
        }
        RecalcFragmentChildren(&mut result, fragment);
    }
    result
}
