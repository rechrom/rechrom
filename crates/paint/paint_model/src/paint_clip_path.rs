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
                crate::contoured_border_geometry::ConicRoundedRectPath(rect, radii).len()
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
    /// Point containment using SkPath's fill, implicit closure, and boundary
    /// semantics. Curves are intersected directly, without flattening.
    /// Source: src/core/SkPathPriv.cpp, SkPathPriv::Contains.
    pub fn contains(&self, point: Offset) -> bool {
        self.contains_with_fill_type(point, self.fill_type)
    }

    pub fn contains_with_fill_type(&self, point: Offset, fill: PaintPathFillType) -> bool {
        match &self.geometry {
            PaintPathGeometry::Commands(commands) => commands_contain(commands, fill, point),
            PaintPathGeometry::RoundedRect { rect, radii } => {
                if !self.is_finite() {
                    return false;
                }
                let rect = sorted_rect(*rect);
                if ![rect.x, rect.y, rect.width, rect.height]
                    .into_iter()
                    .all(f64::is_finite)
                {
                    return false;
                }
                if rect.is_empty() {
                    return fill.is_inverse();
                }
                let radii = normalized_path_radii(rect, *radii);
                let commands =
                    crate::contoured_border_geometry::ConicRoundedRectPath(&rect, &radii);
                commands_contain(&commands, fill, point)
            }
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

// Source: SkRRect::setRectRadii / scaleRadii. Invalid radii make a rectangle;
// a nonpositive component makes its entire corner square. The shared
// FloatRoundedRect constraint uses the same minimum side-pair scale.
fn normalized_path_radii(rect: crate::PaintRect, mut radii: PaintCornerRadii) -> PaintCornerRadii {
    if [
        radii.top_left,
        radii.top_right,
        radii.bottom_right,
        radii.bottom_left,
    ]
    .iter()
    .any(|r| !r.x.is_finite() || !r.y.is_finite())
    {
        return PaintCornerRadii::default();
    }
    for r in [
        &mut radii.top_left,
        &mut radii.top_right,
        &mut radii.bottom_right,
        &mut radii.bottom_left,
    ] {
        if r.x <= 0.0 || r.y <= 0.0 {
            *r = Default::default();
        }
    }
    crate::contoured_border_geometry::Normalize(
        &mut radii,
        layoutng_assembly::internal::layout_input::Size {
            width: rect.width,
            height: rect.height,
        },
    );
    // SkRRect clamps again after scaling, which can underflow one component.
    for r in [
        &mut radii.top_left,
        &mut radii.top_right,
        &mut radii.bottom_right,
        &mut radii.bottom_left,
    ] {
        if r.x <= 0.0 || r.y <= 0.0 {
            *r = Default::default();
        }
    }
    radii
}

// One edge of the existing command owner, consumed immediately by the walk.
// These are Bernstein controls, not a retained or flattened second path.
#[derive(Clone, Copy)]
struct ContainmentEdge {
    points: [Offset; 4],
    degree: usize,
    weight: f64,
}
impl ContainmentEdge {
    fn line(a: Offset, b: Offset) -> Self {
        Self {
            points: [a, b, b, b],
            degree: 1,
            weight: 1.0,
        }
    }
    fn eval(self, t: f64) -> Offset {
        let u = 1.0 - t;
        let weights = match self.degree {
            1 => [u, t, 0.0, 0.0],
            2 => [u * u, 2.0 * self.weight * u * t, t * t, 0.0],
            _ => [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t],
        };
        let denominator: f64 = weights.iter().sum();
        let mut result = Offset::default();
        for (p, w) in self.points.iter().zip(weights) {
            result.x += p.x * w;
            result.y += p.y * w;
        }
        result.x /= denominator;
        result.y /= denominator;
        result
    }
    fn tangent(self, t: f64) -> Offset {
        let p = self.points;
        let u = 1.0 - t;
        let (a, b, c) = match self.degree {
            1 => (1.0, 0.0, 0.0),
            2 if self.weight == 1.0 => (2.0 * u, 2.0 * t, 0.0),
            2 => {
                // Positive denominator omitted; only tangent direction matters.
                let w = self.weight;
                return Offset {
                    x: u * u * w * (p[1].x - p[0].x)
                        + u * t * (p[2].x - p[0].x)
                        + t * t * w * (p[2].x - p[1].x),
                    y: u * u * w * (p[1].y - p[0].y)
                        + u * t * (p[2].y - p[0].y)
                        + t * t * w * (p[2].y - p[1].y),
                };
            }
            _ => (3.0 * u * u, 6.0 * u * t, 3.0 * t * t),
        };
        Offset {
            x: a * (p[1].x - p[0].x) + b * (p[2].x - p[1].x) + c * (p[3].x - p[2].x),
            y: a * (p[1].y - p[0].y) + b * (p[2].y - p[1].y) + c * (p[3].y - p[2].y),
        }
    }
    fn y_extrema(self) -> ([f64; 4], usize) {
        let p = self.points;
        let mut cuts = [0.0, 1.0, 1.0, 1.0];
        let mut count = 2;
        let coefficients = match self.degree {
            2 => {
                let a = self.weight * (p[1].y - p[0].y);
                let b = p[2].y - p[0].y;
                let c = self.weight * (p[2].y - p[1].y);
                Some((a - b + c, b - 2.0 * a, a))
            }
            3 => {
                let a = p[1].y - p[0].y;
                let b = p[2].y - p[1].y;
                let c = p[3].y - p[2].y;
                Some((a - 2.0 * b + c, 2.0 * (b - a), a))
            }
            _ => None,
        };
        if let Some((a, b, c)) = coefficients {
            for t in quadratic_roots(a, b, c).into_iter().flatten() {
                if t > 0.0 && t < 1.0 {
                    cuts[count] = t;
                    count += 1;
                }
            }
        }
        cuts[..count].sort_by(f64::total_cmp);
        (cuts, count)
    }
    fn at_y(self, y: f64, start: f64, end: f64) -> f64 {
        let first = self.eval(start).y;
        let last = self.eval(end).y;
        if y == first {
            return start;
        }
        if y == last {
            return end;
        }
        if self.degree == 2 {
            // SkPathPriv's quadratic/conic equation after moving the query
            // ordinate into the rational numerator (denominator is positive).
            let p = self.points;
            let w = self.weight;
            let a = p[0].y - 2.0 * w * p[1].y + p[2].y - 2.0 * (1.0 - w) * y;
            let b = 2.0 * (w * p[1].y - p[0].y) - 2.0 * (w - 1.0) * y;
            if let Some(t) = quadratic_roots(a, b, p[0].y - y)
                .into_iter()
                .flatten()
                .find(|t| *t >= start && *t <= end)
            {
                return t;
            }
        }
        // SkCubicClipper::ChopMonoAtY also isolates a monotone root. Keeping
        // the original parameter interval avoids constructing chopped paths.
        let (mut lo, mut hi) = (start, end);
        for _ in 0..64 {
            let mid = (lo + hi) * 0.5;
            if mid == lo || mid == hi {
                break;
            }
            if (self.eval(mid).y < y) == (first < last) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (lo + hi) * 0.5
    }
    fn winding(self, point: Offset, on: &mut i32) -> i32 {
        if self.degree == 1 {
            return line_winding(self.points[0], self.points[1], point, on);
        }
        let (cuts, count) = self.y_extrema();
        let mut winding = 0;
        for pair in cuts[..count].windows(2) {
            let a = self.eval(pair[0]);
            let b = self.eval(pair[1]);
            if !between(a.y, point.y, b.y) {
                continue;
            }
            // As in checkOnCurve, count an edge's start but exclude its end.
            if (a.y == b.y && between(a.x, point.x, b.x) && point.x != b.x)
                || (point == a && a.y != b.y)
            {
                *on += 1;
                continue;
            }
            if point.y == a.y.max(b.y) {
                continue;
            }
            let x = self.eval(self.at_y(point.y, pair[0], pair[1])).x;
            if nearly_equal(x, point.x) && point != b {
                *on += 1;
            } else if x < point.x {
                winding += if a.y < b.y { 1 } else { -1 };
            }
        }
        winding
    }
    fn collect_tangents(self, point: Offset, tangents: &mut Vec<Offset>) {
        if self.degree == 1 {
            let a = self.points[0];
            let b = self.points[1];
            if between(a.x, point.x, b.x)
                && between(a.y, point.y, b.y)
                && nearly_equal((point.x - a.x) * (b.y - a.y), (b.x - a.x) * (point.y - a.y))
            {
                add_tangent(tangents, self.tangent(0.0));
            }
            return;
        }
        let (cuts, count) = self.y_extrema();
        let mut previous = None;
        for pair in cuts[..count].windows(2) {
            let a = self.eval(pair[0]);
            let b = self.eval(pair[1]);
            if a.y == b.y || !between(a.y, point.y, b.y) {
                continue;
            }
            let t = self.at_y(point.y, pair[0], pair[1]);
            if previous == Some(t) {
                continue;
            }
            previous = Some(t);
            if nearly_equal(self.eval(t).x, point.x) {
                add_tangent(tangents, self.tangent(t));
            }
        }
    }
}
fn quadratic_roots(a: f64, b: f64, c: f64) -> [Option<f64>; 2] {
    if a == 0.0 {
        return [if b == 0.0 { None } else { Some(-c / b) }, None];
    }
    let d = b * b - 4.0 * a * c;
    if d < 0.0 {
        return [None, None];
    }
    let q = -0.5 * (b + d.sqrt().copysign(b));
    if q == 0.0 {
        return [Some(0.0), None];
    }
    let first = q / a;
    let second = c / q;
    [Some(first), (first != second).then_some(second)]
}

// Source: SkPathBuilder::ensureMove/conicTo and SkPathEdgeIter. Open contours
// close implicitly; a drawing command after close starts at the move point.
fn for_each_edge(commands: &[PaintPathCommand], mut visit: impl FnMut(ContainmentEdge)) {
    let mut start = Offset::default();
    let mut current = start;
    let mut open = false;
    for command in commands {
        match command.verb {
            PaintPathVerb::kMoveTo => {
                if open && current != start {
                    visit(ContainmentEdge::line(current, start));
                }
                start = command.point;
                current = start;
                open = false;
            }
            PaintPathVerb::kClose => {
                if open && current != start {
                    visit(ContainmentEdge::line(current, start));
                }
                current = start;
                open = false;
            }
            _ => {
                let mut edge = ContainmentEdge::line(current, command.point);
                match command.verb {
                    PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
                        let w = if command.verb == PaintPathVerb::kConicTo {
                            command.conic_weight
                        } else {
                            1.0
                        };
                        if w > 0.0 && w.is_finite() {
                            edge.degree = 2;
                            edge.points[1] = command.control1;
                            edge.points[2] = command.point;
                            edge.weight = w;
                        } else if !w.is_finite() && !(w <= 0.0) {
                            // Infinite/NaN weights become two lines; nonpositive
                            // weights discard the control and become one line.
                            visit(ContainmentEdge::line(current, command.control1));
                            edge.points[0] = command.control1;
                        }
                    }
                    PaintPathVerb::kCubicTo => {
                        edge.degree = 3;
                        edge.points[1] = command.control1;
                        edge.points[2] = command.control2;
                        edge.points[3] = command.point;
                    }
                    _ => {}
                }
                visit(edge);
                current = command.point;
                open = true;
            }
        }
    }
    if open && current != start {
        visit(ContainmentEdge::line(current, start));
    }
}
fn between(a: f64, b: f64, c: f64) -> bool {
    a.min(c) <= b && b <= a.max(c)
}
// Source: SkScalarNearlyEqual's default path-containment tolerance.
fn nearly_equal(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0 / 4096.0
}
// Source: SkPathPriv.cpp, winding_line.
fn line_winding(a: Offset, b: Offset, p: Offset, on: &mut i32) -> i32 {
    if !between(a.y, p.y, b.y) {
        return 0;
    }
    if (a.y == b.y && between(a.x, p.x, b.x) && p.x != b.x) || (p == a && a.y != b.y) {
        *on += 1;
        return 0;
    }
    if p.y == a.y.max(b.y) {
        return 0;
    }
    let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
    if cross == 0.0 {
        if p != b {
            *on += 1;
        }
        return 0;
    }
    let direction = if a.y < b.y { 1 } else { -1 };
    if (cross > 0.0) == (direction > 0) {
        0
    } else {
        direction
    }
}
fn add_tangent(tangents: &mut Vec<Offset>, tangent: Offset) {
    if nearly_equal(tangent.x * tangent.x + tangent.y * tangent.y, 0.0) {
        return;
    }
    if let Some(index) = tangents.iter().position(|other| {
        nearly_equal(other.x * tangent.y - other.y * tangent.x, 0.0)
            && other.x * tangent.x <= 0.0
            && other.y * tangent.y <= 0.0
    }) {
        tangents.swap_remove(index);
    } else {
        tangents.push(tangent);
    }
}
fn commands_contain(commands: &[PaintPathCommand], fill: PaintPathFillType, point: Offset) -> bool {
    // SkPath rejects nonfinite path data before applying inverse fill. Inspect
    // normalized builder edges so discarded conic controls do not poison it.
    let mut finite = true;
    let (mut left, mut top, mut right, mut bottom) = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    let mut include = |p: Offset| {
        finite &= p.x.is_finite() && p.y.is_finite();
        left = left.min(p.x);
        right = right.max(p.x);
        top = top.min(p.y);
        bottom = bottom.max(p.y);
    };
    for (index, command) in commands.iter().enumerate() {
        // Consecutive moveTo calls replace the pending move in SkPathBuilder.
        if command.verb == PaintPathVerb::kMoveTo
            && commands
                .get(index + 1)
                .is_none_or(|next| next.verb != PaintPathVerb::kMoveTo)
        {
            include(command.point);
        }
    }
    for_each_edge(commands, |edge| {
        for p in &edge.points[..=edge.degree] {
            include(*p);
        }
    });
    if !finite {
        return false;
    }
    let inverse = fill.is_inverse();
    // Source bounds rejection is inclusive and precedes the curve tolerance.
    if !(left <= point.x && point.x <= right && top <= point.y && point.y <= bottom) {
        return inverse;
    }
    let (mut winding, mut on) = (0, 0);
    for_each_edge(commands, |edge| winding += edge.winding(point, &mut on));
    if fill.is_even_odd() {
        winding &= 1;
    }
    if winding != 0 {
        return !inverse;
    }
    if on <= 1 {
        return (on != 0) ^ inverse;
    }
    if on & 1 != 0 || fill.is_even_odd() {
        return (on & 1 != 0) ^ inverse;
    }
    // Coincident opposite edges have no filled boundary in winding mode.
    let mut tangents = Vec::new();
    for_each_edge(commands, |edge| edge.collect_tangents(point, &mut tangents));
    !tangents.is_empty() ^ inverse
}

#[cfg(test)]
mod containment_tests {
    use super::*;
    fn point(x: f64, y: f64) -> Offset {
        Offset { x, y }
    }
    fn command(verb: PaintPathVerb, x: f64, y: f64) -> PaintPathCommand {
        PaintPathCommand {
            verb,
            point: point(x, y),
            ..Default::default()
        }
    }
    fn rectangle(reverse: bool) -> Vec<PaintPathCommand> {
        let mut points = [
            point(0.0, 0.0),
            point(100.0, 0.0),
            point(100.0, 100.0),
            point(0.0, 100.0),
        ];
        if reverse {
            points.reverse();
        }
        points
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                command(
                    if i == 0 {
                        PaintPathVerb::kMoveTo
                    } else {
                        PaintPathVerb::kLineTo
                    },
                    p.x,
                    p.y,
                )
            })
            .collect()
    }
    #[test]
    fn implicit_closure_fill_rules_and_coincident_boundaries() {
        let mut commands = rectangle(false);
        let path = PaintPath::from_commands(commands.clone(), PaintPathFillType::Winding);
        assert!(path.contains(point(50.0, 50.0)));
        assert!(path.contains(point(100.0, 50.0)));
        assert!(!path.contains(point(120.0, 50.0)));
        commands.extend(rectangle(false));
        let doubled = PaintPath::from_commands(commands.clone(), PaintPathFillType::Winding);
        assert!(doubled.contains(point(50.0, 50.0)));
        assert!(doubled.contains(point(100.0, 50.0)));
        assert!(!doubled.contains_with_fill_type(point(50.0, 50.0), PaintPathFillType::EvenOdd));
        assert!(!doubled.contains_with_fill_type(point(100.0, 50.0), PaintPathFillType::EvenOdd));
        let mut canceled = rectangle(false);
        canceled.extend(rectangle(true));
        let canceled = PaintPath::from_commands(canceled, PaintPathFillType::Winding);
        assert!(!canceled.contains(point(50.0, 50.0)));
        assert!(!canceled.contains(point(100.0, 50.0)));
        assert!(
            canceled.contains_with_fill_type(point(50.0, 50.0), PaintPathFillType::InverseWinding)
        );
    }
    #[test]
    fn curved_fill_uses_quadratic_conic_and_cubic_intersections() {
        for verb in [
            PaintPathVerb::kQuadraticTo,
            PaintPathVerb::kConicTo,
            PaintPathVerb::kCubicTo,
        ] {
            let curve = PaintPathCommand {
                verb,
                control1: point(0.0, 100.0),
                control2: point(100.0, 100.0),
                point: point(100.0, 0.0),
                conic_weight: std::f64::consts::FRAC_1_SQRT_2,
            };
            let path = PaintPath::from_commands(
                vec![command(PaintPathVerb::kMoveTo, 0.0, 0.0), curve],
                PaintPathFillType::Winding,
            );
            let edge = ContainmentEdge {
                points: if verb == PaintPathVerb::kCubicTo {
                    [point(0.0, 0.0), curve.control1, curve.control2, curve.point]
                } else {
                    [point(0.0, 0.0), curve.control1, curve.point, curve.point]
                },
                degree: if verb == PaintPathVerb::kCubicTo {
                    3
                } else {
                    2
                },
                weight: if verb == PaintPathVerb::kConicTo {
                    curve.conic_weight
                } else {
                    1.0
                },
            };
            let boundary = edge.eval(0.5);
            assert!(path.contains(boundary), "{verb:?} boundary");
            assert!(
                path.contains(point(boundary.x, boundary.y * 0.5)),
                "{verb:?} interior"
            );
            assert!(!path.contains(point(boundary.x, 90.0)), "{verb:?} exterior");
        }
    }
    #[test]
    fn rounded_rect_normalizes_radii_and_keeps_exact_conic_corners() {
        let rect = crate::PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let path = PaintPath::rounded_rect(
            rect,
            crate::contoured_border_geometry::UniformCornerRadii(100.0),
        );
        assert!(!path.contains(point(5.0, 5.0)));
        assert!(path.contains(point(50.0, 0.0)));
        assert!(path.contains(point(50.0, 50.0)));
        assert!(!path.contains(point(100.0001, 50.0)));
        // Oversized equal radii constrain to a circle, not to their raw radii.
        assert!(path.contains(point(15.0, 15.0)));
        let invalid = PaintPath::rounded_rect(
            rect,
            PaintCornerRadii {
                top_left: layoutng_assembly::internal::paint_input::PaintCornerRadius {
                    x: f64::NAN,
                    y: 5.0,
                },
                ..Default::default()
            },
        );
        assert!(invalid.contains(point(0.0, 0.0)));
        assert!(invalid.contains(point(100.0, 100.0)));
    }
    #[test]
    fn conic_builder_degeneracies_and_close_restart_match_fill() {
        let mut commands = vec![
            command(PaintPathVerb::kMoveTo, 0.0, 0.0),
            PaintPathCommand {
                verb: PaintPathVerb::kConicTo,
                control1: point(100.0, 100.0),
                point: point(100.0, 0.0),
                conic_weight: -1.0,
                ..Default::default()
            },
        ];
        let line = PaintPath::from_commands(commands.clone(), PaintPathFillType::Winding);
        assert!(!line.contains(point(50.0, 25.0)));
        commands[1].conic_weight = f64::INFINITY;
        let triangle = PaintPath::from_commands(commands, PaintPathFillType::Winding);
        assert!(triangle.contains(point(75.0, 25.0)));
        let mut commands = rectangle(false);
        commands.push(command(PaintPathVerb::kClose, 0.0, 0.0));
        commands.extend([
            command(PaintPathVerb::kLineTo, -100.0, 0.0),
            command(PaintPathVerb::kLineTo, 0.0, -100.0),
        ]);
        let path = PaintPath::from_commands(commands, PaintPathFillType::Winding);
        assert!(path.contains(point(-25.0, -25.0)));
        assert!(path.contains(point(50.0, 50.0)));
    }
}
