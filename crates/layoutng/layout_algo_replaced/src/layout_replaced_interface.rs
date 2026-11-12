#![allow(non_snake_case)]

use std::cell::{Cell, RefCell, UnsafeCell};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

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
        // RecalcVisualOverflow also runs from PrePaint, after the temporary
        // layout pass scope has unwound. LayoutObject retains the exact factory
        // set installed when this concrete object was created; use that owner
        // instead of a thread-local construction scope.
        let callback = unsafe { self.FactorySet().as_ref() }
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

// cpp: core/layout/layout_image_resource.h:37-86
// cpp: core/layout/layout_image_resource.cc:42-75
// LayoutImageResource retains the producer's original ImageResourceContent
// identity. ImageLoader decides when a candidate is complete; this owner keeps
// the previous completed candidate during a pending source change.
pub struct LayoutImageResource {
    cached_image_: Option<Rc<dyn image_resource::ImageResourceContentLease>>,
}

impl LayoutImageResource {
    pub fn new() -> Self {
        Self {
            cached_image_: None,
        }
    }
    pub fn Update(&mut self, update: image_resource::ImageResourceContentUpdate) -> bool {
        let same = match (&self.cached_image_, &update.content) {
            (Some(current), Some(next)) => Rc::ptr_eq(current, next),
            (None, None) => true,
            _ => false,
        };
        if same {
            // ImageNotifyFinished keeps the same ImageResourceContent owner,
            // but its decoded metadata and error state become observable.
            return update.complete;
        }
        if !update.complete && self.cached_image_.is_some() {
            return false;
        }
        self.cached_image_ = update.content;
        true
    }
    pub fn CachedImage(&self) -> Option<Rc<dyn image_resource::ImageResourceContentLease>> {
        self.cached_image_.clone()
    }
    pub fn HasImage(&self) -> bool {
        self.cached_image_.is_some()
    }
    pub fn ErrorOccurred(&self) -> bool {
        self.cached_image_
            .as_ref()
            .is_some_and(|content| content.ErrorOccurred())
    }
    pub fn IsLoading(&self) -> bool {
        self.cached_image_
            .as_ref()
            .is_some_and(|content| content.IsLoading())
    }
    pub fn Metadata(&self) -> Option<image_resource::ImageResourceMetadata> {
        self.cached_image_
            .as_ref()
            .and_then(|content| content.Metadata())
    }
}

impl Default for LayoutImageResource {
    fn default() -> Self {
        Self::new()
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
    image_resource_: RefCell<LayoutImageResource>,
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
            image_resource_: RefCell::new(LayoutImageResource::new()),
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
    pub fn ImageResource(&self) -> std::cell::Ref<'_, LayoutImageResource> {
        self.CheckIsNotDestroyed();
        self.image_resource_.borrow()
    }
    pub fn UpdateImageResource(&self, update: image_resource::ImageResourceContentUpdate) -> bool {
        self.CheckIsNotDestroyed();
        self.image_resource_.borrow_mut().Update(update)
    }
    pub fn UseBrokenImage(
        &self,
        variants: image_resource::platform_resources::BrokenImageResourceVariants,
        device_scale_factor: f64,
    ) -> bool {
        let content = if device_scale_factor >= 2.0 {
            variants.scale200
        } else {
            variants.scale100
        };
        self.UpdateImageResource(image_resource::ImageResourceContentUpdate {
            content: Some(content),
            complete: true,
        })
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
    // Source: core/layout/layout_image.cc:208-221 and
    // LayoutImageResource::ImageSize. The resource owner, not DOM projection,
    // is authoritative once ImageLoader has installed a content identity.
    pub fn UpdateNaturalDimensionsFromImageResource(&self) {
        self.CheckIsNotDestroyed();
        let resource = self.ImageResource();
        if resource.ErrorOccurred() {
            return;
        }
        let Some(metadata) = resource.Metadata() else {
            return;
        };
        let respect_orientation = self.StyleRef().ImageOrientation()
            == foundation::RespectImageOrientationEnum::kRespectImageOrientation;
        let (width, height) = metadata.NaturalSize(respect_orientation);
        let scale = self.ImageDevicePixelRatio() as f64;
        let zoom = self.StyleRef().EffectiveZoom();
        let mut sizing_info = PhysicalNaturalSizingInfo::None();
        sizing_info.has_width = true;
        sizing_info.has_height = true;
        sizing_info.size = PhysicalSize::new(
            LayoutUnit::from_f64(width * scale * zoom as f64),
            LayoutUnit::from_f64(height * scale * zoom as f64),
        );
        if !sizing_info.size.IsEmpty() {
            sizing_info.aspect_ratio = sizing_info.size;
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

// Source: core/layout/layout_media.h:46-96, layout_video.h:42-102.
// Keep distinct derived allocations and the native child-list/cache members.
// All base subobjects have offset zero, as required by assembled C++ virtuals.
#[repr(C)]
pub struct LayoutMedia {
    image_: LayoutInputReplaced,
    children_: UnsafeCell<super::super::layout_object_child_list::LayoutObjectChildList>,
}
impl LayoutMedia {
    pub fn new(element: *mut Element) -> Self {
        Self {
            image_: LayoutInputReplaced::new(element),
            children_: UnsafeCell::new(Default::default()),
        }
    }
    pub fn Children(&self) -> *mut super::super::layout_object_child_list::LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        self.children_.get()
    }
    // Source: core/layout/layout_media.h:53-56.
    pub fn CanHaveChildren(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
    pub fn GetName(&self) -> &'static str {
        if self.IsVideo() {
            "LayoutVideo"
        } else {
            "LayoutMedia"
        }
    }
    pub fn ComputePanelWidth(&self, media_rect: &PhysicalRect) -> LayoutUnit {
        unsafe { LayoutMediaComputePanelWidth(self, media_rect) }
    }
    pub fn IsChildAllowed(&self, child: &LayoutObject, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        if style.HasOutOfFlowPosition()
            || (style.IsFloating() && !style.IsInsideDisplayIgnoringFloatingChildren())
        {
            return false;
        }
        let node = child.GetNode();
        assert!(!node.is_null());
        let element = DynamicTo::<Element>(node);
        if element.is_null() {
            return false;
        }
        match unsafe { &*element }
            .InputElementData()
            .as_ref()
            .and_then(|data| data.media_child_kind)
        {
            Some(super::super::layout_input::MediaChildKind::Controls) => child.IsFlexibleBox(),
            Some(
                super::super::layout_input::MediaChildKind::TextTrackContainer
                | super::super::layout_input::MediaChildKind::RemotingInterstitial
                | super::super::layout_input::MediaChildKind::PictureInPictureInterstitial,
            ) => true,
            None => false,
        }
    }
}
impl Deref for LayoutMedia {
    type Target = LayoutInputReplaced;
    fn deref(&self) -> &Self::Target {
        &self.image_
    }
}
impl DerefMut for LayoutMedia {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image_
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoDisplayMode {
    Poster,
    Video,
}
#[repr(C)]
pub struct LayoutVideo {
    media_: LayoutMedia,
    natural_dimensions_: RefCell<PhysicalNaturalSizingInfo>,
}
impl LayoutVideo {
    pub fn new(element: *mut Element) -> Self {
        Self {
            media_: LayoutMedia::new(element),
            natural_dimensions_: RefCell::new(PhysicalNaturalSizingInfo::None()),
        }
    }
    pub fn ElementState(&self) -> super::super::layout_input::VideoLayoutState {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        assert!(!element.is_null());
        unsafe { &*element }
            .InputElementData()
            .as_ref()
            .and_then(|data| data.video)
            .expect("HTMLVideoElement publishes its player/poster layout facts")
    }
    // Source: core/layout/layout_video.cc:119-132.
    pub fn GetDisplayMode(&self) -> VideoDisplayMode {
        let state = self.ElementState();
        if (state.show_poster || !state.has_available_video_frame) && state.poster_url_present {
            VideoDisplayMode::Poster
        } else {
            VideoDisplayMode::Video
        }
    }
    pub fn CachedNaturalDimensions(&self) -> PhysicalNaturalSizingInfo {
        self.natural_dimensions_.borrow().clone()
    }
    pub fn SetCachedNaturalDimensions(&self, sizing: PhysicalNaturalSizingInfo) {
        *self.natural_dimensions_.borrow_mut() = sizing;
    }
    pub fn PosterNaturalDimensions(&self) -> PhysicalNaturalSizingInfo {
        self.media_.image_.NaturalDimensions()
    }
    pub fn RefreshPosterNaturalDimensions(&self) {
        self.media_.image_.UpdateNaturalDimensionsFromElementInput();
    }
    pub fn UpdateFromElement(&mut self) {
        unsafe { LayoutVideoUpdateFromElement(self) }
    }
    pub fn InvalidateCompositing(&mut self) {
        unsafe { LayoutVideoInvalidateCompositing(self) }
    }
}
impl Deref for LayoutVideo {
    type Target = LayoutMedia;
    fn deref(&self) -> &Self::Target {
        &self.media_
    }
}
impl DerefMut for LayoutVideo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.media_
    }
}
unsafe extern "Rust" {
    fn LayoutMediaComputePanelWidth(this: &LayoutMedia, media_rect: &PhysicalRect) -> LayoutUnit;
    fn LayoutVideoUpdateFromElement(this: &mut LayoutVideo);
    fn LayoutVideoInvalidateCompositing(this: &mut LayoutVideo);
}
const _: () = assert!(std::mem::offset_of!(LayoutMedia, image_) == 0);
const _: () = assert!(std::mem::offset_of!(LayoutVideo, media_) == 0);
