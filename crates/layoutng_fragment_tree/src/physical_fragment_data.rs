use foundation::{DynamicTo, RuntimeEnabledFeatures, To};
use layoutng_style::style::scroll_snap_data::cc;
use layoutng::internal::layout_box_model_object::PaintLayer;
use layoutng::internal::layout_node_metadata::Node;
use layoutng::internal::hit_test_phase::HitTestPhase;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng::internal::layout_object::LayoutObject;
use layoutng_geometry::geometry::overflow_clip_axes::{
    kNoOverflowClip, kOverflowClipX, kOverflowClipY, OverflowClipAxes,
};
use layoutng_style::style::computed_style::ComputedStyle;

use crate::physical_fragment::{PhysicalFragment, PhysicalFragmentFlags};

#[allow(non_snake_case)]
impl PhysicalFragment {
    // C++ dereferences layout_object_ directly in most of these methods.
    // Retain that non-null precondition rather than inventing fallback values.
    fn layout_object_for_data(&self) -> &LayoutObject {
        unsafe { &*self.layout_object_ptr() }
    }

    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:14-18
    pub fn IsColumnSpanAll(&self) -> bool {
        let box_ptr = DynamicTo::<LayoutBox>(self.GetLayoutObject());
        if !box_ptr.is_null() {
            return unsafe { &*box_ptr }.IsColumnSpanAll();
        }
        false
    }

    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:20-22
    pub fn IsFixedPositioned(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().IsFixedPositioned()
    }

    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:24-28
    pub fn IsPositioned(&self) -> bool {
        let layout_object = self.GetLayoutObject();
        if !layout_object.is_null() {
            return unsafe { &*layout_object }.IsPositioned();
        }
        false
    }

    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:30-33
    pub fn HasStickyConstrainedPosition(&self) -> bool {
        self.IsCSSBox()
            && self
                .layout_object_for_data()
                .StyleRef()
                .HasStickyConstrainedPosition()
    }

    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:35-37
    pub fn IsInitialLetterBox(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().IsInitialLetterBox()
    }

    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:39-43
    pub fn IsSnapArea(&self) -> bool {
        self.IsCSSBox()
            && !DynamicTo::<LayoutBox>(self.layout_object_ptr()).is_null()
            && *self
                .layout_object_for_data()
                .StyleRef()
                .GetScrollSnapAlign()
                != cc::ScrollSnapAlign::default()
    }

    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:45-47
    pub fn IsMathML(&self) -> bool {
        self.IsBox() && unsafe { &*self.GetSelfOrContainerLayoutObject() }.IsMathML()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:229
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:49-51
    pub fn IsAnonymousBlockFlow(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().IsAnonymousBlockFlow()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:230
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:53
    pub fn IsFrameSet(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().IsFrameSet()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:231
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:55-57
    pub fn IsListMarker(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().IsLayoutOutsideListMarker()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:233
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:59
    pub fn IsSvg(&self) -> bool {
        self.layout_object_for_data().IsSVG()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:234
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:61
    pub fn IsSvgText(&self) -> bool {
        self.layout_object_for_data().IsSVGText()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:238
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:63
    pub fn IsTable(&self) -> bool {
        self.IsTablePart() && self.layout_object_for_data().IsTable()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:240
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:65-67
    pub fn IsTableRow(&self) -> bool {
        self.IsTablePart() && self.layout_object_for_data().IsTableRow()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:242
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:69-71
    pub fn IsTableSection(&self) -> bool {
        self.IsTablePart() && self.layout_object_for_data().IsTableSection()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:244
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:73-75
    pub fn IsTableCell(&self) -> bool {
        self.IsTablePart() && self.layout_object_for_data().IsTableCell()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:246
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:77
    pub fn IsGrid(&self) -> bool {
        self.layout_object_for_data().IsLayoutGrid()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:247
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:79
    pub fn IsGridLanes(&self) -> bool {
        self.layout_object_for_data().IsLayoutGridLanes()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:298
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:81-83
    pub fn Style(&self) -> &ComputedStyle {
        self.layout_object_for_data()
            .EffectiveStyle(self.GetStyleVariant())
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:300
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:85-87
    pub fn GetNode(&self) -> *mut Node {
        if self.IsCSSBox() {
            self.layout_object_for_data().GetNode()
        } else {
            std::ptr::null_mut()
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:301
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:89-91
    pub fn GeneratingNode(&self) -> *mut Node {
        if self.IsCSSBox() {
            self.layout_object_for_data().GeneratingNode()
        } else {
            std::ptr::null_mut()
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:304
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:93-97
    pub fn NodeForHitTest(&self) -> *mut Node {
        if self.IsFragmentainerBox() {
            return std::ptr::null_mut();
        }
        self.layout_object_for_data().NodeForHitTest()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:306
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:99-101
    pub fn NonPseudoNode(&self) -> *mut Node {
        if self.IsCSSBox() {
            self.layout_object_for_data().NonPseudoNode()
        } else {
            std::ptr::null_mut()
        }
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:308
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:103-113
    pub fn IsInSelfHitTestingPhase(&self, phase: HitTestPhase) -> bool {
        if self.IsFragmentainerBox() {
            return false;
        }
        let box_ptr = DynamicTo::<LayoutBox>(self.GetLayoutObject());
        if !box_ptr.is_null() {
            return unsafe { &*box_ptr }.IsInSelfHitTestingPhase(phase);
        }
        if self.IsInlineBox() {
            return phase == HitTestPhase::kForeground;
        }
        phase == HitTestPhase::kSelfBlockBackground
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:311
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:115
    pub fn HasLayer(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().HasLayer()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:314
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:117-121
    pub fn Layer(&self) -> *mut PaintLayer {
        if !self.HasLayer() {
            return std::ptr::null_mut();
        }
        unsafe { &*To::<LayoutBoxModelObject>(self.layout_object_ptr()) }.Layer()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:317
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:123-126
    pub fn HasSelfPaintingLayer(&self) -> bool {
        self.HasLayer()
            && unsafe { &*To::<LayoutBoxModelObject>(self.layout_object_ptr()) }
                .HasSelfPaintingLayer()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:321
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:128-130
    pub fn HasNonVisibleOverflow(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().HasNonVisibleOverflow()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:323
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:132-137
    pub fn GetOverflowClipAxes(&self) -> OverflowClipAxes {
        if !self.IsCSSBox() {
            return kNoOverflowClip;
        }
        self.layout_object_for_data().GetOverflowClipAxes()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:325
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:139-145
    pub fn HasNonVisibleBlockOverflow(&self) -> bool {
        let clip_axes = self.GetOverflowClipAxes();
        if self.Style().IsHorizontalWritingMode() {
            return clip_axes & kOverflowClipY != 0;
        }
        clip_axes & kOverflowClipX != 0
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:329
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:147-150
    pub fn IsScrollContainer(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().IsScrollContainer()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:333
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:152-154
    pub fn IsNonOverlayOverscrollScrollContainer(&self) -> bool {
        self.IsCSSBox()
            && self
                .layout_object_for_data()
                .IsContentMovingOverscrollContainer()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:341
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:156-158
    pub fn IsEffectiveRootScroller(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().IsEffectiveRootScroller()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:343
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:160-162
    pub fn ShouldApplyLayoutContainment(&self) -> bool {
        self.IsCSSBox() && self.layout_object_for_data().ShouldApplyLayoutContainment()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:345
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:164-166
    pub fn ShouldClipOverflowAlongEitherAxis(&self) -> bool {
        self.IsCSSBox()
            && self
                .layout_object_for_data()
                .ShouldClipOverflowAlongEitherAxis()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:347
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:168-170
    pub fn ShouldClipOverflowAlongBothAxis(&self) -> bool {
        self.IsCSSBox()
            && self
                .layout_object_for_data()
                .ShouldClipOverflowAlongBothAxis()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:349
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:172-174
    pub fn ShouldApplyOverflowClipMargin(&self) -> bool {
        self.IsCSSBox()
            && self
                .layout_object_for_data()
                .ShouldApplyOverflowClipMargin()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:354
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:176-178
    pub fn CanTraverse(&self) -> bool {
        self.layout_object_for_data().CanTraversePhysicalFragments()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:358
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:180-182
    pub fn IsHiddenForPaint(&self) -> bool {
        self.flags_.get(PhysicalFragmentFlags::IS_HIDDEN_FOR_PAINT)
            || self.layout_object_for_data().IsTruncated()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:374
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:184
    pub fn IsExplicitAnchor(&self) -> bool {
        self.IsCSSBox() && !self.Style().AnchorName().Get().is_null()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:634
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:186-190
    pub fn HasOutOfFlowInFragmentainerSubtree(&self) -> bool {
        debug_assert!(
            !self
                .flags_
                .get(PhysicalFragmentFlags::HAS_OUT_OF_FLOW_IN_FRAGMENTAINER_SUBTREE)
                || !RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
        );
        self.flags_
            .get(PhysicalFragmentFlags::HAS_OUT_OF_FLOW_IN_FRAGMENTAINER_SUBTREE)
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:640
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:192-195
    pub fn HasChildAnchors(&self) -> bool {
        let oof_data = self.oof_data_.Get();
        if oof_data.is_null() {
            return false;
        }
        let anchor_map = unsafe { &*oof_data }.GetAnchorMap();
        !anchor_map.is_null() && !unsafe { &*anchor_map }.IsEmpty()
    }

    // cpp: layoutng_fragment_tree/physical_fragment.h:636
    // cpp: layoutng_fragment_tree/physical_fragment_data.cc:197-199
    pub fn HasOutOfFlowPositionedDescendants(&self) -> bool {
        let oof_data = self.oof_data_.Get();
        !oof_data.is_null() && !unsafe { &*oof_data }.OofPositionedDescendants().is_empty()
    }
}
