//! Thin bindings to public canonical Skia value algorithms.
//! No region boolean operations, scan conversion or path algorithms live here.
//! Moved from compositor/src/runtime/skia.rs (which re-exports it): both the
//! main and impl halves name this backend, and the traits it implements live
//! in the commit vocabulary or its shared sub-crates.
use super::region_backend::{SkIRect, SkRegionBackend, SkRegionOp};
use compositor_tree::region::Region;
use std::sync::OnceLock;

pub use skia::src::core::SkPath::SkPath as BoundaryPath;
use skia::src::core::{
    SkRect::SkIRect as SkRegionIRect,
    SkRegion::{Op, RegionIterator as SkRegionIterator, SkRegion as SkRegionStorage},
};

#[derive(Clone, Default, PartialEq)]
pub struct SkRegion(SkRegionStorage);

fn sk_region_rect(rect: SkIRect) -> SkRegionIRect {
    SkRegionIRect::make_ltrb(rect.left, rect.top, rect.right, rect.bottom)
}
fn region_rect(rect: SkRegionIRect) -> SkIRect {
    SkIRect {
        left: rect.fLeft,
        top: rect.fTop,
        right: rect.fRight,
        bottom: rect.fBottom,
    }
}
fn region_op(op: SkRegionOp) -> Op {
    match op {
        SkRegionOp::Difference => Op::Difference,
        SkRegionOp::Intersect => Op::Intersect,
        SkRegionOp::Union => Op::Union,
        SkRegionOp::XOR => Op::Xor,
    }
}
/// Borrowed Skia RLE cursor; each next projects one current LTRB value.
pub struct RegionRectIterator<'a>(SkRegionIterator<'a>);
impl Iterator for RegionRectIterator<'_> {
    type Item = SkIRect;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.done() {
            return None;
        }
        let rect = region_rect(*self.0.rect());
        self.0.next();
        Some(rect)
    }
}
impl SkRegionBackend for SkRegion {
    type BoundaryPath = BoundaryPath;
    type RectIterator<'a> = RegionRectIterator<'a>;
    fn from_rect(rect: SkIRect) -> Self {
        Self(SkRegionStorage::from_rect(sk_region_rect(rect)))
    }
    fn shared_empty_region() -> &'static Region<Self> {
        static EMPTY: OnceLock<Region<SkRegion>> = OnceLock::new();
        EMPTY.get_or_init(Region::default)
    }
    fn translate(&mut self, x: i32, y: i32) {
        SkRegionStorage::translate(&mut self.0, x, y);
    }
    fn setEmpty(&mut self) {
        {
            self.0.set_empty();
        }
    }
    fn isEmpty(&self) -> bool {
        {
            self.0.is_empty()
        }
    }
    fn computeRegionComplexity(&self) -> i32 {
        {
            self.0.compute_region_complexity()
        }
    }
    fn getBoundaryPath(&self) -> BoundaryPath {
        {
            self.0.get_boundary_path()
        }
    }
    fn contains_point(&self, x: i32, y: i32) -> bool {
        {
            self.0.contains_xy(x, y)
        }
    }
    fn contains_rect(&self, rect: SkIRect) -> bool {
        {
            SkRegionStorage::contains_rect(&self.0, &sk_region_rect(rect))
        }
    }
    fn contains_region(&self, region: &Self) -> bool {
        {
            self.0.contains_region(&region.0)
        }
    }
    fn intersects_rect(&self, rect: SkIRect) -> bool {
        {
            SkRegionStorage::intersects_rect(&self.0, &sk_region_rect(rect))
        }
    }
    fn intersects_region(&self, region: &Self) -> bool {
        {
            self.0.intersects_region(&region.0)
        }
    }
    fn op_rect(&mut self, rect: SkIRect, op: SkRegionOp) {
        {
            SkRegionStorage::op_rect(&mut self.0, sk_region_rect(rect), region_op(op));
        }
    }
    fn op_region(&mut self, region: &Self, op: SkRegionOp) {
        {
            SkRegionStorage::op_region(&mut self.0, &region.0, region_op(op));
        }
    }
    fn setRects(&mut self, rects: &[SkIRect]) {
        {
            let rects = rects
                .iter()
                .copied()
                .map(sk_region_rect)
                .collect::<Vec<_>>();
            self.0.set_rects(&rects);
        }
    }
    fn getBounds(&self) -> SkIRect {
        {
            region_rect(*self.0.get_bounds())
        }
    }
    fn rects(&self) -> RegionRectIterator<'_> {
        RegionRectIterator(SkRegionIterator::new(&self.0))
    }
}

use super::geometry::{RRectF, RectF, Vector2dF};
use super::transform::RoundedRectTransformBackend;
use skia::src::core::{
    SkMatrix::SkMatrix as TranslatedMatrix, SkPoint::SkPoint as TranslatedPoint,
    SkRRect::SkRRect as TranslatedRRect, SkRect::SkRect as TranslatedRect,
};
pub type SkRoundedRect = TranslatedRRect;
fn translated_rrect(value: &RRectF) -> TranslatedRRect {
    let rect = TranslatedRect::MakeLTRB(
        value.rect.x(),
        value.rect.y(),
        value.rect.right(),
        value.rect.bottom(),
    );
    let radii = value.corner_radii.map(|r| TranslatedPoint {
        fX: r.x(),
        fY: r.y(),
    });
    TranslatedRRect::MakeRectRadii(&rect, &radii)
}
pub fn MakeSkRoundedRect(value: &RRectF) -> SkRoundedRect {
    return translated_rrect(value);
}
pub fn OffsetSkRoundedRect(value: &mut SkRoundedRect, offset: Vector2dF) {
    value.offset(offset.x(), offset.y());
}
pub fn SkRoundedRectValue(value: &SkRoundedRect) -> RRectF {
    {
        let bounds = value.getBounds();
        return RRectF {
            rect: RectF::new(
                bounds.fLeft,
                bounds.fTop,
                bounds.fRight - bounds.fLeft,
                bounds.fBottom - bounds.fTop,
            ),
            corner_radii: std::array::from_fn(|index| {
                let radius = value.all_radii()[index];
                Vector2dF::new(radius.fX, radius.fY)
            }),
        };
    }
}
impl RoundedRectTransformBackend for SkRegion {
    fn TransformRoundedRect(matrix: [f32; 9], source: &RRectF, output: &mut RRectF) -> bool {
        {
            let matrix = TranslatedMatrix::MakeAll(
                matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5], matrix[6],
                matrix[7], matrix[8],
            );
            let Some(result) = translated_rrect(source).transform(&matrix) else {
                return false;
            };
            let rect = result.getBounds();
            *output = RRectF {
                rect: RectF::new(
                    rect.fLeft,
                    rect.fTop,
                    rect.fRight - rect.fLeft,
                    rect.fBottom - rect.fTop,
                ),
                corner_radii: std::array::from_fn(|i| {
                    let radius = result.all_radii()[i];
                    Vector2dF::new(radius.fX, radius.fY)
                }),
            };
            true
        }
    }
    fn IsRoundedRectValid(value: &RRectF) -> bool {
        return translated_rrect(value).isValid();
    }
}

pub use paint_model::cc::paint::filter_operations::FilterOperations;
pub use skia::src::core::SkPath::SkPath;

fn stroke_backend_path(
    path: &SkPath,
    width: f32,
    cap: paint_model::cc::paint::paint_flags::Cap,
    join: paint_model::cc::paint::paint_flags::Join,
    miter_limit: f32,
    dash_intervals: &[f32],
    dash_phase: f32,
    resolution_scale: f32,
) -> Option<SkPath> {
    if !width.is_finite()
        || width < 0.0
        || !miter_limit.is_finite()
        || !dash_phase.is_finite()
        || !resolution_scale.is_finite()
        || resolution_scale <= 0.0
        || dash_intervals.iter().any(|value| !value.is_finite())
        || !path.isFinite()
        || path.isEmpty()
    {
        return None;
    }
    use skia::src::{
        core::{
            SkPaint::{Cap as SkCap, Join as SkJoin},
            SkPathBuilder::SkPathBuilder,
            SkStrokeRec::{InitStyle, SkStrokeRec},
        },
        effects::SkDashPathEffect,
    };
    let mut stroke = SkStrokeRec::new(InitStyle::kHairline_InitStyle);
    stroke.setStrokeStyle(width, false);
    stroke.setStrokeParams(
        match cap {
            paint_model::cc::paint::paint_flags::Cap::kButt_Cap => SkCap::kButt_Cap,
            paint_model::cc::paint::paint_flags::Cap::kRound_Cap => SkCap::kRound_Cap,
            paint_model::cc::paint::paint_flags::Cap::kSquare_Cap => SkCap::kSquare_Cap,
        },
        match join {
            paint_model::cc::paint::paint_flags::Join::kMiter_Join => SkJoin::kMiter_Join,
            paint_model::cc::paint::paint_flags::Join::kRound_Join => SkJoin::kRound_Join,
            paint_model::cc::paint::paint_flags::Join::kBevel_Join => SkJoin::kBevel_Join,
        },
        miter_limit,
    );
    stroke.setResScale(resolution_scale);
    let mut filtered = SkPathBuilder::new();
    let filtered_path = if dash_intervals.is_empty() {
        None
    } else {
        let effect = SkDashPathEffect::Make(dash_intervals, dash_phase);
        effect
            .get()
            .filter(|effect| effect.filterPathWithoutCTM(&mut filtered, path, &mut stroke))
            .map(|_| filtered.detach(None))
    };
    let source = filtered_path.as_ref().unwrap_or(path);
    if stroke.isFillStyle() {
        return Some(source.clone());
    }
    let mut outline = SkPathBuilder::new();
    stroke
        .applyToPath(&mut outline, source)
        .then(|| outline.detach(None))
}

impl super::skia_value_backend::MainSkiaBackend for SkRegion {
    type Filters = FilterOperations;
    type Path = SkPath;
    fn FromPaintPath(path: &paint_model::paint_clip_path::PaintPath) -> Self::Path {
        paint_skia::path::FromPaintPath(path)
    }

    fn StrokeBounds(
        path: &paint_model::paint_clip_path::PaintPath,
        width: f32,
        resolution_scale: f32,
    ) -> Option<RectF> {
        if !width.is_finite() || !resolution_scale.is_finite() || resolution_scale <= 0.0 {
            return None;
        }
        {
            use skia::src::core::{
                SkPathBuilder::SkPathBuilder,
                SkStrokeRec::{InitStyle, SkStrokeRec},
            };
            let path = Self::FromPaintPath(path);
            if !path.isFinite() || path.isEmpty() {
                return None;
            }
            let stroked = if width == 0.0 {
                path
            } else {
                let mut stroke = SkStrokeRec::new(InitStyle::kHairline_InitStyle);
                stroke.setStrokeStyle(width, false);
                stroke.setResScale(resolution_scale);
                let mut builder = SkPathBuilder::new();
                stroke
                    .applyToPath(&mut builder, &path)
                    .then(|| builder.detach(None))?
            };
            let bounds = stroked.computeTightBounds();
            [bounds.fLeft, bounds.fTop, bounds.fRight, bounds.fBottom]
                .into_iter()
                .all(f32::is_finite)
                .then(|| {
                    RectF::new(
                        bounds.fLeft,
                        bounds.fTop,
                        bounds.fRight - bounds.fLeft,
                        bounds.fBottom - bounds.fTop,
                    )
                })
        }
    }
    fn StrokePath(
        path: &paint_model::paint_clip_path::PaintPath,
        width: f32,
        cap: paint_model::cc::paint::paint_flags::Cap,
        join: paint_model::cc::paint::paint_flags::Join,
        miter_limit: f32,
        dash_intervals: &[f32],
        dash_phase: f32,
        resolution_scale: f32,
    ) -> Option<Self::Path> {
        let path = Self::FromPaintPath(path);
        stroke_backend_path(
            &path,
            width,
            cap,
            join,
            miter_limit,
            dash_intervals,
            dash_phase,
            resolution_scale,
        )
    }
    fn StrokeBackendPath(
        path: &Self::Path,
        width: f32,
        cap: paint_model::cc::paint::paint_flags::Cap,
        join: paint_model::cc::paint::paint_flags::Join,
        miter_limit: f32,
        dash_intervals: &[f32],
        dash_phase: f32,
        resolution_scale: f32,
    ) -> Option<Self::Path> {
        stroke_backend_path(
            path,
            width,
            cap,
            join,
            miter_limit,
            dash_intervals,
            dash_phase,
            resolution_scale,
        )
    }
}
