#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Offset, PaintPathCommand, PaintPathVerb, Size, SvgShapeData, SvgShapeGeometry,
    TransformMatrix,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    PaintStyleData, SvgMarkerInstance, SvgMarkerPrimitive, SvgPaintComponent, SvgPaintServer,
    SvgPaintServerReferenceBox, SvgStrokeLineCap, SvgStrokeLineJoin,
};

use crate::drawing_recorder::DrawingRecorder;
use crate::geometry_mapper::{InvertTransform, MapRectToRoot, MultiplyTransforms};
use crate::paint_auto_dark_mode::{ElementRole, InvertColorIfNeeded};
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};
use crate::paint_info::PaintInfo;
use crate::paint_shader_resolver::{PrivacyPreservingPaintServer, ResolvePaintShaderForBounds};
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::svg_model_object_painter::{SVGDrawingRecorder, SVGModelObjectPainter};
use crate::PaintRect;

// Resolved snapshot consumed by the translated SVGObjectPainter/StrokeData
// responsibilities in SVGShapePainter::PaintShape.
struct ShapePaintStyle {
    fill: Option<Color>,
    stroke: Option<Color>,
    fill_server: Option<SvgPaintServer>,
    stroke_server: Option<SvgPaintServer>,
    fill_server_reference_box: Option<SvgPaintServerReferenceBox>,
    fill_context_transform: Option<TransformMatrix>,
    stroke_server_reference_box: Option<SvgPaintServerReferenceBox>,
    stroke_context_transform: Option<TransformMatrix>,
    stroke_width: f64,
    stroke_dash_array: Vec<f64>,
    stroke_dash_offset: f64,
    stroke_line_cap: SvgStrokeLineCap,
    stroke_line_join: SvgStrokeLineJoin,
    stroke_miter_limit: f64,
    fill_even_odd: bool,
    non_scaling_stroke: bool,
    antialias: bool,
}

fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

fn Finite(point: Offset) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn ValidateTransform(transform: &TransformMatrix, message: &str) {
    for value in transform.values {
        if !value.is_finite() {
            panic!("{message}");
        }
    }
}

fn ValidateColor(color: Color) {
    if !color.red.is_finite()
        || !color.green.is_finite()
        || !color.blue.is_finite()
        || !color.alpha.is_finite()
    {
        panic!("SVG paint color must be finite");
    }
}

fn ValidateShape(shape: &SvgShapeData) {
    if !Finite(shape.bounds_offset)
        || !shape.bounds_width.is_finite()
        || !shape.bounds_height.is_finite()
        || shape.bounds_width < 0.0
        || shape.bounds_height < 0.0
    {
        panic!("SVG shape bounds are invalid");
    }
    if let Some(local_transform) = &shape.local_transform {
        ValidateTransform(local_transform, "SVG shape transform must be finite");
    }
    if shape.geometry != SvgShapeGeometry::kPath {
        return;
    }
    if shape
        .path
        .first()
        .is_none_or(|command| command.verb != PaintPathVerb::kMoveTo)
    {
        panic!("SVG path must start with move-to");
    }
    // A move-only path is valid, even though fill/stroke replay produces no
    // pixels. Keep it distinct from move+close and zero-length drawing verbs:
    // those can produce round/square stroke caps in Skia.
    for command in &shape.path {
        match command.verb {
            PaintPathVerb::kMoveTo => {
                if !Finite(command.point) {
                    panic!("SVG path coordinates must be finite");
                }
            }
            PaintPathVerb::kLineTo => {
                if !Finite(command.point) {
                    panic!("SVG path coordinates must be finite");
                }
            }
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
                if !Finite(command.control1) || !Finite(command.point) {
                    panic!("SVG path coordinates must be finite");
                }
            }
            PaintPathVerb::kCubicTo => {
                if !Finite(command.control1) || !Finite(command.control2) || !Finite(command.point)
                {
                    panic!("SVG path coordinates must be finite");
                }
            }
            PaintPathVerb::kClose => {}
        }
    }
}

fn ValidateStyle(style: &ShapePaintStyle) {
    if !style.stroke_width.is_finite()
        || style.stroke_width < 0.0
        || !style.stroke_dash_offset.is_finite()
        || !style.stroke_miter_limit.is_finite()
        || style.stroke_miter_limit < 1.0
    {
        panic!("SVG stroke geometry is invalid");
    }
    for dash in &style.stroke_dash_array {
        if !dash.is_finite() || *dash < 0.0 {
            panic!("SVG stroke dash lengths must be nonnegative");
        }
    }
    if let Some(fill) = style.fill {
        ValidateColor(fill);
    }
    if let Some(stroke) = style.stroke {
        ValidateColor(stroke);
    }
}

fn ResolvePath(input: &[PaintPathCommand], origin: Offset) -> Vec<PaintPathCommand> {
    let mut output = Vec::with_capacity(input.len());
    for mut command in input.iter().cloned() {
        let add_origin = |point: &mut Offset| {
            point.x += origin.x;
            point.y += origin.y;
        };
        match command.verb {
            PaintPathVerb::kMoveTo | PaintPathVerb::kLineTo => add_origin(&mut command.point),
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
                add_origin(&mut command.control1);
                add_origin(&mut command.point);
            }
            PaintPathVerb::kCubicTo => {
                add_origin(&mut command.control1);
                add_origin(&mut command.control2);
                add_origin(&mut command.point);
            }
            PaintPathVerb::kClose => {}
        }
        output.push(command);
    }
    output
}

fn RoundedRectanglePath(shape: &SvgShapeData, origin: Offset) -> Vec<PaintPathCommand> {
    let left = origin.x + shape.bounds_offset.x;
    let top = origin.y + shape.bounds_offset.y;
    let right = left + shape.bounds_width;
    let bottom = top + shape.bounds_height;
    let rx = shape.rectangle_radius_x.min(shape.bounds_width * 0.5);
    let ry = shape.rectangle_radius_y.min(shape.bounds_height * 0.5);
    const QUARTER_CIRCLE_WEIGHT: f64 = 0.7071067811865475244;
    vec![
        PaintPathCommand {
            verb: PaintPathVerb::kMoveTo,
            point: Offset {
                x: left + rx,
                y: top,
            },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: Offset {
                x: right - rx,
                y: top,
            },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset { x: right, y: top },
            point: Offset {
                x: right,
                y: top + ry,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: Offset {
                x: right,
                y: bottom - ry,
            },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset {
                x: right,
                y: bottom,
            },
            point: Offset {
                x: right - rx,
                y: bottom,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: Offset {
                x: left + rx,
                y: bottom,
            },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset { x: left, y: bottom },
            point: Offset {
                x: left,
                y: bottom - ry,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: Offset {
                x: left,
                y: top + ry,
            },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset { x: left, y: top },
            point: Offset {
                x: left + rx,
                y: top,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        },
    ]
}

fn FillType(geometry: SvgShapeGeometry) -> DisplayItemType {
    match geometry {
        SvgShapeGeometry::kRectangle => DisplayItemType::kDrawRect,
        SvgShapeGeometry::kEllipse => DisplayItemType::kDrawEllipse,
        SvgShapeGeometry::kPath => DisplayItemType::kDrawPath,
    }
}

fn StrokeType(geometry: SvgShapeGeometry) -> DisplayItemType {
    match geometry {
        SvgShapeGeometry::kRectangle => DisplayItemType::kStrokeRect,
        SvgShapeGeometry::kEllipse => DisplayItemType::kStrokeEllipse,
        SvgShapeGeometry::kPath => DisplayItemType::kStrokePath,
    }
}

fn MainStyle(style: &PaintStyleData) -> ShapePaintStyle {
    ShapePaintStyle {
        fill: style.svg_fill,
        stroke: style.svg_stroke,
        fill_server: style.svg_fill_server.clone(),
        stroke_server: style.svg_stroke_server.clone(),
        fill_server_reference_box: None,
        fill_context_transform: None,
        stroke_server_reference_box: None,
        stroke_context_transform: None,
        stroke_width: style.svg_stroke_width,
        stroke_dash_array: style.svg_stroke_dash_array.clone(),
        stroke_dash_offset: style.svg_stroke_dash_offset,
        stroke_line_cap: style.svg_stroke_line_cap,
        stroke_line_join: style.svg_stroke_line_join,
        stroke_miter_limit: style.svg_stroke_miter_limit,
        fill_even_odd: style.svg_fill_even_odd,
        non_scaling_stroke: style.svg_non_scaling_stroke,
        antialias: style.svg_shape_antialias,
    }
}

fn MarkerStyle(primitive: &SvgMarkerPrimitive) -> ShapePaintStyle {
    ShapePaintStyle {
        fill: primitive.fill,
        stroke: primitive.stroke,
        fill_server: primitive.fill_server.clone(),
        stroke_server: primitive.stroke_server.clone(),
        fill_server_reference_box: primitive.fill_server_reference_box.clone(),
        fill_context_transform: None,
        stroke_server_reference_box: primitive.stroke_server_reference_box.clone(),
        stroke_context_transform: None,
        stroke_width: primitive.stroke_width,
        stroke_dash_array: primitive.stroke_dash_array.clone(),
        stroke_dash_offset: primitive.stroke_dash_offset,
        stroke_line_cap: primitive.stroke_line_cap,
        stroke_line_join: primitive.stroke_line_join,
        stroke_miter_limit: primitive.stroke_miter_limit,
        fill_even_odd: primitive.fill_even_odd,
        non_scaling_stroke: primitive.non_scaling_stroke,
        antialias: primitive.antialias,
    }
}

// SVGObjectPainter::ResolveContextTransform keeps a context-fill/context-stroke
// paint server in the coordinate space of the context element. Marker and
// descendant transforms are already active on the canvas, so pre-multiply the
// inverse accumulated transform into the shader exactly once.
fn ResolveShapePaintServer(
    server: &SvgPaintServer,
    resources: Option<&layoutng_assembly::fragment_tree::PaintResources>,
    origin: Offset,
    reference_origin: Offset,
    reference_size: Size,
    is_context_paint: bool,
    extra_transforms: &[TransformMatrix],
    auto_dark_mode_paused: bool,
    privacy_preserving: bool,
) -> Option<SvgPaintServer> {
    let mut resolved = ResolvePaintShaderForBounds(
        server,
        resources,
        origin,
        reference_origin,
        Some(reference_size),
    );
    if is_context_paint && !extra_transforms.is_empty() {
        let context_transform = extra_transforms
            .iter()
            .fold(TransformMatrix::default(), |accumulated, transform| {
                MultiplyTransforms(&accumulated, transform)
            });
        // AffineTransform::Inverse returns identity for a singular transform.
        let inverse = InvertTransform(&context_transform).unwrap_or_default();
        resolved.transform = MultiplyTransforms(&inverse, &resolved.transform);
    }
    if auto_dark_mode_paused {
        resolved.apply_dark_mode = false;
    }
    PrivacyPreservingPaintServer(&resolved, privacy_preserving)
}

// Source: core/paint/svg_shape_painter.cc:98-216,221-251. The source Layout
// boundary has already resolved ComputedStyle, paint servers and StrokeData
// into this immutable input; recording retains the same fill/stroke order,
// geometry kind, winding rule and non-scaling-stroke marker.
fn BuildShapeItems(
    node: &PaintTreeNode<'_>,
    shape: &SvgShapeData,
    style: &ShapePaintStyle,
    origin: Offset,
    extra_transforms: &[TransformMatrix],
    marker_resource_id: u64,
    include_fill: bool,
    include_stroke: bool,
    privacy_preserving: bool,
) -> Vec<DisplayItem> {
    ValidateShape(shape);
    ValidateStyle(style);
    let server_allowed =
        |server: &SvgPaintServer| !privacy_preserving || server.is_in_canvas_subtree;
    let has_fill = style.fill_server.as_ref().is_some_and(server_allowed)
        || style.fill.is_some_and(|fill| fill.alpha > 0.0);
    let has_stroke = style.stroke_width > 0.0
        && (style.stroke_server.as_ref().is_some_and(server_allowed)
            || style.stroke.is_some_and(|stroke| stroke.alpha > 0.0));
    if matches!(
        shape.geometry,
        SvgShapeGeometry::kRectangle | SvgShapeGeometry::kEllipse
    ) && (shape.bounds_width == 0.0 || shape.bounds_height == 0.0)
    {
        return Vec::new();
    }
    let bounds = PaintRect {
        x: origin.x + shape.bounds_offset.x,
        y: origin.y + shape.bounds_offset.y,
        width: shape.bounds_width,
        height: shape.bounds_height,
    };
    let mut visual_bounds = bounds;
    if has_stroke {
        let outset = style.stroke_width * 0.5;
        visual_bounds = PaintRect {
            x: bounds.x - outset,
            y: bounds.y - outset,
            width: bounds.width + 2.0 * outset,
            height: bounds.height + 2.0 * outset,
        };
    }
    if let Some(cull_rect) = node.self_cull_rect {
        // Chromium compares the parent-space cull rect with the shape's
        // LocalSVGTransform-mapped visual rect. PrePaint has already applied
        // the inverse property-tree transform when it stores self_cull_rect,
        // so both operands here are in this object's local property space.
        // Marker transforms remain below that object-local boundary.
        let mapped = if extra_transforms.is_empty() {
            Some(visual_bounds)
        } else {
            MapRectToRoot(visual_bounds, extra_transforms)
        };
        if mapped.is_some_and(|mapped| !Intersects(&mapped, &cull_rect)) {
            return Vec::new();
        }
    }
    let rounded_rectangle = shape.geometry == SvgShapeGeometry::kRectangle
        && shape.rectangle_radius_x > 0.0
        && shape.rectangle_radius_y > 0.0;
    let path = if shape.geometry == SvgShapeGeometry::kPath {
        ResolvePath(&shape.path, origin)
    } else if rounded_rectangle {
        RoundedRectanglePath(shape, origin)
    } else {
        Vec::new()
    };
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let mut items = Vec::new();
    if include_fill && has_fill {
        let mut item = DisplayItem {
            r#type: if rounded_rectangle {
                DisplayItemType::kDrawPath
            } else {
                FillType(shape.geometry)
            },
            phase: PaintPhase::kForeground,
            node_id: fragment.node_id,
            rect: bounds,
            color: style.fill.unwrap_or_default(),
            ..Default::default()
        };
        item.path = path.clone();
        item.even_odd = style.fill_even_odd;
        item.antialias = style.antialias;
        item.svg_marker_resource_id = marker_resource_id;
        if let Some(fill_server) = &style.fill_server {
            let reference = style.fill_server_reference_box.as_ref();
            let reference_origin = reference.map_or(
                Offset {
                    x: bounds.x,
                    y: bounds.y,
                },
                |reference| reference.offset,
            );
            let reference_size = reference.map_or(
                Size {
                    width: bounds.width,
                    height: bounds.height,
                },
                |reference| reference.size,
            );
            item.paint_shader = ResolveShapePaintServer(
                fill_server,
                fragment.paint.resources.as_deref(),
                origin,
                reference_origin,
                reference_size,
                reference.is_some(),
                style
                    .fill_context_transform
                    .as_slice()
                    .first()
                    .map_or(extra_transforms, |_| {
                        style.fill_context_transform.as_slice()
                    }),
                fragment.paint.style.auto_dark_mode_paused(),
                privacy_preserving,
            );
        }
        if item.paint_shader.is_none() {
            item.color = InvertColorIfNeeded(&fragment.paint.style, item.color, ElementRole::kSVG);
        }
        items.push(item);
    }
    if include_stroke && has_stroke {
        let mut item = DisplayItem {
            r#type: if rounded_rectangle {
                DisplayItemType::kStrokePath
            } else {
                StrokeType(shape.geometry)
            },
            phase: PaintPhase::kForeground,
            node_id: fragment.node_id,
            rect: bounds,
            color: style.stroke.unwrap_or_default(),
            line_style: BorderLineStyle::kSolid,
            stroke_width: style.stroke_width,
            ..Default::default()
        };
        item.path = path.clone();
        item.dash_intervals = style.stroke_dash_array.clone();
        item.svg_line_cap = style.stroke_line_cap;
        item.svg_line_join = style.stroke_line_join;
        item.dash_offset = style.stroke_dash_offset;
        item.miter_limit = style.stroke_miter_limit;
        item.antialias = style.antialias;
        item.non_scaling_stroke = style.non_scaling_stroke;
        item.svg_marker_resource_id = marker_resource_id;
        if let Some(stroke_server) = &style.stroke_server {
            let reference = style.stroke_server_reference_box.as_ref();
            let reference_origin = reference.map_or(
                Offset {
                    x: bounds.x,
                    y: bounds.y,
                },
                |reference| reference.offset,
            );
            let reference_size = reference.map_or(
                Size {
                    width: bounds.width,
                    height: bounds.height,
                },
                |reference| reference.size,
            );
            item.paint_shader = ResolveShapePaintServer(
                stroke_server,
                fragment.paint.resources.as_deref(),
                origin,
                reference_origin,
                reference_size,
                reference.is_some(),
                style
                    .stroke_context_transform
                    .as_slice()
                    .first()
                    .map_or(extra_transforms, |_| {
                        style.stroke_context_transform.as_slice()
                    }),
                fragment.paint.style.auto_dark_mode_paused(),
                privacy_preserving,
            );
        }
        if item.paint_shader.is_none() {
            item.color = InvertColorIfNeeded(&fragment.paint.style, item.color, ElementRole::kSVG);
        }
        items.push(item);
    }
    items
}

fn AppendItems(
    context: &RefCell<PaintContext<'_>>,
    node: &PaintTreeNode<'_>,
    items: Vec<DisplayItem>,
) {
    // svg_shape_painter.cc:217-218,252-253. FillShape and StrokeShape mark
    // contentful paint only after a concrete draw reaches GraphicsContext.
    if !items.is_empty() {
        context.borrow_mut().SetFirstContentfulPaint();
    }
    for item in items {
        context.borrow_mut().Append(item, node);
    }
}

// Source: svg_shape_painter.cc:256-323. Borrow original resource Paint nodes;
// their independent property trees were prepared in the ordinary PrePaint pass.
fn PaintNativeMarkers(node: &PaintTreeNode<'_>, info: &PaintInfo<'_, '_>) -> bool {
    let Some(cursor) = node.SourceCursor() else {
        return false;
    };
    let markers = cursor
        .SvgMarkerPaints()
        .expect("native SVG marker provider");
    if markers.is_empty() {
        return true;
    }
    let paint = info
        .svg_resource_painter
        .expect("native resource subtree Paint dispatcher");
    let fragment = node.fragment.as_deref().expect("SVG shape Paint owner");
    let shape = fragment
        .paint
        .svg_shape
        .as_ref()
        .expect("SVG shape geometry owner");
    let reference_box = PaintRect {
        x: shape.bounds_offset.x,
        y: shape.bounds_offset.y,
        width: shape.bounds_width,
        height: shape.bounds_height,
    };
    for marker in markers {
        let resource = info
            .context
            .borrow()
            .SvgResourceNode(marker.resource_client_id)
            .expect("native marker resource must have PrePaint state");
        let command = |kind, transform, rect| DisplayItem {
            r#type: kind,
            phase: info.phase,
            node_id: fragment.node_id,
            svg_marker_resource_id: marker.resource_client_id,
            transform,
            rect,
            ..Default::default()
        };
        info.context.borrow_mut().Append(
            command(
                DisplayItemType::kSave,
                TransformMatrix::default(),
                PaintRect::default(),
            ),
            node,
        );
        info.context.borrow_mut().Append(
            command(
                DisplayItemType::kConcat,
                marker.transform,
                PaintRect::default(),
            ),
            node,
        );
        if let Some(clip) = marker.viewport_clip {
            info.context.borrow_mut().Append(
                command(
                    DisplayItemType::kClipRect,
                    TransformMatrix::default(),
                    PaintRect {
                        x: clip.offset.x,
                        y: clip.offset.y,
                        width: clip.size.width,
                        height: clip.size.height,
                    },
                ),
                node,
            );
        }
        let transform = info
            .GetSvgContextPaints()
            .map_or(marker.transform, |parent| {
                MultiplyTransforms(&parent.transform, &marker.transform)
            });
        let context_paints = crate::svg_context_paints::SvgContextPaints::New(
            &fragment.paint.style,
            reference_box,
            info.GetSvgContextPaints(),
            transform,
        );
        let mut resource_info = info.WithSvgContextPaints(&context_paints);
        resource_info.SetSvgResourceRoot(marker.resource_client_id);
        paint(resource, &resource_info);
        info.context.borrow_mut().Append(
            command(
                DisplayItemType::kRestore,
                TransformMatrix::default(),
                PaintRect::default(),
            ),
            node,
        );
    }
    true
}

// Source: core/paint/svg_shape_painter.cc:256-323. Marker selection and
// resource layout are resolved by Layout; Paint keeps the source transform,
// overflow clip, context-paint snapshot and nested record boundaries.
fn PaintMarkers(
    node: &PaintTreeNode<'_>,
    context: &RefCell<PaintContext<'_>>,
    markers: &[SvgMarkerInstance],
    privacy_preserving: bool,
) {
    for marker in markers {
        if marker.resource_id == 0 || marker.primitives.is_empty() {
            panic!("SVG marker resource is invalid");
        }
        ValidateTransform(&marker.transform, "SVG marker transform must be finite");
        let mut began_marker = false;
        for primitive in &marker.primitives {
            let mut transforms = vec![marker.transform];
            if let Some(transform) = primitive.transform {
                ValidateTransform(&transform, "SVG marker primitive transform must be finite");
                transforms.push(transform);
            }
            if let Some(transform) = primitive.shape.local_transform {
                transforms.push(transform);
            }
            let items = BuildShapeItems(
                node,
                &primitive.shape,
                &MarkerStyle(primitive),
                Offset::default(),
                &transforms,
                marker.resource_id,
                true,
                true,
                privacy_preserving,
            );
            if items.is_empty() {
                continue;
            }
            let node_id = node
                .fragment
                .as_deref()
                .expect("paint tree node has a fragment")
                .node_id;
            if !began_marker {
                context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase: PaintPhase::kForeground,
                        node_id,
                        svg_marker_resource_id: marker.resource_id,
                        ..Default::default()
                    },
                    node,
                );
                context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kConcat,
                        phase: PaintPhase::kForeground,
                        node_id,
                        svg_marker_resource_id: marker.resource_id,
                        transform: marker.transform,
                        ..Default::default()
                    },
                    node,
                );
                if let Some(offset) = marker.clip_offset {
                    context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kClipRect,
                            phase: PaintPhase::kForeground,
                            node_id,
                            svg_marker_resource_id: marker.resource_id,
                            rect: PaintRect {
                                x: offset.x,
                                y: offset.y,
                                width: marker.clip_size.width,
                                height: marker.clip_size.height,
                            },
                            ..Default::default()
                        },
                        node,
                    );
                }
                began_marker = true;
            }
            let has_local =
                primitive.transform.is_some() || primitive.shape.local_transform.is_some();
            if has_local {
                context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase: PaintPhase::kForeground,
                        node_id,
                        svg_marker_resource_id: marker.resource_id,
                        ..Default::default()
                    },
                    node,
                );
                if let Some(transform) = primitive.transform {
                    context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kConcat,
                            phase: PaintPhase::kForeground,
                            node_id,
                            svg_marker_resource_id: marker.resource_id,
                            transform,
                            ..Default::default()
                        },
                        node,
                    );
                }
                if let Some(transform) = primitive.shape.local_transform {
                    context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kConcat,
                            phase: PaintPhase::kForeground,
                            node_id,
                            svg_marker_resource_id: marker.resource_id,
                            transform,
                            ..Default::default()
                        },
                        node,
                    );
                }
            }
            AppendItems(context, node, items);
            if has_local {
                context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase: PaintPhase::kForeground,
                        node_id,
                        svg_marker_resource_id: marker.resource_id,
                        ..Default::default()
                    },
                    node,
                );
            }
        }
        if began_marker {
            context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: node
                        .fragment
                        .as_deref()
                        .expect("paint tree node has a fragment")
                        .node_id,
                    svg_marker_resource_id: marker.resource_id,
                    ..Default::default()
                },
                node,
            );
        }
    }
}

// Source: core/paint/svg_shape_painter.h:20-41.
pub struct SVGShapePainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> SVGShapePainter<'n, 'f, 'c, 'o> {
    // Source: core/paint/svg_shape_painter.h:24-25.
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // Source: core/paint/svg_shape_painter.h:27;
    // core/paint/svg_shape_painter.cc:43-96.
    pub fn Paint(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if paint_info.phase != PaintPhase::kForeground || fragment.paint.svg_shape.is_none() {
            return;
        }
        if !crate::scoped_svg_paint_state::ScopedSVGPaintState::PaintsContent(self.node) {
            return;
        }
        let shape = fragment.paint.svg_shape.as_deref().unwrap();
        if (shape.geometry == SvgShapeGeometry::kPath && shape.path.is_empty())
            || (shape.geometry != SvgShapeGeometry::kPath
                && (shape.bounds_width <= 0.0 || shape.bounds_height <= 0.0))
        {
            return;
        }
        let style = &*fragment.paint.style;
        let mut shape_style = MainStyle(style);
        for stroke in [false, true] {
            if let Some(context) =
                crate::svg_context_paints::ResolveFor(&fragment.paint.style, stroke, paint_info)
            {
                let rect = context.reference_box;
                let reference = Some(SvgPaintServerReferenceBox {
                    offset: Offset {
                        x: rect.x,
                        y: rect.y,
                    },
                    size: Size {
                        width: rect.width,
                        height: rect.height,
                    },
                });
                if stroke {
                    shape_style.stroke = context.color;
                    shape_style.stroke_server = context.server;
                    shape_style.stroke_server_reference_box = reference;
                    shape_style.stroke_context_transform = Some(context.transform);
                } else {
                    shape_style.fill = context.color;
                    shape_style.fill_server = context.server;
                    shape_style.fill_server_reference_box = reference;
                    shape_style.fill_context_transform = Some(context.transform);
                }
            }
        }
        // Chromium records compositing hit-test metadata before consulting the
        // retained drawing cache. The metadata must be rebuilt every paint
        // even when the SVG drawing itself is reused.
        SVGModelObjectPainter::RecordHitTestData(self.node, self.context, paint_info);
        SVGModelObjectPainter::RecordRegionCaptureData(self.node, self.context);
        let in_drawing = self.context.borrow().InDrawingRecorder();
        let drawing_type =
            crate::display_item_id::DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase);
        // Chromium: svg_shape_painter.cc:82-88. A retained SVG child reuses
        // its semantic DrawingDisplayItem before rebuilding fill/stroke/marker
        // commands. When an ancestor already owns the active recorder the
        // commands remain part of that ancestor's cache entry instead.
        if !in_drawing
            && DrawingRecorder::UseCachedDrawingIfPossible(self.context, self.node, drawing_type)
        {
            return;
        }
        let _drawing = (!in_drawing)
            .then(|| SVGDrawingRecorder::for_phase(self.context, self.node, paint_info.phase));
        if paint_info.IsRenderingClipPathAsMaskImage() {
            let mut clip_style = MainStyle(style);
            clip_style.fill = Some(Color {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            });
            clip_style.stroke = None;
            clip_style.fill_server = None;
            clip_style.stroke_server = None;
            clip_style.fill_even_odd = style.svg_clip_even_odd;
            AppendItems(
                self.context,
                self.node,
                BuildShapeItems(
                    self.node,
                    shape,
                    &clip_style,
                    self.node.paint_offset,
                    &[],
                    0,
                    true,
                    false,
                    paint_info.IsPrivacyPreserving(),
                ),
            );
        } else {
            for component in style.svg_paint_order {
                match component {
                    SvgPaintComponent::kFill => AppendItems(
                        self.context,
                        self.node,
                        BuildShapeItems(
                            self.node,
                            shape,
                            &shape_style,
                            self.node.paint_offset,
                            &[],
                            0,
                            true,
                            false,
                            paint_info.IsPrivacyPreserving(),
                        ),
                    ),
                    SvgPaintComponent::kStroke => AppendItems(
                        self.context,
                        self.node,
                        BuildShapeItems(
                            self.node,
                            shape,
                            &shape_style,
                            self.node.paint_offset,
                            &[],
                            0,
                            false,
                            true,
                            paint_info.IsPrivacyPreserving(),
                        ),
                    ),
                    SvgPaintComponent::kMarkers => {
                        if !PaintNativeMarkers(self.node, paint_info) {
                            PaintMarkers(
                                self.node,
                                self.context,
                                &style.svg_markers,
                                paint_info.IsPrivacyPreserving(),
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use layoutng_assembly::fragment_tree::FragmentPaintData;
    use layoutng_assembly::internal::layout_input::{NodeKind, SvgShapeData};
    use layoutng_assembly::internal::paint_input::SvgPatternCommand;

    use super::*;
    use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
    use crate::paint_context::PaintContext;
    use crate::paint_engine::{RasterEffectOutset, RecordedDisplayItem, RecordedDisplayItemKind};
    use crate::PaintRecording;
    use paint_reuse::PaintReuseMetrics;

    #[test]
    fn context_paint_server_cancels_the_accumulated_marker_transform() {
        let mut server = SvgPaintServer::default();
        server.stops = vec![Default::default()];
        server.transform.values[12] = 11.0;
        server.transform.values[13] = 13.0;

        let mut marker = TransformMatrix::default();
        marker.values[12] = 5.0;
        marker.values[13] = 7.0;
        let mut child = TransformMatrix::default();
        child.values[0] = 2.0;
        child.values[5] = 3.0;

        let resolved = ResolveShapePaintServer(
            &server,
            None,
            Offset::default(),
            Offset::default(),
            Size {
                width: 20.0,
                height: 10.0,
            },
            true,
            &[marker, child],
            false,
            false,
        )
        .unwrap();
        let context_transform = MultiplyTransforms(&marker, &child);
        let expected = MultiplyTransforms(
            &InvertTransform(&context_transform).unwrap(),
            &server.transform,
        );
        assert_eq!(resolved.transform, expected);
    }

    #[test]
    fn forced_dark_solid_svg_fill_uses_the_svg_role() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.force_dark = true;
        fragment.paint.style.svg_fill = Some(Color {
            alpha: 1.0,
            ..Color::default()
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let shape = SvgShapeData {
            geometry: SvgShapeGeometry::kRectangle,
            bounds_width: 12.0,
            bounds_height: 8.0,
            ..SvgShapeData::default()
        };
        let items = BuildShapeItems(
            &node,
            &shape,
            &MainStyle(&fragment.paint.style),
            Offset::default(),
            &[],
            0,
            true,
            false,
            false,
        );
        assert_eq!(items.len(), 1);
        assert!((items[0].color.red - 1.0).abs() < 0.00001);
        assert!((items[0].color.green - 1.0).abs() < 0.00001);
        assert!((items[0].color.blue - 1.0).abs() < 0.00001);
    }

    #[test]
    fn paused_svg_root_disables_gradient_stop_filtering_for_descendant_shapes() {
        let mut server = SvgPaintServer::default();
        server.apply_dark_mode = true;
        server.stops = vec![Default::default()];
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.force_dark = true;
        fragment.paint.style.auto_dark_mode_paused = true;
        fragment.paint.style.svg_fill_server = Some(server);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let shape = SvgShapeData {
            geometry: SvgShapeGeometry::kRectangle,
            bounds_width: 12.0,
            bounds_height: 8.0,
            ..SvgShapeData::default()
        };

        let items = BuildShapeItems(
            &node,
            &shape,
            &MainStyle(&fragment.paint.style),
            Offset::default(),
            &[],
            0,
            true,
            false,
            false,
        );

        assert_eq!(items.len(), 1);
        assert!(!items[0].paint_shader.as_ref().unwrap().apply_dark_mode);
    }

    #[test]
    fn privacy_preserving_svg_paint_rejects_external_server_and_uses_fallback() {
        let mut server = SvgPaintServer::default();
        server.stops = vec![Default::default()];
        let fallback = Color {
            red: 0.25,
            green: 0.5,
            blue: 0.75,
            alpha: 1.0,
        };
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.svg_fill_server = Some(server.clone());
        fragment.paint.style.svg_fill = Some(fallback);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let shape = SvgShapeData {
            geometry: SvgShapeGeometry::kRectangle,
            bounds_width: 12.0,
            bounds_height: 8.0,
            ..SvgShapeData::default()
        };

        let rejected = BuildShapeItems(
            &node,
            &shape,
            &MainStyle(&fragment.paint.style),
            Offset::default(),
            &[],
            0,
            true,
            false,
            true,
        );
        assert_eq!(rejected.len(), 1);
        assert!(rejected[0].paint_shader.is_none());
        assert_eq!(rejected[0].color, fallback);

        server.is_in_canvas_subtree = true;
        fragment.paint.style.svg_fill_server = Some(server);
        node.SetOwnedFragmentForTesting(&fragment);
        let admitted = BuildShapeItems(
            &node,
            &shape,
            &MainStyle(&fragment.paint.style),
            Offset::default(),
            &[],
            0,
            true,
            false,
            true,
        );
        assert_eq!(admitted.len(), 1);
        assert!(admitted[0].paint_shader.is_some());
    }

    #[test]
    fn privacy_preserving_pattern_sanitizes_nested_paint_servers() {
        let mut nested = SvgPaintServer::default();
        nested.stops = vec![Default::default()];
        let server = SvgPaintServer {
            is_in_canvas_subtree: true,
            pattern: Some(Arc::new(
                layoutng_assembly::internal::paint_input::SvgPatternData {
                    commands: vec![SvgPatternCommand::DrawShape(SvgMarkerPrimitive {
                        fill_server: Some(nested),
                        fill: Some(Color::default()),
                        ..SvgMarkerPrimitive::default()
                    })],
                    ..Default::default()
                },
            )),
            ..SvgPaintServer::default()
        };

        let server = PrivacyPreservingPaintServer(&server, true).unwrap();
        let primitive = server
            .pattern
            .as_ref()
            .unwrap()
            .commands
            .iter()
            .find_map(|command| match command {
                SvgPatternCommand::DrawShape(primitive) => Some(primitive),
                _ => None,
            })
            .unwrap();
        assert!(primitive.fill_server.is_none());
        assert!(primitive.fill.is_some());
    }

    #[test]
    fn clip_path_mask_paints_black_fill_with_clip_rule() {
        use paint_model::paint_flags::PaintFlag;

        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.is_svg_child = true;
        fragment.paint.svg_shape = Some(Arc::new(SvgShapeData {
            geometry: SvgShapeGeometry::kRectangle,
            bounds_width: 12.0,
            bounds_height: 8.0,
            ..SvgShapeData::default()
        }));
        fragment.paint.style.svg_fill = None;
        fragment.paint.style.svg_stroke = Some(Color {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        });
        fragment.paint.style.svg_stroke_width = 3.0;
        fragment.paint.style.svg_clip_even_odd = true;
        fragment.paint.style.svg_shape_antialias = false;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);

        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new_with_flags(
            &context,
            PaintPhase::kForeground,
            false,
            PaintFlag::kPaintingClipPathAsMask | PaintFlag::kOmitCompositingInfo,
        );
        SVGShapePainter::new(&node, &context).Paint(&paint_info);
        drop(paint_info);
        drop(context);

        let draws = recording
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item.r#type,
                    DisplayItemType::kDrawRect
                        | DisplayItemType::kDrawEllipse
                        | DisplayItemType::kDrawPath
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].color.red, 0.0);
        assert_eq!(draws[0].color.green, 0.0);
        assert_eq!(draws[0].color.blue, 0.0);
        assert_eq!(draws[0].color.alpha, 1.0);
        assert!(draws[0].even_odd);
        assert!(!draws[0].antialias);
        assert!(recording.first_contentful_painted);
        assert!(!recording
            .chunks
            .iter()
            .any(|chunk| chunk.id.r#type == DisplayItemIdType::kHitTest));
    }

    #[test]
    fn svg_shape_records_layout_hit_test_facts_and_svg_opaqueness() {
        use crate::paint_engine::PaintHitTestOpaqueness;

        for (visible_to_hit_testing, expected_opaqueness) in [
            (true, PaintHitTestOpaqueness::Mixed),
            (false, PaintHitTestOpaqueness::Transparent),
        ] {
            let mut fragment = FragmentPaintData::default();
            fragment.paint.has_source = true;
            fragment.paint.source_kind = NodeKind::kSvgShape;
            fragment.paint.is_svg_child = true;
            fragment.paint.display_item_client_id = 73;
            fragment.paint.visible_to_hit_testing = visible_to_hit_testing;
            fragment.paint.effective_allowed_touch_action = foundation::TouchAction::kNone;
            fragment.paint.inside_blocking_wheel_event_handler = true;
            fragment.paint.region_capture_crop_id = Some([17, 29]);
            fragment.paint.svg_shape = Some(Arc::new(SvgShapeData {
                geometry: SvgShapeGeometry::kRectangle,
                bounds_width: 12.0,
                bounds_height: 8.0,
                ..SvgShapeData::default()
            }));
            let mut node = PaintTreeNode::default();
            node.SetOwnedFragmentForTesting(&fragment);

            let mut recording = PaintRecording::default();
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
            SVGShapePainter::new(&node, &context).Paint(&paint_info);
            drop(paint_info);
            drop(context);

            let chunk = recording
                .chunks
                .iter()
                .find(|chunk| chunk.id.r#type == DisplayItemIdType::kHitTest)
                .expect("SVG hit-test chunk");
            let metadata = chunk.compositor.as_deref().expect("compositor metadata");
            assert_eq!(metadata.hit_test_opaqueness, expected_opaqueness);
            let hit_test = metadata.hit_test_data.as_deref().expect("hit-test data");
            assert_eq!(
                hit_test.touch_action_rects,
                vec![(
                    PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 12.0,
                        height: 8.0,
                    },
                    foundation::TouchAction::kNone.bits() as u32,
                )]
            );
            assert_eq!(
                hit_test.wheel_event_rects,
                vec![PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 12.0,
                    height: 8.0,
                }]
            );
            let region_chunk = recording
                .chunks
                .iter()
                .find(|chunk| {
                    chunk
                        .compositor
                        .as_deref()
                        .and_then(|metadata| metadata.region_capture_data.as_deref())
                        .is_some()
                })
                .expect("SVG region-capture chunk");
            assert_eq!(
                region_chunk
                    .compositor
                    .as_deref()
                    .unwrap()
                    .region_capture_data
                    .as_deref()
                    .unwrap()
                    .as_slice(),
                &[(
                    [17, 29],
                    PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 12.0,
                        height: 8.0,
                    },
                )]
            );
        }
    }

    #[test]
    fn retained_shape_reuses_its_foreground_drawing() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.is_svg_child = true;
        fragment.paint.display_item_client_id = 91;
        fragment.paint.effective_allowed_touch_action = foundation::TouchAction::kPanX;
        fragment.paint.region_capture_crop_id = Some([41, 43]);
        fragment.paint.display_item_client_is_cacheable = true;
        fragment.paint.display_item_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kNone);
        fragment.paint.svg_shape = Some(Arc::new(SvgShapeData {
            geometry: SvgShapeGeometry::kRectangle,
            bounds_width: 12.0,
            bounds_height: 8.0,
            ..SvgShapeData::default()
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);

        let item_type = DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kForeground);
        let id = DisplayItemId {
            client_id: 91,
            r#type: item_type,
            fragment: 0,
        };
        let visual_rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 12.0,
            height: 8.0,
        };
        let mut previous = PaintRecording::default();
        previous.items = Arc::new(vec![DisplayItem {
            r#type: DisplayItemType::kDrawRect,
            rect: visual_rect,
            ..DisplayItem::default()
        }]);
        previous.display_items.push(RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            id,
            visual_rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: Default::default(),
            record_background: Default::default(),
            record_has_text: false,
            record_begin: 0,
            record_end: 1,
            scroll_translation: None,
        });

        let mut output = PaintRecording::default();
        let context = RefCell::new(PaintContext::WithEditingAndCache(
            &mut output,
            Default::default(),
            Some(Arc::new(previous)),
        ));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
        SVGShapePainter::new(&node, &context).Paint(&paint_info);
        assert_eq!(
            context.borrow().CacheMetrics(),
            PaintReuseMetrics {
                candidates: 1,
                attempts: 1,
                reused: 1,
                ..PaintReuseMetrics::default()
            }
        );
        drop(paint_info);
        drop(context);
        assert_eq!(output.display_items.len(), 1);
        assert_eq!(output.items.len(), 1);
        let hit_test_chunk = output
            .chunks
            .iter()
            .find(|chunk| chunk.id.r#type == DisplayItemIdType::kHitTest)
            .expect("hit-test data must be rebuilt before cached drawing reuse");
        let metadata = hit_test_chunk.compositor.as_deref().unwrap();
        assert_eq!(
            metadata.hit_test_opaqueness,
            crate::paint_engine::PaintHitTestOpaqueness::Mixed
        );
        assert_eq!(
            metadata
                .hit_test_data
                .as_deref()
                .unwrap()
                .touch_action_rects,
            vec![(visual_rect, foundation::TouchAction::kPanX.bits() as u32)]
        );
        assert_eq!(
            metadata.region_capture_data.as_deref().unwrap().as_slice(),
            &[([41, 43], visual_rect)]
        );
    }
}
