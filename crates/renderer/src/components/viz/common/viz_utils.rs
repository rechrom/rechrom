//! Selected CPU renderer utility; other Viz utilities remain outside this package.
#![allow(non_snake_case)]
use compositor_core::adapter::geometry::RectF;
use viz::common::quads::aggregated_render_pass::NativeSharedQuadState;

// Source: components/viz/common/viz_utils.cc:136-159.
pub(crate) fn QuadRoundedCornersBoundsIntersects(
    shared: &NativeSharedQuadState,
    target: RectF,
) -> bool {
    let info = &shared.mask_filter_info;
    if !info.HasRoundedCorners() {
        return false;
    }
    let rounded = &info.rounded_corner_bounds;
    for (corner, radius) in rounded.corner_radii.iter().enumerate() {
        // Source dependency: ui/gfx/geometry/rrect_f.cc:130-149.
        let x = if corner == 1 || corner == 2 {
            rounded.rect.right() - radius.x()
        } else {
            rounded.rect.x()
        };
        let y = if corner >= 2 {
            rounded.rect.bottom() - radius.y()
        } else {
            rounded.rect.y()
        };
        if RectF::new(x, y, radius.x(), radius.y()).Intersects(target) {
            return true;
        }
    }
    false
}
