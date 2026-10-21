use crate::src::core::SkPath::SkPath;
use crate::src::core::SkPoint::SkPoint;

// Source: src/core/SkPath_interpolate.cpp:16-24
fn span_equal<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    if a.is_empty() {
        return true;
    }
    a.as_ptr() == b.as_ptr() || a == b
}

// Source: src/core/SkPath_interpolate.cpp:26-31
fn scalar_times_point(s: f32, p: SkPoint) -> SkPoint {
    SkPoint::make(p.fX * s, p.fY * s)
}

// Source: src/core/SkPath_interpolate.cpp:33-35
fn lerp(from: SkPoint, to: SkPoint, t: f32) -> SkPoint {
    let diff = SkPoint::make(to.fX - from.fX, to.fY - from.fY);
    let delta = scalar_times_point(t, diff);
    SkPoint::make(from.fX + delta.fX, from.fY + delta.fY)
}

// Source: src/core/SkPath_interpolate.cpp:52-56
fn CanInterpolate(from: &SkPath, to: &SkPath) -> bool {
    from.points().len() == to.points().len()
        && span_equal(from.verbs(), to.verbs())
        && span_equal(from.conicWeights(), to.conicWeights())
}

// Source: src/core/SkPath_interpolate.cpp:68-85
fn Interpolate(from: &SkPath, to: &SkPath, t: f32) -> Option<SkPath> {
    if !CanInterpolate(from, to) {
        return None;
    }
    let fillType = from.getFillType();
    let fromPts = from.points();
    let toPts = to.points();
    let mut dst = vec![SkPoint::make(0.0, 0.0); fromPts.len()];
    for i in 0..fromPts.len() {
        dst[i] = lerp(fromPts[i], toPts[i], t);
    }
    Some(SkPath::Raw(
        &dst,
        from.verbs(),
        from.conicWeights(),
        fillType,
        false,
    ))
}

// Source: src/core/SkPath_interpolate.cpp:91-93
impl SkPath {
    pub fn isInterpolatable(&self, compare: &SkPath) -> bool {
        CanInterpolate(self, compare)
    }
}

// Source: src/core/SkPath_interpolate.cpp:95-97
impl SkPath {
    pub fn makeInterpolate(&self, ending: &SkPath, weight: f32) -> SkPath {
        Interpolate(self, ending, 1.0 - weight).unwrap_or_default()
    }
}

// Source: src/core/SkPath_interpolate.cpp:99-105
impl SkPath {
    pub fn interpolate(&self, ending: &SkPath, weight: f32, out: &mut SkPath) -> bool {
        if let Some(result) = Interpolate(self, ending, 1.0 - weight) {
            *out = result;
            return true;
        }
        false
    }
}
