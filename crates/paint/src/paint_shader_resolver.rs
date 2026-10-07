#![allow(non_snake_case)]

use layoutng_assembly::fragment_tree::PaintResources;
use layoutng_assembly::internal::layout_input::{Offset, Size, TransformMatrix};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    PaintRadialExtent, PaintRadialShape, PaintShader, PaintShaderKind,
};

use crate::geometry_mapper::MultiplyTransforms;

// cpp: paint/paint_shader_resolver.cc:13-15
fn Finite(point: Offset) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

// cpp: paint/paint_shader_resolver.cc:17-21
fn ValidateColor(color: Color) {
    if !color.red.is_finite()
        || !color.green.is_finite()
        || !color.blue.is_finite()
        || !color.alpha.is_finite()
    {
        panic!("paint shader color must be finite");
    }
}

// cpp: paint/paint_shader_resolver.cc:23-28
fn ValidateTransform(transform: &TransformMatrix) {
    for value in transform.values {
        if !value.is_finite() {
            panic!("paint shader transform must be finite");
        }
    }
}

// cpp: paint/paint_shader_resolver.cc:30-40
fn ResolvePoint(point: &mut Offset, origin: Offset, unit_box: Option<Size>) {
    if let Some(unit_box) = unit_box {
        point.x = origin.x + point.x * unit_box.width;
        point.y = origin.y + point.y * unit_box.height;
    } else {
        point.x += origin.x;
        point.y += origin.y;
    }
}

// cpp: paint/paint_shader_resolver.cc:42-60
fn ResolvePhysicalColorStops(shader: &mut PaintShader, basis: f64) {
    if !basis.is_finite() || basis < 0.0 {
        panic!("gradient color-stop basis is invalid");
    }
    let mut previous = 0.0;
    for stop in &mut shader.stops {
        if !stop.offset.is_finite() || !stop.offset_length.is_finite() || stop.offset_length < 0.0 {
            panic!("gradient color-stop position is invalid");
        }
        if stop.offset_length != 0.0 {
            if basis == 0.0 {
                panic!("physical gradient color stop has a zero-length gradient");
            }
            stop.offset += stop.offset_length / basis;
            stop.offset_length = 0.0;
        }
        stop.offset = stop.offset.clamp(previous, 1.0);
        previous = stop.offset;
    }
}

// cpp: paint/paint_shader_resolver.h:10-17
// cpp: paint/paint_shader_resolver.cc:64-340
pub fn ResolvePaintShader(
    input: &PaintShader,
    resources: Option<&PaintResources>,
    mut origin: Offset,
    mut unit_box: Option<Size>,
) -> PaintShader {
    let mut output = input.clone();
    ValidateTransform(&output.transform);
    if !Finite(origin) {
        panic!("paint shader origin must be finite");
    }
    if output.unit_coordinates {
        if unit_box.is_none_or(|unit_box| {
            !unit_box.width.is_finite()
                || !unit_box.height.is_finite()
                || unit_box.width < 0.0
                || unit_box.height < 0.0
        }) {
            panic!("unit shader requires finite paint bounds");
        }
    } else {
        unit_box = None;
    }

    if output.kind == PaintShaderKind::kPattern {
        if output.resource_id == 0
            || !Finite(output.tile_offset)
            || !output.tile_width.is_finite()
            || !output.tile_height.is_finite()
            || output.tile_width <= 0.0
            || output.tile_height <= 0.0
        {
            panic!("pattern shader geometry is invalid");
        }
        let Some(resources) = resources else {
            panic!("pattern shader has no PaintResources");
        };
        let found = resources
            .images
            .iter()
            .find(|image| image.id == output.resource_id);
        if found.is_none_or(|image| {
            image.width == 0
                || image.height == 0
                || !image.resolution_scale.is_finite()
                || image.resolution_scale <= 0.0
                || image.BitmapPixels().is_some_and(|pixels| {
                    pixels.len() != image.width as usize * image.height as usize * 4
                })
        }) {
            panic!("pattern shader image resource is invalid");
        }
        ResolvePoint(&mut output.tile_offset, origin, None);
        output.unit_coordinates = false;
        return output;
    }

    if output.stops.is_empty() {
        panic!("gradient shader requires at least one stop");
    }
    for stop in &output.stops {
        if !stop.offset.is_finite()
            || !stop.offset_length.is_finite()
            || stop.offset < 0.0
            || stop.offset > 1.0
            || stop.offset_length < 0.0
        {
            panic!("gradient color stop is invalid");
        }
        ValidateColor(stop.color);
    }

    if output.object_bounding_box_coordinates {
        let box_size = unit_box.expect("object-bounding-box shader requires paint bounds");
        let mut box_transform = TransformMatrix::default();
        box_transform.values[0] = box_size.width;
        box_transform.values[5] = box_size.height;
        box_transform.values[12] = origin.x;
        box_transform.values[13] = origin.y;
        output.transform = MultiplyTransforms(&box_transform, &output.transform);
        origin = Offset::default();
        unit_box = None;
    }

    match output.kind {
        PaintShaderKind::kLinearGradient => {
            if !Finite(output.start) || !Finite(output.end) {
                panic!("linear gradient geometry is invalid");
            }
            if let Some(linear_angle) = output.linear_angle {
                if unit_box.is_none()
                    || !linear_angle.is_finite()
                    || !output.linear_start_offset.is_finite()
                    || !output.linear_end_offset.is_finite()
                    || output.linear_end_offset <= output.linear_start_offset
                {
                    panic!("angled linear gradient geometry is invalid");
                }
                const K_PI: f32 = 3.14159265358979323846_f32;
                let mut angle = (linear_angle as f32) % 360.0_f32;
                if angle < 0.0 {
                    angle += 360.0;
                }
                let width = unit_box.unwrap().width as f32;
                let height = unit_box.unwrap().height as f32;
                let mut first_x = 0.0_f32;
                let mut first_y = 0.0_f32;
                let mut second_x = 0.0_f32;
                let mut second_y = 0.0_f32;
                if angle == 0.0 {
                    first_y = height;
                } else if angle == 90.0 {
                    second_x = width;
                } else if angle == 180.0 {
                    second_y = height;
                } else if angle == 270.0 {
                    first_x = width;
                } else {
                    let slope = ((90.0_f32 - angle) * (K_PI / 180.0_f32)).tan();
                    let perpendicular_slope = -1.0_f32 / slope;
                    let half_height = height / 2.0_f32;
                    let half_width = width / 2.0_f32;
                    let corner_x = if angle < 180.0 {
                        half_width
                    } else {
                        -half_width
                    };
                    let corner_y = if angle < 90.0 || angle >= 270.0 {
                        half_height
                    } else {
                        -half_height
                    };
                    let c = corner_y - perpendicular_slope * corner_x;
                    let end_x = c / (slope - perpendicular_slope);
                    let end_y = perpendicular_slope * end_x + c;
                    second_x = half_width + end_x;
                    second_y = half_height - end_y;
                    first_x = half_width - end_x;
                    first_y = half_height + end_y;
                }
                let delta_x = second_x - first_x;
                let delta_y = second_y - first_y;
                let start_offset = output.linear_start_offset as f32;
                let end_offset = output.linear_end_offset as f32;
                output.start = Offset {
                    x: origin.x + first_x as f64 + (delta_x * start_offset) as f64,
                    y: origin.y + first_y as f64 + (delta_y * start_offset) as f64,
                };
                output.end = Offset {
                    x: origin.x + first_x as f64 + (delta_x * end_offset) as f64,
                    y: origin.y + first_y as f64 + (delta_y * end_offset) as f64,
                };
                output.linear_angle = None;
            } else {
                ResolvePoint(&mut output.start, origin, unit_box);
                ResolvePoint(&mut output.end, origin, unit_box);
            }
            let basis = (output.end.x - output.start.x).hypot(output.end.y - output.start.y);
            ResolvePhysicalColorStops(&mut output, basis);
        }
        PaintShaderKind::kRadialGradient => {
            let input_radius_y = output.radius_y.unwrap_or(output.radius);
            if !Finite(output.center)
                || !Finite(output.focal)
                || !Finite(output.center_offset)
                || !output.radius.is_finite()
                || !input_radius_y.is_finite()
                || !output.radius_offset.is_finite()
                || !output.radius_y_offset.is_finite()
                || !output.focal_radius.is_finite()
                || output.radius < 0.0
                || input_radius_y < 0.0
                || output.radius_offset < 0.0
                || output.radius_y_offset < 0.0
                || output.focal_radius < 0.0
            {
                panic!("radial gradient geometry is invalid");
            }
            ResolvePoint(&mut output.center, origin, unit_box);
            ResolvePoint(&mut output.focal, origin, unit_box);
            output.center.x += output.center_offset.x;
            output.center.y += output.center_offset.y;
            output.focal.x += output.center_offset.x;
            output.focal.y += output.center_offset.y;
            if output.radial_extent != PaintRadialExtent::kExplicit {
                let Some(unit_box) = unit_box else {
                    panic!("radial extent requires finite paint bounds");
                };
                let point_x = (output.center.x - origin.x) as f32;
                let point_y = (output.center.y - origin.y) as f32;
                let width = unit_box.width as f32;
                let height = unit_box.height as f32;
                let left = point_x.abs();
                let right = (point_x - width).abs();
                let top = point_y.abs();
                let bottom = (point_y - height).abs();
                let farthest = matches!(
                    output.radial_extent,
                    PaintRadialExtent::kFarthestSide | PaintRadialExtent::kFarthestCorner
                );
                let mut radius_x = if farthest {
                    left.max(right)
                } else {
                    left.min(right)
                };
                let mut radius_y = if farthest {
                    top.max(bottom)
                } else {
                    top.min(bottom)
                };
                let corner = matches!(
                    output.radial_extent,
                    PaintRadialExtent::kClosestCorner | PaintRadialExtent::kFarthestCorner
                );
                if corner {
                    let offsets = [
                        [-point_x, -point_y],
                        [width - point_x, -point_y],
                        [width - point_x, height - point_y],
                        [-point_x, height - point_y],
                    ];
                    let mut selected = 0;
                    let mut distance =
                        (offsets[0][0] * offsets[0][0] + offsets[0][1] * offsets[0][1]).sqrt();
                    for i in 1..offsets.len() {
                        let next =
                            (offsets[i][0] * offsets[i][0] + offsets[i][1] * offsets[i][1]).sqrt();
                        if if farthest {
                            next > distance
                        } else {
                            next < distance
                        } {
                            selected = i;
                            distance = next;
                        }
                    }
                    if output.radial_shape == PaintRadialShape::kCircle {
                        radius_y = distance;
                        radius_x = radius_y;
                    } else {
                        let aspect = radius_x / radius_y;
                        if !aspect.is_finite() || aspect == 0.0 {
                            radius_y = 0.0;
                            radius_x = radius_y;
                        } else {
                            let x = offsets[selected][0];
                            let y = offsets[selected][1];
                            radius_x = (x * x + y * y * aspect * aspect).sqrt();
                            radius_y = radius_x / aspect;
                        }
                    }
                } else if output.radial_shape == PaintRadialShape::kCircle {
                    radius_x = if farthest {
                        radius_x.max(radius_y)
                    } else {
                        radius_x.min(radius_y)
                    };
                    radius_y = radius_x;
                }
                output.radius = radius_x as f64;
                output.radius_y = Some(radius_y as f64);
            } else if let Some(unit_box) = unit_box {
                output.radius = output.radius * unit_box.width + output.radius_offset;
                output.radius_y = Some(if output.radial_shape == PaintRadialShape::kCircle {
                    output.radius
                } else {
                    input_radius_y * unit_box.height + output.radius_y_offset
                });
                output.focal_radius *= unit_box.width;
            } else {
                output.radius += output.radius_offset;
                output.radius_y = Some(if output.radial_shape == PaintRadialShape::kCircle {
                    output.radius
                } else {
                    input_radius_y + output.radius_y_offset
                });
            }
            output.radial_extent = PaintRadialExtent::kExplicit;
            if output.radial_start_offset != 0.0 || output.radial_end_offset != 1.0 {
                if !output.radial_start_offset.is_finite()
                    || !output.radial_end_offset.is_finite()
                    || output.radial_start_offset < 0.0
                    || output.radial_end_offset <= output.radial_start_offset
                {
                    panic!("repeating radial gradient period is invalid");
                }
                output.focal = output.center;
                output.focal_radius = output.radius * output.radial_start_offset;
                output.radius *= output.radial_end_offset;
                *output.radius_y.as_mut().unwrap() *= output.radial_end_offset;
                output.radial_start_offset = 0.0;
                output.radial_end_offset = 1.0;
            }
            if !output.radius.is_finite()
                || output.radius_y.is_none()
                || !output.radius_y.unwrap().is_finite()
                || output.radius < 0.0
                || output.radius_y.unwrap() < 0.0
                || output.focal_radius > output.radius
            {
                panic!("resolved radial gradient is invalid");
            }
            let basis = output.radius;
            ResolvePhysicalColorStops(&mut output, basis);
        }
        PaintShaderKind::kConicGradient => {
            if !Finite(output.center)
                || !Finite(output.center_offset)
                || !output.rotation_angle.is_finite()
                || !output.start_angle.is_finite()
                || !output.end_angle.is_finite()
                || output.end_angle <= output.start_angle
            {
                panic!("conic gradient geometry is invalid");
            }
            ResolvePoint(&mut output.center, origin, unit_box);
            output.center.x += output.center_offset.x;
            output.center.y += output.center_offset.y;
            ResolvePhysicalColorStops(&mut output, 1.0);
        }
        PaintShaderKind::kPattern => {}
    }
    let mut previous = -1.0;
    for stop in &output.stops {
        if stop.offset < previous {
            panic!("gradient stops must be ordered");
        }
        previous = stop.offset;
    }
    output.unit_coordinates = false;
    output.object_bounding_box_coordinates = false;
    output
}
