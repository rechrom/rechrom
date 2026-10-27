//! Native `shape-outside` style projection.
//!
//! Included by `boundary::native_input` at the reciprocal assembly boundary.
//! Contour and radius validation scale with the supplied shape, so the public
//! entry stays out of line as a float-algorithm sampling frame.

use super::*;

// cpp: layoutng/internal/boundary/native_input.cc:1079-1135
#[inline(never)]
pub(super) fn ApplyNativeStyleShape(builder: &mut ComputedStyleBuilder, extra: &ExtendedStyle) {
    if let Some(shape) = &extra.shape_outside {
        let validate_coordinate = |coordinate: &ShapeCoordinate, field: &str| {
            Number(coordinate.pixels, field, false);
            Number(coordinate.percentage, field, false);
        };
        let validate_point = |point: &ShapePoint, field: &str| {
            validate_coordinate(&point.x, field);
            validate_coordinate(&point.y, field);
        };
        match shape.kind {
            ShapeOutsideKind::kReferenceBox => {
                if !shape.contour.is_empty() {
                    Invalid("shape_outside reference contour");
                }
            }
            ShapeOutsideKind::kPolygon => {
                if shape.contour.len() < 3 {
                    Invalid("shape_outside polygon");
                }
            }
            ShapeOutsideKind::kCircle | ShapeOutsideKind::kEllipse | ShapeOutsideKind::kInset => {
                if !shape.contour.is_empty() {
                    Invalid("shape_outside non-polygon contour");
                }
            }
            ShapeOutsideKind::kImage => {
                if shape.resource_id == 0 || !shape.contour.is_empty() {
                    Invalid("shape_outside image resource");
                }
            }
        }
        for point in &shape.contour {
            validate_point(point, "shape_outside contour");
        }
        validate_point(&shape.center, "shape_outside center");
        for radius in [&shape.radius_x, &shape.radius_y] {
            match radius.kind {
                ShapeRadiusKind::kLengthPercentage
                | ShapeRadiusKind::kClosestSide
                | ShapeRadiusKind::kFarthestSide
                | ShapeRadiusKind::kClosestCorner
                | ShapeRadiusKind::kFarthestCorner => {}
            }
            validate_coordinate(&radius.value, "shape_outside radius");
        }
        for inset in &shape.insets {
            validate_coordinate(inset, "shape_outside inset");
        }
        for radius in &shape.corner_radii {
            validate_point(radius, "shape_outside corner radius");
        }
        builder.SetShapeOutsideOwned(Member::from_ptr(MakeGarbageCollected(
            ShapeValue::from_box(ConvertShapeReferenceBox(shape.reference_box)),
        )));
    }
    builder.SetShapeMargin(&ShapeMarginLength(&extra.shape_margin));
    builder.SetShapeImageThreshold(Number(
        extra.shape_image_threshold,
        "shape_image_threshold",
        false,
    ) as f32);
}
