// Copyright 2006 The Android Open Source Project. BSD-style license. Unverified translation.
use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::{
    SkFlattenable::{
        self as flat, FactoryResult, FlattenableOwner, SkFlattenable, SkFlattenableView,
    },
    SkMatrix::SkMatrix,
    SkPath::SkPath,
    SkPathBuilder::SkPathBuilder,
    SkPathEffect::SkPathEffect,
    SkPathEffectBase::SkPathEffectBase,
    SkPoint::SkPoint,
    SkRect::SkRect,
    SkStrokeRec::SkStrokeRec,
};
// Source: src/effects/SkDiscretePathEffect.cpp:42-72
struct LCGRandom {
    fSeed: u32,
}
impl LCGRandom {
    fn new(seed: u32) -> Self {
        Self { fSeed: seed }
    }
    fn nextU(&mut self) -> u32 {
        let r = self.fSeed.wrapping_mul(1664525).wrapping_add(1013904223);
        self.fSeed = r;
        r
    }
    fn nextS(&mut self) -> i32 {
        self.nextU() as i32
    }
    fn nextSFixed1(&mut self) -> i32 {
        self.nextS() >> 15
    }
    fn nextSScalar1(&mut self) -> f32 {
        self.nextSFixed1() as f32 * (1.0 / 65536.0)
    }
}
// Source: src/effects/SkDiscretePathEffect.cpp:74-80
fn Perterb(p: &mut SkPoint, tangent: SkPoint, scale: f32) {
    let mut normal = tangent;
    crate::src::core::SkPointPriv::SkPointPriv::RotateCCW(&mut normal);
    normal.set_length(scale);
    p.fX += normal.fX;
    p.fY += normal.fY;
}
// Source: src/effects/SkDiscretePathEffect.cpp:82-91,185-193
struct SkDiscretePathEffectImpl {
    fSegLength: f32,
    fPerterb: f32,
    fSeedAssist: u32,
    fRefCnt: SkRefCnt,
}
impl SkDiscretePathEffectImpl {
    fn new(segLength: f32, deviation: f32, seedAssist: u32) -> Self {
        crate::SkASSERT!(@source b"src/effects/SkDiscretePathEffect.cpp\0".as_ptr(),85; @condition b"SkIsFinite(segLength)\0".as_ptr(); segLength.is_finite());
        crate::SkASSERT!(@source b"src/effects/SkDiscretePathEffect.cpp\0".as_ptr(),86; @condition b"SkIsFinite(deviation)\0".as_ptr(); deviation.is_finite());
        crate::SkASSERT!(@source b"src/effects/SkDiscretePathEffect.cpp\0".as_ptr(),87; @condition b"segLength > SK_ScalarNearlyZero\0".as_ptr(); segLength>1.0/4096.0);
        Self {
            fSegLength: segLength,
            fPerterb: deviation,
            fSeedAssist: seedAssist,
            fRefCnt: SkRefCnt::new(),
        }
    }
    // Source: src/effects/SkDiscretePathEffect.cpp:165-170
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> FactoryResult {
        let seg = buffer.readScalar();
        let deviation = buffer.readScalar();
        let seed = buffer.readUInt();
        let mut owner = Make(seg, deviation, seed);
        if !owner.is_some() {
            return Ok(None);
        }
        let raw = owner.release().unwrap();
        let p = raw.as_ptr() as *mut SkDiscretePathEffectImpl;
        let flat: *mut dyn SkFlattenable = p;
        Ok(Some(FlattenableOwner::Intrusive(unsafe {
            sk_sp::from_raw(flat)
        })))
    }
}
unsafe impl SkRefCntObject for SkDiscretePathEffectImpl {
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }
    fn ref_cnt_base(&self) -> &SkRefCntBase {
        &self.fRefCnt
    }
    fn ref_(&self) {
        SkRefCnt::ref_(&self.fRefCnt);
    }
}
impl SkPathEffect for SkDiscretePathEffectImpl {}
impl SkFlattenable for SkDiscretePathEffectImpl {
    unsafe fn project_path_effect_owner(
        &self,
        allocation: core::ptr::NonNull<()>,
    ) -> Option<core::ptr::NonNull<dyn SkPathEffect>> {
        let p: *mut Self = allocation.as_ptr().cast();
        Some(core::ptr::NonNull::new_unchecked(
            p as *mut dyn SkPathEffect,
        ))
    }
}
impl SkFlattenableView for SkDiscretePathEffectImpl {
    fn getFactory(&self) -> Option<flat::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkDiscretePathEffect")
    }
    fn getFlattenableType(&self) -> flat::Type {
        flat::Type::kSkPathEffect_Type
    }
    // Source: src/effects/SkDiscretePathEffect.cpp:172-176
    fn flatten(
        &self,
        w: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        w.writeScalar(self.fSegLength);
        w.writeScalar(self.fPerterb);
        w.writeUInt(self.fSeedAssist);
        Ok(())
    }
}
impl SkPathEffectBase for SkDiscretePathEffectImpl {
    // Source: src/effects/SkDiscretePathEffect.cpp:93-153
    fn onFilterPath(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        _bounds: Option<&SkRect>,
        _ctm: &SkMatrix,
    ) -> bool {
        let fill = rec.isFillStyle();
        let mut meas = crate::src::core::SkPathMeasure::SkPathMeasure::new(src, fill, 1.0);
        let seed = self.fSeedAssist
            ^ (crate::src::core::SkFloatingPoint::sk_float_round2int(meas.getLength()) as u32);
        let mut rand = LCGRandom::new(seed ^ seed.rotate_left(16));
        let scale = self.fPerterb;
        // Output scratch is observed only after getPosTan reports an actual write.
        let mut p = SkPoint::default();
        let mut v = SkPoint::default();
        loop {
            let length = meas.getLength();
            #[cfg(SK_BUILD_FOR_FUZZER)]
            if length > 1000.0 {
                return false;
            }
            if self.fSegLength * (2 + fill as i32) as f32 > length {
                meas.getSegment(0.0, length, dst, true);
            } else {
                let mut n =
                    crate::src::core::SkFloatingPoint::sk_float_round2int(length / self.fSegLength)
                        .min(100000);
                let delta = length / n as f32;
                let mut distance = 0.0;
                if meas.isClosed() {
                    n -= 1;
                    distance += delta / 2.0;
                }
                if meas.getPosTan(distance, Some(&mut p), Some(&mut v)) {
                    Perterb(&mut p, v, rand.nextSScalar1() * scale);
                    dst.moveTo(p);
                }
                loop {
                    n -= 1;
                    if n < 0 {
                        break;
                    }
                    distance += delta;
                    if meas.getPosTan(distance, Some(&mut p), Some(&mut v)) {
                        Perterb(&mut p, v, rand.nextSScalar1() * scale);
                        dst.lineTo(p);
                    }
                }
                if meas.isClosed() {
                    dst.close();
                }
            }
            if !meas.nextContour() {
                break;
            }
        }
        true
    }
    // Source: src/effects/SkDiscretePathEffect.cpp:155-163
    fn computeFastBounds(&self, bounds: Option<&mut SkRect>) -> bool {
        if let Some(bounds) = bounds {
            let d = self.fPerterb.abs();
            bounds.outset(d, d);
        }
        true
    }
}
// Source: include/effects/SkDiscretePathEffect.h:33-35
pub fn MakeDefault(segLength: f32, deviation: f32) -> sk_sp<dyn SkPathEffect> {
    Make(segLength, deviation, 0)
}
// Source: src/effects/SkDiscretePathEffect.cpp:199-209
pub fn Make(segLength: f32, deviation: f32, seedAssist: u32) -> sk_sp<dyn SkPathEffect> {
    if !segLength.is_finite() || !deviation.is_finite() || segLength <= 1.0 / 4096.0 {
        return sk_sp::default();
    }
    let p: Box<dyn SkPathEffect> = Box::new(SkDiscretePathEffectImpl::new(
        segLength, deviation, seedAssist,
    ));
    unsafe { sk_sp::from_raw(Box::into_raw(p)) }
}
// Source: src/effects/SkDiscretePathEffect.cpp:211-213
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn RegisterFlattenables() {
    unsafe {
        flat::Register(
            b"SkDiscretePathEffect",
            SkDiscretePathEffectImpl::CreateProc,
        );
    }
}
