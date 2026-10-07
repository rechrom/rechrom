#![allow(non_snake_case, non_camel_case_types)]

use foundation::{
    kIndefiniteSize, DynamicTo, LayoutUnit, OverlayScrollbarClipBehavior, RuntimeEnabledFeatures,
};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;

use super::layout_box::LayoutBox;
use super::layout_node_metadata::Element;
use super::layout_object::OverflowRecalcType;
use super::scrollbar_orientation::ScrollbarOrientation;
use super::scrollbar_theme_metrics::ScrollbarThemeThickness;

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

// cpp: layoutng/internal/layout_box_layout.cc:41-50
fn HypotheticalScrollbarThickness(box_: &LayoutBox, _orientation: ScrollbarOrientation) -> i32 {
    let element = DynamicTo::<Element>(box_.GetNode());
    assert!(!box_
        .StyleRef()
        .HasCustomScrollbarStyle(element as *mut layoutng_style::style::forward::Element));
    ScrollbarThemeThickness(
        box_.ScrollbarThemeForLayout(),
        box_.StyleRef().UsedScrollbarWidth(),
    )
}

impl LayoutBox {
    // cpp: layoutng/internal/layout_box_layout.cc:29-37
    pub fn CanSkipComputeScrollbars(&self) -> bool {
        self.CheckIsNotDestroyed();
        (self.StyleRef().IsOverflowVisibleAlongBothAxes() || !self.HasNonVisibleOverflow() || {
            let scrollable_area = self.GetScrollableArea();
            !scrollable_area.is_null()
                && !unsafe { &*scrollable_area }.HasHorizontalScrollbar()
                && !unsafe { &*scrollable_area }.HasVerticalScrollbar()
        }) && self.StyleRef().IsScrollbarGutterAuto()
    }

    // cpp: layoutng/internal/layout_box_layout.cc:54-114
    pub fn ComputeScrollbarsInternal(
        &self,
        clamp_to_content_box: ShouldClampToContentBox,
        overlay_scrollbar_clip_behavior: OverlayScrollbarClipBehavior,
        include_scrollbar_gutter: ShouldIncludeScrollbarGutter,
    ) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        let mut scrollbars = PhysicalBoxStrut::default();
        let scrollable_area = self.GetScrollableArea();

        if include_scrollbar_gutter == ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter
            && self.HasScrollbarGutters(ScrollbarOrientation::kVerticalScrollbar)
        {
            let gutter_size = LayoutUnit::from_signed(HypotheticalScrollbarThickness(
                self,
                ScrollbarOrientation::kVerticalScrollbar,
            ));
            if self.ShouldPlaceVerticalScrollbarOnLeft() {
                scrollbars.left = gutter_size;
                if self.StyleRef().IsScrollbarGutterBothEdges() {
                    scrollbars.right = gutter_size;
                }
            } else {
                scrollbars.right = gutter_size;
                if self.StyleRef().IsScrollbarGutterBothEdges() {
                    scrollbars.left = gutter_size;
                }
            }
        } else if !scrollable_area.is_null() {
            let width = unsafe { &*scrollable_area }
                .VerticalScrollbarWidth(overlay_scrollbar_clip_behavior);
            if self.ShouldPlaceVerticalScrollbarOnLeft() {
                scrollbars.left = LayoutUnit::from_signed(width);
            } else {
                scrollbars.right = LayoutUnit::from_signed(width);
            }
        }

        if include_scrollbar_gutter == ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter
            && self.HasScrollbarGutters(ScrollbarOrientation::kHorizontalScrollbar)
        {
            let gutter_size = LayoutUnit::from_signed(HypotheticalScrollbarThickness(
                self,
                ScrollbarOrientation::kHorizontalScrollbar,
            ));
            scrollbars.bottom = gutter_size;
            if self.StyleRef().IsScrollbarGutterBothEdges() {
                scrollbars.top = gutter_size;
            }
        } else if !scrollable_area.is_null() {
            scrollbars.bottom = LayoutUnit::from_signed(
                unsafe { &*scrollable_area }
                    .HorizontalScrollbarHeight(overlay_scrollbar_clip_behavior),
            );
        }

        if scrollbars.left > LayoutUnit::default()
            && clamp_to_content_box == ShouldClampToContentBox::kClampToContentBox
        {
            let max_width = self.StitchedSize().width
                - (self.BorderOutsets() + self.PaddingOutsets()).HorizontalSum();
            scrollbars.left = std::cmp::min(scrollbars.left, max_width.ClampNegativeToZero());
        }
        scrollbars
    }

    // cpp: layoutng/internal/layout_box_layout.cc:116-128
    pub fn UpdateAfterLayout(&mut self) {
        self.CheckIsNotDestroyed();
        self.SetNeedsOverflowRecalcWithType(OverflowRecalcType::kOnlyVisualOverflowRecalc);
        self.SetScrollableOverflowFromLayoutResults();

        // Transform origins and percentage translations depend on the final
        // box size. Chromium refreshes PaintLayer::transform_ here, after the
        // physical fragments have been installed and before ancestor overflow
        // is calculated. Keep our layout transform cache at the same lifecycle
        // point.
        if self.HasLayer() {
            self.UpdateTransformForLayout();
        }

        let scrollable_area = self.GetScrollableArea();
        if !scrollable_area.is_null() {
            unsafe { &mut *scrollable_area }.UpdateAfterLayout();
        }
        self.ClearNeedsLayout();
    }

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
