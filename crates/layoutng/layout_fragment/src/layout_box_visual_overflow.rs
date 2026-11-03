#![allow(non_snake_case)]

use foundation::{IsHorizontalWritingMode, LayoutUnit, MakeGarbageCollected, Member, PhysicalRect};
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::overflow_clip_axes::{
    kOverflowClipBothAxis, kOverflowClipX, kOverflowClipY, OverflowClipAxes,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::style_overflow_clip_margin::ReferenceBox;

use super::super::ink_overflow::ReadUnsetAsNoneScope;
use super::super::layout_box::LayoutBox;
use super::super::outline_geometry::OutlineOutsetExtent;
use super::super::outline_info::LayoutOutlineInfo;
use super::super::overflow_model::{BoxOverflowModel, BoxVisualOverflowModel};

#[path = "fragment_visual_overflow.rs"]
mod layout_fragment_fragment_visual_overflow;
use layout_fragment_fragment_visual_overflow::LayoutBoxCopyVisualOverflowFromFragmentsWithoutInvalidations;

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

    pub fn ClearVisualOverflow(&mut self) {
        self.CheckIsNotDestroyed();
        if !self.overflow_.Get().is_null() {
            unsafe { &mut *self.overflow_.Get() }.visual_overflow = None;
        }
    }

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

    pub fn CopyVisualOverflowFromFragmentsWithoutInvalidations(&mut self) {
        LayoutBoxCopyVisualOverflowFromFragmentsWithoutInvalidations(self)
    }

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

    #[cfg(debug_assertions)]
    pub fn VisualOverflowRectAllowingUnset(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let _read_unset_as_none = ReadUnsetAsNoneScope::new();
        self.VisualOverflowRect()
    }

    #[cfg(not(debug_assertions))]
    pub fn VisualOverflowRectAllowingUnset(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.VisualOverflowRect()
    }

    #[cfg(debug_assertions)]
    pub fn CheckIsVisualOverflowComputed(&self) {
        self.CheckIsNotDestroyed();
        // The source intentionally disables the remaining checks pending its
        // upstream investigation.
    }
}
