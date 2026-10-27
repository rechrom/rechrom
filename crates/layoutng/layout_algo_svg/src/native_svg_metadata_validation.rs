//! Native SVG metadata validation.
//!
//! Included by `layout_node_metadata` at the reciprocal assembly boundary.
//! Ordinary elements bypass this entry; SVG path commands, text-path points,
//! and transform coefficients are validated under an SVG sampling frame.

use super::*;

#[inline(never)]
pub(super) fn ValidateSvgElementInput(
    kind: NodeKind,
    data: &ElementData,
) -> Result<(), &'static str> {
    if let Some(shape) = data.svg_shape.as_ref() {
        if kind != NodeKind::kSvgShape {
            return Err("SVG shape geometry requires an SVG shape node");
        }
        let finite = |value: f64| value.is_finite() && value.abs() <= f32::MAX as f64;
        if !finite(shape.bounds_offset.x)
            || !finite(shape.bounds_offset.y)
            || !finite(shape.bounds_width)
            || !finite(shape.bounds_height)
            || shape.bounds_width < 0.0
            || shape.bounds_height < 0.0
        {
            return Err("SVG shape bounds must be finite and nonnegative");
        }
        if shape.geometry == SvgShapeGeometry::kPath
            && (shape.path.is_empty() || shape.path[0].verb != PaintPathVerb::kMoveTo)
        {
            return Err("SVG path must start with move-to");
        }
        for command in &shape.path {
            for point in [command.control1, command.control2, command.point] {
                if !finite(point.x) || !finite(point.y) {
                    return Err("SVG path coordinates must be finite native values");
                }
            }
        }
        if let Some(transform) = shape.local_transform.as_ref() {
            for value in transform.values {
                if !finite(value) {
                    return Err("SVG shape transform must be finite native values");
                }
            }
        }
    } else if kind == NodeKind::kSvgShape {
        return Err("An SVG shape node requires resolved geometry");
    }
    if let Some(path) = data.svg_text_path.as_ref() {
        if kind != NodeKind::kSvgTextPath {
            return Err("SVG textPath geometry requires an SVG textPath node");
        }
        if !path.start_offset.is_finite() {
            return Err("SVG textPath offset must be finite");
        }
        for point in &path.points {
            if !point.x.is_finite()
                || !point.y.is_finite()
                || point.x.abs() > f32::MAX as f64
                || point.y.abs() > f32::MAX as f64
            {
                return Err("SVG textPath points must be finite native coordinates");
            }
        }
    }
    Ok(())
}
