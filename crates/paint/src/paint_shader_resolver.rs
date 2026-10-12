#![allow(non_snake_case)]

use layoutng_assembly::fragment_tree::PaintResources;
use layoutng_assembly::internal::layout_input::{Offset, PaintPathVerb, Size, TransformMatrix};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    PaintColorInterpolationSpace, PaintColorStop, PaintHueInterpolationMethod, PaintRadialExtent,
    PaintRadialShape, PaintShader, PaintShaderKind,
};

use crate::geometry_mapper::{InvertTransform, MultiplyTransforms, TranslationTransform};

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

fn ScaleTransform(x: f64, y: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    output.values[0] = x;
    output.values[5] = y;
    output
}

fn PatternViewBoxTransform(
    view_box: &layoutng_assembly::internal::layout_input::SvgViewBoxData,
    width: f64,
    height: f64,
) -> TransformMatrix {
    let mut scale_x = width / view_box.width;
    let mut scale_y = height / view_box.height;
    let mut x = 0.0;
    let mut y = 0.0;
    if !view_box.preserve_none {
        let scale = if view_box.slice {
            scale_x.max(scale_y)
        } else {
            scale_x.min(scale_y)
        };
        scale_x = scale;
        scale_y = scale;
        let extra_x = width - view_box.width * scale;
        let extra_y = height - view_box.height * scale;
        x = match view_box.align_x {
            -1 => 0.0,
            1 => extra_x,
            _ => extra_x * 0.5,
        };
        y = match view_box.align_y {
            -1 => 0.0,
            1 => extra_y,
            _ => extra_y * 0.5,
        };
    }
    MultiplyTransforms(
        &TranslationTransform(x, y),
        &MultiplyTransforms(
            &ScaleTransform(scale_x, scale_y),
            &TranslationTransform(-view_box.x, -view_box.y),
        ),
    )
}

fn ResolvePatternImage(
    image: &mut layoutng_assembly::internal::paint_input::SvgPatternImage,
    resources: Option<&PaintResources>,
) {
    let resources = resources.expect("pattern image has no PaintResources");
    let resource = resources
        .images
        .iter()
        .find(|resource| resource.id == image.resource_id)
        .expect("pattern image resource is missing");
    if resource.width == 0
        || resource.height == 0
        || !resource.resolution_scale.is_finite()
        || resource.resolution_scale <= 0.0
        || resource.BitmapPixels().is_some_and(|pixels| {
            pixels.len() != resource.width as usize * resource.height as usize * 4
        })
    {
        panic!("pattern image resource is invalid");
    }
    let intrinsic_width = f64::from(resource.width);
    let intrinsic_height = f64::from(resource.height);
    // LayoutSVGImage configures an SVG document image from the authored
    // image viewport. preserveAspectRatio may inset the sampled pixels inside
    // that viewport, but it does not change the nested document's viewport.
    let container_width = image.destination_width;
    let container_height = image.destination_height;
    image.source_offset = Offset::default();
    image.source_width = intrinsic_width;
    image.source_height = intrinsic_height;
    if !image.preserve_none {
        let scale_x = image.destination_width / intrinsic_width;
        let scale_y = image.destination_height / intrinsic_height;
        if image.slice {
            let scale = scale_x.max(scale_y);
            let source_width = image.destination_width / scale;
            let source_height = image.destination_height / scale;
            image.source_offset.x = match image.align_x {
                -1 => 0.0,
                1 => intrinsic_width - source_width,
                _ => (intrinsic_width - source_width) * 0.5,
            };
            image.source_offset.y = match image.align_y {
                -1 => 0.0,
                1 => intrinsic_height - source_height,
                _ => (intrinsic_height - source_height) * 0.5,
            };
            image.source_width = source_width;
            image.source_height = source_height;
        } else {
            let scale = scale_x.min(scale_y);
            let fitted_width = intrinsic_width * scale;
            let fitted_height = intrinsic_height * scale;
            image.destination_offset.x += match image.align_x {
                -1 => 0.0,
                1 => image.destination_width - fitted_width,
                _ => (image.destination_width - fitted_width) * 0.5,
            };
            image.destination_offset.y += match image.align_y {
                -1 => 0.0,
                1 => image.destination_height - fitted_height,
                _ => (image.destination_height - fitted_height) * 0.5,
            };
            image.destination_width = fitted_width;
            image.destination_height = fitted_height;
        }
    }
    if let image_resource::PaintImageContent::Document(content) = &resource.content {
        let zoom = resources.layout_zoom_factor.max(f64::MIN_POSITIVE);
        let mut key = image_resource::ContainerKey::new(
            container_width.round().max(1.0) as u32,
            container_height.round().max(1.0) as u32,
            zoom,
            resources.device_pixel_ratio,
        );
        if !image.color_scheme_flags_is_normal {
            key.preferred_color_scheme = u8::from(image.used_dark_color_scheme);
            key.preferred_color_scheme_is_explicit = true;
        }
        if let Some(current) = &resource.document_container_key {
            key.view_specification = current.view_specification.clone();
        } else if let Some(initial) = content.variants().first() {
            key.view_specification = initial.container_key.view_specification.clone();
        }
        image.document_container_key = Some(key);
    }
}

fn PatternOutlineBounds(
    commands: &[layoutng_assembly::internal::layout_input::PaintPathCommand],
) -> Option<(Offset, Size)> {
    let mut bounds: Option<(f64, f64, f64, f64)> = None;
    let mut include = |point: Offset| {
        if !Finite(point) {
            return;
        }
        bounds = Some(bounds.map_or(
            (point.x, point.y, point.x, point.y),
            |(left, top, right, bottom)| {
                (
                    left.min(point.x),
                    top.min(point.y),
                    right.max(point.x),
                    bottom.max(point.y),
                )
            },
        ));
    };
    for command in commands {
        if command.verb == PaintPathVerb::kClose {
            continue;
        }
        include(command.point);
        if matches!(
            command.verb,
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo | PaintPathVerb::kCubicTo
        ) {
            include(command.control1);
        }
        if command.verb == PaintPathVerb::kCubicTo {
            include(command.control2);
        }
    }
    bounds.map(|(left, top, right, bottom)| {
        (
            Offset { x: left, y: top },
            Size {
                width: right - left,
                height: bottom - top,
            },
        )
    })
}

fn ResolvePatternPrimitive(
    mut primitive: layoutng_assembly::internal::paint_input::SvgMarkerPrimitive,
    resources: &PaintResources,
) -> layoutng_assembly::internal::paint_input::SvgMarkerPrimitive {
    let bounds_origin = primitive.shape.bounds_offset;
    let bounds_size = Size {
        width: primitive.shape.bounds_width,
        height: primitive.shape.bounds_height,
    };
    if let Some(shader) = primitive.fill_server.as_mut() {
        *shader = ResolvePaintShaderForBounds(
            shader,
            Some(resources),
            Offset::default(),
            bounds_origin,
            Some(bounds_size),
        );
    }
    if let Some(shader) = primitive.stroke_server.as_mut() {
        *shader = ResolvePaintShaderForBounds(
            shader,
            Some(resources),
            Offset::default(),
            bounds_origin,
            Some(bounds_size),
        );
    }
    primitive
}

fn ResolvePatternCommands(
    commands: Vec<layoutng_assembly::internal::paint_input::SvgPatternCommand>,
    resources: Option<&PaintResources>,
    geometry_size: (f64, f64),
) -> Vec<layoutng_assembly::internal::paint_input::SvgPatternCommand> {
    use layoutng_assembly::internal::paint_input::SvgPatternCommand;
    let resources = resources.expect("SVG pattern content has no PaintResources");
    let mut cursor = Offset::default();
    let mut output = Vec::with_capacity(commands.len());
    for command in commands {
        match command {
            SvgPatternCommand::DrawText(mut text) => {
                if text.reset_cursor {
                    cursor = Offset::default();
                }
                let paths = crate::pre_paint_tree_walk::ResolveSvgClipText(
                    &text.text,
                    &resources.fonts,
                    geometry_size,
                    &mut cursor,
                )
                .unwrap_or_else(|error| panic!("unsupported SVG pattern text: {error}"));
                let commands = paths.into_iter().flatten().collect::<Vec<_>>();
                let Some((origin, size)) = PatternOutlineBounds(&commands) else {
                    continue;
                };
                text.paint.shape.geometry =
                    layoutng_assembly::internal::layout_input::SvgShapeGeometry::kPath;
                text.paint.shape.bounds_offset = origin;
                text.paint.shape.bounds_width = size.width;
                text.paint.shape.bounds_height = size.height;
                text.paint.shape.path = commands;
                if let Some(shader) = text.paint.fill_server.as_mut() {
                    *shader = ResolvePaintShaderForBounds(
                        shader,
                        Some(resources),
                        Offset::default(),
                        origin,
                        Some(size),
                    );
                }
                if let Some(shader) = text.paint.stroke_server.as_mut() {
                    *shader = ResolvePaintShaderForBounds(
                        shader,
                        Some(resources),
                        Offset::default(),
                        origin,
                        Some(size),
                    );
                }
                output.push(SvgPatternCommand::DrawGlyph(text.paint));
            }
            SvgPatternCommand::DrawShape(primitive) => output.push(SvgPatternCommand::DrawShape(
                ResolvePatternPrimitive(primitive, resources),
            )),
            SvgPatternCommand::DrawGlyph(primitive) => output.push(SvgPatternCommand::DrawGlyph(
                ResolvePatternPrimitive(primitive, resources),
            )),
            SvgPatternCommand::DrawImage(mut image) => {
                ResolvePatternImage(&mut image, Some(resources));
                output.push(SvgPatternCommand::DrawImage(image));
            }
            SvgPatternCommand::SaveLayerMask(mask) => {
                let mut mask = mask.as_ref().clone();
                if let Some(reason) = &mask.unsupported {
                    panic!("unsupported SVG pattern mask: {reason}");
                }
                let mask_geometry = if mask.content_units
                    == layoutng_assembly::internal::paint_input::SvgMaskUnits::kObjectBoundingBox
                {
                    (1.0, 1.0)
                } else {
                    geometry_size
                };
                mask.commands =
                    ResolvePatternCommands(mask.commands, Some(resources), mask_geometry);
                output.push(SvgPatternCommand::SaveLayerMask(std::sync::Arc::new(mask)));
            }
            other => output.push(other),
        }
    }
    ResolvePatternReferenceFilters(&mut output);
    ResolvePatternClips(&mut output, geometry_size, resources);
    ResolvePatternMaskGeometry(&mut output, geometry_size);
    output
}

fn PatternPoint(transform: &TransformMatrix, x: f64, y: f64) -> (f64, f64) {
    (
        transform.values[0] * x + transform.values[4] * y + transform.values[12],
        transform.values[1] * x + transform.values[5] * y + transform.values[13],
    )
}

fn PatternRectBounds(
    transform: &TransformMatrix,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Option<[f64; 4]> {
    if ![x, y, width, height].iter().all(|value| value.is_finite()) {
        return None;
    }
    let points = [
        PatternPoint(transform, x, y),
        PatternPoint(transform, x + width, y),
        PatternPoint(transform, x, y + height),
        PatternPoint(transform, x + width, y + height),
    ];
    let (mut left, mut top) = points[0];
    let (mut right, mut bottom) = points[0];
    for (x, y) in points.into_iter().skip(1) {
        left = left.min(x);
        top = top.min(y);
        right = right.max(x);
        bottom = bottom.max(y);
    }
    Some([left, top, right - left, bottom - top])
}

fn PatternCommandBounds(
    commands: &[layoutng_assembly::internal::paint_input::SvgPatternCommand],
) -> Option<[f64; 4]> {
    use layoutng_assembly::internal::paint_input::SvgPatternCommand;
    let mut result: Option<[f64; 4]> = None;
    for command in commands {
        let bounds = match command {
            SvgPatternCommand::DrawShape(shape) | SvgPatternCommand::DrawGlyph(shape) => {
                PatternRectBounds(
                    shape
                        .transform
                        .as_ref()
                        .unwrap_or(&TransformMatrix::default()),
                    shape.shape.bounds_offset.x,
                    shape.shape.bounds_offset.y,
                    shape.shape.bounds_width,
                    shape.shape.bounds_height,
                )
            }
            SvgPatternCommand::DrawImage(image) => PatternRectBounds(
                &image.transform,
                image.destination_offset.x,
                image.destination_offset.y,
                image.destination_width,
                image.destination_height,
            ),
            _ => None,
        };
        if let Some([x, y, width, height]) = bounds {
            result = Some(match result {
                None => [x, y, width, height],
                Some([old_x, old_y, old_width, old_height]) => {
                    let left = old_x.min(x);
                    let top = old_y.min(y);
                    let right = (old_x + old_width).max(x + width);
                    let bottom = (old_y + old_height).max(y + height);
                    [left, top, right - left, bottom - top]
                }
            });
        }
    }
    result
}

fn ResolvePatternMaskGeometry(
    commands: &mut [layoutng_assembly::internal::paint_input::SvgPatternCommand],
    geometry_size: (f64, f64),
) {
    use layoutng_assembly::internal::paint_input::{SvgMaskUnits, SvgPatternCommand};
    let mut index = 0;
    while index < commands.len() {
        let SvgPatternCommand::SaveLayerMask(mask) = &commands[index] else {
            index += 1;
            continue;
        };
        let mut depth = 1usize;
        let mut end = index + 1;
        while end < commands.len() {
            match &commands[end] {
                SvgPatternCommand::SaveLayerOpacity(_)
                | SvgPatternCommand::SaveLayerFilter(_)
                | SvgPatternCommand::SaveLayerReferenceFilter(_)
                | SvgPatternCommand::SaveLayerBlend(_)
                | SvgPatternCommand::SaveLayerClip(_)
                | SvgPatternCommand::SaveLayerMask(_) => depth += 1,
                SvgPatternCommand::Restore => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            end += 1;
        }
        if end == commands.len() {
            panic!("unbalanced SVG pattern mask layer");
        }
        let object = PatternCommandBounds(&commands[index + 1..end]).unwrap_or([0.0; 4]);
        let mut resolved = mask.as_ref().clone();
        let axis = |length: layoutng_assembly::internal::paint_input::SvgClipLength,
                    origin: f64,
                    extent: f64,
                    units: SvgMaskUnits| {
            if units == SvgMaskUnits::kObjectBoundingBox {
                origin + (length.value + length.percentage) * extent
            } else {
                length.value + length.percentage * extent
            }
        };
        let (origin_x, origin_y, basis_width, basis_height) =
            if resolved.mask_units == SvgMaskUnits::kObjectBoundingBox {
                (object[0], object[1], object[2], object[3])
            } else {
                (0.0, 0.0, geometry_size.0, geometry_size.1)
            };
        let x = axis(resolved.x, origin_x, basis_width, resolved.mask_units);
        let y = axis(resolved.y, origin_y, basis_height, resolved.mask_units);
        let width = axis(resolved.width, 0.0, basis_width, resolved.mask_units);
        let height = axis(resolved.height, 0.0, basis_height, resolved.mask_units);
        resolved.resolved_clip = Some([x, y, width.max(0.0), height.max(0.0)]);
        resolved.content_transform = if resolved.content_units == SvgMaskUnits::kObjectBoundingBox {
            let mut transform = ScaleTransform(object[2], object[3]);
            transform.values[12] = object[0];
            transform.values[13] = object[1];
            transform
        } else {
            TransformMatrix::default()
        };
        commands[index] = SvgPatternCommand::SaveLayerMask(std::sync::Arc::new(resolved));
        index += 1;
    }
}

fn ResolvePatternClips(
    commands: &mut [layoutng_assembly::internal::paint_input::SvgPatternCommand],
    _geometry_size: (f64, f64),
    resources: &PaintResources,
) {
    use layoutng_assembly::internal::paint_input::{
        PaintMaskMode, SvgClipLength, SvgMaskGraph, SvgMaskUnits, SvgPatternCommand,
    };
    let mut index = 0;
    while index < commands.len() {
        let SvgPatternCommand::SaveLayerClip(graph) = &commands[index] else {
            index += 1;
            continue;
        };
        if let Some(reason) = &graph.unsupported {
            panic!("unsupported SVG pattern clipPath: {reason}");
        }
        let mut depth = 1usize;
        let mut end = index + 1;
        while end < commands.len() {
            match &commands[end] {
                SvgPatternCommand::SaveLayerOpacity(_)
                | SvgPatternCommand::SaveLayerFilter(_)
                | SvgPatternCommand::SaveLayerReferenceFilter(_)
                | SvgPatternCommand::SaveLayerBlend(_)
                | SvgPatternCommand::SaveLayerClip(_)
                | SvgPatternCommand::SaveLayerMask(_) => depth += 1,
                SvgPatternCommand::Restore => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            end += 1;
        }
        if end == commands.len() {
            panic!("unbalanced SVG pattern clip layer");
        }
        let [x, y, width, height] =
            PatternCommandBounds(&commands[index + 1..end]).unwrap_or([0.0; 4]);
        let reference = crate::PaintRect {
            x,
            y,
            width,
            height,
        };
        let mut runtime_error = None;
        let mut flattened = Vec::new();
        let mut children = Vec::new();
        let clip_bounds = crate::pre_paint_tree_walk::ResolveReferenceClipPath(
            graph,
            reference,
            Offset::default(),
            1.0,
            true,
            &resources.fonts,
            &mut runtime_error,
            &mut flattened,
            &mut children,
        );
        if let Some(error) = runtime_error {
            panic!("unsupported SVG pattern clipPath: {error}");
        }
        let (clip_x, clip_y, clip_width, clip_height) = (
            clip_bounds.x,
            clip_bounds.y,
            clip_bounds.width,
            clip_bounds.height,
        );
        let white = Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        };
        let mask_commands = children
            .into_iter()
            .filter(|child| !child.commands.is_empty())
            .map(|child| {
                let (origin, size) = PatternOutlineBounds(&child.commands)
                    .unwrap_or((Offset::default(), Size::default()));
                let mut shape = layoutng_assembly::internal::layout_input::SvgShapeData::default();
                shape.geometry = layoutng_assembly::internal::layout_input::SvgShapeGeometry::kPath;
                shape.bounds_offset = origin;
                shape.bounds_width = size.width;
                shape.bounds_height = size.height;
                shape.path = child.commands;
                SvgPatternCommand::DrawShape(
                    layoutng_assembly::internal::paint_input::SvgMarkerPrimitive {
                        shape,
                        fill: Some(white),
                        fill_even_odd: child.even_odd,
                        transform: Some(TransformMatrix::default()),
                        ..Default::default()
                    },
                )
            })
            .collect();
        let length = |value| SvgClipLength {
            value,
            percentage: 0.0,
        };
        commands[index] = SvgPatternCommand::SaveLayerMask(std::sync::Arc::new(SvgMaskGraph {
            id: graph.id.clone(),
            mask_units: SvgMaskUnits::kUserSpaceOnUse,
            content_units: SvgMaskUnits::kUserSpaceOnUse,
            x: length(clip_x),
            y: length(clip_y),
            width: length(clip_width),
            height: length(clip_height),
            mode: PaintMaskMode::kAlpha,
            commands: mask_commands,
            content_transform: TransformMatrix::default(),
            resolved_clip: None,
            unsupported: None,
        }));
        index += 1;
    }
}

fn ResolvePatternReferenceFilters(
    commands: &mut [layoutng_assembly::internal::paint_input::SvgPatternCommand],
) {
    use layoutng_assembly::internal::paint_input::SvgPatternCommand;
    let mut index = 0;
    while index < commands.len() {
        let SvgPatternCommand::SaveLayerReferenceFilter(filter) = &commands[index] else {
            index += 1;
            continue;
        };
        let mut depth = 1usize;
        let mut end = index + 1;
        while end < commands.len() {
            match &commands[end] {
                SvgPatternCommand::SaveLayerOpacity(_)
                | SvgPatternCommand::SaveLayerFilter(_)
                | SvgPatternCommand::SaveLayerReferenceFilter(_)
                | SvgPatternCommand::SaveLayerBlend(_)
                | SvgPatternCommand::SaveLayerClip(_)
                | SvgPatternCommand::SaveLayerMask(_) => depth += 1,
                SvgPatternCommand::Restore => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            end += 1;
        }
        if end == commands.len() {
            panic!("unbalanced SVG pattern reference-filter layer");
        }
        let mut resolved = filter.as_ref().clone();
        resolved.reference =
            Some(PatternCommandBounds(&commands[index + 1..end]).unwrap_or([0.0, 0.0, 1.0, 1.0]));
        commands[index] =
            SvgPatternCommand::SaveLayerReferenceFilter(std::sync::Arc::new(resolved));
        index += 1;
    }
}

fn FinalizePatternMasks(
    commands: &mut [layoutng_assembly::internal::paint_input::SvgPatternCommand],
    owner_to_tile: &TransformMatrix,
) {
    use layoutng_assembly::internal::paint_input::SvgPatternCommand;
    for command in commands {
        let SvgPatternCommand::SaveLayerMask(mask) = command else {
            continue;
        };
        let mut resolved = mask.as_ref().clone();
        let [x, y, width, height] = resolved
            .resolved_clip
            .expect("SVG pattern mask geometry was not resolved");
        resolved.resolved_clip = PatternRectBounds(owner_to_tile, x, y, width, height);
        resolved.content_transform = MultiplyTransforms(owner_to_tile, &resolved.content_transform);
        FinalizePatternMasks(&mut resolved.commands, &resolved.content_transform);
        *mask = std::sync::Arc::new(resolved);
    }
}

// Resolve length-percentage positions only after the concrete gradient basis is
// known. CSS allows negative and >100% positions; normalization below adjusts
// the gradient geometry rather than clipping those values.
fn ResolvePhysicalColorStops(shader: &mut PaintShader, basis: f64) {
    if !basis.is_finite() || basis < 0.0 {
        panic!("gradient color-stop basis is invalid");
    }
    let mut previous = f64::NEG_INFINITY;
    for stop in &mut shader.stops {
        if !stop.offset.is_finite() || !stop.offset_length.is_finite() {
            panic!("gradient color-stop position is invalid");
        }
        if stop.offset_length != 0.0 {
            if basis == 0.0 {
                stop.offset = 0.0;
            } else {
                stop.offset += stop.offset_length / basis;
            }
            stop.offset_length = 0.0;
        }
        stop.offset = stop.offset.max(previous);
        previous = stop.offset;
    }
}

fn FoundationSpace(space: PaintColorInterpolationSpace) -> foundation::ColorSpace {
    match space {
        PaintColorInterpolationSpace::kNone | PaintColorInterpolationSpace::kSRGB => {
            foundation::ColorSpace::kSRGB
        }
        PaintColorInterpolationSpace::kSRGBLegacy => foundation::ColorSpace::kSRGBLegacy,
        PaintColorInterpolationSpace::kSRGBLinear => foundation::ColorSpace::kSRGBLinear,
        PaintColorInterpolationSpace::kDisplayP3 => foundation::ColorSpace::kDisplayP3,
        PaintColorInterpolationSpace::kDisplayP3Linear => foundation::ColorSpace::kDisplayP3Linear,
        PaintColorInterpolationSpace::kHSL => foundation::ColorSpace::kHSL,
        PaintColorInterpolationSpace::kHWB => foundation::ColorSpace::kHWB,
        PaintColorInterpolationSpace::kA98RGB => foundation::ColorSpace::kA98RGB,
        PaintColorInterpolationSpace::kProPhotoRGB => foundation::ColorSpace::kProPhotoRGB,
        PaintColorInterpolationSpace::kRec2020 => foundation::ColorSpace::kRec2020,
        PaintColorInterpolationSpace::kRec2100Linear => foundation::ColorSpace::kRec2100Linear,
        PaintColorInterpolationSpace::kLab => foundation::ColorSpace::kLab,
        PaintColorInterpolationSpace::kLch => foundation::ColorSpace::kLch,
        PaintColorInterpolationSpace::kOklab => foundation::ColorSpace::kOklab,
        PaintColorInterpolationSpace::kOklch => foundation::ColorSpace::kOklch,
        PaintColorInterpolationSpace::kXYZD50 => foundation::ColorSpace::kXYZD50,
        PaintColorInterpolationSpace::kXYZD65 => foundation::ColorSpace::kXYZD65,
    }
}

fn SemanticColor(stop: &PaintColorStop) -> foundation::Color {
    if stop.source_color_space == PaintColorInterpolationSpace::kNone {
        return foundation::Color::FromColorSpace(
            foundation::ColorSpace::kSRGB,
            Some(stop.color.red),
            Some(stop.color.green),
            Some(stop.color.blue),
            Some(stop.color.alpha),
        );
    }
    let component = |index: usize| {
        (stop.missing_components & (1 << index) == 0).then_some(stop.source_components[index])
    };
    foundation::Color::FromColorSpace(
        FoundationSpace(stop.source_color_space),
        component(0),
        component(1),
        component(2),
        component(3),
    )
}

fn StopFromSemantic(offset: f64, value: foundation::Color) -> PaintColorStop {
    let (red, green, blue) = value.ToSRGB();
    let source_color_space = match value.GetColorSpace() {
        foundation::ColorSpace::kSRGB => PaintColorInterpolationSpace::kSRGB,
        foundation::ColorSpace::kSRGBLegacy => PaintColorInterpolationSpace::kSRGBLegacy,
        foundation::ColorSpace::kSRGBLinear => PaintColorInterpolationSpace::kSRGBLinear,
        foundation::ColorSpace::kDisplayP3 => PaintColorInterpolationSpace::kDisplayP3,
        foundation::ColorSpace::kDisplayP3Linear => PaintColorInterpolationSpace::kDisplayP3Linear,
        foundation::ColorSpace::kHSL => PaintColorInterpolationSpace::kHSL,
        foundation::ColorSpace::kHWB => PaintColorInterpolationSpace::kHWB,
        foundation::ColorSpace::kA98RGB => PaintColorInterpolationSpace::kA98RGB,
        foundation::ColorSpace::kProPhotoRGB => PaintColorInterpolationSpace::kProPhotoRGB,
        foundation::ColorSpace::kRec2020 => PaintColorInterpolationSpace::kRec2020,
        foundation::ColorSpace::kRec2100Linear => PaintColorInterpolationSpace::kRec2100Linear,
        foundation::ColorSpace::kLab => PaintColorInterpolationSpace::kLab,
        foundation::ColorSpace::kLch => PaintColorInterpolationSpace::kLch,
        foundation::ColorSpace::kOklab => PaintColorInterpolationSpace::kOklab,
        foundation::ColorSpace::kOklch => PaintColorInterpolationSpace::kOklch,
        foundation::ColorSpace::kXYZD50 => PaintColorInterpolationSpace::kXYZD50,
        foundation::ColorSpace::kXYZD65 => PaintColorInterpolationSpace::kXYZD65,
        foundation::ColorSpace::kNone => PaintColorInterpolationSpace::kNone,
    };
    PaintColorStop {
        offset,
        color: Color {
            red,
            green,
            blue,
            alpha: value.Alpha(),
        },
        source_color_space,
        source_components: [
            value.Param0(),
            value.Param1(),
            value.Param2(),
            value.Alpha(),
        ],
        missing_components: u8::from(value.Param0IsNone())
            | (u8::from(value.Param1IsNone()) << 1)
            | (u8::from(value.Param2IsNone()) << 2)
            | (u8::from(value.AlphaIsNone()) << 3),
        offset_length: 0.0,
        is_hint: false,
    }
}

fn InterpolateStop(
    left: &PaintColorStop,
    right: &PaintColorStop,
    amount: f64,
    space: PaintColorInterpolationSpace,
    hue: PaintHueInterpolationMethod,
) -> PaintColorStop {
    if space != PaintColorInterpolationSpace::kNone {
        let space = FoundationSpace(space);
        let hue = match hue {
            PaintHueInterpolationMethod::kShorter => foundation::HueInterpolationMethod::kShorter,
            PaintHueInterpolationMethod::kLonger => foundation::HueInterpolationMethod::kLonger,
            PaintHueInterpolationMethod::kIncreasing => {
                foundation::HueInterpolationMethod::kIncreasing
            }
            PaintHueInterpolationMethod::kDecreasing => {
                foundation::HueInterpolationMethod::kDecreasing
            }
        };
        let result = foundation::Color::InterpolateColors(
            space,
            hue,
            SemanticColor(left),
            SemanticColor(right),
            amount.clamp(0.0, 1.0) as f32,
        );
        return StopFromSemantic(0.0, result);
    }
    let amount = amount.clamp(0.0, 1.0) as f32;
    let inverse = 1.0 - amount;
    let alpha = left.color.alpha * inverse + right.color.alpha * amount;
    let channel = |a: f32, aa: f32, b: f32, ba: f32| {
        if alpha == 0.0 {
            0.0
        } else {
            (a * aa * inverse + b * ba * amount) / alpha
        }
    };
    PaintColorStop {
        color: Color {
            red: channel(
                left.color.red,
                left.color.alpha,
                right.color.red,
                right.color.alpha,
            ),
            green: channel(
                left.color.green,
                left.color.alpha,
                right.color.green,
                right.color.alpha,
            ),
            blue: channel(
                left.color.blue,
                left.color.alpha,
                right.color.blue,
                right.color.alpha,
            ),
            alpha,
        },
        ..Default::default()
    }
}

// css_gradient_value.cc ReplaceColorHintsWithColorStops. The backend has no
// native hint primitive, so each hint becomes Chromium's nine-stop
// approximation after its target-size-dependent offset is known.
fn ReplaceColorHints(shader: &mut PaintShader) {
    let mut index = 1;
    while index + 1 < shader.stops.len() {
        if !shader.stops[index].is_hint {
            index += 1;
            continue;
        }
        let left = shader.stops[index - 1];
        let hint = shader.stops[index];
        let right = shader.stops[index + 1];
        let left_distance = hint.offset - left.offset;
        let right_distance = right.offset - hint.offset;
        let total = right.offset - left.offset;
        let near = |a: f64, b: f64| (a - b).abs() <= f32::EPSILON as f64;
        if near(left_distance, right_distance) {
            shader.stops.remove(index);
            continue;
        }
        if near(left_distance, 0.0) || near(right_distance, 0.0) || total <= 0.0 {
            shader.stops[index] = if near(left_distance, 0.0) {
                right
            } else {
                left
            };
            shader.stops[index].offset = hint.offset;
            shader.stops[index].offset_length = 0.0;
            shader.stops[index].is_hint = false;
            index += 1;
            continue;
        }
        let mut offsets = [0.0; 9];
        if left_distance > right_distance {
            for y in 0..7 {
                offsets[y] = left.offset + left_distance * ((7 + y) as f64 / 13.0);
            }
            offsets[7] = hint.offset + right_distance / 3.0;
            offsets[8] = hint.offset + right_distance * 2.0 / 3.0;
        } else {
            offsets[0] = left.offset + left_distance / 3.0;
            offsets[1] = left.offset + left_distance * 2.0 / 3.0;
            for y in 0..7 {
                offsets[y + 2] = hint.offset + right_distance * (y as f64 / 13.0);
            }
        }
        let hint_relative = left_distance / total;
        let exponent = 0.5_f64.ln() / hint_relative.ln();
        let replacements = offsets.into_iter().map(|offset| {
            let point = (offset - left.offset) / total;
            let weight = point.powf(exponent);
            let mut stop = InterpolateStop(
                &left,
                &right,
                weight,
                shader.color_interpolation_space,
                shader.hue_interpolation_method,
            );
            stop.offset = offset;
            stop
        });
        shader.stops.splice(index..=index, replacements);
        index += 9;
    }
    if shader.stops.iter().any(|stop| stop.is_hint) {
        panic!("gradient color hint is not between color stops");
    }
}

fn NormalizeStops(shader: &mut PaintShader) -> Option<(f64, f64)> {
    let first = shader.stops.first()?.offset;
    let last = shader.stops.last()?.offset;
    let normalize = shader.spread
        == layoutng_assembly::internal::paint_input::PaintSpreadMethod::kRepeat
        || first < 0.0
        || last > 1.0;
    if !normalize {
        return None;
    }
    let span = (last - first).max(0.0);
    if span < f64::EPSILON {
        let offset = first.clamp(0.0, 1.0);
        let first_color = *shader.stops.first().unwrap();
        let last_color = *shader.stops.last().unwrap();
        shader.stops.clear();
        if shader.spread != layoutng_assembly::internal::paint_input::PaintSpreadMethod::kRepeat {
            let mut first_color = first_color;
            first_color.offset = offset;
            first_color.offset_length = 0.0;
            first_color.is_hint = false;
            shader.stops.push(first_color);
        }
        let mut last_color = last_color;
        last_color.offset = offset;
        last_color.offset_length = 0.0;
        last_color.is_hint = false;
        shader.stops.push(last_color);
        shader.spread = layoutng_assembly::internal::paint_input::PaintSpreadMethod::kPad;
        return None;
    }
    for stop in &mut shader.stops {
        stop.offset = ((stop.offset - first) / span).clamp(0.0, 1.0);
    }
    Some((first, last))
}

fn ClampNegativeRadialStops(shader: &mut PaintShader) {
    let mut last_negative: Option<(usize, f64)> = None;
    for index in 0..shader.stops.len() {
        let offset = shader.stops[index].offset;
        if offset >= 0.0 {
            if let Some((previous, previous_offset)) = last_negative {
                let amount = -previous_offset / (offset - previous_offset);
                let mut interpolated = InterpolateStop(
                    &shader.stops[previous],
                    &shader.stops[index],
                    amount,
                    shader.color_interpolation_space,
                    shader.hue_interpolation_method,
                );
                interpolated.offset = shader.stops[previous].offset;
                shader.stops[previous] = interpolated;
            }
            break;
        }
        shader.stops[index].offset = 0.0;
        last_negative = Some((index, offset));
    }
}

// cpp: paint/paint_shader_resolver.h:10-17
// cpp: paint/paint_shader_resolver.cc:64-340
pub fn ResolvePaintShader(
    input: &PaintShader,
    resources: Option<&PaintResources>,
    origin: Offset,
    unit_box: Option<Size>,
) -> PaintShader {
    ResolvePaintShaderInner(input, resources, origin, origin, origin, unit_box)
}

/// SVG paint servers need both coordinate origins: userSpaceOnUse starts at
/// the SVG viewport, while objectBoundingBox starts at the painted geometry.
/// Most CSS callers use one origin and continue through ResolvePaintShader.
pub fn ResolvePaintShaderForBounds(
    input: &PaintShader,
    resources: Option<&PaintResources>,
    user_origin: Offset,
    object_origin: Offset,
    unit_box: Option<Size>,
) -> PaintShader {
    let origin = if input.object_bounding_box_coordinates {
        object_origin
    } else {
        user_origin
    };
    ResolvePaintShaderInner(
        input,
        resources,
        origin,
        user_origin,
        object_origin,
        unit_box,
    )
}

fn ResolvePaintShaderInner(
    input: &PaintShader,
    resources: Option<&PaintResources>,
    mut origin: Offset,
    user_origin: Offset,
    object_origin: Offset,
    mut unit_box: Option<Size>,
) -> PaintShader {
    let mut output = input.clone();
    let target_box = unit_box;
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
        if !Finite(output.tile_offset)
            || !output.tile_width.is_finite()
            || !output.tile_height.is_finite()
            || output.tile_width <= 0.0
            || output.tile_height <= 0.0
        {
            panic!("pattern shader geometry is invalid");
        }
        if let Some(pattern) = output.pattern.as_ref() {
            let target_box = target_box.unwrap_or(Size {
                width: 0.0,
                height: 0.0,
            });
            let (units_x, units_y) = if output.unit_coordinates {
                (target_box.width, target_box.height)
            } else {
                (1.0, 1.0)
            };
            if units_x <= 0.0 || units_y <= 0.0 {
                panic!("object-bounding-box pattern requires nonempty paint bounds");
            }
            let units_to_user = if output.unit_coordinates {
                let mut matrix = ScaleTransform(units_x, units_y);
                matrix.values[12] = object_origin.x;
                matrix.values[13] = object_origin.y;
                matrix
            } else {
                TranslationTransform(user_origin.x, user_origin.y)
            };
            let tile_position = TranslationTransform(output.tile_offset.x, output.tile_offset.y);
            let local_to_pattern_units = ScaleTransform(1.0 / units_x, 1.0 / units_y);
            let tile_without_pattern_transform = MultiplyTransforms(
                &units_to_user,
                &MultiplyTransforms(&tile_position, &local_to_pattern_units),
            );
            let tile_to_user = MultiplyTransforms(
                &units_to_user,
                &MultiplyTransforms(
                    &output.transform,
                    &MultiplyTransforms(&tile_position, &local_to_pattern_units),
                ),
            );
            output.tile_width *= units_x;
            output.tile_height *= units_y;
            if output.tile_width <= 0.0 || output.tile_height <= 0.0 {
                panic!("resolved pattern tile is empty");
            }
            output.tile_offset = Offset {
                x: tile_without_pattern_transform.values[12],
                y: tile_without_pattern_transform.values[13],
            };
            output.transform = tile_to_user;

            let mut resolved_pattern = pattern.as_ref().clone();
            let content_to_user = if resolved_pattern.content_units_object_bounding_box {
                let mut matrix = ScaleTransform(target_box.width, target_box.height);
                matrix.values[12] = object_origin.x;
                matrix.values[13] = object_origin.y;
                matrix
            } else {
                TranslationTransform(user_origin.x, user_origin.y)
            };
            let pattern_geometry_size = resolved_pattern
                .view_box
                .as_ref()
                .map(|view_box| (view_box.width, view_box.height))
                .unwrap_or_else(|| {
                    if resolved_pattern.content_units_object_bounding_box {
                        (1.0, 1.0)
                    } else {
                        (output.tile_width, output.tile_height)
                    }
                });
            resolved_pattern.content_transform = if let Some(view_box) = &resolved_pattern.view_box
            {
                PatternViewBoxTransform(view_box, output.tile_width, output.tile_height)
            } else {
                let inverse_tile = InvertTransform(&tile_without_pattern_transform)
                    .expect("pattern tile transform must be invertible");
                MultiplyTransforms(&inverse_tile, &content_to_user)
            };
            resolved_pattern.view_box = None;
            resolved_pattern.content_units_object_bounding_box = false;
            resolved_pattern.commands = ResolvePatternCommands(
                std::mem::take(&mut resolved_pattern.commands),
                resources,
                pattern_geometry_size,
            );
            FinalizePatternMasks(
                &mut resolved_pattern.commands,
                &resolved_pattern.content_transform,
            );
            output.pattern = Some(std::sync::Arc::new(resolved_pattern));
        } else {
            if output.resource_id == 0 {
                panic!("bitmap pattern has no resource id");
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
        }
        output.unit_coordinates = false;
        output.object_bounding_box_coordinates = false;
        return output;
    }

    if output.stops.is_empty() {
        panic!("gradient shader requires at least one stop");
    }
    for stop in &output.stops {
        if !stop.offset.is_finite() || !stop.offset_length.is_finite() {
            panic!("gradient color stop is invalid");
        }
        if !stop.is_hint {
            ValidateColor(stop.color);
        }
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
            if let Some([x, y]) = output.linear_corner.take() {
                let size = unit_box.expect("corner linear gradient requires paint bounds");
                let rise = if y > 0 { -size.width } else { size.width };
                let run = if x < 0 { -size.height } else { size.height };
                output.linear_angle = Some(90.0 - rise.atan2(run).to_degrees());
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
                    // Clang contracts Chromium's corresponding `a * b + c`
                    // expression on the supported release targets. Rust does
                    // not implicitly contract it, so spell out the same
                    // single-rounding operation.
                    let end_y = perpendicular_slope.mul_add(end_x, c);
                    second_x = half_width + end_x;
                    second_y = half_height - end_y;
                    first_x = half_width - end_x;
                    first_y = half_height + end_y;
                }
                let delta_x = second_x - first_x;
                let delta_y = second_y - first_y;
                let start_offset = output.linear_start_offset as f32;
                let end_offset = output.linear_end_offset as f32;
                // CSSLinearGradientValue::CreateGradient passes the two
                // EndPointsFromAngle results to Gradient::CreateLinear
                // directly.  Preserve those f32 results at the ordinary
                // 0..1 endpoints.  Reconstructing `second` as
                // `first + (second - first) * 1` introduces cancellation and
                // moves a long gradient by an ulp, which changes Skia's
                // dither/UNORM decisions over thousands of pixels.
                let point_at = |offset: f32, first: f32, second: f32, delta: f32| {
                    if offset == 0.0 {
                        first
                    } else if offset == 1.0 {
                        second
                    } else {
                        first + delta * offset
                    }
                };
                output.start = Offset {
                    x: origin.x + point_at(start_offset, first_x, second_x, delta_x) as f64,
                    y: origin.y + point_at(start_offset, first_y, second_y, delta_y) as f64,
                };
                output.end = Offset {
                    x: origin.x + point_at(end_offset, first_x, second_x, delta_x) as f64,
                    y: origin.y + point_at(end_offset, first_y, second_y, delta_y) as f64,
                };
                output.linear_angle = None;
            } else {
                ResolvePoint(&mut output.start, origin, unit_box);
                ResolvePoint(&mut output.end, origin, unit_box);
            }
            let basis = (output.end.x - output.start.x).hypot(output.end.y - output.start.y);
            ResolvePhysicalColorStops(&mut output, basis);
            ReplaceColorHints(&mut output);
            if let Some((first, last)) = NormalizeStops(&mut output) {
                let delta = Offset {
                    x: output.end.x - output.start.x,
                    y: output.end.y - output.start.y,
                };
                let original = output.start;
                output.start = Offset {
                    x: original.x + delta.x * first,
                    y: original.y + delta.y * first,
                };
                output.end = Offset {
                    x: original.x + delta.x * last,
                    y: original.y + delta.y * last,
                };
            }
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
            ReplaceColorHints(&mut output);
            if output.spread != layoutng_assembly::internal::paint_input::PaintSpreadMethod::kRepeat
            {
                ClampNegativeRadialStops(&mut output);
            }
            if let Some((first, last)) = NormalizeStops(&mut output) {
                let mut inner = output.radius * first;
                let mut outer = output.radius * last;
                if output.spread
                    == layoutng_assembly::internal::paint_input::PaintSpreadMethod::kRepeat
                    && inner < 0.0
                {
                    let period = outer - inner;
                    if period > 0.0 {
                        let shift = period * (-inner / period).ceil();
                        inner += shift;
                        outer += shift;
                    }
                }
                let y_scale = output.radius_y.unwrap() / output.radius.max(f64::MIN_POSITIVE);
                output.focal = output.center;
                output.focal_radius = inner.max(0.0);
                output.radius = outer.max(output.focal_radius);
                output.radius_y = Some(output.radius * y_scale);
            }
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
            ReplaceColorHints(&mut output);
            if let Some((first, last)) = NormalizeStops(&mut output) {
                let span = output.end_angle - output.start_angle;
                let start = output.start_angle;
                output.start_angle = start + span * first;
                output.end_angle = start + span * last;
            }
        }
        PaintShaderKind::kPattern => {}
    }
    let mut previous = f64::NEG_INFINITY;
    for stop in &output.stops {
        if stop.is_hint || stop.offset < previous || !(0.0..=1.0).contains(&stop.offset) {
            panic!("gradient stops must be ordered");
        }
        previous = stop.offset;
    }
    output.unit_coordinates = false;
    output.object_bounding_box_coordinates = false;
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::internal::paint_input::{PaintShaderKind, PaintSpreadMethod};

    fn color(red: f32, blue: f32) -> Color {
        Color {
            red,
            green: 0.0,
            blue,
            alpha: 1.0,
        }
    }

    #[test]
    fn css_gradient_geometry_stops_and_hints_resolve_against_the_target_box() {
        let linear = PaintShader {
            kind: PaintShaderKind::kLinearGradient,
            spread: PaintSpreadMethod::kRepeat,
            linear_corner: Some([1, 1]),
            stops: vec![
                PaintColorStop {
                    offset: 0.0,
                    offset_length: 10.0,
                    color: color(1.0, 0.0),
                    is_hint: false,
                    ..Default::default()
                },
                PaintColorStop {
                    offset: 0.4,
                    color: Color::default(),
                    is_hint: true,
                    ..Default::default()
                },
                PaintColorStop {
                    offset: 1.0,
                    color: color(0.0, 1.0),
                    ..Default::default()
                },
            ],
            unit_coordinates: true,
            ..Default::default()
        };
        let linear = ResolvePaintShader(
            &linear,
            None,
            Offset { x: 10.0, y: 20.0 },
            Some(Size {
                width: 200.0,
                height: 100.0,
            }),
        );
        assert!(linear.linear_angle.is_none() && linear.linear_corner.is_none());
        assert!(linear.start != linear.end);
        assert_eq!(linear.stops.len(), 11);
        assert!(linear.stops.iter().all(|stop| {
            !stop.is_hint && stop.offset_length == 0.0 && (0.0..=1.0).contains(&stop.offset)
        }));

        let radial = PaintShader {
            kind: PaintShaderKind::kRadialGradient,
            center: Offset { x: 0.25, y: 0.75 },
            focal: Offset { x: 0.25, y: 0.75 },
            radius: 0.5,
            radius_offset: 10.0,
            radius_y: Some(0.25),
            radial_shape: PaintRadialShape::kEllipse,
            radial_extent: PaintRadialExtent::kExplicit,
            stops: vec![
                PaintColorStop {
                    color: color(1.0, 0.0),
                    ..Default::default()
                },
                PaintColorStop {
                    offset: 1.0,
                    color: color(0.0, 1.0),
                    ..Default::default()
                },
            ],
            unit_coordinates: true,
            ..Default::default()
        };
        let radial = ResolvePaintShader(
            &radial,
            None,
            Offset::default(),
            Some(Size {
                width: 200.0,
                height: 100.0,
            }),
        );
        assert_eq!(radial.center, Offset { x: 50.0, y: 75.0 });
        assert_eq!(radial.radius, 110.0);
        assert_eq!(radial.radius_y, Some(25.0));

        let conic = PaintShader {
            kind: PaintShaderKind::kConicGradient,
            center: Offset { x: 0.25, y: 0.75 },
            center_offset: Offset { x: 2.0, y: -3.0 },
            rotation_angle: 45.0,
            stops: radial.stops.clone(),
            unit_coordinates: true,
            ..Default::default()
        };
        let conic = ResolvePaintShader(
            &conic,
            None,
            Offset { x: 5.0, y: 7.0 },
            Some(Size {
                width: 200.0,
                height: 100.0,
            }),
        );
        assert_eq!(conic.center, Offset { x: 57.0, y: 79.0 });
        assert_eq!(conic.rotation_angle, 45.0);
    }
}
