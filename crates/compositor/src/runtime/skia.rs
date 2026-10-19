//! Thin bindings to public canonical Skia value algorithms.
//! No region boolean operations, scan conversion or path algorithms live here.
use crate::adapter::region_backend::{SkIRect, SkRegionBackend, SkRegionOp};
use crate::cc::base::region::Region;
pub use skia::include::core::SkRegion::{BoundaryPath, SkRegion};
use skia::include::core::SkRegion::{Iterator as SkRegionIterator, Op, SkRegionIRect};
use std::sync::OnceLock;

impl From<SkIRect> for SkRegionIRect {
    fn from(rect: SkIRect) -> Self {
        Self::MakeLTRB(rect.left, rect.top, rect.right, rect.bottom)
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
        SkRegionOp::Difference => Op::kDifference_Op,
        SkRegionOp::Intersect => Op::kIntersect_Op,
        SkRegionOp::Union => Op::kUnion_Op,
        SkRegionOp::XOR => Op::kXOR_Op,
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
        Self::translate(self, x, y);
    }
    fn setEmpty(&mut self) {
        Self::setEmpty(self);
    }
    fn isEmpty(&self) -> bool {
        Self::isEmpty(self)
    }
    fn computeRegionComplexity(&self) -> i32 {
        Self::computeRegionComplexity(self)
    }
    fn getBoundaryPath(&self) -> BoundaryPath {
        Self::getBoundaryPath(self)
    }
    fn contains_point(&self, x: i32, y: i32) -> bool {
        Self::contains_point(self, x, y)
    }
    fn contains_rect(&self, rect: SkIRect) -> bool {
        Self::contains_rect(self, rect.into())
    }
    fn contains_region(&self, region: &Self) -> bool {
        Self::contains(self, region)
    }
    fn intersects_rect(&self, rect: SkIRect) -> bool {
        Self::intersects_rect(self, rect.into())
    }
    fn intersects_region(&self, region: &Self) -> bool {
        Self::intersects(self, region)
    }
    fn op_rect(&mut self, rect: SkIRect, op: SkRegionOp) {
        Self::op_rect(self, rect.into(), region_op(op));
    }
    fn op_region(&mut self, region: &Self, op: SkRegionOp) {
        Self::op(self, region, region_op(op));
    }
    fn setRects(&mut self, rects: &[SkIRect]) {
        Self::setRects(self, rects);
    }
    fn getBounds(&self) -> SkIRect {
        (*Self::getBounds(self)).into()
    }
    fn rects(&self) -> RegionRectIterator<'_> {
        RegionRectIterator(SkRegionIterator::new(self))
    }
}

use crate::adapter::geometry::{RRectF, RectF, Vector2dF};
use crate::runtime::geometry::RoundedRectTransformBackend;
use skia::include::core::SkRRect::{SkRRect, SkRRectBounds, SkRRectMatrix, SkRRectVector};
fn native_rrect(value: &RRectF) -> SkRRect {
    let rect = SkRRectBounds::MakeLTRB(
        value.rect.x(),
        value.rect.y(),
        value.rect.right(),
        value.rect.bottom(),
    );
    let radii = value.corner_radii.map(|r| SkRRectVector::new(r.x(), r.y()));
    let mut result = SkRRect::default();
    result.setRectRadii(rect, radii);
    result
}
impl RoundedRectTransformBackend for SkRegion {
    fn TransformRoundedRect(matrix: [f32; 9], source: &RRectF, output: &mut RRectF) -> bool {
        let Some(result) = native_rrect(source).transform(&SkRRectMatrix::MakeAll(matrix)) else {
            return false;
        };
        let rect = result.getBounds();
        *output = RRectF {
            rect: RectF::new(
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
            ),
            corner_radii: result.radii().map(|r| Vector2dF::new(r.x, r.y)),
        };
        true
    }
    fn IsRoundedRectValid(value: &RRectF) -> bool {
        native_rrect(value).isValid()
    }
}

use crate::adapter::{geometry::Rect, occlusion::SkMatrix};
use foundation::gfx;
use paint::cc::paint::filter_operation::FilterType;
pub use paint::cc::paint::filter_operations::FilterOperations;
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
        let matrix = SkRRectMatrix::MakeAll([scale.x(), 0., 0., 0., scale.y(), 0., 0., 0., 1.]);
        filter_rect(Self::ExpandRect(self, native_filter_rect(rect), &matrix))
    }
}
impl crate::adapter::damage_context::DamageFilters for FilterOperations {
    fn HasFilterThatMovesPixels(&self) -> bool {
        Self::HasFilterThatMovesPixels(self)
    }
    fn MapRect(&self, rect: Rect, transform: &SkMatrix) -> Rect {
        filter_rect(Self::MapRect(
            self,
            native_filter_rect(rect),
            Some(&SkRRectMatrix::MakeAll(transform.values)),
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
        filter_rect(Self::MapRectReverse(
            self,
            native_filter_rect(rect),
            &SkRRectMatrix::MakeAll(transform.values),
        ))
    }
}

pub use skia::include::core::SkPath::SkPath;
impl crate::adapter::geometry::Path for SkPath {
    fn AsRRect(&self) -> Option<RRectF> {
        let rr = self.isRRect()?;
        let b = rr.getBounds();
        Some(RRectF {
            rect: RectF::new(b.left, b.top, b.right - b.left, b.bottom - b.top),
            corner_radii: rr.radii().map(|r| Vector2dF::new(r.x, r.y)),
        })
    }
    fn bounds(&self) -> RectF {
        let b = self.getBounds();
        RectF::new(b.left, b.top, b.right - b.left, b.bottom - b.top)
    }
    fn count_points(&self) -> i32 {
        self.countPoints()
    }
    fn count_verbs(&self) -> i32 {
        self.countVerbs()
    }
}
impl crate::adapter::main_layer::MainLayerPath for SkPath {
    fn FromRRect(value: &RRectF) -> Self {
        Self::FromRRect(&native_rrect(value))
    }
}
impl crate::runtime::types::SkiaValueBackend for SkRegion {
    type Filters = FilterOperations;
    type Path = SkPath;
}

// Source consumer: cc/layers/render_surface_impl.cc:242-246. The selected
// native FilterOperations owns the exact expansion algorithm; this only maps
// the external 3x3/value representations at the package boundary.
impl crate::adapter::render_surface_impl::RenderSurfaceFilters for FilterOperations {
    fn ExpandRectWithMatrix(&self, rect: Rect, matrix: &SkMatrix) -> Rect {
        filter_rect(Self::ExpandRect(
            self,
            native_filter_rect(rect),
            &SkRRectMatrix::MakeAll(matrix.values),
        ))
    }
}
