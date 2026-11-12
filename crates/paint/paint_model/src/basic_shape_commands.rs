//! Paint's boundary projection of canonical platform-neutral ShapePath verbs.
use crate::paint_clip_path::{Offset, PaintPathCommand, PaintPathVerb};
pub use layoutng_geometry::shape_path::ShapePathBuilder as BasicShapeCommands;
use layoutng_geometry::shape_path::{PathVerb, ShapePath};
pub fn IntoPaintCommands(path: &ShapePath) -> Vec<PaintPathCommand> {
    let point = |p: layoutng_geometry::shape_path::PathPoint| Offset { x: p.x, y: p.y };
    path.commands
        .iter()
        .map(|c| PaintPathCommand {
            verb: match c.verb {
                PathVerb::kMoveTo => PaintPathVerb::kMoveTo,
                PathVerb::kLineTo => PaintPathVerb::kLineTo,
                PathVerb::kQuadraticTo => PaintPathVerb::kQuadraticTo,
                PathVerb::kConicTo => PaintPathVerb::kConicTo,
                PathVerb::kCubicTo => PaintPathVerb::kCubicTo,
                PathVerb::kClose => PaintPathVerb::kClose,
            },
            point: point(c.point),
            control1: point(c.control1),
            control2: point(c.control2),
            conic_weight: c.conic_weight,
        })
        .collect()
}
