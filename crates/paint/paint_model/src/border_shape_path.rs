//! Source BorderShapePainter::OuterPath/RelevantSideForBorderShape.
//! Source: core/paint/border_shape_painter.cc:42-102;
//! core/paint/border_shape_utils.cc:54-80.
use crate::basic_shape_path::InvalidBasicShape;
use foundation::{gfx::RectF, EBorderStyle};
use layoutng_style::style::{
    basic_shapes::{BasicShape, BasicShapePolygon, ShapeType},
    computed_style::ComputedStyle,
};
pub trait BorderShapePathSink {
    type Path;
    fn BasicShape(
        &self,
        shape: &dyn BasicShape,
        reference: RectF,
        zoom: f32,
    ) -> Result<Self::Path, InvalidBasicShape>;
    fn SegmentMasks(&self, path: &Self::Path) -> u32;
    fn StrokeUnion(
        &self,
        path: Self::Path,
        thickness: f32,
        miter_limit: f32,
        round_join: bool,
    ) -> Self::Path;
}
pub fn OuterPath<S: BorderShapePathSink>(
    style: &ComputedStyle,
    reference: RectF,
    sink: &S,
) -> Result<S::Path, InvalidBasicShape> {
    let shape =
        unsafe { style.BorderShape().Get().as_ref() }.expect("source requires border-shape");
    let path = sink.BasicShape(shape.OuterShape(), reference, style.EffectiveZoom())?;
    if shape.HasSeparateInnerShape() {
        return Ok(path);
    }
    let widths = style.PhysicalBorderWidthToLogical();
    let styles = style.PhysicalBorderStyleToLogical();
    let sides = [
        (styles.BlockStart(), widths.BlockStart()),
        (styles.InlineStart(), widths.InlineStart()),
        (styles.BlockEnd(), widths.BlockEnd()),
        (styles.InlineEnd(), widths.InlineEnd()),
    ];
    let thickness = sides
        .iter()
        .find(|side| side.0 != EBorderStyle::kNone)
        .unwrap_or(&sides[0])
        .1 as f32;
    if thickness == 0. {
        return Ok(path);
    }
    // Source SkPath::kLine_SegmentMask == 1. Curves retain default miter=4.
    let miter = if sink.SegmentMasks(&path) & !1 == 0 {
        1e10
    } else {
        4.
    };
    let round = shape.OuterShape().GetType() == ShapeType::kBasicShapePolygonType
        && unsafe { &*(shape.OuterShape() as *const dyn BasicShape as *const BasicShapePolygon) }
            .HasRoundingRadius();
    Ok(sink.StrokeUnion(path, thickness, miter, round))
}
