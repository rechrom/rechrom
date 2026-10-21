use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPaint::{SkPaint, Style};
// Source: src/core/SkDrawProcs.h:17-17
pub use crate::src::core::SkDraw::DrawTreatAAStrokeAsHairline;

// Source: src/core/SkDrawProcs.h:25-43
// Uses the canonical native Paint state and original gate order.
pub fn DrawTreatAsHairline(paint: &SkPaint, matrix: &SkMatrix, coverage: &mut f32) -> bool {
    if paint.getStyle() != Style::kStroke_Style {
        return false;
    }
    let strokeWidth = paint.getStrokeWidth();
    if strokeWidth == 0.0 {
        *coverage = 1.0;
        return true;
    }
    if !paint.isAntiAlias() {
        return false;
    }
    DrawTreatAAStrokeAsHairline(strokeWidth, matrix, Some(coverage))
}
