use crate::include::core::SkRasterHandleAllocator::{Handle, SkRasterHandleAllocator};
use crate::include::core::SkSurfaceProps::SkSurfaceProps;
use crate::src::core::SkBitmap::SkBitmap;
use crate::src::core::SkCanvas::SkCanvas;

// Source: src/core/SkCanvas_Raster.cpp:29-32
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// All bitmap backing, including aliases, must uphold the exclusive live
    /// initialized raster lease throughout the returned Canvas lifetime and Drop.
    pub(crate) unsafe fn with_bitmap_props_impl(
        bitmap: &SkBitmap,
        props: &SkSurfaceProps,
    ) -> core::pin::Pin<Box<Self>> {
        let device = unsafe {
            crate::src::core::SkBitmapDevice::SkBitmapDevice::with_props(
                bitmap,
                props,
                core::ptr::null_mut(),
            )
        };
        unsafe {
            Self::with_device_props_allocator(
                crate::src::core::SkDevice::adopt_pinned_device(device),
                *props,
                None,
            )
        }
    }
}
// Source: src/core/SkCanvas_Raster.cpp:34-42
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the same whole-Canvas pixel lease as with_bitmap_props. The
    /// allocator owns the native allocation contract; handle remains live until
    /// bitmap's release callback and must belong to that allocator when present.
    pub(crate) unsafe fn with_bitmap_allocator_impl(
        bitmap: &SkBitmap,
        allocator: Option<Box<dyn SkRasterHandleAllocator>>,
        handle: Handle,
        props: Option<&SkSurfaceProps>,
    ) -> core::pin::Pin<Box<Self>> {
        let props = crate::src::core::SkSurfacePriv::SkSurfacePropsCopyOrDefault(props);
        let device = unsafe {
            crate::src::core::SkBitmapDevice::SkBitmapDevice::with_props(bitmap, &props, handle)
        };
        unsafe {
            Self::with_device_props_allocator(
                crate::src::core::SkDevice::adopt_pinned_device(device),
                props,
                allocator,
            )
        }
    }
}
// Source: src/core/SkCanvas_Raster.cpp:44-44
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the same whole-Canvas pixel lease as with_bitmap_props.
    pub(crate) unsafe fn with_bitmap_impl(bitmap: &SkBitmap) -> core::pin::Pin<Box<Self>> {
        unsafe { Self::with_bitmap_allocator(bitmap, None, core::ptr::null_mut(), None) }
    }
}

// Source: include/core/SkCanvas.h:229-231
#[cfg(SK_BUILD_FOR_ANDROID_FRAMEWORK)]
#[repr(i32)]
pub enum ColorBehavior {
    kLegacy,
}
// Source: src/core/SkCanvas_Raster.cpp:47-52
#[cfg(SK_BUILD_FOR_ANDROID_FRAMEWORK)]
impl<'device> SkCanvas<'device> {
    /// Safety: bitmap pixels and all aliases uphold the same exclusive raster
    /// lease as the native bitmap constructor through the Canvas lifetime.
    pub unsafe fn with_bitmap_color_behavior(
        bitmap: &SkBitmap,
        _: ColorBehavior,
    ) -> core::pin::Pin<Box<Self>> {
        let mut tmp = bitmap.clone();
        // Native clears only this copied Bitmap's ImageInfo color-space owner;
        // setColorSpace updates the same pixmap metadata without replacing pixels.
        tmp.setColorSpace(None);
        let props = SkSurfaceProps::default();
        let device = unsafe {
            crate::src::core::SkBitmapDevice::SkBitmapDevice::with_props(
                &tmp,
                &props,
                core::ptr::null_mut(),
            )
        };
        unsafe {
            Self::with_device_props_allocator(
                crate::src::core::SkDevice::adopt_pinned_device(device),
                props,
                None,
            )
        }
    }
}
