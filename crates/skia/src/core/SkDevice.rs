use crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator;
use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::include::core::SkScalar::{SkScalarFloorToInt, SkScalarFloorToScalar};
use crate::include::core::SkSize::SkISize;
use crate::include::core::SkSurfaceProps::{SkPixelGeometry, SkSurfaceProps};
use crate::src::core::SkImageInfo::SkImageInfo;
use crate::src::core::SkM44::SkM44;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPoint::SkIPoint;
use crate::src::core::SkRect::SkIRect;

// Native metadata/coordinate state is composed into concrete device owners.
// Drawing virtual hooks use the canonical retained SkPaint type.

// Source: src/core/SkDevice.h:542-560
pub struct SkDevice {
    // C++ destroys members in reverse declaration order, then its parent.
    // Rust destroys fields in declaration order: retain the same allocation
    // and fields, declaring only the destruction order in reverse here.
    local_to_device_dirty: bool,
    local_to_device33: SkMatrix,
    global_to_device: SkM44,
    device_to_global: SkM44,
    local_to_device: SkM44,
    surface_props: SkSurfaceProps,
    info: SkImageInfo,
    // Source: src/core/SkDevice.h:107-107
    // The inherited counter dies after all derived/base retained members.
    ref_cnt: SkRefCnt,
}
// Source: src/core/SkDevice.h:117-133
impl SkDevice {
    pub fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.ref_cnt.ref_cnt_base()
    }
    pub fn ref_(&self) {
        self.ref_cnt.ref_();
    }
    pub fn image_info(&self) -> &SkImageInfo {
        &self.info
    }
    pub fn width(&self) -> i32 {
        self.image_info().width()
    }
    pub fn height(&self) -> i32 {
        self.image_info().height()
    }
    pub fn is_opaque(&self) -> bool {
        self.image_info().isOpaque()
    }
    pub fn bounds(&self) -> SkIRect {
        SkIRect::make_wh(self.width(), self.height())
    }
    pub fn size(&self) -> SkISize {
        self.image_info().dimensions()
    }
    pub fn surface_props(&self) -> &SkSurfaceProps {
        &self.surface_props
    }
}
// Source: src/core/SkDevice.h:175-191
impl SkDevice {
    pub fn local_to_device44(&self) -> &SkM44 {
        &self.local_to_device
    }
    pub fn local_to_device(&self) -> &SkMatrix {
        &self.local_to_device33
    }
    pub fn device_to_global(&self) -> &SkM44 {
        &self.device_to_global
    }
    pub fn global_to_device(&self) -> &SkM44 {
        &self.global_to_device
    }
}
// Source: src/core/SkDevice.h:211-215
impl SkDevice {
    pub fn set_local_to_device(&mut self, m: &SkM44) {
        self.local_to_device = m.clone();
        self.local_to_device33 = self.local_to_device.asM33();
        self.local_to_device_dirty = true;
    }
}
// Source: src/core/SkDevice.h:225-228
impl SkDevice {
    pub fn get_global_bounds_into(&self, bounds: &mut SkIRect) {
        // The exclusive output reference supplies the source's nonnull assertion.
        *bounds = crate::src::core::SkMatrixPriv::SkMatrixPriv::MapRect(
            &self.device_to_global,
            &crate::src::core::SkRect::SkRect::Make(&self.bounds()),
        )
        .roundOut();
    }
}
// Source: src/core/SkDevice.h:230-234
impl SkDevice {
    pub fn get_global_bounds(&self) -> SkIRect {
        let mut bounds = SkIRect::make_empty();
        self.get_global_bounds_into(&mut bounds);
        bounds
    }
}
// Source: src/core/SkDevice.h:500-511
impl SkDevice {
    pub fn set_origin(&mut self, global_ctm: &SkM44, x: i32, y: i32) {
        self.set_device_coordinate_system(&SkM44::default(), &SkM44::default(), global_ctm, x, y);
    }
    pub fn check_local_to_device_dirty(&mut self) -> bool {
        let was_dirty = self.local_to_device_dirty;
        self.local_to_device_dirty = false;
        was_dirty
    }
}
// Source: src/core/SkDevice.cpp:48-53
impl SkDevice {
    pub fn new(info: &SkImageInfo, props: &SkSurfaceProps) -> Self {
        let (mut device_to_global, mut global_to_device) = (SkM44::default(), SkM44::default());
        device_to_global.setIdentity();
        global_to_device.setIdentity();
        Self {
            ref_cnt: SkRefCnt::new(),
            info: info.clone(),
            surface_props: *props,
            device_to_global,
            global_to_device,
            local_to_device: SkM44::default(),
            local_to_device33: SkMatrix::default(),
            local_to_device_dirty: true,
        }
    }
}
// Source: src/core/SkDevice.cpp:55-74
impl SkDevice {
    pub fn set_device_coordinate_system(
        &mut self,
        device_to_global: &SkM44,
        global_to_device: &SkM44,
        local_to_device: &SkM44,
        buffer_x: i32,
        buffer_y: i32,
    ) {
        self.device_to_global = device_to_global.clone();
        self.device_to_global.normalizePerspective();
        self.global_to_device = global_to_device.clone();
        self.global_to_device.normalizePerspective();
        self.local_to_device = local_to_device.clone();
        self.local_to_device.normalizePerspective();
        if buffer_x | buffer_y != 0 {
            self.device_to_global
                .preTranslate(buffer_x as f32, buffer_y as f32, 0.0);
            self.global_to_device
                .postTranslate(-(buffer_x as f32), -(buffer_y as f32), 0.0);
            self.local_to_device
                .postTranslate(-(buffer_x as f32), -(buffer_y as f32), 0.0);
        }
        self.local_to_device33 = self.local_to_device.asM33();
        self.local_to_device_dirty = true;
    }
}
// Source: src/core/SkDevice.cpp:76-83
impl SkDevice {
    pub fn set_global_ctm(&mut self, ctm: &SkM44) {
        self.local_to_device = ctm.clone();
        self.local_to_device.normalizePerspective();
        self.local_to_device.postConcat(&self.global_to_device);
        self.local_to_device33 = self.local_to_device.asM33();
        self.local_to_device_dirty = true;
    }
}
// Source: src/core/SkDevice.cpp:85-92
impl SkDevice {
    pub fn is_pixel_aligned_to_global(&self) -> bool {
        let mut aligned = SkM44::default();
        aligned.setRC(0, 3, SkScalarFloorToScalar(self.device_to_global.rc(0, 3)));
        aligned.setRC(1, 3, SkScalarFloorToScalar(self.device_to_global.rc(1, 3)));
        aligned == self.device_to_global
    }
}
// Source: src/core/SkDevice.cpp:94-104
impl SkDevice {
    pub fn get_origin(&self) -> SkIPoint {
        crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 101; @condition b"this->isPixelAlignedToGlobal()\0".as_ptr(); self.is_pixel_aligned_to_global());
        SkIPoint::make(
            SkScalarFloorToInt(self.device_to_global.rc(0, 3)),
            SkScalarFloorToInt(self.device_to_global.rc(1, 3)),
        )
    }
}
// Source: src/core/SkDevice.cpp:106-110
impl SkDevice {
    pub fn get_relative_transform(&self, destination: &Self) -> SkM44 {
        SkM44::concat(&destination.global_to_device, &self.device_to_global)
    }
}
// Source: src/core/SkDevice.cpp:112-114
fn is_int(x: f32) -> bool {
    x == crate::src::core::SkFloatingPoint::sk_float_round2int(x) as f32
}
// Source: src/core/SkDevice.h:298-311
// The native ImageInfo/geometry values are owned and read-only through this API.
// A mutable allocator borrow replaces the raw non-owning pointer; its lifetime
// constrains this temporary configuration only, never the returned child device.
pub struct CreateInfo<'allocator> {
    fInfo: SkImageInfo,
    fPixelGeometry: SkPixelGeometry,
    fAllocator: Option<&'allocator mut (dyn SkRasterHandleAllocator + 'allocator)>,
}
impl<'allocator> CreateInfo<'allocator> {
    pub fn new(
        info: &SkImageInfo,
        pixel_geometry: SkPixelGeometry,
        allocator: Option<&'allocator mut (dyn SkRasterHandleAllocator + 'allocator)>,
    ) -> Self {
        Self {
            fInfo: info.clone(),
            fPixelGeometry: pixel_geometry,
            fAllocator: allocator,
        }
    }
    pub fn info(&self) -> &SkImageInfo {
        &self.fInfo
    }
    pub fn pixel_geometry(&self) -> SkPixelGeometry {
        self.fPixelGeometry
    }
    // The output reference is reborrowed for this call, not for 'allocator.
    pub fn allocator(&mut self) -> Option<&mut (dyn SkRasterHandleAllocator + 'allocator)> {
        self.fAllocator.as_deref_mut()
    }
}

// Source: src/core/SkDevice.h:75-75
// Actual forward-declared GPU class, pointer identity only. No GPU object,
// behavior, allocation or counter is constructed by the CPU default below.
pub enum SubRunControl {}
// Source: src/core/SkDevice.h:79-84
pub struct SkStrikeDeviceInfo {
    pub fSurfaceProps: SkSurfaceProps,
    pub fScalerContextFlags: crate::src::core::SkScalerContext::SkScalerContextFlags,
    pub fSubRunControl: *const SubRunControl,
}

// Rust composition exposes the actual SkDevice base state in place of C++
// inherited data. Concrete devices must supply every translated virtual hook.
// Required drawing hooks use the canonical retained SkPaint type.
/// # Safety
/// Implementors use their actual embedded SkDevice as the sole ref-count base.
/// The complete Box allocation and dynamic metadata must survive until final
/// release; pinning must preserve any inline stack/recorder addresses. No proxy
/// Box, independent counter, or separately owned base may represent this object.
pub unsafe trait SkDeviceVirtual: SkRefCntObject {
    // Source: src/core/SkDevice.h:137-139
    fn strikeDeviceInfo(&self) -> SkStrikeDeviceInfo {
        let base = self.base();
        SkStrikeDeviceInfo {
            fSurfaceProps: *base.surface_props(),
            fScalerContextFlags: base.scaler_context_flags(),
            fSubRunControl: core::ptr::null(),
        }
    }

    // Source: src/core/SkDevice.h:534-536
    // Adapted virtual dispatch: project a function pointer, then release the
    // receiver projection before callbacks reenter the original complete owner.
    fn drawGlyphRunListProc(&self) -> DrawGlyphRunListProc;
    // Source: src/core/SkDevice.h:332-333
    // Source: src/core/SkDevice.cpp:481-484
    fn convertGlyphRunListToSlug(
        &mut self,
        _list: &crate::src::text::GlyphRun::GlyphRunList,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> sk_sp<dyn crate::include::private::chromium::Slug::Slug> {
        sk_sp::default()
    }
    // Source: src/core/SkDevice.h:334-334
    // Source: src/core/SkDevice.cpp:486-488
    unsafe fn drawSlug(
        &mut self,
        _canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
        _slug: &dyn crate::include::private::chromium::Slug::Slug,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        // Native unsupported base explicitly aborts; this is not a no-op producer.
        crate::SK_ABORT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 487; "Slug drawing not supported.")
    }
    // Source: src/core/SkDevice.h:323-323
    /// Native default cannot create a compatible child and returns nullptr.
    ///
    /// The configuration borrow permits mutating its allocator capability only;
    /// it does not lend state to the returned owner. The reviewed CPU BitmapDevice
    /// override calls the static Create path with recorder=None and owns its info,
    /// props and bitmap. It must not borrow the parent recorder or extend any
    /// allocator/context reference to static. Allocator Rec backing/release-context
    /// obligations must survive independently until the created bitmap releases them.
    fn createDevice(
        &mut self,
        _info: &mut CreateInfo<'_>,
        _layer_paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> sk_sp<dyn SkDeviceVirtual + 'static> {
        sk_sp::default()
    }
    // Source: src/core/SkDevice.h:282-282
    fn getRasterHandle(&self) -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
    // Source: src/core/SkDevice.h:284-285
    fn recordingContext(&self) -> *mut GrRecordingContext {
        core::ptr::null_mut()
    }
    fn recorder(&self) -> *mut skgpu::graphite::Recorder {
        core::ptr::null_mut()
    }
    // Source: src/core/SkDevice.h:288-289
    fn asGaneshDevice(&mut self) -> *mut skgpu::ganesh::Device {
        core::ptr::null_mut()
    }
    fn asGraphiteDevice(&mut self) -> *mut skgpu::graphite::Device {
        core::ptr::null_mut()
    }
    // Source: src/core/SkDevice.h:286-286
    fn baseRecorder(&self) -> Option<&dyn crate::include::core::SkRecorder::SkRecorder> {
        None
    }
    // Source: src/core/SkDevice.h:294-294
    fn setImmutable(&mut self) {}
    // Source: src/core/SkDevice.h:362-364
    unsafe fn drawImageRect(
        &mut self,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        src: Option<&crate::src::core::SkRect::SkRect>,
        dst: &crate::src::core::SkRect::SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkDevice.h:383-386
    unsafe fn drawVertices(
        &mut self,
        vertices: &crate::src::core::SkVertices::SkVertices,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
        skip_color_xform: bool,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkDevice.h:387-387
    unsafe fn drawMesh(
        &mut self,
        mesh: &crate::src::core::SkMesh::SkMesh,
        blender: sk_sp<dyn crate::include::core::SkBlender::SkBlender>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkDevice.h:388-388
    // Source: src/utils/SkShadowUtils.cpp:602-861
    /// Safety: Canvas and this actual complete Device are under one exclusive
    /// native draw lease; no independently borrowed Device view may coexist.
    unsafe fn drawShadow(
        &mut self,
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
        path: &crate::src::core::SkPath::SkPath,
        rec: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError> {
        unsafe { crate::src::utils::SkShadowUtils::draw_device_shadow(self, canvas, path, rec) }
    }
    // Source: src/core/SkDevice.h:391-393
    // Source: src/core/SkDevice.cpp:154-163
    unsafe fn drawPatch(
        &mut self,
        cubics: &[crate::src::core::SkPoint::SkPoint; 12],
        colors: Option<&[crate::src::core::SkColor::SkColor; 4]>,
        tex: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        blender: sk_sp<dyn crate::include::core::SkBlender::SkBlender>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let lod = crate::src::utils::SkPatchUtils::SkPatchUtils::GetLevelOfDetail(
            cubics,
            Some(self.base().local_to_device()),
        );
        let vertices = crate::src::utils::SkPatchUtils::SkPatchUtils::MakeVertices(
            Some(cubics),
            colors,
            tex,
            lod.width(),
            lod.height(),
            self.base().image_info().refColorSpace(),
        )?;
        if let Some(vertices) = vertices.get() {
            unsafe { self.drawVertices(vertices, blender, paint, false) }?;
        }
        Ok(())
    }
    // Source: src/core/SkDevice.h:395-397
    // Source: src/core/SkDevice.cpp:207-237
    unsafe fn drawAtlas(
        &mut self,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[crate::src::core::SkRect::SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        blender: sk_sp<dyn crate::include::core::SkBlender::SkBlender>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkVertices::{
            kHasColors_BuilderFlag, kHasTexCoords_BuilderFlag, Builder, VertexMode,
        };
        let quad_count = xform.len();
        let tri_count = quad_count << 1;
        let vertex_count = tri_count.wrapping_mul(3);
        let mut flags = kHasTexCoords_BuilderFlag;
        if !colors.is_empty() {
            flags |= kHasColors_BuilderFlag;
        }
        let mut builder = Builder::new(
            VertexMode::kTriangles_VertexMode,
            vertex_count as i32,
            0,
            flags,
        );
        let mut vpos = builder.positions();
        let mut vtex = builder.texCoords();
        let mut vcol = builder.colors();
        for i in 0..quad_count {
            let mut tmp = [crate::src::core::SkPoint::SkPoint::default(); 4];
            xform[i].toQuad(tex[i].width(), tex[i].height(), &mut tmp);
            vpos = unsafe { quad_to_tris(vpos, &tmp) };
            vtex = unsafe {
                quad_to_tris(
                    vtex,
                    &tex[i].toQuad(crate::include::core::SkPathTypes::SkPathDirection::kCW),
                )
            };
            if !colors.is_empty() {
                for j in 0..6 {
                    unsafe {
                        vcol.add(j).write(colors[i]);
                    }
                }
                vcol = unsafe { vcol.add(6) };
            }
        }
        let vertices = unsafe { builder.detach() };
        unsafe {
            self.drawVertices(
                vertices.get().expect("native atlas builder vertex owner"),
                blender,
                paint,
                false,
            )
        }
    }
    // Source: src/core/SkDevice.h:367-367
    fn shouldDrawAsTiledImageRect(&self) -> bool {
        false
    }
    // Source: src/core/SkDevice.h:368-374
    // Exact native base virtual returns false: CPU BitmapDevice does not select
    // the tiled-image backend path. This is not a replacement drawing fallback.
    unsafe fn drawAsTiledImageRect(
        &mut self,
        _canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
        _image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        _src: Option<&crate::src::core::SkRect::SkRect>,
        _dst: &crate::src::core::SkRect::SkRect,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: &crate::src::core::SkPaint::SkPaint,
        _constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<bool, crate::RasterError> {
        Ok(false)
    }
    // Source: src/core/SkDevice.h:468-470
    fn drawBlurredRRect(
        &mut self,
        _rrect: &SkRRect,
        _paint: &crate::src::core::SkPaint::SkPaint,
        _device_sigma: f32,
    ) -> Result<bool, crate::RasterError> {
        Ok(false)
    }
    // Source: src/core/SkDevice.cpp:304-305
    /// Native base implementation is genuinely empty. CPU BitmapDevice must
    /// override this slot with its actual raster drawBitmap path before use.
    /// Safety: image pixels and destination obey their live exclusive draw leases.
    unsafe fn drawSpecialWithConstraint(
        &mut self,
        _image: &dyn crate::src::core::SkSpecialImage::SkSpecialImage,
        _matrix: &SkMatrix,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: &crate::src::core::SkPaint::SkPaint,
        _constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:446-450
    /// Rust spelling of the actual native default constraint argument.
    unsafe fn drawSpecial(
        &mut self,
        image: &dyn crate::src::core::SkSpecialImage::SkSpecialImage,
        matrix: &SkMatrix,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.drawSpecialWithConstraint(
                image,
                matrix,
                sampling,
                paint,
                crate::src::core::SkCanvas::SrcRectConstraint::kStrict_SrcRectConstraint,
            )
        }
    }
    // Source: src/core/SkDevice.cpp:306-311
    /// Safety: native destination/mask backing leases hold. Only a device that
    /// overrides useDrawCoverageMaskForMaskFilters may receive this virtual call.
    unsafe fn drawCoverageMask(
        &mut self,
        _mask: &dyn crate::src::core::SkSpecialImage::SkSpecialImage,
        _matrix: &SkMatrix,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        // Exact original invariant abort, never a MissingStage fallback.
        crate::SK_ABORT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 310; "Must override if useDrawCoverageMaskForMaskFilters() is true")
    }
    // Source: src/core/SkDevice.cpp:313-313
    // Current canonical SkSpecialImage owners are Arc; native owning migration pending.
    fn snapSpecial(
        &mut self,
        _subset: &SkIRect,
        _force_copy: bool,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        Ok(None)
    }
    // Source: src/core/SkDevice.cpp:314-317
    // This is the original base virtual's null result, not a replacement raster backing.
    fn snapSpecialScaled(
        &mut self,
        _subset: &SkIRect,
        _dst_dims: &SkISize,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        Ok(None)
    }
    // Source: src/core/SkDevice.cpp:318-320
    // Rust spelling of the native zero-argument snapSpecial overload.
    fn snapSpecialAll(
        &mut self,
    ) -> Result<
        Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        crate::RasterError,
    > {
        let subset = SkIRect::make_wh(self.base().width(), self.base().height());
        self.snapSpecial(&subset, false)
    }
    // Source: src/core/SkDevice.h:518-519
    // Source: src/core/SkDevice.cpp:322-324
    // Native const virtual dispatch delegates to the actual raster backend.
    // Arc ownership and the fallible execution boundary are Rust adaptations;
    // there is no nullable substitute or diagnostic-to-success conversion.
    fn createImageFilteringBackend(
        &self,
        surface_props: &SkSurfaceProps,
        color_type: crate::include::core::SkColorType::SkColorType,
    ) -> Result<std::sync::Arc<dyn crate::src::core::SkImageFilterTypes::Backend>, crate::RasterError>
    {
        crate::src::core::SkImageFilterTypes::MakeRasterBackend(surface_props, color_type)
    }
    // Source: src/core/SkDevice.cpp:327-342
    /// Safety: source and destination are live whole devices; their pixel leases
    /// exclude conflicting access for snapshotting and the complete raster draw.
    unsafe fn drawDevice(
        &mut self,
        mut device: core::ptr::NonNull<dyn SkDeviceVirtual + '_>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let image = unsafe { device.as_mut() }.snapSpecialAll()?;
        if let Some(image) = image {
            let matrix = unsafe { device.as_ref() }
                .base()
                .get_relative_transform(self.base())
                .asM33();
            let strict = *sampling
                != crate::include::core::SkSamplingOptions::SkSamplingOptions::new()
                || !matrix.isTranslate()
                || !is_int(matrix.getTranslateX())
                || !is_int(matrix.getTranslateY());
            unsafe {
                self.drawSpecialWithConstraint(
                    image.as_ref(),
                    &matrix,
                    sampling,
                    paint,
                    if strict {
                        crate::src::core::SkCanvas::SrcRectConstraint::kStrict_SrcRectConstraint
                    } else {
                        crate::src::core::SkCanvas::SrcRectConstraint::kFast_SrcRectConstraint
                    },
                )
            }?;
        }
        Ok(())
    }
    // Source: src/core/SkDevice.h:147-147
    /// # Safety
    /// The pixmap allocation must be valid for its format, row bytes and dimensions;
    /// the caller must preclip the source so that its placement fits this device.
    unsafe fn writePixels(
        &mut self,
        src: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.onWritePixels(src, x, y) }
    }
    // Source: src/core/SkDevice.h:153-153
    /// # Safety
    /// The destination allocation must be writable for its format, row bytes and
    /// dimensions. The requested device rectangle must fit the destination.
    unsafe fn readPixels(
        &mut self,
        dst: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.onReadPixels(dst, x, y) }
    }
    // Source: src/core/SkDevice.h:522-522
    /// # Safety
    /// The destination and coordinates must meet readPixels' allocation and
    /// preclipping requirements. Overrides must not access beyond that storage.
    unsafe fn onReadPixels(
        &mut self,
        _dst: &crate::src::core::SkPixmap::SkPixmap,
        _x: i32,
        _y: i32,
    ) -> Result<bool, crate::RasterError> {
        Ok(false)
    }
    // Source: src/core/SkDevice.h:525-525
    /// # Safety
    /// The source and coordinates must meet writePixels' allocation and
    /// preclipping requirements. Overrides must not access beyond that storage.
    unsafe fn onWritePixels(
        &mut self,
        _src: &crate::src::core::SkPixmap::SkPixmap,
        _x: i32,
        _y: i32,
    ) -> Result<bool, crate::RasterError> {
        Ok(false)
    }
    // Source: src/core/SkDevice.h:276-276
    fn useDrawCoverageMaskForMaskFilters(&self) -> bool {
        false
    }
    // Source: src/core/SkDevice.h:280-280
    fn isNoPixelsDevice(&self) -> bool {
        false
    }
    // Source: src/core/SkDevice.h:527-527
    fn onAccessPixels(&mut self, _pixels: &mut crate::src::core::SkPixmap::SkPixmap) -> bool {
        false
    }
    // Source: src/core/SkDevice.h:529-529
    fn onPeekPixels(&self, _pixels: &mut crate::src::core::SkPixmap::SkPixmap) -> bool {
        false
    }
    fn accessPixels(&mut self, pixels: Option<&mut crate::src::core::SkPixmap::SkPixmap>) -> bool {
        access_pixels(self, pixels)
    }
    fn peekPixels(&self, pixels: Option<&mut crate::src::core::SkPixmap::SkPixmap>) -> bool {
        peek_pixels(self, pixels)
    }

    // Result error glue is a Rust integration adaptation; source drawing algorithms
    // retain their native control flow and propagate rejected/missing stages unchanged.
    // Source: src/core/SkDevice.cpp:345-377
    /// Safety: real source immutable pixels and exclusive destination backing
    /// remain leased through native filtering and drawing; no aliases conflict.
    unsafe fn drawFilteredImage(
        &mut self,
        mapping: &crate::src::core::SkImageFilterTypes::Mapping,
        src: Option<&std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
        mut color_type: crate::include::core::SkColorType::SkColorType,
        filter: &dyn crate::src::core::SkImageFilter::SkImageFilter,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkImageFilterTypes::{Context, DeviceSpace, FilterResult, Stats};
        crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 351; @condition b"!paint.getImageFilter() && !paint.getMaskFilter()\0".as_ptr(); paint.getImageFilter().is_none()&&paint.getMaskFilter().is_none());
        let output = mapping.deviceToLayer(&DeviceSpace::new(self.devClipBounds()));
        if color_type == crate::include::core::SkColorType::SkColorType::kUnknown_SkColorType {
            color_type = crate::include::core::SkColorType::SkColorType::kRGBA_8888_SkColorType;
        }
        let stats = core::cell::Cell::new(Stats::default());
        let backend = self.createImageFilteringBackend(
            src.map(|src| src.props())
                .unwrap_or_else(|| self.base().surface_props()),
            color_type,
        )?;
        let ctx = Context::new(
            backend,
            mapping,
            output,
            &FilterResult::from_image(src.cloned()),
            self.base().image_info().refColorSpace(),
            Some(&stats),
        );
        let (result, offset) = {
            let filtered = unsafe { filter.filterImage(&ctx) }?;
            unsafe { filtered.imageAndOffset(&ctx) }?
        };
        stats.get().reportStats();
        if let Some(result) = result {
            let mut matrix = mapping.layerToDeviceMatrix().asM33();
            matrix.preTranslate(offset.geometry().fX as f32, offset.geometry().fY as f32);
            unsafe { self.drawSpecial(result.as_ref(), &matrix, sampling, paint) }?;
        }
        Ok(())
    }

    // Source: src/core/SkDevice.cpp:116-135
    fn drawRegion(
        &mut self,
        region: &SkRegion,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let matrix = self.base().local_to_device();
        let non_translate = matrix.getType() & !SkMatrix::kTranslate_Mask != 0;
        let complex = paint.getStyle() != crate::src::core::SkPaint::Style::kFill_Style
            || paint.getMaskFilter().is_some()
            || paint.getPathEffect().is_some();
        let aa = paint.isAntiAlias()
            && (!is_int(matrix.getTranslateX()) || !is_int(matrix.getTranslateY()));
        if non_translate || complex || aa {
            let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
            region.add_boundary_path(&mut builder);
            builder.setIsVolatile(true);
            return self.drawPath(&builder.detach(None), paint);
        }
        let mut iter = crate::src::core::SkRegion::RegionIterator::new(region);
        while !iter.done() {
            self.drawRect(&SkRect::Make(iter.rect()), paint)?;
            iter.next();
        }
        Ok(())
    }
    // Source: src/core/SkDevice.cpp:137-141
    fn drawArc(
        &mut self,
        arc: &crate::include::core::SkArc::SkArc,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let fill_no_effect = paint.getStyle() == crate::src::core::SkPaint::Style::kFill_Style
            && paint.getPathEffect().is_none();
        let path = crate::src::core::SkPathPriv::SkPathPriv::CreateDrawArcPath(arc, fill_no_effect);
        self.drawPath(&path, paint)
    }
    // Source: src/core/SkDevice.cpp:143-152
    fn drawDRRect(
        &mut self,
        outer: &SkRRect,
        inner: &SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        use crate::include::core::SkPathTypes::{SkPathDirection, SkPathFillType};
        let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
        builder.addRRect(outer, SkPathDirection::kCW, 6);
        builder.addRRect(inner, SkPathDirection::kCW, 6);
        builder.setFillType(SkPathFillType::kEvenOdd);
        builder.setIsVolatile(true);
        self.drawPath(&builder.detach(None), paint)
    }

    // Source: src/core/SkDevice.h:336-336
    fn drawPaint(
        &mut self,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkDevice.h:337-337
    // Shared native Canvas enum; Result adapts the drawing error boundary.
    fn drawPoints(
        &mut self,
        mode: crate::src::core::SkCanvas::PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkDevice.h:338-339
    fn drawRect(
        &mut self,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkDevice.h:342-343
    // Result is the shared RasterError adaptation of the native void contract.
    fn drawOval(
        &mut self,
        oval: &SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkDevice.h:346-347
    fn drawRRect(
        &mut self,
        rrect: &SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;
    // Source: src/core/SkDevice.h:359-360
    fn drawPath(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError>;

    // Source: src/core/SkDevice.h:376-377
    unsafe fn drawImageLattice(
        &mut self,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        lattice: &crate::src::core::SkCanvas::Lattice,
        dst: &SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { draw_image_lattice(self, image, lattice, dst, filter, paint) }
    }
    // Source: src/core/SkDevice.h:403-405
    fn drawEdgeAAQuad(
        &mut self,
        r: &SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        aa: crate::src::core::SkCanvas::QuadAAFlags,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        draw_edge_aa_quad(self, r, clip, aa, color, mode)
    }
    // Source: src/core/SkDevice.h:408-411
    unsafe fn drawEdgeAAImageSet(
        &mut self,
        images: &[crate::src::core::SkCanvas::ImageSetEntry],
        clips: &[crate::src::core::SkPoint::SkPoint],
        matrices: &[SkMatrix],
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: &crate::src::core::SkPaint::SkPaint,
        constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            draw_edge_aa_image_set(self, images, clips, matrices, sampling, paint, constraint)
        }
    }

    fn base(&self) -> &SkDevice;
    /// # Safety
    /// The caller holds the exclusive device lease. Only mutate base state in
    /// place: never move, replace, or separately dispose the embedded base/counter.
    unsafe fn base_mut(&mut self) -> &mut SkDevice;

    // Source: src/core/SkDevice.h:251-258
    // Arc retention and Option for a nullable native sk_sp are language adaptations.
    // The CTM is captured before Difference inverts alpha; neither factory is bypassed.
    fn clipShader(
        &mut self,
        mut shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>,
        op: SkClipOp,
    ) {
        shader = shader.makeWithCTM(self.base().local_to_device());
        if op == SkClipOp::kDifference {
            let inverted = shader.makeInvertAlpha();
            // Native assignment releases the preceding CTM wrapper before dispatch.
            drop(shader);
            self.onClipShader(inverted);
        } else {
            self.onClipShader(Some(shader));
        }
    }
    // Source: src/core/SkDevice.h:531-531
    fn onClipShader(
        &mut self,
        shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    );

    // Source: src/core/SkDevice.h:241-249
    fn devClipBounds(&self) -> SkIRect;
    fn pushClipStack(&mut self);
    fn popClipStack(&mut self);
    fn clipRect(
        &mut self,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    );
    fn clipRRect(
        &mut self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    );
    fn clipPath(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    );
    fn clipRegion(
        &mut self,
        region: &crate::src::core::SkRegion::SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    );

    // Source: src/core/SkDevice.h:259-267
    fn replaceClip(&mut self, rect: &SkIRect);
    fn isClipAntiAliased(&self) -> bool;
    fn isClipEmpty(&self) -> bool;
    fn isClipRect(&self) -> bool;
    fn isClipWideOpen(&self) -> bool;
    fn android_utils_clipAsRgn(&self, region: &mut crate::src::core::SkRegion::SkRegion);
    fn android_utils_clipWithStencil(&mut self) -> bool {
        false
    }
    // Rust dispatch adaptation of native SkDevice.h413. Retrieve the actual
    // function pointer under a SHORT owner borrow, end it, then call with the
    // original raw complete Device. Drawable callbacks may reenter this Canvas;
    // no whole &mut Device may remain live during that callback.
    fn drawDrawableProc(&self) -> DrawDrawableProc {
        draw_drawable_default
    }
    // Source: src/core/SkDevice.h:398-398
    // The native base intentionally has no annotation side effect. A concrete
    // recorder may retain the original Arc data owner when overriding this slot.
    fn drawAnnotation(
        &mut self,
        _rect: &SkRect,
        _key: &std::ffi::CStr,
        _value: Option<&std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) {
    }
    // Source: src/core/SkDevice.cpp:492-494
    // Exact native base result. Concrete BitmapDevice overrides this slot with
    // its real Raster allocation; no unavailable CPU backend is substituted.
    fn makeSurface(
        &self,
        _info: &SkImageInfo,
        _props: &SkSurfaceProps,
    ) -> sk_sp<dyn crate::src::image::SkSurface::SkSurface<'static> + 'static> {
        sk_sp::default()
    }
}

use crate::include::core::SkClipOp::SkClipOp;
use crate::include::private::SkTArray::STArray;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkRRect::SkRRect;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkRegion::SkRegion;

// Source: src/core/SkDevice.h:638-653
#[must_use]
pub struct SkAutoDeviceTransformRestore<'a> {
    device: &'a mut SkDevice,
    previous: SkM44,
}
impl<'a> SkAutoDeviceTransformRestore<'a> {
    pub fn new(device: &'a mut SkDevice, local_to_device: &SkM44) -> Self {
        // Source stores the 3x3 localToDevice() view in an SkM44, rather than
        // copying localToDevice44(); preserve that conversion exactly.
        let previous = SkM44::from(device.local_to_device());
        device.set_local_to_device(local_to_device);
        Self { device, previous }
    }
}
impl Drop for SkAutoDeviceTransformRestore<'_> {
    fn drop(&mut self) {
        self.device.set_local_to_device(&self.previous);
    }
}
// Rust's exclusive guard borrow exposes the same device for operations inside
// the restore scope, without retaining a second mutable alias.
impl core::ops::Deref for SkAutoDeviceTransformRestore<'_> {
    type Target = SkDevice;
    fn deref(&self) -> &SkDevice {
        self.device
    }
}
impl core::ops::DerefMut for SkAutoDeviceTransformRestore<'_> {
    fn deref_mut(&mut self) -> &mut SkDevice {
        self.device
    }
}
// Source: src/core/SkDevice.cpp:165-191
unsafe fn draw_image_lattice<T: SkDeviceVirtual + ?Sized>(
    device: &mut T,
    image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
    lattice: &crate::src::core::SkCanvas::Lattice,
    dst: &SkRect,
    filter: crate::include::core::SkSamplingOptions::SkFilterMode,
    paint: &crate::src::core::SkPaint::SkPaint,
) -> Result<(), crate::RasterError> {
    // The original iterator owns its copied grid and fixed-color arrays.
    let mut iter = unsafe { crate::src::core::SkLatticeIter::SkLatticeIter::new(lattice, dst) };
    let mut src = SkRect::default();
    let mut target = SkRect::default();
    // Rust scratch initialization only: successful next/readPixels must write
    // their real outputs before color can be consumed by the native branches.
    let mut color = 0u32;
    let mut fixed = false;
    let info = SkImageInfo::Make(
        1,
        1,
        crate::include::core::SkColorType::SkColorType::kBGRA_8888_SkColorType,
        crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType,
    );
    while unsafe { iter.next(&mut src, &mut target, Some(&mut fixed), Some(&mut color)) } {
        let single_pixel = if fixed {
            true
        } else if src.width() <= 1.0 && src.height() <= 1.0 {
            unsafe {
                image.readPixels_info_legacy(
                    &info,
                    core::ptr::addr_of_mut!(color).cast(),
                    4,
                    src.fLeft as i32,
                    src.fTop as i32,
                    crate::src::image::SkImage::CachingHint::kAllow_CachingHint,
                )
            }?
        } else {
            false
        };
        if single_pixel {
            if color != 0 || !paint.isSrcOver() {
                let mut copy = paint.clone();
                let alpha = crate::src::core::SkColorPriv::SkAlphaMul(
                    crate::src::core::SkColor::SkColorGetA(color),
                    crate::src::core::SkColorPriv::SkAlpha255To256(paint.getAlpha() as u32),
                );
                copy.setColor(crate::src::core::SkColor::SkColorSetA(color, alpha));
                device.drawRect(&target, &copy)?;
            }
        } else {
            unsafe {
                device.drawImageRect(
                    image,
                    Some(&src),
                    &target,
                    &crate::include::core::SkSamplingOptions::SkSamplingOptions::with_filter(
                        filter,
                    ),
                    paint,
                    crate::src::core::SkCanvas::SrcRectConstraint::kStrict_SrcRectConstraint,
                )
            }?;
        }
    }
    Ok(())
}

// Source: src/core/SkDevice.cpp:193-205
/// Safety: destination addresses six writable points; quad holds four initialized points.
unsafe fn quad_to_tris(
    tris: *mut crate::src::core::SkPoint::SkPoint,
    quad: &[crate::src::core::SkPoint::SkPoint; 4],
) -> *mut crate::src::core::SkPoint::SkPoint {
    unsafe {
        tris.add(0).write(quad[0]);
        tris.add(1).write(quad[1]);
        tris.add(2).write(quad[2]);
        tris.add(3).write(quad[0]);
        tris.add(4).write(quad[2]);
        tris.add(5).write(quad[3]);
        tris.add(6)
    }
}

// Actual native virtual-call protocol, with reentrant receiver references
// replaced by the original complete-owner raw identity. No backend is allocated.
pub type DrawDrawableProc = for<'device, 'canvas> unsafe fn(
    core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
    core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
    &dyn crate::src::core::SkDrawable::SkDrawable,
    Option<&SkMatrix>,
) -> Result<(), crate::RasterError>;
/// Safety: Canvas and Device are their original live pinned allocations; the
/// caller holds the native exclusive operation/pixel lease. No whole-owner
/// references survive the callback, which may reenter Canvas/Device drawing.
pub unsafe fn drawDrawable_raw<'device, 'canvas>(
    device: core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
    drawable: &dyn crate::src::core::SkDrawable::SkDrawable,
    matrix: Option<&SkMatrix>,
) -> Result<(), crate::RasterError> {
    let proc = unsafe { device.as_ref() }.drawDrawableProc();
    unsafe { proc(device, canvas, drawable, matrix) }
}
// Source: src/core/SkDevice.cpp:239-252
fn draw_edge_aa_quad<T: SkDeviceVirtual + ?Sized>(
    device: &mut T,
    r: &SkRect,
    clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
    aa: crate::src::core::SkCanvas::QuadAAFlags,
    color: &crate::src::core::SkColor::SkColor4f,
    mode: crate::src::core::SkBlendMode::SkBlendMode,
) -> Result<(), crate::RasterError> {
    let mut paint = crate::src::core::SkPaint::SkPaint::default();
    paint.setColor4f(color, None);
    paint.setBlendMode(mode);
    paint.setAntiAlias(aa == crate::src::core::SkCanvas::QuadAAFlags::kAll_QuadAAFlags);
    if let Some(clip) = clip {
        device.drawPath(
            &crate::src::core::SkPath::SkPath::Polygon(
                clip,
                true,
                crate::include::core::SkPathTypes::SkPathFillType::kWinding,
                false,
            ),
            &paint,
        )
    } else {
        device.drawRect(r, &paint)
    }
}
// Source: src/core/SkDevice.cpp:254-294
unsafe fn draw_edge_aa_image_set<T: SkDeviceVirtual + ?Sized>(
    device: &mut T,
    images: &[crate::src::core::SkCanvas::ImageSetEntry],
    clips: &[crate::src::core::SkPoint::SkPoint],
    matrices: &[SkMatrix],
    sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    paint: &crate::src::core::SkPaint::SkPaint,
    constraint: crate::src::core::SkCanvas::SrcRectConstraint,
) -> Result<(), crate::RasterError> {
    crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 258; @condition b"paint.getStyle() == SkPaint::kFill_Style\0".as_ptr(); paint.getStyle()==crate::src::core::SkPaint::Style::kFill_Style);
    crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 259; @condition b"!paint.getPathEffect()\0".as_ptr(); paint.getPathEffect().is_none());
    let mut entry_paint = paint.clone();
    let base = device.base().local_to_device44().clone();
    let mut clip_index = 0;
    for image in images {
        entry_paint.setAntiAlias(
            image.fAAFlags == crate::src::core::SkCanvas::QuadAAFlags::kAll_QuadAAFlags.0,
        );
        entry_paint.setAlphaf(paint.getAlphaf() * image.fAlpha);
        if image.fMatrixIndex >= 0 {
            let local = base.clone() * SkM44::from(&matrices[image.fMatrixIndex as usize]);
            unsafe { device.base_mut() }.set_local_to_device(&local);
        }
        if image.fHasClip {
            device.pushClipStack();
            let clip = &clips[clip_index..clip_index + 4];
            let path = crate::src::core::SkPath::SkPath::Polygon(
                clip,
                true,
                crate::include::core::SkPathTypes::SkPathFillType::kWinding,
                false,
            );
            device.clipPath(
                &path,
                crate::include::core::SkClipOp::SkClipOp::kIntersect,
                entry_paint.isAntiAlias(),
            );
            clip_index += 4;
        }
        let result = unsafe {
            device.drawImageRect(
                image
                    .fImage
                    .as_ref()
                    .expect("native image-set entry owns image"),
                Some(unsafe { image.fSrcRect.assume_init_ref() }),
                unsafe { image.fDstRect.assume_init_ref() },
                sampling,
                &entry_paint,
                constraint,
            )
        };
        // Native pop/CTM restoration also runs for the adapted fallible draw.
        if image.fHasClip {
            device.popClipStack();
        }
        if image.fMatrixIndex >= 0 {
            unsafe { device.base_mut() }.set_local_to_device(&base);
        }
        result?;
    }
    Ok(())
}

// Source: src/core/SkDevice.cpp:298-300
unsafe fn draw_drawable_default<'device, 'canvas>(
    _device: core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
    drawable: &dyn crate::src::core::SkDrawable::SkDrawable,
    matrix: Option<&SkMatrix>,
) -> Result<(), crate::RasterError> {
    unsafe { drawable.draw(canvas, matrix) }
}

// Source: src/core/SkDevice.cpp:381-387
fn access_pixels<T: SkDeviceVirtual + ?Sized>(
    device: &mut T,
    pixels: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
) -> bool {
    let mut temporary = crate::src::core::SkPixmap::SkPixmap::default();
    device.onAccessPixels(pixels.unwrap_or(&mut temporary))
}
// Source: src/core/SkDevice.cpp:389-395
fn peek_pixels<T: SkDeviceVirtual + ?Sized>(
    device: &T,
    pixels: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
) -> bool {
    let mut temporary = crate::src::core::SkPixmap::SkPixmap::default();
    device.onPeekPixels(pixels.unwrap_or(&mut temporary))
}
// Source: src/core/SkDevice.cpp:399-422
// Current canonical shader factories retain Arc objects; native sk_sp ownership
// is still adapted. Both native local-matrix concatenation branches are preserved.
fn make_post_inverse_lm(
    shader: Option<&std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    lm: &SkMatrix,
) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
    let inverse_lm = lm.invert();
    let (Some(shader), Some(inverse_lm)) = (shader, inverse_lm) else {
        return None;
    };
    #[cfg(SK_BUILD_FOR_ANDROID_FRAMEWORK)]
    {
        let mut previous_local_matrix = SkMatrix::default();
        let nested_shader = shader.makeAsALocalMatrixShader(Some(&mut previous_local_matrix));
        let shader = nested_shader.as_ref().unwrap_or(shader);
        let mut local_matrix = SkMatrix::default();
        local_matrix.setConcat(&inverse_lm, &previous_local_matrix);
        return Some(shader.makeWithLocalMatrix(&local_matrix));
    }
    #[cfg(not(SK_BUILD_FOR_ANDROID_FRAMEWORK))]
    {
        Some(shader.makeWithLocalMatrix(&inverse_lm))
    }
}
// Rust-only raw form of the native virtual call; no borrowed receiver spans
// a callback. Concrete implementations retain the original complete-owner token.
pub type DrawGlyphRunListProc = for<'device, 'canvas> unsafe fn(
    core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
    core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
    &crate::src::text::GlyphRun::GlyphRunList,
    &crate::src::core::SkPaint::SkPaint,
) -> Result<(), crate::RasterError>;
// Source: src/core/SkDevice.h:609-609
unsafe fn draw_glyph_run_list_no_pixels<'device, 'canvas>(
    _device: core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
    _canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
    _list: &crate::src::text::GlyphRun::GlyphRunList,
    _paint: &crate::src::core::SkPaint::SkPaint,
) -> Result<(), crate::RasterError> {
    Ok(())
}

// Source: src/core/SkDevice.cpp:424-436
/// Safety: original device and Canvas raw operation leases remain live; no
/// whole-owner Rust borrow survives a Canvas callback that may mutate the CTM.
pub unsafe fn drawGlyphRunList_raw<'device, 'canvas>(
    device: core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
    list: &crate::src::text::GlyphRun::GlyphRunList,
    paint: &crate::src::core::SkPaint::SkPaint,
) -> Result<(), crate::RasterError> {
    if !unsafe { device.as_ref() }
        .base()
        .local_to_device()
        .isFinite()
    {
        return Ok(());
    }
    if !unsafe { list.hasRSXForm() } {
        let proc = unsafe { device.as_ref() }.drawGlyphRunListProc();
        unsafe { proc(device, canvas, list, paint) }
    } else {
        unsafe { simplifyGlyphRunRSXFormAndRedraw_raw(device, canvas, list, paint) }
    }
}
// Source: src/core/SkDevice.cpp:438-479
unsafe fn simplifyGlyphRunRSXFormAndRedraw_raw<'device, 'canvas>(
    device: core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
    list: &crate::src::text::GlyphRun::GlyphRunList,
    paint: &crate::src::core::SkPaint::SkPaint,
) -> Result<(), crate::RasterError> {
    use crate::src::core::{SkCanvas::SkCanvas, SkPoint::SkPoint, SkRSXform::SkRSXform};
    for run in unsafe { list.runs() } {
        if unsafe { run.scaledRotations() }.is_empty() {
            let sub = unsafe {
                core::pin::Pin::new_unchecked(&mut *list.builder()).makeGlyphRunList(
                    run,
                    paint,
                    SkPoint::make(0.0, 0.0),
                )
            }?;
            unsafe { drawGlyphRunList_raw(device, canvas, &sub, paint) }?;
        } else {
            let origin = list.origin();
            let shared_pos = [SkPoint::make(0.0, 0.0)];
            // Native sharedGlyphID is assigned before each recursive read.
            // Raw cell projection keeps that same storage alive across the run.
            let mut shared_id = core::mem::MaybeUninit::<u16>::uninit();
            for (i, (&id, &pos)) in unsafe { run.glyphsIDs() }
                .iter()
                .zip(unsafe { run.positions() })
                .enumerate()
            {
                shared_id.write(id);
                let rotation = unsafe { run.scaledRotations() }[i];
                let rsx = SkRSXform::Make(rotation.fX, rotation.fY, pos.fX, pos.fY);
                let mut matrix = SkMatrix::default();
                matrix.setRSXform(&rsx).postTranslate(origin.fX, origin.fY);
                let mut inverse_paint = paint.clone();
                inverse_paint.setShader(make_post_inverse_lm(paint.refShader().as_ref(), &matrix));
                let count = unsafe { canvas.as_ref() }.getSaveCount();
                unsafe { core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).save() };
                unsafe {
                    core::pin::Pin::new_unchecked(&mut *canvas.as_ptr())
                        .concat44(&crate::src::core::SkM44::SkM44::from(&matrix))
                };
                // Initialize the span only after assignment; this native one-glyph
                // run borrows shared_pos/shared_id until the recursive draw ends.
                let glyph_run = unsafe {
                    crate::src::text::GlyphRun::GlyphRun::new(
                        run.font(),
                        &shared_pos,
                        core::slice::from_raw_parts(shared_id.as_ptr(), 1),
                        &[],
                        &[],
                        &[],
                    )
                };
                let result =
                    (|| {
                        let sub =
                            unsafe {
                                core::pin::Pin::new_unchecked(&mut *list.builder())
                                    .makeGlyphRunList(&glyph_run, paint, SkPoint::make(0.0, 0.0))
                            }?;
                        unsafe { drawGlyphRunList_raw(device, canvas, &sub, &inverse_paint) }
                    })();
                // Restore on both Result paths; never discard restoration failure.
                let restore = unsafe { SkCanvas::restoreToCount_raw(canvas, count) };
                restore?;
                result?;
            }
        }
    }
    Ok(())
}

// Source: src/core/SkDevice.cpp:496-506
impl SkDevice {
    pub fn scaler_context_flags(&self) -> crate::src::core::SkScalerContext::SkScalerContextFlags {
        use crate::src::core::SkScalerContext::SkScalerContextFlags;
        if self.info.colorSpace().is_some_and(|cs| cs.gammaIsLinear()) {
            SkScalerContextFlags::BOOST_CONTRAST
        } else {
            SkScalerContextFlags::FAKE_GAMMA_AND_BOOST_CONTRAST
        }
    }
}

// Source: src/core/SkDevice.h:614-628
#[derive(Clone)]
struct ClipState {
    clip_bounds: SkIRect,
    deferred_save_count: i32,
    is_aa: bool,
    is_rect: bool,
}
impl ClipState {
    fn new(bounds: SkIRect, is_aa: bool, is_rect: bool) -> Self {
        Self {
            clip_bounds: bounds,
            deferred_save_count: 0,
            is_aa,
            is_rect,
        }
    }
}
use core::pin::Pin;

// Source: src/core/SkDevice.h:634-636
// Rust structural Pin preserves the native STArray inline-storage address.
// The base is movable independently; the private stack is never projected as
// an unpinned mutable reference or replaced after initialization.
pub struct SkNoPixelsDevice {
    // Rust field drop order preserves the derived stack before the base.
    clip_stack: STArray<ClipState, 4>,
    base: SkDevice,
}
impl SkNoPixelsDevice {
    fn stack(self: Pin<&mut Self>) -> Pin<&mut STArray<ClipState, 4>> {
        unsafe { self.map_unchecked_mut(|device| &mut device.clip_stack) }
    }
    // Source: src/core/SkDevice.h:632-632
    fn clip(&self) -> &ClipState {
        self.clip_stack.back()
    }
    // Source: src/core/SkDevice.cpp:510-523
    pub fn new(bounds: &SkIRect, props: &SkSurfaceProps) -> Pin<Box<Self>> {
        Self::with_color_space(bounds, props, None)
    }
    pub fn with_color_space(
        bounds: &SkIRect,
        props: &SkSurfaceProps,
        color_space: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    ) -> Pin<Box<Self>> {
        use crate::include::core::SkAlphaType::SkAlphaType::kUnknown_SkAlphaType;
        use crate::include::core::SkColorType::SkColorType::kUnknown_SkColorType;
        let info = SkImageInfo::MakeDimensionsWithColorSpace(
            SkISize::Make(bounds.width(), bounds.height()),
            kUnknown_SkColorType,
            kUnknown_SkAlphaType,
            color_space,
        );
        let mut base = SkDevice::new(&info, props);
        base.set_origin(&SkM44::default(), bounds.fLeft, bounds.fTop);
        let clip_bounds = base.bounds();
        let mut device = Box::pin(Self {
            base,
            clip_stack: STArray::new(),
        });
        device
            .as_mut()
            .stack()
            .push_back(ClipState::new(clip_bounds, false, true));
        device
    }
    // Source: src/core/SkDevice.cpp:524-542
    pub fn reset_for_next_picture(mut self: Pin<&mut Self>, bounds: &SkIRect) -> bool {
        crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 527; @condition b"this->isPixelAlignedToGlobal()\0".as_ptr(); self.base.is_pixel_aligned_to_global());
        if bounds.width() != self.base.width() || bounds.height() != self.base.height() {
            return false;
        }
        crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 534; @condition b"fClipStack.size() == 1 && fClipStack[0].fDeferredSaveCount == 0\0".as_ptr(); self.clip_stack.size() == 1 && self.clip_stack.at(0).deferred_save_count == 0);
        let clip_bounds = self.base.bounds();
        let clip = self.as_mut().stack().at_mut(0);
        clip.clip_bounds = clip_bounds;
        clip.is_aa = false;
        clip.is_rect = true;
        unsafe { self.get_unchecked_mut() }.base.set_origin(
            &SkM44::default(),
            bounds.fLeft,
            bounds.fTop,
        );
        true
    }
    // Source: src/core/SkDevice.cpp:544-547
    pub fn push_clip_stack(self: Pin<&mut Self>) {
        crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 545; @condition b"!fClipStack.empty()\0".as_ptr(); !self.clip_stack.empty());
        let current = self.stack().back_mut();
        current.deferred_save_count = current.deferred_save_count.wrapping_add(1);
    }
    // Source: src/core/SkDevice.cpp:549-557
    pub fn pop_clip_stack(mut self: Pin<&mut Self>) {
        crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 550; @condition b"!fClipStack.empty()\0".as_ptr(); !self.clip_stack.empty());
        if self.clip().deferred_save_count > 0 {
            self.as_mut().stack().back_mut().deferred_save_count -= 1;
        } else {
            self.as_mut().stack().pop_back();
            crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 555; @condition b"!fClipStack.empty()\0".as_ptr(); !self.clip_stack.empty());
        }
    }
    // Source: src/core/SkDevice.cpp:559-572
    fn writable_clip(mut self: Pin<&mut Self>) -> &mut ClipState {
        crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 560; @condition b"!fClipStack.empty()\0".as_ptr(); !self.clip_stack.empty());
        if self.clip().deferred_save_count > 0 {
            let (bounds, aa, rect) = {
                let current = self.as_mut().stack().back_mut();
                current.deferred_save_count -= 1;
                (current.clip_bounds, current.is_aa, current.is_rect)
            };
            self.stack().push_back(ClipState::new(bounds, aa, rect))
        } else {
            self.stack().back_mut()
        }
    }
    // Source: src/core/SkDevice.cpp:574-577
    pub fn clip_rect(self: Pin<&mut Self>, rect: &SkRect, op: SkClipOp, aa: bool) {
        let transform = *self.base.local_to_device44();
        self.writable_clip().op(op, &transform, rect, aa, true);
    }
    // Source: src/core/SkDevice.cpp:579-582
    pub fn clip_rrect(self: Pin<&mut Self>, rrect: &SkRRect, op: SkClipOp, aa: bool) {
        let transform = *self.base.local_to_device44();
        self.writable_clip()
            .op(op, &transform, &rrect.getBounds(), aa, rrect.isRect());
    }
    // Source: src/core/SkDevice.cpp:584-591
    pub fn clip_path(self: Pin<&mut Self>, path: &SkPath, mut op: SkClipOp, aa: bool) {
        if path.isInverseFillType() {
            op = if op == SkClipOp::kDifference {
                SkClipOp::kIntersect
            } else {
                SkClipOp::kDifference
            };
        }
        let transform = *self.base.local_to_device44();
        self.writable_clip()
            .op(op, &transform, &path.getBounds(), aa, false);
    }
    // Source: src/core/SkDevice.cpp:593-596
    pub fn clip_region(self: Pin<&mut Self>, region: &SkRegion, op: SkClipOp) {
        let transform = *self.base.global_to_device();
        self.writable_clip().op(
            op,
            &transform,
            &SkRect::Make(region.get_bounds()),
            false,
            region.is_rect(),
        );
    }
    // Source: src/core/SkDevice.cpp:598-600
    pub fn on_clip_shader(
        self: Pin<&mut Self>,
        _shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    ) {
        self.writable_clip().is_rect = false;
    }
    // Source: src/core/SkDevice.cpp:602-611
    pub fn replace_clip(self: Pin<&mut Self>, rect: &SkIRect) {
        let mut device_rect = crate::src::core::SkMatrixPriv::SkMatrixPriv::MapRect(
            self.base.global_to_device(),
            &SkRect::Make(rect),
        )
        .round();
        if !device_rect.intersect(&self.base.bounds()) {
            device_rect.set_empty();
        }
        let clip = self.writable_clip();
        clip.clip_bounds = device_rect;
        clip.is_rect = true;
        clip.is_aa = false;
    }
}
// The dynamic receiver is the complete native object, never a Box handle wrapper.
// Safety: construction pins the single allocation before inline storage is bound;
// the counter is embedded in its base and final disposal drops this complete type.
unsafe impl SkRefCntObject for SkNoPixelsDevice {
    // Source: src/core/SkDevice.h:563-563
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.base.ref_cnt_base()
    }
    fn ref_(&self) {
        self.base.ref_();
    }
}
unsafe impl SkDeviceVirtual for SkNoPixelsDevice {
    // Source: src/core/SkDevice.h:609-609
    fn drawGlyphRunListProc(&self) -> DrawGlyphRunListProc {
        draw_glyph_run_list_no_pixels
    }
    unsafe fn drawImageRect(
        &mut self,
        _image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        _src: Option<&crate::src::core::SkRect::SkRect>,
        _dst: &crate::src::core::SkRect::SkRect,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: &crate::src::core::SkPaint::SkPaint,
        _constraint: crate::src::core::SkCanvas::SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }

    // Source: src/core/SkDevice.h:605-605
    unsafe fn drawVertices(
        &mut self,
        _vertices: &crate::src::core::SkVertices::SkVertices,
        _blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        _paint: &crate::src::core::SkPaint::SkPaint,
        _skip_color_xform: bool,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:604-604
    unsafe fn drawDevice(
        &mut self,
        _device: core::ptr::NonNull<dyn SkDeviceVirtual + '_>,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }

    // Source: src/core/SkDevice.h:606-606
    unsafe fn drawMesh(
        &mut self,
        _mesh: &crate::src::core::SkMesh::SkMesh,
        _blender: sk_sp<dyn crate::include::core::SkBlender::SkBlender>,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:608-608
    unsafe fn drawSlug(
        &mut self,
        _canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
        _slug: &dyn crate::include::private::chromium::Slug::Slug,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:611-611
    fn isNoPixelsDevice(&self) -> bool {
        true
    }

    // Source: src/core/SkDevice.h:595-595
    fn drawPaint(
        &mut self,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:596-596
    fn drawPoints(
        &mut self,
        _mode: crate::src::core::SkCanvas::PointMode,
        _points: &[crate::src::core::SkPoint::SkPoint],
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:600-600
    fn drawRect(
        &mut self,
        _rect: &SkRect,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:601-601
    // Native NoPixelsDevice intentionally draws nothing; only void-to-Result is adapted.
    fn drawOval(
        &mut self,
        _oval: &SkRect,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:602-602
    fn drawRRect(
        &mut self,
        _rrect: &SkRRect,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
    // Source: src/core/SkDevice.h:603-603
    fn drawPath(
        &mut self,
        _path: &SkPath,
        _paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }

    // Delegate to the actual native NoPixels clip-state mutation.
    fn onClipShader(
        &mut self,
        shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    ) {
        unsafe { Pin::new_unchecked(self) }.on_clip_shader(shader);
    }

    fn base(&self) -> &SkDevice {
        &self.base
    }
    unsafe fn base_mut(&mut self) -> &mut SkDevice {
        &mut self.base
    }
    fn pushClipStack(&mut self) {
        unsafe { Pin::new_unchecked(self) }.push_clip_stack();
    }
    fn popClipStack(&mut self) {
        unsafe { Pin::new_unchecked(self) }.pop_clip_stack();
    }
    fn clipRect(&mut self, r: &SkRect, op: SkClipOp, aa: bool) {
        unsafe { Pin::new_unchecked(self) }.clip_rect(r, op, aa);
    }
    fn clipRRect(&mut self, r: &SkRRect, op: SkClipOp, aa: bool) {
        unsafe { Pin::new_unchecked(self) }.clip_rrect(r, op, aa);
    }
    fn clipPath(&mut self, p: &SkPath, op: SkClipOp, aa: bool) {
        unsafe { Pin::new_unchecked(self) }.clip_path(p, op, aa);
    }
    fn clipRegion(&mut self, r: &SkRegion, op: SkClipOp) {
        unsafe { Pin::new_unchecked(self) }.clip_region(r, op);
    }
    fn replaceClip(&mut self, r: &SkIRect) {
        unsafe { Pin::new_unchecked(self) }.replace_clip(r);
    }
    // Source: src/core/SkDevice.h:581-591
    fn isClipAntiAliased(&self) -> bool {
        self.clip().is_aa
    }
    fn isClipEmpty(&self) -> bool {
        self.devClipBounds().is_empty()
    }
    fn isClipRect(&self) -> bool {
        self.clip().is_rect && !self.isClipEmpty()
    }
    fn isClipWideOpen(&self) -> bool {
        self.clip().is_rect && self.devClipBounds() == self.base.bounds()
    }
    fn android_utils_clipAsRgn(&self, rgn: &mut SkRegion) {
        rgn.set_rect(self.devClipBounds());
    }
    fn devClipBounds(&self) -> SkIRect {
        self.clip().clip_bounds
    }
}

// C++ destroys the derived stack before the SkDevice base's retained image info.
impl Drop for SkNoPixelsDevice {
    fn drop(&mut self) {
        // Drop preserves the structural pin until all initialized ClipStates die.
        unsafe { Pin::new_unchecked(&mut self.clip_stack) }.clear();
    }
}

// Source: src/core/SkDevice.cpp:613-643
impl ClipState {
    fn op(
        &mut self,
        op: crate::include::core::SkClipOp::SkClipOp,
        transform: &SkM44,
        bounds: &crate::src::core::SkRect::SkRect,
        is_aa: bool,
        fills_bounds: bool,
    ) {
        use crate::include::core::SkClipOp::SkClipOp;
        use crate::src::core::SkMatrixPriv::SkMatrixPriv;
        use crate::src::core::SkRect::SkRect;
        use crate::src::core::SkRectPriv::SkRectPriv;
        let is_rect = fills_bounds && SkMatrixPriv::IsScaleTranslateAsM33(transform);
        self.is_aa |= is_aa;
        let dev_bounds = if bounds.isEmpty() {
            SkRect::MakeEmpty()
        } else {
            SkMatrixPriv::MapRect(transform, bounds)
        };
        if op == SkClipOp::kIntersect {
            if !self.clip_bounds.intersect(&if is_aa {
                dev_bounds.roundOut()
            } else {
                dev_bounds.round()
            }) {
                self.clip_bounds.set_empty();
            }
            self.is_rect &= is_rect;
        } else if is_rect {
            crate::SkASSERT!(@source b"src/core/SkDevice.cpp\0".as_ptr(), 630; @condition b"op == SkClipOp::kDifference\0".as_ptr(); (op) == (SkClipOp::kDifference));
            let mut difference = SkIRect::make_empty();
            if SkRectPriv::SubtractIRect(
                &self.clip_bounds,
                &if is_aa {
                    dev_bounds.roundIn()
                } else {
                    dev_bounds.round()
                },
                &mut difference,
            ) {
                self.clip_bounds = difference;
            } else {
                self.is_rect = false;
            }
        } else {
            self.is_rect = false;
        }
    }
}

// Rust ownership adaptation of the native SkDevice inheritance allocation.
// Transfer the existing allocation; do not allocate a Box around its pinned handle.
pub fn adopt_pinned_device<'device, T: SkDeviceVirtual + 'device>(
    device: Pin<Box<T>>,
) -> sk_sp<dyn SkDeviceVirtual + 'device> {
    assert!(
        device.ref_cnt_base().unique(),
        "adoption requires the initial sole reference"
    );
    // Safety: the unsafe virtual contract preserves the pinned complete allocation;
    // Box::into_raw consumes the sole owner without moving the allocation's contents.
    let object: *mut T = Box::into_raw(unsafe { Pin::into_inner_unchecked(device) });
    let dynamic: *mut (dyn SkDeviceVirtual + 'device) = object;
    unsafe { sk_sp::from_raw(dynamic) }
}
impl SkNoPixelsDevice {
    pub fn into_device(self: Pin<Box<Self>>) -> sk_sp<dyn SkDeviceVirtual> {
        adopt_pinned_device(self)
    }
}
/// Borrow the complete object for one native drawing/state operation.
/// # Safety
/// The pointer must have a live retained owner throughout the returned lease.
/// No other shared or exclusive references to this object or its base/recorder/
/// pixel context may overlap the lease. Retained handles may exist but may not be
/// dereferenced concurrently. Do not move/replace any structurally pinned fields.
/// The chosen borrow lifetime must be contained in that exclusive operation.
pub unsafe fn device_mut<'lease, 'device>(
    device: core::ptr::NonNull<dyn SkDeviceVirtual + 'device>,
) -> &'lease mut (dyn SkDeviceVirtual + 'device)
where
    'device: 'lease,
{
    unsafe { &mut *device.as_ptr() }
}

// Remaining fixed-file native dependencies (all original rows remain pending):
// src/core/SkDevice.h:284-289: actual Ganesh/Graphite context/device types.
// src/core/SkDevice.h:383-387,605-606; src/core/SkDevice.cpp:154-162,207-237:
//   actual SkVertices/SkMesh producer and native draw virtuals; patch uses
//   SkPatchUtils LOD/MakeVertices, atlas uses genuine SkVertices::Builder.
// src/core/SkDevice.h:608; src/core/SkDevice.cpp:481-487:
//   actual sktext::gpu::Slug owner/type. Even native null/abort defaults need
//   that real protocol; no erased placeholder or empty CPU replacement is used.

// Source: src/core/SkDevice.h:43-43
// Source: src/core/SkDevice.h:67-73
// Genuine native forward declarations: no size, constructor, fields or GPU backend.
unsafe extern "C" {
    pub type GrRecordingContext;
}
pub mod skgpu {
    pub mod ganesh {
        unsafe extern "C" {
            pub type Device;
        }
    }
    pub mod graphite {
        unsafe extern "C" {
            pub type Device;
            pub type Recorder;
        }
    }
}
