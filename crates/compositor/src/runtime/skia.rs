//! Thin bindings to public canonical Skia value algorithms.
//! No region boolean operations, scan conversion or path algorithms live here.
use crate::adapter::region_backend::{SkIRect, SkRegionBackend, SkRegionOp};
use crate::cc::base::region::Region;
use std::sync::OnceLock;

pub use skia::src::core::SkPath::SkPath as BoundaryPath;
pub use skia::src::core::SkRegion::SkRegion;
use skia::src::core::{
    SkRect::SkIRect as SkRegionIRect,
    SkRegion::{Op, RegionIterator as SkRegionIterator},
};

impl From<SkIRect> for SkRegionIRect {
    fn from(rect: SkIRect) -> Self {
        {
            Self::make_ltrb(rect.left, rect.top, rect.right, rect.bottom)
        }
    }
}
impl From<SkRegionIRect> for SkIRect {
    fn from(rect: SkRegionIRect) -> Self {
        Self {
            left: rect.fLeft,
            top: rect.fTop,
            right: rect.fRight,
            bottom: rect.fBottom,
        }
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
/// Borrowed native RLE cursor; each next projects one current LTRB value.
pub struct RegionRectIterator<'a>(SkRegionIterator<'a>);
impl Iterator for RegionRectIterator<'_> {
    type Item = SkIRect;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.done() {
            return None;
        }
        let rect = (*self.0.rect()).into();
        self.0.next();
        Some(rect)
    }
}
impl SkRegionBackend for SkRegion {
    type BoundaryPath = BoundaryPath;
    type RectIterator<'a> = RegionRectIterator<'a>;
    fn from_rect(rect: SkIRect) -> Self {
        Self::from_rect(rect.into())
    }
    fn shared_empty_region() -> &'static Region<Self> {
        static EMPTY: OnceLock<Region<SkRegion>> = OnceLock::new();
        EMPTY.get_or_init(Region::default)
    }
    fn translate(&mut self, x: i32, y: i32) {
        SkRegion::translate(self, x, y);
    }
    fn setEmpty(&mut self) {
        {
            self.set_empty();
        }
    }
    fn isEmpty(&self) -> bool {
        {
            self.is_empty()
        }
    }
    fn computeRegionComplexity(&self) -> i32 {
        {
            self.compute_region_complexity()
        }
    }
    fn getBoundaryPath(&self) -> BoundaryPath {
        {
            self.get_boundary_path()
        }
    }
    fn contains_point(&self, x: i32, y: i32) -> bool {
        {
            self.contains_xy(x, y)
        }
    }
    fn contains_rect(&self, rect: SkIRect) -> bool {
        {
            SkRegion::contains_rect(self, &rect.into())
        }
    }
    fn contains_region(&self, region: &Self) -> bool {
        {
            self.contains_region(region)
        }
    }
    fn intersects_rect(&self, rect: SkIRect) -> bool {
        {
            SkRegion::intersects_rect(self, &rect.into())
        }
    }
    fn intersects_region(&self, region: &Self) -> bool {
        {
            self.intersects_region(region)
        }
    }
    fn op_rect(&mut self, rect: SkIRect, op: SkRegionOp) {
        {
            SkRegion::op_rect(self, rect.into(), region_op(op));
        }
    }
    fn op_region(&mut self, region: &Self, op: SkRegionOp) {
        {
            SkRegion::op_region(self, region, region_op(op));
        }
    }
    fn setRects(&mut self, rects: &[SkIRect]) {
        {
            let rects = rects.iter().copied().map(Into::into).collect::<Vec<_>>();
            self.set_rects(&rects);
        }
    }
    fn getBounds(&self) -> SkIRect {
        {
            (*self.get_bounds()).into()
        }
    }
    fn rects(&self) -> RegionRectIterator<'_> {
        RegionRectIterator(SkRegionIterator::new(self))
    }
}

use crate::adapter::geometry::{RRectF, RectF, Vector2dF};
use crate::runtime::geometry::RoundedRectTransformBackend;
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

use crate::adapter::{geometry::Rect, occlusion::SkMatrix};
use foundation::gfx;
use paint::cc::paint::filter_operation::{FilterMapTransform, FilterType};
pub use paint::cc::paint::filter_operations::FilterOperations;

/// Feature-selected native matrix used only for FilterOperation::mapVector.
/// Adapter property trees keep their source-shaped nine scalar values while
/// each production Skia path executes its own matrix implementation.
struct NativeFilterMapTransform {
    matrix: TranslatedMatrix,
}
impl NativeFilterMapTransform {
    fn new(values: [f32; 9]) -> Self {
        {
            Self {
                matrix: TranslatedMatrix::MakeAll(
                    values[0], values[1], values[2], values[3], values[4], values[5], values[6],
                    values[7], values[8],
                ),
            }
        }
    }
}
impl FilterMapTransform for NativeFilterMapTransform {
    fn MapVector(&self, x: f32, y: f32) -> (f32, f32) {
        {
            let mapped = self.matrix.mapVector(TranslatedPoint::make(x, y));
            (mapped.x(), mapped.y())
        }
    }
}
fn native_filter_rect(rect: Rect) -> gfx::Rect {
    gfx::Rect::new(
        gfx::Point::new(rect.x, rect.y),
        gfx::Size::new(rect.width, rect.height),
    )
}
fn filter_rect(rect: gfx::Rect) -> Rect {
    Rect::new(rect.x(), rect.y(), rect.width(), rect.height())
}
impl crate::adapter::property_types::FilterOperations for FilterOperations {
    fn IsEmpty(&self) -> bool {
        Self::IsEmpty(self)
    }
    fn ToString(&self) -> String {
        Self::ToString(self)
    }
}
impl crate::adapter::property_tree::PropertyTreeFilters for FilterOperations {
    fn AllowsLCDText(&self) -> bool {
        Self::AllowsLCDText(self)
    }
    fn HasZoomFilter(&self) -> bool {
        self.HasFilterOfType(FilterType::ZOOM)
    }
}
impl crate::adapter::draw_property::DrawFilters for FilterOperations {
    fn HasReferenceFilter(&self) -> bool {
        Self::HasReferenceFilter(self)
    }
    fn HasFilterThatMovesPixels(&self) -> bool {
        Self::HasFilterThatMovesPixels(self)
    }
    fn HasBlurFilter(&self) -> bool {
        self.HasFilterOfType(FilterType::BLUR)
    }
    fn ExpandRect(&self, rect: Rect, scale: Vector2dF) -> Rect {
        let matrix =
            NativeFilterMapTransform::new([scale.x(), 0., 0., 0., scale.y(), 0., 0., 0., 1.]);
        filter_rect(Self::ExpandRect(
            self,
            native_filter_rect(rect),
            Some(&matrix),
        ))
    }
}
impl crate::adapter::damage_context::DamageFilters for FilterOperations {
    fn HasFilterThatMovesPixels(&self) -> bool {
        Self::HasFilterThatMovesPixels(self)
    }
    fn MapRect(&self, rect: Rect, transform: &SkMatrix) -> Rect {
        let transform = NativeFilterMapTransform::new(transform.values);
        filter_rect(Self::MapRect(
            self,
            native_filter_rect(rect),
            Some(&transform),
        ))
    }
}
impl crate::adapter::occlusion::OcclusionFilters for FilterOperations {
    fn HasFilterThatAffectsOpacity(&self) -> bool {
        Self::HasFilterThatAffectsOpacity(self)
    }
    fn HasFilterThatMovesPixels(&self) -> bool {
        Self::HasFilterThatMovesPixels(self)
    }
    fn MapRectReverse(&self, rect: Rect, transform: &SkMatrix) -> Rect {
        let transform = NativeFilterMapTransform::new(transform.values);
        filter_rect(Self::MapRectReverse(
            self,
            native_filter_rect(rect),
            Some(&transform),
        ))
    }
}

pub use skia::src::core::SkPath::SkPath;
impl crate::adapter::geometry::Path for SkPath {
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
impl crate::adapter::main_layer::MainLayerPath for SkPath {
    fn FromRRect(value: &RRectF) -> Self {
        {
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
}
impl crate::runtime::types::SkiaValueBackend for SkRegion {
    type Filters = FilterOperations;
    type Path = SkPath;
    fn FromPaintPath(path: &paint::paint_clip_path::PaintPath) -> Self::Path {
        {
            use paint::paint_clip_path::{PaintPathFillType, PaintPathGeometry, PaintPathVerb};
            use skia::{
                include::core::SkPathTypes::{SkPathDirection, SkPathFillType},
                src::core::{
                    SkPathBuilder::SkPathBuilder, SkPoint::SkPoint, SkRRect::SkRRect,
                    SkRect::SkRect,
                },
            };
            if let PaintPathGeometry::RoundedRect { rect, radii } = path.geometry() {
                let rect = SkRect::MakeLTRB(
                    rect.x as f32,
                    rect.y as f32,
                    (rect.x + rect.width) as f32,
                    (rect.y + rect.height) as f32,
                );
                let radii = [
                    radii.top_left,
                    radii.top_right,
                    radii.bottom_right,
                    radii.bottom_left,
                ]
                .map(|radius| SkPoint {
                    fX: radius.x as f32,
                    fY: radius.y as f32,
                });
                return SkPath::RRectDefault(
                    &SkRRect::MakeRectRadii(&rect, &radii),
                    SkPathDirection::kCW,
                );
            }
            let fill = match path.fill_type() {
                PaintPathFillType::Winding => SkPathFillType::kWinding,
                PaintPathFillType::EvenOdd => SkPathFillType::kEvenOdd,
                PaintPathFillType::InverseWinding => SkPathFillType::kInverseWinding,
                PaintPathFillType::InverseEvenOdd => SkPathFillType::kInverseEvenOdd,
            };
            let mut result = SkPathBuilder::with_fill_type(fill);
            let PaintPathGeometry::Commands(commands) = path.geometry() else {
                unreachable!("rounded path returned above")
            };
            for command in commands.iter() {
                match command.verb {
                    PaintPathVerb::kMoveTo => {
                        result.moveToXY(command.point.x as f32, command.point.y as f32);
                    }
                    PaintPathVerb::kLineTo => {
                        result.lineToXY(command.point.x as f32, command.point.y as f32);
                    }
                    PaintPathVerb::kQuadraticTo => {
                        result.quadToXY(
                            command.control1.x as f32,
                            command.control1.y as f32,
                            command.point.x as f32,
                            command.point.y as f32,
                        );
                    }
                    PaintPathVerb::kConicTo => {
                        result.conicToXY(
                            command.control1.x as f32,
                            command.control1.y as f32,
                            command.point.x as f32,
                            command.point.y as f32,
                            command.conic_weight as f32,
                        );
                    }
                    PaintPathVerb::kCubicTo => {
                        result.cubicToXY(
                            command.control1.x as f32,
                            command.control1.y as f32,
                            command.control2.x as f32,
                            command.control2.y as f32,
                            command.point.x as f32,
                            command.point.y as f32,
                        );
                    }
                    PaintPathVerb::kClose => {
                        result.close();
                    }
                }
            }
            result.detach(None)
        }
    }
    fn StrokeBounds(
        path: &paint::paint_clip_path::PaintPath,
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
}

// Source consumer: cc/layers/render_surface_impl.cc:242-246. The selected
// native FilterOperations owns the exact expansion algorithm; this only maps
// the external 3x3/value representations at the package boundary.
impl crate::adapter::render_surface_impl::RenderSurfaceFilters for FilterOperations {
    fn ExpandRectWithMatrix(&self, rect: Rect, matrix: &SkMatrix) -> Rect {
        let matrix = NativeFilterMapTransform::new(matrix.values);
        filter_rect(Self::ExpandRect(
            self,
            native_filter_rect(rect),
            Some(&matrix),
        ))
    }
}
