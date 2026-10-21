use crate::include::core::SkSamplingOptions::{SkFilterMode, SkSamplingOptions};
use crate::src::core::SkBlendMode::SkBlendMode;
use crate::src::core::SkCanvas::{
    ImageSetEntry, Lattice, PointMode, QuadAAFlags, SkCanvas, SrcRectConstraint,
};
use crate::src::core::SkColor::{SkColor, SkColor4f};
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPaint::SkPaint;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRRect::SkRRect;
use crate::src::core::SkRect::{SkIRect, SkRect};
use crate::src::image::SkImage::SkImage;
use core::pin::Pin;

use crate::include::core::SkCanvasVirtualEnforcer::{SkCanvasVirtual, SkCanvasVirtualEnforcer};

// Source: include/utils/SkNoDrawCanvas.h:24-27
// Rust inheritance uses a pinned owning base allocation and a non-owning
// virtual identity pointing back to this actual pinned derived owner.
pub struct SkNoDrawCanvas<'device> {
    inherited: Pin<Box<SkCanvas<'device>>>,
    pinned: core::marker::PhantomPinned,
}
impl<'device> SkNoDrawCanvas<'device> {
    pub(crate) fn inherited(&self) -> &SkCanvas<'device> {
        self.inherited.as_ref().get_ref()
    }
    /// Public base-class view. C++ callers obtain these const SkCanvas queries
    /// through public inheritance; the Rust owner keeps the derived allocation
    /// pinned while exposing the same read-only base subobject.
    pub fn canvas_ref(self: Pin<&Self>) -> &SkCanvas<'device> {
        self.get_ref().inherited.as_ref().get_ref()
    }
    /// Borrow the actual pinned base; the derived owner must remain pinned/live.
    pub fn canvas(self: Pin<&mut Self>) -> Pin<&mut SkCanvas<'device>> {
        unsafe { self.get_unchecked_mut() }.inherited.as_mut()
    }
    pub(crate) fn from_inherited(inherited: Pin<Box<SkCanvas<'device>>>) -> Pin<Box<Self>> {
        Self::bind(Box::pin(Self {
            inherited,
            pinned: core::marker::PhantomPinned,
        }))
    }
    pub(crate) fn bind(mut owner: Pin<Box<Self>>) -> Pin<Box<Self>> {
        let object = unsafe { owner.as_mut().get_unchecked_mut() };
        let concrete = core::ptr::NonNull::from(&mut *object);
        let hooks: *mut (dyn SkCanvasVirtualEnforcer<'device> + 'device) = concrete.as_ptr();
        unsafe {
            object
                .inherited
                .as_mut()
                .get_unchecked_mut()
                .install_derived_hooks(core::ptr::NonNull::new_unchecked(hooks));
        }
        owner
    }
    // Source: include/utils/SkNoDrawCanvas.h:30-30
    pub unsafe fn resetCanvas(
        self: Pin<&mut Self>,
        w: i32,
        h: i32,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.resetCanvas_rect(&SkIRect::make_wh(w, h)) }
    }
    // Source: include/utils/SkNoDrawCanvas.h:31-31
    pub unsafe fn resetCanvas_rect(
        self: Pin<&mut Self>,
        rect: &SkIRect,
    ) -> Result<(), crate::RasterError> {
        let base = unsafe {
            self.get_unchecked_mut()
                .inherited
                .as_mut()
                .get_unchecked_mut()
        };
        unsafe { SkCanvas::resetForNextPicture_raw(core::ptr::NonNull::from(base), rect) }
    }
}

impl<'device> SkCanvasVirtual<'device> for SkNoDrawCanvas<'device> {
    // Source: include/utils/SkNoDrawCanvas.h:61-62
    unsafe fn onDrawAtlas2(
        &self,
        _canvas: core::ptr::NonNull<SkCanvas<'device>>,
        atlas: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[crate::src::core::SkRect::SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        cull: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        self.onDrawAtlas2_nodraw(
            Some(atlas.as_ref()),
            xform,
            tex,
            if colors.is_empty() {
                None
            } else {
                Some(colors)
            },
            mode,
            sampling,
            cull,
            paint,
        );
        Ok(())
    }

    // Rust virtual bridge to the existing native empty override (header line64).
    unsafe fn onDrawVerticesObject(
        &self,
        _canvas: core::ptr::NonNull<SkCanvas<'device>>,
        vertices: &crate::src::core::SkVertices::SkVertices,
        mode: SkBlendMode,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        self.onDrawVerticesObject_nodraw(Some(vertices), mode, paint);
        Ok(())
    }

    // Source: include/utils/SkNoDrawCanvas.h:34-34
    fn getSaveLayerStrategy(
        &self,
        rec: &crate::src::core::SkCanvas::SaveLayerRec<'_>,
    ) -> crate::src::core::SkCanvas::SaveLayerStrategy {
        self.getSaveLayerStrategy_impl(rec)
    }
    // Source: include/utils/SkNoDrawCanvas.h:35-35
    fn onDoSaveBehind(&self, bounds: Option<&SkRect>) -> bool {
        self.onDoSaveBehind_impl(bounds)
    }
    unsafe fn onDrawImage2(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        _image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        _x: f32,
        _y: f32,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) {
        unsafe { self.onDrawImage2_nodraw(canvas, _image, _x, _y, _sampling, _paint) }
    }
    unsafe fn onDrawImageRect2(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        src: &crate::src::core::SkRect::SkRect,
        dst: &crate::src::core::SkRect::SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.onDrawImageRect2_nodraw(canvas, image, src, dst, sampling, paint, constraint)
        }
    }
    unsafe fn onDrawImageLattice2(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        lattice: &Lattice,
        dst: &crate::src::core::SkRect::SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawImageLattice2_nodraw(canvas, image, lattice, dst, filter, paint) }
    }
    unsafe fn onDrawEdgeAAImageSet2(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        images: &[ImageSetEntry],
        clips: &[crate::src::core::SkPoint::SkPoint],
        matrices: &[crate::src::core::SkMatrix::SkMatrix],
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.onDrawEdgeAAImageSet2_nodraw(
                canvas, images, clips, matrices, sampling, paint, constraint,
            )
        }
    }
}

impl<'device> SkNoDrawCanvas<'device> {
    // Source: include/utils/SkNoDrawCanvas.h:38-38
    pub(crate) unsafe fn onDrawAnnotation_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        key: &std::ffi::CStr,
        value: Option<&std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) {
    }
    // Source: include/utils/SkNoDrawCanvas.h:39-39
    pub(crate) unsafe fn onDrawDRRect_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        outer: &crate::src::core::SkRRect::SkRRect,
        inner: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:40-40
    pub(crate) unsafe fn onDrawDrawable_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        drawable: &dyn crate::src::core::SkDrawable::SkDrawable,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:41-41
    pub(crate) unsafe fn onDrawTextBlob_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        blob: &crate::src::core::SkTextBlob::SkTextBlob,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:42-43
    pub(crate) fn onDrawPatch_nodraw(
        &self,
        cubics: &[SkPoint; 12],
        colors: Option<&[SkColor; 4]>,
        tex: Option<&[SkPoint; 4]>,
        mode: SkBlendMode,
        paint: &SkPaint,
    ) {
    }
    // Source: include/utils/SkNoDrawCanvas.h:45-45
    pub(crate) unsafe fn onDrawPaint_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:46-46
    pub(crate) unsafe fn onDrawBehind_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:47-47
    pub(crate) unsafe fn onDrawPoints_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        mode: PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:48-48
    pub(crate) unsafe fn onDrawRect_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:49-49
    pub(crate) unsafe fn onDrawRegion_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        region: &crate::src::core::SkRegion::SkRegion,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:50-50
    pub(crate) unsafe fn onDrawOval_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        oval: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:51-51
    pub(crate) unsafe fn onDrawArc_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        oval: &crate::src::core::SkRect::SkRect,
        start: f32,
        sweep: f32,
        use_center: bool,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:52-52
    pub(crate) unsafe fn onDrawRRect_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:53-53
    pub(crate) unsafe fn onDrawPath_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:55-56
    pub(crate) unsafe fn onDrawImage2_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        _image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        _x: f32,
        _y: f32,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) {
    }
    // Source: include/utils/SkNoDrawCanvas.h:57-58
    pub(crate) unsafe fn onDrawImageRect2_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        src: &crate::src::core::SkRect::SkRect,
        dst: &crate::src::core::SkRect::SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:59-60
    pub(crate) unsafe fn onDrawImageLattice2_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        lattice: &Lattice,
        dst: &crate::src::core::SkRect::SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:61-62
    pub(crate) fn onDrawAtlas2_nodraw(
        &self,
        _: Option<&dyn SkImage>,
        _: &[crate::src::core::SkRSXform::SkRSXform],
        _: &[SkRect],
        _: Option<&[SkColor]>,
        _: SkBlendMode,
        _: &SkSamplingOptions,
        _: Option<&SkRect>,
        _: Option<&SkPaint>,
    ) {
    }
    // Source: include/utils/SkNoDrawCanvas.h:64-64
    pub(crate) fn onDrawVerticesObject_nodraw(
        &self,
        _: Option<&crate::src::core::SkVertices::SkVertices>,
        _: SkBlendMode,
        _: &SkPaint,
    ) {
    }
    // Source: include/utils/SkNoDrawCanvas.h:66-66
    pub(crate) fn onDrawPicture_nodraw(
        &self,
        _picture: &dyn crate::src::core::SkPicture::SkPicture,
        _matrix: Option<&SkMatrix>,
        _paint: Option<&SkPaint>,
    ) {
    }
    // Source: include/utils/SkNoDrawCanvas.h:68-69
    pub(crate) unsafe fn onDrawEdgeAAQuad_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        r: &crate::src::core::SkRect::SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        aa: QuadAAFlags,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: include/utils/SkNoDrawCanvas.h:70-72
    pub(crate) unsafe fn onDrawEdgeAAImageSet2_nodraw(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        images: &[ImageSetEntry],
        clips: &[crate::src::core::SkPoint::SkPoint],
        matrices: &[crate::src::core::SkMatrix::SkMatrix],
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
}

// Rust virtual bridges forward to the native empty bodies above.
impl<'device> SkCanvasVirtualEnforcer<'device> for SkNoDrawCanvas<'device> {
    // Source: include/utils/SkNoDrawCanvas.h:65-65
    unsafe fn onDrawShadowRec(
        &self,
        _canvas: core::ptr::NonNull<SkCanvas<'device>>,
        _path: &crate::src::core::SkPath::SkPath,
        _rec: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }

    // Rust dispatch bridge to the native empty Picture override above.
    unsafe fn onDrawPicture(
        &self,
        _canvas: core::ptr::NonNull<SkCanvas<'device>>,
        picture: &dyn crate::src::core::SkPicture::SkPicture,
        matrix: Option<&SkMatrix>,
        paint: Option<&SkPaint>,
    ) -> Result<(), crate::RasterError> {
        self.onDrawPicture_nodraw(picture, matrix, paint);
        Ok(())
    }

    unsafe fn onDrawAnnotation(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        key: &std::ffi::CStr,
        value: Option<&std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) {
        unsafe { self.onDrawAnnotation_nodraw(canvas, rect, key, value) }
    }
    unsafe fn onDrawDRRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        outer: &crate::src::core::SkRRect::SkRRect,
        inner: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawDRRect_nodraw(canvas, outer, inner, paint) }
    }
    unsafe fn onDrawDrawable(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        drawable: &dyn crate::src::core::SkDrawable::SkDrawable,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawDrawable_nodraw(canvas, drawable, matrix) }
    }
    unsafe fn onDrawTextBlob(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        blob: &crate::src::core::SkTextBlob::SkTextBlob,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawTextBlob_nodraw(canvas, blob, x, y, paint) }
    }
    unsafe fn onDrawPatch(
        &self,
        _canvas: core::ptr::NonNull<SkCanvas<'device>>,
        cubics: &[crate::src::core::SkPoint::SkPoint; 12],
        colors: Option<&[crate::src::core::SkColor::SkColor; 4]>,
        tex: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        self.onDrawPatch_nodraw(cubics, colors, tex, mode, paint);
        Ok(())
    }
    unsafe fn onDrawPaint(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawPaint_nodraw(canvas, paint) }
    }
    unsafe fn onDrawBehind(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawBehind_nodraw(canvas, paint) }
    }
    unsafe fn onDrawPoints(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        mode: PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawPoints_nodraw(canvas, mode, points, paint) }
    }
    unsafe fn onDrawRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawRect_nodraw(canvas, rect, paint) }
    }
    unsafe fn onDrawRegion(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        region: &crate::src::core::SkRegion::SkRegion,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawRegion_nodraw(canvas, region, paint) }
    }
    unsafe fn onDrawOval(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        oval: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawOval_nodraw(canvas, oval, paint) }
    }
    unsafe fn onDrawArc(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        oval: &crate::src::core::SkRect::SkRect,
        start: f32,
        sweep: f32,
        use_center: bool,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawArc_nodraw(canvas, oval, start, sweep, use_center, paint) }
    }
    unsafe fn onDrawRRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawRRect_nodraw(canvas, rrect, paint) }
    }
    unsafe fn onDrawPath(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawPath_nodraw(canvas, path, paint) }
    }
    unsafe fn onDrawEdgeAAQuad(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        r: &crate::src::core::SkRect::SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        aa: QuadAAFlags,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        unsafe { self.onDrawEdgeAAQuad_nodraw(canvas, r, clip, aa, color, mode) }
    }
}

// Rust owning projection preserves the concrete derived allocation while
// returning access to its original base, matching a native unique_ptr<SkCanvas>.
impl<'device> crate::include::core::SkCanvasVirtualEnforcer::SkCanvasOwner<'device>
    for SkNoDrawCanvas<'device>
{
    fn canvas(self: Pin<&mut Self>) -> Pin<&mut SkCanvas<'device>> {
        SkNoDrawCanvas::canvas(self)
    }
    fn canvas_ref(&self) -> &SkCanvas<'device> {
        self.inherited()
    }
}

// Rust drops the owned base after this body, matching the native derived-to-base
// vtable transition. No callback may reference the derived object during base Drop.
impl Drop for SkNoDrawCanvas<'_> {
    fn drop(&mut self) {
        unsafe { self.inherited.as_mut().get_unchecked_mut() }.clear_derived_hooks();
    }
}
