//! Native path values produced at Paint's property boundary. An SVG clipper
//! unions its independently filled children; concatenating their verbs would
//! change even-odd and opposite-winding overlap semantics.
use layoutng_assembly::internal::layout_input::{
    Offset, PaintClipPathChild, PaintPathCommand, PaintPathVerb,
};
use skia::include::core::SkPath::{SkPath, SkPathFillType, SkPathPoint, SkPathVerb};
use std::sync::Arc;

/// Source: paint_property_tree_builder.cc (BackdropFilterInfo bounds).
/// The actual producer creates the native rounded bounds once; consumers only
/// borrow this immutable SkPath allocation.
pub fn rounded_bounds(
    rect: crate::PaintRect,
    radii: layoutng_assembly::internal::paint_input::PaintCornerRadii,
) -> Arc<SkPath> {
    use skia::include::core::SkRRect::{SkRRect, SkRRectBounds, SkRRectVector};
    let mut value = SkRRect::default();
    value.setRectRadii(
        SkRRectBounds::MakeLTRB(
            rect.x as f32,
            rect.y as f32,
            (rect.x + rect.width) as f32,
            (rect.y + rect.height) as f32,
        ),
        [
            radii.top_left,
            radii.top_right,
            radii.bottom_right,
            radii.bottom_left,
        ]
        .map(|r| SkRRectVector {
            x: r.x as f32,
            y: r.y as f32,
        }),
    );
    Arc::new(SkPath::FromRRect(&value))
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintClipPath {
    paths: Arc<[Arc<SkPath>]>,
}
impl PaintClipPath {
    /// Wraps one actual native path allocation; commands are never projected
    /// into another recording or combined with a different fill-rule child.
    pub fn from_path(path: Arc<SkPath>) -> Self {
        Self {
            paths: Arc::from([path]),
        }
    }
    /// Retains native child identities and all four fill types unchanged.
    pub fn from_native(paths: Arc<[Arc<SkPath>]>) -> Self {
        Self { paths }
    }
    /// Source: third_party/blink/renderer/core/paint/clip_path_clipper.cc
    /// and layout/svg/layout_svg_resource_clipper.cc. This is an input value
    /// conversion only; union rasterization remains with the actual clip owner.
    /// The point delta is applied once by the producer in property space.
    pub fn from_authored(
        commands: &[PaintPathCommand],
        children: &[PaintClipPathChild],
        even_odd: bool,
        delta: Offset,
    ) -> Self {
        let paths = if children.is_empty() {
            vec![Arc::new(compile_path(commands, even_odd, delta))]
        } else {
            children
                .iter()
                .map(|child| Arc::new(compile_path(&child.commands, child.even_odd, delta)))
                .collect()
        };
        Self {
            paths: paths.into(),
        }
    }
    pub fn paths(&self) -> &[Arc<SkPath>] {
        &self.paths
    }
    pub fn is_finite(&self) -> bool {
        self.paths.iter().all(|path| path.isFinite())
    }
    pub fn is_empty(&self) -> bool {
        self.paths.iter().all(|path| path.isEmpty())
    }
    pub fn single_path(&self) -> Option<&Arc<SkPath>> {
        if self.paths.len() == 1 {
            self.paths.first()
        } else {
            None
        }
    }
}
fn compile_path(commands: &[PaintPathCommand], even_odd: bool, delta: Offset) -> SkPath {
    let mut points = Vec::new();
    let mut verbs = Vec::with_capacity(commands.len());
    let mut conics = Vec::new();
    let point = |p: Offset| SkPathPoint::new((p.x + delta.x) as f32, (p.y + delta.y) as f32);
    for c in commands {
        use PaintPathVerb::*;
        verbs.push(match c.verb {
            kMoveTo => {
                points.push(point(c.point));
                SkPathVerb::kMove
            }
            kLineTo => {
                points.push(point(c.point));
                SkPathVerb::kLine
            }
            kQuadraticTo => {
                points.extend([point(c.control1), point(c.point)]);
                SkPathVerb::kQuad
            }
            kConicTo => {
                points.extend([point(c.control1), point(c.point)]);
                conics.push(c.conic_weight as f32);
                SkPathVerb::kConic
            }
            kCubicTo => {
                points.extend([point(c.control1), point(c.control2), point(c.point)]);
                SkPathVerb::kCubic
            }
            kClose => SkPathVerb::kClose,
        });
    }
    SkPath::Raw(
        &points,
        &verbs,
        &conics,
        if even_odd {
            SkPathFillType::kEvenOdd
        } else {
            SkPathFillType::kWinding
        },
        false,
    )
}
/// A drawing-command projection at the legacy raster ABI. The property owner
/// and compositor retain the native Arc path; no projected command list is
/// cached or stored back on a property node.
pub fn replay_path(path: &SkPath) -> Result<Vec<PaintPathCommand>, NativePathReplayError> {
    if !path.isFinite() {
        return Err(NativePathReplayError::NonFinite);
    }
    if path.isInverseFillType() {
        return Err(NativePathReplayError::InverseFill);
    }
    let mut point_index = 0;
    let mut conic_index = 0;
    Ok(path
        .verbs()
        .iter()
        .map(|verb| {
            let mut c = PaintPathCommand::default();
            let mut next = || {
                let p = path.points()[point_index];
                point_index += 1;
                Offset {
                    x: f64::from(p.x),
                    y: f64::from(p.y),
                }
            };
            use SkPathVerb::*;
            c.verb = match verb {
                kMove => {
                    c.point = next();
                    PaintPathVerb::kMoveTo
                }
                kLine => {
                    c.point = next();
                    PaintPathVerb::kLineTo
                }
                kQuad => {
                    c.control1 = next();
                    c.point = next();
                    PaintPathVerb::kQuadraticTo
                }
                kConic => {
                    c.control1 = next();
                    c.point = next();
                    c.conic_weight = f64::from(path.conicWeights()[conic_index]);
                    conic_index += 1;
                    PaintPathVerb::kConicTo
                }
                kCubic => {
                    c.control1 = next();
                    c.control2 = next();
                    c.point = next();
                    PaintPathVerb::kCubicTo
                }
                kClose => PaintPathVerb::kClose,
            };
            c
        })
        .collect())
}
pub fn is_even_odd(path: &SkPath) -> bool {
    matches!(
        path.getFillType(),
        SkPathFillType::kEvenOdd | SkPathFillType::kInverseEvenOdd
    )
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePathReplayError {
    NonFinite,
    InverseFill,
}
impl std::fmt::Display for NativePathReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native path cannot use legacy replay: {self:?}")
    }
}
impl std::error::Error for NativePathReplayError {}
impl From<NativePathReplayError> for std::io::Error {
    fn from(error: NativePathReplayError) -> Self {
        Self::new(std::io::ErrorKind::Unsupported, error)
    }
}
