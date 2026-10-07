#![allow(non_snake_case, non_camel_case_types)]

use foundation::{gfx, InterpolationQuality, String as BlinkString, Visitor};

use crate::internal::loader::fetch::resource_priority::ResourcePriority;

// cpp: layoutng/internal/loader/resource/image_resource_observer.h:37-37
pub type WrappedImagePtr = *const std::ffi::c_void;

// cpp: layoutng/internal/loader/resource/image_resource_observer.h:44-51
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanDeferInvalidation {
    kYes,
    kNo,
}

// ImageResourceContent and ImageAnimationPolicy are only forward-declared by
// the source header. Concrete observers supply the owning types when their
// external resource and settings dependencies are connected.
// cpp: layoutng/internal/loader/resource/image_resource_observer.h:38-114
pub trait ImageResourceObserver {
    type ImageResourceContent;
    type ImageAnimationPolicy;

    // C++ overloads are named separately because Rust does not overload by
    // argument type. Both preserve the original empty virtual default.
    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:57-57
    fn ImageChangedContent(
        &mut self,
        _image: *mut Self::ImageResourceContent,
        _can_defer: CanDeferInvalidation,
    ) {
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:61-61
    fn ImageChangedWrapped(&mut self, _image: WrappedImagePtr, _can_defer: CanDeferInvalidation) {}

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:65-65
    fn ImageNotifyFinished(&mut self, _image: *mut Self::ImageResourceContent) {}

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:69-69
    fn NotifyImageFullyRemoved(&mut self, _image: *mut Self::ImageResourceContent) {}

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:79-79
    fn WillRenderImage(&mut self) -> bool {
        false
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:83-85
    fn GetImageAnimationPolicy(&mut self, _policy: &mut Self::ImageAnimationPolicy) -> bool {
        false
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:90-92
    fn ComputeResourcePriority(&self) -> ResourcePriority {
        ResourcePriority::default()
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:94-96
    fn CachedResourcePriority(&self) -> Option<ResourcePriority> {
        None
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:98-98
    fn CanBeSpeculativelyDecoded(&self) -> bool {
        true
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:99-99
    fn ComputeSpeculativeDecodeSize(&self) -> gfx::Size {
        gfx::Size::default()
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:100-100
    fn CachedSpeculativeDecodeSize(&self) -> gfx::Size {
        gfx::Size::default()
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:101-103
    fn ComputeSpeculativeDecodeQuality(&self) -> InterpolationQuality {
        foundation::kInterpolationNone
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:104-106
    fn CachedSpeculativeDecodeQuality(&self) -> InterpolationQuality {
        foundation::kInterpolationNone
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:108-109
    fn DebugName(&self) -> BlinkString;

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:111-111
    fn IsExpectedType(_observer: *mut Self) -> bool
    where
        Self: Sized,
    {
        true
    }

    // cpp: layoutng/internal/loader/resource/image_resource_observer.h:113-113
    fn Trace(&self, _visitor: &mut Visitor) {}
}
