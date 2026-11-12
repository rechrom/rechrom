//! Source core/paint/contoured_border_geometry.cc:27-269.
//! All style reads borrow its canonical compiled ComputedStyle owner.
use super::{ContouredRect, Corner, CornerCurvature, FloatRoundedRect, Radii};
use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform,
    gfx::{OutsetsF, PointF, QuadF, RectF, SizeF},
    gfx_line_f::LineF,
    gfx_quad_geometry::QuadFHitTestExt,
    PhysicalRect, PhysicalSize, SizeForLengthSize,
};
use layoutng_geometry::geometry::{
    box_sides::PhysicalBoxSides, box_strut::PhysicalBoxStrut, physical_rect::PhysicalRectExt,
};
use layoutng_style::style::computed_style::ComputedStyle;
fn hull(corner: Corner) -> QuadF {
    let half = corner.HalfCorner();
    let normal = half + LineF::new(corner.Outer(), half).Normal();
    let tangent = LineF::new(half, normal);
    let a = tangent
        .IntersectionWith(&LineF::new(corner.Start(), corner.Center()))
        .unwrap_or(corner.Center());
    let b = tangent
        .IntersectionWith(&LineF::new(corner.End(), corner.Center()))
        .unwrap_or(corner.Center());
    QuadF::new(corner.Start(), a, b, corner.End())
}
fn scaled(quad: QuadF, origin: PointF, scale: f32) -> QuadF {
    let mut transform = AffineTransform::default();
    transform
        .Translate(origin.x() as f64, origin.y() as f64)
        .Scale(scale as f64, scale as f64)
        .Translate(-origin.x() as f64, -origin.y() as f64);
    transform.MapQuad(quad)
}
fn optimal(a: QuadF, ao: PointF, b: QuadF, bo: PointF, min: f32, max: f32) -> f32 {
    let check = (min + max) / 2.;
    if max - min <= 0.05 {
        return min;
    }
    if scaled(a, ao, check).IntersectsQuad(&scaled(b, bo, check)) {
        optimal(a, ao, b, bo, min, check)
    } else {
        optimal(a, ao, b, bo, check, max)
    }
}
fn opposite(a: Corner, b: Corner) -> f32 {
    if !a.BoundingBox().Intersects(&b.BoundingBox()) {
        return 1.;
    }
    let ah = hull(a);
    let bh = hull(b);
    if !ah.IntersectsQuad(&bh) {
        return 1.;
    }
    optimal(ah, a.Outer(), bh, b.Outer(), 0., 1.)
}
fn compute(
    style: &ComputedStyle,
    border_rect: RectF,
    size: PhysicalSize,
    sides: PhysicalBoxSides,
) -> ContouredRect {
    let mut rounded = FloatRoundedRect::new(border_rect, Radii::default());
    if style.HasBorderRadius() {
        let size = SizeF::new(size.width.ToFloat(), size.height.ToFloat());
        let values = [
            style.BorderTopLeftRadius(),
            style.BorderTopRightRadius(),
            style.BorderBottomRightRadius(),
            style.BorderBottomLeftRadius(),
        ];
        let shapes = [
            style.CornerTopLeftShape(),
            style.CornerTopRightShape(),
            style.CornerBottomRightShape(),
            style.CornerBottomLeftShape(),
        ];
        let included = [
            sides.top && sides.left,
            sides.top && sides.right,
            sides.bottom && sides.right,
            sides.bottom && sides.left,
        ];
        rounded.radii = Radii(std::array::from_fn(|i| {
            if included[i] && !shapes[i].IsDegenerate() {
                SizeForLengthSize(values[i], &size)
            } else {
                SizeF::default()
            }
        }));
        rounded.ConstrainRadii();
    }
    let mut result = ContouredRect::new(rounded);
    if !rounded.IsRounded() {
        return result;
    }
    let shapes = [
        style.CornerTopLeftShape(),
        style.CornerTopRightShape(),
        style.CornerBottomRightShape(),
        style.CornerBottomLeftShape(),
    ];
    let curvature = CornerCurvature(std::array::from_fn(|i| {
        if rounded.radii.0[i].IsEmpty() {
            CornerCurvature::ROUND
        } else {
            shapes[i].Exponent() as f32
        }
    }));
    if curvature.IsRound() {
        return result;
    }
    result.curvature = curvature;
    if !curvature.IsConvex() {
        let factor = 1_f32.min(
            opposite(result.TopLeftCorner(), result.BottomRightCorner())
                .min(opposite(result.BottomLeftCorner(), result.TopRightCorner())),
        );
        assert!(factor <= 1.);
        if factor != 1. {
            result.rounded.radii.Scale(factor);
        }
    }
    result.SetOriginRect(result.rounded);
    result
}
pub fn ContouredBorder(
    style: &ComputedStyle,
    rect: PhysicalRect,
    sides: PhysicalBoxSides,
) -> ContouredRect {
    compute(style, RectF::from(rect), rect.size, sides)
}
pub fn PixelSnappedContouredBorder(
    style: &ComputedStyle,
    rect: PhysicalRect,
    sides: PhysicalBoxSides,
) -> ContouredRect {
    compute(
        style,
        RectF::from(foundation::ToPixelSnappedRect(rect)),
        rect.size,
        sides,
    )
}
pub fn ContouredInnerBorder(style: &ComputedStyle, rect: PhysicalRect) -> ContouredRect {
    let mut result = ContouredBorder(style, rect, PhysicalBoxSides::default());
    result.Inset(OutsetsF::new(
        style.BorderTopWidth() as f32,
        style.BorderRightWidth() as f32,
        style.BorderBottomWidth() as f32,
        style.BorderLeftWidth() as f32,
    ));
    result
}
pub fn PixelSnappedContouredInnerBorder(
    style: &ComputedStyle,
    rect: PhysicalRect,
    sides: PhysicalBoxSides,
) -> ContouredRect {
    PixelSnappedContouredBorderWithOutsets(
        style,
        rect,
        PhysicalBoxStrut::FromInts(
            -style.BorderTopWidth(),
            -style.BorderRightWidth(),
            -style.BorderBottomWidth(),
            -style.BorderLeftWidth(),
        ),
        sides,
    )
}
pub fn PixelSnappedContouredBorderWithOutsets(
    style: &ComputedStyle,
    rect: PhysicalRect,
    outsets: PhysicalBoxStrut,
    sides: PhysicalBoxSides,
) -> ContouredRect {
    let adjusted = PhysicalBoxStrut::new(
        if sides.top {
            outsets.top
        } else {
            Default::default()
        },
        if sides.right {
            outsets.right
        } else {
            Default::default()
        },
        if sides.bottom {
            outsets.bottom
        } else {
            Default::default()
        },
        if sides.left {
            outsets.left
        } else {
            Default::default()
        },
    );
    let mut expanded = rect;
    expanded.Expand(&adjusted);
    expanded.size.ClampNegativeToZero();
    let snapped = foundation::gfx::Rect::new(
        foundation::ToRoundedPoint(expanded.offset),
        foundation::gfx::Size::new(
            foundation::SnapSizeToPixelAllowingZero(expanded.Width(), expanded.X()),
            foundation::SnapSizeToPixelAllowingZero(expanded.Height(), expanded.Y()),
        ),
    );
    let mut result = ContouredRect::new(FloatRoundedRect::new(
        RectF::from(snapped),
        Radii::default(),
    ));
    if style.HasBorderRadius() {
        let mut border = PixelSnappedContouredBorder(style, rect, sides);
        border.Outset(OutsetsF::new(
            adjusted.top.ToFloat(),
            adjusted.right.ToFloat(),
            adjusted.bottom.ToFloat(),
            adjusted.left.ToFloat(),
        ));
        result.rounded.radii = border.rounded.radii;
        result.curvature = border.curvature;
        result.SetOriginRect(border.GetOriginRect());
    }
    result
}

/// Source ClipPathClipper::RoundedReferenceBox, clip_path_clipper.cc:242-257.
pub fn GeometryBoxReference(
    style: &ComputedStyle,
    border_rect: PhysicalRect,
    geometry: layoutng_style::style::computed_style_constants::GeometryBox,
    outsets: OutsetsF,
) -> ContouredRect {
    let mut result = ContouredBorder(style, border_rect, PhysicalBoxSides::default());
    if geometry == layoutng_style::style::computed_style_constants::GeometryBox::kMarginBox {
        result.OutsetWithCornerCorrection(outsets);
    } else {
        result.Outset(outsets);
    }
    result
}
