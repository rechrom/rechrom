//! Backend-independent port to the canonical path geometry owner.
//!
//! Construct once from Paint's immutable path, then borrow that same native
//! path for point containment and quad PathOps. Interaction never lowers,
//! flattens or retains a second path representation.
use crate::paint_clip_path::{PaintPath, PaintPathFillType};
use foundation::gfx::{PointF, QuadF};

pub trait PaintPathHitTest: Send + Sync {
    fn count_verbs(&self) -> i32;
    fn contains(&self, point: PointF, fill: Option<PaintPathFillType>) -> bool;
    fn intersects(&self, quad: &QuadF, fill: Option<PaintPathFillType>) -> bool;
}

/// The native cc::Region/SkRegion used by a single culled-inline query.
pub trait PaintHitTestRegion {
    fn union(&mut self, rect: foundation::gfx::Rect);
    fn contains(&self, rect: &foundation::gfx::Rect) -> bool;
}

pub trait PaintPathHitTestFactory: Send + Sync {
    fn new_region(&self) -> Box<dyn PaintHitTestRegion>;
    fn create(&self, path: &PaintPath) -> Box<dyn PaintPathHitTest>;
    fn create_basic_shape(
        &self,
        shape: &dyn layoutng_style::style::basic_shapes::BasicShape,
        reference: foundation::gfx::RectF,
        zoom: f32,
        path_scale: f32,
    ) -> Result<Box<dyn PaintPathHitTest>, crate::basic_shape_path::InvalidBasicShape>;
    fn create_border_shape(
        &self,
        style: &layoutng_style::style::computed_style::ComputedStyle,
        outer_reference: foundation::gfx::RectF,
    ) -> Result<Box<dyn PaintPathHitTest>, crate::basic_shape_path::InvalidBasicShape>;
    fn create_contoured_rect(
        &self,
        rect: &crate::contoured_rect::ContouredRect,
    ) -> Box<dyn PaintPathHitTest>;
}

/// Borrow the exact native Path retained by SVG Layout. This projection adds
/// no geometry storage and performs no command lowering or path reconstruction.
pub struct BorrowedNativePath<'a> {
    pub path: &'a dyn foundation::native_path::NativePath,
    pub rule: foundation::WindRule,
}
impl PaintPathHitTest for BorrowedNativePath<'_> {
    fn count_verbs(&self) -> i32 {
        self.path.CountVerbs()
    }
    fn contains(&self, point: PointF, fill: Option<PaintPathFillType>) -> bool {
        let rule = fill.map_or(self.rule, |f| {
            if f.is_even_odd() {
                foundation::WindRule::RULE_EVENODD
            } else {
                foundation::WindRule::RULE_NONZERO
            }
        });
        let contains = self.path.Contains(point, rule);
        if fill.is_some_and(PaintPathFillType::is_inverse) {
            !contains
        } else {
            contains
        }
    }
    fn intersects(&self, quad: &QuadF, fill: Option<PaintPathFillType>) -> bool {
        // SVG clip-rule contains only noninverse fill rules.
        assert!(!fill.is_some_and(PaintPathFillType::is_inverse));
        let rule = fill.map_or(self.rule, |f| {
            if f.is_even_odd() {
                foundation::WindRule::RULE_EVENODD
            } else {
                foundation::WindRule::RULE_NONZERO
            }
        });
        self.path.Intersects(quad, rule)
    }
}
