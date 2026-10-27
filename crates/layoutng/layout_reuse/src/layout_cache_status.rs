// cpp: layoutng/internal/layout_utils.h:19-30
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum LayoutCacheStatus {
    kHit,
    kNeedsLayout,
    kNeedsSimplifiedLayout,
    kCanReuseLines,
}
