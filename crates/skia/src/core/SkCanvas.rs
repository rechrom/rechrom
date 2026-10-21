// Source: include/core/SkCanvas.h:173-176
impl<'device> SkCanvas<'device> {
    /// Safety: the declared N32 allocation remains live/initialized and
    /// exclusively leased throughout the pinned Canvas lifetime and all draws.
    pub unsafe fn MakeRasterDirectN32(
        width: i32,
        height: i32,
        pixels: *mut crate::src::core::SkColor::SkPMColor,
        row_bytes: usize,
    ) -> Option<core::pin::Pin<Box<Self>>> {
        unsafe {
            Self::MakeRasterDirect(
                &crate::src::core::SkImageInfo::SkImageInfo::MakeN32Premul(width, height),
                pixels.cast(),
                row_bytes,
                None,
            )
        }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:224-224
    /// Safety: retain the original bitmap's initialized storage and exclusive
    /// native Canvas raster lease for the complete returned pinned owner lifetime.
    pub unsafe fn with_bitmap(
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
    ) -> core::pin::Pin<Box<Self>> {
        unsafe { Self::with_bitmap_impl(bitmap) }
    }
    // Source: include/core/SkCanvas.h:256-256
    pub unsafe fn with_bitmap_props(
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> core::pin::Pin<Box<Self>> {
        unsafe { Self::with_bitmap_props_impl(bitmap, props) }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:648-650
    /// Safety: preserve the existing pinned Canvas/layer operation lease.
    pub unsafe fn saveLayer_rect(
        self: core::pin::Pin<&mut Self>,
        bounds: &crate::src::core::SkRect::SkRect,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<i32, crate::RasterError> {
        unsafe { self.saveLayer_bounds(Some(bounds), paint) }
    }
    // Source: include/core/SkCanvas.h:676-678
    pub unsafe fn saveLayerAlpha(
        self: core::pin::Pin<&mut Self>,
        bounds: Option<&crate::src::core::SkRect::SkRect>,
        alpha: u32,
    ) -> Result<i32, crate::RasterError> {
        unsafe { self.saveLayerAlphaf(bounds, alpha as f32 * (1.0 / 255.0)) }
    }
}

// Source: include/core/SkCanvas.h:685-690
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SaveLayerFlagsSet {
    kPreserveLCDText_SaveLayerFlag = 1 << 1,
    kInitWithPrevious_SaveLayerFlag = 1 << 2,
    kF16ColorType = 1 << 4,
}
// Source: include/core/SkCanvas.h:692-692
pub type SaveLayerFlags = u32;
// Source: include/core/SkCanvas.h:693-693
// Canonical ImageFilter currently uses Arc shared ownership; no cloned span owner.
pub type FilterSpan<'a> =
    &'a [Option<std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>];
// Source: include/core/SkCanvas.h:694-694
pub const kMaxFiltersPerLayer: i32 = 16;

// Source: include/core/SkCanvas.h:699-847
// Native non-owning descriptor: all optional pointers and the filter span borrow
// the caller's actual objects. Copying the descriptor never retains their owners.
// Arc-backed filter/color-space inputs borrow their canonical owner token, so a
// later native sk_ref_sp step can clone that exact owner without reconstructing
// an Arc from an arbitrary &dyn object/raw pointer.
#[derive(Clone, Copy)]
pub struct SaveLayerRec<'a> {
    pub fBounds: Option<&'a crate::src::core::SkRect::SkRect>,
    pub fPaint: Option<&'a crate::src::core::SkPaint::SkPaint>,
    pub fFilters: FilterSpan<'a>,
    pub fBackdrop: Option<&'a std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
    pub fBackdropTileMode: crate::include::core::SkTileMode::SkTileMode,
    pub fColorSpace: Option<&'a std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    pub fSaveLayerFlags: SaveLayerFlags,
    pub(crate) fExperimentalBackdropScale: f32,
}
impl Default for SaveLayerRec<'_> {
    fn default() -> Self {
        Self {
            fBounds: None,
            fPaint: None,
            fFilters: &[],
            fBackdrop: None,
            fBackdropTileMode: crate::include::core::SkTileMode::SkTileMode::kClamp,
            fColorSpace: None,
            fSaveLayerFlags: 0,
            fExperimentalBackdropScale: 1.0,
        }
    }
}
impl<'a> SaveLayerRec<'a> {
    pub fn new(
        bounds: Option<&'a crate::src::core::SkRect::SkRect>,
        paint: Option<&'a crate::src::core::SkPaint::SkPaint>,
        flags: SaveLayerFlags,
    ) -> Self {
        Self::with_parameters(
            bounds,
            paint,
            None,
            None,
            1.0,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            flags,
            &[],
        )
    }
    pub fn with_backdrop(
        bounds: Option<&'a crate::src::core::SkRect::SkRect>,
        paint: Option<&'a crate::src::core::SkPaint::SkPaint>,
        backdrop: Option<&'a std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        flags: SaveLayerFlags,
    ) -> Self {
        Self::with_parameters(
            bounds,
            paint,
            backdrop,
            None,
            1.0,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            flags,
            &[],
        )
    }
    pub fn with_color_space(
        bounds: Option<&'a crate::src::core::SkRect::SkRect>,
        paint: Option<&'a crate::src::core::SkPaint::SkPaint>,
        backdrop: Option<&'a std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        color_space: Option<&'a std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        flags: SaveLayerFlags,
    ) -> Self {
        Self::with_parameters(
            bounds,
            paint,
            backdrop,
            color_space,
            1.0,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            flags,
            &[],
        )
    }
    pub fn with_backdrop_tile_mode(
        bounds: Option<&'a crate::src::core::SkRect::SkRect>,
        paint: Option<&'a crate::src::core::SkPaint::SkPaint>,
        backdrop: Option<&'a std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        tile_mode: crate::include::core::SkTileMode::SkTileMode,
        color_space: Option<&'a std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        flags: SaveLayerFlags,
    ) -> Self {
        Self::with_parameters(
            bounds,
            paint,
            backdrop,
            color_space,
            1.0,
            tile_mode,
            flags,
            &[],
        )
    }
    pub(crate) fn with_parameters(
        bounds: Option<&'a crate::src::core::SkRect::SkRect>,
        paint: Option<&'a crate::src::core::SkPaint::SkPaint>,
        backdrop: Option<&'a std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        color_space: Option<&'a std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        backdrop_scale: f32,
        tile_mode: crate::include::core::SkTileMode::SkTileMode,
        flags: SaveLayerFlags,
        filters: FilterSpan<'a>,
    ) -> Self {
        crate::SkASSERT!(@source b"include/core/SkCanvas.h\0".as_ptr(), 839; @condition b"fFilters.empty() || !paint || !paint->getImageFilter()\0".as_ptr(); filters.is_empty()||paint.is_none()||paint.unwrap().getImageFilter().is_none());
        crate::SkASSERT!(@source b"include/core/SkCanvas.h\0".as_ptr(), 841; @condition b"fFilters.size() <= kMaxFiltersPerLayer\0".as_ptr(); filters.len()<=kMaxFiltersPerLayer as usize);
        Self {
            fBounds: bounds,
            fPaint: paint,
            fFilters: filters,
            fBackdrop: backdrop,
            fBackdropTileMode: tile_mode,
            fColorSpace: color_space,
            fSaveLayerFlags: flags,
            fExperimentalBackdropScale: backdrop_scale,
        }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1030-1032
    /// Safety: preserve the pinned Canvas/clip state lease.
    pub unsafe fn clipRect_op(
        self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            self.clipRect(rect, op, false);
        }
    }
    // Source: include/core/SkCanvas.h:1042-1044
    pub unsafe fn clipRect_intersect(
        self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRect::SkRect,
        anti_alias: bool,
    ) {
        unsafe {
            self.clipRect(
                rect,
                crate::include::core::SkClipOp::SkClipOp::kIntersect,
                anti_alias,
            );
        }
    }
}

// Source: include/core/SkCanvas.h:1046-1048
impl<'device> SkCanvas<'device> {
    /// Safety: raw points to the original live pinned Canvas with exclusive
    /// state access; this clip-only operation performs no Surface COW callback.
    pub unsafe fn clipIRect_raw(
        mut canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkIRect,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            core::pin::Pin::new_unchecked(canvas.as_mut()).clipRect(
                &crate::src::core::SkRect::SkRect::Make(rect),
                op,
                false,
            );
        }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1084-1086
    /// Safety: preserve the pinned Canvas/clip state lease.
    pub unsafe fn clipRRect_op(
        self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRRect::SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            self.clipRRect(rect, op, false);
        }
    }
    // Source: include/core/SkCanvas.h:1095-1097
    pub unsafe fn clipRRect_intersect(
        self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRRect::SkRRect,
        anti_alias: bool,
    ) {
        unsafe {
            self.clipRRect(
                rect,
                crate::include::core::SkClipOp::SkClipOp::kIntersect,
                anti_alias,
            );
        }
    }
    // Source: include/core/SkCanvas.h:1124-1126
    pub unsafe fn clipPath_op(
        self: core::pin::Pin<&mut Self>,
        path: &crate::src::core::SkPath::SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            self.clipPath(path, op, false);
        }
    }
    // Source: include/core/SkCanvas.h:1138-1140
    pub unsafe fn clipPath_intersect(
        self: core::pin::Pin<&mut Self>,
        path: &crate::src::core::SkPath::SkPath,
        anti_alias: bool,
    ) {
        unsafe {
            self.clipPath(
                path,
                crate::include::core::SkClipOp::SkClipOp::kIntersect,
                anti_alias,
            );
        }
    }
    // Source: include/core/SkCanvas.h:1200-1203
    pub fn getLocalClipBounds_into(&self, bounds: &mut crate::src::core::SkRect::SkRect) -> bool {
        *bounds = self.getLocalClipBounds();
        !bounds.isEmpty()
    }
    // Source: include/core/SkCanvas.h:1224-1227
    pub fn getDeviceClipBounds_into(&self, bounds: &mut crate::src::core::SkRect::SkIRect) -> bool {
        *bounds = self.getDeviceClipBounds();
        !bounds.is_empty()
    }
    // Source: include/core/SkCanvas.h:1237-1239
    /// Safety: preserve the original raw Canvas/Surface pixel and COW lease.
    pub unsafe fn drawColor_u32_raw(
        canvas: core::ptr::NonNull<Self>,
        color: crate::src::core::SkColor::SkColor,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::drawColor_raw(
                canvas,
                &crate::src::core::SkColor::SkColor4f::FromColor(color),
                mode,
            )
        }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1254-1256
    /// Safety: native Canvas/Surface and exclusive pixel leases hold for the
    /// complete draw; no outer mutable Canvas reference crosses Surface COW.
    pub unsafe fn clear_raw(
        canvas: core::ptr::NonNull<Self>,
        color: crate::src::core::SkColor::SkColor,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::clear4f_raw(
                canvas,
                &crate::src::core::SkColor::SkColor4f::FromColor(color),
            )
        }
    }
    // Source: include/core/SkCanvas.h:1263-1265
    pub unsafe fn clear4f_raw(
        canvas: core::ptr::NonNull<Self>,
        color: &crate::src::core::SkColor::SkColor4f,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::drawColor_raw(
                canvas,
                color,
                crate::src::core::SkBlendMode::SkBlendMode::kSrc,
            )
        }
    }
}

// Source: include/core/SkCanvas.h:1279-1279
impl<'device> SkCanvas<'device> {
    /// Safety: preserve the pinned original Canvas/Surface operation lease;
    /// no whole-owner/pixel reference may cross raw Surface COW.
    pub unsafe fn discard_raw(canvas: core::ptr::NonNull<Self>) {
        unsafe {
            Self::onDiscard_raw(canvas);
        }
    }
}

// Source: include/core/SkCanvas.h:1295-1299
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PointMode {
    kPoints_PointMode,
    kLines_PointMode,
    kPolygon_PointMode,
}

// Source: include/core/SkCanvas.h:1361-1363
impl<'device> SkCanvas<'device> {
    /// Safety: preserve original whole Canvas/device raster and filter leases.
    pub unsafe fn drawPoint_point_raw(
        canvas: core::ptr::NonNull<Self>,
        p: crate::src::core::SkPoint::SkPoint,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::drawPoint_raw(canvas, p.fX, p.fY, paint) }
    }
    // Source: include/core/SkCanvas.h:1389-1391
    pub unsafe fn drawLine_points_raw(
        canvas: core::ptr::NonNull<Self>,
        p0: crate::src::core::SkPoint::SkPoint,
        p1: crate::src::core::SkPoint::SkPoint,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::drawLine_raw(canvas, p0.fX, p0.fY, p1.fX, p1.fY, paint) }
    }
}

// Source: include/core/SkCanvas.h:1413-1417
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Same pinned Canvas and native pixel/filter lease as drawRect().
    pub unsafe fn drawIRect_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkIRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::drawRect_raw(canvas, &crate::src::core::SkRect::SkRect::Make(rect), paint)?;
        }
        Ok(())
    }
}
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1500-1502
    /// Safety: preserve the raw Canvas and exclusive native raster/filter lease.
    pub unsafe fn drawCircle_center_raw(
        canvas: core::ptr::NonNull<Self>,
        center: crate::src::core::SkPoint::SkPoint,
        radius: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::drawCircle_raw(canvas, center.fX, center.fY, radius, paint) }
    }
    // Source: include/core/SkCanvas.h:1544-1546
    pub unsafe fn drawArc_arc_raw(
        canvas: core::ptr::NonNull<Self>,
        arc: &crate::include::core::SkArc::SkArc,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::drawArc_raw(
                canvas,
                &arc.fOval,
                arc.fStartAngle,
                arc.fSweepAngle,
                arc.isWedge(),
                paint,
            )
        }
    }
}

// Source: include/core/SkCanvas.h:1583-1588
impl<'device> SkCanvas<'device> {
    /// Safety: preserve the Canvas and optional source image raster leases for
    /// the complete delegated native draw. None retains RETURN_ON_NULL behavior.
    pub unsafe fn drawImage_default_raw(
        canvas: core::ptr::NonNull<Self>,
        image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        left: f32,
        top: f32,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::drawImage_raw(
                canvas,
                image,
                left,
                top,
                &crate::include::core::SkSamplingOptions::SkSamplingOptions::new(),
                None,
            )
        }
    }
}
// Source: include/core/SkCanvas.h:1596-1599
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SrcRectConstraint {
    kStrict_SrcRectConstraint,
    kFast_SrcRectConstraint,
}

// Source: include/core/SkCanvas.h:1611-1615
impl<'device> SkCanvas<'device> {
    /// Safety: original source owner pixels and exclusive pinned Canvas raster
    /// lease remain live through the complete native draw/filter operation.
    pub unsafe fn drawImageRect_shared_raw(
        canvas: core::ptr::NonNull<Self>,
        image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        src: &crate::src::core::SkRect::SkRect,
        dst: &crate::src::core::SkRect::SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        let Some(image) = image else {
            return Ok(());
        };
        unsafe {
            Self::drawImageRect_raw(canvas, image, Some(src), dst, sampling, paint, constraint)
        }
    }
    // Source: include/core/SkCanvas.h:1616-1619
    pub unsafe fn drawImageRect_shared_dst_raw(
        canvas: core::ptr::NonNull<Self>,
        image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        dst: &crate::src::core::SkRect::SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let Some(image) = image else {
            return Ok(());
        };
        unsafe {
            Self::drawImageRect_raw(
                canvas,
                image,
                None,
                dst,
                sampling,
                paint,
                SrcRectConstraint::kFast_SrcRectConstraint,
            )
        }
    }
}

// Source: include/core/SkCanvas.h:1662-1666
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LatticeRectType {
    kDefault = 0,
    kTransparent,
    kFixedColor,
}

// Source: include/core/SkCanvas.h:1668-1675
// Raw pointers retain the native borrowed storage, including null optional arrays.
// No safe dereferencing API is exposed without the original external length contract.
pub struct Lattice {
    pub fXDivs: *const i32,
    pub fYDivs: *const i32,
    pub fRectTypes: *const LatticeRectType,
    pub fXCount: i32,
    pub fYCount: i32,
    pub fBounds: *const crate::src::core::SkRect::SkIRect,
    pub fColors: *const crate::src::core::SkColor::SkColor,
}

// Source: include/core/SkCanvas.h:1713-1721
// Newtype retains the unsigned bitmask representation and arbitrary edge combinations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct QuadAAFlags(pub u32);
impl QuadAAFlags {
    pub const kLeft_QuadAAFlag: Self = Self(0b0001);
    pub const kTop_QuadAAFlag: Self = Self(0b0010);
    pub const kRight_QuadAAFlag: Self = Self(0b0100);
    pub const kBottom_QuadAAFlag: Self = Self(0b1000);
    pub const kNone_QuadAAFlags: Self = Self(0b0000);
    pub const kAll_QuadAAFlags: Self = Self(0b1111);
}

// Source: include/core/SkCanvas.h:1736-1744
// Native shared-image owner follows image owner's documented Arc<dyn SkImage> contract.
// Native default construction leaves rectangle scalar storage uninitialized.
// MaybeUninit preserves that state in the SAME canonical owner; callers must
// initialize both fields before passing an entry to any unsafe drawing API.
// Source: include/core/SkCanvas.h:1732-1732
pub struct ImageSetEntry {
    pub fImage: Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
    // Source: include/core/SkCanvas.h:1737-1737
    pub fSrcRect: core::mem::MaybeUninit<crate::src::core::SkRect::SkRect>,
    // Source: include/core/SkCanvas.h:1738-1738
    pub fDstRect: core::mem::MaybeUninit<crate::src::core::SkRect::SkRect>,
    pub fMatrixIndex: i32,
    pub fAlpha: f32,
    pub fAAFlags: u32,
    pub fHasClip: bool,
}

// Source: include/core/SkCanvas.h:2241-2243
impl<'device> SkCanvas<'device> {
    /// Safety: same raw Canvas/Surface annotation lease as drawAnnotation_raw.
    pub unsafe fn drawAnnotation_shared_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        key: Option<&std::ffi::CStr>,
        value: &Option<std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) {
        unsafe {
            Self::drawAnnotation_raw(canvas, rect, key, value.as_ref());
        }
    }
}

// Source: include/core/SkCanvas.h:2276-2278
impl SkCanvas<'_> {
    pub fn getLocalToDeviceAs3x3(&self) -> crate::src::core::SkMatrix::SkMatrix {
        self.getLocalToDevice().asM33()
    }
}

// Source: include/core/SkCanvas.h:2317-2320
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SaveLayerStrategy {
    kFullLayer_SaveLayerStrategy,
    kNoLayer_SaveLayerStrategy,
}

// Source: include/core/SkCanvas.h:2322-2322
impl<'device> SkCanvas<'device> {
    unsafe fn willSave(&mut self) {
        let canvas = core::ptr::NonNull::from(&mut *self);
        unsafe {
            Self::willSave_raw(canvas);
        }
    }
    pub unsafe fn willSave_raw(canvas: core::ptr::NonNull<Self>) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::willSave(hooks.as_ref(), canvas);
            }
        } else {
            unsafe {
                Self::willSave_base_raw(canvas);
            }
        }
    }
    // Qualified INHERITED bypass: native default is empty.
    pub unsafe fn willSave_base_raw(_canvas: core::ptr::NonNull<Self>) {}
}
// Source: include/core/SkCanvas.h:2324-2326
impl<'device> SkCanvas<'device> {
    pub(crate) fn getSaveLayerStrategy(&self, rec: &SaveLayerRec<'_>) -> SaveLayerStrategy {
        if let Some(hooks) = self.fVirtual {
            unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtual::getSaveLayerStrategy(
                    hooks.as_ref(),
                    rec,
                )
            }
        } else {
            self.inherited_getSaveLayerStrategy(rec)
        }
    }
}
// Source: include/core/SkCanvas.h:2329-2329
impl<'device> SkCanvas<'device> {
    fn onDoSaveBehind(&self, bounds: Option<&crate::src::core::SkRect::SkRect>) -> bool {
        if let Some(hooks) = self.fVirtual {
            unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtual::onDoSaveBehind(
                    hooks.as_ref(),
                    bounds,
                )
            }
        } else {
            true
        }
    }
}
// Source: include/core/SkCanvas.h:2330-2330
impl<'device> SkCanvas<'device> {
    unsafe fn willRestore(&mut self) {
        let canvas = core::ptr::NonNull::from(&mut *self);
        unsafe {
            Self::willRestore_raw(canvas);
        }
    }
    pub unsafe fn willRestore_raw(canvas: core::ptr::NonNull<Self>) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::willRestore(hooks.as_ref(), canvas);
            }
        } else {
            unsafe {
                Self::willRestore_base_raw(canvas);
            }
        }
    }
    // Qualified INHERITED bypass: native default is empty.
    pub unsafe fn willRestore_base_raw(_canvas: core::ptr::NonNull<Self>) {}
}
// Source: include/core/SkCanvas.h:2331-2331
impl<'device> SkCanvas<'device> {
    unsafe fn didRestore(&mut self) {
        let canvas = core::ptr::NonNull::from(&mut *self);
        unsafe {
            Self::didRestore_raw(canvas);
        }
    }
    pub unsafe fn didRestore_raw(canvas: core::ptr::NonNull<Self>) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::didRestore(hooks.as_ref(), canvas);
            }
        } else {
            unsafe {
                Self::didRestore_base_raw(canvas);
            }
        }
    }
    // Qualified INHERITED bypass: native default is empty.
    pub unsafe fn didRestore_base_raw(_canvas: core::ptr::NonNull<Self>) {}
}
// Source: include/core/SkCanvas.h:2333-2333
impl<'device> SkCanvas<'device> {
    unsafe fn didConcat44(&mut self, m: &crate::src::core::SkM44::SkM44) {
        let canvas = core::ptr::NonNull::from(&mut *self);
        unsafe {
            Self::didConcat44_raw(canvas, m);
        }
    }
    pub unsafe fn didConcat44_raw(
        canvas: core::ptr::NonNull<Self>,
        m: &crate::src::core::SkM44::SkM44,
    ) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::didConcat44(hooks.as_ref(), canvas, m);
            }
        } else {
            unsafe {
                Self::didConcat44_base_raw(canvas, m);
            }
        }
    }
    // Qualified INHERITED bypass: native default is empty.
    pub unsafe fn didConcat44_base_raw(
        _canvas: core::ptr::NonNull<Self>,
        m: &crate::src::core::SkM44::SkM44,
    ) {
    }
}
// Source: include/core/SkCanvas.h:2334-2334
impl<'device> SkCanvas<'device> {
    unsafe fn didSetM44(&mut self, m: &crate::src::core::SkM44::SkM44) {
        let canvas = core::ptr::NonNull::from(&mut *self);
        unsafe {
            Self::didSetM44_raw(canvas, m);
        }
    }
    pub unsafe fn didSetM44_raw(
        canvas: core::ptr::NonNull<Self>,
        m: &crate::src::core::SkM44::SkM44,
    ) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::didSetM44(hooks.as_ref(), canvas, m);
            }
        } else {
            unsafe {
                Self::didSetM44_base_raw(canvas, m);
            }
        }
    }
    // Qualified INHERITED bypass: native default is empty.
    pub unsafe fn didSetM44_base_raw(
        _canvas: core::ptr::NonNull<Self>,
        m: &crate::src::core::SkM44::SkM44,
    ) {
    }
}
// Source: include/core/SkCanvas.h:2335-2335
impl<'device> SkCanvas<'device> {
    unsafe fn didTranslate(&mut self, dx: f32, dy: f32) {
        let canvas = core::ptr::NonNull::from(&mut *self);
        unsafe {
            Self::didTranslate_raw(canvas, dx, dy);
        }
    }
    pub unsafe fn didTranslate_raw(canvas: core::ptr::NonNull<Self>, dx: f32, dy: f32) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::didTranslate(hooks.as_ref(), canvas, dx, dy);
            }
        } else {
            unsafe {
                Self::didTranslate_base_raw(canvas, dx, dy);
            }
        }
    }
    // Qualified INHERITED bypass: native default is empty.
    pub unsafe fn didTranslate_base_raw(_canvas: core::ptr::NonNull<Self>, dx: f32, dy: f32) {}
}
// Source: include/core/SkCanvas.h:2336-2336
impl<'device> SkCanvas<'device> {
    unsafe fn didScale(&mut self, sx: f32, sy: f32) {
        let canvas = core::ptr::NonNull::from(&mut *self);
        unsafe {
            Self::didScale_raw(canvas, sx, sy);
        }
    }
    pub unsafe fn didScale_raw(canvas: core::ptr::NonNull<Self>, sx: f32, sy: f32) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::didScale(hooks.as_ref(), canvas, sx, sy);
            }
        } else {
            unsafe {
                Self::didScale_base_raw(canvas, sx, sy);
            }
        }
    }
    // Qualified INHERITED bypass: native default is empty.
    pub unsafe fn didScale_base_raw(_canvas: core::ptr::NonNull<Self>, sx: f32, sy: f32) {}
}

// Source: include/core/SkCanvas.h:2389-2392
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ClipEdgeStyle {
    kHard_ClipEdgeStyle,
    kSoft_ClipEdgeStyle,
}

// Source: include/core/SkCanvas.h:2413-2419
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct PredrawFlags(pub u32);
impl PredrawFlags {
    pub const kNone: Self = Self(0);
    pub const kOpaqueShaderOverride: Self = Self(1);
    pub const kNonOpaqueShaderOverride: Self = Self(2);
    pub const kCheckForOverwrite: Self = Self(4);
    pub const kSkipMaskFilterAutoLayer: Self = Self(8);
}

// Source: include/core/SkCanvas.h:2439-2442
impl<'device> SkCanvas<'device> {
    fn rootDevice(
        &self,
    ) -> core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device> {
        crate::SkASSERT!(@source b"include/core/SkCanvas.h\0".as_ptr(), 2440; @condition b"fRootDevice\0".as_ptr(); self.fRootDevice.is_some());
        self.fRootDevice.as_ptr().expect("native root device")
    }
}
// Source: include/core/SkCanvas.h:2451-2467
// Native member destruction is reversed in the field declarations. Pin preserves
// STArray inline addresses; current canonical ImageFilter owns shared Arc refs.
struct Layer<'device> {
    fIncludesPadding: bool,
    fDiscard: bool,
    fIsCoverage: bool,
    fPaint: Option<crate::src::core::SkPaint::SkPaint>,
    fImageFilters: crate::include::private::SkTArray::STArray<
        Option<std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
        1,
        true,
    >,
    fDevice: crate::include::core::SkRefCnt::sk_sp<
        dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device,
    >,
}
// Source: include/core/SkCanvas.h:2470-2480
struct BackImage {
    fLoc: crate::src::core::SkPoint::SkIPoint,
    fImage: Option<std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>>,
}
// Source: include/core/SkCanvas.h:2482-2508
// Placement records never move after construction. fDevice is non-owning and
// remains retained by the closest Layer or Canvas root until the record pops.
struct MCRec<'device> {
    fDeferredSaveCount: i32,
    fMatrix: crate::src::core::SkM44::SkM44,
    fBackImage: Option<Box<BackImage>>,
    fDevice: core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device>,
    fLayer: Option<core::pin::Pin<Box<Layer<'device>>>>,
}
// Source: include/core/SkCanvas.h:2510-2514
// Selected native build has no SK_CANVAS_SAVE_RESTORE_PREALLOC_COUNT override.
const kMCRecCount: usize = 32;
// Source: include/core/SkCanvas.h:2518-2540
// Canvas must be pinned before its external-storage deque is initialized. The
// Option is construction staging only; native deque drops at this field position.
// Internal 'static marker erases the self-borrow only: stack is private, never
// escapes Canvas, and is destroyed before its borrowed inline storage below.
// Device object lifetime is explicit and can retain borrowed recorder-backed
// devices; it is independent of the private inline deque marker.
// Intrusive device trait bound and omitted canonical allocator/surface/text
// modules are true pending dependencies, not alternate owner implementations.
// Source: include/core/SkCanvas.h:2686-2686
// Source: include/core/SkCanvas.h:2713-2714
pub struct SkCanvas<'device> {
    fRunBuildersUsed: i32,
    fRunBuilders: crate::include::private::SkTArray::STArray<
        Box<crate::src::text::GlyphRun::GlyphRunBuilder>,
        1,
        true,
    >,
    fQuickRejectBounds: crate::src::core::SkRect::SkRect,
    fClipRestrictionSaveCount: i32,
    fClipRestrictionRect: crate::src::core::SkRect::SkIRect,
    fSurfaceBase:
        Option<core::ptr::NonNull<crate::src::image::SkSurface_Base::SkSurface_Base<'device>>>,
    fAllocator:
        Option<Box<dyn crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator>>,
    fSaveCount: i32,
    fProps: crate::include::core::SkSurfaceProps::SkSurfaceProps,
    fRootDevice: crate::include::core::SkRefCnt::sk_sp<
        dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device,
    >,
    fMCRec: Option<core::ptr::NonNull<MCRec<'device>>>,
    fMCStack: Option<crate::src::core::SkDeque::SkDeque<'static>>,
    fMCRecStorage: [core::mem::MaybeUninit<MCRec<'device>>; kMCRecCount],
    fVirtual: Option<
        core::ptr::NonNull<
            dyn crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer<'device>
                + 'device,
        >,
    >,
    initialized: bool,
    pinned: core::marker::PhantomPinned,
}
impl<'device> SkCanvas<'device> {
    // Safety: the non-owning surface remains live and exclusively callable for the
    // remaining Canvas lifetime; its callbacks must not invalidate Canvas leases.
    unsafe fn setSurfaceBase(
        self: core::pin::Pin<&mut Self>,
        surface: Option<
            core::ptr::NonNull<crate::src::image::SkSurface_Base::SkSurface_Base<'device>>,
        >,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        this.fSurfaceBase = surface;
        unsafe {
            this.rebind_stack();
        }
    }
}
// Raw friend projections never borrow the whole Canvas across a Surface callback.
impl<'device> SkCanvas<'device> {
    pub(crate) unsafe fn top_device_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device> {
        let record = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fMCRec).read() }
            .expect("initialized Canvas MCRec");
        unsafe { core::ptr::addr_of!((*record.as_ptr()).fDevice).read() }
    }
    pub(crate) unsafe fn root_device_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device> {
        let root = unsafe { &*core::ptr::addr_of!((*canvas.as_ptr()).fRootDevice) };
        root.as_ptr().expect("native root device")
    }
    pub(crate) unsafe fn get_surface_base_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> Option<core::ptr::NonNull<crate::src::image::SkSurface_Base::SkSurface_Base<'device>>>
    {
        unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fSurfaceBase).read() }
    }
    pub(crate) unsafe fn set_surface_base_raw(
        canvas: core::ptr::NonNull<Self>,
        surface: Option<
            core::ptr::NonNull<crate::src::image::SkSurface_Base::SkSurface_Base<'device>>,
        >,
    ) {
        unsafe {
            core::ptr::addr_of_mut!((*canvas.as_ptr()).fSurfaceBase).write(surface);
        }
    }
    // Source: include/core/SkCanvas.h:2546-2546
    // The actual Surface destructor reaches the complete derived Canvas hook.
    pub(crate) unsafe fn on_surface_delete_raw(canvas: core::ptr::NonNull<Self>) {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::onSurfaceDelete(hooks.as_ref(), canvas);
            }
        } else {
            unsafe {
                Self::on_surface_delete_base_raw(canvas);
            }
        }
    }
    // Qualified inherited bypass: the native base virtual is genuinely empty.
    pub(crate) unsafe fn on_surface_delete_base_raw(_canvas: core::ptr::NonNull<Self>) {}
}
// Native field location: include/core/SkCanvas.h line2686; unmapped declaration adaptation.
// fQuickRejectBounds is declared in the complete Canvas above in native drop order.
// Native field location: include/core/SkCanvas.h lines2713-2714; unmapped declaration adaptation.
// fRunBuilders/fRunBuildersUsed are declared above; text GlyphRun scope is pending.

// Source: include/core/SkCanvas.h:2567-2568
impl<'device> SkCanvas<'device> {
    /// Safety: retain the native bitmap allocation/release callback and allocator
    /// handle lease throughout the complete pinned Canvas lifetime and teardown.
    pub(crate) unsafe fn with_bitmap_allocator(
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        allocator: Option<
            Box<dyn crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator>,
        >,
        handle: crate::include::core::SkRasterHandleAllocator::Handle,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> core::pin::Pin<Box<Self>> {
        unsafe { Self::with_bitmap_allocator_impl(bitmap, allocator, handle, props) }
    }
}

// Source: include/core/SkCanvas.h:2628-2638
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
enum DeviceCompatibleWithFilter {
    kUnknown,
    kYes,
    kYesWithPadding,
}

// Rust friend access for explicit fallible filter-layer finalization. On Err,
// an uncommitted save record is canceled without painting before error return.
impl<'device> SkCanvas<'device> {
    pub(crate) unsafe fn rebind_filter_guard(&mut self) {
        unsafe {
            self.rebind_stack();
        }
    }
    // Borrow-provenance/error-exit adaptation shared by the native matrix/paint guard.
    pub(crate) unsafe fn cancel_matrix_paint_raw(canvas: core::ptr::NonNull<Self>, count: i32) {
        while unsafe { canvas.as_ref() }.getSaveCount() > count {
            unsafe {
                (&mut *canvas.as_ptr()).cancel_restore_inner();
            }
        }
    }

    pub(crate) unsafe fn restore_filter_layer_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            (*canvas.as_ptr()).fSaveCount -= 1;
            Self::internalRestore_raw(canvas)
        }
    }
    pub(crate) unsafe fn cancel_filter_layer(&mut self) {
        self.fSaveCount -= 1;
        unsafe {
            self.cancel_internal_restore();
        }
    }
    unsafe fn begin_layer_record(
        &mut self,
        rec: &SaveLayerRec<'_>,
        strategy: SaveLayerStrategy,
        coverage_only: bool,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.rebind_stack();
        }
        let save_count_anchor = self.fSaveCount;
        let record_count_anchor = self.fMCStack.as_ref().unwrap().count();
        self.fSaveCount += 1;
        let result = unsafe { self.internalSaveLayer(rec, strategy, coverage_only) };
        if result.is_err() {
            // addLayer has not accepted ownership yet. Cancel every record created
            // by this call, including a record pushed before a fallible filter query.
            // This is state/lease rollback only; already written pixels stay written.
            while self.fMCStack.as_ref().unwrap().count() > record_count_anchor {
                self.fSaveCount -= 1;
                unsafe {
                    self.cancel_internal_restore();
                }
            }
            self.fSaveCount = save_count_anchor;
            unsafe {
                self.finish_restore_bookkeeping();
            }
        }
        result
    }
    pub(crate) unsafe fn save_filter_layer(
        &mut self,
        paint: &crate::src::core::SkPaint::SkPaint,
        bounds: Option<&crate::src::core::SkRect::SkRect>,
        coverage_only: bool,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            self.begin_layer_record(
                &SaveLayerRec::new(bounds, Some(paint), 0),
                SaveLayerStrategy::kFullLayer_SaveLayerStrategy,
                coverage_only,
            )
        }
    }
}

// Source: src/core/SkCanvas.cpp:84-84
// Adapted: one Rust unsigned mask newtype represents native enum + SkEnumBitMask.
impl core::ops::BitOr for PredrawFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl core::ops::BitAnd for PredrawFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl core::ops::BitXor for PredrawFlags {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl core::ops::Not for PredrawFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

// Source: src/core/SkCanvas.cpp:94-141
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2666-2667
    fn wouldOverwriteEntireSurface(
        &self,
        rect: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        flags: PredrawFlags,
    ) -> bool {
        use crate::src::core::SkPaintPriv::ShaderOverrideOpacity;
        let opacity = if (flags & PredrawFlags::kOpaqueShaderOverride).0 != 0 {
            ShaderOverrideOpacity::kOpaque_ShaderOverrideOpacity
        } else if (flags & PredrawFlags::kNonOpaqueShaderOverride).0 != 0 {
            ShaderOverrideOpacity::kNotOpaque_ShaderOverrideOpacity
        } else {
            ShaderOverrideOpacity::kNone_ShaderOverrideOpacity
        };
        let size = self.getBaseLayerSize();
        let bounds = crate::src::core::SkRect::SkRect::MakeIWH(size.width(), size.height());
        let root = self.rootDevice();
        let top = self.topDevice();
        if !core::ptr::addr_eq(root.as_ptr(), top.as_ptr()) {
            return false;
        }
        if !unsafe { root.as_ref() }.isClipWideOpen() {
            return false;
        }
        if let Some(rect) = rect {
            if !self.getTotalMatrix().isScaleTranslate() {
                return false;
            }
            let mut dev_rect = crate::src::core::SkRect::SkRect::default();
            self.getTotalMatrix()
                .mapRectScaleTranslate(&mut dev_rect, rect);
            if !dev_rect.contains(&bounds) {
                return false;
            }
        }
        if let Some(paint) = paint {
            let style = paint.getStyle();
            if !(style == crate::src::core::SkPaint::Style::kFill_Style
                || style == crate::src::core::SkPaint::Style::kStrokeAndFill_Style)
            {
                return false;
            }
            if paint.getMaskFilter().is_some()
                || paint.getPathEffect().is_some()
                || paint.getImageFilter().is_some()
            {
                return false;
            }
        }
        crate::src::core::SkPaintPriv::Overwrites(paint, opacity)
    }
}
// Source: src/core/SkCanvas.cpp:145-154
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2425-2425
    pub unsafe fn predraw_notify_overwrite_raw(
        canvas: core::ptr::NonNull<Self>,
        overwrite: bool,
    ) -> bool {
        unsafe {
            (&mut *canvas.as_ptr()).rebind_stack();
        }
        let surface = unsafe { Self::get_surface_base_raw(canvas) };
        if let Some(surface) = surface {
            let mode = if overwrite {
                crate::src::image::SkSurface::ContentChangeMode::kDiscard_ContentChangeMode
            } else {
                crate::src::image::SkSurface::ContentChangeMode::kRetain_ContentChangeMode
            };
            if !unsafe {
                crate::src::image::SkSurface_Base::SkSurface_Base::aboutToDraw_raw(surface, mode)
            } {
                return false;
            }
        }
        true
    }
}
// Source: src/core/SkCanvas.cpp:156-173
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2426-2426
    pub unsafe fn predraw_notify_rect_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        flags: PredrawFlags,
    ) -> bool {
        unsafe {
            (&mut *canvas.as_ptr()).rebind_stack();
        }
        let surface = unsafe { Self::get_surface_base_raw(canvas) };
        if let Some(surface) = surface {
            let mut mode =
                crate::src::image::SkSurface::ContentChangeMode::kRetain_ContentChangeMode;
            if unsafe {
                crate::src::image::SkSurface_Base::SkSurface_Base::outstandingImageSnapshot_raw(
                    surface,
                )
            } {
                if unsafe { canvas.as_ref() }.wouldOverwriteEntireSurface(rect, paint, flags) {
                    mode =
                        crate::src::image::SkSurface::ContentChangeMode::kDiscard_ContentChangeMode;
                }
            }
            if !unsafe {
                crate::src::image::SkSurface_Base::SkSurface_Base::aboutToDraw_raw(surface, mode)
            } {
                return false;
            }
        }
        true
    }
}

// Source: src/core/SkCanvas.cpp:178-193
impl<'device> Layer<'device> {
    fn new(
        device: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device,
        >,
        filters: &[Option<std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>],
        paint: &crate::src::core::SkPaint::SkPaint,
        is_coverage: bool,
        includes_padding: bool,
    ) -> core::pin::Pin<Box<Self>> {
        let mut layer = Box::pin(Self {
            fDevice: device,
            fImageFilters: crate::include::private::SkTArray::STArray::new(),
            fPaint: None,
            fIsCoverage: is_coverage,
            fDiscard: false,
            fIncludesPadding: includes_padding,
        });
        unsafe { layer.as_mut().map_unchecked_mut(|l| &mut l.fImageFilters) }
            .init_from_slice(filters);
        // Native filters construct before paint. Option is only staged initialization;
        // every fully constructed Layer contains Some(actual Paint).
        unsafe { layer.as_mut().get_unchecked_mut() }.fPaint = Some(paint.clone());
        unsafe { layer.as_mut().map_unchecked_mut(|l| &mut l.fImageFilters) }.init();
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 189; @condition b"fDevice\0".as_ptr(); layer.fDevice.is_some());
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 192; @condition b"!fPaint.getImageFilter()\0".as_ptr(); layer.fPaint.as_ref().unwrap().getImageFilter().is_none());
        layer
    }
}
// Source: src/core/SkCanvas.cpp:195-200
impl BackImage {
    fn new(
        image: std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage>,
        loc: crate::src::core::SkPoint::SkIPoint,
    ) -> Self {
        // Native fImage(img) retains the lvalue parameter, which dies on return.
        let retained = image.clone();
        Self {
            fImage: Some(retained),
            fLoc: loc,
        }
    }
    fn move_from(source: &mut Self) -> Self {
        let image = source.fImage.take();
        let loc = source.fLoc;
        Self {
            fImage: image,
            fLoc: loc,
        }
    }
    // Valid saved records always have an image; the temporary move source is
    // destroyed without ever entering restore/draw.
    unsafe fn image(
        &self,
    ) -> &std::sync::Arc<dyn crate::src::core::SkSpecialImage::SkSpecialImage> {
        unsafe { self.fImage.as_ref().unwrap_unchecked() }
    }
}
// Source: src/core/SkCanvas.cpp:197-200
impl Clone for BackImage {
    fn clone(&self) -> Self {
        let image = self.fImage.clone();
        let loc = self.fLoc;
        Self {
            fImage: image,
            fLoc: loc,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        let retained = source.fImage.clone();
        let old = core::mem::replace(&mut self.fImage, retained);
        drop(old);
        self.fLoc = source.fLoc;
    }
}
// Source: src/core/SkCanvas.cpp:202-210
impl<'device> MCRec<'device> {
    fn new(
        device: core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device>,
    ) -> Self {
        Self {
            fLayer: None,
            fDevice: device,
            fBackImage: None,
            fMatrix: crate::src::core::SkM44::SkM44::default(),
            fDeferredSaveCount: 0,
        }
    }
    fn from_previous(previous: &Self) -> Self {
        Self {
            fLayer: None,
            fDevice: previous.fDevice,
            fBackImage: None,
            fMatrix: previous.fMatrix.clone(),
            fDeferredSaveCount: 0,
        }
    }
}
// Source: src/core/SkCanvas.cpp:212-223
impl<'device> MCRec<'device> {
    fn newLayer(
        &mut self,
        device: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device,
        >,
        filters: &[Option<std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>],
        paint: &crate::src::core::SkPaint::SkPaint,
        is_coverage: bool,
        includes_padding: bool,
    ) {
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 217; @condition b"!fBackImage\0".as_ptr(); self.fBackImage.is_none());
        let replacement = Some(Layer::new(
            device,
            filters,
            paint,
            is_coverage,
            includes_padding,
        ));
        let old = core::mem::replace(&mut self.fLayer, replacement);
        drop(old);
        self.fDevice = self
            .fLayer
            .as_ref()
            .unwrap()
            .fDevice
            .as_ptr()
            .expect("native layer device");
    }
}
// Source: src/core/SkCanvas.cpp:225-231
impl<'device> MCRec<'device> {
    fn reset(
        &mut self,
        device: core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device>,
    ) {
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 227; @condition b"!fLayer\0".as_ptr(); self.fLayer.is_none());
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 229; @condition b"fDeferredSaveCount == 0\0".as_ptr(); (self.fDeferredSaveCount) == (0));
        self.fDevice = device;
        self.fMatrix.setIdentity();
    }
}

// Source: src/core/SkCanvas.cpp:234-254
struct AutoUpdateQRBounds<'a, 'device> {
    fCanvas: &'a mut SkCanvas<'device>,
}
impl<'a, 'device> AutoUpdateQRBounds<'a, 'device> {
    unsafe fn new(canvas: &'a mut SkCanvas<'device>) -> Self {
        unsafe {
            canvas.rebind_stack();
        }
        canvas.validateClip();
        Self { fCanvas: canvas }
    }
}
impl Drop for AutoUpdateQRBounds<'_, '_> {
    fn drop(&mut self) {
        unsafe {
            self.fCanvas.rebind_stack();
        }
        self.fCanvas.fQuickRejectBounds = self.fCanvas.computeDeviceClipBounds(true);
        self.fCanvas.validateClip();
    }
}

// Source: src/core/SkCanvas.cpp:258-283
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2429-2432
    unsafe fn aboutToDraw_raw<'operation>(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
        raw_bounds: Option<&crate::src::core::SkRect::SkRect>,
        flags: PredrawFlags,
    ) -> Result<
        Option<crate::src::core::SkCanvasPriv::AutoLayerForImageFilter<'operation, 'device>>,
        crate::RasterError,
    > {
        if (flags & PredrawFlags::kCheckForOverwrite).0 != 0 {
            if !unsafe { Self::predraw_notify_rect_raw(canvas, raw_bounds, Some(paint), flags) } {
                return Ok(None);
            }
        } else if !unsafe { Self::predraw_notify_overwrite_raw(canvas, false) } {
            return Ok(None);
        }
        let skip_mask_layer = (flags & PredrawFlags::kSkipMaskFilterAutoLayer).0 != 0
            || !unsafe { Self::top_device_raw(canvas).as_ref() }
                .useDrawCoverageMaskForMaskFilters();
        Ok(Some(unsafe {
            crate::src::core::SkCanvasPriv::AutoLayerForImageFilter::new(
                canvas,
                paint,
                raw_bounds,
                skip_mask_layer,
            )
        }?))
    }
    // Source: include/core/SkCanvas.h:2433-2435
    unsafe fn aboutToDraw_default_raw<'operation>(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
        raw_bounds: Option<&crate::src::core::SkRect::SkRect>,
    ) -> Result<
        Option<crate::src::core::SkCanvasPriv::AutoLayerForImageFilter<'operation, 'device>>,
        crate::RasterError,
    > {
        unsafe { Self::aboutToDraw_raw(canvas, paint, raw_bounds, PredrawFlags::kNone) }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2604-2604
    // Source: src/core/SkCanvas.cpp:287-302
    /// Safety: Canvas is the original pinned allocation and its root is the
    /// actual complete SkNoPixelsDevice (the same native static-cast invariant).
    /// No external device/Canvas reference may overlap restore or root replacement.
    pub(crate) unsafe fn resetForNextPicture_raw(
        canvas: core::ptr::NonNull<Self>,
        bounds: &crate::src::core::SkRect::SkIRect,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::restoreToCount_raw(canvas, 1) }?;
        let root = unsafe { Self::root_device_raw(canvas) };
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 292; @condition b"fRootDevice->isNoPixelsDevice()\0".as_ptr(); unsafe { root.as_ref() }.isNoPixelsDevice());
        let complete = root.as_ptr() as *mut crate::src::core::SkDevice::SkNoPixelsDevice;
        let reset =
            unsafe { core::pin::Pin::new_unchecked(&mut *complete) }.reset_for_next_picture(bounds);
        if !reset {
            let (props, color_space) = {
                let original = unsafe { root.as_ref() };
                (
                    *original.base().surface_props(),
                    original.base().image_info().refColorSpace(),
                )
            };
            let device = crate::src::core::SkDevice::SkNoPixelsDevice::with_color_space(
                bounds,
                &props,
                color_space,
            );
            let device = crate::src::core::SkDevice::adopt_pinned_device(device);
            let previous =
                unsafe { core::ptr::addr_of_mut!((*canvas.as_ptr()).fRootDevice).replace(device) };
            drop(previous);
        }
        let mut record = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fMCRec).read() }
            .expect("native initialized MCRec");
        unsafe { record.as_mut() }.reset(unsafe { Self::root_device_raw(canvas) });
        let bounds = unsafe { canvas.as_ref() }.computeDeviceClipBounds(true);
        unsafe {
            core::ptr::addr_of_mut!((*canvas.as_ptr()).fQuickRejectBounds).write(bounds);
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:304-322
impl<'device> SkCanvas<'device> {
    // Safety: called only after pinning the complete Canvas and binding its native
    // deque storage. Device owners and their raw mutable dispatch require exclusive
    // native access throughout Canvas lifetime, including destruction/restore.
    // Source: include/core/SkCanvas.h:2609-2609
    unsafe fn init(
        &mut self,
        mut device: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device,
        >,
    ) {
        let storage = core::ptr::addr_of_mut!(self.fMCRecStorage).cast::<u8>();
        unsafe {
            self.fMCStack
                .as_mut()
                .unwrap()
                .rebind_initial_storage(storage, core::mem::size_of_val(&self.fMCRecStorage));
        }
        if !device.is_some() {
            device = crate::src::core::SkDevice::adopt_pinned_device(
                crate::src::core::SkDevice::SkNoPixelsDevice::new(
                    &crate::src::core::SkRect::SkIRect::make_empty(),
                    &self.fProps,
                ),
            );
        }
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 310; @condition b"device\0".as_ptr(); device.is_some());
        self.fSaveCount = 1;
        let record = MCRec::new(device.as_ptr().expect("native root device"));
        let stack = unsafe { self.fMCStack.as_mut().unwrap() };
        let dst = unsafe { stack.push_back().cast::<MCRec<'device>>() };
        unsafe {
            dst.write(record);
        }
        self.fMCRec = core::ptr::NonNull::new(dst);
        debug_assert_eq!(
            self.fProps.pixelGeometry(),
            device.base().surface_props().pixelGeometry()
        );
        self.fSurfaceBase = None;
        self.fRootDevice = device;
        self.initialized = true;
        unsafe { core::pin::Pin::new_unchecked(&mut self.fRunBuilders) }
            .push_back(Box::new(crate::src::text::GlyphRun::GlyphRunBuilder::new()));
        self.fQuickRejectBounds = self.computeDeviceClipBounds(true);
    }
}
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:185-185
    // Source: src/core/SkCanvas.cpp:324-326
    // Adapted: the actual self-referential deque is initialized only after pinning.
    pub fn new() -> core::pin::Pin<Box<Self>> {
        unsafe {
            Self::with_device_props_allocator(
                crate::include::core::SkRefCnt::sk_sp::default(),
                crate::include::core::SkSurfaceProps::SkSurfaceProps::default(),
                None,
            )
        }
    }
    // Source: include/core/SkCanvas.h:204-204
    // Source: src/core/SkCanvas.cpp:328-333
    pub fn with_dimensions(
        width: i32,
        height: i32,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> core::pin::Pin<Box<Self>> {
        let props = crate::src::core::SkSurfacePriv::SkSurfacePropsCopyOrDefault(props);
        let bounds = crate::src::core::SkRect::SkIRect::make_wh(width.max(0), height.max(0));
        let device = crate::src::core::SkDevice::SkNoPixelsDevice::new(&bounds, &props);
        unsafe {
            Self::with_device_props_allocator(
                crate::src::core::SkDevice::adopt_pinned_device(device),
                props,
                None,
            )
        }
    }
    // Source: include/core/SkCanvas.h:2564-2564
    // Source: src/core/SkCanvas.cpp:335-339
    pub(crate) fn with_bounds(
        bounds: &crate::src::core::SkRect::SkIRect,
    ) -> core::pin::Pin<Box<Self>> {
        let props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
        let rect = if bounds.is_empty() {
            crate::src::core::SkRect::SkIRect::make_empty()
        } else {
            *bounds
        };
        let device = crate::src::core::SkDevice::SkNoPixelsDevice::new(&rect, &props);
        unsafe {
            Self::with_device_props_allocator(
                crate::src::core::SkDevice::adopt_pinned_device(device),
                props,
                None,
            )
        }
    }
}

// Source: src/core/SkCanvas.cpp:341-345
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// The native shared device handle must support exclusive mutable device/pixel
    /// access for this Canvas lifetime, including Drop. External owners may retain
    /// it but must not concurrently access or invalidate it or its raw pixel backing.
    // Source: include/core/SkCanvas.h:208-208
    pub unsafe fn with_device(
        device: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device,
        >,
    ) -> core::pin::Pin<Box<Self>> {
        let props = *device.base().surface_props();
        unsafe { Self::with_device_props_allocator(device, props, None) }
    }
    // Shared native constructor staging: initialize allocator before init(), while
    // the real Deque borrows the final pinned complete Canvas allocation.
    pub(crate) unsafe fn with_device_props_allocator(
        device: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device,
        >,
        props: crate::include::core::SkSurfaceProps::SkSurfaceProps,
        allocator: Option<
            Box<dyn crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator>,
        >,
    ) -> core::pin::Pin<Box<Self>> {
        let mut canvas = Box::pin(Self {
            fRunBuildersUsed: 0,
            fRunBuilders: crate::include::private::SkTArray::STArray::new(),
            fQuickRejectBounds: crate::src::core::SkRect::SkRect::default(),
            fClipRestrictionSaveCount: -1,
            fClipRestrictionRect: crate::src::core::SkRect::SkIRect::make_empty(),
            fSurfaceBase: None,
            fAllocator: allocator,
            fSaveCount: 0,
            fProps: props,
            fRootDevice: crate::include::core::SkRefCnt::sk_sp::default(),
            fMCRec: None,
            fMCStack: None,
            fMCRecStorage: [const { core::mem::MaybeUninit::uninit() }; kMCRecCount],
            fVirtual: None,
            initialized: false,
            pinned: core::marker::PhantomPinned,
        });
        let this = unsafe { canvas.as_mut().get_unchecked_mut() };
        let storage = core::ptr::addr_of_mut!(this.fMCRecStorage).cast::<u8>();
        this.fMCStack = Some(unsafe {
            crate::src::core::SkDeque::SkDeque::with_storage(
                core::mem::size_of::<MCRec<'device>>(),
                storage,
                core::mem::size_of_val(&this.fMCRecStorage),
                1,
            )
        });
        unsafe {
            this.init(device);
        }
        canvas
    }
}
// Source: src/core/SkCanvas.cpp:347-362
// Source: include/core/SkCanvas.h:263-263
impl<'device> Drop for SkCanvas<'device> {
    fn drop(&mut self) {
        if !self.initialized {
            return;
        }
        unsafe {
            self.rebind_stack();
        }
        {
            let stack = unsafe { self.fMCStack.as_ref().unwrap() };
            let mut iter = crate::src::core::SkDeque::Iter::new(
                stack,
                crate::src::core::SkDeque::IterStart::kFront_IterStart,
            );
            loop {
                let ptr = iter.next();
                if ptr.is_null() {
                    break;
                }
                if let Some(layer) = unsafe { &mut *ptr.cast::<MCRec<'device>>() }
                    .fLayer
                    .as_mut()
                {
                    unsafe { layer.as_mut().get_unchecked_mut() }.fDiscard = true;
                }
            }
        }
        // Cancellation is not successful finalization: try_finish()/restoreToCount(1)
        // must be called explicitly and its Result observed before frame publication.
        while self.fSaveCount > 1 {
            unsafe {
                self.cancel_restore_inner();
            }
        }
        unsafe {
            self.cancel_internal_restore();
        }
        // Records are destroyed above. The staged Option<actual SkDeque> now drops
        // naturally at its field position, after builders/allocator/root device and
        // before inline storage, matching the native reverse member order.
        self.initialized = false;
    }
}
// Source: include/core/SkCanvas.h:348-348
// Source: src/core/SkCanvas.cpp:365-367
impl<'device> SkCanvas<'device> {
    /// Borrowed original surface identity; never retains or reconstructs an owner.
    pub fn getSurface(
        &self,
    ) -> Option<core::ptr::NonNull<crate::src::image::SkSurface_Base::SkSurface_Base<'device>>>
    {
        self.fSurfaceBase
    }
}

// Source: src/core/SkCanvas.cpp:369-371
// Source: include/core/SkCanvas.h:309-309
impl<'device> SkCanvas<'device> {
    pub fn getBaseLayerSize(&self) -> crate::include::core::SkSize::SkISize {
        unsafe { self.rootDevice().as_ref() }
            .base()
            .image_info()
            .dimensions()
    }
}
// Source: src/core/SkCanvas.cpp:373-376
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2446-2446
    fn topDevice(
        &self,
    ) -> core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device> {
        unsafe { self.fMCRec.expect("native initialized MCRec").as_ref() }.fDevice
    }
}

// Rust-only self-borrow projection: never exposes mutable Canvas or stack outside
// Pin. Refresh inline-block provenance for each fresh exclusive Canvas borrow.
impl<'device> SkCanvas<'device> {
    unsafe fn rebind_stack(&mut self) {
        if self.initialized {
            let storage = core::ptr::addr_of_mut!(self.fMCRecStorage).cast::<u8>();
            let bytes = core::mem::size_of_val(&self.fMCRecStorage);
            let stack = unsafe { self.fMCStack.as_mut().unwrap() };
            unsafe {
                stack.rebind_initial_storage(storage, bytes);
            }
            self.fMCRec = core::ptr::NonNull::new(stack.back_mut().cast());
        }
    }
}

// Source: src/core/SkCanvas.cpp:378-421
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Destination pixmap must cover its declared writable layout and remain live
    /// and exclusively accessible through this read. Preserve Canvas's device lease.
    // Source: include/core/SkCanvas.h:470-470
    pub unsafe fn readPixels(
        self: core::pin::Pin<&mut Self>,
        pm: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        if pm.addr().is_null() {
            return Ok(false);
        }
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { this.rootDevice().as_mut().readPixels(pm, x, y) }
    }
    /// # Safety
    /// Same lease as readPixels; dst must cover dst_info/row_bytes writable storage.
    // Source: include/core/SkCanvas.h:432-433
    pub unsafe fn readPixels_raw(
        self: core::pin::Pin<&mut Self>,
        dst_info: &crate::src::core::SkImageInfo::SkImageInfo,
        dst: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        let pm = crate::src::core::SkPixmap::SkPixmap::new(dst_info, dst, row_bytes);
        unsafe { self.readPixels(&pm, x, y) }
    }
    /// # Safety
    /// Bitmap's actual pixel backing, including all aliases, must satisfy readPixels'
    /// exclusive writable destination lease. Metadata alone establishes no validity.
    // Source: include/core/SkCanvas.h:507-507
    pub unsafe fn readPixels_bitmap(
        self: core::pin::Pin<&mut Self>,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        let mut pm = crate::src::core::SkPixmap::SkPixmap::default();
        if bitmap.peekPixels(Some(&mut pm)) {
            unsafe { self.readPixels(&pm, x, y) }
        } else {
            Ok(false)
        }
    }
    /// # Safety
    /// Source bitmap pixels must be valid for reading and preserve Canvas's exclusive
    /// device lease and native copy overlap requirements for the duration of the call.
    // Source: include/core/SkCanvas.h:585-585
    pub unsafe fn writePixels_bitmap(
        canvas: core::ptr::NonNull<Self>,
        bitmap: &crate::src::core::SkBitmap::SkBitmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        let mut pm = crate::src::core::SkPixmap::SkPixmap::default();
        if bitmap.peekPixels(Some(&mut pm)) {
            unsafe { Self::writePixels_raw(canvas, pm.info(), pm.addr(), pm.rowBytes(), x, y) }
        } else {
            Ok(false)
        }
    }
    /// # Safety
    /// Source pixels must cover the live readable src_info/row_bytes layout, and
    /// satisfy native overlap rules. Preserve Canvas's exclusive root device lease.
    // Source: include/core/SkCanvas.h:545-545
    pub unsafe fn writePixels_raw(
        canvas: core::ptr::NonNull<Self>,
        src_info: &crate::src::core::SkImageInfo::SkImageInfo,
        pixels: *const u8,
        row_bytes: usize,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        let device = unsafe { Self::root_device_raw(canvas) };
        let mut src_rect =
            crate::src::core::SkRect::SkIRect::make_xywh(x, y, src_info.width(), src_info.height());
        let bounds = unsafe { device.as_ref() }.base().image_info().bounds();
        if !src_rect.intersect(&bounds) {
            return Ok(false);
        }
        let complete_overwrite =
            src_rect.size() == unsafe { device.as_ref() }.base().image_info().dimensions();
        if !unsafe { Self::predraw_notify_overwrite_raw(canvas, complete_overwrite) } {
            return Ok(false);
        }
        let pm = crate::src::core::SkPixmap::SkPixmap::new(src_info, pixels, row_bytes);
        unsafe {
            Self::root_device_raw(canvas)
                .as_mut()
                .writePixels(&pm, x, y)
        }
    }
}

// Source: src/core/SkCanvas.cpp:425-429
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2543-2543
    unsafe fn checkForDeferredSave(&mut self) {
        unsafe {
            self.rebind_stack();
        }
        if unsafe { self.fMCRec.unwrap().as_ref() }.fDeferredSaveCount > 0 {
            unsafe {
                self.doSave();
            }
        }
    }
}
// Source: src/core/SkCanvas.cpp:431-445
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:888-888
    pub fn getSaveCount(&self) -> i32 {
        #[cfg(sk_debug)]
        {
            let mut count = 0;
            let stack = unsafe { self.fMCStack.as_ref().unwrap() };
            let mut iter = crate::src::core::SkDeque::Iter::new(
                stack,
                crate::src::core::SkDeque::IterStart::kFront_IterStart,
            );
            loop {
                let p = iter.next();
                if p.is_null() {
                    break;
                }
                count += 1 + unsafe { &*p.cast::<MCRec<'device>>() }.fDeferredSaveCount;
            }
            crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 442; @condition b"count == fSaveCount\0".as_ptr(); (count) == (self.fSaveCount));
        }
        self.fSaveCount
    }
}
// Source: src/core/SkCanvas.cpp:447-451
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the constructor's exclusive device lease and the Canvas pin.
    // Source: include/core/SkCanvas.h:603-603
    pub unsafe fn save(self: core::pin::Pin<&mut Self>) -> i32 {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.rebind_stack();
        }
        this.fSaveCount += 1;
        unsafe { this.fMCRec.unwrap().as_mut() }.fDeferredSaveCount += 1;
        this.getSaveCount() - 1
    }
}
// Source: src/core/SkCanvas.cpp:453-459
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2542-2542
    unsafe fn doSave(&mut self) {
        unsafe {
            self.willSave();
            self.rebind_stack();
        }
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 456; @condition b"fMCRec->fDeferredSaveCount > 0\0".as_ptr(); unsafe{self.fMCRec.unwrap().as_ref()}.fDeferredSaveCount>0);
        unsafe { self.fMCRec.unwrap().as_mut() }.fDeferredSaveCount -= 1;
        unsafe {
            self.internalSave();
        }
    }
}
// Source: src/core/SkCanvas.cpp:461-476
impl<'device> SkCanvas<'device> {
    /// Safety: canvas is a live pinned complete Canvas allocation under the
    /// operation's exclusive raw lease. No Canvas/Surface/Device/pixel reference
    /// may survive a COW-capable callback. This entry never accepts an outer Pin borrow.
    // Source: include/core/SkCanvas.h:878-878
    pub unsafe fn restore_raw(canvas: core::ptr::NonNull<Self>) -> Result<(), crate::RasterError> {
        let needs_restore = {
            let this = unsafe { &mut *canvas.as_ptr() };
            unsafe {
                this.rebind_stack();
            }
            if unsafe { this.fMCRec.unwrap().as_ref() }.fDeferredSaveCount > 0 {
                crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 463; @condition b"fSaveCount > 1\0".as_ptr(); this.fSaveCount > 1);
                this.fSaveCount -= 1;
                unsafe { this.fMCRec.unwrap().as_mut() }.fDeferredSaveCount -= 1;
                false
            } else if unsafe { this.fMCStack.as_ref().unwrap() }.count() > 1 {
                unsafe {
                    this.willRestore();
                }
                crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 470; @condition b"fSaveCount > 1\0".as_ptr(); this.fSaveCount > 1);
                this.fSaveCount -= 1;
                true
            } else {
                false
            }
        };
        if needs_restore {
            let result = unsafe { Self::internalRestore_raw(canvas) };
            {
                let this = unsafe { &mut *canvas.as_ptr() };
                unsafe {
                    this.didRestore();
                    this.rebind_stack();
                }
            }
            result?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:478-488
impl<'device> SkCanvas<'device> {
    /// Safety: same bounded exclusive raw operation contract as restore_raw.
    // Source: include/core/SkCanvas.h:900-900
    pub unsafe fn restoreToCount_raw(
        canvas: core::ptr::NonNull<Self>,
        mut count: i32,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            (&mut *canvas.as_ptr()).rebind_stack();
        }
        if count < 1 {
            count = 1;
        }
        let n = unsafe { canvas.as_ref() }.getSaveCount() - count;
        for _ in 0..n {
            unsafe { Self::restore_raw(canvas) }?;
        }
        Ok(())
    }
    pub unsafe fn try_finish_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::restoreToCount_raw(canvas, 1) }
    }
}
// Source: src/core/SkCanvas.cpp:490-494
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2625-2625
    unsafe fn internalSave(&mut self) {
        unsafe {
            self.rebind_stack();
        }
        let previous = self.fMCRec.unwrap();
        let place = unsafe {
            self.fMCStack
                .as_mut()
                .unwrap()
                .push_back()
                .cast::<MCRec<'device>>()
        };
        unsafe {
            place.write(MCRec::from_previous(previous.as_ref()));
        }
        self.fMCRec = core::ptr::NonNull::new(place);
        unsafe { self.topDevice().as_mut() }.pushClipStack();
    }
}

// Source: src/core/SkCanvas.cpp:496-513
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas/device lease through all layer allocations and
    /// later restoration; descriptor objects remain borrowed during this call.
    // Source: include/core/SkCanvas.h:627-627
    pub unsafe fn saveLayer_bounds(
        self: core::pin::Pin<&mut Self>,
        bounds: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<i32, crate::RasterError> {
        unsafe { self.saveLayer(&SaveLayerRec::new(bounds, paint, 0)) }
    }
    /// # Safety
    /// Same pinned Canvas/device/filter lifetime lease as saveLayer_bounds().
    // Source: include/core/SkCanvas.h:867-867
    pub unsafe fn saveLayer(
        mut self: core::pin::Pin<&mut Self>,
        rec: &SaveLayerRec<'_>,
    ) -> Result<i32, crate::RasterError> {
        if rec.fPaint.is_some_and(|paint| self.nothingToDraw(paint)) {
            unsafe {
                self.as_mut().save();
                self.as_mut().clipRect(
                    &crate::src::core::SkRect::SkRect::MakeEmpty(),
                    crate::include::core::SkClipOp::SkClipOp::kIntersect,
                    false,
                );
            }
        } else {
            let this = unsafe { self.as_mut().get_unchecked_mut() };
            let strategy = this.getSaveLayerStrategy(rec);
            unsafe { this.begin_layer_record(rec, strategy, false) }?;
        }
        Ok(self.getSaveCount() - 1)
    }
}

// Source: src/core/SkCanvas.cpp:514-528
impl<'device> SkCanvas<'device> {
    /// Safety: preserve the pinned whole Canvas/device/pixel lease through snapshot
    /// and drawing; no external Canvas reference may cross a Surface callback.
    // Source: include/core/SkCanvas.h:2596-2596
    pub unsafe fn only_axis_aligned_saveBehind_raw(
        mut canvas: core::ptr::NonNull<Self>,
        bounds: Option<&crate::src::core::SkRect::SkRect>,
    ) -> Result<i32, crate::RasterError> {
        if bounds.is_some_and(|bounds| {
            !unsafe { canvas.as_ref() }
                .getLocalClipBounds()
                .intersects(bounds)
        }) {
            unsafe {
                core::pin::Pin::new_unchecked(canvas.as_mut()).save();
            }
        } else {
            let do_the_work = unsafe { canvas.as_ref() }.onDoSaveBehind(bounds);
            unsafe { canvas.as_mut() }.fSaveCount += 1;
            unsafe {
                canvas.as_mut().internalSave();
            }
            if do_the_work {
                unsafe {
                    Self::internalSaveBehind_raw(canvas, bounds)?;
                }
            }
        }
        Ok(unsafe { canvas.as_ref() }.getSaveCount() - 1)
    }
}

// Source: src/core/SkCanvas.cpp:532-550
fn compute_decomposition_center(
    dst_to_local: &crate::src::core::SkM44::SkM44,
    content_bounds: Option<
        crate::src::core::SkImageFilterTypes::ParameterSpace<crate::src::core::SkRect::SkRect>,
    >,
    target_output: &crate::src::core::SkImageFilterTypes::DeviceSpace<
        crate::src::core::SkRect::SkIRect,
    >,
) -> crate::src::core::SkImageFilterTypes::ParameterSpace<crate::src::core::SkPoint::SkPoint> {
    let rect = content_bounds.map_or_else(
        || crate::src::core::SkRect::SkRect::Make(target_output.geometry()),
        |r| *r.geometry(),
    );
    let mut center = crate::src::core::SkPoint::SkPoint::make(rect.centerX(), rect.centerY());
    if content_bounds.is_none() {
        let mapped = dst_to_local.map(center.fX, center.fY, 0.0, 1.0);
        center = crate::src::core::SkPoint::SkPoint::make(
            crate::src::core::SkFloatingPoint::sk_ieee_float_divide(mapped.x, mapped.w),
            crate::src::core::SkFloatingPoint::sk_ieee_float_divide(mapped.y, mapped.w),
        );
    }
    crate::src::core::SkImageFilterTypes::ParameterSpace::new(center)
}
// Source: src/core/SkCanvas.cpp:551-559
struct FilterToSpan {
    fFilter: Option<std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
}
impl FilterToSpan {
    fn new(
        filter: Option<&std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
    ) -> Self {
        Self {
            fFilter: filter.cloned(),
        }
    }
    fn as_span(&self) -> FilterSpan<'_> {
        if self.fFilter.is_some() {
            core::slice::from_ref(&self.fFilter)
        } else {
            &[]
        }
    }
}

// Source: src/core/SkCanvas.cpp:569-666
fn get_layer_mapping_and_bounds(
    filters: FilterSpan<'_>,
    local_to_dst: &crate::src::core::SkM44::SkM44,
    target_output: &crate::src::core::SkImageFilterTypes::DeviceSpace<
        crate::src::core::SkRect::SkIRect,
    >,
    content_bounds: Option<
        crate::src::core::SkImageFilterTypes::ParameterSpace<crate::src::core::SkRect::SkRect>,
    >,
    scale_factor: f32,
) -> Result<
    Option<(
        crate::src::core::SkImageFilterTypes::Mapping,
        crate::src::core::SkImageFilterTypes::LayerSpace<crate::src::core::SkRect::SkIRect>,
    )>,
    crate::RasterError,
> {
    use crate::src::core::SkImageFilterTypes::{LayerSpace, Mapping, MatrixCapability};
    use crate::src::core::SkM44::SkM44;
    use crate::src::core::SkRect::{SkIRect, SkRect};
    let mut dst_to_local = SkM44::default();
    if !local_to_dst.isFinite() || !local_to_dst.invert(&mut dst_to_local) {
        return Ok(None);
    }
    let center = compute_decomposition_center(&dst_to_local, content_bounds, target_output);
    let mut mapping = Mapping::default();
    let mut capability = MatrixCapability::kComplex;
    for filter in filters {
        if let Some(filter) = filter {
            let filter_capability = filter.getCTMCapability();
            if (filter_capability as i32) < (capability as i32) {
                capability = filter_capability;
            }
        }
    }
    if !mapping.decomposeCTM(local_to_dst, capability, &center) {
        return Ok(None);
    }
    if scale_factor != 1.0
        && !mapping.adjustLayerSpace(&SkM44::Scale(scale_factor, scale_factor, 1.0))
    {
        return Ok(None);
    }
    const kMinDimThreshold: i32 = 2048;
    // Inline the native saturating int64->int32 conversion at this callsite;
    // no source from the unrelated math header is claimed by this translation.
    let dimension = 2 * target_output
        .geometry()
        .width64()
        .max(target_output.geometry().height64());
    let mut max_layer_dim =
        (dimension.clamp(i32::MIN as i64, i32::MAX as i64) as i32).max(kMinDimThreshold);
    let mut base_layer_bounds = mapping.deviceToLayer(target_output);
    if let Some(content) = content_bounds {
        let known = mapping.paramToLayer(&content).roundOut();
        if !base_layer_bounds.intersect(&known) {
            base_layer_bounds = LayerSpace::<SkIRect>::Empty();
        }
    }
    let mut layer_bounds;
    if !filters.is_empty() {
        debug_assert!(filters.len() <= i32::MAX as usize);
        let bound_for = |filter: &Option<
            std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>,
        >|
         -> Result<LayerSpace<SkIRect>, crate::RasterError> {
            if let Some(filter) = filter {
                filter.getInputBounds(&mapping, target_output, content_bounds)
            } else {
                Ok(base_layer_bounds)
            }
        };
        layer_bounds = bound_for(&filters[0])?;
        for filter in &filters[1..] {
            layer_bounds.join(&bound_for(filter)?);
        }
        if layer_bounds.width() > max_layer_dim || layer_bounds.height() > max_layer_dim {
            let ideal_mapping = Mapping::new(*mapping.layerMatrix());
            for filter in filters {
                if let Some(filter) = filter {
                    let ideal_bounds =
                        filter.getInputBounds(&ideal_mapping, target_output, content_bounds)?;
                    max_layer_dim = ideal_bounds
                        .width()
                        .max(ideal_bounds.height())
                        .max(max_layer_dim);
                }
            }
        }
    } else {
        if base_layer_bounds.isEmpty() {
            return Ok(None);
        }
        layer_bounds = base_layer_bounds;
    }
    if layer_bounds.width() > max_layer_dim || layer_bounds.height() > max_layer_dim {
        let new_bounds = LayerSpace::new(SkIRect::make_wh(
            layer_bounds.width().min(max_layer_dim),
            layer_bounds.height().min(max_layer_dim),
        ));
        let adjust = SkM44::RectToRect(
            &SkRect::Make(layer_bounds.geometry()),
            &SkRect::Make(new_bounds.geometry()),
        );
        if !mapping.adjustLayerSpace(&adjust) {
            return Ok(None);
        } else {
            layer_bounds = new_bounds;
        }
    }
    Ok(Some((mapping, layer_bounds)))
}

// Source: src/core/SkCanvas.cpp:671-680
fn image_filter_color_type(
    dstInfo: &crate::src::core::SkImageInfo::SkColorInfo,
) -> crate::include::core::SkColorType::SkColorType {
    use crate::include::core::SkColorType::*;
    if dstInfo.bytesPerPixel() <= 4
        && dstInfo.colorType() != kRGBA_8888_SkColorType
        && dstInfo.colorType() != kBGRA_8888_SkColorType
    {
        kN32_SkColorType
    } else {
        dstInfo.colorType()
    }
}

// Source: src/core/SkCanvas.cpp:682-697
// Arc retained filters and Result propagation are Rust ownership/error adaptations.
unsafe fn apply_alpha_and_colorfilter(
    ctx: &crate::src::core::SkImageFilterTypes::Context<'_>,
    image: &crate::src::core::SkImageFilterTypes::FilterResult,
    paint: &crate::src::core::SkPaint::SkPaint,
) -> Result<crate::src::core::SkImageFilterTypes::FilterResult, crate::RasterError> {
    let mut result = image.clone();
    if paint.getAlphaf() < 1.0 {
        let filter = crate::src::effects::colorfilters::SkBlendModeColorFilter::Blend(
            &paint.getColor4f(),
            None,
            crate::src::core::SkBlendMode::SkBlendMode::kDstIn,
        );
        result = unsafe { result.applyColorFilter(ctx, filter) }?;
    }
    if paint.getColorFilter().is_some() {
        result = unsafe { result.applyColorFilter(ctx, paint.refColorFilter()) }?;
    }
    Ok(result)
}

// Source: include/core/SkCanvas.h:2653-2659
impl<'device> SkCanvas<'device> {
    /// Safety: all device/image/pixel leases are exclusive for this native draw.
    /// This entry is used while constructing a layer, before any Canvas COW
    /// callback. The raw entry below ends its Canvas projection before drawing.
    unsafe fn internalDrawDeviceWithFilter(
        &mut self,
        src: Option<core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device>>,
        dst: core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device>,
        filters: FilterSpan<'_>,
        paint: &crate::src::core::SkPaint::SkPaint,
        compat: DeviceCompatibleWithFilter,
        filter_color_info: &crate::src::core::SkImageInfo::SkColorInfo,
        scale_factor: f32,
        src_tile_mode: crate::include::core::SkTileMode::SkTileMode,
        src_is_coverage_layer: bool,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::internalDrawDeviceWithFilter_raw(
                core::ptr::NonNull::from(self),
                src,
                dst,
                filters,
                paint,
                compat,
                filter_color_info,
                scale_factor,
                src_tile_mode,
                src_is_coverage_layer,
            )
        }
    }
    /// Safety: Canvas and complete devices stay pinned/live. No whole Canvas,
    /// Device or pixel reference may overlap a downstream mutable device lease.
    // Source: src/core/SkCanvas.cpp:699-877
    unsafe fn internalDrawDeviceWithFilter_raw(
        canvas: core::ptr::NonNull<Self>,
        mut src: Option<
            core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device>,
        >,
        mut dst: core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device>,
        filters: FilterSpan<'_>,
        paint: &crate::src::core::SkPaint::SkPaint,
        compat: DeviceCompatibleWithFilter,
        filter_color_info: &crate::src::core::SkImageInfo::SkColorInfo,
        scale_factor: f32,
        src_tile_mode: crate::include::core::SkTileMode::SkTileMode,
        src_is_coverage_layer: bool,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkImageFilterTypes::{
            Context, DeviceSpace, FilterResult, LayerSpace, Mapping, Stats,
        };
        use crate::src::core::SkM44::SkM44;
        use crate::src::core::SkRect::SkIRect;
        let current_matrix = unsafe {
            (*canvas.as_ptr())
                .fMCRec
                .expect("native Canvas record")
                .as_ref()
        }
        .fMatrix
        .clone();
        let filter_color_space = filter_color_info.refColorSpace();
        let filter_color_type = if src_is_coverage_layer {
            crate::include::core::SkColorType::SkColorType::kAlpha_8_SkColorType
        } else {
            image_filter_color_type(filter_color_info)
        };
        let local_to_src = if let Some(src) = src {
            SkM44::concat(
                unsafe { src.as_ref() }.base().global_to_device(),
                &current_matrix,
            )
        } else {
            SkM44::default()
        };
        let src_dims = if let Some(src) = src {
            unsafe { src.as_ref() }.base().image_info().dimensions()
        } else {
            crate::include::core::SkSize::SkISize::Make(0, 0)
        };
        let mut src_to_layer = None;
        let output_bounds = DeviceSpace::new(unsafe { dst.as_ref() }.devClipBounds());
        let (mapping, mut required_input) = if compat != DeviceCompatibleWithFilter::kUnknown {
            crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 738; @condition b"src\0".as_ptr(); src.is_some());
            crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 739; @condition b"scaleFactor == 1.0f\0".as_ptr(); scale_factor==1.0);
            crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 740; @condition b"!srcDims.isEmpty()\0".as_ptr(); !src_dims.isEmpty());
            let src = unsafe { src.unwrap().as_ref() };
            let dst = unsafe { dst.as_ref() };
            let mapping = Mapping::from_decomposition(
                src.base().get_relative_transform(dst.base()),
                dst.base().get_relative_transform(src.base()),
                local_to_src,
            );
            src_to_layer = Some(LayerSpace::new(
                crate::src::core::SkMatrix::SkMatrix::default(),
            ));
            (mapping, LayerSpace::new(SkIRect::make_size(src_dims)))
        } else {
            let scale = if scale_factor < 0.0 {
                0.0
            } else if scale_factor > 1.0 {
                1.0
            } else {
                scale_factor
            };
            let Some((mapping, required)) = get_layer_mapping_and_bounds(
                filters,
                unsafe { dst.as_ref() }.base().local_to_device44(),
                &output_bounds,
                None,
                scale,
            )?
            else {
                return Ok(());
            };
            if src.is_some() && !required.isEmpty() {
                let mut src_to_local = SkM44::default();
                if !local_to_src.invert(&mut src_to_local) {
                    return Ok(());
                }
                src_to_layer = Some(LayerSpace::new(
                    SkM44::concat(mapping.layerMatrix(), &src_to_local).asM33(),
                ));
            }
            (mapping, required)
        };
        if compat == DeviceCompatibleWithFilter::kUnknown && src.is_none() {
            required_input = LayerSpace::<SkIRect>::Empty();
        }
        // Copy props before dispatch, so no whole source/destination reference
        // remains live while the actual destination constructs its backend.
        let props = *unsafe { src.unwrap_or(dst).as_ref() }
            .base()
            .surface_props();
        let backend =
            unsafe { dst.as_ref() }.createImageFilteringBackend(&props, filter_color_type)?;
        let stats = core::cell::Cell::new(Stats::default());
        let mut ctx = Context::new(
            backend,
            &mapping,
            required_input,
            &FilterResult::default(),
            filter_color_space,
            Some(&stats),
        );
        let mut source = FilterResult::default();
        if let Some(src) = src.as_mut().filter(|_| !required_input.isEmpty()) {
            let src_to_layer = src_to_layer.expect("native required source mapping initialized");
            let mut src_subset = LayerSpace::<SkIRect>::Empty();
            if !src_to_layer.inverseMapIRect(&required_input, &mut src_subset) {
                return Ok(());
            }
            ctx.markNewSurface();
            let dims = unsafe { src.as_ref() }.base().image_info().dimensions();
            let avail_src =
                LayerSpace::new(SkIRect::make_size(dims)).relevantSubset(src_subset, src_tile_mode);
            if src_to_layer.geometry().isScaleTranslate() {
                let required_subset = src_to_layer.mapIRect(&avail_src);
                if required_subset.width() == avail_src.width()
                    && required_subset.height() == avail_src.height()
                {
                    source = FilterResult::new(
                        unsafe { src.as_mut() }.snapSpecial(avail_src.geometry(), false)?,
                        required_subset.topLeft(),
                    );
                } else {
                    crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 809; @condition b"compat == DeviceCompatibleWithFilter::kUnknown\0".as_ptr(); compat==DeviceCompatibleWithFilter::kUnknown);
                    source = FilterResult::new(
                        unsafe { src.as_mut() }.snapSpecialScaled(
                            avail_src.geometry(),
                            required_subset.size().geometry(),
                        )?,
                        required_subset.topLeft(),
                    );
                    ctx.markNewSurface();
                }
            }
            if compat == DeviceCompatibleWithFilter::kYesWithPadding {
                crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 820; @condition b"!filters.empty()\0".as_ptr(); !filters.is_empty());
                source = source.insetForSaveLayer();
            } else if compat == DeviceCompatibleWithFilter::kYes {
                // Native source already prepared for the requested mapping.
            } else if source.has_image() {
                source = unsafe { source.applyCrop(&ctx, source.layerBounds(), src_tile_mode) }?;
            } else if !required_input.isEmpty() {
                source = FilterResult::new(
                    unsafe { src.as_mut() }.snapSpecial(avail_src.geometry(), false)?,
                    avail_src.topLeft(),
                );
                source = unsafe {
                    source.applyCrop(
                        &ctx.withNewDesiredOutput(src_subset),
                        source.layerBounds(),
                        src_tile_mode,
                    )
                }?;
                source = unsafe {
                    source.applyTransform(
                        &ctx,
                        src_to_layer,
                        crate::include::core::SkSamplingOptions::SkSamplingOptions::with_filter(
                            crate::include::core::SkSamplingOptions::SkFilterMode::kLinear,
                        ),
                    )
                }?;
            }
        }
        ctx = ctx
            .withNewDesiredOutput(mapping.deviceToLayer(&output_bounds))
            .withNewSource(&source);
        let null_filter = [None];
        let filters_or_null = if filters.is_empty() {
            &null_filter[..]
        } else {
            filters
        };
        for filter in filters_or_null {
            let mut result = if let Some(filter) = filter {
                unsafe { filter.filterImage(&ctx) }?
            } else {
                source.clone()
            };
            if src_is_coverage_layer {
                crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 858; @condition b"dst->useDrawCoverageMaskForMaskFilters()\0".as_ptr(); unsafe{dst.as_ref()}.useDrawCoverageMaskForMaskFilters());
                let (coverage_mask, origin) = unsafe { result.imageAndOffset(&ctx) }?;
                if let Some(coverage_mask) = coverage_mask {
                    let mut device_matrix_with_offset = mapping.layerToDeviceMatrix().clone();
                    device_matrix_with_offset.preTranslate(
                        origin.x() as f32,
                        origin.y() as f32,
                        0.0,
                    );
                    unsafe {
                        dst.as_mut().drawCoverageMask(
                            coverage_mask.as_ref(),
                            &device_matrix_with_offset.asM33(),
                            &result.sampling(),
                            paint,
                        )
                    }?;
                }
            } else {
                result = unsafe { apply_alpha_and_colorfilter(&ctx, &result, paint) }?;
                unsafe { result.draw(&ctx, dst, paint.getBlender()) }?;
            }
        }
        stats.get().reportStats();
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:878-1090
impl<'device> SkCanvas<'device> {
    // Safety: caller owns the entry save-count/record anchor and cancels newly
    // pushed records on Err. All device pixel/allocator leases remain exclusive.
    // Source: include/core/SkCanvas.h:2619-2619
    unsafe fn internalSaveLayer(
        &mut self,
        rec: &SaveLayerRec<'_>,
        strategy: SaveLayerStrategy,
        coverage_only: bool,
    ) -> Result<(), crate::RasterError> {
        use crate::include::core::SkRefCnt::sk_sp;
        use crate::src::core::SkDevice::{
            adopt_pinned_device, CreateInfo, SkDeviceVirtual, SkNoPixelsDevice,
        };
        use crate::src::core::SkImageFilterTypes::{DeviceSpace, LayerSpace, ParameterSpace};
        use crate::src::core::SkPaint::{SkPaint, Style};
        use crate::src::core::SkRect::{SkIRect, SkRect};
        unsafe {
            self.internalSave();
        }
        if self.isClipEmpty() {
            return Ok(());
        }
        let mut restore_paint = rec.fPaint.cloned().unwrap_or_default();
        restore_paint.setStyle(Style::kFill_Style);
        restore_paint.setPathEffect(crate::include::core::SkRefCnt::sk_sp::default());
        restore_paint.setMaskFilter(None);
        restore_paint.setImageFilter(None);
        restore_paint.setAntiAlias(true);
        let paint_filter = rec.fPaint.and_then(|p| p.refImageFilter());
        let paint_filter_span = FilterToSpan {
            fFilter: paint_filter,
        };
        let mut filters = if paint_filter_span.fFilter.is_some() {
            paint_filter_span.as_span()
        } else {
            rec.fFilters
        };
        if filters.len() > kMaxFiltersPerLayer as usize {
            filters = &filters[..kMaxFiltersPerLayer as usize];
        }
        let cf = restore_paint.getColorFilter();
        let blender = restore_paint.getBlender();
        let mut filters_prior_device = rec.fBackdrop.is_some();
        filters_prior_device |=
            rec.fSaveLayerFlags & SaveLayerFlagsSet::kInitWithPrevious_SaveLayerFlag as u32 != 0
                && (!filters.is_empty()
                    || cf.is_some()
                    || blender.is_some()
                    || restore_paint.getAlphaf() < 1.0);
        let draw_device_must_fill_clip = if filters.is_empty() {
            if let Some(cf) = cf {
                if cf.affectsTransparentBlack()? {
                    true
                } else {
                    blender.is_some_and(|b| b.affectsTransparentBlack())
                }
            } else {
                blender.is_some_and(|b| b.affectsTransparentBlack())
            }
        } else {
            false
        };
        let trivial_restore = !filters_prior_device && !draw_device_must_fill_clip;
        let mut prior_device = self.topDevice();
        let output_bounds = DeviceSpace::new(unsafe { prior_device.as_ref() }.devClipBounds());
        let content_bounds = if trivial_restore {
            rec.fBounds.map(|b| ParameterSpace::new(*b))
        } else {
            None
        };
        let mapping_and_bounds = get_layer_mapping_and_bounds(
            filters,
            unsafe { prior_device.as_ref() }.base().local_to_device44(),
            &output_bounds,
            content_bounds,
            1.0,
        )?;
        let Some((new_layer_mapping, mut layer_bounds)) = mapping_and_bounds else {
            unsafe {
                self.abort_layer();
            }
            return Ok(());
        };
        let mut padded_layer = false;
        if layer_bounds.isEmpty() {
            if !filters.is_empty() && !unsafe { prior_device.as_ref() }.isNoPixelsDevice() {
                let mut filter_color_info = unsafe { prior_device.as_ref() }
                    .base()
                    .image_info()
                    .colorInfo()
                    .clone();
                if let Some(cs) = rec.fColorSpace {
                    filter_color_info = filter_color_info.makeColorSpace(Some(cs.clone()));
                }
                unsafe {
                    self.internalDrawDeviceWithFilter(
                        None,
                        prior_device,
                        filters,
                        &restore_paint,
                        DeviceCompatibleWithFilter::kUnknown,
                        &filter_color_info,
                        1.0,
                        crate::include::core::SkTileMode::SkTileMode::kDecal,
                        false,
                    )
                }?;
            }
            unsafe {
                self.abort_layer();
            }
            return Ok(());
        } else if !filters.is_empty() {
            let mut padded_bounds = layer_bounds.clone();
            padded_bounds.outset(LayerSpace::new(
                crate::include::core::SkSize::SkISize::Make(1, 1),
            ));
            if padded_bounds.left() < layer_bounds.left()
                && padded_bounds.top() < layer_bounds.top()
                && padded_bounds.right() > layer_bounds.right()
                && padded_bounds.bottom() > layer_bounds.bottom()
            {
                layer_bounds = padded_bounds;
                padded_layer = true;
            }
        }
        let mut new_device: sk_sp<dyn SkDeviceVirtual + 'device> = sk_sp::default();
        if strategy == SaveLayerStrategy::kFullLayer_SaveLayerStrategy {
            crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1010; @condition b"!layerBounds.isEmpty()\0".as_ptr(); !layer_bounds.isEmpty());
            use crate::include::core::SkColorType::SkColorType;
            let layer_color_type = if coverage_only {
                SkColorType::kAlpha_8_SkColorType
            } else if rec.fSaveLayerFlags & SaveLayerFlagsSet::kF16ColorType as u32 != 0 {
                SkColorType::kRGBA_F16_SkColorType
            } else {
                image_filter_color_type(
                    unsafe { prior_device.as_ref() }
                        .base()
                        .image_info()
                        .colorInfo(),
                )
            };
            let info = crate::src::core::SkImageInfo::SkImageInfo::MakeWithColorSpace(
                layer_bounds.width(),
                layer_bounds.height(),
                layer_color_type,
                crate::include::core::SkAlphaType::SkAlphaType::kPremul_SkAlphaType,
                rec.fColorSpace.cloned().or_else(|| {
                    unsafe { prior_device.as_ref() }
                        .base()
                        .image_info()
                        .refColorSpace()
                }),
            );
            let geo = if rec.fSaveLayerFlags
                & SaveLayerFlagsSet::kPreserveLCDText_SaveLayerFlag as u32
                != 0
            {
                self.fProps.pixelGeometry()
            } else {
                crate::include::core::SkSurfaceProps::SkPixelGeometry::kUnknown_SkPixelGeometry
            };
            let allocator=self.fAllocator.as_mut().map(|owner| &mut **owner as &mut (dyn crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator+'_));
            let mut create_info = CreateInfo::new(&info, geo, allocator);
            new_device =
                unsafe { prior_device.as_mut() }.createDevice(&mut create_info, rec.fPaint);
        }
        let mut init_backdrop =
            rec.fSaveLayerFlags & SaveLayerFlagsSet::kInitWithPrevious_SaveLayerFlag as u32 != 0
                || rec.fBackdrop.is_some();
        if new_device.is_none() {
            new_device = unsafe {
                adopt_pinned_device(SkNoPixelsDevice::with_color_space(
                    &SkIRect::make_wh(layer_bounds.width(), layer_bounds.height()),
                    &self.fProps,
                    self.imageInfo().refColorSpace(),
                ))
            };
            init_backdrop = false;
        }
        let mut new_ptr = new_device.as_ptr().expect("native non-null layer device");
        if padded_layer {
            let bounds = unsafe { new_ptr.as_ref() }.devClipBounds().make_inset(1, 1);
            unsafe { new_ptr.as_mut() }.clipRect(
                &SkRect::Make(&bounds),
                crate::include::core::SkClipOp::SkClipOp::kIntersect,
                false,
            );
        }
        unsafe { new_ptr.as_mut().base_mut() }.set_device_coordinate_system(
            &crate::src::core::SkM44::SkM44::concat(
                unsafe { prior_device.as_ref() }.base().device_to_global(),
                new_layer_mapping.layerToDeviceMatrix(),
            ),
            &crate::src::core::SkM44::SkM44::concat(
                new_layer_mapping.deviceToLayerMatrix(),
                unsafe { prior_device.as_ref() }.base().global_to_device(),
            ),
            new_layer_mapping.layerMatrix(),
            layer_bounds.left(),
            layer_bounds.top(),
        );
        if init_backdrop {
            crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1067; @condition b"!coverageOnly\0".as_ptr(); !coverage_only);
            let backdrop_paint = SkPaint::default();
            let backdrop_as_span = FilterToSpan::new(rec.fBackdrop);
            let scale_backdrop = rec.fExperimentalBackdropScale != 1.0;
            let compat = if !filters.is_empty() || rec.fBackdrop.is_some() || scale_backdrop {
                DeviceCompatibleWithFilter::kUnknown
            } else {
                DeviceCompatibleWithFilter::kYes
            };
            // Separate owned ColorInfo keeps its canonical color-space reference
            // alive without borrowing the destination whole object during dispatch.
            let filter_color_info = unsafe { new_ptr.as_ref() }
                .base()
                .image_info()
                .colorInfo()
                .clone();
            unsafe {
                self.internalDrawDeviceWithFilter(
                    Some(prior_device),
                    new_ptr,
                    backdrop_as_span.as_span(),
                    &backdrop_paint,
                    compat,
                    &filter_color_info,
                    rec.fExperimentalBackdropScale,
                    rec.fBackdropTileMode,
                    false,
                )
            }?;
        }
        unsafe {
            self.rebind_stack();
            self.fMCRec.unwrap().as_mut()
        }
        .newLayer(
            new_device,
            filters,
            &restore_paint,
            coverage_only,
            padded_layer,
        );
        self.fQuickRejectBounds = self.computeDeviceClipBounds(true);
        Ok(())
    }
    // Native abortLayer lambda: guard updates quick bounds after the empty clip.
    unsafe fn abort_layer(&mut self) {
        let mut guard = unsafe { AutoUpdateQRBounds::new(self) };
        unsafe { guard.fCanvas.topDevice().as_mut() }.clipRect(
            &crate::src::core::SkRect::SkRect::MakeEmpty(),
            crate::include::core::SkClipOp::SkClipOp::kIntersect,
            false,
        );
    }
}

// Source: src/core/SkCanvas.cpp:1092-1101
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Same pinned Canvas/device/filter lifetime lease as saveLayer().
    // Source: include/core/SkCanvas.h:674-674
    pub unsafe fn saveLayerAlphaf(
        self: core::pin::Pin<&mut Self>,
        bounds: Option<&crate::src::core::SkRect::SkRect>,
        alpha: f32,
    ) -> Result<i32, crate::RasterError> {
        if alpha >= 1.0 {
            unsafe { self.saveLayer_bounds(bounds, None) }
        } else {
            let mut paint = crate::src::core::SkPaint::SkPaint::default();
            paint.setAlphaf(alpha);
            unsafe { self.saveLayer_bounds(bounds, Some(&paint)) }
        }
    }
}

// Source: src/core/SkCanvas.cpp:1102-1139
impl<'device> SkCanvas<'device> {
    /// Safety: the original pinned Canvas and exclusive device/pixel lease remain
    /// live through the snapshot, save record allocation, and fallible draw.
    // Source: include/core/SkCanvas.h:2620-2620
    unsafe fn internalSaveBehind_raw(
        mut canvas: core::ptr::NonNull<Self>,
        local_bounds: Option<&crate::src::core::SkRect::SkRect>,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkRect::{SkIRect, SkRect};
        let mut device = unsafe { Self::top_device_raw(canvas) };
        let mut dev_bounds = SkIRect::default();
        if let Some(local_bounds) = local_bounds {
            let mut tmp = SkRect::default();
            unsafe { device.as_ref() }
                .base()
                .local_to_device()
                .mapRect(&mut tmp, local_bounds);
            if !dev_bounds.intersect_ab(tmp.round(), unsafe { device.as_ref() }.devClipBounds()) {
                dev_bounds.set_empty();
            }
        } else {
            dev_bounds = unsafe { device.as_ref() }.devClipBounds();
        }
        if dev_bounds.is_empty() {
            return Ok(());
        }
        let Some(back_image) = unsafe { device.as_mut() }.snapSpecial(&dev_bounds, true)? else {
            return Ok(());
        };
        unsafe {
            canvas.as_mut().checkForDeferredSave();
            canvas.as_mut().rebind_stack();
        }
        let mut temporary = BackImage::new(
            back_image,
            crate::src::core::SkPoint::SkIPoint::make(dev_bounds.fLeft, dev_bounds.fTop),
        );
        let replacement = Some(Box::new(BackImage::move_from(&mut temporary)));
        let slot = unsafe { &mut canvas.as_mut().fMCRec.unwrap().as_mut().fBackImage };
        let old = core::mem::replace(slot, replacement);
        drop(old);
        let mut paint = crate::src::core::SkPaint::SkPaint::default();
        paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kClear);
        unsafe { Self::drawClippedToSaveBehind_raw(canvas, &paint) }
    }
}

// Source: src/core/SkCanvas.cpp:1141-1211
// Rust error adaptation splits native record teardown from fallible painting.
// Field destruction order is deliberate: original locals release backImage
// before layer. These are the actual detached native owners, not copies.
struct DetachedRestore<'device> {
    back_image: Option<Box<BackImage>>,
    layer: Option<core::pin::Pin<Box<Layer<'device>>>>,
    had_parent: bool,
}
impl<'device> SkCanvas<'device> {
    unsafe fn detach_restore_record(&mut self) -> DetachedRestore<'device> {
        unsafe {
            self.rebind_stack();
        }
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1142; @condition b"!fMCStack.empty()\0".as_ptr(); !unsafe{self.fMCStack.as_ref().unwrap()}.empty());
        let record = unsafe { self.fMCRec.unwrap().as_mut() };
        let layer = record.fLayer.take();
        let back_image = record.fBackImage.take();
        unsafe {
            core::ptr::drop_in_place(record);
            self.fMCStack.as_mut().unwrap().pop_back();
        }
        self.fMCRec = core::ptr::NonNull::new(
            unsafe { self.fMCStack.as_mut().unwrap() }
                .back_mut()
                .cast::<MCRec<'device>>(),
        );
        let had_parent = self.fMCRec.is_some();
        if had_parent {
            unsafe { self.topDevice().as_mut() }.popClipStack();
            let matrix = unsafe { self.fMCRec.unwrap().as_ref() }.fMatrix.clone();
            unsafe { self.topDevice().as_mut().base_mut() }.set_global_ctm(&matrix);
        }
        DetachedRestore {
            back_image,
            layer,
            had_parent,
        }
    }
    unsafe fn finish_restore_bookkeeping(&mut self) {
        if self.getSaveCount() < self.fClipRestrictionSaveCount {
            self.fClipRestrictionRect.set_empty();
            self.fClipRestrictionSaveCount = -1;
        }
        self.fQuickRejectBounds = self.computeDeviceClipBounds(true);
        self.validateClip();
    }
    unsafe fn draw_detached_restore_raw(
        canvas: core::ptr::NonNull<Self>,
        detached: &DetachedRestore<'device>,
    ) -> Result<(), crate::RasterError> {
        if let Some(back) = detached.back_image.as_ref() {
            let mut paint = crate::src::core::SkPaint::SkPaint::default();
            paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kDstOver);
            unsafe { Self::top_device_raw(canvas).as_mut() }.drawSpecial(
                unsafe { back.image() }.as_ref(),
                &crate::src::core::SkMatrix::SkMatrix::Translate(
                    back.fLoc.fX as f32,
                    back.fLoc.fY as f32,
                ),
                &crate::include::core::SkSamplingOptions::SkSamplingOptions::new(),
                &paint,
            )?;
        }
        if let Some(layer) = detached.layer.as_ref() {
            if !layer.fDevice.isNoPixelsDevice() && !layer.fDiscard {
                unsafe { layer.fDevice.as_ptr().unwrap().as_mut() }.setImmutable();
                if unsafe { Self::predraw_notify_overwrite_raw(canvas, false) } {
                    unsafe {
                        (&mut *canvas.as_ptr()).rebind_stack();
                    }
                    let mut dst = unsafe { Self::top_device_raw(canvas) };
                    if !layer.fImageFilters.empty() {
                        let compat = if layer.fIncludesPadding {
                            DeviceCompatibleWithFilter::kYesWithPadding
                        } else {
                            DeviceCompatibleWithFilter::kYes
                        };
                        // Rust ownership mapping of SkCanvas.cpp:1182-1190: release
                        // the source-device metadata borrow before snapSpecial takes
                        // a mutable source-device lease inside filter evaluation.
                        let source_color_info =
                            layer.fDevice.base().image_info().colorInfo().clone();
                        unsafe {
                            Self::internalDrawDeviceWithFilter_raw(
                                canvas,
                                Some(layer.fDevice.as_ptr().unwrap()),
                                dst,
                                layer.fImageFilters.as_slice(),
                                layer.fPaint.as_ref().unwrap(),
                                compat,
                                &source_color_info,
                                1.0,
                                crate::include::core::SkTileMode::SkTileMode::kDecal,
                                layer.fIsCoverage,
                            )
                        }?;
                    } else {
                        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1195; @condition b"!layer->fIsCoverage && !layer->fIncludesPadding\0".as_ptr(); !layer.fIsCoverage&&!layer.fIncludesPadding);
                        unsafe { dst.as_mut() }.drawDevice(
                            layer.fDevice.as_ptr().unwrap(),
                            &crate::include::core::SkSamplingOptions::SkSamplingOptions::new(),
                            layer.fPaint.as_ref().unwrap(),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
    // Source: include/core/SkCanvas.h:2626-2626
    unsafe fn internalRestore_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> Result<(), crate::RasterError> {
        let detached = unsafe { (&mut *canvas.as_ptr()).detach_restore_record() };
        if !detached.had_parent {
            return Ok(());
        }
        let result = unsafe { Self::draw_detached_restore_raw(canvas, &detached) };
        unsafe {
            (&mut *canvas.as_ptr()).finish_restore_bookkeeping();
        }
        drop(detached);
        result
    }
    // Cancellation deliberately omits all painting, including saved back-images.
    // It is a Rust error-boundary adaptation, never the normal successful outlet.
    unsafe fn cancel_internal_restore(&mut self) {
        let detached = unsafe { self.detach_restore_record() };
        if detached.had_parent {
            unsafe {
                self.finish_restore_bookkeeping();
            }
        }
        drop(detached);
    }
    unsafe fn cancel_restore_inner(&mut self) {
        unsafe {
            self.rebind_stack();
        }
        if unsafe { self.fMCRec.unwrap().as_ref() }.fDeferredSaveCount > 0 {
            debug_assert!(self.fSaveCount > 1);
            self.fSaveCount -= 1;
            unsafe { self.fMCRec.unwrap().as_mut() }.fDeferredSaveCount -= 1;
        } else if unsafe { self.fMCStack.as_ref().unwrap() }.count() > 1 {
            debug_assert!(self.fSaveCount > 1);
            self.fSaveCount -= 1;
            unsafe {
                self.cancel_internal_restore();
            }
        }
    }
}

impl SkCanvas<'_> {
    // Source: include/core/SkCanvas.h:323-323
    // Source: src/core/SkCanvas.cpp:1213-1218
    pub fn makeSurface(
        &self,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> crate::include::core::SkRefCnt::sk_sp<
        dyn crate::src::image::SkSurface::SkSurface<'static> + 'static,
    > {
        self.onNewSurface(info, props.unwrap_or(&self.fProps))
    }
    // Source: include/core/SkCanvas.h:2306-2306
    // Source: src/core/SkCanvas.cpp:1220-1222
    fn onNewSurface_inherited(
        &self,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> crate::include::core::SkRefCnt::sk_sp<
        dyn crate::src::image::SkSurface::SkSurface<'static> + 'static,
    > {
        unsafe { self.rootDevice().as_ref() }.makeSurface(info, props)
    }
}

// Source: src/core/SkCanvas.cpp:1224-1261
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:272-272
    pub fn imageInfo(&self) -> crate::src::core::SkImageInfo::SkImageInfo {
        self.onImageInfo()
    }
    // Source: include/core/SkCanvas.h:2311-2311
    fn onImageInfo_inherited(&self) -> crate::src::core::SkImageInfo::SkImageInfo {
        unsafe { self.rootDevice().as_ref() }
            .base()
            .image_info()
            .clone()
    }
    // Source: include/core/SkCanvas.h:284-284
    pub fn getProps(
        &self,
        props: Option<&mut crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> bool {
        self.onGetProps(props, false)
    }
    // Source: include/core/SkCanvas.h:291-291
    pub fn getBaseProps(&self) -> crate::include::core::SkSurfaceProps::SkSurfaceProps {
        let mut props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
        self.onGetProps(Some(&mut props), false);
        props
    }
    // Source: include/core/SkCanvas.h:299-299
    pub fn getTopProps(&self) -> crate::include::core::SkSurfaceProps::SkSurfaceProps {
        let mut props = crate::include::core::SkSurfaceProps::SkSurfaceProps::default();
        self.onGetProps(Some(&mut props), true);
        props
    }
    // Source: include/core/SkCanvas.h:2312-2312
    fn onGetProps_inherited(
        &self,
        props: Option<&mut crate::include::core::SkSurfaceProps::SkSurfaceProps>,
        top: bool,
    ) -> bool {
        if let Some(props) = props {
            *props = if top {
                *unsafe { self.topDevice().as_ref() }.base().surface_props()
            } else {
                self.fProps
            };
        }
        true
    }
    // Source: include/core/SkCanvas.h:396-396
    pub fn peekPixels(&self, pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>) -> bool {
        self.onPeekPixels(pmap)
    }
    // Source: include/core/SkCanvas.h:2309-2309
    fn onPeekPixels_inherited(
        &self,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        unsafe { self.rootDevice().as_ref() }.peekPixels(pmap)
    }
}
// Source: src/core/SkCanvas.cpp:1263-1292
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Returned raw storage remains borrowed from the top device. Preserve the
    /// whole-Canvas pixel lease, and end use before device replacement, restoration,
    /// destruction, or another overlapping pixel access. Raw pointer escape creates
    /// no extra ownership and cannot outlive the underlying bitmap release callback.
    // Source: include/core/SkCanvas.h:366-366
    pub unsafe fn accessTopLayerPixels(
        self: core::pin::Pin<&mut Self>,
        info: Option<&mut crate::src::core::SkImageInfo::SkImageInfo>,
        row_bytes: Option<&mut usize>,
        origin: Option<&mut crate::src::core::SkPoint::SkIPoint>,
    ) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        let mut pmap = crate::src::core::SkPixmap::SkPixmap::default();
        if !unsafe { this.onAccessTopLayerPixels(Some(&mut pmap)) } {
            return core::ptr::null_mut();
        }
        if let Some(info) = info {
            *info = pmap.info().clone();
        }
        if let Some(row_bytes) = row_bytes {
            *row_bytes = pmap.rowBytes();
        }
        if let Some(origin) = origin {
            let device = unsafe { this.topDevice().as_ref() }.base();
            if device.is_pixel_aligned_to_global() {
                *origin = device.get_origin();
            } else {
                return core::ptr::null_mut();
            }
        }
        pmap.writable_addr()
    }
    // Source: include/core/SkCanvas.h:2310-2310
    unsafe fn onAccessTopLayerPixels_inherited(
        &mut self,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        unsafe { self.topDevice().as_mut() }.accessPixels(pmap)
    }
}

// Source: src/core/SkCanvas.cpp:1296-1305
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and exclusive native device lease from construction.
    // Source: include/core/SkCanvas.h:915-915
    pub unsafe fn translate(self: core::pin::Pin<&mut Self>, dx: f32, dy: f32) {
        let this = unsafe { self.get_unchecked_mut() };
        if dx != 0.0 || dy != 0.0 {
            unsafe {
                this.checkForDeferredSave();
                this.rebind_stack();
            }
            unsafe { this.fMCRec.unwrap().as_mut() }
                .fMatrix
                .preTranslate(dx, dy, 0.0);
            let matrix = unsafe { this.fMCRec.unwrap().as_ref() }.fMatrix.clone();
            unsafe { this.topDevice().as_mut() }
                .base_mut()
                .set_global_ctm(&matrix);
            unsafe {
                this.didTranslate(dx, dy);
                this.rebind_stack();
            }
        }
    }
}
// Source: src/core/SkCanvas.cpp:1307-1316
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:930-930
    pub unsafe fn scale(self: core::pin::Pin<&mut Self>, sx: f32, sy: f32) {
        let this = unsafe { self.get_unchecked_mut() };
        if sx != 1.0 || sy != 1.0 {
            unsafe {
                this.checkForDeferredSave();
                this.rebind_stack();
            }
            unsafe { this.fMCRec.unwrap().as_mut() }
                .fMatrix
                .preScale(sx, sy);
            let matrix = unsafe { this.fMCRec.unwrap().as_ref() }.fMatrix.clone();
            unsafe { this.topDevice().as_mut() }
                .base_mut()
                .set_global_ctm(&matrix);
            unsafe {
                this.didScale(sx, sy);
                this.rebind_stack();
            }
        }
    }
}
// Source: src/core/SkCanvas.cpp:1318-1334
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:944-944
    pub unsafe fn rotate(self: core::pin::Pin<&mut Self>, degrees: f32) {
        let mut matrix = crate::src::core::SkMatrix::SkMatrix::default();
        matrix.setRotate(degrees);
        unsafe {
            self.concat(&matrix);
        }
    }
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:962-962
    pub unsafe fn rotatePivot(self: core::pin::Pin<&mut Self>, degrees: f32, px: f32, py: f32) {
        let mut matrix = crate::src::core::SkMatrix::SkMatrix::default();
        matrix.setRotatePivot(degrees, px, py);
        unsafe {
            self.concat(&matrix);
        }
    }
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:978-978
    pub unsafe fn skew(self: core::pin::Pin<&mut Self>, sx: f32, sy: f32) {
        let mut matrix = crate::src::core::SkMatrix::SkMatrix::default();
        matrix.setSkew(sx, sy);
        unsafe {
            self.concat(&matrix);
        }
    }
}
// Source: src/core/SkCanvas.cpp:1336-1341
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:989-989
    pub unsafe fn concat(
        self: core::pin::Pin<&mut Self>,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
    ) {
        if matrix.isIdentity() {
            return;
        }
        unsafe {
            self.concat44(&crate::src::core::SkM44::SkM44::from(matrix));
        }
    }
}
// Source: src/core/SkCanvas.cpp:1343-1349
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2622-2622
    unsafe fn internalConcat44(&mut self, m: &crate::src::core::SkM44::SkM44) {
        unsafe {
            self.checkForDeferredSave();
            self.rebind_stack();
        }
        unsafe { self.fMCRec.unwrap().as_mut() }
            .fMatrix
            .preConcat(m);
        let matrix = unsafe { self.fMCRec.unwrap().as_ref() }.fMatrix.clone();
        unsafe { self.topDevice().as_mut() }
            .base_mut()
            .set_global_ctm(&matrix);
    }
}
// Source: src/core/SkCanvas.cpp:1351-1355
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:990-990
    pub unsafe fn concat44(self: core::pin::Pin<&mut Self>, m: &crate::src::core::SkM44::SkM44) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.internalConcat44(m);
        }
        unsafe {
            this.didConcat44(m);
            this.rebind_stack();
        }
    }
}
// Source: src/core/SkCanvas.cpp:1357-1361
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2544-2544
    unsafe fn internalSetMatrix(&mut self, m: &crate::src::core::SkM44::SkM44) {
        unsafe {
            self.rebind_stack();
        }
        unsafe { self.fMCRec.unwrap().as_mut() }.fMatrix = m.clone();
        unsafe { self.topDevice().as_mut() }
            .base_mut()
            .set_global_ctm(m);
    }
}
// Source: src/core/SkCanvas.cpp:1363-1375
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:1002-1002
    pub unsafe fn setMatrix(
        self: core::pin::Pin<&mut Self>,
        m: &crate::src::core::SkMatrix::SkMatrix,
    ) {
        unsafe {
            self.setMatrix44(&crate::src::core::SkM44::SkM44::from(m));
        }
    }
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:999-999
    pub unsafe fn setMatrix44(self: core::pin::Pin<&mut Self>, m: &crate::src::core::SkM44::SkM44) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.checkForDeferredSave();
            this.internalSetMatrix(m);
        }
        unsafe {
            this.didSetM44(m);
            this.rebind_stack();
        }
    }
    /// # Safety
    /// Same pinned Canvas and exclusive native device lease as translate().
    // Source: include/core/SkCanvas.h:1009-1009
    pub unsafe fn resetMatrix(self: core::pin::Pin<&mut Self>) {
        unsafe {
            self.setMatrix44(&crate::src::core::SkM44::SkM44::default());
        }
    }
}
// Source: src/core/SkCanvas.cpp:1379-1394
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and exclusive native device lease from construction.
    // Source: include/core/SkCanvas.h:1021-1021
    pub unsafe fn clipRect(
        self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    ) {
        if !rect.isFinite() {
            return;
        }
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.checkForDeferredSave();
        }
        let edge = if aa {
            ClipEdgeStyle::kSoft_ClipEdgeStyle
        } else {
            ClipEdgeStyle::kHard_ClipEdgeStyle
        };
        unsafe {
            this.onClipRect(&rect.makeSorted(), op, edge);
        }
    }
    // Source: include/core/SkCanvas.h:2394-2394
    unsafe fn onClipRect_inherited(
        &mut self,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1389; @condition b"rect.isSorted()\0".as_ptr(); rect.isSorted());
        let aa = edge == ClipEdgeStyle::kSoft_ClipEdgeStyle;
        let aqr = unsafe { AutoUpdateQRBounds::new(self) };
        unsafe { aqr.fCanvas.topDevice().as_mut() }.clipRect(rect, op, aa);
    }
}

// Source: src/core/SkCanvas.cpp:1396-1435
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and exclusive device lease established at construction.
    // Source: include/core/SkCanvas.h:1062-1062
    pub unsafe fn androidFramework_setDeviceClipRestriction(
        self: core::pin::Pin<&mut Self>,
        rect: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        debug_assert!(core::ptr::addr_eq(
            this.topDevice().as_ptr(),
            this.rootDevice().as_ptr()
        ));
        debug_assert!(this.fClipRestrictionSaveCount < 0 && this.fClipRestrictionRect.is_empty());
        if this.fClipRestrictionSaveCount < 0 && !rect.is_empty() {
            this.fClipRestrictionRect = *rect;
            this.fClipRestrictionSaveCount = this.getSaveCount();
            unsafe {
                this.checkForDeferredSave();
            }
            let aqr = unsafe { AutoUpdateQRBounds::new(this) };
            unsafe { aqr.fCanvas.topDevice().as_mut() }.clipRegion(
                &crate::src::core::SkRegion::SkRegion::from_rect(*rect),
                crate::include::core::SkClipOp::SkClipOp::kIntersect,
            );
        }
    }
}
// Source: src/core/SkCanvas.cpp:1437-1455
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and exclusive device lease established at construction.
    // Source: include/core/SkCanvas.h:2680-2680
    pub unsafe fn internal_private_resetClip(self: core::pin::Pin<&mut Self>) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.checkForDeferredSave();
            this.onResetClip();
        }
    }
    // Source: include/core/SkCanvas.h:2399-2399
    unsafe fn onResetClip_inherited(&mut self) {
        let mut restriction = unsafe { self.topDevice().as_ref() }
            .base()
            .image_info()
            .bounds();
        if self.fClipRestrictionSaveCount >= 0
            && core::ptr::addr_eq(self.topDevice().as_ptr(), self.rootDevice().as_ptr())
        {
            if !restriction.intersect(&self.fClipRestrictionRect) {
                restriction = crate::src::core::SkRect::SkIRect::make_empty();
            }
        }
        let aqr = unsafe { AutoUpdateQRBounds::new(self) };
        unsafe { aqr.fCanvas.topDevice().as_mut() }.replaceClip(&restriction);
    }
}
// Source: src/core/SkCanvas.cpp:1457-1472
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and exclusive device lease established at construction.
    // Source: include/core/SkCanvas.h:1075-1075
    pub unsafe fn clipRRect(
        self: core::pin::Pin<&mut Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.checkForDeferredSave();
        }
        let edge = if aa {
            ClipEdgeStyle::kSoft_ClipEdgeStyle
        } else {
            ClipEdgeStyle::kHard_ClipEdgeStyle
        };
        if rrect.isRect() {
            unsafe {
                this.onClipRect(rrect.getBounds(), op, edge);
            }
        } else {
            unsafe {
                this.onClipRRect(rrect, op, edge);
            }
        }
    }
    // Source: include/core/SkCanvas.h:2395-2395
    unsafe fn onClipRRect_inherited(
        &mut self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        let aa = edge == ClipEdgeStyle::kSoft_ClipEdgeStyle;
        let aqr = unsafe { AutoUpdateQRBounds::new(self) };
        unsafe { aqr.fCanvas.topDevice().as_mut() }.clipRRect(rrect, op, aa);
    }
}
// Source: src/core/SkCanvas.cpp:1474-1504
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and exclusive device lease established at construction.
    // Source: include/core/SkCanvas.h:1111-1111
    pub unsafe fn clipPath(
        self: core::pin::Pin<&mut Self>,
        path: &crate::src::core::SkPath::SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
        aa: bool,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.checkForDeferredSave();
            this.rebind_stack();
        }
        let edge = if aa {
            ClipEdgeStyle::kSoft_ClipEdgeStyle
        } else {
            ClipEdgeStyle::kHard_ClipEdgeStyle
        };
        if !path.isInverseFillType()
            && unsafe { this.fMCRec.unwrap().as_ref() }
                .fMatrix
                .asM33()
                .rectStaysRect()
        {
            // Initialized output temporaries avoid forming references to native
            // uninitialized stack values; false queries leave them unobserved.
            let mut rect = crate::src::core::SkRect::SkRect::default();
            if path.isRect(Some(&mut rect), None, None) {
                unsafe {
                    this.onClipRect(&rect, op, edge);
                }
                return;
            }
            let mut rrect = crate::src::core::SkRRect::SkRRect::default();
            if path.isOval(Some(&mut rect)) {
                rrect.setOval(&rect);
                unsafe {
                    this.onClipRRect(&rrect, op, edge);
                }
                return;
            }
            if path.isRRect(Some(&mut rrect)) {
                unsafe {
                    this.onClipRRect(&rrect, op, edge);
                }
                return;
            }
        }
        unsafe {
            this.onClipPath(path, op, edge);
        }
    }
    // Source: include/core/SkCanvas.h:2396-2396
    unsafe fn onClipPath_inherited(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        let aa = edge == ClipEdgeStyle::kSoft_ClipEdgeStyle;
        let aqr = unsafe { AutoUpdateQRBounds::new(self) };
        unsafe { aqr.fCanvas.topDevice().as_mut() }.clipPath(path, op, aa);
    }
}
// Source: src/core/SkCanvas.cpp:1506-1526
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and exclusive device lease established at construction.
    // Source: include/core/SkCanvas.h:1142-1142
    pub unsafe fn clipShader(
        mut self: core::pin::Pin<&mut Self>,
        shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        if let Some(shader) = shader {
            if shader.isOpaque() {
                if op == crate::include::core::SkClipOp::SkClipOp::kIntersect {
                } else {
                    crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1512; @condition b"op == SkClipOp::kDifference\0".as_ptr(); (op) == (crate::include::core::SkClipOp::SkClipOp::kDifference));
                    unsafe {
                        self.as_mut().clipRect(
                            &crate::src::core::SkRect::SkRect::MakeEmpty(),
                            crate::include::core::SkClipOp::SkClipOp::kIntersect,
                            false,
                        );
                    }
                }
            } else {
                let this = unsafe { self.get_unchecked_mut() };
                unsafe {
                    this.checkForDeferredSave();
                    this.onClipShader(shader, op);
                }
            }
        }
    }
    // Source: include/core/SkCanvas.h:2397-2397
    unsafe fn onClipShader_inherited(
        &mut self,
        shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        let aqr = unsafe { AutoUpdateQRBounds::new(self) };
        // Native Device wrapper applies localToDevice and optional alpha inversion
        // before dispatching its onClipShader; never bypass that wrapper here.
        unsafe { aqr.fCanvas.topDevice().as_mut() }.clipShader(shader, op);
    }
}
// Source: src/core/SkCanvas.cpp:1528-1536
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and exclusive device lease established at construction.
    // Source: include/core/SkCanvas.h:1153-1153
    pub unsafe fn clipRegion(
        self: core::pin::Pin<&mut Self>,
        region: &crate::src::core::SkRegion::SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.checkForDeferredSave();
            this.onClipRegion(region, op);
        }
    }
    // Source: include/core/SkCanvas.h:2398-2398
    unsafe fn onClipRegion_inherited(
        &mut self,
        region: &crate::src::core::SkRegion::SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        let aqr = unsafe { AutoUpdateQRBounds::new(self) };
        unsafe { aqr.fCanvas.topDevice().as_mut() }.clipRegion(region, op);
    }
}

// Source: src/core/SkCanvas.cpp:1538-1547
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2704-2704
    fn validateClip(&self) {
        #[cfg(sk_debug)]
        {
            let tmp = self.computeDeviceClipBounds(true);
            if self.isClipEmpty() {
                crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1542; @condition b"fQuickRejectBounds.isEmpty()\0".as_ptr(); self.fQuickRejectBounds.isEmpty());
            } else {
                crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1544; @condition b"tmp == fQuickRejectBounds\0".as_ptr(); (tmp) == (self.fQuickRejectBounds));
            }
        }
    }
}
// Source: src/core/SkCanvas.cpp:1549-1551
// Source: include/core/SkCanvas.h:2673-2673
impl<'device> SkCanvas<'device> {
    pub fn androidFramework_isClipAA(&self) -> bool {
        unsafe { self.topDevice().as_ref() }.isClipAntiAliased()
    }
}
// Source: src/core/SkCanvas.cpp:1553-1563
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2299-2299
    pub fn temporary_internal_getRgnClip(&self, rgn: &mut crate::src::core::SkRegion::SkRegion) {
        rgn.set_empty();
        let device = unsafe { self.topDevice().as_ref() };
        if device.base().is_pixel_aligned_to_global() {
            device.android_utils_clipAsRgn(rgn);
            let origin = device.base().get_origin();
            if origin.fX | origin.fY != 0 {
                rgn.translate(origin.fX, origin.fY);
            }
        }
    }
}
// Source: src/core/SkCanvas.cpp:1567-1573
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2255-2255
    pub fn isClipEmpty(&self) -> bool {
        unsafe { self.topDevice().as_ref() }.isClipEmpty()
    }
    // Source: include/core/SkCanvas.h:2264-2264
    pub fn isClipRect(&self) -> bool {
        unsafe { self.topDevice().as_ref() }.isClipRect()
    }
}
// Source: src/core/SkCanvas.cpp:1575-1587
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1165-1165
    pub fn quickReject(&self, src: &crate::src::core::SkRect::SkRect) -> bool {
        #[cfg(sk_debug)]
        self.validateClip();
        let dev = crate::src::core::SkMatrixPriv::SkMatrixPriv::MapRect(
            &unsafe { self.fMCRec.unwrap().as_ref() }.fMatrix,
            src,
        );
        !dev.isFinite() || !dev.intersects(&self.fQuickRejectBounds)
    }
    // Source: include/core/SkCanvas.h:1177-1177
    pub fn quickRejectPath(&self, path: &crate::src::core::SkPath::SkPath) -> bool {
        path.isEmpty() || self.quickReject(path.getBounds())
    }
}
// Source: src/core/SkCanvas.cpp:1589-1605
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2611-2611
    fn nothingToDraw(&self, paint: &crate::src::core::SkPaint::SkPaint) -> bool {
        !unsafe { self.topDevice().as_ref() }
            .base()
            .surface_props()
            .preservesTransparentDraws()
            && paint.nothingToDraw()
    }
    // Source: include/core/SkCanvas.h:2615-2616
    fn internalQuickReject(
        &self,
        bounds: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Result<bool, crate::RasterError> {
        if !bounds.isFinite() || self.nothingToDraw(paint) {
            return Ok(true);
        }
        if paint.canComputeFastBounds()? {
            let input = if let Some(m) = matrix {
                let mut r = crate::src::core::SkRect::SkRect::default();
                m.mapRect(&mut r, bounds);
                r
            } else {
                *bounds
            };
            // Separate initialized storage avoids an aliasing &input / &mutinput pair.
            let mut tmp = input;
            return Ok(self.quickReject(paint.computeFastBounds(&input, &mut tmp)?));
        }
        Ok(false)
    }
}
// Source: src/core/SkCanvas.cpp:1608-1628
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1189-1189
    pub fn getLocalClipBounds(&self) -> crate::src::core::SkRect::SkRect {
        let bounds = self.getDeviceClipBounds();
        if bounds.is_empty() {
            return crate::src::core::SkRect::SkRect::MakeEmpty();
        }
        let Some(inverse) = unsafe { self.fMCRec.unwrap().as_ref() }
            .fMatrix
            .asM33()
            .invert()
        else {
            return crate::src::core::SkRect::SkRect::MakeEmpty();
        };
        let margin = 1;
        let input = crate::src::core::SkRect::SkRect::Make(&bounds.make_outset(margin, margin));
        let mut result = crate::src::core::SkRect::SkRect::default();
        inverse.mapRect(&mut result, &input);
        result
    }
    // Source: include/core/SkCanvas.h:1214-1214
    pub fn getDeviceClipBounds(&self) -> crate::src::core::SkRect::SkIRect {
        self.computeDeviceClipBounds(false).roundOut()
    }
}
// Source: src/core/SkCanvas.cpp:1630-1644
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2690-2690
    fn computeDeviceClipBounds(&self, outset_aa: bool) -> crate::src::core::SkRect::SkRect {
        let dev = unsafe { self.topDevice().as_ref() };
        if dev.isClipEmpty() {
            crate::src::core::SkRect::SkRect::MakeEmpty()
        } else {
            let mut bounds = crate::src::core::SkMatrixPriv::SkMatrixPriv::MapRect(
                dev.base().device_to_global(),
                &crate::src::core::SkRect::SkRect::Make(&dev.devClipBounds()),
            );
            if outset_aa {
                bounds.outset(1.0, 1.0);
            }
            bounds
        }
    }
}
// Source: src/core/SkCanvas.cpp:1648-1654
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2290-2290
    pub fn getTotalMatrix(&self) -> crate::src::core::SkMatrix::SkMatrix {
        unsafe { self.fMCRec.unwrap().as_ref() }.fMatrix.asM33()
    }
    // Source: include/core/SkCanvas.h:2271-2271
    pub fn getLocalToDevice(&self) -> crate::src::core::SkM44::SkM44 {
        unsafe { self.fMCRec.unwrap().as_ref() }.fMatrix.clone()
    }
}

// Source: src/core/SkCanvas.cpp:1664-1666
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:343-343
    pub fn baseRecorder(&self) -> Option<&dyn crate::include::core::SkRecorder::SkRecorder> {
        unsafe { self.topDevice().as_ref() }.baseRecorder()
    }
}

// Source: src/core/SkCanvas.cpp:1668-1689
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1475-1475
    pub unsafe fn drawDRRect_raw(
        canvas: core::ptr::NonNull<Self>,
        outer: &crate::src::core::SkRRect::SkRRect,
        inner: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if outer.isEmpty() {
            return Ok(());
        }
        if inner.isEmpty() {
            unsafe {
                Self::drawRRect_raw(canvas, outer, paint)?;
            }
            return Ok(());
        }
        if !outer.getBounds().contains(inner.getBounds()) {
            return Ok(());
        }
        unsafe {
            Self::onDrawDRRect_raw(canvas, outer, inner, paint)?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:1691-1694
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and whole-device pixel lease through draw and any
    /// filter-layer restores; native chooser/filter arena contracts remain required.
    // Source: include/core/SkCanvas.h:1289-1289
    pub unsafe fn drawPaint_raw(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::onDrawPaint_raw(canvas, paint)?;
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:1696-1738
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1403-1403
    pub unsafe fn drawRect_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::onDrawRect_raw(canvas, &rect.makeSorted(), paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:2602-2602
    pub unsafe fn drawClippedToSaveBehind_raw(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::onDrawBehind_raw(canvas, paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1429-1429
    pub unsafe fn drawRegion_raw(
        canvas: core::ptr::NonNull<Self>,
        region: &crate::src::core::SkRegion::SkRegion,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if region.is_empty() {
            return Ok(());
        }
        if region.is_rect() {
            unsafe {
                Self::drawIRect_raw(canvas, region.get_bounds(), paint)?;
            }
            return Ok(());
        }
        unsafe {
            Self::onDrawRegion_raw(canvas, region, paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1440-1440
    pub unsafe fn drawOval_raw(
        canvas: core::ptr::NonNull<Self>,
        oval: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::onDrawOval_raw(canvas, &oval.makeSorted(), paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1454-1454
    pub unsafe fn drawRRect_raw(
        canvas: core::ptr::NonNull<Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::onDrawRRect_raw(canvas, rrect, paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1332-1332
    pub unsafe fn drawPoints_raw(
        canvas: core::ptr::NonNull<Self>,
        mode: PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if !points.is_empty() {
            unsafe {
                Self::onDrawPoints_raw(canvas, mode, points, paint)?;
            }
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:1740-1743
// Source: include/core/SkCanvas.h:2104-2104
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2101-2101
    pub unsafe fn drawVertices_sp_raw(
        canvas: core::ptr::NonNull<Self>,
        vertices: &crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkVertices::SkVertices>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::drawVertices_raw(canvas, vertices.get(), mode, paint) }
    }
}
// Source: src/core/SkCanvas.cpp:1745-1762
// Source: include/core/SkCanvas.h:2081-2081
impl<'device> SkCanvas<'device> {
    pub unsafe fn drawVertices_raw(
        canvas: core::ptr::NonNull<Self>,
        vertices: Option<&crate::src::core::SkVertices::SkVertices>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let Some(vertices) = vertices else {
            return Ok(());
        };
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 1750; @condition b"vertices->priv().mode() != SkVertices::kTriangleFan_VertexMode\0".as_ptr(); (vertices.priv_().mode()) != (crate::src::core::SkVertices::VertexMode::kTriangleFan_VertexMode));
        #[cfg(sk_build_for_android_framework)]
        if paint.getShader().is_some() && !vertices.priv_().hasTexCoords() {
            let mut no_shader_paint = paint.clone();
            no_shader_paint.setShader(None);
            unsafe { Self::onDrawVerticesObject_raw(canvas, vertices, mode, &no_shader_paint) }?;
            return Ok(());
        }
        unsafe { Self::onDrawVerticesObject_raw(canvas, vertices, mode, paint) }
    }
}

// Source: src/core/SkCanvas.cpp:1764-1770
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2123-2123
    pub unsafe fn drawMesh_raw(
        canvas: core::ptr::NonNull<Self>,
        mesh: &crate::src::core::SkMesh::SkMesh,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawMesh");
        let blender = if blender.is_none() {
            <dyn crate::include::core::SkBlender::SkBlender>::Mode(
                crate::src::core::SkBlendMode::SkBlendMode::kModulate,
            )
        } else {
            blender
        };
        unsafe { Self::onDrawMesh_raw(canvas, mesh, blender, paint) }
    }
}

// Source: src/core/SkCanvas.cpp:1772-1775
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1581-1581
    pub unsafe fn drawPath_raw(
        canvas: core::ptr::NonNull<Self>,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::onDrawPath_raw(canvas, path, paint)?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:1778-1782
fn fillable(r: &crate::src::core::SkRect::SkRect) -> bool {
    let w = r.width();
    let h = r.height();
    crate::src::core::SkFloatingPoint::SkIsFinite(&[w, h]) && w > 0.0 && h > 0.0
}

// Source: src/core/SkCanvas.cpp:1784-1793
// Canonical Paint clone preserves native shared-effect ownership.
fn clean_paint_for_lattice(
    paint: Option<&crate::src::core::SkPaint::SkPaint>,
) -> crate::src::core::SkPaint::SkPaint {
    let mut cleaned = crate::src::core::SkPaint::SkPaint::default();
    if let Some(paint) = paint {
        cleaned = paint.clone();
        cleaned.setMaskFilter(None);
        cleaned.setAntiAlias(false);
    }
    cleaned
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2229-2229
    // Source: src/core/SkCanvas.cpp:1852-1857
    /// Safety: preserve the original Canvas/Surface operation lease across COW.
    pub unsafe fn drawAnnotation_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        key: Option<&std::ffi::CStr>,
        value: Option<&std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawAnnotation");
        if let Some(key) = key {
            unsafe {
                Self::onDrawAnnotation_raw(canvas, rect, key, value);
            }
        }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1644-1645
    // Source: src/core/SkCanvas.cpp:1794-1809
    /// Safety: actual image and raw lattice scratch backing remain live through
    /// synchronous native callbacks and the complete Canvas pixel/COW operation.
    pub unsafe fn drawImageNine_raw(
        canvas: core::ptr::NonNull<Self>,
        image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        center: &crate::src::core::SkRect::SkIRect,
        dst: &crate::src::core::SkRect::SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let Some(image) = image else {
            return Ok(());
        };
        let x = [center.fLeft, center.fRight];
        let y = [center.fTop, center.fBottom];
        let lattice = Lattice {
            fXDivs: x.as_ptr(),
            fYDivs: y.as_ptr(),
            fRectTypes: core::ptr::null(),
            fXCount: 2,
            fYCount: 2,
            fBounds: core::ptr::null(),
            fColors: core::ptr::null(),
        };
        unsafe { Self::drawImageLattice_raw(canvas, Some(image), &lattice, dst, filter, paint) }
    }
    // Source: include/core/SkCanvas.h:1704-1705
    // Source: src/core/SkCanvas.cpp:1811-1833
    pub unsafe fn drawImageLattice_raw(
        canvas: core::ptr::NonNull<Self>,
        image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        lattice: &Lattice,
        dst: &crate::src::core::SkRect::SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawImageLattice");
        let Some(image) = image else {
            return Ok(());
        };
        if dst.isEmpty() {
            return Ok(());
        }
        let bounds = crate::src::core::SkRect::SkIRect::make_wh(image.width(), image.height());
        let mut with_bounds = Lattice {
            fXDivs: lattice.fXDivs,
            fYDivs: lattice.fYDivs,
            fRectTypes: lattice.fRectTypes,
            fXCount: lattice.fXCount,
            fYCount: lattice.fYCount,
            fBounds: lattice.fBounds,
            fColors: lattice.fColors,
        };
        if with_bounds.fBounds.is_null() {
            with_bounds.fBounds = &bounds;
        }
        let paint = clean_paint_for_lattice(paint);
        // Validate with the original lattice owner; invalid grids take the
        // native strict drawImageRect branch below.
        if unsafe {
            crate::src::core::SkLatticeIter::SkLatticeIter::Valid(
                image.width(),
                image.height(),
                &with_bounds,
            )
        } {
            unsafe {
                Self::onDrawImageLattice2_raw(
                    canvas,
                    image,
                    &with_bounds,
                    dst,
                    filter,
                    Some(&paint),
                )
            }
        } else {
            unsafe {
                Self::drawImageRect_raw(
                    canvas,
                    image,
                    Some(&crate::src::core::SkRect::SkRect::MakeIWH(
                        image.width(),
                        image.height(),
                    )),
                    dst,
                    &crate::include::core::SkSamplingOptions::SkSamplingOptions::with_filter(
                        filter,
                    ),
                    Some(&paint),
                    SrcRectConstraint::kStrict_SrcRectConstraint,
                )
            }
        }
    }
}

// Source: include/core/SkCanvas.h:1706-1708
impl<'device> SkCanvas<'device> {
    /// Same original pinned Canvas, image read and Surface COW leases as the
    /// full drawImageLattice_raw entry. The default is native nearest sampling.
    pub unsafe fn drawImageLattice_default_raw(
        canvas: core::ptr::NonNull<Self>,
        image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        lattice: &Lattice,
        dst: &crate::src::core::SkRect::SkRect,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::drawImageLattice_raw(
                canvas,
                image,
                lattice,
                dst,
                crate::include::core::SkSamplingOptions::SkFilterMode::kNearest,
                None,
            )
        }
    }
}

// Source: src/core/SkCanvas.cpp:1835-1850
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2183-2185
    pub unsafe fn drawAtlas_raw(
        canvas: core::ptr::NonNull<Self>,
        atlas: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[crate::src::core::SkRect::SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        cull: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawAtlas");
        let Some(atlas) = atlas else {
            return Ok(());
        };
        let mut count = xform.len().min(tex.len());
        if !colors.is_empty() {
            count = count.min(colors.len());
        }
        if count == 0 {
            return Ok(());
        }
        unsafe {
            Self::onDrawAtlas2_raw(
                canvas,
                atlas,
                &xform[..count],
                &tex[..count],
                if colors.is_empty() {
                    colors
                } else {
                    &colors[..count]
                },
                mode,
                sampling,
                cull,
                paint,
            )
        }
    }
}

// Source: src/core/SkCanvas.cpp:1859-1871
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2301-2301
    /// Safety: actual pinned Canvas exclusive operation lease; no Canvas,
    /// Surface, Device or pixel projection survives a COW-capable callback.
    pub unsafe fn private_draw_shadow_rec_raw(
        canvas: core::ptr::NonNull<Self>,
        path: &crate::src::core::SkPath::SkPath,
        rec: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::private_draw_shadow_rec");
        unsafe { Self::onDrawShadowRec_raw(canvas, path, rec) }
    }
    // Source: include/core/SkCanvas.h:2380-2380
    pub unsafe fn onDrawShadowRec_base_raw(
        canvas: core::ptr::NonNull<Self>,
        path: &crate::src::core::SkPath::SkPath,
        rec: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError> {
        if !unsafe { Self::predraw_notify_overwrite_raw(canvas, false) } {
            return Ok(());
        }
        let mut device = unsafe { Self::top_device_raw(canvas) };
        unsafe { device.as_mut().drawShadow(canvas, path, rec) }
    }
    pub unsafe fn onDrawShadowRec_raw(
        canvas: core::ptr::NonNull<Self>,
        path: &crate::src::core::SkPath::SkPath,
        rec: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawShadowRec(hooks.as_ref(),canvas,path,rec)
            };
        }
        unsafe { Self::onDrawShadowRec_base_raw(canvas, path, rec) }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1764-1765
    // Source: src/core/SkCanvas.cpp:1873-1879
    pub unsafe fn experimental_DrawEdgeAAQuad_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        flags: QuadAAFlags,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::experimental_DrawEdgeAAQuad");
        unsafe { Self::onDrawEdgeAAQuad_raw(canvas, &rect.makeSorted(), clip, flags, color, mode) }
    }
    // Source: include/core/SkCanvas.h:1766-1769
    /// Safety: same original pinned Canvas/device/pixel lease as the color4f entry.
    pub unsafe fn experimental_DrawEdgeAAQuad_color_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        flags: QuadAAFlags,
        color: crate::src::core::SkColor::SkColor,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::experimental_DrawEdgeAAQuad_raw(
                canvas,
                rect,
                clip,
                flags,
                &crate::src::core::SkColor::SkColor4f::FromColor(color),
                mode,
            )
        }
    }

    // Source: include/core/SkCanvas.h:1799-1803
    // Source: src/core/SkCanvas.cpp:1881-1910
    /// Every entry has both original rectangle fields initialized; image, clip
    /// and matrix arrays satisfy the same complete native drawing contracts.
    pub unsafe fn experimental_DrawEdgeAAImageSet_raw(
        canvas: core::ptr::NonNull<Self>,
        images: &[ImageSetEntry],
        clips: &[crate::src::core::SkPoint::SkPoint],
        matrices: &[crate::src::core::SkMatrix::SkMatrix],
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::experimental_DrawEdgeAAImageSet");
        if paint.is_some_and(|p| p.getImageFilter().is_some() || p.getMaskFilter().is_some())
            && images.len() == 1
        {
            let entry = &images[0];
            let can_map = entry.fMatrixIndex < 0 || {
                let matrix = &matrices[entry.fMatrixIndex as usize];
                matrix.isScaleTranslate() && matrix.getScaleX() > 0.0 && matrix.getScaleY() > 0.0
            };
            if !entry.fHasClip && can_map {
                let dst = if entry.fMatrixIndex >= 0 {
                    matrices[entry.fMatrixIndex as usize]
                        .mapRect_value(unsafe { entry.fDstRect.assume_init_ref() })
                } else {
                    unsafe { entry.fDstRect.assume_init() }
                };
                unsafe {
                    Self::drawImageRect_shared_raw(
                        canvas,
                        entry.fImage.as_ref(),
                        unsafe { entry.fSrcRect.assume_init_ref() },
                        &dst,
                        sampling,
                        paint,
                        constraint,
                    )
                }?;
                return Ok(());
            }
        }
        unsafe {
            Self::onDrawEdgeAAImageSet2_raw(
                canvas, images, clips, matrices, sampling, paint, constraint,
            )
        }
    }
}

// Source: src/core/SkCanvas.cpp:1916-1920
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2401-2401
    pub unsafe fn onDiscard_base_raw(canvas: core::ptr::NonNull<Self>) {
        let surface = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fSurfaceBase).read() };
        if let Some(surface) = surface {
            // Native ignores this boolean only; this actual COW API has no
            // RasterError, so no rendering error is lost.
            let _ = unsafe {
                crate::src::image::SkSurface_Base::SkSurface_Base::aboutToDraw_raw(
                    surface,
                    crate::src::image::SkSurface::ContentChangeMode::kDiscard_ContentChangeMode,
                )
            };
        }
    }
}

// Source: src/core/SkCanvas.cpp:1922-1937
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2341-2341
    pub(crate) unsafe fn onDrawPaint_base_raw(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::internalDrawPaint_raw(canvas, paint) }
    }
    // Source: include/core/SkCanvas.h:2618-2618
    unsafe fn internalDrawPaint_raw(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if unsafe { canvas.as_ref() }.nothingToDraw(paint)
            || unsafe { canvas.as_ref() }.isClipEmpty()
        {
            return Ok(());
        }
        if let Some(mut layer) =
            unsafe { Self::aboutToDraw_raw(canvas, paint, None, PredrawFlags::kCheckForOverwrite) }?
        {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawPaint(paint)?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:1939-1977
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2359-2360
    pub(crate) unsafe fn onDrawPoints_base_raw(
        canvas: core::ptr::NonNull<Self>,
        mode: PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        // Native cast to signed long precedes the empty count test.
        if points.len() as isize <= 0 || unsafe { canvas.as_ref() }.nothingToDraw(paint) {
            return Ok(());
        }
        let mut stroke_paint = paint.clone();
        stroke_paint.setStyle(crate::src::core::SkPaint::Style::kStroke_Style);
        let bounds_storage = if paint.getImageFilter().is_some() || paint.getMaskFilter().is_some()
        {
            let Some(bounds) = crate::src::core::SkRect::SkRect::Bounds(points) else {
                return Ok(());
            };
            if unsafe { canvas.as_ref() }.internalQuickReject(&bounds, &stroke_paint, None)? {
                return Ok(());
            }
            Some(bounds)
        } else {
            None
        };
        if let Some(mut layer) = unsafe {
            Self::aboutToDraw_default_raw(canvas, &stroke_paint, bounds_storage.as_ref())
        }? {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawPoints(mode, points, paint)?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:1979-2013
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2694-2694
    unsafe fn canAttemptBlurredRRectDraw_raw<'paint>(
        canvas: core::ptr::NonNull<Self>,
        paint: &'paint crate::src::core::SkPaint::SkPaint,
    ) -> Option<&'paint crate::src::core::SkBlurMaskFilterImpl::SkBlurMaskFilterImpl> {
        if !unsafe { Self::top_device_raw(canvas).as_ref() }.useDrawCoverageMaskForMaskFilters() {
            return None;
        }
        if paint.getPathEffect().is_some() {
            return None;
        }
        if crate::src::core::SkStrokeRec::SkStrokeRec::from_paint(paint, 1.0).getStyle()
            != crate::src::core::SkStrokeRec::Style::kFill_Style
        {
            return None;
        }
        let filter = paint.getMaskFilter()?;
        if filter.r#type() != crate::src::core::SkMaskFilterBase::Type::kBlur {
            return None;
        }
        let blur = filter
            .asBlurImpl()
            .expect("native blur discriminator identifies the actual blur implementation");
        if blur.blurStyle() != crate::include::core::SkBlurTypes::SkBlurStyle::kNormal_SkBlurStyle {
            return None;
        }
        if !unsafe { canvas.as_ref() }
            .getTotalMatrix()
            .isSimilarity(crate::include::core::SkScalar::SK_ScalarNearlyZero)
        {
            return None;
        }
        Some(blur)
    }
}
// Source: src/core/SkCanvas.cpp:2015-2040
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2698-2701
    unsafe fn attemptBlurredRRectDraw_raw<'operation>(
        canvas: core::ptr::NonNull<Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        blur: &crate::src::core::SkBlurMaskFilterImpl::SkBlurMaskFilterImpl,
        paint: &crate::src::core::SkPaint::SkPaint,
        flags: PredrawFlags,
    ) -> Result<
        Option<crate::src::core::SkCanvasPriv::AutoLayerForImageFilter<'operation, 'device>>,
        crate::RasterError,
    > {
        debug_assert!(
            unsafe { Self::canAttemptBlurredRRectDraw_raw(canvas, paint) }
                .is_some_and(|expected| core::ptr::eq(blur, expected))
                && flags.0 & PredrawFlags::kSkipMaskFilterAutoLayer.0 == 0
        );
        let bounds = rrect.getBounds();
        let Some(mut layer) = unsafe {
            Self::aboutToDraw_raw(
                canvas,
                paint,
                Some(bounds),
                PredrawFlags(flags.0 | PredrawFlags::kSkipMaskFilterAutoLayer.0),
            )
        }?
        else {
            return Ok(None);
        };
        let sigma = blur.computeXformedSigma(&unsafe { canvas.as_ref() }.getTotalMatrix());
        if unsafe { Self::top_device_raw(canvas).as_mut() }.drawBlurredRRect(
            rrect,
            layer.paint(),
            sigma,
        )? {
            unsafe { layer.finish() }?;
            return Ok(None);
        }
        unsafe { layer.addMaskFilterLayer(Some(bounds)) }?;
        Ok(Some(layer))
    }
}
// Source: src/core/SkCanvas.cpp:2042-2062
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2343-2343
    pub(crate) unsafe fn onDrawRect_base_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        debug_assert!(rect.isSorted());
        if unsafe { canvas.as_ref() }.internalQuickReject(rect, paint, None)? {
            return Ok(());
        }
        let layer = if let Some(blur) =
            unsafe { Self::canAttemptBlurredRRectDraw_raw(canvas, paint) }
        {
            unsafe {
                Self::attemptBlurredRRectDraw_raw(
                    canvas,
                    &crate::src::core::SkRRect::SkRRect::MakeRect(rect),
                    blur,
                    paint,
                    PredrawFlags::kCheckForOverwrite,
                )
            }?
        } else {
            unsafe {
                Self::aboutToDraw_raw(canvas, paint, Some(rect), PredrawFlags::kCheckForOverwrite)
            }?
        };
        if let Some(mut layer) = layer {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawRect(rect, paint)?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2064-2075
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2350-2350
    pub(crate) unsafe fn onDrawRegion_base_raw(
        canvas: core::ptr::NonNull<Self>,
        region: &crate::src::core::SkRegion::SkRegion,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounds = crate::src::core::SkRect::SkRect::Make(region.get_bounds());
        if unsafe { canvas.as_ref() }.internalQuickReject(&bounds, paint, None)? {
            return Ok(());
        }
        if let Some(mut layer) =
            unsafe { Self::aboutToDraw_default_raw(canvas, paint, Some(&bounds)) }?
        {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawRegion(region, paint)?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2076-2118
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2342-2342
    pub(crate) unsafe fn onDrawBehind_base_raw(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let mut device = unsafe { Self::top_device_raw(canvas) };
        let bounds = {
            let this = unsafe { canvas.as_ref() };
            let mut iter = crate::src::core::SkDeque::Iter::new(
                this.fMCStack.as_ref().unwrap(),
                crate::src::core::SkDeque::IterStart::kBack_IterStart,
            );
            loop {
                let record = iter.prev().cast::<MCRec<'device>>();
                if record.is_null() {
                    return Ok(());
                }
                let record = unsafe { &*record };
                if let Some(back) = record.fBackImage.as_ref() {
                    debug_assert!(core::ptr::addr_eq(device.as_ptr(), record.fDevice.as_ptr()));
                    break crate::src::core::SkRect::SkIRect::make_xywh(
                        back.fLoc.fX,
                        back.fLoc.fY,
                        unsafe { back.image() }.width(),
                        unsafe { back.image() }.height(),
                    );
                }
            }
        };
        unsafe { device.as_mut() }.pushClipStack();
        // Preserve native temporary matrix restore with disjoint, SHORT base leases:
        // do not retain a base reference while calling a whole-device virtual.
        let previous = crate::src::core::SkM44::SkM44::from(
            unsafe { device.as_ref() }.base().local_to_device(),
        );
        unsafe { device.as_mut().base_mut() }
            .set_local_to_device(&crate::src::core::SkM44::SkM44::default());
        unsafe { device.as_mut() }.clipRect(
            &crate::src::core::SkRect::SkRect::Make(&bounds),
            crate::include::core::SkClipOp::SkClipOp::kIntersect,
            false,
        );
        unsafe { device.as_mut().base_mut() }.set_local_to_device(&previous);
        let result = (|| -> Result<(), crate::RasterError> {
            if let Some(mut layer) = unsafe { Self::aboutToDraw_default_raw(canvas, paint, None) }?
            {
                {
                    let (canvas, paint) = layer.canvas_and_paint();
                    unsafe { Self::top_device_raw(canvas).as_mut() }.drawPaint(paint)?;
                }
                unsafe { layer.finish() }?;
            }
            Ok(())
        })();
        unsafe { device.as_mut() }.popClipStack();
        result
    }
}
// Source: src/core/SkCanvas.cpp:2120-2139
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2346-2346
    pub(crate) unsafe fn onDrawOval_base_raw(
        canvas: core::ptr::NonNull<Self>,
        oval: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 2121; @condition b"oval.isSorted()\0".as_ptr(); oval.isSorted());
        if unsafe { canvas.as_ref() }.internalQuickReject(oval, paint, None)? {
            return Ok(());
        }
        let layer =
            if let Some(blur) = unsafe { Self::canAttemptBlurredRRectDraw_raw(canvas, paint) } {
                unsafe {
                    Self::attemptBlurredRRectDraw_raw(
                        canvas,
                        &crate::src::core::SkRRect::SkRRect::MakeOval(oval),
                        blur,
                        paint,
                        PredrawFlags::kNone,
                    )
                }?
            } else {
                unsafe { Self::aboutToDraw_default_raw(canvas, paint, Some(oval)) }?
            };
        if let Some(mut layer) = layer {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawOval(oval, paint)?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2141-2165
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2347-2348
    pub(crate) unsafe fn onDrawArc_base_raw(
        canvas: core::ptr::NonNull<Self>,
        oval: &crate::src::core::SkRect::SkRect,
        start: f32,
        sweep: f32,
        use_center: bool,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 2144; @condition b"oval.isSorted()\0".as_ptr(); oval.isSorted());
        if unsafe { canvas.as_ref() }.internalQuickReject(oval, paint, None)? {
            return Ok(());
        }
        let blur = unsafe { Self::canAttemptBlurredRRectDraw_raw(canvas, paint) };
        let layer = if let Some(blur) = blur.filter(|_| sweep.abs() >= 360.0) {
            unsafe {
                Self::attemptBlurredRRectDraw_raw(
                    canvas,
                    &crate::src::core::SkRRect::SkRRect::MakeOval(oval),
                    blur,
                    paint,
                    PredrawFlags::kNone,
                )
            }?
        } else {
            unsafe { Self::aboutToDraw_default_raw(canvas, paint, Some(oval)) }?
        };
        if let Some(mut layer) = layer {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawArc(
                    &crate::include::core::SkArc::SkArc::MakeWithCenter(
                        oval, start, sweep, use_center,
                    ),
                    paint,
                )?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2167-2197
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2344-2344
    pub(crate) unsafe fn onDrawRRect_base_raw(
        canvas: core::ptr::NonNull<Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounds = rrect.getBounds();
        if rrect.isRect() {
            return unsafe { Self::drawRect_raw(canvas, bounds, paint) };
        }
        if rrect.isOval() {
            return unsafe { Self::drawOval_raw(canvas, bounds, paint) };
        }
        if unsafe { canvas.as_ref() }.internalQuickReject(bounds, paint, None)? {
            return Ok(());
        }
        let layer = if let Some(blur) =
            unsafe { Self::canAttemptBlurredRRectDraw_raw(canvas, paint) }
        {
            unsafe {
                Self::attemptBlurredRRectDraw_raw(canvas, rrect, blur, paint, PredrawFlags::kNone)
            }?
        } else {
            unsafe { Self::aboutToDraw_default_raw(canvas, paint, Some(bounds)) }?
        };
        if let Some(mut layer) = layer {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawRRect(rrect, paint)?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2199-2209
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2345-2345
    pub(crate) unsafe fn onDrawDRRect_base_raw(
        canvas: core::ptr::NonNull<Self>,
        outer: &crate::src::core::SkRRect::SkRRect,
        inner: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounds = outer.getBounds();
        if unsafe { canvas.as_ref() }.internalQuickReject(bounds, paint, None)? {
            return Ok(());
        }
        if let Some(mut layer) =
            unsafe { Self::aboutToDraw_default_raw(canvas, paint, Some(bounds)) }?
        {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawDRRect(outer, inner, paint)?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2211-2230
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2349-2349
    pub(crate) unsafe fn onDrawPath_base_raw(
        canvas: core::ptr::NonNull<Self>,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if !path.isFinite() {
            return Ok(());
        }
        let bounds = path.getBounds();
        if !path.isInverseFillType()
            && unsafe { canvas.as_ref() }.internalQuickReject(bounds, paint, None)?
        {
            return Ok(());
        }
        if path.isInverseFillType() && bounds.width() <= 0.0 && bounds.height() <= 0.0 {
            unsafe { Self::internalDrawPaint_raw(canvas, paint) }?;
            return Ok(());
        }
        if let Some(mut layer) = unsafe {
            Self::aboutToDraw_default_raw(
                canvas,
                paint,
                if path.isInverseFillType() {
                    None
                } else {
                    Some(bounds)
                },
            )
        }? {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }.drawPath(path, paint)?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2232-2241
fn clean_paint_for_drawImage(
    paint: Option<&crate::src::core::SkPaint::SkPaint>,
) -> crate::src::core::SkPaint::SkPaint {
    let mut cleaned = crate::src::core::SkPaint::SkPaint::default();
    if let Some(paint) = paint {
        cleaned = paint.clone();
        cleaned.setStyle(crate::src::core::SkPaint::Style::kFill_Style);
        cleaned.setPathEffect(crate::include::core::SkRefCnt::sk_sp::default());
    }
    cleaned
}

// Source: src/core/SkCanvas.cpp:2244-2249
fn clean_paint_for_drawVertices(
    mut paint: crate::src::core::SkPaint::SkPaint,
) -> crate::src::core::SkPaint::SkPaint {
    paint.setStyle(crate::src::core::SkPaint::Style::kFill_Style);
    paint.setMaskFilter(None);
    paint.setPathEffect(crate::include::core::SkRefCnt::sk_sp::default());
    paint
}

// Source: include/core/SkCanvas.h:2362-2363
// Source: src/core/SkCanvas.cpp:2252-2255
impl<'device> SkCanvas<'device> {
    // This obsolete native virtual is deliberately a fail-stop trap, never a
    // drawing fallback. The active image path is drawImageRect/onDrawImageRect2.
    pub(crate) unsafe fn onDrawImage2_base_raw(
        _canvas: core::ptr::NonNull<Self>,
        _image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        _x: f32,
        _y: f32,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) {
        crate::SkUNREACHABLE!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 2254; );
    }
}

// Source: src/core/SkCanvas.cpp:2257-2269
fn clean_sampling_for_constraint(
    sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    constraint: SrcRectConstraint,
) -> crate::include::core::SkSamplingOptions::SkSamplingOptions {
    use crate::include::core::SkSamplingOptions::{SkFilterMode, SkMipmapMode, SkSamplingOptions};
    if constraint == SrcRectConstraint::kStrict_SrcRectConstraint {
        if sampling.mipmap() != SkMipmapMode::kNone {
            return SkSamplingOptions::with_filter(sampling.filter());
        }
        if sampling.isAniso() {
            return SkSamplingOptions::with_filter(SkFilterMode::kLinear);
        }
    }
    *sampling
}

// Source: src/core/SkCanvas.cpp:2271-2369
impl<'device> SkCanvas<'device> {
    /// Safety: the actual Canvas/device/image raster leases remain live and
    /// exclusive; raw COW notification must not overlap whole-owner references.
    // Source: include/core/SkCanvas.h:2364-2365
    pub(crate) unsafe fn onDrawImageRect2_base_raw(
        canvas: core::ptr::NonNull<Self>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        src: &crate::src::core::SkRect::SkRect,
        dst: &crate::src::core::SkRect::SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::SkImageFilterTypes::{
            Context, DeviceSpace, FilterResult, ParameterSpace, Stats,
        };
        let mut real_paint = clean_paint_for_drawImage(paint);
        let real_sampling = clean_sampling_for_constraint(sampling, constraint);
        if unsafe { canvas.as_ref() }.internalQuickReject(dst, &real_paint, None)? {
            return Ok(());
        }
        if unsafe { Self::top_device_raw(canvas).as_ref() }.shouldDrawAsTiledImageRect() {
            if unsafe {
                Self::top_device_raw(canvas).as_mut().drawAsTiledImageRect(
                    canvas,
                    image,
                    Some(src),
                    dst,
                    &real_sampling,
                    &real_paint,
                    constraint,
                )
            }? {
                return Ok(());
            }
        }
        if real_paint.getImageFilter().is_some()
            && !image.isAlphaOnly()
            && real_paint.getMaskFilter().is_none()
        {
            let mut device = unsafe { Self::top_device_raw(canvas) };
            let image_bounds = ParameterSpace::new(*dst);
            let output_bounds = DeviceSpace::new(unsafe { device.as_ref() }.devClipBounds());
            let filter_as_span = FilterToSpan::new(real_paint.refImageFilter().as_ref());
            let Some((mapping, src_bounds)) = get_layer_mapping_and_bounds(
                filter_as_span.as_span(),
                unsafe { device.as_ref() }.base().local_to_device44(),
                &output_bounds,
                Some(image_bounds),
                1.0,
            )?
            else {
                return Ok(());
            };
            if !unsafe { Self::predraw_notify_overwrite_raw(canvas, false) } {
                return Ok(());
            }
            let color_info = unsafe { device.as_ref() }
                .base()
                .image_info()
                .colorInfo()
                .clone();
            let props = *unsafe { device.as_ref() }.base().surface_props();
            let backend = unsafe { device.as_ref() }
                .createImageFilteringBackend(&props, image_filter_color_type(&color_info))?;
            let stats = core::cell::Cell::new(Stats::default());
            let mut ctx = Context::new(
                backend,
                &mapping,
                src_bounds,
                &FilterResult::default(),
                color_info.refColorSpace(),
                Some(&stats),
            );
            let mut source = unsafe {
                FilterResult::MakeFromImage(&ctx, image.clone(), *src, image_bounds, *sampling)
            }?;
            source = unsafe { apply_alpha_and_colorfilter(&ctx, &source, &real_paint) }?;
            ctx = ctx
                .withNewDesiredOutput(mapping.deviceToLayer(&output_bounds))
                .withNewSource(&source);
            let result = unsafe {
                real_paint
                    .getImageFilter()
                    .expect("native image filter branch")
                    .filterImage(&ctx)
            }?;
            unsafe { result.draw(&ctx, device, real_paint.getBlender()) }?;
            stats.get().reportStats();
            return Ok(());
        }
        if real_paint.getMaskFilter().is_some()
            && unsafe { Self::top_device_raw(canvas).as_ref() }.useDrawCoverageMaskForMaskFilters()
        {
            let (draw_dst_rect, shader) =
                crate::src::shaders::SkImageShader::SkImageShader::MakeForDrawRect(
                    image,
                    &real_paint,
                    sampling,
                    *src,
                    *dst,
                    constraint == SrcRectConstraint::kStrict_SrcRectConstraint,
                );
            if draw_dst_rect.isEmpty() || shader.is_none() {
                return Ok(());
            }
            real_paint.setShader(shader);
            unsafe { Self::drawRect_raw(canvas, &draw_dst_rect, &real_paint) }?;
            return Ok(());
        }
        let shader_flag = if image.isOpaque() {
            PredrawFlags::kOpaqueShaderOverride
        } else {
            PredrawFlags::kNonOpaqueShaderOverride
        };
        if let Some(mut layer) = unsafe {
            Self::aboutToDraw_raw(
                canvas,
                &real_paint,
                Some(dst),
                PredrawFlags(PredrawFlags::kCheckForOverwrite.0 | shader_flag.0),
            )
        }? {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe {
                    Self::top_device_raw(canvas).as_mut().drawImageRect(
                        image,
                        Some(src),
                        dst,
                        &real_sampling,
                        paint,
                        constraint,
                    )
                }?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2366-2367
    // Source: src/core/SkCanvas.cpp:2371-2383
    pub(crate) unsafe fn onDrawImageLattice2_base_raw(
        canvas: core::ptr::NonNull<Self>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        lattice: &Lattice,
        dst: &crate::src::core::SkRect::SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let real_paint = clean_paint_for_drawImage(paint);
        if unsafe { canvas.as_ref() }.internalQuickReject(dst, &real_paint, None)? {
            return Ok(());
        }
        if let Some(mut layer) =
            unsafe { Self::aboutToDraw_default_raw(canvas, &real_paint, Some(dst)) }?
        {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe {
                    Self::top_device_raw(canvas)
                        .as_mut()
                        .drawImageLattice(image, lattice, dst, filter, paint)
                }?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}

// Dual-source mapping: the header declaration/inline overloads and the .cpp
// implementation share this single Rust method. Its canonical position follows
// SkCanvas.cpp:2385-2396; relocating it into header order would reverse the
// implementation-file order. No duplicate declaration/body is manufactured.
// Source: include/core/SkCanvas.h:1601-1606
// Source: src/core/SkCanvas.cpp:2385-2396
impl<'device> SkCanvas<'device> {
    /// Safety: actual source pixels stay initialized/readable and disjoint from
    /// the exclusive Canvas destination throughout the native draw/filter chain.
    /// Option<&Arc> preserves native nullable borrowed input and retain capability.
    pub unsafe fn drawImage_raw(
        canvas: core::ptr::NonNull<Self>,
        image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        x: f32,
        y: f32,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        // Browser's scoped trace is the platform-neutral TRACE_EVENT0 adapter.
        let _trace = browser_tracing::span("skia", "SkCanvas::drawImage");
        let Some(image) = image else {
            return Ok(());
        };
        let src =
            crate::src::core::SkRect::SkRect::MakeWH(image.width() as f32, image.height() as f32);
        let dst = crate::src::core::SkRect::SkRect::MakeXYWH(
            x,
            y,
            image.width() as f32,
            image.height() as f32,
        );
        unsafe {
            Self::drawImageRect_raw(
                canvas,
                image,
                Some(&src),
                &dst,
                sampling,
                paint,
                SrcRectConstraint::kFast_SrcRectConstraint,
            )
        }
    }
}
// Dual-source mapping: the header declaration/inline overloads and the .cpp
// implementation share this single Rust method. Its canonical position follows
// SkCanvas.cpp:2398-2413; relocating it into header order would reverse the
// implementation-file order. No duplicate declaration/body is manufactured.
// Source: include/core/SkCanvas.h:1607-1610
// Source: src/core/SkCanvas.cpp:2398-2413
impl<'device> SkCanvas<'device> {
    /// Safety: source pixels remain readable and destination pixels exclusively
    /// writable through the complete native draw/filter/COW operation.
    /// A retained image reference encodes the native non-null image branch;
    /// src=None selects the native whole-image overload.
    pub unsafe fn drawImageRect_raw(
        canvas: core::ptr::NonNull<Self>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        src: Option<&crate::src::core::SkRect::SkRect>,
        dst: &crate::src::core::SkRect::SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        let full_src;
        let (src, constraint) = if let Some(src) = src {
            (src, constraint)
        } else {
            full_src = crate::src::core::SkRect::SkRect::MakeIWH(image.width(), image.height());
            (&full_src, SrcRectConstraint::kFast_SrcRectConstraint)
        };
        if !fillable(dst) || !fillable(src) {
            return Ok(());
        }
        unsafe { Self::onDrawImageRect2_raw(canvas, image, src, dst, sampling, paint, constraint) }
    }
}

// Source: src/core/SkCanvas.cpp:2415-2422
impl<'device> SkCanvas<'device> {
    /// Safety: Canvas is the original live pinned allocation; builder leases
    /// are nested in native stack order and never escape the matching release.
    // Source: include/core/SkCanvas.h:2706-2706
    unsafe fn obtainGlyphRunBuilder_raw(
        mut canvas: core::ptr::NonNull<Self>,
    ) -> core::ptr::NonNull<crate::src::text::GlyphRun::GlyphRunBuilder> {
        let this = unsafe { canvas.as_mut() };
        if this.fRunBuildersUsed >= this.fRunBuilders.size() {
            unsafe { core::pin::Pin::new_unchecked(&mut this.fRunBuilders) }
                .push_back(Box::new(crate::src::text::GlyphRun::GlyphRunBuilder::new()));
        }
        // The array moves Box handles, never the original builder allocation.
        // Derive writable provenance from the actual unique Box, not &Builder.
        let result = core::ptr::NonNull::from(
            unsafe { core::pin::Pin::new_unchecked(&mut this.fRunBuilders) }
                .at_mut(this.fRunBuildersUsed)
                .as_mut(),
        );
        this.fRunBuildersUsed += 1;
        result
    }
    // Source: src/core/SkCanvas.cpp:2424-2427
    // Source: include/core/SkCanvas.h:2708-2708
    unsafe fn releaseGlyphRunBuilder_raw(canvas: core::ptr::NonNull<Self>) {
        let used = unsafe { core::ptr::addr_of_mut!((*canvas.as_ptr()).fRunBuildersUsed) };
        debug_assert!(unsafe { *used } > 0);
        unsafe {
            *used -= 1;
        }
    }
}

// Source: src/core/SkCanvas.cpp:2429-2440
// Original two borrowed identities only. The guard neither moves/owns the
// builder nor lends a whole Canvas reference across drawing/COW callbacks.
struct AutoGlyphRunBuilder<'device> {
    fCanvas: core::ptr::NonNull<SkCanvas<'device>>,
    fGlyphRunBuilder: core::ptr::NonNull<crate::src::text::GlyphRun::GlyphRunBuilder>,
}
impl<'device> AutoGlyphRunBuilder<'device> {
    /// Safety: Canvas remains live/pinned and its nested builder operation lease
    /// remains exclusive until Drop; builder storage/views cannot escape it.
    unsafe fn new(canvas: core::ptr::NonNull<SkCanvas<'device>>) -> Self {
        Self {
            fCanvas: canvas,
            fGlyphRunBuilder: unsafe { SkCanvas::obtainGlyphRunBuilder_raw(canvas) },
        }
    }
    fn builder(&self) -> core::ptr::NonNull<crate::src::text::GlyphRun::GlyphRunBuilder> {
        self.fGlyphRunBuilder
    }
}
impl Drop for AutoGlyphRunBuilder<'_> {
    fn drop(&mut self) {
        unsafe {
            SkCanvas::releaseGlyphRunBuilder_raw(self.fCanvas);
        }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2352-2353
    // Source: src/core/SkCanvas.cpp:2442-2447
    /// Safety: blob, pinned Canvas, builder allocation and destination pixel
    /// leases remain live through all synchronous draw/COW callbacks.
    pub(crate) unsafe fn onDrawTextBlob_base_raw(
        canvas: core::ptr::NonNull<Self>,
        blob: &crate::src::core::SkTextBlob::SkTextBlob,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let scratch = unsafe { AutoGlyphRunBuilder::new(canvas) };
        let mut builder = scratch.builder();
        let list = unsafe {
            core::pin::Pin::new_unchecked(builder.as_mut())
                .blobToGlyphRunList(blob, crate::src::core::SkPoint::SkPoint::make(x, y))
        }?;
        unsafe { Self::onDrawGlyphRunList_raw(canvas, list, paint) }
    }
    // Source: include/core/SkCanvas.h:2355-2355
    // Source: src/core/SkCanvas.cpp:2449-2463
    pub(crate) unsafe fn onDrawGlyphRunList_base_raw(
        canvas: core::ptr::NonNull<Self>,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounds = list.sourceBoundsWithOrigin();
        if unsafe { canvas.as_ref() }.internalQuickReject(&bounds, paint, None)? {
            return Ok(());
        }
        if let Some(mut layer) = unsafe {
            Self::aboutToDraw_raw(
                canvas,
                paint,
                Some(&bounds),
                PredrawFlags::kSkipMaskFilterAutoLayer,
            )
        }? {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                let device = unsafe { Self::top_device_raw(canvas) };
                unsafe {
                    crate::src::core::SkDevice::drawGlyphRunList_raw(device, canvas, list, paint)
                }?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:2465-2504
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2580-2581
    pub unsafe fn convertBlobToSlug_raw(
        canvas: core::ptr::NonNull<Self>,
        blob: &crate::src::core::SkTextBlob::SkTextBlob,
        origin: crate::src::core::SkPoint::SkPoint,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<
        crate::include::core::SkRefCnt::sk_sp<dyn crate::include::private::chromium::Slug::Slug>,
        crate::RasterError,
    > {
        let _trace = browser_tracing::span("skia", "SkCanvas::convertBlobToSlug");
        let scratch = unsafe { AutoGlyphRunBuilder::new(canvas) };
        let mut builder = scratch.builder();
        let list = unsafe {
            core::pin::Pin::new_unchecked(builder.as_mut()).blobToGlyphRunList(blob, origin)
        }?;
        unsafe { Self::onConvertGlyphRunListToSlug_raw(canvas, list, paint) }
    }
    pub unsafe fn onConvertGlyphRunListToSlug_base_raw(
        canvas: core::ptr::NonNull<Self>,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<
        crate::include::core::SkRefCnt::sk_sp<dyn crate::include::private::chromium::Slug::Slug>,
        crate::RasterError,
    > {
        let bounds = list.sourceBoundsWithOrigin();
        if bounds.isEmpty() || !bounds.isFinite() || unsafe { canvas.as_ref() }.nothingToDraw(paint)
        {
            return Ok(crate::include::core::SkRefCnt::sk_sp::default());
        }
        if let Some(mut layer) = unsafe {
            Self::aboutToDraw_raw(
                canvas,
                paint,
                Some(&bounds),
                PredrawFlags::kSkipMaskFilterAutoLayer,
            )
        }? {
            let slug = {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe { Self::top_device_raw(canvas).as_mut() }
                    .convertGlyphRunListToSlug(list, paint)
            };
            unsafe { layer.finish() }?;
            return Ok(slug);
        }
        Ok(crate::include::core::SkRefCnt::sk_sp::default())
    }
    // Source: include/core/SkCanvas.h:2586-2586
    pub unsafe fn drawSlug_raw(
        canvas: core::ptr::NonNull<Self>,
        slug: Option<&dyn crate::include::private::chromium::Slug::Slug>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawSlug");
        if let Some(slug) = slug {
            unsafe { Self::onDrawSlug_raw(canvas, slug, paint) }?;
        }
        Ok(())
    }
    pub unsafe fn onDrawSlug_base_raw(
        canvas: core::ptr::NonNull<Self>,
        slug: &dyn crate::include::private::chromium::Slug::Slug,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let bounds = slug.sourceBoundsWithOrigin();
        if unsafe { canvas.as_ref() }.internalQuickReject(&bounds, paint, None)? {
            return Ok(());
        }
        if let Some(mut layer) = unsafe {
            Self::aboutToDraw_raw(
                canvas,
                paint,
                Some(&bounds),
                PredrawFlags::kSkipMaskFilterAutoLayer,
            )
        }? {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe {
                    Self::top_device_raw(canvas)
                        .as_mut()
                        .drawSlug(canvas, slug, paint)
                }?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
    pub unsafe fn onDrawSlug_raw(
        canvas: core::ptr::NonNull<Self>,
        slug: &dyn crate::include::private::chromium::Slug::Slug,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe { SkCanvasVirtual::onDrawSlug(hooks.as_ref(), canvas, slug, paint) };
        }
        unsafe { Self::onDrawSlug_base_raw(canvas, slug, paint) }
    }
    pub unsafe fn onConvertGlyphRunListToSlug_raw(
        canvas: core::ptr::NonNull<Self>,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<
        crate::include::core::SkRefCnt::sk_sp<dyn crate::include::private::chromium::Slug::Slug>,
        crate::RasterError,
    > {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe {
                SkCanvasVirtual::onConvertGlyphRunListToSlug(hooks.as_ref(), canvas, list, paint)
            };
        }
        unsafe { Self::onConvertGlyphRunListToSlug_base_raw(canvas, list, paint) }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:1831-1832
    // Source: src/core/SkCanvas.cpp:2507-2520
    pub unsafe fn drawSimpleText_raw(
        canvas: core::ptr::NonNull<Self>,
        text: *const core::ffi::c_void,
        byteLength: usize,
        encoding: crate::include::core::SkFontTypes::SkTextEncoding,
        x: f32,
        y: f32,
        font: &crate::src::core::SkFont::SkFont,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawSimpleText");
        if byteLength != 0 {
            let scratch = unsafe { AutoGlyphRunBuilder::new(canvas) };
            let mut builder = scratch.builder();
            let list = unsafe {
                core::pin::Pin::new_unchecked(builder.as_mut()).textToGlyphRunList(
                    font,
                    paint,
                    text,
                    byteLength,
                    crate::src::core::SkPoint::SkPoint::make(x, y),
                    encoding,
                )
            }?;
            if !list.empty() {
                unsafe { Self::onDrawGlyphRunList_raw(canvas, list, paint) }?;
            }
        }
        Ok(())
    }
    // Source: include/core/SkCanvas.h:1858-1861
    /// Safety: preserve the original Canvas/device/pixel lease for synchronous text drawing.
    pub unsafe fn drawString_raw(
        canvas: core::ptr::NonNull<Self>,
        text: &core::ffi::CStr,
        x: f32,
        y: f32,
        font: &crate::src::core::SkFont::SkFont,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::drawSimpleText_raw(
                canvas,
                text.as_ptr().cast(),
                text.to_bytes().len(),
                crate::include::core::SkFontTypes::SkTextEncoding::kUTF8,
                x,
                y,
                font,
                paint,
            )
        }
    }
    // Source: include/core/SkCanvas.h:1887-1890
    /// Borrowed projection of native SkString's actual c_str()/size() byte span.
    /// Embedded NULs remain part of the explicit byte length, unlike CStr input.
    /// This view does not construct or stand in for a SkString storage owner.
    /// Safety: preserve Canvas/device/pixel/font leases through synchronous drawing.
    pub unsafe fn drawString_bytes_raw(
        canvas: core::ptr::NonNull<Self>,
        text: &[u8],
        x: f32,
        y: f32,
        font: &crate::src::core::SkFont::SkFont,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            Self::drawSimpleText_raw(
                canvas,
                text.as_ptr().cast(),
                text.len(),
                crate::include::core::SkFontTypes::SkTextEncoding::kUTF8,
                x,
                y,
                font,
                paint,
            )
        }
    }

    // Source: include/core/SkCanvas.h:1912-1914
    // Source: src/core/SkCanvas.cpp:2522-2539
    pub unsafe fn drawGlyphs_clusters_raw(
        canvas: core::ptr::NonNull<Self>,
        glyphs: &[u16],
        positions: &[crate::src::core::SkPoint::SkPoint],
        clusters: &[u32],
        utf8text: &[u8],
        origin: crate::src::core::SkPoint::SkPoint,
        font: &crate::src::core::SkFont::SkFont,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if glyphs.is_empty() {
            return Ok(());
        }
        let scratch = unsafe { AutoGlyphRunBuilder::new(canvas) };
        let run = unsafe {
            crate::src::text::GlyphRun::GlyphRun::new(
                font,
                positions,
                glyphs,
                utf8text,
                clusters,
                &[],
            )
        };
        let mut builder = scratch.builder();
        let list = unsafe {
            core::pin::Pin::new_unchecked(builder.as_mut()).makeGlyphRunList(&run, paint, origin)
        }?;
        unsafe { Self::onDrawGlyphRunList_raw(canvas, &list, paint) }
    }
    // Source: include/core/SkCanvas.h:1934-1935
    // Source: src/core/SkCanvas.cpp:2541-2558
    pub unsafe fn drawGlyphs_raw(
        canvas: core::ptr::NonNull<Self>,
        glyphs: &[u16],
        positions: &[crate::src::core::SkPoint::SkPoint],
        origin: crate::src::core::SkPoint::SkPoint,
        font: &crate::src::core::SkFont::SkFont,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if glyphs.is_empty() {
            return Ok(());
        }
        let scratch = unsafe { AutoGlyphRunBuilder::new(canvas) };
        let run = unsafe {
            crate::src::text::GlyphRun::GlyphRun::new(font, positions, glyphs, &[], &[], &[])
        };
        let mut builder = scratch.builder();
        let list = unsafe {
            core::pin::Pin::new_unchecked(builder.as_mut()).makeGlyphRunList(&run, paint, origin)
        }?;
        unsafe { Self::onDrawGlyphRunList_raw(canvas, &list, paint) }
    }
    // Source: include/core/SkCanvas.h:1956-1957
    // Source: src/core/SkCanvas.cpp:2560-2578
    pub unsafe fn drawGlyphsRSXform_raw(
        canvas: core::ptr::NonNull<Self>,
        glyphs: &[u16],
        xforms: &[crate::src::core::SkRSXform::SkRSXform],
        origin: crate::src::core::SkPoint::SkPoint,
        font: &crate::src::core::SkFont::SkFont,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if glyphs.is_empty() {
            return Ok(());
        }
        let scratch = unsafe { AutoGlyphRunBuilder::new(canvas) };
        let mut builder = scratch.builder();
        // The returned raw-span run retains the original builder's allocations;
        // end its temporary Rust projection before taking the next builder lease.
        let run = {
            let (positions, rotations) =
                unsafe { core::pin::Pin::new_unchecked(builder.as_mut()).convertRSXForm(xforms) };
            unsafe {
                crate::src::text::GlyphRun::GlyphRun::new(
                    font,
                    positions,
                    glyphs,
                    &[],
                    &[],
                    rotations,
                )
            }
        };
        let list = unsafe {
            core::pin::Pin::new_unchecked(builder.as_mut()).makeGlyphRunList(&run, paint, origin)
        }?;
        unsafe { Self::onDrawGlyphRunList_raw(canvas, &list, paint) }
    }
    // Source: include/core/SkCanvas.h:1982-1982
    // Source: src/core/SkCanvas.cpp:2580-2600
    pub unsafe fn drawTextBlob_raw(
        canvas: core::ptr::NonNull<Self>,
        blob: Option<&crate::src::core::SkTextBlob::SkTextBlob>,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawTextBlob");
        let Some(blob) = blob else {
            return Ok(());
        };
        if !blob.bounds().makeOffset(x, y).isFinite() {
            return Ok(());
        }
        let mut total = 0;
        let mut iter = unsafe { crate::src::core::SkTextBlob::Iter::new(blob) };
        let mut run = crate::src::core::SkTextBlob::Run {
            fTypeface: core::ptr::null(),
            fGlyphCount: 0,
            fGlyphIndices: core::ptr::null(),
            #[cfg(SK_UNTIL_CRBUG_1187654_IS_FIXED)]
            fClusterIndex_forTest: core::ptr::null(),
            #[cfg(SK_UNTIL_CRBUG_1187654_IS_FIXED)]
            fUtf8Size_forTest: 0,
            #[cfg(SK_UNTIL_CRBUG_1187654_IS_FIXED)]
            fUtf8_forTest: core::ptr::null(),
        };
        while unsafe { iter.next(Some(&mut run)) } {
            let left = (1 << 21) - total;
            if run.fGlyphCount > left {
                return Ok(());
            }
            total += run.fGlyphCount;
        }
        unsafe { Self::onDrawTextBlob_raw(canvas, blob, x, y, paint) }
    }
    // Source: include/core/SkCanvas.h:2002-2004
    /// Safety: same Canvas/device/pixel and synchronous font leases as drawTextBlob_raw.
    pub unsafe fn drawTextBlob_shared_raw(
        canvas: core::ptr::NonNull<Self>,
        blob: &crate::include::core::SkRefCnt::sk_sp<crate::src::core::SkTextBlob::SkTextBlob>,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::drawTextBlob_raw(canvas, blob.get(), x, y, paint) }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2382-2382
    // Source: src/core/SkCanvas.cpp:2602-2615
    unsafe fn onDrawVerticesObject_base_raw(
        canvas: core::ptr::NonNull<Self>,
        vertices: &crate::src::core::SkVertices::SkVertices,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let simple_paint = clean_paint_for_drawVertices(paint.clone());
        let bounds = vertices.bounds();
        if unsafe { canvas.as_ref() }.internalQuickReject(bounds, &simple_paint, None)? {
            return Ok(());
        }
        if let Some(mut layer) =
            unsafe { Self::aboutToDraw_default_raw(canvas, &simple_paint, Some(bounds)) }?
        {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe {
                    Self::top_device_raw(canvas).as_mut().drawVertices(
                        vertices,
                        <dyn crate::include::core::SkBlender::SkBlender>::Mode(mode),
                        paint,
                        false,
                    )
                }?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:2617-2623
impl<'device> SkCanvas<'device> {
    pub unsafe fn onDrawMesh_base_raw(
        canvas: core::ptr::NonNull<Self>,
        mesh: &crate::src::core::SkMesh::SkMesh,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let simple = clean_paint_for_drawVertices(paint.clone());
        if let Some(mut layer) = unsafe { Self::aboutToDraw_default_raw(canvas, &simple, None) }? {
            let (canvas, _) = layer.canvas_and_paint();
            unsafe {
                Self::top_device_raw(canvas)
                    .as_mut()
                    .drawMesh(mesh, blender, paint)
            }?;
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
    // Source: src/core/SkCanvas.cpp:2625-2634
    // Source: include/core/SkCanvas.h:2154-2155
    pub unsafe fn drawPatch_raw(
        canvas: core::ptr::NonNull<Self>,
        cubics: Option<&[crate::src::core::SkPoint::SkPoint; 12]>,
        colors: Option<&[crate::src::core::SkColor::SkColor; 4]>,
        tex: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawPatch");
        let Some(cubics) = cubics else {
            return Ok(());
        };
        unsafe { Self::onDrawPatch_raw(canvas, cubics, colors, tex, mode, paint) }
    }
    // Source: src/core/SkCanvas.cpp:2636-2659
    pub unsafe fn onDrawPatch_base_raw(
        canvas: core::ptr::NonNull<Self>,
        cubics: &[crate::src::core::SkPoint::SkPoint; 12],
        colors: Option<&[crate::src::core::SkColor::SkColor; 4]>,
        tex: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let Some(bounds) = crate::src::core::SkRect::SkRect::Bounds(cubics) else {
            return Ok(());
        };
        let simple = clean_paint_for_drawVertices(paint.clone());
        if unsafe { canvas.as_ref() }.internalQuickReject(&bounds, &simple, None)? {
            return Ok(());
        }
        if let Some(mut layer) =
            unsafe { Self::aboutToDraw_default_raw(canvas, &simple, Some(&bounds)) }?
        {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe {
                    Self::top_device_raw(canvas).as_mut().drawPatch(
                        cubics,
                        colors,
                        tex,
                        <dyn crate::include::core::SkBlender::SkBlender>::Mode(mode),
                        paint,
                    )
                }?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2215-2215
    // Source: src/core/SkCanvas.cpp:2661-2672
    /// Safety: retain drawable backing and original Canvas/Device pixel leases;
    /// every temporary Canvas projection ends before drawable callbacks reenter.
    pub unsafe fn drawDrawable_xy_raw(
        canvas: core::ptr::NonNull<Self>,
        drawable: Option<&dyn crate::src::core::SkDrawable::SkDrawable>,
        x: f32,
        y: f32,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawDrawable");
        let Some(drawable) = drawable else {
            return Ok(());
        };
        if x != 0.0 || y != 0.0 {
            let matrix = crate::src::core::SkMatrix::SkMatrix::Translate(x, y);
            unsafe { Self::onDrawDrawable_raw(canvas, drawable, Some(&matrix)) }
        } else {
            unsafe { Self::onDrawDrawable_raw(canvas, drawable, None) }
        }
    }
    // Source: include/core/SkCanvas.h:2200-2200
    // Source: src/core/SkCanvas.cpp:2674-2683
    pub unsafe fn drawDrawable_raw(
        canvas: core::ptr::NonNull<Self>,
        drawable: Option<&dyn crate::src::core::SkDrawable::SkDrawable>,
        mut matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        let _trace = browser_tracing::span("skia", "SkCanvas::drawDrawable");
        let Some(drawable) = drawable else {
            return Ok(());
        };
        if matrix.is_some_and(|m| m.isIdentity()) {
            matrix = None;
        }
        unsafe { Self::onDrawDrawable_raw(canvas, drawable, matrix) }
    }
    // Source: src/core/SkCanvas.cpp:2685-2691
    pub(crate) unsafe fn onDrawDrawable_base_raw(
        canvas: core::ptr::NonNull<Self>,
        drawable: &dyn crate::src::core::SkDrawable::SkDrawable,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        if unsafe { Self::predraw_notify_overwrite_raw(canvas, false) } {
            let device = unsafe { Self::top_device_raw(canvas) };
            unsafe {
                crate::src::core::SkDevice::drawDrawable_raw(device, canvas, drawable, matrix)
            }?;
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:2693-2715
impl<'device> SkCanvas<'device> {
    pub unsafe fn onDrawAtlas2_base_raw(
        canvas: core::ptr::NonNull<Self>,
        atlas: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[crate::src::core::SkRect::SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        cull: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let mut real = clean_paint_for_drawVertices(clean_paint_for_drawImage(paint));
        real.setShader(atlas.makeShader_default(sampling));
        if let Some(cull) = cull {
            if unsafe { canvas.as_ref() }.internalQuickReject(cull, &real, None)? {
                return Ok(());
            }
        }
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 2707; @condition b"!realPaint.getMaskFilter()\0".as_ptr(); real.getMaskFilter().is_none());
        if let Some(mut layer) = unsafe { Self::aboutToDraw_default_raw(canvas, &real, None) }? {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe {
                    Self::top_device_raw(canvas).as_mut().drawAtlas(
                        xform,
                        tex,
                        colors,
                        <dyn crate::include::core::SkBlender::SkBlender>::Mode(mode),
                        paint,
                    )
                }?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2379-2379
    // Source: src/core/SkCanvas.cpp:2716-2722
    pub(crate) unsafe fn onDrawAnnotation_base_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        key: &std::ffi::CStr,
        value: Option<&std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) {
        if unsafe { Self::predraw_notify_overwrite_raw(canvas, false) } {
            unsafe { Self::top_device_raw(canvas).as_mut() }.drawAnnotation(rect, key, value);
        }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2386-2387
    // Source: src/core/SkCanvas.cpp:2724-2737
    pub(crate) unsafe fn onDrawEdgeAAQuad_base_raw(
        canvas: core::ptr::NonNull<Self>,
        r: &crate::src::core::SkRect::SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        aa: QuadAAFlags,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkCanvas.cpp\0".as_ptr(), 2726; @condition b"r.isSorted()\0".as_ptr(); r.isSorted());
        let mut paint = crate::src::core::SkPaint::SkPaint::from_color4f(color, None);
        paint.setBlendMode(mode);
        if unsafe { canvas.as_ref() }.internalQuickReject(r, &paint, None)? {
            return Ok(());
        }
        if unsafe { Self::predraw_notify_overwrite_raw(canvas, false) } {
            unsafe { Self::top_device_raw(canvas).as_mut() }
                .drawEdgeAAQuad(r, clip, aa, color, mode)?;
        }
        Ok(())
    }
    // Source: include/core/SkCanvas.h:2371-2374
    // Source: src/core/SkCanvas.cpp:2739-2825
    pub(crate) unsafe fn onDrawEdgeAAImageSet2_base_raw(
        canvas: core::ptr::NonNull<Self>,
        images: &[ImageSetEntry],
        clips: &[crate::src::core::SkPoint::SkPoint],
        matrices: &[crate::src::core::SkMatrix::SkMatrix],
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        if images.is_empty() {
            return Ok(());
        }
        let real_paint = clean_paint_for_drawImage(paint);
        let real_sampling = clean_sampling_for_constraint(sampling, constraint);
        let needs_layer =
            real_paint.getImageFilter().is_some() || real_paint.getMaskFilter().is_some();
        let bounds_valid = images.len() == 1 || needs_layer;
        let first = &images[0];
        let mut bounds = if first.fMatrixIndex >= 0 {
            matrices[first.fMatrixIndex as usize]
                .mapRect_value(unsafe { first.fDstRect.assume_init_ref() })
        } else {
            unsafe { first.fDstRect.assume_init() }
        };
        if needs_layer {
            for entry in &images[1..] {
                let r = if entry.fMatrixIndex >= 0 {
                    matrices[entry.fMatrixIndex as usize]
                        .mapRect_value(unsafe { entry.fDstRect.assume_init_ref() })
                } else {
                    unsafe { entry.fDstRect.assume_init() }
                };
                bounds.joinPossiblyEmptyRect(&r);
            }
        }
        if bounds_valid
            && unsafe { canvas.as_ref() }.internalQuickReject(&bounds, &real_paint, None)?
        {
            return Ok(());
        }
        let coverage =
            unsafe { Self::top_device_raw(canvas).as_ref() }.useDrawCoverageMaskForMaskFilters();
        if real_paint.getMaskFilter().is_some() && coverage {
            let mut clip_index = 0;
            for entry in images {
                let mut image_paint = real_paint.clone();
                let (draw_rect, shader) =
                    crate::src::shaders::SkImageShader::SkImageShader::MakeForDrawRect(
                        entry
                            .fImage
                            .as_ref()
                            .expect("native image-set entry owns image"),
                        &image_paint,
                        sampling,
                        unsafe { entry.fSrcRect.assume_init() },
                        unsafe { entry.fDstRect.assume_init() },
                        constraint == SrcRectConstraint::kStrict_SrcRectConstraint,
                    );
                let Some(shader) = shader.filter(|_| !draw_rect.isEmpty()) else {
                    return Ok(());
                };
                image_paint.setShader(Some(shader));
                if let Some(mut layer) = unsafe {
                    Self::aboutToDraw_default_raw(canvas, &image_paint, Some(&draw_rect))
                }? {
                    {
                        let (canvas, paint) = layer.canvas_and_paint();
                        let mut device = unsafe { Self::top_device_raw(canvas) };
                        if entry.fMatrixIndex >= 0 {
                            let local = unsafe { device.as_ref() }
                                .base()
                                .local_to_device44()
                                .clone()
                                * crate::src::core::SkM44::SkM44::from(
                                    &matrices[entry.fMatrixIndex as usize],
                                );
                            unsafe { device.as_mut().base_mut() }.set_local_to_device(&local);
                        }
                        let clip = if entry.fHasClip {
                            Some(
                                <&[crate::src::core::SkPoint::SkPoint; 4]>::try_from(
                                    &clips[clip_index..clip_index + 4],
                                )
                                .expect("native four-point clip"),
                            )
                        } else {
                            None
                        };
                        unsafe { device.as_mut() }.drawEdgeAAQuad(
                            &draw_rect,
                            clip,
                            QuadAAFlags(entry.fAAFlags),
                            &paint.getColor4f(),
                            crate::src::core::SkBlendMode::SkBlendMode::kSrcOver,
                        )?;
                    }
                    unsafe { layer.finish() }?;
                }
                clip_index += 4 * usize::from(entry.fHasClip);
            }
            return Ok(());
        }
        if let Some(mut layer) = unsafe {
            Self::aboutToDraw_default_raw(
                canvas,
                &real_paint,
                if bounds_valid { Some(&bounds) } else { None },
            )
        }? {
            {
                let (canvas, paint) = layer.canvas_and_paint();
                unsafe {
                    Self::top_device_raw(canvas).as_mut().drawEdgeAAImageSet(
                        images,
                        clips,
                        matrices,
                        &real_sampling,
                        paint,
                        constraint,
                    )
                }?;
            }
            unsafe { layer.finish() }?;
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:2832-2880
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1247-1247
    pub unsafe fn drawColor_raw(
        canvas: core::ptr::NonNull<Self>,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        let mut paint = crate::src::core::SkPaint::SkPaint::default();
        paint.setColor4f(color, None);
        paint.setBlendMode(mode);
        unsafe {
            Self::drawPaint_raw(canvas, &paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1348-1348
    pub unsafe fn drawPoint_raw(
        canvas: core::ptr::NonNull<Self>,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let points = [crate::src::core::SkPoint::SkPoint::make(x, y)];
        unsafe {
            Self::drawPoints_raw(canvas, PointMode::kPoints_PointMode, &points, paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1378-1378
    pub unsafe fn drawLine_raw(
        canvas: core::ptr::NonNull<Self>,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let points = [
            crate::src::core::SkPoint::SkPoint::make(x0, y0),
            crate::src::core::SkPoint::SkPoint::make(x1, y1),
        ];
        unsafe {
            Self::drawPoints_raw(canvas, PointMode::kLines_PointMode, &points, paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1489-1489
    pub unsafe fn drawCircle_raw(
        canvas: core::ptr::NonNull<Self>,
        cx: f32,
        cy: f32,
        mut radius: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if radius < 0.0 {
            radius = 0.0;
        }
        let rect = crate::src::core::SkRect::SkRect::MakeLTRB(
            cx - radius,
            cy - radius,
            cx + radius,
            cy + radius,
        );
        unsafe {
            Self::drawOval_raw(canvas, &rect, paint)?;
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1565-1565
    pub unsafe fn drawRoundRect_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        rx: f32,
        ry: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if rx > 0.0 && ry > 0.0 {
            let mut rrect = crate::src::core::SkRRect::SkRRect::default();
            rrect.setRectXY(rect, rx, ry);
            unsafe {
                Self::drawRRect_raw(canvas, &rrect, paint)?;
            }
        } else {
            unsafe {
                Self::drawRect_raw(canvas, rect, paint)?;
            }
        }
        Ok(())
    }
    /// # Safety
    /// Preserve the pinned Canvas and native pixel/filter lease through draw/restore.
    // Source: include/core/SkCanvas.h:1524-1525
    pub unsafe fn drawArc_raw(
        canvas: core::ptr::NonNull<Self>,
        oval: &crate::src::core::SkRect::SkRect,
        start_angle: f32,
        sweep_angle: f32,
        use_center: bool,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        if oval.isEmpty() || sweep_angle == 0.0 {
            return Ok(());
        }
        unsafe {
            Self::onDrawArc_raw(canvas, oval, start_angle, sweep_angle, use_center, paint)?;
        }
        Ok(())
    }
}

// Source: src/core/SkCanvas.cpp:2884-2884
#[cfg(sk_disable_skpicture)]
impl<'device> SkCanvas<'device> {
    pub unsafe fn drawPicture_raw(
        _canvas: core::ptr::NonNull<Self>,
        _picture: Option<&dyn crate::src::core::SkPicture::SkPicture>,
        _matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        _paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2887-2888
#[cfg(sk_disable_skpicture)]
impl<'device> SkCanvas<'device> {
    unsafe fn onDrawPicture_base_raw(
        _canvas: core::ptr::NonNull<Self>,
        _picture: &dyn crate::src::core::SkPicture::SkPicture,
        _matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        _paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        Ok(())
    }
}
// Source: src/core/SkCanvas.cpp:2891-2904
// Source: include/core/SkCanvas.h:2044-2044
#[cfg(not(sk_disable_skpicture))]
impl<'device> SkCanvas<'device> {
    /// Safety: retain the actual pinned Canvas and Picture owners, with no whole
    /// Canvas reference spanning playback/COW callbacks; preserve pixel leases.
    pub unsafe fn drawPicture_raw(
        canvas: core::ptr::NonNull<Self>,
        picture: Option<&dyn crate::src::core::SkPicture::SkPicture>,
        mut matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let Some(picture) = picture else {
            return Ok(());
        };
        if matrix.is_some_and(|m| m.isIdentity()) {
            matrix = None;
        }
        if picture.approximateOpCount(false)
            <= crate::src::core::SkCanvasPriv::kMaxPictureOpsToUnrollInsteadOfRef
        {
            let bounds = picture.cullRect();
            let acmp = unsafe {
                crate::src::core::SkCanvasPriv::SkAutoCanvasMatrixPaint::new(
                    canvas, matrix, paint, &bounds,
                )
            }?;
            unsafe { picture.playback(canvas, None) }?;
            unsafe { acmp.finish() }
        } else {
            unsafe { Self::onDrawPicture_raw(canvas, picture, matrix, paint) }
        }
    }
}
// Source: src/core/SkCanvas.cpp:2906-2914
#[cfg(not(sk_disable_skpicture))]
impl<'device> SkCanvas<'device> {
    unsafe fn onDrawPicture_base_raw(
        canvas: core::ptr::NonNull<Self>,
        picture: &dyn crate::src::core::SkPicture::SkPicture,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let bounds = picture.cullRect();
        let fallback = crate::src::core::SkPaint::SkPaint::default();
        if unsafe { canvas.as_ref() }.internalQuickReject(
            &bounds,
            paint.unwrap_or(&fallback),
            matrix,
        )? {
            return Ok(());
        }
        let acmp = unsafe {
            crate::src::core::SkCanvasPriv::SkAutoCanvasMatrixPaint::new(
                canvas, matrix, paint, &bounds,
            )
        }?;
        unsafe { picture.playback(canvas, None) }?;
        unsafe { acmp.finish() }
    }
}

// Source: include/core/SkCanvas.h:1731-1731
// Source: src/core/SkCanvas.cpp:2919-2919
impl Default for ImageSetEntry {
    fn default() -> Self {
        Self {
            fImage: None,
            fSrcRect: core::mem::MaybeUninit::uninit(),
            fDstRect: core::mem::MaybeUninit::uninit(),
            fMatrixIndex: -1,
            fAlpha: 1.0,
            fAAFlags: QuadAAFlags::kNone_QuadAAFlags.0,
            fHasClip: false,
        }
    }
}
// Copies of rectangle storage below use MaybeUninit's Copy operation; they do
// not read an indeterminate rectangle or introduce a second initialization flag.
impl Clone for ImageSetEntry {
    // Source: include/core/SkCanvas.h:1733-1733
    // Source: src/core/SkCanvas.cpp:2921-2921
    // Adapted: canonical Arc retains the actual image owner, not a proxy image.
    fn clone(&self) -> Self {
        Self {
            fImage: self.fImage.clone(),
            fSrcRect: self.fSrcRect,
            fDstRect: self.fDstRect,
            fMatrixIndex: self.fMatrixIndex,
            fAlpha: self.fAlpha,
            fAAFlags: self.fAAFlags,
            fHasClip: self.fHasClip,
        }
    }
    // Source: include/core/SkCanvas.h:1734-1734
    // Source: src/core/SkCanvas.cpp:2922-2922
    fn clone_from(&mut self, source: &Self) {
        self.fImage.clone_from(&source.fImage);
        self.fSrcRect = source.fSrcRect;
        self.fDstRect = source.fDstRect;
        self.fMatrixIndex = source.fMatrixIndex;
        self.fAlpha = source.fAlpha;
        self.fAAFlags = source.fAAFlags;
        self.fHasClip = source.fHasClip;
    }
}

// Source: src/core/SkCanvas.cpp:2924-2933
// Source: include/core/SkCanvas.h:1725-1726
impl ImageSetEntry {
    pub fn new(
        image: Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        srcRect: &crate::src::core::SkRect::SkRect,
        dstRect: &crate::src::core::SkRect::SkRect,
        matrixIndex: i32,
        alpha: f32,
        aaFlags: u32,
        hasClip: bool,
    ) -> Self {
        Self {
            fImage: image,
            fSrcRect: core::mem::MaybeUninit::new(*srcRect),
            fDstRect: core::mem::MaybeUninit::new(*dstRect),
            fMatrixIndex: matrixIndex,
            fAlpha: alpha,
            fAAFlags: aaFlags,
            fHasClip: hasClip,
        }
    }
}

// Source: src/core/SkCanvas.cpp:2935-2941
// Source: include/core/SkCanvas.h:1728-1729
impl ImageSetEntry {
    pub fn without_matrix(
        image: Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        srcRect: &crate::src::core::SkRect::SkRect,
        dstRect: &crate::src::core::SkRect::SkRect,
        alpha: f32,
        aaFlags: u32,
    ) -> Self {
        Self {
            fImage: image,
            fSrcRect: core::mem::MaybeUninit::new(*srcRect),
            fDstRect: core::mem::MaybeUninit::new(*dstRect),
            fMatrixIndex: -1,
            fAlpha: alpha,
            fAAFlags: aaFlags,
            fHasClip: false,
        }
    }
}

// Source: src/core/SkCanvas.cpp:2945-2959
impl<'device> SkCanvas<'device> {
    /// Safety: caller retains the actual external allocation, initialized for
    /// its declared native layout and exclusively leased until Canvas drops.
    /// Installing the bitmap retains no replacement owner and frees no pixels.
    // Source: include/core/SkCanvas.h:144-146
    pub unsafe fn MakeRasterDirect(
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        pixels: *mut core::ffi::c_void,
        row_bytes: usize,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> Option<core::pin::Pin<Box<Self>>> {
        if !crate::src::core::SkSurfacePriv::SkSurfaceValidateRasterInfo(info, row_bytes) {
            return None;
        }
        let mut bitmap = crate::src::core::SkBitmap::SkBitmap::default();
        if !unsafe { bitmap.installPixels(info, pixels, row_bytes, None, core::ptr::null_mut()) } {
            return None;
        }
        Some(match props {
            Some(props) => unsafe { Self::with_bitmap_props(&bitmap, props) },
            None => unsafe { Self::with_bitmap(&bitmap) },
        })
    }
}

// Source: src/core/SkCanvas.cpp:2963-2964
impl<'device> crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas<'device> {
    pub fn new(width: i32, height: i32) -> core::pin::Pin<Box<Self>> {
        Self::with_bounds(&crate::src::core::SkRect::SkIRect::make_wh(width, height))
    }
}
// Source: src/core/SkCanvas.cpp:2966-2967
impl<'device> crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas<'device> {
    pub fn with_bounds(bounds: &crate::src::core::SkRect::SkIRect) -> core::pin::Pin<Box<Self>> {
        Self::from_inherited(SkCanvas::with_bounds(bounds))
    }
}

// Source: src/core/SkCanvas.cpp:2969-2972
impl crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas<'_> {
    pub(crate) fn getSaveLayerStrategy_impl(&self, rec: &SaveLayerRec<'_>) -> SaveLayerStrategy {
        let _ = self.inherited().inherited_getSaveLayerStrategy(rec);
        SaveLayerStrategy::kNoLayer_SaveLayerStrategy
    }
}
// Source: src/core/SkCanvas.cpp:2974-2976
impl crate::include::utils::SkNoDrawCanvas::SkNoDrawCanvas<'_> {
    pub(crate) fn onDoSaveBehind_impl(&self, _: Option<&crate::src::core::SkRect::SkRect>) -> bool {
        false
    }
}

// Source: src/core/SkCanvas.cpp:2980-2980
const _: () = assert!(
    crate::src::core::SkRegion::Op::Difference as i32
        == crate::include::core::SkClipOp::SkClipOp::kDifference as i32
);
// Source: src/core/SkCanvas.cpp:2981-2981
const _: () = assert!(
    crate::src::core::SkRegion::Op::Intersect as i32
        == crate::include::core::SkClipOp::SkClipOp::kIntersect as i32
);

// Source: src/core/SkCanvas.cpp:2985-2998
impl<'device> SkCanvas<'device> {
    /// # Safety
    /// Preserve the pinned Canvas/exclusive device lease and keep the installed
    /// allocator's handle valid until the corresponding bitmap release callback.
    // Source: include/core/SkCanvas.h:379-379
    pub unsafe fn accessTopRasterHandle(
        self: core::pin::Pin<&mut Self>,
    ) -> crate::include::core::SkRasterHandleAllocator::Handle {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            this.rebind_stack();
        }
        let device = this.topDevice();
        if let Some(allocator) = this.fAllocator.as_mut() {
            let handle = unsafe { device.as_ref() }.getRasterHandle();
            let mut clip = unsafe { device.as_ref() }.devClipBounds();
            let base = unsafe { device.as_ref() }.base();
            if !clip.intersect(&crate::src::core::SkRect::SkIRect::make_wh(
                base.width(),
                base.height(),
            )) {
                clip.set_empty();
            }
            unsafe {
                allocator.updateHandle(handle, base.local_to_device(), &clip);
            }
            handle
        } else {
            core::ptr::null_mut()
        }
    }
}

// Source: src/core/SkCanvas.cpp:3000-3003
unsafe fn install_raster_handle(
    bm: &mut crate::src::core::SkBitmap::SkBitmap,
    info: &crate::src::core::SkImageInfo::SkImageInfo,
    rec: &crate::include::core::SkRasterHandleAllocator::Rec,
) -> bool {
    unsafe {
        bm.installPixels(
            info,
            rec.fPixels,
            rec.fRowBytes,
            rec.fReleaseProc,
            rec.fReleaseCtx,
        )
    }
}
// Source: src/core/SkCanvas.cpp:3005-3012
impl dyn crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator + '_ {
    /// # Safety
    /// Retain the installed bitmap's pixel lease and native handle until the
    /// backing's release callback. No other alias may access its pixels meanwhile.
    // Source: include/core/SkRasterHandleAllocator.h:91-91
    pub(crate) unsafe fn allocBitmap(
        &mut self,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        bm: &mut crate::src::core::SkBitmap::SkBitmap,
    ) -> crate::include::core::SkRasterHandleAllocator::Handle {
        let mut rec = core::mem::MaybeUninit::uninit();
        if !self.allocHandle(info, &mut rec) {
            return core::ptr::null_mut();
        }
        // The unsafe allocator implementation guarantees full initialization on true.
        let rec = unsafe { rec.assume_init() };
        if !unsafe { install_raster_handle(bm, info, &rec) } {
            return core::ptr::null_mut();
        }
        rec.fHandle
    }
}
// Source: src/core/SkCanvas.cpp:3014-3032
impl dyn crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator + '_ {
    /// # Safety
    /// The allocator/optional initial Rec must establish the same exclusive live
    /// pixel/release lease as Canvas::with_bitmap_allocator for the whole Canvas
    /// lifetime. A supplied Rec transfers its release obligation on installation.
    // Source: include/core/SkRasterHandleAllocator.h:79-81
    pub unsafe fn MakeCanvas<'device>(
        allocator: Option<
            Box<dyn crate::include::core::SkRasterHandleAllocator::SkRasterHandleAllocator>,
        >,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        rec: Option<&crate::include::core::SkRasterHandleAllocator::Rec>,
        props: Option<&crate::include::core::SkSurfaceProps::SkSurfaceProps>,
    ) -> Option<core::pin::Pin<Box<SkCanvas<'device>>>> {
        let Some(mut allocator) = allocator else {
            return None;
        };
        let row_bytes = rec.map_or(crate::src::core::SkSurfacePriv::kIgnoreRowBytesValue, |r| {
            r.fRowBytes
        });
        // Original native validator is owned by image's SkSurface_Raster.cpp39.
        // No metadata-only substitute is supplied while that actual file is pending.
        if !crate::src::image::SkSurface_Raster::SkSurfaceValidateRasterInfo(info, row_bytes) {
            return None;
        }
        let mut bitmap = crate::src::core::SkBitmap::SkBitmap::default();
        let handle = if let Some(rec) = rec {
            if unsafe { install_raster_handle(&mut bitmap, info, rec) } {
                rec.fHandle
            } else {
                core::ptr::null_mut()
            }
        } else {
            unsafe { allocator.allocBitmap(info, &mut bitmap) }
        };
        if !handle.is_null() {
            Some(unsafe {
                SkCanvas::with_bitmap_allocator(&bitmap, Some(allocator), handle, props)
            })
        } else {
            None
        }
    }
}

// Source: include/core/SkCanvas.h:2731-2738
impl<'canvas, 'device> SkAutoCanvasRestore<'canvas, 'device> {
    /// Safety: canvas is the original pinned allocation under an exclusive
    /// Canvas/device/pixel lease until explicit restoration or cancellation.
    pub unsafe fn new(canvas: Option<core::ptr::NonNull<SkCanvas<'device>>>, doSave: bool) -> Self {
        let mut result = Self {
            fCanvas: canvas,
            fSaveCount: 0,
            lease: core::marker::PhantomData,
        };
        if let Some(canvas) = result.fCanvas {
            result.fSaveCount = unsafe { canvas.as_ref() }.getSaveCount();
            if doSave {
                unsafe { core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).save() };
            }
        }
        result
    }
}
// Source: include/core/SkCanvas.h:2743-2747
impl SkAutoCanvasRestore<'_, '_> {
    /// Explicit native destructor outlet. An actual RasterError propagates;
    /// dropping after that error only cancels uncommitted remaining state.
    /// Safety: preserve the original Canvas/device/pixel operation lease.
    pub unsafe fn finish(mut self) -> Result<(), crate::RasterError> {
        unsafe { self.restore() }
    }
}
// Source: include/core/SkCanvas.h:2752-2757
impl SkAutoCanvasRestore<'_, '_> {
    /// Safety: preserve the original pinned Canvas and exclusive drawing lease.
    pub unsafe fn restore(&mut self) -> Result<(), crate::RasterError> {
        if let Some(canvas) = self.fCanvas {
            unsafe { SkCanvas::restoreToCount_raw(canvas, self.fSaveCount) }?;
            self.fCanvas = None;
        }
        Ok(())
    }
}
// Source: include/core/SkCanvas.h:2760-2761
// Native Canvas/clip/CTM ownership stays on the original pinned allocation.
// Rust normal destruction is explicit and fallible; Drop is error-exit rollback.
#[must_use = "call finish or restore to perform the native fallible restoration"]
pub struct SkAutoCanvasRestore<'canvas, 'device> {
    fCanvas: Option<core::ptr::NonNull<SkCanvas<'device>>>,
    fSaveCount: i32,
    lease: core::marker::PhantomData<&'canvas mut SkCanvas<'device>>,
}
// This Drop does NOT implement native painting restoration. The normal outlet
// above must be called explicitly. It cancels state only on abandoned/error exits,
// reusing the same deferred-save/MCRec cleanup without swallowing a drawing error.
impl Drop for SkAutoCanvasRestore<'_, '_> {
    fn drop(&mut self) {
        if let Some(canvas) = self.fCanvas.take() {
            unsafe { SkCanvas::cancel_matrix_paint_raw(canvas, self.fSaveCount) };
        }
    }
}

// Remaining fixed-file native dependencies (no substitute owners implemented):
// include/core/SkCanvas.h:2014-2060; src/core/SkCanvas.cpp:2891-2914:
//   SkPicture cullRect/approximateOpCount/playback and real Picture owner.
// include/core/SkCanvas.h:2081-2185,2357-2378;
// src/core/SkCanvas.cpp:1740-1769,1835-1849,2602-2714:
//   actual SkVertices/SkMesh owners and Device virtuals; Patch additionally
//   needs SkPatchUtils::GetLevelOfDetail/MakeVertices. Atlas cannot bypass
//   the native Vertices::Builder producer with a replacement polygon engine.
// include/core/SkCanvas.h:2301,2380; src/core/SkCanvas.cpp:1859-1871:
//   actual SkDrawShadowRec and SkDevice::drawShadow producer.
// include/core/SkCanvas.h:2682 requires real SkPaintFilterCanvas projection.

// Native virtual-call bridge: hooks borrow the real pinned derived owner.
impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2341-2341
    pub(crate) unsafe fn onDrawPaint_raw(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawPaint(
                    hooks.as_ref(),
                    canvas,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawPaint_base_raw(canvas, paint) }
    }
    // Source: include/core/SkCanvas.h:2342-2342
    pub(crate) unsafe fn onDrawBehind_raw(
        canvas: core::ptr::NonNull<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawBehind(
                    hooks.as_ref(),
                    canvas,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawBehind_base_raw(canvas, paint) }
    }
    // Source: include/core/SkCanvas.h:2359-2360
    pub(crate) unsafe fn onDrawPoints_raw(
        canvas: core::ptr::NonNull<Self>,
        mode: PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawPoints(
                    hooks.as_ref(),
                    canvas,
                    mode,
                    points,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawPoints_base_raw(canvas, mode, points, paint) }
    }
    // Source: include/core/SkCanvas.h:2343-2343
    pub(crate) unsafe fn onDrawRect_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawRect(
                    hooks.as_ref(),
                    canvas,
                    rect,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawRect_base_raw(canvas, rect, paint) }
    }
    // Source: include/core/SkCanvas.h:2350-2350
    pub(crate) unsafe fn onDrawRegion_raw(
        canvas: core::ptr::NonNull<Self>,
        region: &crate::src::core::SkRegion::SkRegion,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawRegion(
                    hooks.as_ref(),
                    canvas,
                    region,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawRegion_base_raw(canvas, region, paint) }
    }
    // Source: include/core/SkCanvas.h:2346-2346
    pub(crate) unsafe fn onDrawOval_raw(
        canvas: core::ptr::NonNull<Self>,
        oval: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawOval(
                    hooks.as_ref(),
                    canvas,
                    oval,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawOval_base_raw(canvas, oval, paint) }
    }
    // Source: include/core/SkCanvas.h:2347-2348
    pub(crate) unsafe fn onDrawArc_raw(
        canvas: core::ptr::NonNull<Self>,
        oval: &crate::src::core::SkRect::SkRect,
        start: f32,
        sweep: f32,
        use_center: bool,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawArc(
                    hooks.as_ref(),
                    canvas,
                    oval,
                    start,
                    sweep,
                    use_center,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawArc_base_raw(canvas, oval, start, sweep, use_center, paint) }
    }
    // Source: include/core/SkCanvas.h:2344-2344
    pub(crate) unsafe fn onDrawRRect_raw(
        canvas: core::ptr::NonNull<Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawRRect(
                    hooks.as_ref(),
                    canvas,
                    rrect,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawRRect_base_raw(canvas, rrect, paint) }
    }
    // Source: include/core/SkCanvas.h:2345-2345
    pub(crate) unsafe fn onDrawDRRect_raw(
        canvas: core::ptr::NonNull<Self>,
        outer: &crate::src::core::SkRRect::SkRRect,
        inner: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawDRRect(
                    hooks.as_ref(),
                    canvas,
                    outer,
                    inner,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawDRRect_base_raw(canvas, outer, inner, paint) }
    }
    // Source: include/core/SkCanvas.h:2349-2349
    pub(crate) unsafe fn onDrawPath_raw(
        canvas: core::ptr::NonNull<Self>,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawPath(
                    hooks.as_ref(),
                    canvas,
                    path,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawPath_base_raw(canvas, path, paint) }
    }
    // Source: include/core/SkCanvas.h:2362-2363
    pub(crate) unsafe fn onDrawImage2_raw(
        _canvas: core::ptr::NonNull<Self>,
        _image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        _x: f32,
        _y: f32,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) {
        let hooks = unsafe { core::ptr::addr_of!((*_canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtual::onDrawImage2(
                    hooks.as_ref(),
                    _canvas,
                    _image,
                    _x,
                    _y,
                    _sampling,
                    _paint,
                )
            };
        }
        unsafe { Self::onDrawImage2_base_raw(_canvas, _image, _x, _y, _sampling, _paint) }
    }
    // Source: include/core/SkCanvas.h:2364-2365
    pub(crate) unsafe fn onDrawImageRect2_raw(
        canvas: core::ptr::NonNull<Self>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        src: &crate::src::core::SkRect::SkRect,
        dst: &crate::src::core::SkRect::SkRect,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtual::onDrawImageRect2(
                    hooks.as_ref(),
                    canvas,
                    image,
                    src,
                    dst,
                    sampling,
                    paint,
                    constraint,
                )
            };
        }
        unsafe {
            Self::onDrawImageRect2_base_raw(canvas, image, src, dst, sampling, paint, constraint)
        }
    }
    // Source: include/core/SkCanvas.h:2366-2367
    pub(crate) unsafe fn onDrawImageLattice2_raw(
        canvas: core::ptr::NonNull<Self>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        lattice: &Lattice,
        dst: &crate::src::core::SkRect::SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtual::onDrawImageLattice2(
                    hooks.as_ref(),
                    canvas,
                    image,
                    lattice,
                    dst,
                    filter,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawImageLattice2_base_raw(canvas, image, lattice, dst, filter, paint) }
    }
    // Source: include/core/SkCanvas.h:2352-2353
    pub unsafe fn onDrawTextBlob_raw(
        canvas: core::ptr::NonNull<Self>,
        blob: &crate::src::core::SkTextBlob::SkTextBlob,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawTextBlob(hooks.as_ref(), canvas, blob, x, y, paint)
            };
        }
        unsafe { Self::onDrawTextBlob_base_raw(canvas, blob, x, y, paint) }
    }
    // Source: include/core/SkCanvas.h:2355-2355
    pub unsafe fn onDrawGlyphRunList_raw(
        canvas: core::ptr::NonNull<Self>,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtual::onDrawGlyphRunList(
                    hooks.as_ref(),
                    canvas,
                    list,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawGlyphRunList_base_raw(canvas, list, paint) }
    }
    // Source: include/core/SkCanvas.h:2382-2382
    pub(crate) unsafe fn onDrawDrawable_raw(
        canvas: core::ptr::NonNull<Self>,
        drawable: &dyn crate::src::core::SkDrawable::SkDrawable,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawDrawable(hooks.as_ref(), canvas, drawable, matrix)
            };
        }
        unsafe { Self::onDrawDrawable_base_raw(canvas, drawable, matrix) }
    }
    // Source: include/core/SkCanvas.h:2379-2379
    pub(crate) unsafe fn onDrawAnnotation_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        key: &std::ffi::CStr,
        value: Option<&std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawAnnotation(hooks.as_ref(), canvas, rect, key, value)
            };
        }
        unsafe { Self::onDrawAnnotation_base_raw(canvas, rect, key, value) }
    }
    // Source: include/core/SkCanvas.h:2386-2387
    pub(crate) unsafe fn onDrawEdgeAAQuad_raw(
        canvas: core::ptr::NonNull<Self>,
        r: &crate::src::core::SkRect::SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        aa: QuadAAFlags,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawEdgeAAQuad(hooks.as_ref(), canvas, r, clip, aa, color, mode)
            };
        }
        unsafe { Self::onDrawEdgeAAQuad_base_raw(canvas, r, clip, aa, color, mode) }
    }
    // Source: include/core/SkCanvas.h:2371-2374
    pub(crate) unsafe fn onDrawEdgeAAImageSet2_raw(
        canvas: core::ptr::NonNull<Self>,
        images: &[ImageSetEntry],
        clips: &[crate::src::core::SkPoint::SkPoint],
        matrices: &[crate::src::core::SkMatrix::SkMatrix],
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        constraint: SrcRectConstraint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtual::onDrawEdgeAAImageSet2(hooks.as_ref(), canvas, images, clips, matrices, sampling, paint, constraint)
            };
        }
        unsafe {
            Self::onDrawEdgeAAImageSet2_base_raw(
                canvas, images, clips, matrices, sampling, paint, constraint,
            )
        }
    }
}

impl<'device> SkCanvas<'device> {
    /// Safety: hooks point to this Canvas's actual pinned derived owner and
    /// outlive every base call including destruction. Only that owner installs it.
    pub(crate) unsafe fn install_derived_hooks(
        &mut self,
        hooks: core::ptr::NonNull<
            dyn crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer<'device>
                + 'device,
        >,
    ) {
        self.fVirtual = Some(hooks);
    }
    pub(crate) fn inherited_getSaveLayerStrategy(&self, _: &SaveLayerRec<'_>) -> SaveLayerStrategy {
        SaveLayerStrategy::kFullLayer_SaveLayerStrategy
    }
}

impl SkCanvas<'_> {
    // Native base destruction uses the base vtable after the derived destructor.
    pub(crate) fn clear_derived_hooks(&mut self) {
        self.fVirtual = None;
    }
}

// Rust virtual dispatch interface for the native inherited SkCanvas hooks.
// Default methods call the already translated qualified base implementations;
// they never substitute an empty draw for an untranslated native algorithm.
pub trait SkCanvasVirtual<'device> {
    // Source: include/core/SkCanvas.h:2546-2546
    /// Safety: complete pinned owner survives this deletion notification; the
    /// Surface link has already been cleared. No whole Canvas lease spans it.
    unsafe fn onSurfaceDelete(&self, canvas: core::ptr::NonNull<SkCanvas<'device>>) {
        unsafe {
            SkCanvas::on_surface_delete_base_raw(canvas);
        }
    }

    // Source: include/core/SkCanvas.h:2682-2682
    unsafe fn internal_private_asPaintFilterCanvas(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
    ) -> *mut SkPaintFilterCanvas {
        unsafe { SkCanvas::internal_private_asPaintFilterCanvas_base_raw(canvas) }
    }

    // Source: include/core/SkCanvas.h:2306-2306
    /// Safety: query-only shared Canvas lease; no base mutation or pixel invalidation.
    unsafe fn onNewSurface(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::image::SkSurface::SkSurface<'static>>
    {
        unsafe { SkCanvas::onNewSurface_base_raw(canvas, info, props) }
    }
    // Source: include/core/SkCanvas.h:2309-2309
    /// Safety: query-only shared Canvas lease; retain native pixel backing through use.
    unsafe fn onPeekPixels(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        unsafe { SkCanvas::onPeekPixels_base_raw(canvas, pmap) }
    }

    // Source: include/core/SkCanvas.h:2310-2310
    unsafe fn onAccessTopLayerPixels(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        unsafe { SkCanvas::onAccessTopLayerPixels_base_raw(canvas, pmap) }
    }
    // Source: include/core/SkCanvas.h:2311-2311
    unsafe fn onImageInfo(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
    ) -> crate::src::core::SkImageInfo::SkImageInfo {
        unsafe { SkCanvas::onImageInfo_base_raw(canvas) }
    }
    // Source: include/core/SkCanvas.h:2312-2312
    unsafe fn onGetProps(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        props: Option<&mut crate::include::core::SkSurfaceProps::SkSurfaceProps>,
        top: bool,
    ) -> bool {
        unsafe { SkCanvas::onGetProps_base_raw(canvas, props, top) }
    }
    // Source: include/core/SkCanvas.h:2322-2322
    unsafe fn willSave(&self, canvas: core::ptr::NonNull<SkCanvas<'device>>) {
        unsafe {
            SkCanvas::willSave_base_raw(canvas);
        }
    }
    // Source: include/core/SkCanvas.h:2324-2326
    fn getSaveLayerStrategy(
        &self,
        _: &crate::src::core::SkCanvas::SaveLayerRec<'_>,
    ) -> crate::src::core::SkCanvas::SaveLayerStrategy {
        crate::src::core::SkCanvas::SaveLayerStrategy::kFullLayer_SaveLayerStrategy
    }
    // Source: include/core/SkCanvas.h:2329-2329
    fn onDoSaveBehind(&self, _: Option<&crate::src::core::SkRect::SkRect>) -> bool {
        true
    }
    // Source: include/core/SkCanvas.h:2330-2330
    unsafe fn willRestore(&self, canvas: core::ptr::NonNull<SkCanvas<'device>>) {
        unsafe {
            SkCanvas::willRestore_base_raw(canvas);
        }
    }
    // Source: include/core/SkCanvas.h:2331-2331
    unsafe fn didRestore(&self, canvas: core::ptr::NonNull<SkCanvas<'device>>) {
        unsafe {
            SkCanvas::didRestore_base_raw(canvas);
        }
    }
    // Source: include/core/SkCanvas.h:2333-2333
    unsafe fn didConcat44(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        m: &crate::src::core::SkM44::SkM44,
    ) {
        unsafe {
            SkCanvas::didConcat44_base_raw(canvas, m);
        }
    }
    // Source: include/core/SkCanvas.h:2334-2334
    unsafe fn didSetM44(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        m: &crate::src::core::SkM44::SkM44,
    ) {
        unsafe {
            SkCanvas::didSetM44_base_raw(canvas, m);
        }
    }
    // Source: include/core/SkCanvas.h:2335-2335
    unsafe fn didTranslate(&self, canvas: core::ptr::NonNull<SkCanvas<'device>>, dx: f32, dy: f32) {
        unsafe {
            SkCanvas::didTranslate_base_raw(canvas, dx, dy);
        }
    }
    // Source: include/core/SkCanvas.h:2336-2336
    unsafe fn didScale(&self, canvas: core::ptr::NonNull<SkCanvas<'device>>, sx: f32, sy: f32) {
        unsafe {
            SkCanvas::didScale_base_raw(canvas, sx, sy);
        }
    }
    // Source: include/core/SkCanvas.h:2341-2341
    unsafe fn onDrawPaint(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawPaint_base_raw(canvas, paint) }
    }
    // Source: include/core/SkCanvas.h:2342-2342
    unsafe fn onDrawBehind(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawBehind_base_raw(canvas, paint) }
    }
    // Source: include/core/SkCanvas.h:2343-2343
    unsafe fn onDrawRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawRect_base_raw(canvas, rect, paint) }
    }
    // Source: include/core/SkCanvas.h:2344-2344
    unsafe fn onDrawRRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawRRect_base_raw(canvas, rrect, paint) }
    }
    // Source: include/core/SkCanvas.h:2345-2345
    unsafe fn onDrawDRRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        outer: &crate::src::core::SkRRect::SkRRect,
        inner: &crate::src::core::SkRRect::SkRRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawDRRect_base_raw(canvas, outer, inner, paint) }
    }
    // Source: include/core/SkCanvas.h:2346-2346
    unsafe fn onDrawOval(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        oval: &crate::src::core::SkRect::SkRect,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawOval_base_raw(canvas, oval, paint) }
    }
    // Source: include/core/SkCanvas.h:2347-2348
    unsafe fn onDrawArc(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        oval: &crate::src::core::SkRect::SkRect,
        start: f32,
        sweep: f32,
        use_center: bool,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawArc_base_raw(canvas, oval, start, sweep, use_center, paint) }
    }
    // Source: include/core/SkCanvas.h:2349-2349
    unsafe fn onDrawPath(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        path: &crate::src::core::SkPath::SkPath,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawPath_base_raw(canvas, path, paint) }
    }
    // Source: include/core/SkCanvas.h:2350-2350
    unsafe fn onDrawRegion(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        region: &crate::src::core::SkRegion::SkRegion,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawRegion_base_raw(canvas, region, paint) }
    }
    // Source: include/core/SkCanvas.h:2352-2353
    unsafe fn onDrawTextBlob(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        blob: &crate::src::core::SkTextBlob::SkTextBlob,
        x: f32,
        y: f32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawTextBlob_base_raw(canvas, blob, x, y, paint) }
    }
    // Source: include/core/SkCanvas.h:2355-2355
    unsafe fn onDrawGlyphRunList(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawGlyphRunList_base_raw(canvas, list, paint) }
    }
    // Source: include/core/SkCanvas.h:2357-2358
    unsafe fn onDrawPatch(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        cubics: &[crate::src::core::SkPoint::SkPoint; 12],
        colors: Option<&[crate::src::core::SkColor::SkColor; 4]>,
        tex: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawPatch_base_raw(canvas, cubics, colors, tex, mode, paint) }
    }
    // Source: include/core/SkCanvas.h:2359-2360
    unsafe fn onDrawPoints(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        mode: PointMode,
        points: &[crate::src::core::SkPoint::SkPoint],
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawPoints_base_raw(canvas, mode, points, paint) }
    }
    // Source: include/core/SkCanvas.h:2362-2363
    unsafe fn onDrawImage2(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        _image: Option<&std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
        _x: f32,
        _y: f32,
        _sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) {
        unsafe { SkCanvas::onDrawImage2_base_raw(canvas, _image, _x, _y, _sampling, _paint) }
    }
    // Source: include/core/SkCanvas.h:2364-2365
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
            SkCanvas::onDrawImageRect2_base_raw(
                canvas, image, src, dst, sampling, paint, constraint,
            )
        }
    }
    // Source: include/core/SkCanvas.h:2366-2367
    unsafe fn onDrawImageLattice2(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        image: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        lattice: &Lattice,
        dst: &crate::src::core::SkRect::SkRect,
        filter: crate::include::core::SkSamplingOptions::SkFilterMode,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkCanvas::onDrawImageLattice2_base_raw(canvas, image, lattice, dst, filter, paint)
        }
    }
    // Source: include/core/SkCanvas.h:2368-2370
    unsafe fn onDrawAtlas2(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        atlas: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[crate::src::core::SkRect::SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        cull: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            SkCanvas::onDrawAtlas2_base_raw(
                canvas, atlas, xform, tex, colors, mode, sampling, cull, paint,
            )
        }
    }
    // Source: include/core/SkCanvas.h:2371-2374
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
            SkCanvas::onDrawEdgeAAImageSet2_base_raw(
                canvas, images, clips, matrices, sampling, paint, constraint,
            )
        }
    } // Source: include/core/SkCanvas.h:2376-2377
    unsafe fn onDrawVerticesObject(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        vertices: &crate::src::core::SkVertices::SkVertices,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawVerticesObject_base_raw(canvas, vertices, mode, paint) }
    }
    // Source: include/core/SkCanvas.h:2378-2378
    unsafe fn onDrawMesh(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        mesh: &crate::src::core::SkMesh::SkMesh,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawMesh_base_raw(canvas, mesh, blender, paint) }
    }

    // Source: include/core/SkCanvas.h:2379-2379
    unsafe fn onDrawAnnotation(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        key: &std::ffi::CStr,
        value: Option<&std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) {
        unsafe { SkCanvas::onDrawAnnotation_base_raw(canvas, rect, key, value) }
    }
    // Source: include/core/SkCanvas.h:2380-2380
    unsafe fn onDrawShadowRec(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        path: &crate::src::core::SkPath::SkPath,
        rec: &crate::src::core::SkDrawShadowInfo::SkDrawShadowRec,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawShadowRec_base_raw(canvas, path, rec) }
    }

    // Source: include/core/SkCanvas.h:2382-2382
    unsafe fn onDrawDrawable(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        drawable: &dyn crate::src::core::SkDrawable::SkDrawable,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawDrawable_base_raw(canvas, drawable, matrix) }
    }
    // Source: include/core/SkCanvas.h:2383-2384
    unsafe fn onDrawPicture(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        picture: &dyn crate::src::core::SkPicture::SkPicture,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawPicture_base_raw(canvas, picture, matrix, paint) }
    }
    // Source: include/core/SkCanvas.h:2386-2387
    unsafe fn onDrawEdgeAAQuad(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        r: &crate::src::core::SkRect::SkRect,
        clip: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        aa: QuadAAFlags,
        color: &crate::src::core::SkColor::SkColor4f,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawEdgeAAQuad_base_raw(canvas, r, clip, aa, color, mode) }
    } // Source: include/core/SkCanvas.h:2394-2394
    unsafe fn onClipRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            SkCanvas::onClipRect_base_raw(canvas, rect, op, edge);
        }
    }
    // Source: include/core/SkCanvas.h:2395-2395
    unsafe fn onClipRRect(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            SkCanvas::onClipRRect_base_raw(canvas, rrect, op, edge);
        }
    }
    // Source: include/core/SkCanvas.h:2396-2396
    unsafe fn onClipPath(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        path: &crate::src::core::SkPath::SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            SkCanvas::onClipPath_base_raw(canvas, path, op, edge);
        }
    }
    // Source: include/core/SkCanvas.h:2397-2397
    unsafe fn onClipShader(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            SkCanvas::onClipShader_base_raw(canvas, shader, op);
        }
    }
    // Source: include/core/SkCanvas.h:2398-2398
    unsafe fn onClipRegion(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        region: &crate::src::core::SkRegion::SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            SkCanvas::onClipRegion_base_raw(canvas, region, op);
        }
    }
    // Source: include/core/SkCanvas.h:2399-2399
    unsafe fn onResetClip(&self, canvas: core::ptr::NonNull<SkCanvas<'device>>) {
        unsafe {
            SkCanvas::onResetClip_base_raw(canvas);
        }
    }

    // Source: include/core/SkCanvas.h:2401-2401
    unsafe fn onDiscard(&self, canvas: core::ptr::NonNull<SkCanvas<'device>>) {
        unsafe {
            SkCanvas::onDiscard_base_raw(canvas);
        }
    }

    // Source: include/core/SkCanvas.h:2405-2406
    unsafe fn onConvertGlyphRunListToSlug(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<
        crate::include::core::SkRefCnt::sk_sp<dyn crate::include::private::chromium::Slug::Slug>,
        crate::RasterError,
    > {
        unsafe { SkCanvas::onConvertGlyphRunListToSlug_base_raw(canvas, list, paint) }
    }
    // Source: include/core/SkCanvas.h:2410-2410
    unsafe fn onDrawSlug(
        &self,
        canvas: core::ptr::NonNull<SkCanvas<'device>>,
        slug: &dyn crate::include::private::chromium::Slug::Slug,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        unsafe { SkCanvas::onDrawSlug_base_raw(canvas, slug, paint) }
    }
}

// Rust ownership substitution for a native polymorphic unique_ptr<SkCanvas>.
// The handle owns the real concrete object; projecting a base never erases the
// derived destructor, virtual identity, or lifetime.
pub trait SkCanvasOwner<'device> {
    fn canvas(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut SkCanvas<'device>>;
    fn canvas_ref(&self) -> &SkCanvas<'device>;
}
pub type SkCanvasOwnerHandle<'device> = core::pin::Pin<Box<dyn SkCanvasOwner<'device> + 'device>>;
impl<'device> SkCanvasOwner<'device> for SkCanvas<'device> {
    fn canvas(self: core::pin::Pin<&mut Self>) -> core::pin::Pin<&mut SkCanvas<'device>> {
        self
    }
    fn canvas_ref(&self) -> &SkCanvas<'device> {
        self
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2376-2377
    unsafe fn onDrawVerticesObject_raw(
        canvas: core::ptr::NonNull<Self>,
        vertices: &crate::src::core::SkVertices::SkVertices,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                SkCanvasVirtual::onDrawVerticesObject(hooks.as_ref(), canvas, vertices, mode, paint)
            };
        }
        unsafe { Self::onDrawVerticesObject_base_raw(canvas, vertices, mode, paint) }
    }
}

impl<'device> SkCanvas<'device> {
    // Source: include/core/SkCanvas.h:2014-2016
    pub unsafe fn drawPicture_default_raw(
        canvas: core::ptr::NonNull<Self>,
        picture: Option<&dyn crate::src::core::SkPicture::SkPicture>,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::drawPicture_raw(canvas, picture, None, None) }
    }
    // Source: include/core/SkCanvas.h:2026-2028
    pub unsafe fn drawPicture_sp_default_raw(
        canvas: core::ptr::NonNull<Self>,
        picture: &crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::drawPicture_default_raw(canvas, picture.get()) }
    }
    // Source: include/core/SkCanvas.h:2058-2061
    pub unsafe fn drawPicture_sp_raw(
        canvas: core::ptr::NonNull<Self>,
        picture: &crate::include::core::SkRefCnt::sk_sp<dyn crate::src::core::SkPicture::SkPicture>,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        unsafe { Self::drawPicture_raw(canvas, picture.get(), matrix, paint) }
    }
    // Source: include/core/SkCanvas.h:2383-2384
    unsafe fn onDrawPicture_raw(
        canvas: core::ptr::NonNull<Self>,
        picture: &dyn crate::src::core::SkPicture::SkPicture,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { core::ptr::addr_of!((*canvas.as_ptr()).fVirtual).read() };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawPicture(hooks.as_ref(),canvas,picture,matrix,paint)
            };
        }
        unsafe { Self::onDrawPicture_base_raw(canvas, picture, matrix, paint) }
    }
}

// Source: include/core/SkCanvas.h:2394-2394
impl<'device> SkCanvas<'device> {
    unsafe fn onClipRect(
        &mut self,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            Self::onClipRect_raw(core::ptr::NonNull::from(&mut *self), rect, op, edge);
        }
    }
    pub unsafe fn onClipRect_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::onClipRect(hooks.as_ref(), canvas, rect, op, edge);
            }
        } else {
            unsafe {
                Self::onClipRect_base_raw(canvas, rect, op, edge);
            }
        }
    }
    pub unsafe fn onClipRect_base_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: &crate::src::core::SkRect::SkRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            (&mut *canvas.as_ptr()).onClipRect_inherited(rect, op, edge);
        }
    }
}

// Source: include/core/SkCanvas.h:2395-2395
impl<'device> SkCanvas<'device> {
    unsafe fn onClipRRect(
        &mut self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            Self::onClipRRect_raw(core::ptr::NonNull::from(&mut *self), rrect, op, edge);
        }
    }
    pub unsafe fn onClipRRect_raw(
        canvas: core::ptr::NonNull<Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::onClipRRect(hooks.as_ref(), canvas, rrect, op, edge);
            }
        } else {
            unsafe {
                Self::onClipRRect_base_raw(canvas, rrect, op, edge);
            }
        }
    }
    pub unsafe fn onClipRRect_base_raw(
        canvas: core::ptr::NonNull<Self>,
        rrect: &crate::src::core::SkRRect::SkRRect,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            (&mut *canvas.as_ptr()).onClipRRect_inherited(rrect, op, edge);
        }
    }
}

// Source: include/core/SkCanvas.h:2396-2396
impl<'device> SkCanvas<'device> {
    unsafe fn onClipPath(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            Self::onClipPath_raw(core::ptr::NonNull::from(&mut *self), path, op, edge);
        }
    }
    pub unsafe fn onClipPath_raw(
        canvas: core::ptr::NonNull<Self>,
        path: &crate::src::core::SkPath::SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::onClipPath(hooks.as_ref(), canvas, path, op, edge);
            }
        } else {
            unsafe {
                Self::onClipPath_base_raw(canvas, path, op, edge);
            }
        }
    }
    pub unsafe fn onClipPath_base_raw(
        canvas: core::ptr::NonNull<Self>,
        path: &crate::src::core::SkPath::SkPath,
        op: crate::include::core::SkClipOp::SkClipOp,
        edge: ClipEdgeStyle,
    ) {
        unsafe {
            (&mut *canvas.as_ptr()).onClipPath_inherited(path, op, edge);
        }
    }
}

// Source: include/core/SkCanvas.h:2397-2397
impl<'device> SkCanvas<'device> {
    unsafe fn onClipShader(
        &mut self,
        shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            Self::onClipShader_raw(core::ptr::NonNull::from(&mut *self), shader, op);
        }
    }
    pub unsafe fn onClipShader_raw(
        canvas: core::ptr::NonNull<Self>,
        shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::onClipShader(hooks.as_ref(), canvas, shader, op);
            }
        } else {
            unsafe {
                Self::onClipShader_base_raw(canvas, shader, op);
            }
        }
    }
    pub unsafe fn onClipShader_base_raw(
        canvas: core::ptr::NonNull<Self>,
        shader: std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            (&mut *canvas.as_ptr()).onClipShader_inherited(shader, op);
        }
    }
}

// Source: include/core/SkCanvas.h:2398-2398
impl<'device> SkCanvas<'device> {
    unsafe fn onClipRegion(
        &mut self,
        region: &crate::src::core::SkRegion::SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            Self::onClipRegion_raw(core::ptr::NonNull::from(&mut *self), region, op);
        }
    }
    pub unsafe fn onClipRegion_raw(
        canvas: core::ptr::NonNull<Self>,
        region: &crate::src::core::SkRegion::SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::onClipRegion(hooks.as_ref(), canvas, region, op);
            }
        } else {
            unsafe {
                Self::onClipRegion_base_raw(canvas, region, op);
            }
        }
    }
    pub unsafe fn onClipRegion_base_raw(
        canvas: core::ptr::NonNull<Self>,
        region: &crate::src::core::SkRegion::SkRegion,
        op: crate::include::core::SkClipOp::SkClipOp,
    ) {
        unsafe {
            (&mut *canvas.as_ptr()).onClipRegion_inherited(region, op);
        }
    }
}

// Source: include/core/SkCanvas.h:2399-2399
impl<'device> SkCanvas<'device> {
    unsafe fn onResetClip(&mut self) {
        unsafe {
            Self::onResetClip_raw(core::ptr::NonNull::from(&mut *self));
        }
    }
    pub unsafe fn onResetClip_raw(canvas: core::ptr::NonNull<Self>) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::onResetClip(hooks.as_ref(), canvas);
            }
        } else {
            unsafe {
                Self::onResetClip_base_raw(canvas);
            }
        }
    }
    pub unsafe fn onResetClip_base_raw(canvas: core::ptr::NonNull<Self>) {
        unsafe {
            (&mut *canvas.as_ptr()).onResetClip_inherited();
        }
    }
}

// Source: include/core/SkCanvas.h:2425-2426
impl<'device> SkCanvas<'device> {
    /// Protected/friend projection: exclusive Canvas/Surface operation lease;
    /// no references to Canvas, Device, Surface or pixels survive this COW callback.
    pub unsafe fn predrawNotify_raw(canvas: core::ptr::NonNull<Self>, overwrite: bool) -> bool {
        unsafe { Self::predraw_notify_overwrite_raw(canvas, overwrite) }
    }
    pub unsafe fn predrawNotify_rect_raw(
        canvas: core::ptr::NonNull<Self>,
        rect: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        flags: PredrawFlags,
    ) -> bool {
        unsafe { Self::predraw_notify_rect_raw(canvas, rect, paint, flags) }
    }
}

// Source: include/core/SkCanvas.h:2401-2401
impl<'device> SkCanvas<'device> {
    pub unsafe fn onDiscard_raw(canvas: core::ptr::NonNull<Self>) {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            unsafe {
                SkCanvasVirtual::onDiscard(hooks.as_ref(), canvas);
            }
        } else {
            unsafe {
                Self::onDiscard_base_raw(canvas);
            }
        }
    }
}

// Source: include/core/SkCanvas.h:2324-2326
impl<'device> SkCanvas<'device> {
    /// Qualified Canvas inherited strategy, bypassing this owner's override.
    pub unsafe fn getSaveLayerStrategy_base_raw(
        canvas: core::ptr::NonNull<Self>,
        rec: &SaveLayerRec<'_>,
    ) -> SaveLayerStrategy {
        unsafe { canvas.as_ref() }.inherited_getSaveLayerStrategy(rec)
    }
    // Source: include/core/SkCanvas.h:2329-2329
    pub unsafe fn onDoSaveBehind_base_raw(
        _canvas: core::ptr::NonNull<Self>,
        _bounds: Option<&crate::src::core::SkRect::SkRect>,
    ) -> bool {
        true
    }
}

// Source: include/core/SkCanvas.h:2357-2358
impl<'device> SkCanvas<'device> {
    pub unsafe fn onDrawPatch_raw(
        canvas: core::ptr::NonNull<Self>,
        cubics: &[crate::src::core::SkPoint::SkPoint; 12],
        colors: Option<&[crate::src::core::SkColor::SkColor; 4]>,
        tex: Option<&[crate::src::core::SkPoint::SkPoint; 4]>,
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe {
                crate::include::core::SkCanvasVirtualEnforcer::SkCanvasVirtualEnforcer::onDrawPatch(
                    hooks.as_ref(),
                    canvas,
                    cubics,
                    colors,
                    tex,
                    mode,
                    paint,
                )
            };
        }
        unsafe { Self::onDrawPatch_base_raw(canvas, cubics, colors, tex, mode, paint) }
    }
}
// Source: include/core/SkCanvas.h:2368-2370
impl<'device> SkCanvas<'device> {
    pub unsafe fn onDrawAtlas2_raw(
        canvas: core::ptr::NonNull<Self>,
        atlas: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
        xform: &[crate::src::core::SkRSXform::SkRSXform],
        tex: &[crate::src::core::SkRect::SkRect],
        colors: &[crate::src::core::SkColor::SkColor],
        mode: crate::src::core::SkBlendMode::SkBlendMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        cull: Option<&crate::src::core::SkRect::SkRect>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe {
                SkCanvasVirtual::onDrawAtlas2(
                    hooks.as_ref(),
                    canvas,
                    atlas,
                    xform,
                    tex,
                    colors,
                    mode,
                    sampling,
                    cull,
                    paint,
                )
            };
        }
        unsafe {
            Self::onDrawAtlas2_base_raw(
                canvas, atlas, xform, tex, colors, mode, sampling, cull, paint,
            )
        }
    }
}
// Source: include/core/SkCanvas.h:2378-2378
impl<'device> SkCanvas<'device> {
    pub unsafe fn onDrawMesh_raw(
        canvas: core::ptr::NonNull<Self>,
        mesh: &crate::src::core::SkMesh::SkMesh,
        blender: crate::include::core::SkRefCnt::sk_sp<
            dyn crate::include::core::SkBlender::SkBlender,
        >,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> Result<(), crate::RasterError> {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe {
                SkCanvasVirtual::onDrawMesh(hooks.as_ref(), canvas, mesh, blender, paint)
            };
        }
        unsafe { Self::onDrawMesh_base_raw(canvas, mesh, blender, paint) }
    }
}

// Source: include/core/SkCanvas.h:2306-2306
impl<'device> SkCanvas<'device> {
    fn onNewSurface(
        &self,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::image::SkSurface::SkSurface<'static>>
    {
        unsafe { Self::onNewSurface_raw(core::ptr::NonNull::from(self), info, props) }
    }
    /// Safety: preserve this native query's shared Canvas/base lease; hooks cannot
    /// mutate the base or invalidate outstanding Device/pixel references.
    pub unsafe fn onNewSurface_raw(
        canvas: core::ptr::NonNull<Self>,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::image::SkSurface::SkSurface<'static>>
    {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe { SkCanvasVirtual::onNewSurface(hooks.as_ref(), canvas, info, props) };
        }
        unsafe { Self::onNewSurface_base_raw(canvas, info, props) }
    }
    pub unsafe fn onNewSurface_base_raw(
        canvas: core::ptr::NonNull<Self>,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> crate::include::core::SkRefCnt::sk_sp<dyn crate::src::image::SkSurface::SkSurface<'static>>
    {
        unsafe { canvas.as_ref() }.onNewSurface_inherited(info, props)
    }
    // Source: include/core/SkCanvas.h:2309-2309
    fn onPeekPixels(&self, pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>) -> bool {
        unsafe { Self::onPeekPixels_raw(core::ptr::NonNull::from(self), pmap) }
    }
    pub unsafe fn onPeekPixels_raw(
        canvas: core::ptr::NonNull<Self>,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe { SkCanvasVirtual::onPeekPixels(hooks.as_ref(), canvas, pmap) };
        }
        unsafe { Self::onPeekPixels_base_raw(canvas, pmap) }
    }
    pub unsafe fn onPeekPixels_base_raw(
        canvas: core::ptr::NonNull<Self>,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        unsafe { canvas.as_ref() }.onPeekPixels_inherited(pmap)
    }
}

// Source: include/core/SkCanvas.h:2311-2311
impl<'device> SkCanvas<'device> {
    fn onImageInfo(&self) -> crate::src::core::SkImageInfo::SkImageInfo {
        unsafe { Self::onImageInfo_raw(core::ptr::NonNull::from(self)) }
    }
    /// Safety: query-only shared base lease; output storage is disjoint from Canvas.
    pub unsafe fn onImageInfo_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> crate::src::core::SkImageInfo::SkImageInfo {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe { SkCanvasVirtual::onImageInfo(hooks.as_ref(), canvas) };
        }
        unsafe { Self::onImageInfo_base_raw(canvas) }
    }
    pub unsafe fn onImageInfo_base_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> crate::src::core::SkImageInfo::SkImageInfo {
        unsafe { canvas.as_ref() }.onImageInfo_inherited()
    }
}

// Source: include/core/SkCanvas.h:2312-2312
impl<'device> SkCanvas<'device> {
    fn onGetProps(
        &self,
        props: Option<&mut crate::include::core::SkSurfaceProps::SkSurfaceProps>,
        top: bool,
    ) -> bool {
        unsafe { Self::onGetProps_raw(core::ptr::NonNull::from(self), props, top) }
    }
    /// Safety: query-only shared base lease; output storage is disjoint from Canvas.
    pub unsafe fn onGetProps_raw(
        canvas: core::ptr::NonNull<Self>,
        props: Option<&mut crate::include::core::SkSurfaceProps::SkSurfaceProps>,
        top: bool,
    ) -> bool {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe { SkCanvasVirtual::onGetProps(hooks.as_ref(), canvas, props, top) };
        }
        unsafe { Self::onGetProps_base_raw(canvas, props, top) }
    }
    pub unsafe fn onGetProps_base_raw(
        canvas: core::ptr::NonNull<Self>,
        props: Option<&mut crate::include::core::SkSurfaceProps::SkSurfaceProps>,
        top: bool,
    ) -> bool {
        unsafe { canvas.as_ref() }.onGetProps_inherited(props, top)
    }
}

// Source: include/core/SkCanvas.h:2310-2310
impl<'device> SkCanvas<'device> {
    unsafe fn onAccessTopLayerPixels(
        &mut self,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        unsafe { Self::onAccessTopLayerPixels_raw(core::ptr::NonNull::from(&mut *self), pmap) }
    }
    /// Safety: caller retains the original pinned Canvas/Device exclusive pixel
    /// lease; returned storage never outlives that actual backing allocation.
    pub unsafe fn onAccessTopLayerPixels_raw(
        canvas: core::ptr::NonNull<Self>,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe {
                SkCanvasVirtual::onAccessTopLayerPixels(hooks.as_ref(), canvas, pmap)
            };
        }
        unsafe { Self::onAccessTopLayerPixels_base_raw(canvas, pmap) }
    }
    pub unsafe fn onAccessTopLayerPixels_base_raw(
        canvas: core::ptr::NonNull<Self>,
        pmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>,
    ) -> bool {
        unsafe { (&mut *canvas.as_ptr()).onAccessTopLayerPixels_inherited(pmap) }
    }
}

// Source: include/core/SkCanvas.h:2682-2682
// The native pointer names a forward-declared class. No complete derived owner
// or provider is required for the native null base implementation.
unsafe extern "C" {
    pub type SkPaintFilterCanvas;
}
impl<'device> SkCanvas<'device> {
    /// Safety: preserve the original Canvas shared query lease through dispatch.
    pub unsafe fn internal_private_asPaintFilterCanvas_raw(
        canvas: core::ptr::NonNull<Self>,
    ) -> *mut SkPaintFilterCanvas {
        let hooks = unsafe { (*canvas.as_ptr()).fVirtual };
        if let Some(hooks) = hooks {
            return unsafe { hooks.as_ref().internal_private_asPaintFilterCanvas(canvas) };
        }
        unsafe { Self::internal_private_asPaintFilterCanvas_base_raw(canvas) }
    }
    pub unsafe fn internal_private_asPaintFilterCanvas_base_raw(
        _: core::ptr::NonNull<Self>,
    ) -> *mut SkPaintFilterCanvas {
        core::ptr::null_mut()
    }
}
