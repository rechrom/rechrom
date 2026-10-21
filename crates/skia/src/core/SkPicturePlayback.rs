// Copyright 2014 Google Inc. BSD-style license.
use super::{
    SkBlendMode::SkBlendMode,
    SkCanvas::{PointMode, SkAutoCanvasRestore, SkCanvas, SrcRectConstraint},
    SkM44::SkM44,
    SkMatrix::SkMatrix,
    SkPicture::AbortCallback,
    SkPictureData::SkPictureData,
    SkPictureFlat::*,
    SkPoint::SkPoint,
    SkRRect::SkRRect,
    SkReadBuffer::SkReadBuffer,
    SkRect::SkRect,
    SkRegion::{Op, SkRegion},
};
use crate::include::core::{
    SkClipOp::SkClipOp,
    SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
};
use core::{cell::Cell, pin::Pin, ptr::NonNull};
// Source: src/core/SkPicturePlayback.h:25-26
// Source: src/core/SkPicturePlayback.h:33-40
pub struct SkPicturePlayback<'data> {
    fPictureData: &'data SkPictureData,
    fCurOffset: Cell<usize>,
}
impl<'data> SkPicturePlayback<'data> {
    pub fn new(data: &'data SkPictureData) -> Self {
        Self {
            fPictureData: data,
            fCurOffset: Cell::new(0),
        }
    }
    pub fn curOpID(&self) -> usize {
        self.fCurOffset.get()
    }
    pub fn resetOpID(&self) {
        self.fCurOffset.set(0);
    }
}
// Source: src/core/SkPicturePlayback.h:48-59
struct AutoResetOpID<'a> {
    fPlayback: &'a Cell<usize>,
}
impl Drop for AutoResetOpID<'_> {
    fn drop(&mut self) {
        self.fPlayback.set(0);
    }
}
// Source: src/core/SkPicturePlayback.cpp:52-59
fn get_rect_ptr(reader: &mut SkReadBuffer<'_>) -> Option<SkRect> {
    if reader.readBool() {
        Some(reader.readRect())
    } else {
        None
    }
}
// Source: src/core/SkPicturePlayback.cpp:105-109
fn validate_offsetToRestore(reader: &mut SkReadBuffer<'_>, offsetToRestore: usize) {
    if offsetToRestore != 0 {
        reader.validate(offsetToRestore & 3 == 0 && offsetToRestore >= reader.offset());
    }
}
// Source: src/core/SkPicturePlayback.cpp:111-130
unsafe fn do_clip_op(
    reader: &mut SkReadBuffer<'_>,
    canvas: NonNull<SkCanvas<'_>>,
    op: Op,
) -> Option<SkClipOp> {
    match op {
        Op::Difference => Some(SkClipOp::kDifference),
        Op::Intersect => Some(SkClipOp::kIntersect),
        Op::Replace => {
            crate::SkASSERT!(@source b"src/core/SkPicturePlayback.cpp\0".as_ptr(),121; @condition b"reader->isVersionLT(SkPicturePriv::kNoExpandingClipOps)\0".as_ptr(); reader.isVersionLT(super::SkPicturePriv::kNoExpandingClipOps));
            unsafe {
                super::SkCanvasPriv::ResetClip(canvas);
            }
            Some(SkClipOp::kIntersect)
        }
        _ => {
            crate::SkASSERT!(@source b"src/core/SkPicturePlayback.cpp\0".as_ptr(),127; @condition b"reader->isVersionLT(SkPicturePriv::kNoExpandingClipOps)\0".as_ptr(); reader.isVersionLT(super::SkPicturePriv::kNoExpandingClipOps));
            None
        }
    }
}
// Rust byte-storage projection only: native skipT/count validation is retained.
// Copies avoid forming aligned typed references into arbitrary caller byte storage.
fn read_array<T: Copy>(reader: &mut SkReadBuffer<'_>, count: usize) -> Option<Vec<T>> {
    let ptr = reader.skipT_count::<T>(count);
    if !reader.isValid() {
        return None;
    }
    Some(
        (0..count)
            .map(|i| unsafe { ptr.add(i).read_unaligned() })
            .collect(),
    )
}
fn read_blend(reader: &mut SkReadBuffer<'_>) -> SkBlendMode {
    let v = reader.read32LE(SkBlendMode::kLastMode as u32);
    unsafe { core::mem::transmute(v as u8) }
}
impl SkPicturePlayback<'_> {
    // Source: src/core/SkPicturePlayback.cpp:61-103
    pub unsafe fn draw<'device>(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        mut callback: Option<&mut dyn AbortCallback>,
        mut buffer: Option<&mut SkReadBuffer<'_>>,
    ) -> Result<(), crate::RasterError> {
        let _aroi = AutoResetOpID {
            fPlayback: &self.fCurOffset,
        };
        crate::SkASSERT!(@source b"src/core/SkPicturePlayback.cpp\0".as_ptr(),65; @condition b"0 == fCurOffset\0".as_ptr(); 0==self.fCurOffset.get());
        let data = self
            .fPictureData
            .opData()
            .expect("native playback requires op data");
        let mut reader =
            SkReadBuffer::new(unsafe { core::slice::from_raw_parts(data.bytes(), data.size()) });
        reader.setVersion(self.fPictureData.info().getVersion() as i32);
        let initialMatrix = unsafe { canvas.as_ref() }.getLocalToDevice();
        let acr = unsafe { SkAutoCanvasRestore::new(Some(canvas), false) };
        let result = unsafe {
            self.draw_loop(
                canvas,
                &mut callback,
                &mut reader,
                &initialMatrix,
                &mut buffer,
            )
        };
        let restored = unsafe { acr.finish() };
        result?;
        restored
    }
    // The separate scope ensures the real native restore runs on every early exit,
    // while parent-buffer invalidity propagates only on native loop completion.
    unsafe fn draw_loop<'device>(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
        callback: &mut Option<&mut dyn AbortCallback>,
        reader: &mut SkReadBuffer<'_>,
        initialMatrix: &SkM44,
        buffer: &mut Option<&mut SkReadBuffer<'_>>,
    ) -> Result<(), crate::RasterError> {
        while !reader.eof() && reader.isValid() {
            if callback.as_mut().is_some_and(|callback| callback.abort()) {
                return Ok(());
            }
            self.fCurOffset.set(reader.offset());
            let bits = reader.readInt() as u32;
            let op = bits >> 24;
            let mut size = bits & 0xffffff;
            if size == 0xffffff {
                size = reader.readInt() as u32;
            }
            if !reader.validate(size > 0 && op > UNUSED as u32 && op <= LAST_DRAWTYPE_ENUM) {
                return Ok(());
            }
            unsafe {
                self.handleOp(
                    reader,
                    core::mem::transmute(op),
                    size,
                    canvas,
                    initialMatrix,
                )
            }?;
        }
        if let Some(buffer) = buffer {
            buffer.validate(reader.isValid());
        }
        Ok(())
    }
    // Source: src/core/SkPicturePlayback.cpp:132-762
    unsafe fn handleOp<'device>(
        &self,
        reader: &mut SkReadBuffer<'_>,
        op: DrawType,
        size: u32,
        mut canvas: NonNull<SkCanvas<'device>>,
        initialMatrix: &SkM44,
    ) -> Result<(), crate::RasterError> {
        unsafe {
            macro_rules! valid {
                () => {
                    if !reader.isValid() {
                        return Ok(());
                    }
                };
            }
            macro_rules! array {
                ($ty:ty,$count:expr) => {
                    match read_array::<$ty>(reader, $count) {
                        Some(array) => array,
                        None => return Ok(()),
                    }
                };
            }
            let data = self.fPictureData;
            match op {
                NOOP => {
                    crate::SkASSERT!(@source b"src/core/SkPicturePlayback.cpp\0".as_ptr(),141; @condition b"size >= 4\0".as_ptr(); size>=4);
                    reader.skip((size - 4) as usize);
                }
                FLUSH | PUSH_CULL | POP_CULL => {}
                CLIP_PATH => {
                    let path = data.getPath(reader);
                    let packed = reader.readInt() as u32;
                    let region = ClipParams_unpackRegionOp(reader, packed);
                    let aa = ClipParams_unpackDoAA(packed);
                    let offset = reader.readInt() as usize;
                    validate_offsetToRestore(reader, offset);
                    valid!();
                    if let Some(op) = do_clip_op(reader, canvas, region) {
                        Pin::new_unchecked(canvas.as_mut()).clipPath(path, op, aa);
                    }
                    if canvas.as_ref().isClipEmpty() && offset != 0 {
                        reader.skip(offset - reader.offset());
                    }
                }
                CLIP_REGION => {
                    let mut region = SkRegion::new();
                    reader.readRegion(&mut region);
                    let packed = reader.readInt() as u32;
                    let op = ClipParams_unpackRegionOp(reader, packed);
                    let offset = reader.readInt() as usize;
                    validate_offsetToRestore(reader, offset);
                    valid!();
                    if let Some(op) = do_clip_op(reader, canvas, op) {
                        Pin::new_unchecked(canvas.as_mut()).clipRegion(&region, op);
                    }
                    if canvas.as_ref().isClipEmpty() && offset != 0 {
                        reader.skip(offset - reader.offset());
                    }
                }
                CLIP_RECT => {
                    let rect = reader.readRect();
                    let packed = reader.readInt() as u32;
                    let region = ClipParams_unpackRegionOp(reader, packed);
                    let aa = ClipParams_unpackDoAA(packed);
                    let offset = reader.readInt() as usize;
                    validate_offsetToRestore(reader, offset);
                    valid!();
                    if let Some(op) = do_clip_op(reader, canvas, region) {
                        Pin::new_unchecked(canvas.as_mut()).clipRect(&rect, op, aa);
                    }
                    if canvas.as_ref().isClipEmpty() && offset != 0 {
                        reader.skip(offset - reader.offset());
                    }
                }
                CLIP_RRECT => {
                    let mut rect = SkRRect::default();
                    reader.readRRect(&mut rect);
                    let packed = reader.readInt() as u32;
                    let region = ClipParams_unpackRegionOp(reader, packed);
                    let aa = ClipParams_unpackDoAA(packed);
                    let offset = reader.readInt() as usize;
                    validate_offsetToRestore(reader, offset);
                    valid!();
                    if let Some(op) = do_clip_op(reader, canvas, region) {
                        Pin::new_unchecked(canvas.as_mut()).clipRRect(&rect, op, aa);
                    }
                    if canvas.as_ref().isClipEmpty() && offset != 0 {
                        reader.skip(offset - reader.offset());
                    }
                }
                CLIP_SHADER_IN_PAINT => {
                    let paint = data.requiredPaint(reader);
                    let op = reader.checkRange(SkClipOp::kDifference, SkClipOp::kIntersect);
                    valid!();
                    Pin::new_unchecked(canvas.as_mut()).clipShader(paint.refShader(), op);
                }
                RESET_CLIP => {
                    super::SkCanvasPriv::ResetClip(canvas);
                }
                CONCAT => {
                    let mut matrix = SkMatrix::default();
                    reader.readMatrix(&mut matrix);
                    valid!();
                    Pin::new_unchecked(canvas.as_mut()).concat(&matrix);
                }
                CONCAT44 => {
                    let colMaj = array!(f32, 16);
                    valid!();
                    let array: [f32; 16] = colMaj.try_into().unwrap();
                    Pin::new_unchecked(canvas.as_mut()).concat44(&SkM44::ColMajor(&array));
                }
                DRAW_ANNOTATION => {
                    let rect = reader.readRect();
                    let mut key = Vec::new();
                    reader.readString(&mut key);
                    let value = reader.readByteArrayAsData();
                    valid!();
                    crate::SkASSERT!(@source b"src/core/SkPicturePlayback.cpp\0".as_ptr(),254; @condition b"data\0".as_ptr(); value.is_some());
                    key.push(0);
                    let key = core::ffi::CStr::from_ptr(key.as_ptr().cast());
                    SkCanvas::drawAnnotation_raw(canvas, &rect, Some(key), value.as_ref());
                }
                DRAW_ARC => {
                    let paint = data.requiredPaint(reader);
                    let rect = reader.readRect();
                    let start = reader.readScalar();
                    let sweep = reader.readScalar();
                    let center = reader.readInt();
                    valid!();
                    SkCanvas::drawArc_raw(canvas, &rect, start, sweep, center != 0, paint)?;
                }
                DRAW_CLEAR => {
                    let color = reader.readInt() as u32;
                    valid!();
                    SkCanvas::clear_raw(canvas, color)?;
                }
                DRAW_DATA => {
                    let length = reader.readInt() as usize;
                    reader.skip(length);
                }
                DRAW_DRAWABLE => {
                    let drawable = data.getDrawable(reader);
                    valid!();
                    SkCanvas::drawDrawable_raw(canvas, drawable, None)?;
                }
                DRAW_DRAWABLE_MATRIX => {
                    let mut matrix = SkMatrix::default();
                    reader.readMatrix(&mut matrix);
                    let drawable = data.getDrawable(reader);
                    valid!();
                    SkCanvas::drawDrawable_raw(canvas, drawable, Some(&matrix))?;
                }
                DRAW_DRRECT => {
                    let paint = data.requiredPaint(reader);
                    let mut outer = SkRRect::default();
                    let mut inner = SkRRect::default();
                    reader.readRRect(&mut outer);
                    reader.readRRect(&mut inner);
                    valid!();
                    SkCanvas::drawDRRect_raw(canvas, &outer, &inner, paint)?;
                }
                DRAW_OVAL => {
                    let paint = data.requiredPaint(reader);
                    let rect = reader.readRect();
                    valid!();
                    SkCanvas::drawOval_raw(canvas, &rect, paint)?;
                }
                DRAW_PAINT => {
                    let paint = data.requiredPaint(reader);
                    valid!();
                    SkCanvas::drawPaint_raw(canvas, paint)?;
                }
                DRAW_BEHIND_PAINT => {
                    let paint = data.requiredPaint(reader);
                    valid!();
                    super::SkCanvasPriv::DrawBehind(canvas, paint)?;
                }
                DRAW_PATH => {
                    let paint = data.requiredPaint(reader);
                    let path = data.getPath(reader);
                    valid!();
                    SkCanvas::drawPath_raw(canvas, path, paint)?;
                }
                DRAW_PICTURE => {
                    let pic = data.getPicture(reader);
                    valid!();
                    SkCanvas::drawPicture_raw(canvas, pic, None, None)?;
                }
                DRAW_PICTURE_MATRIX_PAINT => {
                    let paint = data.optionalPaint(reader);
                    let mut matrix = SkMatrix::default();
                    reader.readMatrix(&mut matrix);
                    let pic = data.getPicture(reader);
                    valid!();
                    SkCanvas::drawPicture_raw(canvas, pic, Some(&matrix), paint)?;
                }
                DRAW_POINTS => {
                    let paint = data.requiredPaint(reader);
                    let mode = reader
                        .checkRange(PointMode::kPoints_PointMode, PointMode::kPolygon_PointMode);
                    let count = reader.readInt() as usize;
                    let points = array!(SkPoint, count);
                    valid!();
                    SkCanvas::drawPoints_raw(canvas, mode, &points, paint)?;
                }
                DRAW_RECT => {
                    let paint = data.requiredPaint(reader);
                    let rect = reader.readRect();
                    valid!();
                    SkCanvas::drawRect_raw(canvas, &rect, paint)?;
                }
                DRAW_REGION => {
                    let paint = data.requiredPaint(reader);
                    let mut region = SkRegion::new();
                    reader.readRegion(&mut region);
                    valid!();
                    SkCanvas::drawRegion_raw(canvas, &region, paint)?;
                }
                DRAW_RRECT => {
                    let paint = data.requiredPaint(reader);
                    let mut rect = SkRRect::default();
                    reader.readRRect(&mut rect);
                    valid!();
                    SkCanvas::drawRRect_raw(canvas, &rect, paint)?;
                }
                DRAW_TEXT_BLOB => {
                    let paint = data.requiredPaint(reader);
                    let blob = data.getTextBlob(reader);
                    let x = reader.readScalar();
                    let y = reader.readScalar();
                    valid!();
                    SkCanvas::drawTextBlob_raw(canvas, blob, x, y, paint)?;
                }
                DRAW_SLUG => {
                    let paint = data.requiredPaint(reader);
                    let slug = data.getSlug(reader);
                    valid!();
                    SkCanvas::drawSlug_raw(canvas, slug, paint)?;
                }
                DRAW_SHADOW_REC => {
                    let path = data.getPath(reader);
                    let mut rec = super::SkDrawShadowInfo::SkDrawShadowRec {
                        fZPlaneParams: Default::default(),
                        fLightPos: Default::default(),
                        fLightRadius: 0.,
                        fAmbientColor: 0,
                        fSpotColor: 0,
                        fFlags: 0,
                    };
                    reader.readPoint3(&mut rec.fZPlaneParams);
                    reader.readPoint3(&mut rec.fLightPos);
                    rec.fLightRadius = reader.readScalar();
                    rec.fAmbientColor = reader.read32() as u32;
                    rec.fSpotColor = reader.read32() as u32;
                    rec.fFlags = reader.read32() as u32;
                    valid!();
                    SkCanvas::private_draw_shadow_rec_raw(canvas, path, &rec)?;
                }
                DRAW_VERTICES_OBJECT => {
                    let paint = data.requiredPaint(reader);
                    let vertices = data.getVertices(reader);
                    let boneCount = reader.readInt() as usize;
                    reader.skipCount(
                        boneCount,
                        core::mem::size_of::<super::SkVerticesPriv::SkVertices_DeprecatedBone>(),
                    );
                    let mode = read_blend(reader);
                    valid!();
                    if let Some(vertices) = vertices {
                        SkCanvas::drawVertices_raw(canvas, Some(vertices), mode, paint)?;
                    }
                }
                RESTORE => {
                    SkCanvas::restore_raw(canvas)?;
                }
                ROTATE => {
                    let degrees = reader.readScalar();
                    Pin::new_unchecked(canvas.as_mut()).rotate(degrees);
                }
                SAVE => {
                    Pin::new_unchecked(canvas.as_mut()).save();
                }
                SAVE_BEHIND => {
                    let flags = reader.readInt() as u32;
                    let subset = if flags & SAVEBEHIND_HAS_SUBSET != 0 {
                        Some(reader.readRect())
                    } else {
                        None
                    };
                    super::SkCanvasPriv::SaveBehind(canvas, subset.as_ref())?;
                }
                SCALE => {
                    let sx = reader.readScalar();
                    let sy = reader.readScalar();
                    Pin::new_unchecked(canvas.as_mut()).scale(sx, sy);
                }
                SET_M44 => {
                    let mut m = SkM44::default();
                    reader.readM44(&mut m);
                    Pin::new_unchecked(canvas.as_mut()).setMatrix44(&(initialMatrix.clone() * m));
                }
                SET_MATRIX => {
                    let mut matrix = SkMatrix::default();
                    reader.readMatrix(&mut matrix);
                    Pin::new_unchecked(canvas.as_mut())
                        .setMatrix44(&(initialMatrix.clone() * SkM44::from(&matrix)));
                }
                SKEW => {
                    let sx = reader.readScalar();
                    let sy = reader.readScalar();
                    Pin::new_unchecked(canvas.as_mut()).skew(sx, sy);
                }
                TRANSLATE => {
                    let dx = reader.readScalar();
                    let dy = reader.readScalar();
                    Pin::new_unchecked(canvas.as_mut()).translate(dx, dy);
                }
                DRAW_IMAGE | DRAW_IMAGE2 => {
                    let paint = data.optionalPaint(reader);
                    let image = data.getImageOwner(reader);
                    let loc = reader.readPoint();
                    let sampling = if op == DRAW_IMAGE2 {
                        reader.readSampling()
                    } else {
                        SkSamplingOptions::with_filter(SkFilterMode::kNearest)
                    };
                    valid!();
                    SkCanvas::drawImage_raw(canvas, image, loc.fX, loc.fY, &sampling, paint)?;
                }
                DRAW_IMAGE_LATTICE | DRAW_IMAGE_LATTICE2 => {
                    let paint = data.optionalPaint(reader);
                    let image = data.getImageOwner(reader);
                    let mut lattice = core::mem::MaybeUninit::<super::SkCanvas::Lattice>::uninit();
                    let ok = super::SkCanvasPriv::ReadLattice(reader, lattice.as_mut_ptr());
                    reader.validate(ok);
                    let dst = array!(SkRect, 1);
                    let filter = if op == DRAW_IMAGE_LATTICE2 {
                        core::mem::transmute(reader.read32LE(SkFilterMode::kLinear as u32) as i32)
                    } else {
                        SkFilterMode::kNearest
                    };
                    valid!();
                    SkCanvas::drawImageLattice_raw(
                        canvas,
                        image,
                        &lattice.assume_init(),
                        &dst[0],
                        filter,
                        paint,
                    )?;
                }
                DRAW_IMAGE_NINE => {
                    let paint = data.optionalPaint(reader);
                    let image = data.getImageOwner(reader);
                    let mut center = super::SkRect::SkIRect::default();
                    reader.readIRect(&mut center);
                    let dst = reader.readRect();
                    valid!();
                    SkCanvas::drawImageNine_raw(
                        canvas,
                        image,
                        &center,
                        &dst,
                        SkFilterMode::kNearest,
                        paint,
                    )?;
                }
                DRAW_IMAGE_RECT => {
                    let paint = data.optionalPaint(reader);
                    let image = data.getImageOwner(reader);
                    let src = get_rect_ptr(reader);
                    let dst = reader.readRect();
                    let mut constraint = SrcRectConstraint::kStrict_SrcRectConstraint;
                    if op == DRAW_IMAGE_RECT {
                        constraint = reader.checkRange(
                            SrcRectConstraint::kStrict_SrcRectConstraint,
                            SrcRectConstraint::kFast_SrcRectConstraint,
                        );
                    }
                    valid!();
                    if let Some(image) = image {
                        SkCanvas::drawImageRect_raw(
                            canvas,
                            image,
                            src.as_ref(),
                            &dst,
                            &SkSamplingOptions::with_filter(SkFilterMode::kNearest),
                            paint,
                            constraint,
                        )?;
                    }
                }
                DRAW_IMAGE_RECT2 => {
                    let paint = data.optionalPaint(reader);
                    let image = data.getImageOwner(reader);
                    let src = reader.readRect();
                    let dst = reader.readRect();
                    let sampling = reader.readSampling();
                    let constraint =
                        reader.read32LE(SrcRectConstraint::kFast_SrcRectConstraint as u32);
                    valid!();
                    if let Some(image) = image {
                        SkCanvas::drawImageRect_raw(
                            canvas,
                            image,
                            Some(&src),
                            &dst,
                            &sampling,
                            paint,
                            core::mem::transmute(constraint as i32),
                        )?;
                    }
                }
                DRAW_PATCH => {
                    let paint = data.requiredPaint(reader);
                    let cubics = array!(SkPoint, 12);
                    let flag = reader.readInt() as u32;
                    let colors = if flag & DRAW_VERTICES_HAS_COLORS != 0 {
                        Some(array!(u32, 4))
                    } else {
                        None
                    };
                    let tex = if flag & DRAW_VERTICES_HAS_TEXS != 0 {
                        Some(array!(SkPoint, 4))
                    } else {
                        None
                    };
                    let mut mode = SkBlendMode::kModulate;
                    if flag & DRAW_VERTICES_HAS_XFER != 0 {
                        let m = reader.readInt() as u32;
                        if m <= SkBlendMode::kLastMode as u32 {
                            mode = core::mem::transmute(m as u8);
                        }
                    }
                    valid!();
                    SkCanvas::drawPatch_raw(
                        canvas,
                        Some(cubics.as_slice().try_into().unwrap()),
                        colors.as_ref().map(|v| v.as_slice().try_into().unwrap()),
                        tex.as_ref().map(|v| v.as_slice().try_into().unwrap()),
                        mode,
                        paint,
                    )?;
                }
                DRAW_ATLAS => {
                    let paint = data.optionalPaint(reader);
                    let atlas = data.getImageOwner(reader);
                    let flags = reader.readUInt();
                    let count = reader.readUInt() as usize;
                    let xform = array!(super::SkRSXform::SkRSXform, count);
                    let tex = array!(SkRect, count);
                    let mut mode = SkBlendMode::kDst;
                    let colors = if flags & DRAW_ATLAS_HAS_COLORS != 0 {
                        let colors = array!(u32, count);
                        mode = read_blend(reader);
                        valid!();
                        Some(colors)
                    } else {
                        None
                    };
                    let cull = if flags & DRAW_ATLAS_HAS_CULL != 0 {
                        Some(array!(SkRect, 1)[0])
                    } else {
                        None
                    };
                    valid!();
                    let sampling = if flags & DRAW_ATLAS_HAS_SAMPLING != 0 {
                        let sampling = reader.readSampling();
                        valid!();
                        sampling
                    } else {
                        SkSamplingOptions::default()
                    };
                    SkCanvas::drawAtlas_raw(
                        canvas,
                        atlas,
                        &xform,
                        &tex,
                        colors.as_deref().unwrap_or(&[]),
                        mode,
                        &sampling,
                        cull.as_ref(),
                        paint,
                    )?;
                }
                DRAW_EDGEAA_QUAD => {
                    let rect = reader.readRect();
                    let aa = reader.read32() as u32;
                    let mut color = super::SkColor::SkColor4f {
                        fR: 0.,
                        fG: 0.,
                        fB: 0.,
                        fA: 0.,
                    };
                    reader.readColor4f(&mut color);
                    let blend = read_blend(reader);
                    valid!();
                    let hasClip = reader.readInt() != 0;
                    let clip = if hasClip {
                        Some(array!(SkPoint, 4))
                    } else {
                        None
                    };
                    valid!();
                    SkCanvas::experimental_DrawEdgeAAQuad_raw(
                        canvas,
                        &rect,
                        clip.as_ref().map(|v| v.as_slice().try_into().unwrap()),
                        super::SkCanvas::QuadAAFlags(aa),
                        &color,
                        blend,
                    )?;
                }
                DRAW_EDGEAA_IMAGE_SET | DRAW_EDGEAA_IMAGE_SET2 => {
                    const ENTRY_SIZE: usize = 4 * 4 + 2 * core::mem::size_of::<SkRect>() + 4;
                    const MATRIX_SIZE: usize = 9 * 4;
                    let count = reader.readInt();
                    if !reader.validate(count >= 0) {
                        return Ok(());
                    }
                    let paint = data.optionalPaint(reader);
                    let sampling = if op == DRAW_EDGEAA_IMAGE_SET2 {
                        reader.readSampling()
                    } else {
                        SkSamplingOptions::with_filter(SkFilterMode::kNearest)
                    };
                    let constraint = reader.checkRange(
                        SrcRectConstraint::kStrict_SrcRectConstraint,
                        SrcRectConstraint::kFast_SrcRectConstraint,
                    );
                    if !reader.validate(
                        (count as usize)
                            .checked_mul(ENTRY_SIZE)
                            .is_some_and(|n| n <= reader.available()),
                    ) {
                        return Ok(());
                    }
                    let mut expectedClipPointCount = 0i32;
                    let mut maxMatrixIndex = -1;
                    let mut set = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        if !reader.isValid() {
                            break;
                        }
                        let image = data.getImageOwner(reader).cloned();
                        let src = reader.readRect();
                        let dst = reader.readRect();
                        let matrix = reader.readInt();
                        let alpha = reader.readScalar();
                        let aa = reader.readUInt();
                        let clip = reader.readInt() != 0;
                        expectedClipPointCount =
                            expectedClipPointCount.wrapping_add(if clip { 4 } else { 0 });
                        maxMatrixIndex = maxMatrixIndex.max(matrix);
                        set.push(super::SkCanvas::ImageSetEntry::new(
                            image, &src, &dst, matrix, alpha, aa, clip,
                        ));
                    }
                    let dstClipPointCount = reader.readInt();
                    if !reader.validate(dstClipPointCount >= 0)
                        || !reader.validate(expectedClipPointCount == dstClipPointCount)
                    {
                        return Ok(());
                    }
                    let dstClips = if dstClipPointCount > 0 {
                        array!(SkPoint, dstClipPointCount as usize)
                    } else {
                        Vec::new()
                    };
                    let matrixCount = reader.readInt();
                    if !reader.validate(matrixCount >= 0)
                        || !reader.validate(maxMatrixIndex == matrixCount.wrapping_sub(1))
                        || !reader.validate(
                            (matrixCount as usize)
                                .checked_mul(MATRIX_SIZE)
                                .is_some_and(|n| n <= reader.available()),
                        )
                    {
                        return Ok(());
                    }
                    let mut matrices = Vec::with_capacity(matrixCount as usize);
                    for _ in 0..matrixCount {
                        if !reader.isValid() {
                            break;
                        }
                        let mut matrix = SkMatrix::default();
                        reader.readMatrix(&mut matrix);
                        matrices.push(matrix);
                    }
                    valid!();
                    SkCanvas::experimental_DrawEdgeAAImageSet_raw(
                        canvas, &set, &dstClips, &matrices, &sampling, paint, constraint,
                    )?;
                }
                SAVE_LAYER_SAVELAYERREC => {
                    let mut rec = super::SkCanvas::SaveLayerRec::with_backdrop(None, None, None, 0);
                    let flags = reader.readInt() as u32;
                    let bounds = if flags & SAVELAYERREC_HAS_BOUNDS != 0 {
                        Some(reader.readRect())
                    } else {
                        None
                    };
                    rec.fBounds = bounds.as_ref();
                    if flags & SAVELAYERREC_HAS_PAINT != 0 {
                        rec.fPaint = Some(data.requiredPaint(reader));
                    }
                    if flags & SAVELAYERREC_HAS_BACKDROP != 0 {
                        rec.fBackdrop = data.requiredPaint(reader).imageFilterOwner();
                    }
                    if flags & SAVELAYERREC_HAS_FLAGS != 0 {
                        rec.fSaveLayerFlags = reader.readInt() as u32;
                    }
                    if flags & SAVELAYERREC_HAS_CLIPMASK_OBSOLETE != 0 {
                        data.getImage(reader);
                    }
                    if flags & SAVELAYERREC_HAS_CLIPMATRIX_OBSOLETE != 0 {
                        let mut ignored = SkMatrix::default();
                        reader.readMatrix(&mut ignored);
                    }
                    if !reader.isVersionLT(super::SkPicturePriv::kBackdropScaleFactor)
                        && flags & SAVELAYERREC_HAS_BACKDROP_SCALE != 0
                    {
                        super::SkCanvasPriv::SetBackdropScaleFactor(&mut rec, reader.readScalar());
                    }
                    let mut filters = Vec::new();
                    if !reader.isVersionLT(super::SkPicturePriv::kMultipleFiltersOnSaveLayer)
                        && flags & SAVELAYERREC_HAS_MULTIPLE_FILTERS != 0
                    {
                        let filterCount = reader.readUInt() as i32;
                        reader.validate(
                            filterCount > 0 && filterCount <= super::SkCanvas::kMaxFiltersPerLayer,
                        );
                        valid!();
                        filters.reserve(filterCount as usize);
                        for _ in 0..filterCount {
                            filters.push(data.requiredPaint(reader).refImageFilter());
                        }
                        rec.fFilters = &filters;
                    }
                    if !reader.isVersionLT(super::SkPicturePriv::kSaveLayerBackdropTileMode)
                        && flags & SAVELAYERREC_HAS_BACKDROP_TILEMODE != 0
                    {
                        let mode = reader.read32LE(
                            crate::include::core::SkTileMode::SkTileMode::kLastTileMode as u32,
                        );
                        rec.fBackdropTileMode = core::mem::transmute(mode as i32);
                    }
                    valid!();
                    Pin::new_unchecked(canvas.as_mut()).saveLayer(&rec)?;
                }
                _ => {
                    reader.validate(false);
                }
            }
            Ok(())
        }
    }
}
