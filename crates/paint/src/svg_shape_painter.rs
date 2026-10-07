#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Offset, PaintPathCommand, PaintPathVerb, Size, SvgShapeData, SvgShapeGeometry,
    TransformMatrix,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    PaintStyleData, SvgMarkerInstance, SvgMarkerPrimitive, SvgPaintComponent, SvgPaintServer,
    SvgStrokeLineCap, SvgStrokeLineJoin,
};

use crate::display_item_id::DisplayItemIdType;
use crate::drawing_recorder::DrawingRecorder;
use crate::geometry_mapper::MapRectToRoot;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};
use crate::paint_info::PaintInfo;
use crate::paint_shader_resolver::ResolvePaintShader;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/svg_shape_painter.cc:13-27
struct ShapePaintStyle {
    fill: Option<Color>,
    stroke: Option<Color>,
    fill_server: Option<SvgPaintServer>,
    stroke_server: Option<SvgPaintServer>,
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

// cpp: paint/svg_shape_painter.cc:29-32
fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

// cpp: paint/svg_shape_painter.cc:34-36
fn Finite(point: Offset) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

// cpp: paint/svg_shape_painter.cc:38-44
fn ValidateTransform(transform: &TransformMatrix, message: &str) {
    for value in transform.values {
        if !value.is_finite() {
            panic!("{message}");
        }
    }
}

// cpp: paint/svg_shape_painter.cc:46-50
fn ValidateColor(color: Color) {
    if !color.red.is_finite()
        || !color.green.is_finite()
        || !color.blue.is_finite()
        || !color.alpha.is_finite()
    {
        panic!("SVG paint color must be finite");
    }
}

// cpp: paint/svg_shape_painter.cc:52-96
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
    let mut has_geometry = false;
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
                has_geometry = true;
            }
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
                if !Finite(command.control1) || !Finite(command.point) {
                    panic!("SVG path coordinates must be finite");
                }
                has_geometry = true;
            }
            PaintPathVerb::kCubicTo => {
                if !Finite(command.control1) || !Finite(command.control2) || !Finite(command.point)
                {
                    panic!("SVG path coordinates must be finite");
                }
                has_geometry = true;
            }
            PaintPathVerb::kClose => has_geometry = true,
        }
    }
    if !has_geometry {
        panic!("SVG path has no drawable geometry");
    }
}

// cpp: paint/svg_shape_painter.cc:98-112
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

// cpp: paint/svg_shape_painter.cc:114-145
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

// cpp: paint/svg_shape_painter.cc:147-182
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

// cpp: paint/svg_shape_painter.cc:184-190
fn FillType(geometry: SvgShapeGeometry) -> DisplayItemType {
    match geometry {
        SvgShapeGeometry::kRectangle => DisplayItemType::kDrawRect,
        SvgShapeGeometry::kEllipse => DisplayItemType::kDrawEllipse,
        SvgShapeGeometry::kPath => DisplayItemType::kDrawPath,
    }
}

// cpp: paint/svg_shape_painter.cc:192-198
fn StrokeType(geometry: SvgShapeGeometry) -> DisplayItemType {
    match geometry {
        SvgShapeGeometry::kRectangle => DisplayItemType::kStrokeRect,
        SvgShapeGeometry::kEllipse => DisplayItemType::kStrokeEllipse,
        SvgShapeGeometry::kPath => DisplayItemType::kStrokePath,
    }
}

// cpp: paint/svg_shape_painter.cc:200-214
fn MainStyle(style: &PaintStyleData) -> ShapePaintStyle {
    ShapePaintStyle {
        fill: style.svg_fill,
        stroke: style.svg_stroke,
        fill_server: style.svg_fill_server.clone(),
        stroke_server: style.svg_stroke_server.clone(),
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

// cpp: paint/svg_shape_painter.cc:216-230
fn MarkerStyle(primitive: &SvgMarkerPrimitive) -> ShapePaintStyle {
    ShapePaintStyle {
        fill: primitive.fill,
        stroke: primitive.stroke,
        fill_server: primitive.fill_server.clone(),
        stroke_server: primitive.stroke_server.clone(),
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

// cpp: paint/svg_shape_painter.cc:232-327
fn BuildShapeItems(
    node: &PaintTreeNode<'_>,
    shape: &SvgShapeData,
    style: &ShapePaintStyle,
    origin: Offset,
    extra_transforms: &[TransformMatrix],
    marker_resource_id: u64,
    include_fill: bool,
    include_stroke: bool,
) -> Vec<DisplayItem> {
    ValidateShape(shape);
    ValidateStyle(style);
    let has_fill = style.fill_server.is_some() || style.fill.is_some_and(|fill| fill.alpha > 0.0);
    let has_stroke = style.stroke_width > 0.0
        && (style.stroke_server.is_some() || style.stroke.is_some_and(|stroke| stroke.alpha > 0.0));
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
    if let Some(cull_rect) = node.cull_rect {
        let mut transforms = node.transforms.clone();
        transforms.extend_from_slice(extra_transforms);
        let mapped = MapRectToRoot(visual_bounds, &transforms);
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
            item.paint_shader = Some(ResolvePaintShader(
                fill_server,
                fragment.paint.resources.as_deref(),
                Offset {
                    x: bounds.x,
                    y: bounds.y,
                },
                Some(Size {
                    width: bounds.width,
                    height: bounds.height,
                }),
            ));
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
            item.paint_shader = Some(ResolvePaintShader(
                stroke_server,
                fragment.paint.resources.as_deref(),
                Offset {
                    x: bounds.x,
                    y: bounds.y,
                },
                Some(Size {
                    width: bounds.width,
                    height: bounds.height,
                }),
            ));
        }
        items.push(item);
    }
    items
}

// cpp: paint/svg_shape_painter.cc:329-334
fn AppendItems(
    context: &RefCell<PaintContext<'_>>,
    node: &PaintTreeNode<'_>,
    items: Vec<DisplayItem>,
) {
    for item in items {
        context.borrow_mut().Append(item, node);
    }
}

// cpp: paint/svg_shape_painter.cc:336-415
fn PaintMarkers(
    node: &PaintTreeNode<'_>,
    context: &RefCell<PaintContext<'_>>,
    markers: &[SvgMarkerInstance],
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

// cpp: paint/svg_shape_painter.h:7-19
pub struct SVGShapePainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> SVGShapePainter<'n, 'f, 'c, 'o> {
    // cpp: paint/svg_shape_painter.h:9-10
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // cpp: paint/svg_shape_painter.h:12
    // cpp: paint/svg_shape_painter.cc:419-443
    pub fn Paint(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if paint_info.phase != PaintPhase::kForeground || fragment.paint.svg_shape.is_none() {
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
        let shape_style = MainStyle(style);
        // SVGShapePainter::Paint (.cc:85) owns one SVGDrawingRecorder for
        // PaintShape; fill, stroke and markers are operations in that record.
        // SVGDrawingRecorder's exact visual overflow is not exposed here.
        let stroke_outset = if shape_style.stroke.is_some() || shape_style.stroke_server.is_some() {
            shape_style.stroke_width * 0.5
        } else {
            0.0
        };
        let visual_rect = PaintRect {
            x: self.node.paint_offset.x + shape.bounds_offset.x - stroke_outset,
            y: self.node.paint_offset.y + shape.bounds_offset.y - stroke_outset,
            width: shape.bounds_width + 2.0 * stroke_outset,
            height: shape.bounds_height + 2.0 * stroke_outset,
        };
        let in_drawing = self.context.borrow().InDrawingRecorder();
        let _drawing = (!in_drawing).then(|| {
            DrawingRecorder::new(
                self.context,
                self.node,
                DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase),
                visual_rect,
            )
        });
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
                    ),
                ),
                SvgPaintComponent::kMarkers => {
                    PaintMarkers(self.node, self.context, &style.svg_markers)
                }
            }
        }
    }
}
