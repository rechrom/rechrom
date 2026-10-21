#![cfg(target_vendor = "apple")]
use crate::src::ports::SkScalerContext_mac_ct::CGRect;
// Source: src/utils/mac/SkCGGeometry.h:27-29
pub fn SkCGRectIsEmpty(rect: &CGRect) -> bool {
    rect.size.width <= 0.0 || rect.size.height <= 0.0
}
// Source: src/utils/mac/SkCGGeometry.h:31-33
pub fn SkCGRectGetMinX(rect: &CGRect) -> f64 {
    rect.origin.x
}
// Source: src/utils/mac/SkCGGeometry.h:35-37
pub fn SkCGRectGetMaxX(rect: &CGRect) -> f64 {
    rect.origin.x + rect.size.width
}
// Source: src/utils/mac/SkCGGeometry.h:39-41
pub fn SkCGRectGetMinY(rect: &CGRect) -> f64 {
    rect.origin.y
}
// Source: src/utils/mac/SkCGGeometry.h:43-45
pub fn SkCGRectGetMaxY(rect: &CGRect) -> f64 {
    rect.origin.y + rect.size.height
}
// Source: src/utils/mac/SkCGGeometry.h:47-49
pub fn SkCGRectGetWidth(rect: &CGRect) -> f64 {
    rect.size.width
}
