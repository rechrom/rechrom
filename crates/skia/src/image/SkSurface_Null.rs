// Copyright 2023 Google LLC. BSD-style license.
use super::{
    SkImage::SkImage,
    SkSurface::{ContentChangeMode, SkSurface, SkSurfaceState},
    SkSurface_Base::{SkSurface_Base, SurfaceDispatch, Type},
};
use crate::include::core::SkSamplingOptions::SkSamplingOptions;
use crate::include::core::{
    SkCapabilities::SkCapabilities,
    SkRefCnt::{sk_sp, SkRefCntBase, SkRefCntObject},
};
use crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas;
use crate::src::core::{
    SkCanvas::{SkCanvas, SkCanvasOwnerHandle},
    SkImageInfo::SkImageInfo,
    SkPaint::SkPaint,
    SkPixmap::SkPixmap,
    SkRect::SkIRect,
};
use core::{pin::Pin, ptr::NonNull};
use std::sync::Arc;

// Source: src/image/SkSurface_Null.cpp:23-25
// The complete derived allocation remains pinned through intrusive ownership;
// its embedded Base is projected, never extracted into another owning object.
struct SkNullSurface {
    base: SkSurface_Base<'static>,
}
impl SkNullSurface {
    fn new(width: i32, height: i32) -> Pin<Box<Self>> {
        let mut owner = Box::pin(Self {
            base: SkSurface_Base::new(width, height, None, None),
        });
        let whole = unsafe { owner.as_mut().get_unchecked_mut() } as *mut Self;
        let base = unsafe { NonNull::new_unchecked(core::ptr::addr_of_mut!((*whole).base)) };
        unsafe {
            SkSurface_Base::bind_raw(
                base,
                SurfaceDispatch {
                    // Actual inherited native Base null/false ordinary virtuals.
                    on_get_recording_context: None,
                    on_get_graphite_recorder: None,
                    on_wait: None,
                    on_characterize: None,
                    on_is_compatible: None,
                    on_replace_backend_texture: None,
                    on_capabilities: Self::onCapabilities,
                    on_get_base_recorder: None,
                    type_: Self::type_callback,
                    on_new_canvas: Self::onNewCanvas,
                    on_new_surface: Self::onNewSurface,
                    on_write_pixels: Self::onWritePixels,
                    on_async_rescale_pixels: None,
                    on_async_rescale_yuv420: None,
                    on_make_temporary_image: None,
                    on_draw: Some(Self::onDraw),
                    on_copy_on_write: Self::onCopyOnWrite,
                    on_restore_backing_mutability: None,
                    on_discard: None,
                    on_new_image_snapshot: Some(Self::onNewImageSnapshot),
                },
            );
        }
        owner
    }
    // Source: src/image/SkSurface_Null.cpp:28-28
    unsafe fn type_callback(_base: NonNull<SkSurface_Base<'static>>) -> Type {
        Type::kNull
    }
    // Source: src/image/SkSurface_Null.cpp:31-33
    unsafe fn onNewCanvas(
        base: NonNull<SkSurface_Base<'static>>,
    ) -> Option<SkCanvasOwnerHandle<'static>> {
        // Only scalar metadata is borrowed before construction; the returned
        // polymorphic owner retains the actual SkNoDrawCanvas and its hooks.
        let (width, height) = {
            let state = unsafe { &*core::ptr::addr_of!((*base.as_ptr()).surface) };
            (state.width(), state.height())
        };
        let owner: SkCanvasOwnerHandle<'static> = SkNoDrawCanvas::new(width, height);
        Some(owner)
    }
    // Source: src/image/SkSurface_Null.cpp:34-36
    unsafe fn onNewSurface(
        _base: NonNull<SkSurface_Base<'static>>,
        info: &SkImageInfo,
    ) -> Option<sk_sp<dyn SkSurface<'static> + 'static>> {
        Null(info.width(), info.height())
    }
    // Source: src/image/SkSurface_Null.cpp:37-37
    unsafe fn onNewImageSnapshot(
        _base: NonNull<SkSurface_Base<'static>>,
        _subset: Option<&SkIRect>,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        Ok(None)
    }
    // Source: src/image/SkSurface_Null.cpp:38-38
    unsafe fn onWritePixels(
        _base: NonNull<SkSurface_Base<'static>>,
        _pixmap: &SkPixmap,
        _x: i32,
        _y: i32,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/image/SkSurface_Null.cpp:39-39
    unsafe fn onDraw<'target>(
        _base: NonNull<SkSurface_Base<'static>>,
        _canvas: NonNull<SkCanvas<'target>>,
        _x: f32,
        _y: f32,
        _sampling: &SkSamplingOptions,
        _paint: Option<&SkPaint>,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/image/SkSurface_Null.cpp:40-40
    unsafe fn onCopyOnWrite(
        _base: NonNull<SkSurface_Base<'static>>,
        _mode: ContentChangeMode,
    ) -> bool {
        true
    }
    // Source: src/image/SkSurface_Null.cpp:41-44
    unsafe fn onCapabilities(_base: NonNull<SkSurface_Base<'static>>) -> Arc<SkCapabilities> {
        SkCapabilities::RasterBackend()
    }
}
// The actual native inherited ref-count member and virtual Surface slots belong
// to this same complete allocation; these Rust bridges add no second owner.
unsafe impl SkRefCntObject for SkNullSurface {
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.base.ref_count
    }
    fn ref_(&self) {
        self.base.ref_count.ref_();
    }
    // Adapted Source: src/image/SkSurface_Null.cpp:23-23
    // Concrete destructor dispatch pairs this original complete allocation
    // with its actual Sized type; no trait metadata or owner is reconstructed here.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
}
unsafe impl SkSurface<'static> for SkNullSurface {
    fn state(&self) -> &SkSurfaceState {
        &self.base.surface
    }
    fn base_address(&self) -> NonNull<SkSurface_Base<'static>> {
        self.base.bound_address()
    }
    // Source: src/image/SkSurface_Null.cpp:45-47
    fn imageInfo(&self) -> SkImageInfo {
        SkImageInfo::MakeUnknown(self.base.surface.width(), self.base.surface.height())
    }
}
// Source: src/image/SkSurface_Null.cpp:52-57
pub fn Null(width: i32, height: i32) -> Option<sk_sp<dyn SkSurface<'static> + 'static>> {
    if width < 1 || height < 1 {
        return None;
    }
    let owner = SkNullSurface::new(width, height);
    // Native sk_sp adopts this exact freshly allocated complete object and its
    // one embedded counter. Pin is retained by the intrusive owner's contract.
    let owned = unsafe { Pin::into_inner_unchecked(owner) };
    let concrete = Box::into_raw(owned);
    let dynamic: *mut (dyn SkSurface<'static> + 'static) = concrete;
    Some(unsafe { sk_sp::from_raw(dynamic) })
}
