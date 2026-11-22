// Chromium LayoutView viewport and scroll geometry. This file is loaded by the
// reciprocal LayoutNG assembly because LayoutView is a concrete core type.
// Keeping the implementation here makes viewport projection and clipping show
// up as layout_fragment work instead of generic LayoutView self time.

use super::{
    gfx, EOverflow, IncludeScrollbarsInRect, LayoutUnit, LayoutView, OverlayScrollbarClipBehavior,
    PhysicalOffset, PhysicalRect, PhysicalSize, RuntimeEnabledFeatures, ScrollbarMode,
    ShouldIncludeScrollbarGutter, ToFlooredSize,
};

impl LayoutView {
    pub fn IsBeingAutoSized(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn InitialContainingBlockSize(&self) -> PhysicalSize {
        self.CheckIsNotDestroyed();
        let size = self.GetLayoutSize(IncludeScrollbarsInRect::kIncludeScrollbars);
        PhysicalSize::new(
            LayoutUnit::from_signed(size.width()),
            LayoutUnit::from_signed(size.height()),
        )
    }

    pub fn SetAutosizeScrollbarModes(&mut self, h_mode: ScrollbarMode, v_mode: ScrollbarMode) {
        self.CheckIsNotDestroyed();
        debug_assert_eq!(
            v_mode == ScrollbarMode::kAuto,
            h_mode == ScrollbarMode::kAuto
        );
        self.autosize_v_scrollbar_mode_ = v_mode;
        self.autosize_h_scrollbar_mode_ = h_mode;
    }

    pub fn DocumentRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.ScrollableOverflowRect()
    }

    pub fn SubtractUnconditionalScrollbarsFromViewportUnits(
        &self,
        viewport_size: &gfx::SizeF,
    ) -> gfx::SizeF {
        if !RuntimeEnabledFeatures::SmallerViewportUnitsEnabled() {
            return *viewport_size;
        }
        let mut removed = *viewport_size
            - gfx::SizeF::new(
                self.vertical_scrollbar_width_for_viewport_units_ as f32,
                self.horizontal_scrollbar_height_for_viewport_units_ as f32,
            );
        removed.SetToMax(&gfx::SizeF::new(0.0, 0.0));
        removed
    }

    pub fn SetScrollbarSizesForViewportUnits(&mut self, size: &gfx::Size) -> bool {
        self.CheckIsNotDestroyed();
        let mut changed = false;
        if size.width() != self.vertical_scrollbar_width_for_viewport_units_ {
            self.vertical_scrollbar_width_for_viewport_units_ = size.width();
            changed = true;
        }
        if size.height() != self.horizontal_scrollbar_height_for_viewport_units_ {
            self.horizontal_scrollbar_height_for_viewport_units_ = size.height();
            changed = true;
        }
        changed
    }

    pub fn PreviousBackgroundRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.previous_background_rect_.get()
    }

    pub fn SetPreviousBackgroundRect(&self, rect: PhysicalRect) {
        self.CheckIsNotDestroyed();
        self.previous_background_rect_.set(rect);
    }

    #[inline(never)]
    pub fn GetLayoutSize(&self, inclusion: IncludeScrollbarsInRect) -> gfx::Size {
        self.CheckIsNotDestroyed();
        if self.InputOwnerForLayout().InputPrinting() {
            return ToFlooredSize(&self.initial_containing_block_size_for_printing_);
        }
        self.GetNonPrintingLayoutSize(inclusion)
    }

    pub fn GetNonPrintingLayoutSize(&self, inclusion: IncludeScrollbarsInRect) -> gfx::Size {
        self.CheckIsNotDestroyed();
        let viewport = self
            .InputOwnerForLayout()
            .InputViewport()
            .as_ref()
            .expect("layout viewport required");
        let mut rect = PhysicalRect::new(
            PhysicalOffset::default(),
            PhysicalSize::new(
                LayoutUnit::from_signed(viewport.size.width),
                LayoutUnit::from_signed(viewport.size.height),
            ),
        );
        if inclusion == IncludeScrollbarsInRect::kExcludeScrollbars && self.IsScrollContainer() {
            self.ExcludeScrollbars(
                &mut rect,
                OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
                ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter,
            );
        }
        ToFlooredSize(&rect.size)
    }

    pub fn ViewLogicalHeightForPercentages(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let size = if self.InputOwnerForLayout().InputPrinting() {
            self.initial_containing_block_size_for_printing_
        } else {
            let size = self.GetLayoutSize(IncludeScrollbarsInRect::kExcludeScrollbars);
            PhysicalSize::new(
                LayoutUnit::from_signed(size.width()),
                LayoutUnit::from_signed(size.height()),
            )
        };
        if self.IsHorizontalWritingMode() {
            size.height
        } else {
            size.width
        }
    }

    pub fn ViewRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        if self.InputOwnerForLayout().InputPrinting() {
            return PhysicalRect::new(PhysicalOffset::default(), self.StitchedSize());
        }
        let viewport = self
            .InputOwnerForLayout()
            .InputViewport()
            .as_ref()
            .expect("layout viewport required");
        let size = viewport
            .transition_snapshot_size
            .as_ref()
            .unwrap_or(&viewport.size);
        PhysicalRect::new(
            PhysicalOffset::default(),
            PhysicalSize::new(
                LayoutUnit::from_signed(size.width),
                LayoutUnit::from_signed(size.height),
            ),
        )
    }

    pub fn OverflowClipRectWithBehavior(
        &self,
        behavior: OverlayScrollbarClipBehavior,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.OverflowClipRectInternal(behavior, false)
    }

    pub fn OverflowClipRectForScrollNode(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.OverflowClipRectInternal(
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            true,
        )
    }

    #[inline(never)]
    fn OverflowClipRectInternal(
        &self,
        behavior: OverlayScrollbarClipBehavior,
        _for_scroll_node: bool,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let mut rect = self.ViewRect();
        if rect.IsEmpty() {
            return self.base_.OverflowClipRectWithBehavior(behavior);
        }
        if self.IsScrollContainer() {
            self.ExcludeScrollbars(
                &mut rect,
                behavior,
                ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter,
            );
        }
        rect
    }

    pub fn CalculateScrollbarModes(
        &self,
        horizontal: &mut ScrollbarMode,
        vertical: &mut ScrollbarMode,
        overflow_x: Option<EOverflow>,
        overflow_y: Option<EOverflow>,
    ) {
        self.CheckIsNotDestroyed();
        assert_eq!(overflow_x.is_some(), overflow_y.is_some());
        let viewport = self.ViewportDefiningElementForLayout();
        let viewport_object = if viewport.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*viewport }.GetLayoutObject()
        };
        let style = if viewport_object.is_null() {
            self.StyleRef()
        } else {
            unsafe { &*viewport_object }.StyleRef()
        };
        let x = overflow_x.unwrap_or_else(|| style.OverflowX());
        let y = overflow_y.unwrap_or_else(|| style.OverflowY());
        let mode = |overflow: EOverflow| match overflow {
            EOverflow::kScroll => ScrollbarMode::kAlwaysOn,
            EOverflow::kHidden | EOverflow::kClip => ScrollbarMode::kAlwaysOff,
            _ => ScrollbarMode::kAuto,
        };
        *horizontal = mode(x);
        *vertical = mode(y);
    }

    pub fn ShouldPlaceBlockDirectionScrollbarOnLogicalLeft(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef()
            .ShouldPlaceBlockDirectionScrollbarOnLogicalLeft()
    }

    // Source core/layout/layout_view.cc:534-537.
    pub fn OffsetForFixedPosition(&self) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        if self.IsScrollContainer() {
            self.ScrolledContentOffset()
        } else {
            PhysicalOffset::default()
        }
    }

    pub fn SmallViewportSizeForViewportUnits(&self) -> gfx::SizeF {
        let viewport = self
            .InputOwnerForLayout()
            .InputViewport()
            .as_ref()
            .expect("layout viewport required");
        self.SubtractUnconditionalScrollbarsFromViewportUnits(&gfx::SizeF::new(
            viewport.size.width as f32,
            viewport.size.height as f32,
        ))
    }

    pub fn LargeViewportSizeForViewportUnits(&self) -> gfx::SizeF {
        self.SmallViewportSizeForViewportUnits()
    }

    pub fn DynamicViewportSizeForViewportUnits(&self) -> gfx::SizeF {
        self.SmallViewportSizeForViewportUnits()
    }

    pub fn PaginationViewportSizeForMediaQueries(&self) -> gfx::SizeF {
        gfx::SizeF::from(self.initial_containing_block_size_for_printing_)
    }

    pub fn DebugRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let size = self.GetLayoutSize(IncludeScrollbarsInRect::kIncludeScrollbars);
        PhysicalRect::new(
            PhysicalOffset::default(),
            PhysicalSize::new(
                LayoutUnit::from_signed(size.width()),
                LayoutUnit::from_signed(size.height()),
            ),
        )
    }

    pub fn AutosizeHorizontalScrollbarMode(&self) -> ScrollbarMode {
        self.CheckIsNotDestroyed();
        self.autosize_h_scrollbar_mode_
    }

    pub fn AutosizeVerticalScrollbarMode(&self) -> ScrollbarMode {
        self.CheckIsNotDestroyed();
        self.autosize_v_scrollbar_mode_
    }

    pub fn SetInitialContainingBlockSizeForPrinting(&mut self, size: PhysicalSize) {
        self.CheckIsNotDestroyed();
        self.initial_containing_block_size_for_printing_ = size;
    }

    pub fn InitialContainingBlockSizeForPrinting(&self) -> PhysicalSize {
        self.CheckIsNotDestroyed();
        self.initial_containing_block_size_for_printing_
    }

    pub fn SetPaginationScaleFactor(&mut self, factor: f32) {
        self.CheckIsNotDestroyed();
        self.pagination_scale_factor_ = factor;
    }

    pub fn PaginationScaleFactor(&self) -> f32 {
        self.CheckIsNotDestroyed();
        self.pagination_scale_factor_
    }

    pub fn BackgroundRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.OverflowClipRect()
    }
}
