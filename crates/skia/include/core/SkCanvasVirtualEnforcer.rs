use crate::src::core::SkCanvas::{
    ImageSetEntry, Lattice, PointMode, QuadAAFlags, SkCanvas, SrcRectConstraint,
};

pub use crate::src::core::SkCanvas::{SkCanvasOwner, SkCanvasOwnerHandle, SkCanvasVirtual};

// Source: include/core/SkCanvasVirtualEnforcer.h:16-19
// Template inheritance becomes a supertrait; constructors stay on concrete
// pinned owners. Required hooks have no fallback and are checked by Rust.
pub trait SkCanvasVirtualEnforcer<'device>: SkCanvasVirtual<'device> {
    // Source: include/core/SkCanvasVirtualEnforcer.h:22-22
    unsafe fn onDrawPaint(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:23-23
    unsafe fn onDrawBehind(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/core/SkCanvasVirtualEnforcer.h:24-24
    unsafe fn onDrawRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:25-25
    unsafe fn onDrawRRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:26-27
    unsafe fn onDrawDRRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        outer: &crate::src::core::SkRRect::SkRRect,
        inner: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:28-28
    unsafe fn onDrawOval(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        oval: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:29-30
    unsafe fn onDrawArc(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        oval: &crate::src::core::SkRect::SkRect,
        start: f32,
        sweep: f32,
        use_center: bool,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:31-31
    unsafe fn onDrawPath(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:32-32
    unsafe fn onDrawRegion(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        region: &crate::src::core::SkRegion::SkRegion,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:34-35
    unsafe fn onDrawTextBlob(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        blob: &crate::src::core::SkTextBlob::SkTextBlob,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:37-39
    unsafe fn onDrawPatch(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        cubics: &[crate::src::core::SkPoint::SkPoint; 12],
        colors: Option<&[crate::src::core::SkColor::SkColor; 4]>,
        tex: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:40-41
    unsafe fn onDrawPoints(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        mode: PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:46-47
    #[cfg(sk_build_for_android_framework)]
    unsafe fn onDrawEdgeAAQuad(
        &self,
        _canvas: core::ptr::NonNull<SkCanvas<'device>>,
        _rect: &crate::src::core::SkRect::SkRect,
        _clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        _aa: QuadAAFlags,
        _color: &crate::src::core::SkColor::SkColor4f,
        _mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/core/SkCanvasVirtualEnforcer.h:49-50
    #[cfg(not(sk_build_for_android_framework))]
    unsafe fn onDrawEdgeAAQuad(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        r: &crate::src::core::SkRect::SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        aa: QuadAAFlags,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:53-53
    unsafe fn onDrawAnnotation(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        key: &std::ffi::CStr,
        value: Option<&std::sync::Arc<crate::src::core::SkData::SkData>>,
    );
    // Source: include/core/SkCanvasVirtualEnforcer.h:54-54
    unsafe fn onDrawShadowRec(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        path: &crate::src::core::SkPath::SkPath,
        rec: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:56-56
    unsafe fn onDrawDrawable(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        drawable: &dyn crate::src::core::SkDrawable::SkDrawable,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Result<(), crate::RasterError>;
    // Source: include/core/SkCanvasVirtualEnforcer.h:57-58
    unsafe fn onDrawPicture(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        picture: &dyn crate::src::core::SkPicture::SkPicture,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError>;
}
