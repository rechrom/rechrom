#![allow(non_snake_case, non_camel_case_types)]

use foundation::{kIndefiniteSize, DynamicTo, LayoutUnit, RuntimeEnabledFeatures};

use super::layout_box::LayoutBox;
use super::layout_node_metadata::Element;
use super::layout_object::OverflowRecalcType;

#[path = "../../../layout_commit/src/native_layout_box_post_layout_commit.rs"]
mod layout_commit_native_layout_box_post_layout_commit;

// cpp: layoutng/internal/layout_box.h:75-80
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShouldClampToContentBox {
    kDoNotClampToContentBox,
    kClampToContentBox,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShouldIncludeScrollbarGutter {
    kExcludeScrollbarGutter,
    kIncludeScrollbarGutter,
}

impl LayoutBox {
    // cpp: layoutng/internal/layout_box_layout.cc:130-166
    pub fn OverrideIntrinsicContentInlineSize(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        if !self.ShouldApplyInlineSizeContainment() {
            return kIndefiniteSize;
        }
        let style = self.StyleRef();
        let intrinsic_length = style.EffectiveContainIntrinsicInlineSize();
        if intrinsic_length.HasAuto() {
            let element = DynamicTo::<Element>(self.GetNode());
            let is_locked =
                !element.is_null() && unsafe { &*element }.InputContentLock().unwrap_or(false);
            let is_vt_scope = style.HasSizeContainmentForViewTransitionScope()
                && RuntimeEnabledFeatures::ScopedViewTransitionSizeContainmentEnabled();
            if (is_locked || is_vt_scope) && !element.is_null() {
                if let Some(inline_size) = unsafe { &*element }.LastRememberedInlineSize() {
                    return LayoutUnit::FromFloatRound(
                        inline_size.ToFloat() * style.EffectiveZoom(),
                    );
                }
            }
        }
        if let Some(length) = intrinsic_length.GetLength() {
            debug_assert!(length.IsFixed());
            return LayoutUnit::from_f32(length.Pixels());
        }
        kIndefiniteSize
    }

    // cpp: layoutng/internal/layout_box_layout.cc:168-204
    pub fn OverrideIntrinsicContentBlockSize(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        if !self.ShouldApplyBlockSizeContainment() {
            return kIndefiniteSize;
        }
        let style = self.StyleRef();
        let intrinsic_length = style.EffectiveContainIntrinsicBlockSize();
        if intrinsic_length.HasAuto() {
            let element = DynamicTo::<Element>(self.GetNode());
            let is_locked =
                !element.is_null() && unsafe { &*element }.InputContentLock().unwrap_or(false);
            let is_vt_scope = style.HasSizeContainmentForViewTransitionScope()
                && RuntimeEnabledFeatures::ScopedViewTransitionSizeContainmentEnabled();
            if (is_locked || is_vt_scope) && !element.is_null() {
                if let Some(block_size) = unsafe { &*element }.LastRememberedBlockSize() {
                    return LayoutUnit::FromFloatRound(
                        block_size.ToFloat() * style.EffectiveZoom(),
                    );
                }
            }
        }
        if let Some(length) = intrinsic_length.GetLength() {
            debug_assert!(length.IsFixed());
            return LayoutUnit::from_f32(length.Pixels());
        }
        kIndefiniteSize
    }
}
