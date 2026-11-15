//! Shared immutable platform Path capability. Layout retains the canonical
//! owner; Paint and Interaction borrow it without rebuilding its geometry.
use crate::{
    gfx::{PointF, QuadF, RectF},
    LineCap, LineJoin, WindRule,
};

#[derive(Clone, Copy, PartialEq)]
pub struct StrokeData {
    pub width: f32,
    pub miter_limit: f32,
    pub cap: LineCap,
    pub join: LineJoin,
}
#[derive(Clone, Copy)]
pub enum NativePathElement {
    Move(PointF),
    Line(PointF),
    Quad(PointF, PointF),
    Cubic(PointF, PointF, PointF),
    Close,
}
pub trait NativePath: Send + Sync {
    fn Apply(&self, callback: &mut dyn FnMut(NativePathElement));
    fn IsEmpty(&self) -> bool;
    fn Length(&self) -> f32;
    fn IsLine(&self) -> bool;
    fn Transformed(
        &self,
        transform: &crate::blink_geometry::transforms::affine_transform::AffineTransform,
    ) -> std::sync::Arc<dyn NativePath>;
    fn StrokePath(
        &self,
        stroke: StrokeData,
        dashes: &[f32],
        dash_offset: f32,
        precision: f32,
    ) -> std::sync::Arc<dyn NativePath>;
    fn CountVerbs(&self) -> i32;
    fn BoundingRect(&self) -> RectF;
    fn StrokeBoundingRect(&self, stroke: StrokeData) -> RectF;
    fn Contains(&self, point: PointF, rule: WindRule) -> bool;
    fn Intersects(&self, quad: &QuadF, rule: WindRule) -> bool;
}

/// Source SVG shape-specific fill data, in the same local user space as Path.
#[derive(Clone, Copy)]
pub enum ShapeFillGeometry {
    Path,
    Rectangle(RectF),
    Ellipse {
        center: PointF,
        radii: crate::gfx::SizeF,
    },
}

/// StrokeData::SetLineDash, platform/geometry/stroke_data.cc:35-50. Share the
/// source odd-length repetition with both native Path and PaintFlags adapters.
pub fn RepeatedDashIntervals(dashes: &[f32]) -> Vec<f32> {
    let count = if dashes.len().is_multiple_of(2) {
        dashes.len()
    } else {
        dashes.len() * 2
    };
    (0..count).map(|i| dashes[i % dashes.len()]).collect()
}

#[derive(Clone)]
pub enum ShapeStrokeGeometry {
    Rectangle {
        rect: RectF,
        width: f32,
    },
    Circle {
        center: PointF,
        radius: f32,
        width: f32,
    },
    Path {
        path: std::sync::Arc<dyn NativePath>,
        host_transform: crate::blink_geometry::transforms::affine_transform::AffineTransform,
    },
}

/// The primitive PathBuilder operations used by DOM-owned HTMLAreaElement.
/// The embedding supplies the same native geometry backend as Layout/Paint;
/// this port accepts geometry only and makes no DOM or hit-test decisions.
pub trait NativePathFactory: Send + Sync {
    fn Empty(&self) -> std::sync::Arc<dyn NativePath>;
    fn Rect(&self, origin: PointF, opposite: PointF) -> std::sync::Arc<dyn NativePath>;
    fn Ellipse(
        &self,
        center: PointF,
        radius_x: f32,
        radius_y: f32,
    ) -> std::sync::Arc<dyn NativePath>;
    fn Polygon(&self, points: &[PointF], rule: WindRule) -> std::sync::Arc<dyn NativePath>;
}

/// Values read from the actual referencing LayoutObject. Default area shape
/// uses PhysicalBorderBoxRect; a genuine non-LayoutBox has no such rectangle.
#[derive(Clone, Copy)]
pub struct ImageMapContainer {
    pub effective_zoom: f32,
    pub border_box: Option<RectF>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageMapAreaShape {
    #[default]
    kRect,
    kCircle,
    kPoly,
    kDefault,
}
