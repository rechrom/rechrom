#![allow(non_snake_case)]

use foundation::{
    kIndefiniteSize, DynamicTo, InfiniteIntRect, LayoutUnit, PhysicalRect, ScopedCSSName,
    UnsupportedLayout,
};
use layout_fragment::inline_cursor::InlineCursor;
#[cfg(debug_assertions)]
use layout_fragment::physical_box_fragment::AllowPostLayoutScope;
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
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

#[path = "../../../layout_fragment/src/native_scrollable_overflow_recalc.rs"]
mod layout_fragment_native_scrollable_overflow_recalc;
pub use layout_fragment_native_scrollable_overflow_recalc::{
    LayoutBoxRecalcChildScrollableOverflowNG, LayoutBoxRecalcScrollableOverflowNG,
};

#[path = "../../../layout_algo_out_of_flow/src/native_anchor_lookup.rs"]
mod layout_algo_out_of_flow_native_anchor_lookup;
pub use layout_algo_out_of_flow_native_anchor_lookup::{
    LayoutBoxAcceptableImplicitAnchor, LayoutBoxFindTargetAnchor,
};

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
