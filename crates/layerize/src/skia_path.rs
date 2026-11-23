//! The Skia path type's projection onto the cc path facet used by property
//! trees. Moved from compositor/src/runtime/skia.rs: coherence requires the
//! impl beside its trait, which now lives in layerize::commit.
use crate::commit::geometry::{RRectF, RectF, Vector2dF};
use skia::src::core::SkPath::SkPath;

impl crate::commit::geometry::Path for SkPath {
    fn AsRRect(&self) -> Option<RRectF> {
        {
            let mut rr = skia::src::core::SkRRect::SkRRect::default();
            self.isRRect(Some(&mut rr)).then(|| {
                let b = rr.getBounds();
                RRectF {
                    rect: RectF::new(b.fLeft, b.fTop, b.fRight - b.fLeft, b.fBottom - b.fTop),
                    corner_radii: std::array::from_fn(|i| {
                        let r = rr.all_radii()[i];
                        Vector2dF::new(r.fX, r.fY)
                    }),
                }
            })
        }
    }
    fn bounds(&self) -> RectF {
        let b = self.getBounds();
        return RectF::new(b.fLeft, b.fTop, b.fRight - b.fLeft, b.fBottom - b.fTop);
    }
    fn count_points(&self) -> i32 {
        self.countPoints() as i32
    }
    fn count_verbs(&self) -> i32 {
        self.countVerbs()
    }
    fn is_finite(&self) -> bool {
        self.isFinite()
    }
    fn is_convex(&self) -> bool {
        self.isConvex()
    }
    fn is_volatile(&self) -> bool {
        self.isVolatile()
    }
    fn is_inverse_fill_type(&self) -> bool {
        self.isInverseFillType()
    }
}
