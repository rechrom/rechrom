//! Backend-neutral immutable path values produced at Paint's property boundary.
//!
//! An SVG clipper unions its independently filled children; concatenating their
//! verbs would change even-odd and opposite-winding overlap semantics. Paint
//! therefore retains one value per child, while compositor selects and builds
//! the concrete legacy or translated Skia path exactly once during lowering.
pub use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
use layoutng_assembly::internal::{
    layout_input::PaintClipPathChild, paint_input::PaintCornerRadii,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintPathFillType {
    #[default]
    Winding,
    EvenOdd,
    InverseWinding,
    InverseEvenOdd,
}
impl PaintPathFillType {
    pub fn is_even_odd(self) -> bool {
        matches!(self, Self::EvenOdd | Self::InverseEvenOdd)
    }
    pub fn is_inverse(self) -> bool {
        matches!(self, Self::InverseWinding | Self::InverseEvenOdd)
    }
}

/// The two source-authored path forms that must survive until backend
/// selection. Rounded rectangles remain structured so each Skia implementation
/// performs its own canonical radius normalization and path construction.
#[derive(Clone, Debug, PartialEq)]
pub enum PaintPathGeometry {
    Commands(Arc<[PaintPathCommand]>),
    RoundedRect {
        rect: crate::PaintRect,
        radii: PaintCornerRadii,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintPath {
    geometry: PaintPathGeometry,
    fill_type: PaintPathFillType,
}
impl PaintPath {
    pub fn from_commands(
        commands: impl Into<Arc<[PaintPathCommand]>>,
        fill_type: PaintPathFillType,
    ) -> Self {
        Self {
            geometry: PaintPathGeometry::Commands(commands.into()),
            fill_type,
        }
    }
    pub fn rounded_rect(rect: crate::PaintRect, radii: PaintCornerRadii) -> Self {
        Self {
            geometry: PaintPathGeometry::RoundedRect { rect, radii },
            fill_type: PaintPathFillType::Winding,
        }
    }
    pub fn geometry(&self) -> &PaintPathGeometry {
        &self.geometry
    }
    pub fn fill_type(&self) -> PaintPathFillType {
        self.fill_type
    }
    pub fn is_even_odd(&self) -> bool {
        self.fill_type.is_even_odd()
    }
    pub fn is_inverse(&self) -> bool {
        self.fill_type.is_inverse()
    }
    pub fn verb_count(&self) -> usize {
        match &self.geometry {
            PaintPathGeometry::Commands(commands) => commands.len(),
            PaintPathGeometry::RoundedRect { rect, radii } => {
                crate::border_shape_utils::ConicRoundedRectPath(rect, radii).len()
            }
        }
    }
    pub fn is_finite(&self) -> bool {
        match &self.geometry {
            PaintPathGeometry::Commands(commands) => commands.iter().all(command_is_finite),
            // SkRRect::setRectRadii normalizes non-finite radii to a plain
            // rectangle. Only the source rectangle can make this path
            // non-finite.
            PaintPathGeometry::RoundedRect { rect, .. } => {
                [rect.x, rect.y, rect.width, rect.height]
                    .into_iter()
                    .all(f64::is_finite)
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        match &self.geometry {
            PaintPathGeometry::Commands(commands) => !commands.iter().any(|command| {
                matches!(
                    command.verb,
                    PaintPathVerb::kLineTo
                        | PaintPathVerb::kQuadraticTo
                        | PaintPathVerb::kConicTo
                        | PaintPathVerb::kCubicTo
                )
            }),
            PaintPathGeometry::RoundedRect { rect, .. } => {
                !self.is_finite() || rect.width == 0.0 || rect.height == 0.0
            }
        }
    }
    /// SkPath::getBounds semantics: control-point bounds rather than the tight
    /// extrema bounds of curves. Empty/non-finite values return an empty rect.
    pub fn bounds(&self) -> crate::PaintRect {
        if !self.is_finite() || self.is_empty() {
            return crate::PaintRect::default();
        }
        match &self.geometry {
            PaintPathGeometry::RoundedRect { rect, .. } => sorted_rect(*rect),
            PaintPathGeometry::Commands(commands) => command_bounds(commands),
        }
    }
    /// Conservative source-compatible rectangle recognition used only by cc's
    /// culling fast path. Curves and multiple contours deliberately return None.
    pub fn is_rect(&self) -> Option<crate::PaintRect> {
        if self.is_inverse() || !self.is_finite() || self.is_empty() {
            return None;
        }
        match &self.geometry {
            PaintPathGeometry::RoundedRect { rect, radii } => {
                let radii = [
                    radii.top_left,
                    radii.top_right,
                    radii.bottom_right,
                    radii.bottom_left,
                ];
                // Source SkRRect converts any non-finite radius set to a
                // rectangle, and clamps a corner to square when either axis
                // is non-positive.
                let all_square = radii
                    .iter()
                    .any(|radius| !radius.x.is_finite() || !radius.y.is_finite())
                    || radii
                        .into_iter()
                        .all(|radius| radius.x <= 0.0 || radius.y <= 0.0);
                all_square.then(|| sorted_rect(*rect))
            }
            PaintPathGeometry::Commands(commands) => command_rect(commands),
        }
    }
}

/// Source: paint_property_tree_builder.cc (BackdropFilterInfo bounds).
pub fn rounded_bounds(rect: crate::PaintRect, radii: PaintCornerRadii) -> Arc<PaintPath> {
    Arc::new(PaintPath::rounded_rect(rect, radii))
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaintClipPath {
    paths: Arc<[Arc<PaintPath>]>,
}
impl PaintClipPath {
    pub fn from_path(path: Arc<PaintPath>) -> Self {
        Self {
            paths: Arc::from([path]),
        }
    }
    pub fn from_paths(paths: Arc<[Arc<PaintPath>]>) -> Self {
        Self { paths }
    }
    /// Source: third_party/blink/renderer/core/paint/clip_path_clipper.cc
    /// and layout/svg/layout_svg_resource_clipper.cc. The point delta is
    /// applied once by the producer in property space.
    pub fn from_authored(
        commands: &[PaintPathCommand],
        children: &[PaintClipPathChild],
        even_odd: bool,
        delta: Offset,
    ) -> Self {
        let paths = if children.is_empty() {
            vec![Arc::new(authored_path(commands, even_odd, delta))]
        } else {
            children
                .iter()
                .map(|child| Arc::new(authored_path(&child.commands, child.even_odd, delta)))
                .collect()
        };
        Self {
            paths: paths.into(),
        }
    }
    pub fn paths(&self) -> &[Arc<PaintPath>] {
        &self.paths
    }
    pub fn is_finite(&self) -> bool {
        self.paths.iter().all(|path| path.is_finite())
    }
    pub fn is_empty(&self) -> bool {
        self.paths.iter().all(|path| path.is_empty())
    }
    pub fn single_path(&self) -> Option<&Arc<PaintPath>> {
        (self.paths.len() == 1).then(|| &self.paths[0])
    }
}

fn authored_path(commands: &[PaintPathCommand], even_odd: bool, delta: Offset) -> PaintPath {
    let mut translated = Vec::with_capacity(commands.len());
    for command in commands {
        let mut command = *command;
        let add = |point: &mut Offset| {
            point.x += delta.x;
            point.y += delta.y;
        };
        match command.verb {
            PaintPathVerb::kMoveTo | PaintPathVerb::kLineTo => add(&mut command.point),
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
                add(&mut command.control1);
                add(&mut command.point);
            }
            PaintPathVerb::kCubicTo => {
                add(&mut command.control1);
                add(&mut command.control2);
                add(&mut command.point);
            }
            PaintPathVerb::kClose => {}
        }
        translated.push(command);
    }
    PaintPath::from_commands(
        translated,
        if even_odd {
            PaintPathFillType::EvenOdd
        } else {
            PaintPathFillType::Winding
        },
    )
}

fn command_is_finite(command: &PaintPathCommand) -> bool {
    let point = |point: Offset| point.x.is_finite() && point.y.is_finite();
    match command.verb {
        PaintPathVerb::kMoveTo | PaintPathVerb::kLineTo => point(command.point),
        PaintPathVerb::kQuadraticTo => point(command.control1) && point(command.point),
        PaintPathVerb::kConicTo => {
            point(command.control1) && point(command.point) && command.conic_weight.is_finite()
        }
        PaintPathVerb::kCubicTo => {
            point(command.control1) && point(command.control2) && point(command.point)
        }
        PaintPathVerb::kClose => true,
    }
}

fn sorted_rect(rect: crate::PaintRect) -> crate::PaintRect {
    let left = rect.x.min(rect.x + rect.width);
    let right = rect.x.max(rect.x + rect.width);
    let top = rect.y.min(rect.y + rect.height);
    let bottom = rect.y.max(rect.y + rect.height);
    crate::PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

fn command_points(command: &PaintPathCommand) -> impl Iterator<Item = Offset> {
    let points = match command.verb {
        PaintPathVerb::kMoveTo | PaintPathVerb::kLineTo => [Some(command.point), None, None],
        PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
            [Some(command.control1), Some(command.point), None]
        }
        PaintPathVerb::kCubicTo => [
            Some(command.control1),
            Some(command.control2),
            Some(command.point),
        ],
        PaintPathVerb::kClose => [None, None, None],
    };
    points.into_iter().flatten()
}

fn command_bounds(commands: &[PaintPathCommand]) -> crate::PaintRect {
    let mut points = commands.iter().flat_map(command_points);
    let Some(first) = points.next() else {
        return crate::PaintRect::default();
    };
    let (mut left, mut top, mut right, mut bottom) = (first.x, first.y, first.x, first.y);
    for point in points {
        left = left.min(point.x);
        top = top.min(point.y);
        right = right.max(point.x);
        bottom = bottom.max(point.y);
    }
    crate::PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

fn command_rect(commands: &[PaintPathCommand]) -> Option<crate::PaintRect> {
    let mut points = Vec::with_capacity(5);
    let mut closed = false;
    for command in commands {
        match command.verb {
            PaintPathVerb::kMoveTo if points.is_empty() => points.push(command.point),
            PaintPathVerb::kLineTo if !closed => {
                if points.last().copied() != Some(command.point) {
                    points.push(command.point);
                }
            }
            PaintPathVerb::kClose if !points.is_empty() && !closed => closed = true,
            _ => return None,
        }
    }
    if points.last() == points.first() {
        points.pop();
    }
    if points.len() != 4 {
        return None;
    }
    let bounds = command_bounds(commands);
    let corners = [
        Offset {
            x: bounds.x,
            y: bounds.y,
        },
        Offset {
            x: bounds.x + bounds.width,
            y: bounds.y,
        },
        Offset {
            x: bounds.x + bounds.width,
            y: bounds.y + bounds.height,
        },
        Offset {
            x: bounds.x,
            y: bounds.y + bounds.height,
        },
    ];
    let every_corner = corners.iter().all(|corner| points.contains(corner));
    let axis_aligned = (0..4).all(|index| {
        let a = points[index];
        let b = points[(index + 1) % 4];
        a.x == b.x || a.y == b.y
    });
    (closed && every_corner && axis_aligned).then_some(bounds)
}
