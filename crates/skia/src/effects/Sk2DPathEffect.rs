// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::include::core::SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject};
use crate::src::core::{
    SkFlattenable::{Factory, FactoryResult, SkFlattenable, SkFlattenableView, Type},
    SkMatrix::SkMatrix,
    SkPath::{AddPathMode, SkPath},
    SkPathBuilder::SkPathBuilder,
    SkPathEffect::SkPathEffect,
    SkPathEffectBase::SkPathEffectBase,
    SkPoint::SkPoint,
    SkReadBuffer::SkReadBuffer,
    SkRect::{SkIRect, SkRect},
    SkRegion::{RegionIterator, SkRegion},
    SkStrokeRec::SkStrokeRec,
    SkWriteBuffer::SkWriteBuffer,
};
// Source: include/effects/Sk2DPathEffect.h:19-31
pub struct SkLine2DPathEffect;
pub struct SkPath2DPathEffect;
// Source: src/effects/Sk2DPathEffect.cpp:26-114
struct Sk2DPathEffectState {
    fMatrix: SkMatrix,
    fInverse: SkMatrix,
    fMatrixIsInvertible: bool,
    base: SkRefCnt,
}
impl Sk2DPathEffectState {
    fn new(mat: &SkMatrix) -> Self {
        let matrix = mat.clone();
        let mut inverse = SkMatrix::default();
        let invertible = matrix.invert_to(Some(&mut inverse));
        Self {
            fMatrix: matrix,
            fInverse: inverse,
            fMatrixIsInvertible: invertible,
            base: SkRefCnt::new(),
        }
    }
}
// These are the original abstract base's native virtuals and its qualified
// filter implementation. Each concrete object embeds the real base state.
trait Sk2DPathEffect: SkPathEffectBase {
    fn inherited(&self) -> &Sk2DPathEffectState;
    fn getMatrix(&self) -> &SkMatrix {
        &self.inherited().fMatrix
    }
    fn flatten_base(&self, buffer: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        buffer.writeMatrix(self.getMatrix());
        Ok(())
    }
    fn begin(&self, _: &SkIRect, _: &mut SkPathBuilder) {}
    fn next(&self, _: &SkPoint, _: i32, _: i32, _: &mut SkPathBuilder) {}
    fn end(&self, _: &mut SkPathBuilder) {}
    fn nextSpan(&self, mut x: i32, y: i32, mut ucount: i32, builder: &mut SkPathBuilder) {
        let inherited = self.inherited();
        if !inherited.fMatrixIsInvertible {
            return;
        }
        #[cfg(sk_build_for_fuzzer)]
        if ucount > 100 {
            return;
        }
        let mat = &inherited.fMatrix;
        let mut src = SkPoint::make(x as f32 + 0.5, y as f32 + 0.5);
        loop {
            let dst = mat.mapPoint(src);
            self.next(&dst, x, y, builder);
            x = x.wrapping_add(1);
            src.fX += 1.;
            ucount -= 1;
            if ucount <= 0 {
                break;
            }
        }
    }
    fn filter_spans(&self, dst: &mut SkPathBuilder, src: &SkPath) -> bool {
        let inherited = self.inherited();
        if !inherited.fMatrixIsInvertible {
            return false;
        }
        let tmp = src.makeTransform(&inherited.fInverse);
        let ir = tmp.getBounds().round();
        if !ir.is_empty() {
            self.begin(&ir, dst);
            let mut rgn = SkRegion::new();
            rgn.set_path(&tmp, &SkRegion::from_rect(ir));
            let mut iter = RegionIterator::new(&rgn);
            while !iter.done() {
                let rect = *iter.rect();
                #[cfg(sk_build_for_fuzzer)]
                if rect.height() > 100 {
                    iter.next();
                    continue;
                }
                for y in rect.fTop..rect.fBottom {
                    self.nextSpan(rect.fLeft, y, rect.width(), dst);
                }
                iter.next();
            }
            self.end(dst);
        }
        true
    }
}
// Source: src/effects/Sk2DPathEffect.cpp:118-168
struct SkLine2DPathEffectImpl {
    fWidth: f32,
    inherited: Sk2DPathEffectState,
}
impl SkLine2DPathEffectImpl {
    fn new(width: f32, matrix: &SkMatrix) -> Self {
        crate::SkASSERT!(@source b"src/effects/Sk2DPathEffect.cpp\0".as_ptr(),124; @condition b"width >= 0\0".as_ptr(); width>=0.);
        Self {
            fWidth: width,
            inherited: Sk2DPathEffectState::new(matrix),
        }
    }
    fn CreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        let mut matrix = SkMatrix::default();
        buffer.readMatrix(&mut matrix);
        let width = buffer.readScalar();
        Ok(crate::src::core::SkPathEffect::into_flattenable(
            SkLine2DPathEffect::Make(width, &matrix),
        ))
    }
}
impl Sk2DPathEffect for SkLine2DPathEffectImpl {
    fn inherited(&self) -> &Sk2DPathEffectState {
        &self.inherited
    }
    fn nextSpan(&self, u: i32, v: i32, ucount: i32, dst: &mut SkPathBuilder) {
        if ucount > 1 {
            let src = [
                SkPoint::make(u as f32 + 0.5, v as f32 + 0.5),
                SkPoint::make(u.wrapping_add(ucount) as f32 + 0.5, v as f32 + 0.5),
            ];
            let mut dstP = [SkPoint::make(0., 0.); 2];
            self.inherited.fMatrix.mapPoints(&mut dstP, &src);
            dst.moveTo(dstP[0]);
            dst.lineTo(dstP[1]);
        }
    }
}
impl SkPathEffectBase for SkLine2DPathEffectImpl {
    fn onFilterPath(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        rec: &mut SkStrokeRec,
        _: Option<&SkRect>,
        _: &SkMatrix,
    ) -> bool {
        if self.filter_spans(dst, src) {
            rec.setStrokeStyle(self.fWidth, false);
            true
        } else {
            false
        }
    }
    fn computeFastBounds(&self, _: Option<&mut SkRect>) -> bool {
        false
    }
}
// Source: src/effects/Sk2DPathEffect.cpp:172-201
struct SkPath2DPathEffectImpl {
    fPath: SkPath,
    inherited: Sk2DPathEffectState,
}
impl SkPath2DPathEffectImpl {
    fn new(matrix: &SkMatrix, path: &SkPath) -> Self {
        Self {
            fPath: path.clone(),
            inherited: Sk2DPathEffectState::new(matrix),
        }
    }
    fn CreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        let mut matrix = SkMatrix::default();
        buffer.readMatrix(&mut matrix);
        if let Some(path) = buffer.readPath() {
            return Ok(crate::src::core::SkPathEffect::into_flattenable(
                SkPath2DPathEffect::Make(&matrix, &path),
            ));
        }
        Ok(None)
    }
}
impl Sk2DPathEffect for SkPath2DPathEffectImpl {
    fn inherited(&self) -> &Sk2DPathEffectState {
        &self.inherited
    }
    fn next(&self, loc: &SkPoint, _: i32, _: i32, dst: &mut SkPathBuilder) {
        dst.addPathOffset(
            &self.fPath,
            loc.fX,
            loc.fY,
            AddPathMode::kAppend_AddPathMode,
        );
    }
}
impl SkPathEffectBase for SkPath2DPathEffectImpl {
    fn onFilterPath(
        &self,
        dst: &mut SkPathBuilder,
        src: &SkPath,
        _: &mut SkStrokeRec,
        _: Option<&SkRect>,
        _: &SkMatrix,
    ) -> bool {
        self.filter_spans(dst, src)
    }
    fn computeFastBounds(&self, _: Option<&mut SkRect>) -> bool {
        false
    }
}
// Source: src/effects/Sk2DPathEffect.cpp:205-222
impl SkLine2DPathEffect {
    pub fn Make(width: f32, matrix: &SkMatrix) -> sk_sp<dyn SkPathEffect> {
        if !(width >= 0.) {
            return sk_sp::default();
        }
        let object: Box<dyn SkPathEffect> = Box::new(SkLine2DPathEffectImpl::new(width, matrix));
        unsafe { sk_sp::from_raw(Box::into_raw(object)) }
    }
    /// Safety: exclusive native registration phase, before once publication;
    /// no concurrent lookup/registration, with space in the fixed factory table.
    pub unsafe fn RegisterFlattenables() {
        unsafe {
            #[cfg(not(sk_disable_effect_deserialization))]
            crate::src::core::SkFlattenable::Register(
                b"SkLine2DPathEffectImpl",
                SkLine2DPathEffectImpl::CreateProc,
            );
        }
    }
}
impl SkPath2DPathEffect {
    pub fn Make(matrix: &SkMatrix, path: &SkPath) -> sk_sp<dyn SkPathEffect> {
        let object: Box<dyn SkPathEffect> = Box::new(SkPath2DPathEffectImpl::new(matrix, path));
        unsafe { sk_sp::from_raw(Box::into_raw(object)) }
    }
    /// Safety: exclusive native registration phase, before once publication;
    /// no concurrent lookup/registration, with space in the fixed factory table.
    pub unsafe fn RegisterFlattenables() {
        unsafe {
            #[cfg(not(sk_disable_effect_deserialization))]
            crate::src::core::SkFlattenable::Register(
                b"SkPath2DPathEffectImpl",
                SkPath2DPathEffectImpl::CreateProc,
            );
        }
    }
}
// Structural inherited owner/metadata projection; both classes retain exactly
// one complete allocation and the original embedded intrusive counter.
macro_rules! owner {
    ($ty:ty,$name:literal,$flatten:expr) => {
        unsafe impl SkRefCntObject for $ty {
            fn internal_dispose_raw_proc(
                &self,
            ) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
                crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
            }
            fn ref_cnt_base(&self) -> &SkRefCntBase {
                &self.inherited.base
            }
            fn ref_(&self) {
                SkRefCnt::ref_(&self.inherited.base);
            }
        }
        impl SkFlattenable for $ty {
            unsafe fn project_path_effect_owner(
                &self,
                allocation: core::ptr::NonNull<()>,
            ) -> Option<core::ptr::NonNull<dyn SkPathEffect>> {
                let concrete: *mut Self = allocation.as_ptr().cast();
                Some(unsafe {
                    core::ptr::NonNull::new_unchecked(concrete as *mut dyn SkPathEffect)
                })
            }
        }
        impl SkFlattenableView for $ty {
            fn getFactory(&self) -> Option<Factory> {
                Some(Self::CreateProc)
            }
            fn getTypeName(&self) -> Option<&str> {
                Some($name)
            }
            fn getFlattenableType(&self) -> Type {
                Type::kSkPathEffect_Type
            }
            fn flatten(&self, buffer: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
                ($flatten)(self, buffer);
                Ok(())
            }
        }
    };
}
// Source: src/effects/Sk2DPathEffect.cpp:156-159
owner!(
    SkLine2DPathEffectImpl,
    "SkLine2DPathEffect",
    |effect: &SkLine2DPathEffectImpl, buffer: &mut dyn SkWriteBuffer| {
        buffer.writeMatrix(&effect.inherited.fMatrix);
        buffer.writeScalar(effect.fWidth);
    }
);
// Source: src/effects/Sk2DPathEffect.cpp:189-192
owner!(
    SkPath2DPathEffectImpl,
    "SkPath2DPathEffect",
    |effect: &SkPath2DPathEffectImpl, buffer: &mut dyn SkWriteBuffer| {
        buffer.writeMatrix(&effect.inherited.fMatrix);
        buffer.writePath(&effect.fPath);
    }
);
