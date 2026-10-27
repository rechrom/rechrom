use super::*;

// cpp: layoutng/internal/layout_box_geometry.cc:772-779
#[inline(never)]
pub(super) fn LayoutBoxTransformsChangeMayRequireLayout(box_: &LayoutBox) -> bool {
    for fragment in box_.PhysicalFragments() {
        if fragment.HasAnchorsToPropagate() {
            return true;
        }
    }
    false
}
