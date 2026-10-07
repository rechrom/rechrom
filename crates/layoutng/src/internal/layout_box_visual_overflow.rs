#![allow(non_snake_case)]

use foundation::{IsHorizontalWritingMode, LayoutUnit, MakeGarbageCollected, Member, PhysicalRect};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::overflow_clip_axes::{
    kOverflowClipBothAxis, kOverflowClipX, kOverflowClipY, OverflowClipAxes,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::style_overflow_clip_margin::ReferenceBox;

use super::ink_overflow::ReadUnsetAsNoneScope;
use super::layout_box::LayoutBox;
use super::outline_geometry::OutlineOutsetExtent;
use super::outline_info::LayoutOutlineInfo;
use super::overflow_model::{BoxOverflowModel, BoxVisualOverflowModel};

// cpp: layoutng/internal/layout_box_visual_overflow.cc:38-49
fn ApplyOverflowClip(
    overflow_clip_axes: OverflowClipAxes,
    no_overflow_rect: &PhysicalRect,
    result: &mut PhysicalRect,
) {
    if overflow_clip_axes & kOverflowClipX != 0 {
        result.offset.left = no_overflow_rect.X();
        result.size.width = no_overflow_rect.Width();
    }
    if overflow_clip_axes & kOverflowClipY != 0 {
        result.offset.top = no_overflow_rect.Y();
        result.size.height = no_overflow_rect.Height();
    }
}

impl LayoutBox {
    // cpp: layoutng/internal/layout_box_visual_overflow.cc:52-69
    pub fn AddSelfVisualOverflow(&mut self, rect: &PhysicalRect) {
        self.CheckIsNotDestroyed();
        if rect.IsEmpty() {
            return;
        }
        let border_box = self.PhysicalBorderBoxRect();
        if border_box.ContainsRect(rect) {
            return;
        }
        if !self.VisualOverflowIsSet() {
            if self.overflow_.Get().is_null() {
                self.overflow_ =
                    Member::from_ptr(MakeGarbageCollected(BoxOverflowModel::default()));
            }
            unsafe { &mut *self.overflow_.Get() }
                .visual_overflow
                .get_or_insert_with(|| BoxVisualOverflowModel::new(&border_box));
        }
        unsafe { &mut *self.overflow_.Get() }
            .visual_overflow
            .as_mut()
            .expect("visual overflow initialized")
            .AddSelfVisualOverflow(rect);
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:71-92
    pub fn AddContentsVisualOverflow(&mut self, rect: &PhysicalRect) {
        self.CheckIsNotDestroyed();
        if rect.IsEmpty() {
            return;
        }
        let border_box = self.PhysicalBorderBoxRect();
        if !self.HasNonVisibleOverflow() && border_box.ContainsRect(rect) {
            return;
        }
        if !self.VisualOverflowIsSet() {
            if self.overflow_.Get().is_null() {
                self.overflow_ =
                    Member::from_ptr(MakeGarbageCollected(BoxOverflowModel::default()));
            }
            unsafe { &mut *self.overflow_.Get() }
                .visual_overflow
                .get_or_insert_with(|| BoxVisualOverflowModel::new(&border_box));
        }
        unsafe { &mut *self.overflow_.Get() }
            .visual_overflow
            .as_mut()
            .expect("visual overflow initialized")
            .AddContentsVisualOverflow(rect);
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:94-103
    pub fn UpdateHasSubpixelVisualEffectOutsets(&mut self, outsets: &PhysicalBoxStrut) {
        self.CheckIsNotDestroyed();
        if !self.VisualOverflowIsSet() {
            return;
        }
        unsafe { &mut *self.overflow_.Get() }
            .visual_overflow
            .as_mut()
            .expect("visual overflow set")
            .SetHasSubpixelVisualEffectOutsets(
                !outsets.top.IsInteger()
                    || !outsets.right.IsInteger()
                    || !outsets.bottom.IsInteger()
                    || !outsets.left.IsInteger(),
            );
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:105-132
    pub fn SetVisualOverflow(&mut self, self_rect: &PhysicalRect, contents: &PhysicalRect) {
        self.CheckIsNotDestroyed();
        self.ClearVisualOverflow();
        self.AddSelfVisualOverflow(self_rect);
        self.AddContentsVisualOverflow(contents);
        if !self.VisualOverflowIsSet() {
            return;
        }
        let overflow_rect = *unsafe { &*self.overflow_.Get() }
            .visual_overflow
            .as_ref()
            .expect("visual overflow set")
            .SelfVisualOverflowRect();
        let box_size = self.StitchedSize();
        let outsets = PhysicalBoxStrut::new(
            -overflow_rect.Y(),
            overflow_rect.Right() - box_size.width,
            overflow_rect.Bottom() - box_size.height,
            -overflow_rect.X(),
        );
        self.UpdateHasSubpixelVisualEffectOutsets(&outsets);
        let style = self.StyleRef();
        if style.HasOutline() {
            let outline_extent = LayoutUnit::from_signed(OutlineOutsetExtent(
                style,
                &LayoutOutlineInfo::GetFromStyle(style),
            ));
            self.SetOutlineMayBeAffectedByDescendants(
                outsets.top != outline_extent
                    || outsets.right != outline_extent
                    || outsets.bottom != outline_extent
                    || outsets.left != outline_extent,
            );
        }
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:134-140
    pub fn ClearVisualOverflow(&mut self) {
        self.CheckIsNotDestroyed();
        if !self.overflow_.Get().is_null() {
            unsafe { &mut *self.overflow_.Get() }.visual_overflow = None;
        }
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:142-152
    pub fn CanUseFragmentsForVisualOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.PhysicalFragmentCount() == 0 {
            return false;
        }
        let fragment = unsafe { &*self.GetPhysicalFragment(0) };
        if !fragment.CanUseFragmentsForInkOverflow() {
            return false;
        }
        true
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:154-164
    pub fn CopyVisualOverflowFromFragments(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.CanUseFragmentsForVisualOverflow());
        let previous_visual_overflow = self.VisualOverflowRectAllowingUnset();
        self.CopyVisualOverflowFromFragmentsWithoutInvalidations();
        let visual_overflow = self.VisualOverflowRect();
        if visual_overflow == previous_visual_overflow {
            return;
        }
        self.SetShouldCheckForPaintInvalidation();
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:166-239
    pub fn CopyVisualOverflowFromFragmentsWithoutInvalidations(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.CanUseFragmentsForVisualOverflow());
        let fragment_count = self.PhysicalFragmentCount();
        if fragment_count == 0 {
            debug_assert!(self.IsLayoutTableCol());
            self.ClearVisualOverflow();
            return;
        }
        if fragment_count == 1 {
            let fragment = unsafe { &*self.GetPhysicalFragment(0) };
            debug_assert!(fragment.CanUseFragmentsForInkOverflow());
            if !fragment.HasInkOverflow() {
                self.ClearVisualOverflow();
                return;
            }
            self.SetVisualOverflow(
                &fragment.SelfInkOverflowRect(),
                &fragment.ContentsInkOverflowRect(),
            );
            return;
        }

        let containing_block = self.ContainingBlock();
        debug_assert!(!containing_block.is_null());
        let writing_mode = unsafe { &*containing_block }.StyleRef().GetWritingMode();
        let mut has_overflow = false;
        let mut self_rect = PhysicalRect::default();
        let mut contents_rect = PhysicalRect::default();
        let mut last_fragment: *const PhysicalBoxFragment = std::ptr::null();
        // PhysicalFragments() is a view over the same result vector. Indexing
        // keeps its iteration order without requiring that pending view owner.
        for index in 0..fragment_count {
            let fragment = unsafe { &*self.GetPhysicalFragment(index) };
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
            self.ClearVisualOverflow();
            return;
        }
        self.SetVisualOverflow(&self_rect, &contents_rect);
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:241-261
    pub fn BorderOutsetsForClipping(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        let padding_box = -self.BorderOutsets();
        if !self.ShouldApplyOverflowClipMargin() {
            return padding_box;
        }
        let clip_margin = self
            .StyleRef()
            .OverflowClipMargin()
            .as_ref()
            .expect("overflow clip margin exists");
        let mut overflow_clip_margin = PhysicalBoxStrut::default();
        match clip_margin.GetReferenceBox() {
            ReferenceBox::kBorderBox => {}
            ReferenceBox::kPaddingBox => overflow_clip_margin = padding_box,
            ReferenceBox::kContentBox => {
                overflow_clip_margin = padding_box - self.PaddingOutsets();
            }
        }
        overflow_clip_margin.Inflate(clip_margin.GetMargin());
        overflow_clip_margin
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:263-300
    pub fn VisualOverflowRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        if !self.VisualOverflowIsSet() {
            return self.PhysicalBorderBoxRect();
        }
        let visual_overflow = unsafe { &*self.overflow_.Get() }
            .visual_overflow
            .as_ref()
            .expect("visual overflow set");
        let self_visual_overflow_rect = visual_overflow.SelfVisualOverflowRect();
        if self.HasMask() {
            return *self_visual_overflow_rect;
        }

        let overflow_clip_axes = self.GetOverflowClipAxes();
        if self.ShouldApplyOverflowClipMargin() {
            debug_assert_eq!(overflow_clip_axes, kOverflowClipBothAxis);
            let contents_visual_overflow_rect = visual_overflow.ContentsVisualOverflowRect();
            if !contents_visual_overflow_rect.IsEmpty() {
                let mut result = self.PhysicalBorderBoxRect();
                let outsets = self.BorderOutsetsForClipping();
                result.ExpandEdges(outsets.top, outsets.right, outsets.bottom, outsets.left);
                result.Intersect(contents_visual_overflow_rect);
                result.Unite(self_visual_overflow_rect);
                return result;
            }
        }

        if overflow_clip_axes == kOverflowClipBothAxis {
            return *self_visual_overflow_rect;
        }
        let mut result = *visual_overflow.ContentsVisualOverflowRect();
        result.Unite(self_visual_overflow_rect);
        ApplyOverflowClip(overflow_clip_axes, self_visual_overflow_rect, &mut result);
        result
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:303-307
    #[cfg(debug_assertions)]
    pub fn VisualOverflowRectAllowingUnset(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let _read_unset_as_none = ReadUnsetAsNoneScope::new();
        self.VisualOverflowRect()
    }

    // cpp: layoutng/internal/layout_box.h:349-353
    #[cfg(not(debug_assertions))]
    pub fn VisualOverflowRectAllowingUnset(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.VisualOverflowRect()
    }

    // cpp: layoutng/internal/layout_box_visual_overflow.cc:309-326
    #[cfg(debug_assertions)]
    pub fn CheckIsVisualOverflowComputed(&self) {
        self.CheckIsNotDestroyed();
        // The source intentionally disables the remaining checks pending its
        // upstream investigation.
    }
}
