use std::ffi::c_void;

use foundation::gfx::SizeF;
use foundation::{RespectImageOrientationEnum, ScopedRefPtr, Visitor};

use super::computed_style::ComputedStyle;
use super::forward::{
    CSSValue, CSSValuePhase, Image, ImageResourceContent, ImageResourceObserver, Node,
};
use super::natural_sizing_info::NaturalSizingInfo;

// cpp: layoutng_style/style/style_image.h:47-51
pub type WrappedImagePtr = *const c_void;

// cpp: layoutng_style/style/style_image.h:56-58
// cpp: layoutng_style/style/style_image.h:66-197
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

// cpp: layoutng_style/style/style_image.h:56-58
// cpp: layoutng_style/style/style_image.h:180-200
#[repr(C)]
pub struct StyleImage {
    vtable_: &'static StyleImageVTable,
    flags_: u8,
}

#[allow(non_snake_case)]
impl StyleImage {
    // cpp: layoutng_style/style/style_image.h:180-195
    // Derived implementations embed this base first and supply their vtable.
    pub fn new_for_derived(vtable: &'static StyleImageVTable) -> Self {
        Self {
            vtable_: vtable,
            flags_: 0,
        }
    }

    // cpp: layoutng_style/style/style_image.h:62-66
    pub fn CssValue(&self) -> *mut CSSValue {
        (self.vtable_.CssValue)(self)
    }

    // cpp: layoutng_style/style/style_image.h:68-74
    pub fn ComputedCSSValue(
        &self,
        style: &ComputedStyle,
        allow_visited_style: bool,
        value_phase: CSSValuePhase,
    ) -> *mut CSSValue {
        (self.vtable_.ComputedCSSValue)(self, style, allow_visited_style, value_phase)
    }

    // cpp: layoutng_style/style/style_image.h:76-77
    pub fn CanRender(&self) -> bool {
        (self.vtable_.CanRender)(self)
    }

    // cpp: layoutng_style/style/style_image.h:79-80
    pub fn IsLoaded(&self) -> bool {
        (self.vtable_.IsLoaded)(self)
    }

    // cpp: layoutng_style/style/style_image.h:82-83
    pub fn IsLoading(&self) -> bool {
        (self.vtable_.IsLoading)(self)
    }

    // cpp: layoutng_style/style/style_image.h:85-86
    pub fn ErrorOccurred(&self) -> bool {
        (self.vtable_.ErrorOccurred)(self)
    }

    // cpp: layoutng_style/style/style_image.h:88-89
    pub fn IsCorsSameOrigin(&self) -> bool {
        (self.vtable_.IsCorsSameOrigin)(self)
    }

    // cpp: layoutng_style/style/style_image.h:91-98
    pub fn GetNaturalSizingInfo(
        &self,
        multiplier: f32,
        orientation: RespectImageOrientationEnum,
    ) -> NaturalSizingInfo {
        (self.vtable_.GetNaturalSizingInfo)(self, multiplier, orientation)
    }

    // cpp: layoutng_style/style/style_image.h:100-119
    pub fn ImageSize(
        &self,
        multiplier: f32,
        default_object_size: &SizeF,
        orientation: RespectImageOrientationEnum,
    ) -> SizeF {
        (self.vtable_.ImageSize)(self, multiplier, default_object_size, orientation)
    }

    // cpp: layoutng_style/style/style_image.h:121-125
    pub fn HasIntrinsicSize(&self) -> bool {
        (self.vtable_.HasIntrinsicSize)(self)
    }

    // cpp: layoutng_style/style/style_image.h:127
    pub fn AddClient(&mut self, client: *mut ImageResourceObserver) {
        (self.vtable_.AddClient)(self, client)
    }

    // cpp: layoutng_style/style/style_image.h:128
    pub fn RemoveClient(&mut self, client: *mut ImageResourceObserver) {
        (self.vtable_.RemoveClient)(self, client)
    }

    // cpp: layoutng_style/style/style_image.h:130-144
    pub fn GetImage(
        &self,
        observer: &ImageResourceObserver,
        node: &Node,
        style: &ComputedStyle,
        target_size: &SizeF,
    ) -> ScopedRefPtr<Image> {
        (self.vtable_.GetImage)(self, observer, node, style, target_size)
    }

    // cpp: layoutng_style/style/style_image.h:146-147
    pub fn Data(&self) -> WrappedImagePtr {
        (self.vtable_.Data)(self)
    }

    // cpp: layoutng_style/style/style_image.h:149-151
    pub fn ImageScaleFactor(&self) -> f32 {
        (self.vtable_.ImageScaleFactor)(self)
    }

    // cpp: layoutng_style/style/style_image.h:153-155
    pub fn CachedImage(&self) -> *mut ImageResourceContent {
        (self.vtable_.CachedImage)(self)
    }

    // cpp: layoutng_style/style/style_image.h:157-160
    // No definition exists in the supplied C++ tree.
    pub fn ForceOrientationIfNecessary(
        &self,
        orientation: RespectImageOrientationEnum,
    ) -> RespectImageOrientationEnum {
        unsafe { StyleImageForceOrientationIfNecessary(self, orientation) }
    }

    // cpp: layoutng_style/style/style_image.h:162-163
    pub fn DependsOnCurrentColor(&self) -> bool {
        (self.vtable_.DependsOnCurrentColor)(self)
    }

    // cpp: layoutng_style/style/style_image.h:165
    pub fn IsImageResource(&self) -> bool {
        self.flags_ & IMAGE_RESOURCE != 0
    }

    // cpp: layoutng_style/style/style_image.h:166
    pub fn IsPendingImage(&self) -> bool {
        self.flags_ & PENDING_IMAGE != 0
    }

    // cpp: layoutng_style/style/style_image.h:167
    pub fn IsGeneratedImage(&self) -> bool {
        self.flags_ & GENERATED_IMAGE != 0
    }

    // cpp: layoutng_style/style/style_image.h:168
    pub fn IsContentful(&self) -> bool {
        !self.IsGeneratedImage()
    }

    // cpp: layoutng_style/style/style_image.h:169-171
    pub fn IsImageResourceSet(&self) -> bool {
        self.flags_ & IMAGE_RESOURCE_SET != 0
    }

    // cpp: layoutng_style/style/style_image.h:172
    pub fn IsMaskSource(&self) -> bool {
        self.flags_ & MASK_SOURCE != 0
    }

    // cpp: layoutng_style/style/style_image.h:173
    pub fn IsPaintImage(&self) -> bool {
        self.flags_ & PAINT_IMAGE != 0
    }

    // cpp: layoutng_style/style/style_image.h:174
    pub fn IsCrossfadeImage(&self) -> bool {
        self.flags_ & CROSSFADE != 0
    }

    // cpp: layoutng_style/style/style_image.h:176
    pub fn IsLoadedAfterMouseover(&self) -> bool {
        (self.vtable_.IsLoadedAfterMouseover)(self)
    }

    // cpp: layoutng_style/style/style_image.h:178
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

    // cpp: layoutng_style/style/style_image.h:189
    pub fn set_image_resource_for_derived(&mut self, value: bool) {
        self.set_flag(IMAGE_RESOURCE, value);
    }

    // cpp: layoutng_style/style/style_image.h:190
    pub fn set_pending_image_for_derived(&mut self, value: bool) {
        self.set_flag(PENDING_IMAGE, value);
    }

    // cpp: layoutng_style/style/style_image.h:191
    pub fn set_generated_image_for_derived(&mut self, value: bool) {
        self.set_flag(GENERATED_IMAGE, value);
    }

    // cpp: layoutng_style/style/style_image.h:192
    pub fn set_image_resource_set_for_derived(&mut self, value: bool) {
        self.set_flag(IMAGE_RESOURCE_SET, value);
    }

    // cpp: layoutng_style/style/style_image.h:193
    pub fn set_crossfade_for_derived(&mut self, value: bool) {
        self.set_flag(CROSSFADE, value);
    }

    // cpp: layoutng_style/style/style_image.h:194
    pub fn set_mask_source_for_derived(&mut self, value: bool) {
        self.set_flag(MASK_SOURCE, value);
    }

    // cpp: layoutng_style/style/style_image.h:195
    pub fn set_paint_image_for_derived(&mut self, value: bool) {
        self.set_flag(PAINT_IMAGE, value);
    }

    // cpp: layoutng_style/style/style_image.h:199
    // No definition exists in the supplied C++ tree.
    pub fn ApplyZoom(size: &SizeF, multiplier: f32) -> SizeF {
        unsafe { StyleImageApplyZoom(size, multiplier) }
    }
}

// cpp: layoutng_style/style/style_image.h:60
// cpp: layoutng_style/style/style_image.h:197
impl PartialEq for StyleImage {
    fn eq(&self, other: &Self) -> bool {
        (self.vtable_.IsEqual)(self, other)
    }
}

// cpp: layoutng_style/style/style_image.h:77
#[allow(non_snake_case)]
pub fn StyleImageDefaultCanRender(_: &StyleImage) -> bool {
    true
}

// cpp: layoutng_style/style/style_image.h:80
#[allow(non_snake_case)]
pub fn StyleImageDefaultIsLoaded(_: &StyleImage) -> bool {
    true
}

// cpp: layoutng_style/style/style_image.h:83
#[allow(non_snake_case)]
pub fn StyleImageDefaultIsLoading(_: &StyleImage) -> bool {
    false
}

// cpp: layoutng_style/style/style_image.h:86
#[allow(non_snake_case)]
pub fn StyleImageDefaultErrorOccurred(_: &StyleImage) -> bool {
    false
}

// cpp: layoutng_style/style/style_image.h:151
#[allow(non_snake_case)]
pub fn StyleImageDefaultImageScaleFactor(_: &StyleImage) -> f32 {
    1.0
}

// cpp: layoutng_style/style/style_image.h:155
#[allow(non_snake_case)]
pub fn StyleImageDefaultCachedImage(_: &StyleImage) -> *mut ImageResourceContent {
    std::ptr::null_mut()
}

// cpp: layoutng_style/style/style_image.h:163
#[allow(non_snake_case)]
pub fn StyleImageDefaultDependsOnCurrentColor(_: &StyleImage) -> bool {
    false
}

// cpp: layoutng_style/style/style_image.h:176
#[allow(non_snake_case)]
pub fn StyleImageDefaultIsLoadedAfterMouseover(_: &StyleImage) -> bool {
    false
}

// cpp: layoutng_style/style/style_image.h:178
#[allow(non_snake_case)]
pub fn StyleImageDefaultTrace(_: &StyleImage, _: &mut Visitor) {}

// cpp: layoutng_style/style/style_image.h:202-204
#[allow(non_snake_case)]
pub fn EqualResolutions(res1: f32, res2: f32) -> bool {
    (res1 - res2).abs() < f32::EPSILON
}

unsafe extern "Rust" {
    fn StyleImageForceOrientationIfNecessary(
        value: &StyleImage,
        orientation: RespectImageOrientationEnum,
    ) -> RespectImageOrientationEnum;
    fn StyleImageApplyZoom(size: &SizeF, multiplier: f32) -> SizeF;
}
