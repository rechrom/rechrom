use crate::include::core::SkScalar::{SK_ScalarNearlyZero, SkScalarNearlyZero};
use crate::src::core::SkPaint::Cap;
use crate::src::core::SkPathBuilder::SkPathBuilder;
use crate::src::core::SkPoint::{SkPoint, SkVector};
use crate::src::core::SkPointPriv::SkPointPriv;
// Source: src/core/SkStrokerPriv.h:18-21
// Function substitutions for C preprocessor coordinate expressions; each operand evaluated once.
pub(crate) fn CWX(_x: f32, y: f32) -> f32 {
    -y
}
pub(crate) fn CWY(x: f32, _y: f32) -> f32 {
    x
}
pub(crate) fn CCWX(_x: f32, y: f32) -> f32 {
    y
}
pub(crate) fn CCWY(x: f32, _y: f32) -> f32 {
    -x
}
// Source: src/core/SkStrokerPriv.h:23-23
pub(crate) const CUBIC_ARC_FACTOR: f32 = (std::f32::consts::SQRT_2 - 1.0) * 4.0 / 3.0;
// Source: src/core/SkStrokerPriv.h:26-26
pub(crate) const QUAD_STROKE_APPROX_EXTENDED_DEBUGGING: bool = false;
// Source: src/core/SkStrokerPriv.h:30-34
pub type CapProc = fn(&mut SkPathBuilder, SkPoint, SkVector, SkPoint, bool);
// Source: src/core/SkStrokerPriv.h:36-41
pub type JoinProc = for<'a> fn(
    &'a mut SkPathBuilder,
    &'a mut SkPathBuilder,
    SkVector,
    SkPoint,
    SkVector,
    f32,
    f32,
    bool,
    bool,
);
// Source: src/core/SkStrokerPriv.cpp:17-20
pub(crate) fn ButtCapper(
    sink: &mut SkPathBuilder,
    _pivot: SkPoint,
    _normal: SkVector,
    stop: SkPoint,
    _: bool,
) {
    sink.lineTo(stop);
}
// Source: src/core/SkStrokerPriv.cpp:22-31
pub(crate) fn RoundCapper(
    sink: &mut SkPathBuilder,
    pivot: SkPoint,
    normal: SkVector,
    stop: SkPoint,
    _: bool,
) {
    let mut parallel = normal;
    SkPointPriv::RotateCW(&mut parallel);
    let projectedCenter = pivot + parallel;
    sink.conicTo(
        projectedCenter + normal,
        projectedCenter,
        std::f32::consts::FRAC_1_SQRT_2,
    );
    sink.conicTo(
        projectedCenter - normal,
        stop,
        std::f32::consts::FRAC_1_SQRT_2,
    );
}
// Source: src/core/SkStrokerPriv.cpp:33-46
pub(crate) fn SquareCapper(
    sink: &mut SkPathBuilder,
    pivot: SkPoint,
    normal: SkVector,
    stop: SkPoint,
    extendLastPt: bool,
) {
    let mut parallel = normal;
    SkPointPriv::RotateCW(&mut parallel);
    if extendLastPt {
        // SkPathBuilder.h:954-956 inlines setLastPoint as setPoint(points().size()-1,p).
        sink.setPoint(
            sink.points().len().wrapping_sub(1),
            pivot + normal + parallel,
        );
        sink.lineTo(pivot - normal + parallel);
    } else {
        sink.lineTo(pivot + normal + parallel);
        sink.lineTo(pivot - normal + parallel);
        sink.lineTo(stop);
    }
}
// Source: src/core/SkStrokerPriv.cpp:50-52
pub(crate) fn is_clockwise(before: SkVector, after: SkVector) -> bool {
    before.fX * after.fY > before.fY * after.fX
}
// Source: src/core/SkStrokerPriv.cpp:54-59
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AngleType {
    kNearly180_AngleType,
    kSharp_AngleType,
    kShallow_AngleType,
    kNearlyLine_AngleType,
}
// Source: src/core/SkStrokerPriv.cpp:61-70
pub(crate) fn Dot2AngleType(dot: f32) -> AngleType {
    if dot >= 0.0 {
        if SkScalarNearlyZero(1.0 - dot, SK_ScalarNearlyZero) {
            AngleType::kNearlyLine_AngleType
        } else {
            AngleType::kShallow_AngleType
        }
    } else {
        if SkScalarNearlyZero(1.0 + dot, SK_ScalarNearlyZero) {
            AngleType::kNearly180_AngleType
        } else {
            AngleType::kSharp_AngleType
        }
    }
}
// Source: src/core/SkStrokerPriv.cpp:72-84
fn HandleInnerJoin(inner: &mut SkPathBuilder, pivot: SkPoint, after: SkVector) {
    inner.lineTo(pivot);
    inner.lineTo(SkPoint::make(pivot.fX - after.fX, pivot.fY - after.fY));
}
// Source: src/core/SkStrokerPriv.cpp:86-101
pub(crate) fn BluntJoiner<'a>(
    mut outer: &'a mut SkPathBuilder,
    mut inner: &'a mut SkPathBuilder,
    beforeUnitNormal: SkVector,
    pivot: SkPoint,
    afterUnitNormal: SkVector,
    radius: f32,
    _invMiterLimit: f32,
    _: bool,
    _: bool,
) {
    let mut after = afterUnitNormal * radius;
    if !is_clockwise(beforeUnitNormal, afterUnitNormal) {
        std::mem::swap(&mut outer, &mut inner);
        after.negate();
    }
    outer.lineTo(SkPoint::make(pivot.fX + after.fX, pivot.fY + after.fY));
    HandleInnerJoin(inner, pivot, after);
}
// Source: src/core/SkStrokerPriv.cpp:103-137
pub(crate) fn RoundJoiner<'a>(
    mut outer: &'a mut SkPathBuilder,
    mut inner: &'a mut SkPathBuilder,
    beforeUnitNormal: SkVector,
    pivot: SkPoint,
    afterUnitNormal: SkVector,
    radius: f32,
    _invMiterLimit: f32,
    _: bool,
    _: bool,
) {
    use crate::include::core::SkPathTypes::SkPathDirection;
    use crate::src::core::SkGeometry::SkConic;
    use crate::src::core::SkMatrix::SkMatrix;
    let dotProd = SkPoint::dot_product(beforeUnitNormal, afterUnitNormal);
    let angleType = Dot2AngleType(dotProd);
    if angleType == AngleType::kNearlyLine_AngleType {
        return;
    }
    let mut before = beforeUnitNormal;
    let mut after = afterUnitNormal;
    let mut dir = SkPathDirection::kCW;
    if !is_clockwise(before, after) {
        std::mem::swap(&mut outer, &mut inner);
        before.negate();
        after.negate();
        dir = SkPathDirection::kCCW;
    }
    let mut matrix = SkMatrix::default();
    matrix.setScale(radius, radius);
    matrix.postTranslate(pivot.fX, pivot.fY);
    let mut conics = [SkConic::default(); SkConic::kMaxConicsForArc];
    let count = SkConic::BuildUnitArc(&before, &after, dir, Some(&matrix), &mut conics);
    if count > 0 {
        for i in 0..count as usize {
            outer.conicTo(conics[i].fPts[1], conics[i].fPts[2], conics[i].fW);
        }
        after.scale(radius);
        HandleInnerJoin(inner, pivot, after);
    }
}
// Source: src/core/SkStrokerPriv.cpp:139-139
pub(crate) const kOneOverSqrt2: f32 = 0.707106781;

// Source: src/core/SkStrokerPriv.cpp:141-220
// Option<mid> expresses the forward DO_MITER/DO_BLUNT jumps without changing arithmetic order.
pub(crate) fn MiterJoiner<'a>(
    mut outer: &'a mut SkPathBuilder,
    mut inner: &'a mut SkPathBuilder,
    beforeUnitNormal: SkVector,
    pivot: SkPoint,
    afterUnitNormal: SkVector,
    radius: f32,
    invMiterLimit: f32,
    prevIsLine: bool,
    mut currIsLine: bool,
) {
    let dotProd = SkPoint::dot_product(beforeUnitNormal, afterUnitNormal);
    let angleType = Dot2AngleType(dotProd);
    let mut before = beforeUnitNormal;
    let mut after = afterUnitNormal;
    if angleType == AngleType::kNearlyLine_AngleType {
        return;
    }
    let mid = if angleType == AngleType::kNearly180_AngleType {
        currIsLine = false;
        None
    } else {
        let ccw = !is_clockwise(before, after);
        if ccw {
            std::mem::swap(&mut outer, &mut inner);
            before.negate();
            after.negate();
        }
        if dotProd == 0.0 && invMiterLimit <= kOneOverSqrt2 {
            Some((before + after) * radius)
        } else {
            let sinHalfAngle = ((1.0 + dotProd) / 2.0_f32).sqrt();
            if sinHalfAngle < invMiterLimit {
                currIsLine = false;
                None
            } else {
                let mut mid = if angleType == AngleType::kSharp_AngleType {
                    let mut v = SkPoint::make(after.fY - before.fY, before.fX - after.fX);
                    if ccw {
                        v.negate();
                    }
                    v
                } else {
                    before + after
                };
                mid.set_length(radius / sinHalfAngle);
                Some(mid)
            }
        }
    };
    if let Some(mid) = mid {
        if prevIsLine {
            outer.setPoint(outer.points().len().wrapping_sub(1), pivot + mid);
        } else {
            outer.lineTo(pivot + mid);
        }
    }
    after.scale(radius);
    if !currIsLine {
        outer.lineTo(SkPoint::make(pivot.fX + after.fX, pivot.fY + after.fY));
    }
    HandleInnerJoin(inner, pivot, after);
}
// Source: src/core/SkStrokerPriv.cpp:224-231
pub fn CapFactory(cap: Cap) -> CapProc {
    const gCappers: [CapProc; 3] = [ButtCapper, RoundCapper, SquareCapper];
    debug_assert!((cap as u32) < crate::src::core::SkPaint::kCapCount as u32);
    gCappers[cap as usize]
}

// Source: src/core/SkStrokerPriv.cpp:233-240
pub fn JoinFactory(join: crate::src::core::SkPaint::Join) -> JoinProc {
    const gJoiners: [JoinProc; 3] = [MiterJoiner, RoundJoiner, BluntJoiner];
    debug_assert!((join as u32) < crate::src::core::SkPaint::kJoinCount as u32);
    gJoiners[join as usize]
}
