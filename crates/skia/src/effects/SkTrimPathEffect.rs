// Copyright 2018 Google LLC. BSD-style license.
use super::SkTrimPE::SkTrimPE;
use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt};
use crate::src::core::{
    SkFlattenable::FactoryResult, SkMatrix::SkMatrix, SkPath::SkPath, SkPathBuilder::SkPathBuilder,
    SkPathEffect::SkPathEffect, SkPathEffectBase::SkPathEffectBase, SkPathMeasure::SkPathMeasure,
    SkReadBuffer::SkReadBuffer, SkRect::SkRect, SkStrokeRec::SkStrokeRec,
    SkWriteBuffer::SkWriteBuffer,
};
// Source: include/effects/SkTrimPathEffect.h:17-42
pub struct SkTrimPathEffect;
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    kNormal,
    kInverted,
}
// Source: src/effects/SkTrimPathEffect.cpp:34-60
fn add_segments(
    src: &SkPath,
    start: f32,
    stop: f32,
    dst: &mut SkPathBuilder,
    requires_moveto: bool,
) -> usize {
    crate::SkASSERT!(@source b"src/effects/SkTrimPathEffect.cpp\0".as_ptr(),36; @condition b"start < stop\0".as_ptr(); start<stop);
    let mut measure = SkPathMeasure::new(src, false, 1.0);
    let mut current_segment_offset = 0.;
    let mut contour_count = 1;
    loop {
        let next_offset = current_segment_offset + measure.getLength();
        if start < next_offset {
            measure.getSegment(
                start - current_segment_offset,
                stop - current_segment_offset,
                dst,
                requires_moveto,
            );
            if stop <= next_offset {
                break;
            }
        }
        contour_count += 1;
        current_segment_offset = next_offset;
        if !measure.nextContour() {
            break;
        }
    }
    contour_count
}
// Source: src/effects/SkTrimPathEffect.cpp:64-65
impl SkTrimPE {
    pub(crate) fn new(startT: f32, stopT: f32, mode: Mode) -> Self {
        Self {
            fStartT: startT,
            fStopT: stopT,
            fMode: mode,
            base: SkRefCnt::new(),
        }
    }
}
// Source: src/effects/SkTrimPathEffect.cpp:67-115
impl SkPathEffectBase for SkTrimPE {
    fn onFilterPath(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        _: &mut SkStrokeRec,
        _: Option<&SkRect>,
        _: &SkMatrix,
    ) -> bool {
        if self.fStartT >= self.fStopT {
            crate::SkASSERT!(@source b"src/effects/SkTrimPathEffect.cpp\0".as_ptr(),70; @condition b"fMode == SkTrimPathEffect::Mode::kNormal\0".as_ptr(); self.fMode==Mode::kNormal);
            return true;
        }
        let mut len = 0.;
        let mut meas = SkPathMeasure::new(src, false, 1.0);
        loop {
            len += meas.getLength();
            if !meas.nextContour() {
                break;
            }
        }
        let arcStart = len * self.fStartT;
        let arcStop = len * self.fStopT;
        if self.fMode == Mode::kNormal {
            if arcStart < arcStop {
                add_segments(src, arcStart, arcStop, dst, true);
            }
        } else {
            let mut requires_moveto = true;
            if arcStop < len {
                let contour_count = add_segments(src, arcStop, len, dst, true);
                if contour_count == 1 && src.isLastContourClosed() {
                    requires_moveto = false;
                }
            }
            if 0. < arcStart {
                add_segments(src, 0., arcStart, dst, requires_moveto);
            }
        }
        true
    }
    // Structural qualified virtual projection; native inline body stays in SkTrimPE.rs.
    fn computeFastBounds(&self, bounds: Option<&mut SkRect>) -> bool {
        self.computeFastBounds_impl(bounds)
    }
}
// Source: src/effects/SkTrimPathEffect.cpp:117-130
impl SkTrimPE {
    pub(crate) fn flatten_impl(
        &self,
        buffer: &mut dyn SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeScalar(self.fStartT);
        buffer.writeScalar(self.fStopT);
        buffer.writeUInt(self.fMode as u32);
        Ok(())
    }
    pub(crate) fn CreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        let start = buffer.readScalar();
        let stop = buffer.readScalar();
        let mode = buffer.readUInt();
        Ok(crate::src::core::SkPathEffect::into_flattenable(
            SkTrimPathEffect::Make(
                start,
                stop,
                if mode & 1 != 0 {
                    Mode::kInverted
                } else {
                    Mode::kNormal
                },
            ),
        ))
    }
}
// Source: src/effects/SkTrimPathEffect.cpp:134-151
impl SkTrimPathEffect {
    pub fn Make(mut startT: f32, mut stopT: f32, mode: Mode) -> sk_sp<dyn SkPathEffect> {
        if !startT.is_finite() || !stopT.is_finite() {
            return sk_sp::default();
        }
        if startT <= 0. && stopT >= 1. && mode == Mode::kNormal {
            return sk_sp::default();
        }
        startT = crate::include::private::SkTPin::SkTPin(startT, 0., 1.);
        stopT = crate::include::private::SkTPin::SkTPin(stopT, 0., 1.);
        if startT >= stopT && mode == Mode::kInverted {
            return sk_sp::default();
        }
        let object: Box<dyn SkPathEffect> = Box::new(SkTrimPE::new(startT, stopT, mode));
        unsafe { sk_sp::from_raw(Box::into_raw(object)) }
    }
    // Source: include/effects/SkTrimPathEffect.h:42-42
    pub fn Make_default(startT: f32, stopT: f32) -> sk_sp<dyn SkPathEffect> {
        Self::Make(startT, stopT, Mode::kNormal)
    }
}
