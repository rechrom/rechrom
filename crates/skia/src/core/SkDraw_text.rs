use crate::src::core::{SkDraw::Draw, SkGlyph::SkGlyph, SkPaint::SkPaint, SkPoint::SkPoint};

// Source: src/core/SkDraw_text.cpp:42-51
fn check_glyph_position(position: SkPoint) -> bool {
    let gt = |a: f32, b: i32| !(a <= b as f32);
    let lt = |a: f32, b: i32| !(a >= b as f32);
    !(gt(position.fX, i32::MAX - (i16::MAX as i32 + u16::MAX as i32))
        || lt(position.fX, i32::MIN - i16::MIN as i32)
        || gt(position.fY, i32::MAX - (i16::MAX as i32 + u16::MAX as i32))
        || lt(position.fY, i32::MIN - i16::MIN as i32))
}

impl Draw<'_> {
    // Source: src/core/SkDraw_text.cpp:54-126
    /// # Safety
    /// Uphold Draw's exclusive destination/CTM/clip/chooser leases. Every glyph
    /// and its initialized image storage remain alive for the entire operation.
    /// Accepted positions have the integral rounding required by SkGlyph::maskAt.
    // Adapted: tuple slice retains native zipped glyph/position order and borrowed
    // glyph image ownership. Arena pinning and chooser/sprite errors are explicit.
    pub unsafe fn paintMasks(
        &self,
        accepted: &[(&SkGlyph, SkPoint)],
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        use crate::src::core::{
            SkArenaAlloc::SkSTArenaAlloc,
            SkBitmap::SkBitmap,
            SkDrawTypes::{kSkBlitterContextSize, SkDrawCoverage},
            SkImageInfo::SkImageInfo,
            SkMask::Format,
            SkRasterClip::SkAAClipBlitterWrapper,
            SkRect::{SkIRect, SkRect},
            SkRegion::Cliperator,
        };
        let mut alloc = core::pin::pin!(SkSTArenaAlloc::<{ kSkBlitterContextSize }>::new(0));
        let clip = self.fRC.expect("native nonnull raster clip");
        let props = self.fProps.copied().unwrap_or_default();
        let mut blitter = unsafe {
            (self.fBlitterChooser)(
                &self.fDst,
                self.fCTM.expect("native nonnull CTM"),
                paint,
                alloc.as_mut().pinned_arena(),
                SkDrawCoverage::kNo,
                clip.clip_shader(),
                &props,
                &SkRect::MakeEmpty(),
            )
        }?
        .expect("native nonnull glyph blitter");
        let mut wrapper = SkAAClipBlitterWrapper::from_clip(clip, unsafe {
            core::pin::Pin::new_unchecked(blitter.as_mut())
        });
        let mut wrapper = core::pin::pin!(wrapper);
        let mut blitter = wrapper.as_mut().get_blitter();
        let use_region = clip.is_bw() && !clip.is_rect();
        if use_region {
            for &(glyph, pos) in accepted {
                if check_glyph_position(pos) {
                    let mask = glyph.maskAt(pos);
                    let mut clipper = Cliperator::new(clip.bw_rgn(), mask.fBounds);
                    if !clipper.done() {
                        if mask.fFormat == Format::kARGB32_Format {
                            let mut bm = SkBitmap::default();
                            unsafe {
                                bm.installPixels(
                                    &SkImageInfo::MakeN32Premul(
                                        mask.fBounds.width(),
                                        mask.fBounds.height(),
                                    ),
                                    mask.fImage.cast_mut().cast(),
                                    mask.fRowBytes as usize,
                                    None,
                                    core::ptr::null_mut(),
                                )
                            };
                            bm.setImmutable();
                            unsafe {
                                self.drawSprite(&bm, mask.fBounds.fLeft, mask.fBounds.fTop, paint)
                            }?;
                        } else {
                            loop {
                                unsafe { blitter.as_mut().blitMask(&mask, clipper.rect()) };
                                clipper.next();
                                if clipper.done() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let clip_bounds = if clip.is_bw() {
                *clip.bw_rgn().get_bounds()
            } else {
                *clip.aa_rgn().get_bounds()
            };
            for &(glyph, pos) in accepted {
                if check_glyph_position(pos) {
                    let mask = glyph.maskAt(pos);
                    let mut storage = SkIRect::make_empty();
                    let mut bounds = &mask.fBounds;
                    if !clip_bounds.contains_no_empty_check(&mask.fBounds) {
                        if !storage.intersect_ab(mask.fBounds, clip_bounds) {
                            continue;
                        }
                        bounds = &storage;
                    }
                    if mask.fFormat == Format::kARGB32_Format {
                        let mut bm = SkBitmap::default();
                        unsafe {
                            bm.installPixels(
                                &SkImageInfo::MakeN32Premul(
                                    mask.fBounds.width(),
                                    mask.fBounds.height(),
                                ),
                                mask.fImage.cast_mut().cast(),
                                mask.fRowBytes as usize,
                                None,
                                core::ptr::null_mut(),
                            )
                        };
                        bm.setImmutable();
                        unsafe {
                            self.drawSprite(&bm, mask.fBounds.fLeft, mask.fBounds.fTop, paint)
                        }?;
                    } else {
                        unsafe { blitter.as_mut().blitMask(&mask, bounds) };
                    }
                }
            }
        }
        Ok(())
    }
}

impl Draw<'_> {
    // Source: src/core/SkDraw_text.cpp:128-140
    /// Safety: Draw state is independent from reentrant Device state; original
    /// destination pixels, Canvas and glyph owners retain their native leases.
    pub unsafe fn drawGlyphRunList<'canvas>(
        &self,
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'canvas>>,
        painter: &crate::src::core::SkGlyphRunPainter::GlyphRunListPainter,
        list: &crate::src::text::GlyphRun::GlyphRunList,
        paint: &SkPaint,
    ) -> Result<(), crate::RasterError> {
        self.validate();
        if self.fRC.expect("native configured raster clip").is_empty() {
            return Ok(());
        }
        unsafe {
            painter.drawForBitmapDevice(
                canvas,
                self,
                list,
                paint,
                self.fCTM.expect("native configured CTM"),
            )
        }
    }
}
