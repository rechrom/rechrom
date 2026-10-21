// Source: src/core/SkCanvasPriv.h:28-36
// The same original Canvas owner is borrowed for the scope. Normal destruction is
// explicit/fallible; Drop only rolls back unfinished state on an error exit.
#[must_use]
pub struct SkAutoCanvasMatrixPaint<'canvas, 'device> {
    fCanvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
    fSaveCount: i32,
    lease: core::marker::PhantomData<&'canvas mut crate::src::core::SkCanvas::SkCanvas<'device>>,
}
// Source: src/core/SkCanvasPriv.cpp:32-49
impl<'canvas, 'device> SkAutoCanvasMatrixPaint<'canvas, 'device> {
    /// Safety: Canvas remains pinned and exclusively leased, including its
    /// devices/pixels, until finish or cancellation. No reference crosses a draw.
    pub unsafe fn new(
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
        matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        bounds: &crate::src::core::SkRect::SkRect,
    ) -> Result<Self, crate::RasterError> {
        let result = Self {
            fCanvas: canvas,
            fSaveCount: unsafe { canvas.as_ref() }.getSaveCount(),
            lease: core::marker::PhantomData,
        };
        if let Some(paint) = paint {
            let mut new_bounds = *bounds;
            if let Some(matrix) = matrix {
                matrix.mapRect_in_place(&mut new_bounds);
            }
            unsafe {
                core::pin::Pin::new_unchecked(&mut *canvas.as_ptr())
                    .saveLayer_bounds(Some(&new_bounds), Some(paint))
            }?;
        } else if matrix.is_some() {
            unsafe { core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).save() };
        }
        if let Some(matrix) = matrix {
            unsafe { core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()).concat(matrix) };
        }
        Ok(result)
    }
}
// Source: src/core/SkCanvasPriv.cpp:51-53
impl SkAutoCanvasMatrixPaint<'_, '_> {
    /// Explicit native destructor path: restoration errors propagate unchanged.
    /// Safety: preserve the original pinned Canvas/device/pixel lease.
    pub unsafe fn finish(self) -> Result<(), crate::RasterError> {
        unsafe {
            crate::src::core::SkCanvas::SkCanvas::restoreToCount_raw(self.fCanvas, self.fSaveCount)
        }
    }
}
// Rust error-exit adaptation only. Successful callers must use finish; dropping
// after a failed draw cancels the remaining records without invoking more draws.
impl Drop for SkAutoCanvasMatrixPaint<'_, '_> {
    fn drop(&mut self) {
        unsafe {
            crate::src::core::SkCanvas::SkCanvas::cancel_matrix_paint_raw(
                self.fCanvas,
                self.fSaveCount,
            )
        };
    }
}

// Source: src/core/SkCanvasPriv.h:49-51
/// Safety: original pinned Canvas and its exclusive device/pixel lease remain
/// live through snapshotting, drawing and any Surface COW operation.
pub unsafe fn SaveBehind<'device>(
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
    subset: Option<&crate::src::core::SkRect::SkRect>,
) -> Result<i32, crate::RasterError> {
    unsafe {
        crate::src::core::SkCanvas::SkCanvas::only_axis_aligned_saveBehind_raw(canvas, subset)
    }
}

// Source: src/core/SkCanvasPriv.h:52-54
/// Safety: canvas is the original live pinned allocation with exclusive native
/// device/pixel leases. No whole-Canvas borrow spans save-behind restore/drawing.
pub unsafe fn DrawBehind<'device>(
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
    paint: &crate::src::core::SkPaint::SkPaint,
) -> Result<(), crate::RasterError> {
    unsafe { crate::src::core::SkCanvas::SkCanvas::drawClippedToSaveBehind_raw(canvas, paint) }
}
// Source: src/core/SkCanvasPriv.h:57-59
/// Safety: caller owns exclusive state access to this original pinned Canvas.
/// This clip-only operation does not perform a Surface COW callback.
pub unsafe fn ResetClip<'device>(
    mut canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
) {
    unsafe {
        core::pin::Pin::new_unchecked(canvas.as_mut()).internal_private_resetClip();
    }
}

// Source: src/core/SkCanvasPriv.h:61-63
/// Safety: canvas is its original live pinned owner under the caller's native
/// operation lease. The returned Device pointer is borrowed, never adopted or
/// retained here, and must not outlive its Canvas/save-layer record. Any mutable
/// projection must exclude conflicting references and Surface COW callbacks.
pub unsafe fn TopDevice<'device>(
    canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
) -> core::ptr::NonNull<dyn crate::src::core::SkDevice::SkDeviceVirtual + 'device> {
    unsafe { crate::src::core::SkCanvas::SkCanvas::top_device_raw(canvas) }
}

// Source: src/core/SkCanvasPriv.cpp:116-129
// A borrowed slice carries the original array/count pair without cloning image owners.
// Canonical ImageSetEntry shares the original image owner.
// Source: src/core/SkCanvasPriv.h:68-69
pub fn GetDstClipAndMatrixCounts(
    set: &[crate::src::core::SkCanvas::ImageSetEntry],
    totalDstClipCount: &mut i32,
    totalMatrixCount: &mut i32,
) {
    let mut dstClipCount = 0;
    let mut maxMatrixIndex = -1;
    for entry in set {
        dstClipCount += 4 * i32::from(entry.fHasClip);
        if entry.fMatrixIndex > maxMatrixIndex {
            maxMatrixIndex = entry.fMatrixIndex;
        }
    }
    *totalDstClipCount = dstClipCount;
    *totalMatrixCount = maxMatrixIndex + 1;
}

// Source: src/core/SkCanvasPriv.cpp:134-173
// Uncompiled pending ImageFilter::asAColorFilter. Paint and ColorFilter use canonical
// Arc trait owner; composition retains original filter order and shared identities.
// Source: src/core/SkCanvasPriv.h:105-105
pub fn ImageToColorFilter(
    paint: &mut crate::src::core::SkPaint::SkPaint,
) -> Result<bool, crate::RasterError> {
    debug_assert!(paint.getImageFilter().is_some());
    if paint.getMaskFilter().is_some() {
        return Ok(false);
    }
    let mut imgCFPtr: Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>> =
        None;
    if !paint
        .getImageFilter()
        .expect("native non-null image-filter precondition")
        .asAColorFilter(&mut imgCFPtr)?
    {
        return Ok(false);
    }
    let mut imgCF = imgCFPtr;
    if paint.getColorFilter().is_some() {
        imgCF = Some(
            imgCF
                .expect("successful asAColorFilter yields a color filter")
                .makeComposed(paint.refColorFilter()),
        );
    }
    paint.setColorFilter(imgCF);
    paint.setImageFilter(None);
    Ok(true)
}

// Source: src/core/SkCanvasPriv.h:71-80
pub fn ScaledBackdropLayerWithTileMode<'a>(
    bounds: Option<&'a crate::src::core::SkRect::SkRect>,
    paint: Option<&'a crate::src::core::SkPaint::SkPaint>,
    backdrop: Option<&'a std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
    backdrop_scale: f32,
    tile_mode: crate::include::core::SkTileMode::SkTileMode,
    flags: crate::src::core::SkCanvas::SaveLayerFlags,
    filters: crate::src::core::SkCanvas::FilterSpan<'a>,
) -> crate::src::core::SkCanvas::SaveLayerRec<'a> {
    crate::src::core::SkCanvas::SaveLayerRec::with_parameters(
        bounds,
        paint,
        backdrop,
        None,
        backdrop_scale,
        tile_mode,
        flags,
        filters,
    )
}
// Source: src/core/SkCanvasPriv.h:82-90
pub fn ScaledBackdropLayer<'a>(
    bounds: Option<&'a crate::src::core::SkRect::SkRect>,
    paint: Option<&'a crate::src::core::SkPaint::SkPaint>,
    backdrop: Option<&'a std::sync::Arc<dyn crate::src::core::SkImageFilter::SkImageFilter>>,
    backdrop_scale: f32,
    flags: crate::src::core::SkCanvas::SaveLayerFlags,
    filters: crate::src::core::SkCanvas::FilterSpan<'a>,
) -> crate::src::core::SkCanvas::SaveLayerRec<'a> {
    ScaledBackdropLayerWithTileMode(
        bounds,
        paint,
        backdrop,
        backdrop_scale,
        crate::include::core::SkTileMode::SkTileMode::kClamp,
        flags,
        filters,
    )
}
// Source: src/core/SkCanvasPriv.h:92-94
pub fn GetBackdropScaleFactor(rec: &crate::src::core::SkCanvas::SaveLayerRec<'_>) -> f32 {
    rec.fExperimentalBackdropScale
}
// Source: src/core/SkCanvasPriv.h:96-98
pub fn SetBackdropScaleFactor(rec: &mut crate::src::core::SkCanvas::SaveLayerRec<'_>, scale: f32) {
    rec.fExperimentalBackdropScale = scale;
}

// Source: src/core/SkCanvasPriv.h:115-115
pub const kMaxPictureOpsToUnrollInsteadOfRef: i32 = 1;
// Source: src/core/SkCanvasPriv.h:129-167
// Native filter guard exclusively borrows the actual pinned Canvas operation;
// Explicit native moves preserve a valid moved-from guard with zero layers.
// No cloned Canvas or second device/filter backend is introduced.
pub struct AutoLayerForImageFilter<'canvas, 'device> {
    fCanvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
    operation:
        core::marker::PhantomData<&'canvas mut crate::src::core::SkCanvas::SkCanvas<'device>>,
    fTempLayersForFilters: i32,
    #[cfg(sk_debug)]
    fSaveCount: i32,
    fPaint: crate::src::core::SkPaint::SkPaint,
}
impl AutoLayerForImageFilter<'_, '_> {
    pub fn paint(&self) -> &crate::src::core::SkPaint::SkPaint {
        &self.fPaint
    }
}
// Field ownership/drop representation is in the complete guard declaration above.
impl<'canvas, 'device> AutoLayerForImageFilter<'canvas, 'device> {
    // Private friend adaptation lends disjoint Canvas and working Paint fields for
    // exactly one draw; the guard retains exclusive Canvas ownership until restore.
    pub(crate) fn canvas_and_paint(
        &mut self,
    ) -> (
        core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
        &crate::src::core::SkPaint::SkPaint,
    ) {
        unsafe {
            (&mut *self.fCanvas.as_ptr()).rebind_filter_guard();
        }
        (self.fCanvas, &self.fPaint)
    }
}

// Source: src/core/SkCanvasPriv.cpp:175-207
impl<'canvas, 'device> AutoLayerForImageFilter<'canvas, 'device> {
    /// # Safety
    /// Canvas is at its stable pinned address with an exclusive device/pixel lease
    /// for this whole guard lifetime, including every restore performed in Drop.
    pub(crate) unsafe fn new(
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'device>>,
        paint: &crate::src::core::SkPaint::SkPaint,
        raw_bounds: Option<&crate::src::core::SkRect::SkRect>,
        skip_mask_filter_layer: bool,
    ) -> Result<Self, crate::RasterError> {
        let paint = paint.clone();
        unsafe {
            (&mut *canvas.as_ptr()).rebind_filter_guard();
        }
        #[cfg(sk_debug)]
        let save_count = unsafe { canvas.as_ref() }.getSaveCount();
        let mut layer = Self {
            fPaint: paint,
            fCanvas: canvas,
            operation: core::marker::PhantomData,
            fTempLayersForFilters: 0,
            #[cfg(sk_debug)]
            fSaveCount: save_count,
        };
        if layer.fPaint.getImageFilter().is_some() && !ImageToColorFilter(&mut layer.fPaint)? {
            unsafe { layer.addImageFilterLayer(raw_bounds) }?;
        }
        if layer.fPaint.getMaskFilter().is_some() && !skip_mask_filter_layer {
            unsafe { layer.addMaskFilterLayer(raw_bounds) }?;
        }
        Ok(layer)
    }
}
// Source: src/core/SkCanvasPriv.cpp:209-216
impl AutoLayerForImageFilter<'_, '_> {
    /// Explicit normal native restore outlet. A successful draw must call this;
    /// dropping the guard cancels unfinished layers without drawing them.
    /// # Safety
    /// Preserve the pinned Canvas's exclusive native device/pixel/filter lease.
    pub unsafe fn finish(mut self) -> Result<(), crate::RasterError> {
        while self.fTempLayersForFilters > 0 {
            // Ownership of this popped record is consumed even if its draw fails.
            self.fTempLayersForFilters -= 1;
            unsafe {
                crate::src::core::SkCanvas::SkCanvas::restore_filter_layer_raw(self.fCanvas)
            }?;
        }
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkCanvasPriv.cpp\0".as_ptr(), 215; @condition b"fSaveCount < 0 || fCanvas->getSaveCount() == fSaveCount\0".as_ptr(); self.fSaveCount<0||unsafe{self.fCanvas.as_ref()}.getSaveCount()==self.fSaveCount);
        Ok(())
    }
}
impl Drop for AutoLayerForImageFilter<'_, '_> {
    fn drop(&mut self) {
        while self.fTempLayersForFilters > 0 {
            self.fTempLayersForFilters -= 1;
            unsafe {
                (&mut *self.fCanvas.as_ptr()).cancel_filter_layer();
            }
        }
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/core/SkCanvasPriv.cpp\0".as_ptr(), 215; @condition b"fSaveCount < 0 || fCanvas->getSaveCount() == fSaveCount\0".as_ptr(); self.fSaveCount<0||unsafe{self.fCanvas.as_ref()}.getSaveCount()==self.fSaveCount);
    }
}
// Source: src/core/SkCanvasPriv.cpp:218-220
impl<'canvas, 'device> AutoLayerForImageFilter<'canvas, 'device> {
    /// Safety: preserve the original exclusive Canvas operation lease through
    /// the transferred guard's finish/cancellation. Source remains moved-from.
    pub unsafe fn take_from(source: &mut Self) -> Self {
        let moved = Self {
            fPaint: crate::src::core::SkPaint::SkPaint::take_from(&mut source.fPaint),
            fCanvas: source.fCanvas,
            operation: core::marker::PhantomData,
            fTempLayersForFilters: source.fTempLayersForFilters,
            #[cfg(sk_debug)]
            fSaveCount: source.fSaveCount,
        };
        source.fTempLayersForFilters = 0;
        #[cfg(sk_debug)]
        {
            source.fSaveCount = -1;
        }
        moved
    }
}
// Source: src/core/SkCanvasPriv.cpp:222-232
impl<'canvas, 'device> AutoLayerForImageFilter<'canvas, 'device> {
    /// Safety: both guards obey their native operation leases. As in C++, this
    /// assignment does not finish or cancel overwritten destination layers;
    /// caller must satisfy the native responsibility for that prior state.
    pub unsafe fn assign_move<'this>(&'this mut self, source: &mut Self) -> &'this mut Self {
        self.fPaint.assign_move(&mut source.fPaint);
        self.fCanvas = source.fCanvas;
        self.fTempLayersForFilters = source.fTempLayersForFilters;
        #[cfg(sk_debug)]
        {
            self.fSaveCount = source.fSaveCount;
        }
        source.fTempLayersForFilters = 0;
        #[cfg(sk_debug)]
        {
            source.fSaveCount = -1;
        }
        self
    }
}
// Source: src/core/SkCanvasPriv.cpp:234-252
impl AutoLayerForImageFilter<'_, '_> {
    unsafe fn addImageFilterLayer(
        &mut self,
        draw_bounds: Option<&crate::src::core::SkRect::SkRect>,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkCanvasPriv.cpp\0".as_ptr(), 236; @condition b"fPaint.getImageFilter()\0".as_ptr(); self.fPaint.getImageFilter().is_some());
        let mut restore_paint = crate::src::core::SkPaint::SkPaint::default();
        restore_paint.setImageFilter(self.fPaint.refImageFilter());
        restore_paint.setBlender(self.fPaint.refBlender());
        self.fPaint.setImageFilter(None);
        self.fPaint
            .setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrcOver);
        unsafe { self.addLayer(&restore_paint, draw_bounds, false) }?;
        Ok(())
    }
}
// Source: src/core/SkCanvasPriv.cpp:254-298
impl AutoLayerForImageFilter<'_, '_> {
    /// # Safety
    /// Preserve the guard's exclusive pinned Canvas/device lease through restore.
    pub unsafe fn addMaskFilterLayer(
        &mut self,
        draw_bounds: Option<&crate::src::core::SkRect::SkRect>,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/core/SkCanvasPriv.cpp\0".as_ptr(), 256; @condition b"fPaint.getMaskFilter()\0".as_ptr(); self.fPaint.getMaskFilter().is_some());
        crate::SkASSERT!(@source b"src/core/SkCanvasPriv.cpp\0".as_ptr(), 260; @condition b"!fPaint.getImageFilter()\0".as_ptr(); self.fPaint.getImageFilter().is_none());
        let (filter, applies_shading) = self
            .fPaint
            .getMaskFilter()
            .expect("native mask-filter precondition")
            .asImageFilter(
                &unsafe { self.fCanvas.as_ref() }.getTotalMatrix(),
                &self.fPaint,
            );
        let Some(filter) = filter else {
            return Ok(());
        };
        let mut restore_paint = crate::src::core::SkPaint::SkPaint::default();
        if !applies_shading {
            restore_paint.setColor4f(&self.fPaint.getColor4f(), None);
            restore_paint.setShader(self.fPaint.refShader());
            restore_paint.setColorFilter(self.fPaint.refColorFilter());
            restore_paint.setDither(self.fPaint.isDither());
        }
        restore_paint.setBlender(self.fPaint.refBlender());
        restore_paint.setImageFilter(Some(filter));
        self.fPaint
            .setColor4f(&crate::src::core::SkColor::SkColors::kWhite, None);
        self.fPaint.setShader(None);
        self.fPaint.setColorFilter(None);
        self.fPaint.setMaskFilter(None);
        self.fPaint.setDither(false);
        self.fPaint
            .setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrcOver);
        unsafe { self.addLayer(&restore_paint, draw_bounds, !applies_shading) }?;
        Ok(())
    }
}
// Source: src/core/SkCanvasPriv.cpp:300-316
impl AutoLayerForImageFilter<'_, '_> {
    unsafe fn addLayer(
        &mut self,
        restore_paint: &crate::src::core::SkPaint::SkPaint,
        draw_bounds: Option<&crate::src::core::SkRect::SkRect>,
        coverage_only: bool,
    ) -> Result<(), crate::RasterError> {
        let mut storage = crate::src::core::SkRect::SkRect::default();
        let content_bounds = if let Some(bounds) = draw_bounds {
            if self.fPaint.canComputeFastBounds()? {
                Some(self.fPaint.computeFastBounds(bounds, &mut storage)?)
            } else {
                None
            }
        } else {
            None
        };
        unsafe {
            (&mut *self.fCanvas.as_ptr()).save_filter_layer(
                restore_paint,
                content_bounds,
                coverage_only,
            )
        }?;
        self.fTempLayersForFilters += 1;
        Ok(())
    }
}
// Source: src/core/SkCanvasPriv.h:41-41
// Source: src/core/SkCanvasPriv.cpp:57-83
/// Safety: lattice is writable uninitialized native Lattice storage. The returned
/// pointers borrow the reader's original bytes and cannot outlive that input lease.
pub unsafe fn ReadLattice(
    buffer: &mut super::SkReadBuffer::SkReadBuffer<'_>,
    lattice: *mut super::SkCanvas::Lattice,
) -> bool {
    unsafe {
        let xCount = buffer.readInt();
        core::ptr::addr_of_mut!((*lattice).fXCount).write(xCount);
        if xCount < 0 {
            return false;
        }
        core::ptr::addr_of_mut!((*lattice).fXDivs)
            .write(buffer.skipT_count::<i32>(xCount as usize));
        let yCount = buffer.readInt();
        core::ptr::addr_of_mut!((*lattice).fYCount).write(yCount);
        if yCount < 0 {
            return false;
        }
        core::ptr::addr_of_mut!((*lattice).fYDivs)
            .write(buffer.skipT_count::<i32>(yCount as usize));
        let flagCount = buffer.readInt();
        core::ptr::addr_of_mut!((*lattice).fRectTypes).write(core::ptr::null());
        core::ptr::addr_of_mut!((*lattice).fColors).write(core::ptr::null());
        if flagCount != 0 {
            if flagCount < 0 || flagCount as u64 != (xCount as u64 + 1) * (yCount as u64 + 1) {
                return false;
            }
            core::ptr::addr_of_mut!((*lattice).fRectTypes)
                .write(buffer.skipT_count::<super::SkCanvas::LatticeRectType>(flagCount as usize));
            core::ptr::addr_of_mut!((*lattice).fColors)
                .write(buffer.skipT_count::<super::SkColor::SkColor>(flagCount as usize));
        }
        core::ptr::addr_of_mut!((*lattice).fBounds).write(buffer.skipT::<super::SkRect::SkIRect>());
        buffer.isValid()
    }
}
// Source: src/core/SkCanvasPriv.h:47-47
// Source: src/core/SkCanvasPriv.cpp:85-107
/// Safety: all original lattice arrays cover their native counts, bounds is
/// nonnull; a nonnull output covers the returned byte count and is 4-byte aligned.
pub unsafe fn WriteLattice(output: *mut u8, lattice: &super::SkCanvas::Lattice) -> usize {
    unsafe {
        let flagCount = if !lattice.fRectTypes.is_null() {
            (lattice.fXCount + 1) * (lattice.fYCount + 1)
        } else {
            0
        };
        let align4 = |size: usize| (size + 3) & !3;
        let size = (1 + lattice.fXCount + 1 + lattice.fYCount + 1) as usize
            * core::mem::size_of::<i32>()
            + align4(flagCount as usize * core::mem::size_of::<super::SkCanvas::LatticeRectType>())
            + align4(flagCount as usize * core::mem::size_of::<super::SkColor::SkColor>())
            + core::mem::size_of::<super::SkRect::SkIRect>();
        if !output.is_null() {
            let mut writer = super::SkWriter32::SkWriter32::new(output, size);
            writer.write32(lattice.fXCount);
            writer.write(lattice.fXDivs.cast(), lattice.fXCount as usize * 4);
            writer.write32(lattice.fYCount);
            writer.write(lattice.fYDivs.cast(), lattice.fYCount as usize * 4);
            writer.write32(flagCount);
            writer.writePad(lattice.fRectTypes.cast(), flagCount as usize);
            writer.write(
                lattice.fColors.cast(),
                flagCount as usize * core::mem::size_of::<super::SkColor::SkColor>(),
            );
            crate::SkASSERT!(@source b"src/core/SkCanvasPriv.cpp\0".as_ptr(), 102; @condition b"lattice.fBounds\0".as_ptr(); !lattice.fBounds.is_null());
            writer.write(
                lattice.fBounds.cast(),
                core::mem::size_of::<super::SkRect::SkIRect>(),
            );
            crate::SkASSERT!(@source b"src/core/SkCanvasPriv.cpp\0".as_ptr(), 104; @condition b"writer.bytesWritten() == size\0".as_ptr(); (writer.bytesWritten()) == (size));
        }
        size
    }
}
// Source: src/core/SkCanvasPriv.h:43-43
// Source: src/core/SkCanvasPriv.cpp:109-114
/// Safety: same original lattice array/bounds leases as the raw storage overload.
pub unsafe fn WriteLattice_buffer(
    buffer: &mut dyn super::SkWriteBuffer::SkWriteBuffer,
    lattice: &super::SkCanvas::Lattice,
) {
    unsafe {
        let size = WriteLattice(core::ptr::null_mut(), lattice);
        // u32 backing retains the native temporary allocation's required alignment.
        let mut storage = vec![core::mem::MaybeUninit::<u32>::uninit(); (size + 3) / 4];
        WriteLattice(storage.as_mut_ptr().cast(), lattice);
        buffer.writePad32(storage.as_ptr().cast(), size);
    }
}
