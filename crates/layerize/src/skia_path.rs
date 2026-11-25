//! The Skia path type's projection onto the cc path facet used by property
//! trees. Moved from compositor/src/runtime/skia.rs: coherence requires the
//! impl beside its trait, which now lives in layerize::commit.
use crate::commit::geometry::{RRectF, RectF, Vector2dF};
use skia::src::core::SkPath::SkPath;

// Moved from compositor/src/runtime/skia.rs with MainLayerPath.
impl crate::commit::layer_value_backend::MainLayerPath for SkPath {
    fn FromRRect(value: &RRectF) -> Self {
        use skia::{
            include::core::SkPathTypes::SkPathDirection,
            src::core::{SkPoint::SkPoint, SkRRect::SkRRect, SkRect::SkRect},
        };
        let rect = SkRect::MakeLTRB(
            value.rect.x(),
            value.rect.y(),
            value.rect.right(),
            value.rect.bottom(),
        );
        let radii = value.corner_radii.map(|r| SkPoint {
            fX: r.x(),
            fY: r.y(),
        });
        SkPath::RRectDefault(&SkRRect::MakeRectRadii(&rect, &radii), SkPathDirection::kCW)
    }
}
