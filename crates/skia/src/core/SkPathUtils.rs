//! CPU paint-to-path entry points. Concrete native SkPaint and MatrixPriv resScale
//! remain shared dependencies; no substitute paint representation is used here.
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkMatrixPriv::SkMatrixPriv;
use crate::src::core::SkPaint::SkPaint;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkRect::SkRect;
use crate::src::core::SkStrokeRec::SkStrokeRec;
// Source: src/core/SkPathUtils.cpp:21-29
pub fn FillPathWithPaint(
    origSrc: &SkPath,
    paint: &SkPaint,
    builder: &mut SkPathBuilder,
    cullRect: Option<&SkRect>,
    ctm: &SkMatrix,
) -> bool {
    if !origSrc.isFinite() {
        builder.reset();
        return false;
    }
    let resScale = SkMatrixPriv::ComputeResScaleForStroking(ctm);
    let mut rec = SkStrokeRec::from_paint(paint, resScale);
    // Native SK_BUILD_FOR_FUZZER timeout shortcut is outside the CPU production branch.
    // Source: src/core/SkPathUtils.cpp:38-53
    let mut srcPtr = origSrc;
    let mut pathStorage = SkPath::default();
    let pe = paint.getPathEffect();
    if let Some(pe) = pe {
        if pe.filterPath(builder, origSrc, &mut rec, cullRect, ctm) {
            pathStorage = builder.detach(None);
            srcPtr = &pathStorage;
        }
    }
    if !rec.applyToPath(builder, srcPtr) {
        builder.assign_path(srcPtr);
    }
    if !builder.isFinite() {
        builder.reset();
    }
    !rec.isHairlineStyle()
}
// Source: src/core/SkPathUtils.cpp:55-57
pub fn FillPathWithPaintWithoutCTM(src: &SkPath, paint: &SkPaint, dst: &mut SkPathBuilder) -> bool {
    FillPathWithPaint(src, paint, dst, None, &SkMatrix::default())
}
// Source: src/core/SkPathUtils.cpp:59-66
pub fn FillPathWithPaintToPath(
    src: &SkPath,
    paint: &SkPaint,
    isFillPtr: Option<&mut bool>,
) -> SkPath {
    let mut builder = SkPathBuilder::new();
    let isFill = FillPathWithPaintWithoutCTM(src, paint, &mut builder);
    if let Some(isFillPtr) = isFillPtr {
        *isFillPtr = isFill;
    }
    builder.detach(None)
}
