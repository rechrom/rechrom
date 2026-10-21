use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPoint::SkVector;

// Source: src/utils/SkMatrix22.cpp:14-41
pub fn SkComputeGivensRotation(h: &SkVector, g: &mut SkMatrix) {
    let a = h.fX;
    let b = h.fY;
    let (c, s);
    if b == 0.0 {
        c = 1.0f32.copysign(a);
        s = 0.0;
    } else if a == 0.0 {
        c = 0.0;
        s = -1.0f32.copysign(b);
    } else if b.abs() > a.abs() {
        let t = a / b;
        let u = (1.0 + t * t).sqrt().copysign(b);
        s = -1.0 / u;
        c = -s * t;
    } else {
        let t = b / a;
        let u = (1.0 + t * t).sqrt().copysign(a);
        c = 1.0 / u;
        s = -c * t;
    }
    g.setSinCos(s, c);
}
