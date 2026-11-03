use std::ffi::c_void;

use foundation::gfx::{ScaleSizeUniform, SizeF};
use foundation::{RespectImageOrientationEnum, ScopedRefPtr, Visitor};

use super::computed_style::ComputedStyle;
use super::forward::{
    CSSValue, CSSValuePhase, Image, ImageResourceContent, ImageResourceObserver, Node,
};
use super::natural_sizing_info::NaturalSizingInfo;

// cpp: core/style/style_image.h:48-52
pub type WrappedImagePtr = *const c_void;

// cpp: core/style/style_image.h:57-59
// cpp: core/style/style_image.h:63-204
// Rust's explicit table keeps a thin StyleImage base pointer while retaining
// virtual dispatch for implementations in other packages.
#[allow(non_snake_case)]
pub struct StyleImageVTable {
    pub CssValue: fn(&StyleImage) -> *mut CSSValue,
    pub ComputedCSSValue: fn(&StyleImage, &ComputedStyle, bool, CSSValuePhase) -> *mut CSSValue,
    pub CanRender: fn(&StyleImage) -> bool,
    pub IsLoaded: fn(&StyleImage) -> bool,
    pub IsLoading: fn(&StyleImage) -> bool,
    pub ErrorOccurred: fn(&StyleImage) -> bool,
    pub IsCorsSameOrigin: fn(&StyleImage) -> bool,
    pub GetNaturalSizingInfo:
        fn(&StyleImage, f32, RespectImageOrientationEnum) -> NaturalSizingInfo,
    pub ImageSize: fn(&StyleImage, f32, &SizeF, RespectImageOrientationEnum) -> SizeF,
    pub HasIntrinsicSize: fn(&StyleImage) -> bool,
    pub AddClient: fn(&mut StyleImage, *mut ImageResourceObserver),
    pub RemoveClient: fn(&mut StyleImage, *mut ImageResourceObserver),
    pub GetImage: fn(
        &StyleImage,
        &ImageResourceObserver,
        &Node,
        &ComputedStyle,
        &SizeF,
    ) -> ScopedRefPtr<Image>,
    pub Data: fn(&StyleImage) -> WrappedImagePtr,
    pub ImageScaleFactor: fn(&StyleImage) -> f32,
    pub KnownToBeOpaque: fn(&StyleImage, &ComputedStyle) -> bool,
    pub CachedImage: fn(&StyleImage) -> *mut ImageResourceContent,
    pub DependsOnCurrentColor: fn(&StyleImage) -> bool,
    pub IsLoadedAfterMouseover: fn(&StyleImage) -> bool,
    pub Trace: fn(&StyleImage, &mut Visitor),
    pub IsEqual: fn(&StyleImage, &StyleImage) -> bool,
}

const IMAGE_RESOURCE: u8 = 1 << 0;
const PENDING_IMAGE: u8 = 1 << 1;
const GENERATED_IMAGE: u8 = 1 << 2;
const IMAGE_RESOURCE_SET: u8 = 1 << 3;
const CROSSFADE: u8 = 1 << 4;
const MASK_SOURCE: u8 = 1 << 5;
const PAINT_IMAGE: u8 = 1 << 6;

// cpp: core/style/style_image.h:57-59
// cpp: core/style/style_image.h:185-205
#[repr(C)]
pub struct StyleImage {
    vtable_: &'static StyleImageVTable,
    flags_: u8,
}

#[allow(non_snake_case)]
impl StyleImage {
    // cpp: core/style/style_image.h:186-200
    // Derived implementations embed this base first and supply their vtable.
    pub fn new_for_derived(vtable: &'static StyleImageVTable) -> Self {
        Self {
            vtable_: vtable,
            flags_: 0,
        }
    }

    // cpp: core/style/style_image.h:63-67
    pub fn CssValue(&self) -> *mut CSSValue {
        (self.vtable_.CssValue)(self)
    }

    // cpp: core/style/style_image.h:69-75
    pub fn ComputedCSSValue(
        &self,
        style: &ComputedStyle,
        allow_visited_style: bool,
        value_phase: CSSValuePhase,
    ) -> *mut CSSValue {
        (self.vtable_.ComputedCSSValue)(self, style, allow_visited_style, value_phase)
    }

    // cpp: core/style/style_image.h:77-78
    pub fn CanRender(&self) -> bool {
        (self.vtable_.CanRender)(self)
    }

    // cpp: core/style/style_image.h:80-81
    pub fn IsLoaded(&self) -> bool {
        (self.vtable_.IsLoaded)(self)
    }

    // cpp: core/style/style_image.h:83-84
    pub fn IsLoading(&self) -> bool {
        (self.vtable_.IsLoading)(self)
    }

    // cpp: core/style/style_image.h:86-87
    pub fn ErrorOccurred(&self) -> bool {
        (self.vtable_.ErrorOccurred)(self)
    }

    // cpp: core/style/style_image.h:89-90
    pub fn IsCorsSameOrigin(&self) -> bool {
        (self.vtable_.IsCorsSameOrigin)(self)
    }

    // cpp: core/style/style_image.h:92-99
    pub fn GetNaturalSizingInfo(
        &self,
        multiplier: f32,
        orientation: RespectImageOrientationEnum,
    ) -> NaturalSizingInfo {
        (self.vtable_.GetNaturalSizingInfo)(self, multiplier, orientation)
    }

    // cpp: core/style/style_image.h:101-120
    pub fn ImageSize(
        &self,
        multiplier: f32,
        default_object_size: &SizeF,
        orientation: RespectImageOrientationEnum,
    ) -> SizeF {
        (self.vtable_.ImageSize)(self, multiplier, default_object_size, orientation)
    }

    // cpp: core/style/style_image.h:122-126
    pub fn HasIntrinsicSize(&self) -> bool {
        (self.vtable_.HasIntrinsicSize)(self)
    }

    // cpp: core/style/style_image.h:128
    pub fn AddClient(&mut self, client: *mut ImageResourceObserver) {
        (self.vtable_.AddClient)(self, client)
    }

    // cpp: core/style/style_image.h:129
    pub fn RemoveClient(&mut self, client: *mut ImageResourceObserver) {
        (self.vtable_.RemoveClient)(self, client)
    }

    // cpp: core/style/style_image.h:131-145
    pub fn GetImage(
        &self,
        observer: &ImageResourceObserver,
        node: &Node,
        style: &ComputedStyle,
        target_size: &SizeF,
    ) -> ScopedRefPtr<Image> {
        (self.vtable_.GetImage)(self, observer, node, style, target_size)
    }

    // cpp: core/style/style_image.h:147-148
    pub fn Data(&self) -> WrappedImagePtr {
        (self.vtable_.Data)(self)
    }

    // cpp: core/style/style_image.h:150-152
    pub fn ImageScaleFactor(&self) -> f32 {
        (self.vtable_.ImageScaleFactor)(self)
    }

    // cpp: core/style/style_image.h:154-156. Document-owned resource facts
    // are resolved before constructing the source StyleImage; generated
    // images still resolve their colors against this ComputedStyle.
    pub fn KnownToBeOpaque(&self, style: &ComputedStyle) -> bool {
        (self.vtable_.KnownToBeOpaque)(self, style)
    }

    // cpp: core/style/style_image.h:158-160
    pub fn CachedImage(&self) -> *mut ImageResourceContent {
        (self.vtable_.CachedImage)(self)
    }

    // cpp: core/style/style_image.cc:11-19
    pub fn ForceOrientationIfNecessary(
        &self,
        default_orientation: RespectImageOrientationEnum,
    ) -> RespectImageOrientationEnum {
        if default_orientation == RespectImageOrientationEnum::kRespectImageOrientation {
            return RespectImageOrientationEnum::kRespectImageOrientation;
        }
        debug_assert_eq!(
            default_orientation,
            RespectImageOrientationEnum::kDoNotRespectImageOrientation
        );
        if self.IsCorsSameOrigin() {
            RespectImageOrientationEnum::kDoNotRespectImageOrientation
        } else {
            RespectImageOrientationEnum::kRespectImageOrientation
        }
    }

    // cpp: core/style/style_image.h:167-168
    pub fn DependsOnCurrentColor(&self) -> bool {
        (self.vtable_.DependsOnCurrentColor)(self)
    }

    // cpp: core/style/style_image.h:170
    pub fn IsImageResource(&self) -> bool {
        self.flags_ & IMAGE_RESOURCE != 0
    }

    // cpp: core/style/style_image.h:171
    pub fn IsPendingImage(&self) -> bool {
        self.flags_ & PENDING_IMAGE != 0
    }

    // cpp: core/style/style_image.h:172
    pub fn IsGeneratedImage(&self) -> bool {
        self.flags_ & GENERATED_IMAGE != 0
    }

    // cpp: core/style/style_image.h:173
    pub fn IsContentful(&self) -> bool {
        !self.IsGeneratedImage()
    }

    // cpp: core/style/style_image.h:174-176
    pub fn IsImageResourceSet(&self) -> bool {
        self.flags_ & IMAGE_RESOURCE_SET != 0
    }

    // cpp: core/style/style_image.h:177
    pub fn IsMaskSource(&self) -> bool {
        self.flags_ & MASK_SOURCE != 0
    }

    // cpp: core/style/style_image.h:178
    pub fn IsPaintImage(&self) -> bool {
        self.flags_ & PAINT_IMAGE != 0
    }

    // cpp: core/style/style_image.h:179
    pub fn IsCrossfadeImage(&self) -> bool {
        self.flags_ & CROSSFADE != 0
    }

    // cpp: core/style/style_image.h:181
    pub fn IsLoadedAfterMouseover(&self) -> bool {
        (self.vtable_.IsLoadedAfterMouseover)(self)
    }

    // cpp: core/style/style_image.h:183
    pub fn Trace(&self, visitor: &mut Visitor) {
        (self.vtable_.Trace)(self, visitor)
    }

    fn set_flag(&mut self, mask: u8, value: bool) {
        if value {
            self.flags_ |= mask;
        } else {
            self.flags_ &= !mask;
        }
    }

    // cpp: core/style/style_image.h:187
    pub fn set_image_resource_for_derived(&mut self, value: bool) {
        self.set_flag(IMAGE_RESOURCE, value);
    }

    // cpp: core/style/style_image.h:188
    pub fn set_pending_image_for_derived(&mut self, value: bool) {
        self.set_flag(PENDING_IMAGE, value);
    }

    // cpp: core/style/style_image.h:189
    pub fn set_generated_image_for_derived(&mut self, value: bool) {
        self.set_flag(GENERATED_IMAGE, value);
    }

    // cpp: core/style/style_image.h:190
    pub fn set_image_resource_set_for_derived(&mut self, value: bool) {
        self.set_flag(IMAGE_RESOURCE_SET, value);
    }

    // cpp: core/style/style_image.h:191
    pub fn set_crossfade_for_derived(&mut self, value: bool) {
        self.set_flag(CROSSFADE, value);
    }

    // cpp: core/style/style_image.h:192
    pub fn set_mask_source_for_derived(&mut self, value: bool) {
        self.set_flag(MASK_SOURCE, value);
    }

    // cpp: core/style/style_image.h:193
    pub fn set_paint_image_for_derived(&mut self, value: bool) {
        self.set_flag(PAINT_IMAGE, value);
    }

    // cpp: core/style/style_image.cc:21-38
    pub fn ApplyZoom(size: &SizeF, multiplier: f32) -> SizeF {
        if multiplier == 1.0 {
            return *size;
        }

        let mut scaled_size = ScaleSizeUniform(*size, multiplier);

        // Don't let images that have a width/height >= 1 shrink below 1 when zoomed.
        if size.width() > 0.0 {
            scaled_size.set_width(1.0_f32.max(scaled_size.width()));
        }

        if size.height() > 0.0 {
            scaled_size.set_height(1.0_f32.max(scaled_size.height()));
        }

        scaled_size
    }
}

// cpp: core/style/style_image.h:61
// cpp: core/style/style_image.h:202
impl PartialEq for StyleImage {
    fn eq(&self, other: &Self) -> bool {
        (self.vtable_.IsEqual)(self, other)
    }
}

// cpp: core/style/style_image.h:78
#[allow(non_snake_case)]
pub fn StyleImageDefaultCanRender(_: &StyleImage) -> bool {
    true
}

// cpp: core/style/style_image.h:81
#[allow(non_snake_case)]
pub fn StyleImageDefaultIsLoaded(_: &StyleImage) -> bool {
    true
}

// cpp: core/style/style_image.h:84
#[allow(non_snake_case)]
pub fn StyleImageDefaultIsLoading(_: &StyleImage) -> bool {
    false
}

// cpp: core/style/style_image.h:87
#[allow(non_snake_case)]
pub fn StyleImageDefaultErrorOccurred(_: &StyleImage) -> bool {
    false
}

// cpp: core/style/style_image.h:152
#[allow(non_snake_case)]
pub fn StyleImageDefaultImageScaleFactor(_: &StyleImage) -> f32 {
    1.0
}

// cpp: core/style/style_image.h:160
#[allow(non_snake_case)]
pub fn StyleImageDefaultCachedImage(_: &StyleImage) -> *mut ImageResourceContent {
    std::ptr::null_mut()
}

// cpp: core/style/style_image.h:168
#[allow(non_snake_case)]
pub fn StyleImageDefaultDependsOnCurrentColor(_: &StyleImage) -> bool {
    false
}

// cpp: core/style/style_image.h:181
#[allow(non_snake_case)]
pub fn StyleImageDefaultIsLoadedAfterMouseover(_: &StyleImage) -> bool {
    false
}

// cpp: core/style/style_image.h:183
#[allow(non_snake_case)]
pub fn StyleImageDefaultTrace(_: &StyleImage, _: &mut Visitor) {}

// cpp: core/style/style_image.h:207-209
#[allow(non_snake_case)]
pub fn EqualResolutions(res1: f32, res2: f32) -> bool {
    (res1 - res2).abs() < f32::EPSILON
}
