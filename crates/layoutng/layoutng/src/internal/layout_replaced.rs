#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, ToRoundedSize};
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use layoutng_style::style::natural_sizing_info::PhysicalNaturalSizingInfo;

use super::hit_test_phase::HitTestPhase;
use super::layout_box::LayoutBox;
use super::layout_node_metadata::Element;
use super::layout_object::{HitTestLocation, HitTestResult, LayoutObject, PaintInfo};

// Other non-inline LayoutReplaced bodies belong to
// //src/layoutng_replaced/layout_replaced.cc.
unsafe extern "Rust" {
    fn LayoutReplacedNew(element: *mut Element) -> LayoutReplaced;
    fn LayoutReplacedContentRect(this: &LayoutReplaced) -> PhysicalRect;
    fn LayoutReplacedContentRectFrom(this: &LayoutReplaced, base: &PhysicalRect) -> PhysicalRect;
    fn LayoutReplacedDefaultWidth() -> i32;
    fn LayoutReplacedDefaultHeight() -> i32;
    fn DispatchLayoutReplacedGetNaturalDimensions(
        this: &LayoutReplaced,
    ) -> PhysicalNaturalSizingInfo;
    fn LayoutReplacedComputeNaturalSizingInfo(this: &LayoutReplaced) -> PhysicalNaturalSizingInfo;
    fn LayoutReplacedRespectsCSSOverflow(this: &LayoutReplaced) -> bool;
    fn LayoutReplacedClipsToContentBox(this: &LayoutReplaced) -> bool;
    fn LayoutReplacedComputeReplacedContentRect(
        this: &LayoutReplaced,
        base: &PhysicalRect,
        sizing: &PhysicalNaturalSizingInfo,
    ) -> PhysicalRect;
    fn LayoutReplacedComputeObjectViewBoxRect(
        this: &LayoutReplaced,
        sizing: &PhysicalNaturalSizingInfo,
    ) -> Option<PhysicalRect>;
    fn LayoutReplacedComputeObjectFitAndPositionRect(
        this: &LayoutReplaced,
        base: &PhysicalRect,
        sizing: &PhysicalNaturalSizingInfo,
    ) -> PhysicalRect;
    fn LayoutInputReplacedGetNaturalDimensions(
        this: &LayoutInputReplaced,
    ) -> PhysicalNaturalSizingInfo;
}

// cpp: layoutng/internal/layout_replaced.h:38-150
#[repr(C)]
pub struct LayoutReplaced {
    box_: LayoutBox,
}

// cpp: layoutng/internal/layout_replaced.h:162-167
impl foundation::DowncastFrom<LayoutObject> for LayoutReplaced {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutReplaced()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutReplaced, box_) == 0);

impl LayoutReplaced {
    // The owning layoutng_replaced crate supplies the C++ constructor body;
    // this base-first helper exposes only the core LayoutBox field construction.
    pub fn FromLayoutBox(box_: LayoutBox) -> Self {
        Self { box_ }
    }

    // cpp: layoutng/internal/layout_replaced.h:48-49
    // This base is abstract in C++; only derived constructors call it.
    pub fn new(element: *mut Element) -> Self {
        unsafe { LayoutReplacedNew(element) }
    }

    // cpp: layoutng/internal/layout_replaced.h:51-67
    pub fn ReplacedContentRect(&self) -> PhysicalRect {
        unsafe { LayoutReplacedContentRect(self) }
    }
    pub fn ReplacedContentRectFrom(&self, base: &PhysicalRect) -> PhysicalRect {
        unsafe { LayoutReplacedContentRectFrom(self, base) }
    }
    pub fn PreSnappedRectForPersistentSizing(rect: &PhysicalRect) -> PhysicalRect {
        // cpp: layoutng/internal/layout_box_geometry.cc:934-937
        let rounded = ToRoundedSize(rect.size);
        PhysicalRect::new(
            rect.offset,
            PhysicalSize::new(
                LayoutUnit::from_signed(rounded.width()),
                LayoutUnit::from_signed(rounded.height()),
            ),
        )
    }
    pub fn kDefaultWidth() -> i32 {
        unsafe { LayoutReplacedDefaultWidth() }
    }
    pub fn kDefaultHeight() -> i32 {
        unsafe { LayoutReplacedDefaultHeight() }
    }

    // cpp: layoutng/internal/layout_replaced.h:68-82
    pub fn CanHaveChildren(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }
    pub fn PaintReplaced(&self, _info: &PaintInfo, _offset: &PhysicalOffset) {
        self.CheckIsNotDestroyed();
    }
    pub fn HasObjectFit(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().GetObjectFit() != ComputedStyleInitialValues::InitialObjectFit()
    }

    // cpp: layoutng/internal/layout_replaced.h:84-101
    pub fn GetNaturalDimensions(&self) -> PhysicalNaturalSizingInfo {
        unsafe { DispatchLayoutReplacedGetNaturalDimensions(self) }
    }
    pub fn ComputeNaturalSizingInfo(&self) -> PhysicalNaturalSizingInfo {
        unsafe { LayoutReplacedComputeNaturalSizingInfo(self) }
    }
    pub fn RespectsCSSOverflow(&self) -> bool {
        unsafe { LayoutReplacedRespectsCSSOverflow(self) }
    }
    pub fn ClipsToContentBox(&self) -> bool {
        unsafe { LayoutReplacedClipsToContentBox(self) }
    }

    // cpp: layoutng/internal/layout_replaced.h:103-122
    pub fn ShouldApplyObjectViewBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.RuntimeClass() != super::layout_object::LayoutObjectClass::SvgRoot
    }
    pub fn HitTestChildren(
        &self,
        _result: &mut HitTestResult,
        _location: &HitTestLocation,
        _offset: &PhysicalOffset,
        _phase: HitTestPhase,
    ) -> bool {
        self.CheckIsNotDestroyed();
        false
    }
    pub fn ComputeReplacedContentRect(
        &self,
        base: &PhysicalRect,
        sizing: &PhysicalNaturalSizingInfo,
    ) -> PhysicalRect {
        unsafe { LayoutReplacedComputeReplacedContentRect(self, base, sizing) }
    }

    // cpp: layoutng/internal/layout_replaced.h:124-138
    pub fn IsLayoutReplaced(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
    pub fn IsMonolithic(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
    pub fn ComputeCanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_replaced.h:140-149
    fn ComputeObjectViewBoxRect(&self, sizing: &PhysicalNaturalSizingInfo) -> Option<PhysicalRect> {
        unsafe { LayoutReplacedComputeObjectViewBoxRect(self, sizing) }
    }
    fn ComputeObjectFitAndPositionRect(
        &self,
        base: &PhysicalRect,
        sizing: &PhysicalNaturalSizingInfo,
    ) -> PhysicalRect {
        unsafe { LayoutReplacedComputeObjectFitAndPositionRect(self, base, sizing) }
    }

    // cpp: layoutng/internal/layout_replaced.h:162-167
    pub fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutReplaced()
    }
}

impl Deref for LayoutReplaced {
    type Target = LayoutBox;
    fn deref(&self) -> &Self::Target {
        &self.box_
    }
}
impl DerefMut for LayoutReplaced {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.box_
    }
}

// cpp: layoutng/internal/layout_replaced.h:152-160
#[repr(C)]
pub struct LayoutInputReplaced {
    replaced_: LayoutReplaced,
}

impl LayoutInputReplaced {
    // cpp: layoutng/internal/layout_replaced.h:157-159
    pub fn new(element: *mut Element) -> Self {
        Self {
            replaced_: LayoutReplaced::new(element),
        }
    }
    pub fn GetName(&self) -> &'static str {
        "LayoutInputReplaced"
    }
    pub fn GetNaturalDimensions(&self) -> PhysicalNaturalSizingInfo {
        unsafe { LayoutInputReplacedGetNaturalDimensions(self) }
    }
}

impl Deref for LayoutInputReplaced {
    type Target = LayoutReplaced;
    fn deref(&self) -> &Self::Target {
        &self.replaced_
    }
}
impl DerefMut for LayoutInputReplaced {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.replaced_
    }
}
