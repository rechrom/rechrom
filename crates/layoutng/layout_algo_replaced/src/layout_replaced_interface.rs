#![allow(non_snake_case)]

use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};

use foundation::{
    gfx, DynamicTo, EAspectRatioType, InterpolationQuality, LayoutRatioFromSizeF, LayoutUnit,
    PhysicalOffset, PhysicalRect, PhysicalSize, ToFlooredSize, ToRoundedSize,
};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use layoutng_style::style::natural_sizing_info::{
    ConcreteObjectSizePhysical, PhysicalNaturalSizingInfo,
};
use layoutng_style::style::style_difference::StyleDifference;

use super::super::hit_test_phase::HitTestPhase;
use super::super::layout_box::LayoutBox;
use super::super::layout_node_metadata::Element;
use super::super::layout_object::{HitTestLocation, HitTestResult, LayoutObject, PaintInfo};
use super::super::layout_pass_scope::LayoutObjectFactoryScope;
use super::super::loader::fetch::resource_priority::ResourcePriority;

// Other non-inline LayoutReplaced bodies correspond to
// core/layout/layout_replaced.cc.
unsafe extern "Rust" {
    fn LayoutReplacedNew(element: *mut Element) -> LayoutReplaced;
    fn LayoutReplacedStyleDidChange(
        this: &mut LayoutReplaced,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &super::super::layout_object::StyleChangeContext,
    );
    fn LayoutReplacedNaturalSizeChanged(this: &mut LayoutReplaced);
    fn LayoutReplacedComputeSpeculativeDecodeSize(this: &LayoutReplaced) -> gfx::Size;
    fn LayoutReplacedLocalSelectionVisualRect(this: &LayoutReplaced) -> PhysicalRect;
    fn LayoutReplacedPositionForPoint(
        this: &LayoutReplaced,
        point: &PhysicalOffset,
    ) -> super::super::editing::forward::PositionWithAffinity;
    fn LayoutReplacedHitTestChildren(
        this: &LayoutReplaced,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
    fn LayoutReplacedRecalcVisualOverflow(this: &mut LayoutReplaced);
    fn LayoutReplacedContentRect(this: &LayoutReplaced) -> PhysicalRect;
    fn LayoutReplacedContentRectFrom(this: &LayoutReplaced, base: &PhysicalRect) -> PhysicalRect;
    fn LayoutReplacedDefaultWidth() -> i32;
    fn LayoutReplacedDefaultHeight() -> i32;
    fn DispatchLayoutReplacedGetNaturalDimensions(
        this: &LayoutReplaced,
    ) -> PhysicalNaturalSizingInfo;
    fn LayoutReplacedComputeNaturalSizingInfo(this: &LayoutReplaced) -> PhysicalNaturalSizingInfo;
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
    fn LayoutInputReplacedStyleDidChange(
        this: &mut LayoutInputReplaced,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &super::super::layout_object::StyleChangeContext,
    );
}

// cpp: core/layout/layout_replaced.h:46-195
#[repr(C)]
pub struct LayoutReplaced {
    box_: LayoutBox,
}

// cpp: core/layout/layout_replaced.h:197-202
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

    // cpp: core/layout/layout_replaced.h:48-49
    // This base is abstract in C++; only derived constructors call it.
    pub fn new(element: *mut Element) -> Self {
        unsafe { LayoutReplacedNew(element) }
    }

    // cpp: core/layout/layout_replaced.h:51-69
    pub fn ReplacedContentRect(&self) -> PhysicalRect {
        unsafe { LayoutReplacedContentRect(self) }
    }
    pub fn ReplacedContentRectFrom(&self, base: &PhysicalRect) -> PhysicalRect {
        unsafe { LayoutReplacedContentRectFrom(self, base) }
    }
    pub fn PreSnappedRectForPersistentSizing(rect: &PhysicalRect) -> PhysicalRect {
        // cpp: core/layout/layout_replaced.cc:504-507
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

    // cpp: core/layout/layout_replaced.h:70-89
    pub fn CanHaveChildren(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }
    pub fn DrawsBackgroundOntoContentLayer(&self) -> bool {
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

    // cpp: core/layout/layout_replaced.h:98-123
    pub fn GetNaturalDimensions(&self) -> PhysicalNaturalSizingInfo {
        unsafe { DispatchLayoutReplacedGetNaturalDimensions(self) }
    }
    pub fn ComputeNaturalSizingInfo(&self) -> PhysicalNaturalSizingInfo {
        unsafe { LayoutReplacedComputeNaturalSizingInfo(self) }
    }
    // cpp: core/layout/layout_replaced.h:111-117,156-159
    pub fn NaturalSizeChanged(&mut self) {
        unsafe { LayoutReplacedNaturalSizeChanged(self) }
    }
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &super::super::layout_object::StyleChangeContext,
    ) {
        unsafe {
            LayoutReplacedStyleDidChange(self, diff, old_style, new_style, style_change_context)
        }
    }
    // cpp: core/layout/layout_replaced.h:64-64
    pub fn RecalcVisualOverflow(&mut self) {
        unsafe { LayoutReplacedRecalcVisualOverflow(self) }
    }
    pub fn RespectsCSSOverflow(&self) -> bool {
        let factories = LayoutObjectFactoryScope::Objects();
        let callback = unsafe { factories.as_ref() }
            .and_then(|factories| factories.replaced_respects_css_overflow)
            .expect("replaced overflow virtual dispatch is not installed");
        callback(self)
    }
    pub fn ClipsToContentBox(&self) -> bool {
        unsafe { LayoutReplacedClipsToContentBox(self) }
    }
    // cpp: core/layout/layout_replaced.h:179-180
    pub fn ComputeSpeculativeDecodeSize(&self) -> gfx::Size {
        unsafe { LayoutReplacedComputeSpeculativeDecodeSize(self) }
    }
    // cpp: core/layout/layout_replaced.h:83-83
    pub fn LocalSelectionVisualRect(&self) -> PhysicalRect {
        unsafe { LayoutReplacedLocalSelectionVisualRect(self) }
    }
    // cpp: core/layout/layout_replaced.h:161-161
    pub fn PositionForPoint(
        &self,
        point: &PhysicalOffset,
    ) -> super::super::editing::forward::PositionWithAffinity {
        unsafe { LayoutReplacedPositionForPoint(self, point) }
    }

    // cpp: core/layout/layout_replaced.h:125-154
    pub fn ShouldApplyObjectViewBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.RuntimeClass() != super::super::layout_object::LayoutObjectClass::SvgRoot
    }
    pub fn HitTestChildren(
        &self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        unsafe { LayoutReplacedHitTestChildren(self, result, location, offset, phase) }
    }
    // cpp: core/layout/layout_replaced.h:139-147
    pub fn IsInSelfHitTestingPhase(&self, phase: HitTestPhase) -> bool {
        self.CheckIsNotDestroyed();
        let box_: &LayoutBox = self;
        if box_.IsInSelfHitTestingPhase(phase) {
            return true;
        }
        self.RespectsCSSOverflow() && phase == HitTestPhase::kSelfBlockBackground
    }
    pub fn ComputeReplacedContentRect(
        &self,
        base: &PhysicalRect,
        sizing: &PhysicalNaturalSizingInfo,
    ) -> PhysicalRect {
        unsafe { LayoutReplacedComputeReplacedContentRect(self, base, sizing) }
    }

    // cpp: core/layout/layout_replaced.h:163-177
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

    // cpp: core/layout/layout_replaced.h:186-194
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

    // cpp: core/layout/layout_replaced.h:197-202
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

// cpp: core/layout/layout_image.h:47-49,183-200
// cpp: core/layout/layout_image.cc:60-60
// Architecture-preserving specialization used by the assembled LayoutImage.
// It retains Chromium's derived LayoutImage state while decoded resource input
// continues to enter through the DOM/layout projection boundary.
#[repr(C)]
pub struct LayoutInputReplaced {
    replaced_: LayoutReplaced,
    natural_dimensions_: RefCell<PhysicalNaturalSizingInfo>,
    image_device_pixel_ratio_: Cell<f32>,
    is_generated_content_: Cell<bool>,
    did_increment_visually_non_empty_pixel_count_: Cell<bool>,
    cached_resource_priority_: Cell<Option<ResourcePriority>>,
    cached_speculative_decode_size_: Cell<gfx::Size>,
    cached_speculative_decode_quality_: Cell<InterpolationQuality>,
}

// cpp: core/layout/layout_image.h:212-217
impl foundation::DowncastFrom<LayoutObject> for LayoutInputReplaced {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutImage()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutInputReplaced, replaced_) == 0);

impl LayoutInputReplaced {
    // cpp: core/layout/layout_image.h:49-49
    // cpp: core/layout/layout_image.cc:52-52
    pub fn new(element: *mut Element) -> Self {
        let element = unsafe { &*element };
        let image_device_pixel_ratio = element
            .InputElementData()
            .as_ref()
            .map_or(1.0, |data| data.image_device_pixel_ratio as f32);
        Self {
            replaced_: LayoutReplaced::new(element as *const Element as *mut Element),
            natural_dimensions_: RefCell::new(PhysicalNaturalSizingInfo::None()),
            image_device_pixel_ratio_: Cell::new(image_device_pixel_ratio),
            is_generated_content_: Cell::new(element.InputIsStyleGenerated()),
            did_increment_visually_non_empty_pixel_count_: Cell::new(false),
            cached_resource_priority_: Cell::new(None),
            cached_speculative_decode_size_: Cell::new(gfx::Size::default()),
            cached_speculative_decode_quality_: Cell::new(foundation::kInterpolationNone),
        }
    }
    pub fn GetName(&self) -> &'static str {
        if self.IsLayoutImage() {
            "LayoutImage"
        } else {
            "LayoutInputReplaced"
        }
    }
    pub fn GetNaturalDimensions(&self) -> PhysicalNaturalSizingInfo {
        unsafe { LayoutInputReplacedGetNaturalDimensions(self) }
    }
    pub fn NaturalDimensions(&self) -> PhysicalNaturalSizingInfo {
        self.CheckIsNotDestroyed();
        self.natural_dimensions_.borrow().clone()
    }
    // cpp: core/layout/layout_image.cc:208-221
    pub fn UpdateNaturalDimensionsFromElementInput(&self) {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        assert!(!element.is_null());
        let mut sizing_info = PhysicalNaturalSizingInfo::None();
        if let Some(data) = unsafe { &*element }.InputElementData().as_ref() {
            // core/layout/layout_image.cc:218-220. Error completion invalidates
            // against an empty candidate size but retains the last successful
            // natural_dimensions_ value.
            if data.image_resource_error {
                return;
            }
            let zoom = self.StyleRef().EffectiveZoom();
            sizing_info.has_width = data.natural_width.is_some();
            sizing_info.has_height = data.natural_height.is_some();
            sizing_info.size = PhysicalSize::new(
                LayoutUnit::from_f64(data.natural_width.unwrap_or(0.0) * zoom as f64),
                LayoutUnit::from_f64(data.natural_height.unwrap_or(0.0) * zoom as f64),
            );
            if let Some(ratio) = data.natural_aspect_ratio {
                sizing_info.aspect_ratio = LayoutRatioFromSizeF(gfx::SizeF::new(ratio as f32, 1.0));
            } else if !sizing_info.size.IsEmpty() {
                sizing_info.aspect_ratio = sizing_info.size;
            }
        }
        *self.natural_dimensions_.borrow_mut() = sizing_info;
    }
    // cpp: core/layout/layout_image.h:74-90
    pub fn SetIsGeneratedContent(&self, generated: bool) {
        self.CheckIsNotDestroyed();
        self.is_generated_content_.set(generated);
    }
    pub fn IsGeneratedContent(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.is_generated_content_.get()
    }
    pub fn SetImageDevicePixelRatio(&self, factor: f32) {
        self.CheckIsNotDestroyed();
        self.image_device_pixel_ratio_.set(factor);
    }
    pub fn ImageDevicePixelRatio(&self) -> f32 {
        self.CheckIsNotDestroyed();
        self.image_device_pixel_ratio_.get()
    }

    // cpp: core/layout/layout_image.cc:195-205
    pub fn IncrementVisuallyNonEmptyPixelCountIfNeeded(&self) {
        self.CheckIsNotDestroyed();
        if self.did_increment_visually_non_empty_pixel_count_.get() {
            return;
        }
        let element = DynamicTo::<Element>(self.GetNode());
        if element.is_null()
            || !unsafe { &*element }
                .InputElementData()
                .as_ref()
                .is_some_and(|data| data.image_resource_is_loaded || data.image_resource_error)
        {
            return;
        }
        let view = self.View();
        if view.is_null() {
            return;
        }
        let zoom = self.StyleRef().EffectiveZoom();
        let mut default_object_size =
            PhysicalSize::new(LayoutUnit::from_signed(300), LayoutUnit::from_signed(150));
        default_object_size.ScaleFloat(zoom);
        let mut concrete_object_size =
            ConcreteObjectSizePhysical(&self.NaturalDimensions(), &default_object_size);
        concrete_object_size.ScaleFloat(1.0 / zoom);
        unsafe { &mut *view }
            .IncrementVisuallyNonEmptyPixelCount(&ToFlooredSize(&concrete_object_size));
        self.did_increment_visually_non_empty_pixel_count_.set(true);
    }
    // cpp: core/layout/layout_image.cc:398-409
    // cpp: core/layout/layout_image.h:55-55
    pub fn IsUnsizedImage(&self) -> bool {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        let explicit_width = style.LogicalWidth().HasOnlyFixedAndPercent();
        let explicit_height = style.LogicalHeight().HasOnlyFixedAndPercent();
        let has_aspect_ratio = style.AspectRatio().GetType() == EAspectRatioType::kRatio;
        let is_fixed_size = (explicit_width && explicit_height)
            || (has_aspect_ratio && (explicit_width || explicit_height));
        !is_fixed_size
    }
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &super::super::layout_object::StyleChangeContext,
    ) {
        unsafe {
            LayoutInputReplacedStyleDidChange(
                self,
                diff,
                old_style,
                new_style,
                style_change_context,
            )
        }
    }

    // cpp: core/layout/layout_image.cc:241-269
    // cpp: core/layout/layout_image.h:102-107,205-209
    pub fn ComputeResourcePriority(&self) -> ResourcePriority {
        let priority = self.replaced_.ComputeResourcePriority();
        self.cached_resource_priority_.set(Some(priority));
        priority
    }
    pub fn CachedResourcePriority(&self) -> Option<ResourcePriority> {
        self.cached_resource_priority_.get()
    }
    pub fn ComputeSpeculativeDecodeSize(&self) -> gfx::Size {
        let size = self.replaced_.ComputeSpeculativeDecodeSize();
        self.cached_speculative_decode_size_.set(size);
        size
    }
    pub fn CachedSpeculativeDecodeSize(&self) -> gfx::Size {
        self.cached_speculative_decode_size_.get()
    }
    pub fn ComputeSpeculativeDecodeQuality(&self) -> InterpolationQuality {
        // cpp: core/layout/layout_object.cc:4819-4822
        let quality = self.StyleRef().GetInterpolationQuality();
        self.cached_speculative_decode_quality_.set(quality);
        quality
    }
    pub fn CachedSpeculativeDecodeQuality(&self) -> InterpolationQuality {
        self.cached_speculative_decode_quality_.get()
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
