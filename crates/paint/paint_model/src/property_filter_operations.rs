//! Paint owns the compositor filter value. Authored style operations are consumed
//! here, before publishing Effect property nodes; compositor never rebuilds a
//! filter list while traversing the PaintArtifact.
use crate::cc::paint::{
    filter_operation::{
        FilterColor, FilterMapTransform, FilterOperation, FilterType, MapDirection, PaintFilter,
        PaintFilterReplay,
    },
    filter_operations::FilterOperations,
};
use crate::svg_filter_geometry::{
    graph_requires_viewport, map_svg_filter_forward_bounds, resolve_svg_filter_geometry,
    ResolvedSvgFilterGeometry,
};
use crate::PaintRect;
use foundation::gfx::{Point, Rect, Size};
use layoutng_assembly::internal::{
    layout_input::Offset,
    layout_input_types::Color,
    paint_input::{
        PaintCanvasFilter, PaintFilterOperation, PaintFilterType, SvgFilterGraph, SvgFilterInput,
        SvgFilterLength, SvgFilterPrimitiveKind, SvgFilterRegion, SvgFilterUnits,
    },
};
use std::{collections::HashMap, fmt, sync::Arc};

// Source: svg_filter_builder.cc:110-132,182-205 and
// filter_effect.cc:129-135. This computes the taint of LastEffect(), not the
// taint of every primitive that happened to be declared in the graph.
fn svg_filter_origin_tainted(graph: &SvgFilterGraph, source_tainted: bool) -> bool {
    let mut previous = None;
    let mut named = HashMap::<&str, bool>::new();
    let lookup =
        |input: &SvgFilterInput, previous: Option<bool>, named: &HashMap<&str, bool>| match input {
            SvgFilterInput::kSourceGraphic | SvgFilterInput::kSourceAlpha => source_tainted,
            SvgFilterInput::kFillPaint => {
                if graph.fill_paint.is_some() {
                    source_tainted
                } else {
                    previous.unwrap_or(source_tainted)
                }
            }
            SvgFilterInput::kStrokePaint => {
                if graph.stroke_paint.is_some() {
                    source_tainted
                } else {
                    previous.unwrap_or(source_tainted)
                }
            }
            SvgFilterInput::kNamed(name) if name.is_empty() => previous.unwrap_or(source_tainted),
            SvgFilterInput::kNamed(name) => named
                .get(name.as_str())
                .copied()
                .unwrap_or_else(|| previous.unwrap_or(source_tainted)),
        };
    for primitive in &graph.primitives {
        let first = lookup(&primitive.input, previous, &named);
        let inputs_tainted = match &primitive.kind {
            SvgFilterPrimitiveKind::kFlood { .. }
            | SvgFilterPrimitiveKind::kTurbulence { .. }
            | SvgFilterPrimitiveKind::kImage { .. } => false,
            SvgFilterPrimitiveKind::kBlend { input2, .. }
            | SvgFilterPrimitiveKind::kComposite { input2, .. }
            | SvgFilterPrimitiveKind::kDisplacementMap { input2, .. } => {
                first || lookup(input2, previous, &named)
            }
            SvgFilterPrimitiveKind::kMerge { inputs } => {
                inputs.iter().any(|input| lookup(input, previous, &named))
            }
            _ => first,
        };
        let tainted = primitive.origin_tainted || inputs_tainted;
        let result_is_builtin = primitive.result == "SourceGraphic"
            || primitive.result == "SourceAlpha"
            || (primitive.result == "FillPaint" && graph.fill_paint.is_some())
            || (primitive.result == "StrokePaint" && graph.stroke_paint.is_some());
        if result_is_builtin {
            continue;
        }
        if !primitive.result.is_empty() {
            named.insert(primitive.result.as_str(), tainted);
        }
        previous = Some(tainted);
    }
    previous.unwrap_or(source_tainted)
}

fn luminance_to_alpha_matrix() -> [f32; 20] {
    let mut matrix = [0.0; 20];
    matrix[15] = 0.2125;
    matrix[16] = 0.7154;
    matrix[17] = 0.0721;
    matrix
}

fn generic_color_matrix(values: &[f64]) -> [f32; 20] {
    let mut matrix = [0.0; 20];
    matrix[0] = 1.0;
    matrix[6] = 1.0;
    matrix[12] = 1.0;
    matrix[18] = 1.0;
    if values.len() == matrix.len() {
        for (output, value) in matrix.iter_mut().zip(values) {
            *output = *value as f32;
        }
    }
    matrix
}

fn canvas_filter_is_finite(filter: &PaintCanvasFilter) -> bool {
    match filter {
        PaintCanvasFilter::kConvolveMatrix {
            divisor,
            bias,
            kernel,
            ..
        } => divisor.is_finite() && bias.is_finite() && kernel.iter().all(|v| v.is_finite()),
        PaintCanvasFilter::kComponentTransfer { functions } => functions.iter().all(|function| {
            [
                function.slope,
                function.intercept,
                function.amplitude,
                function.exponent,
                function.offset,
            ]
            .iter()
            .all(|v| v.is_finite())
                && function.table_values.iter().all(|v| v.is_finite())
        }),
        PaintCanvasFilter::kTurbulence {
            base_x,
            base_y,
            seed,
            ..
        } => base_x.is_finite() && base_y.is_finite() && seed.is_finite(),
    }
}

/// Explicitly retained unsupported input, never an empty reference filter.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterInputBlocker {
    pub operation_index: usize,
    pub operation_count: usize,
    pub reason: FilterInputBlockReason,
    pub source: Arc<PaintFilterOperation>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterInputBlockReason {
    NonFiniteValue,
    ReferenceOwnerNotLowered,
    ReferenceViewportNotLowered,
    ReflectionMaskRecordNotLowered,
    CanvasPayloadNotLowered,
}
impl fmt::Display for FilterInputBlocker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "filter operation {}: {:?}",
            self.operation_index, self.reason
        )
    }
}
impl std::error::Error for FilterInputBlocker {}

/// Source: third_party/blink/renderer/platform/graphics/paint/
/// effect_paint_property_node.h:90-105,245-280. One canonical operations owner per effect,
/// rather than a second authored vector beside a compositor conversion cache.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyFilterOperations {
    Ready {
        operations: FilterOperations,
        origin_tainted: bool,
    },
    Blocked(FilterInputBlocker),
}
impl Default for PropertyFilterOperations {
    fn default() -> Self {
        Self::Ready {
            operations: FilterOperations::default(),
            origin_tainted: false,
        }
    }
}
impl PropertyFilterOperations {
    /// Source: third_party/blink/renderer/core/paint/filter_effect_builder.cc:373-510;
    /// platform/graphics/compositor_filter_operations.cc:22-109. PrePaint has
    /// already resolved zoom, percentages, currentColor and image resources.
    pub fn from_authored(
        input: &[PaintFilterOperation],
        reference_box: Option<PaintRect>,
        reference_viewport: Option<PaintRect>,
        reference_origin: Offset,
        reference_scale: f64,
    ) -> Self {
        Self::from_authored_with_input_taint(
            input,
            reference_box,
            reference_viewport,
            reference_origin,
            reference_scale,
            false,
        )
    }

    /// Source: filter_effect_builder.cc:144-152 and
    /// filter_effect_builder.h:62-63. Canvas can supply an already-tainted
    /// SourceGraphic; ordinary CSS/PrePaint callers use `from_authored()` and
    /// therefore retain the source default of false.
    pub fn from_authored_with_input_taint(
        input: &[PaintFilterOperation],
        reference_box: Option<PaintRect>,
        reference_viewport: Option<PaintRect>,
        reference_origin: Offset,
        reference_scale: f64,
        input_origin_tainted: bool,
    ) -> Self {
        let mut operations = Vec::with_capacity(input.len());
        let mut origin_tainted = input_origin_tainted;
        for (operation_index, op) in input.iter().enumerate() {
            let blocked = |reason| {
                Self::Blocked(FilterInputBlocker {
                    operation_index,
                    operation_count: input.len(),
                    reason,
                    source: Arc::new(op.clone()),
                })
            };
            if ![
                op.amount,
                op.offset.x,
                op.offset.y,
                op.blur_radius,
                f64::from(op.color.red),
                f64::from(op.color.green),
                f64::from(op.color.blue),
                f64::from(op.color.alpha),
            ]
            .iter()
            .all(|v| v.is_finite())
                || !op.color_matrix.iter().all(|value| value.is_finite())
            {
                return blocked(FilterInputBlockReason::NonFiniteValue);
            }
            use PaintFilterType::*;
            let value = match op.r#type {
                kGrayscale => FilterOperation::CreateGrayscaleFilter(op.amount as f32),
                kSepia => FilterOperation::CreateSepiaFilter(op.amount as f32),
                kSaturate => FilterOperation::CreateSaturateFilter(op.amount as f32),
                kHueRotate => FilterOperation::CreateHueRotateFilter(op.amount as f32),
                kLuminanceToAlpha => {
                    FilterOperation::CreateColorMatrixFilter(luminance_to_alpha_matrix())
                }
                kColorMatrix => {
                    FilterOperation::CreateColorMatrixFilter(generic_color_matrix(&op.color_matrix))
                }
                kInvert => FilterOperation::CreateInvertFilter(op.amount as f32),
                kBrightness => FilterOperation::CreateBrightnessFilter(op.amount as f32),
                kContrast => FilterOperation::CreateContrastFilter(op.amount as f32),
                kOpacity => FilterOperation::CreateOpacityFilter(op.amount as f32),
                kBlur => FilterOperation::CreateBlurFilter(op.amount as f32),
                kDropShadow => {
                    // Source: filter_effect_builder.cc:472-483. The resolved
                    // color is sufficient for rendering, but dependence on
                    // currentColor is retained separately for origin-taint
                    // propagation on the compositor filter value.
                    origin_tainted |= op.uses_current_color;
                    FilterOperation::CreateDropShadowFilter(
                        Point::new(op.offset.x.floor() as i32, op.offset.y.floor() as i32),
                        op.blur_radius as f32,
                        // AppendDropShadowFilter passes Color::Rgb through SkColor.
                        FilterColor::new(
                            byte_color(op.color.red),
                            byte_color(op.color.green),
                            byte_color(op.color.blue),
                            byte_color(op.color.alpha),
                        ),
                    )
                }
                kConvolveMatrix | kComponentTransfer | kTurbulence => {
                    let Some(canvas_filter) = op.canvas_filter.as_ref() else {
                        return blocked(FilterInputBlockReason::CanvasPayloadNotLowered);
                    };
                    if !canvas_filter_is_finite(canvas_filter) {
                        return blocked(FilterInputBlockReason::NonFiniteValue);
                    }
                    FilterOperation::CreateReferenceFilter(Some(Arc::new(CanvasPaintFilter {
                        operation: op.clone(),
                    })))
                }
                kReference => {
                    // Source: filter_effect_builder.cc:380-405,519-523. An
                    // unresolved/wrong-kind SVG resource produces no
                    // compositor operation; it does not poison later filter
                    // functions in the same list. A graph that reached this
                    // boundary but carries an unsupported lowering error is a
                    // different case and must remain explicit.
                    let Some(graph) = op.reference_graph.as_ref() else {
                        continue;
                    };
                    if op.reference_error.is_some() {
                        return blocked(FilterInputBlockReason::ReferenceOwnerNotLowered);
                    }
                    let Some(reference_box) = reference_box else {
                        return blocked(FilterInputBlockReason::ReferenceOwnerNotLowered);
                    };
                    if reference_viewport.is_none() && graph_requires_viewport(graph) {
                        return blocked(FilterInputBlockReason::ReferenceViewportNotLowered);
                    }
                    let geometry = resolve_svg_filter_geometry(
                        graph,
                        reference_box,
                        reference_viewport,
                        reference_scale,
                    )
                    .expect("reference viewport was validated before SVG geometry resolution");
                    // Source: filter_effect_builder.cc:545-550 and
                    // svg_filter_builder.cc:182-205. An empty filter region
                    // returns before graph construction, and a graph with no
                    // filter-effect children never acquires LastEffect(); in
                    // both cases BuildFilterOperations appends nothing.
                    if geometry.filter_region.width <= 0.0
                        || geometry.filter_region.height <= 0.0
                        || geometry.primitives.is_empty()
                    {
                        continue;
                    }
                    let input_origin_tainted = origin_tainted;
                    origin_tainted |= svg_filter_origin_tainted(graph, input_origin_tainted);
                    FilterOperation::CreateReferenceFilter(Some(Arc::new(ReferencePaintFilter {
                        operation: op.clone(),
                        reference_box,
                        reference_viewport,
                        reference_origin,
                        reference_scale,
                        input_origin_tainted,
                        geometry,
                    })))
                }
                kBoxReflect => {
                    if op.reflection_mask_layers.iter().any(|layer| {
                        matches!(
                            layer.image.as_ref().map(|image| &image.content),
                            Some(image_resource::PaintImageContent::Document(_))
                        )
                    }) {
                        return blocked(FilterInputBlockReason::ReflectionMaskRecordNotLowered);
                    }
                    FilterOperation::CreateReferenceFilter(Some(Arc::new(ReflectionPaintFilter {
                        operation: op.clone(),
                    })))
                }
            };
            operations.push(value);
        }
        Self::Ready {
            operations: FilterOperations::new(operations),
            origin_tainted,
        }
    }
    pub fn canonical(&self) -> Result<&FilterOperations, &FilterInputBlocker> {
        match self {
            Self::Ready { operations, .. } => Ok(operations),
            Self::Blocked(error) => Err(error),
        }
    }
    pub fn blocker(&self) -> Option<&FilterInputBlocker> {
        self.canonical().err()
    }
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Ready { operations, .. } if operations.IsEmpty())
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Ready { operations, .. } => operations.size(),
            Self::Blocked(error) => error.operation_count,
        }
    }
    pub fn origin_tainted(&self) -> bool {
        matches!(
            self,
            Self::Ready {
                origin_tainted: true,
                ..
            }
        )
    }
    pub fn is_finite(&self) -> bool {
        self.canonical().is_ok()
    }
    pub fn reference_output_bounds(&self, input: PaintRect) -> Option<PaintRect> {
        let operations = self.canonical().ok()?;
        if !operations.HasReferenceFilter() {
            return None;
        }
        let input = input.enclosing();
        let mapped = operations.MapRect(
            Rect::new(
                Point::new(input.x as i32, input.y as i32),
                Size::new(input.width as i32, input.height as i32),
            ),
            None,
        );
        Some(PaintRect {
            x: f64::from(mapped.x()),
            y: f64::from(mapped.y()),
            width: f64::from(mapped.width()),
            height: f64::from(mapped.height()),
        })
    }
    /// Reflection mask resources remain reachable from the executable
    /// reference-filter graph. A document mask that could not be precompiled
    /// stays reachable through its typed blocker as well.
    pub fn reflection_mask_layers(
        &self,
    ) -> &[layoutng_assembly::internal::paint_input::PaintReflectionMaskLayer] {
        match self {
            Self::Ready { operations, .. } => {
                for operation in operations.operations() {
                    if operation.r#type() != FilterType::REFERENCE {
                        continue;
                    }
                    let Some(filter) = operation.image_filter() else {
                        continue;
                    };
                    if let PaintFilterReplay::Reflection(source) = filter.ReplayGraph() {
                        return &source.reflection_mask_layers;
                    }
                }
                &[]
            }
            Self::Blocked(error) if error.source.r#type == PaintFilterType::kBoxReflect => {
                &error.source.reflection_mask_layers
            }
            _ => &[],
        }
    }

    /// Explicit legacy raster command boundary. The returned values are the
    /// newly emitted replay command payload, never cached on a property node.
    /// A blocked source owner propagates its typed error before any operation
    /// is emitted; it cannot become a partial or empty chain.
    pub fn replay_operations(&self) -> Result<Vec<PaintFilterOperation>, FilterReplayError> {
        self.canonical()
            .map_err(|e| FilterReplayError::Input(e.clone()))?
            .operations()
            .iter()
            .map(replay_operation)
            .collect()
    }
}

#[derive(Clone, Debug)]
struct CanvasPaintFilter {
    operation: PaintFilterOperation,
}

impl PaintFilter for CanvasPaintFilter {
    fn MapRect(
        &self,
        rect: Rect,
        ctm: Option<&dyn FilterMapTransform>,
        direction: MapDirection,
    ) -> Rect {
        let Some(PaintCanvasFilter::kConvolveMatrix {
            order_x,
            order_y,
            target_x,
            target_y,
            kernel,
            ..
        }) = self.operation.canvas_filter.as_ref()
        else {
            // FEComponentTransfer and FETurbulence use FilterEffect's identity
            // MapEffect. The previous input is still chained at
            // filter_effect_builder.cc:356-364.
            return rect;
        };
        let valid = *order_x > 0
            && *order_y > 0
            && i64::from(*order_x)
                .checked_mul(i64::from(*order_y))
                .and_then(|area| usize::try_from(area).ok())
                == Some(kernel.len())
            && *target_x >= 0
            && *target_y >= 0
            && *target_x < *order_x
            && *target_y < *order_y;
        if !valid {
            // FEConvolveMatrix::MapEffect returns its input when parameters are
            // invalid even though CreateImageFilter renders transparent black.
            return rect;
        }
        let (left, top, right, bottom) = match direction {
            MapDirection::Forward => (
                -*target_x,
                -*target_y,
                *order_x - *target_x,
                *order_y - *target_y,
            ),
            MapDirection::Reverse => (
                *target_x - *order_x,
                *target_y - *order_y,
                *target_x,
                *target_y,
            ),
        };
        let points = [
            (left as f32, top as f32),
            (right as f32, top as f32),
            (left as f32, bottom as f32),
            (right as f32, bottom as f32),
        ]
        .map(|(x, y)| ctm.map_or((x, y), |matrix| matrix.MapVector(x, y)));
        let left = points
            .iter()
            .map(|point| point.0)
            .fold(f32::INFINITY, f32::min)
            .floor() as i32;
        let top = points
            .iter()
            .map(|point| point.1)
            .fold(f32::INFINITY, f32::min)
            .floor() as i32;
        let right = points
            .iter()
            .map(|point| point.0)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil() as i32;
        let bottom = points
            .iter()
            .map(|point| point.1)
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil() as i32;
        Rect::new(
            Point::new(rect.x().saturating_add(left), rect.y().saturating_add(top)),
            Size::new(
                rect.width().saturating_add(right.saturating_sub(left)),
                rect.height().saturating_add(bottom.saturating_sub(top)),
            ),
        )
    }

    fn TypeToString(&self) -> &'static str {
        "CanvasFilter"
    }

    fn ReplayGraph(&self) -> PaintFilterReplay<'_> {
        PaintFilterReplay::Canvas(&self.operation)
    }
}

#[derive(Clone, Debug)]
struct ReferencePaintFilter {
    operation: PaintFilterOperation,
    reference_box: PaintRect,
    reference_viewport: Option<PaintRect>,
    reference_origin: Offset,
    reference_scale: f64,
    input_origin_tainted: bool,
    geometry: ResolvedSvgFilterGeometry,
}

impl PaintFilter for ReferencePaintFilter {
    fn MapRect(
        &self,
        rect: Rect,
        ctm: Option<&dyn FilterMapTransform>,
        direction: MapDirection,
    ) -> Rect {
        // Source: core/style/filter_operation.cc:45-51 and FilterEffect's
        // MapInputs -> MapEffect -> ApplyBounds chain. With no device CTM,
        // Chromium asks the Blink graph for exact forward local-space bounds.
        if direction == MapDirection::Forward && ctm.is_none() {
            let graph = self
                .operation
                .reference_graph
                .as_ref()
                .expect("canonical reference filter retains its SVG graph");
            let mapped = map_svg_filter_forward_bounds(
                graph,
                &self.geometry,
                PaintRect {
                    x: f64::from(rect.x()),
                    y: f64::from(rect.y()),
                    width: f64::from(rect.width()),
                    height: f64::from(rect.height()),
                },
            )
            .enclosing();
            return Rect::new(
                Point::new(mapped.x as i32, mapped.y as i32),
                Size::new(mapped.width as i32, mapped.height as i32),
            );
        }
        // Device-space forward and reverse mapping must ultimately match the
        // source SkImageFilter graph. Until that graph-level mapper is owned by
        // Paint, retain the explicit conservative filter crop for this path.
        let region = self.geometry.filter_region.enclosing();
        let points = [
            (region.x as f32, region.y as f32),
            ((region.x + region.width) as f32, region.y as f32),
            (region.x as f32, (region.y + region.height) as f32),
            (
                (region.x + region.width) as f32,
                (region.y + region.height) as f32,
            ),
        ];
        let points = points.map(|(x, y)| ctm.map_or((x, y), |matrix| matrix.MapVector(x, y)));
        let left = points
            .iter()
            .map(|point| point.0)
            .fold(f32::INFINITY, f32::min);
        let top = points
            .iter()
            .map(|point| point.1)
            .fold(f32::INFINITY, f32::min);
        let right = points
            .iter()
            .map(|point| point.0)
            .fold(f32::NEG_INFINITY, f32::max);
        let bottom = points
            .iter()
            .map(|point| point.1)
            .fold(f32::NEG_INFINITY, f32::max);
        Rect::new(
            Point::new(left.floor() as i32, top.floor() as i32),
            Size::new(
                (right.ceil() - left.floor()).max(0.0) as i32,
                (bottom.ceil() - top.floor()).max(0.0) as i32,
            ),
        )
    }

    fn TypeToString(&self) -> &'static str {
        "ReferenceFilter"
    }

    fn ReplayGraph(&self) -> PaintFilterReplay<'_> {
        PaintFilterReplay::Reference {
            operation: &self.operation,
            reference_box: self.reference_box,
            reference_viewport: self.reference_viewport,
            reference_origin: self.reference_origin,
            reference_scale: self.reference_scale,
            input_origin_tainted: self.input_origin_tainted,
        }
    }
}

#[derive(Clone, Debug)]
struct ReflectionPaintFilter {
    operation: PaintFilterOperation,
}

impl PaintFilter for ReflectionPaintFilter {
    fn MapRect(
        &self,
        rect: Rect,
        ctm: Option<&dyn FilterMapTransform>,
        _direction: MapDirection,
    ) -> Rect {
        // Source: platform/graphics/box_reflection.cc:32-53. PrePaint has
        // already resolved above/below/left/right into the source matrix's
        // vertical/horizontal axis and final translation.
        let left = rect.x() as f64;
        let top = rect.y() as f64;
        let right = left + rect.width() as f64;
        let bottom = top + rect.height() as f64;
        let vertical = matches!(
            self.operation.reflection_direction,
            layoutng_assembly::internal::paint_input::PaintReflectionDirection::kAbove
                | layoutng_assembly::internal::paint_input::PaintReflectionDirection::kBelow
        );
        let offset = ctm.map_or(self.operation.reflection_offset, |transform| {
            let mapped = if vertical {
                transform.MapVector(0.0, self.operation.reflection_offset as f32)
            } else {
                transform.MapVector(self.operation.reflection_offset as f32, 0.0)
            };
            f64::from(if vertical { mapped.1 } else { mapped.0 })
        });
        let (reflected_left, reflected_top, reflected_right, reflected_bottom) =
            match self.operation.reflection_direction {
                layoutng_assembly::internal::paint_input::PaintReflectionDirection::kAbove
                | layoutng_assembly::internal::paint_input::PaintReflectionDirection::kBelow => {
                    (left, offset - bottom, right, offset - top)
                }
                layoutng_assembly::internal::paint_input::PaintReflectionDirection::kLeft
                | layoutng_assembly::internal::paint_input::PaintReflectionDirection::kRight => {
                    (offset - right, top, offset - left, bottom)
                }
            };
        let union_left = left.min(reflected_left).floor();
        let union_top = top.min(reflected_top).floor();
        let union_right = right.max(reflected_right).ceil();
        let union_bottom = bottom.max(reflected_bottom).ceil();
        Rect::new(
            Point::new(union_left as i32, union_top as i32),
            Size::new(
                (union_right - union_left).max(0.0) as i32,
                (union_bottom - union_top).max(0.0) as i32,
            ),
        )
    }

    fn TypeToString(&self) -> &'static str {
        "BoxReflectionFilter"
    }

    fn ReplayGraph(&self) -> PaintFilterReplay<'_> {
        PaintFilterReplay::Reflection(&self.operation)
    }
}

fn byte_color(value: f32) -> f32 {
    ((value.clamp(0., 1.) * 255.).round() as u8) as f32 / 255.
}
#[derive(Clone, Debug, PartialEq)]
pub enum FilterReplayError {
    Input(FilterInputBlocker),
    UnsupportedCanonicalType(FilterType),
}
impl fmt::Display for FilterReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(e) => e.fmt(f),
            Self::UnsupportedCanonicalType(t) => {
                write!(f, "legacy raster cannot replay canonical {t:?}")
            }
        }
    }
}
impl std::error::Error for FilterReplayError {}
impl From<FilterReplayError> for std::io::Error {
    fn from(e: FilterReplayError) -> Self {
        Self::new(std::io::ErrorKind::Unsupported, e)
    }
}
fn replay_operation(op: &FilterOperation) -> Result<PaintFilterOperation, FilterReplayError> {
    use FilterType::*;
    let mut value = PaintFilterOperation::default();
    value.r#type = match op.r#type() {
        GRAYSCALE => PaintFilterType::kGrayscale,
        SEPIA => PaintFilterType::kSepia,
        SATURATE => PaintFilterType::kSaturate,
        HUE_ROTATE => PaintFilterType::kHueRotate,
        INVERT => PaintFilterType::kInvert,
        BRIGHTNESS => PaintFilterType::kBrightness,
        CONTRAST => PaintFilterType::kContrast,
        OPACITY => PaintFilterType::kOpacity,
        BLUR => PaintFilterType::kBlur,
        DROP_SHADOW => PaintFilterType::kDropShadow,
        COLOR_MATRIX => PaintFilterType::kColorMatrix,
        REFERENCE => {
            let Some(filter) = op.image_filter() else {
                return Err(FilterReplayError::UnsupportedCanonicalType(REFERENCE));
            };
            if let PaintFilterReplay::Canvas(source) = filter.ReplayGraph() {
                return Ok(source.clone());
            }
            return Err(FilterReplayError::UnsupportedCanonicalType(REFERENCE));
        }
        kind => return Err(FilterReplayError::UnsupportedCanonicalType(kind)),
    };
    if op.r#type() == COLOR_MATRIX {
        value.color_matrix = op.matrix().iter().map(|value| f64::from(*value)).collect();
    } else {
        value.amount = f64::from(op.amount());
    }
    if op.r#type() == DROP_SHADOW {
        let offset = op.offset();
        value.offset = Offset {
            x: f64::from(offset.x()),
            y: f64::from(offset.y()),
        };
        value.blur_radius = f64::from(op.amount());
        let color = op.drop_shadow_color();
        value.color = Color {
            red: color.red,
            green: color.green,
            blue: color.blue,
            alpha: color.alpha,
        };
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user_space_reference(region: SvgFilterRegion) -> PaintFilterOperation {
        PaintFilterOperation {
            r#type: PaintFilterType::kReference,
            reference_graph: Some(Arc::new(SvgFilterGraph {
                resource_node_id:0,
                id: "viewport-filter".into(),
                filter_units: SvgFilterUnits::kUserSpaceOnUse,
                primitive_units: SvgFilterUnits::kUserSpaceOnUse,
                region,
                primitives: vec![
                    layoutng_assembly::internal::paint_input::SvgFilterPrimitive {
                        input: layoutng_assembly::internal::paint_input::SvgFilterInput::kSourceGraphic,
                        result: String::new(),
                        region: SvgFilterRegion::default(),
                        linear_rgb: false,
                        origin_tainted: false,
                        kind: layoutng_assembly::internal::paint_input::SvgFilterPrimitiveKind::kOffset {
                            dx: 0.0,
                            dy: 0.0,
                        },
                    },
                ],
                fill_paint: None,
                stroke_paint: None,
                unsupported: None,
            })),
            ..Default::default()
        }
    }

    #[test]
    fn user_space_percentages_resolve_against_filter_viewport() {
        let percent = |value| {
            Some(SvgFilterLength {
                value,
                percentage: true,
            })
        };
        let operation = user_space_reference(SvgFilterRegion {
            x: percent(50.0),
            y: percent(25.0),
            width: percent(50.0),
            height: percent(50.0),
        });
        let reference = PaintRect {
            x: 10.0,
            y: 20.0,
            width: 40.0,
            height: 60.0,
        };
        let viewport = PaintRect {
            x: 3.0,
            y: 4.0,
            width: 200.0,
            height: 100.0,
        };
        let filters = PropertyFilterOperations::from_authored(
            std::slice::from_ref(&operation),
            Some(reference),
            Some(viewport),
            Offset {
                x: viewport.x,
                y: viewport.y,
            },
            1.0,
        );
        let geometry = resolve_svg_filter_geometry(
            operation
                .reference_graph
                .as_deref()
                .expect("reference graph"),
            reference,
            Some(viewport),
            1.0,
        )
        .expect("resolved geometry");
        assert_eq!(
            geometry.filter_region,
            PaintRect {
                x: 100.0,
                y: 25.0,
                width: 100.0,
                height: 50.0,
            }
        );
        assert_eq!(
            filters.reference_output_bounds(reference),
            Some(PaintRect::default()),
            "an out-of-input filter region does not manufacture output"
        );
    }

    #[test]
    fn missing_user_space_viewport_is_explicitly_blocked() {
        let operation = user_space_reference(SvgFilterRegion::default());
        let filters = PropertyFilterOperations::from_authored(
            &[operation],
            Some(PaintRect::default()),
            None,
            Offset::default(),
            1.0,
        );
        assert_eq!(
            filters.blocker().map(|blocker| &blocker.reason),
            Some(&FilterInputBlockReason::ReferenceViewportNotLowered)
        );
    }

    #[test]
    fn absolute_user_space_lengths_use_filter_zoom_without_an_invented_origin() {
        let length = |value| {
            Some(SvgFilterLength {
                value,
                percentage: false,
            })
        };
        let operation = user_space_reference(SvgFilterRegion {
            x: length(5.0),
            y: length(6.0),
            width: length(10.0),
            height: length(11.0),
        });
        let reference = PaintRect::default();
        let filters = PropertyFilterOperations::from_authored(
            std::slice::from_ref(&operation),
            Some(reference),
            None,
            Offset { x: 7.0, y: 9.0 },
            2.0,
        );
        let geometry = resolve_svg_filter_geometry(
            operation
                .reference_graph
                .as_deref()
                .expect("reference graph"),
            reference,
            None,
            2.0,
        )
        .expect("resolved geometry");
        assert_eq!(
            geometry.filter_region,
            PaintRect {
                x: 10.0,
                y: 12.0,
                width: 20.0,
                height: 22.0,
            }
        );
        assert_eq!(
            filters.reference_output_bounds(reference),
            Some(PaintRect::default()),
            "an empty source stays empty after the zero-offset primitive"
        );
    }

    #[test]
    fn unresolved_reference_is_skipped_without_dropping_later_filters() {
        let unresolved = PaintFilterOperation {
            r#type: PaintFilterType::kReference,
            reference_url: Some("#missing".into()),
            reference_error: Some("missing local SVG filter #missing".into()),
            ..Default::default()
        };
        let opacity = PaintFilterOperation {
            r#type: PaintFilterType::kOpacity,
            amount: 0.25,
            ..Default::default()
        };
        let filters = PropertyFilterOperations::from_authored(
            &[unresolved, opacity],
            None,
            None,
            Offset::default(),
            1.0,
        );
        let operations = filters
            .canonical()
            .expect("a source invalid resource is not a lowering blocker");
        assert_eq!(operations.size(), 1);
        assert_eq!(operations.at(0).r#type(), FilterType::OPACITY);
        assert_eq!(operations.at(0).amount(), 0.25);
    }

    #[test]
    fn empty_reference_graph_and_region_append_no_operation() {
        let opacity = PaintFilterOperation {
            r#type: PaintFilterType::kOpacity,
            amount: 0.5,
            ..Default::default()
        };
        let mut empty_graph = user_space_reference(SvgFilterRegion::default());
        Arc::make_mut(
            empty_graph
                .reference_graph
                .as_mut()
                .expect("test reference graph"),
        )
        .primitives
        .clear();
        let empty_region = user_space_reference(SvgFilterRegion {
            width: Some(SvgFilterLength {
                value: 0.0,
                percentage: false,
            }),
            ..Default::default()
        });
        let reference = PaintRect {
            width: 40.0,
            height: 30.0,
            ..Default::default()
        };

        for reference_filter in [empty_graph, empty_region] {
            let filters = PropertyFilterOperations::from_authored(
                &[reference_filter, opacity.clone()],
                Some(reference),
                Some(reference),
                Offset::default(),
                1.0,
            );
            let operations = filters
                .canonical()
                .expect("source empty references are not blockers");
            assert_eq!(operations.size(), 1);
            assert_eq!(operations.at(0).r#type(), FilterType::OPACITY);
        }
    }

    #[test]
    fn unsupported_resolved_reference_remains_explicitly_blocked() {
        let mut operation = user_space_reference(SvgFilterRegion::default());
        operation.reference_error = Some("unsupported primitive".into());
        let filters = PropertyFilterOperations::from_authored(
            &[operation],
            Some(PaintRect::default()),
            Some(PaintRect::default()),
            Offset::default(),
            1.0,
        );
        assert_eq!(
            filters.blocker().map(|blocker| &blocker.reason),
            Some(&FilterInputBlockReason::ReferenceOwnerNotLowered)
        );
    }

    #[test]
    fn current_color_drop_shadow_marks_canonical_chain_origin_tainted() {
        let mut shadow = PaintFilterOperation {
            r#type: PaintFilterType::kDropShadow,
            uses_current_color: true,
            ..Default::default()
        };
        let filters = PropertyFilterOperations::from_authored(
            &[shadow.clone()],
            None,
            None,
            Offset::default(),
            1.0,
        );
        assert!(filters.origin_tainted());

        shadow.uses_current_color = false;
        let filters =
            PropertyFilterOperations::from_authored(&[shadow], None, None, Offset::default(), 1.0);
        assert!(!filters.origin_tainted());
    }

    #[test]
    fn explicit_input_taint_seeds_the_chain_and_reference_source() {
        let reference = PaintRect {
            width: 10.0,
            height: 10.0,
            ..Default::default()
        };
        let filters = PropertyFilterOperations::from_authored_with_input_taint(
            &[
                PaintFilterOperation {
                    r#type: PaintFilterType::kOpacity,
                    amount: 0.5,
                    ..Default::default()
                },
                user_space_reference(SvgFilterRegion::default()),
            ],
            Some(reference),
            Some(reference),
            Offset::default(),
            1.0,
            true,
        );
        assert!(filters.origin_tainted());
        let operations = filters
            .canonical()
            .expect("tainted chain remains executable");
        let reference = operations
            .at(1)
            .image_filter()
            .as_ref()
            .expect("reference operation");
        assert!(matches!(
            reference.ReplayGraph(),
            PaintFilterReplay::Reference {
                input_origin_tainted: true,
                ..
            }
        ));
    }

    #[test]
    fn canvas_color_matrices_use_fe_color_matrix_defaults() {
        let mut values = vec![0.0; 20];
        values[0] = 0.5;
        values[6] = 0.6;
        values[12] = 0.7;
        values[18] = 0.8;
        let filters = PropertyFilterOperations::from_authored_with_input_taint(
            &[
                PaintFilterOperation {
                    r#type: PaintFilterType::kLuminanceToAlpha,
                    ..Default::default()
                },
                PaintFilterOperation {
                    r#type: PaintFilterType::kColorMatrix,
                    color_matrix: values.clone(),
                    ..Default::default()
                },
                PaintFilterOperation {
                    r#type: PaintFilterType::kColorMatrix,
                    color_matrix: vec![1.0; 19],
                    ..Default::default()
                },
            ],
            None,
            None,
            Offset::default(),
            1.0,
            false,
        );
        let operations = filters.canonical().expect("canvas matrices are canonical");
        assert_eq!(operations.at(0).matrix(), &luminance_to_alpha_matrix());
        assert_eq!(operations.at(1).matrix(), &generic_color_matrix(&values));
        assert_eq!(operations.at(2).matrix(), &generic_color_matrix(&[]));
        let replay = filters.replay_operations().expect("matrix replay");
        assert!(replay
            .iter()
            .all(|operation| operation.r#type == PaintFilterType::kColorMatrix));
        assert_eq!(replay[0].color_matrix.len(), 20);
    }

    #[test]
    fn svg_reference_propagates_only_the_last_effect_origin_taint() {
        let primitive = |result: &str, origin_tainted, kind| {
            layoutng_assembly::internal::paint_input::SvgFilterPrimitive {
                input: SvgFilterInput::kSourceGraphic,
                result: result.into(),
                region: SvgFilterRegion::default(),
                linear_rgb: false,
                origin_tainted,
                kind,
            }
        };
        let mut graph = SvgFilterGraph {
            resource_node_id: 0,
            id: "taint".into(),
            filter_units: SvgFilterUnits::kUserSpaceOnUse,
            primitive_units: SvgFilterUnits::kUserSpaceOnUse,
            region: SvgFilterRegion::default(),
            primitives: vec![primitive(
                "tainted",
                true,
                SvgFilterPrimitiveKind::kFlood {
                    color: Color::default(),
                },
            )],
            fill_paint: None,
            stroke_paint: None,
            unsupported: None,
        };
        assert!(svg_filter_origin_tainted(&graph, false));
        graph.primitives.push(primitive(
            "clean",
            false,
            SvgFilterPrimitiveKind::kTurbulence {
                base_x: 0.0,
                base_y: 0.0,
                octaves: 1,
                seed: 0.0,
                stitch: false,
                kind: layoutng_assembly::internal::paint_input::SvgTurbulenceType::kTurbulence,
            },
        ));
        assert!(!svg_filter_origin_tainted(&graph, false));

        let filters = PropertyFilterOperations::from_authored(
            &[PaintFilterOperation {
                r#type: PaintFilterType::kReference,
                reference_graph: Some(Arc::new(graph)),
                ..Default::default()
            }],
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            Offset::default(),
            1.0,
        );
        assert!(filters.blocker().is_none(), "test graph must be canonical");
        assert!(!filters.origin_tainted());
    }

    #[test]
    fn svg_result_keywords_and_missing_names_follow_builder_fallbacks() {
        let primitive = |input, result: &str, origin_tainted| {
            layoutng_assembly::internal::paint_input::SvgFilterPrimitive {
                input,
                result: result.into(),
                region: SvgFilterRegion::default(),
                linear_rgb: false,
                origin_tainted,
                kind: SvgFilterPrimitiveKind::kOffset { dx: 0.0, dy: 0.0 },
            }
        };
        let graph = SvgFilterGraph {
            resource_node_id: 0,
            id: "builder-fallbacks".into(),
            filter_units: SvgFilterUnits::kUserSpaceOnUse,
            primitive_units: SvgFilterUnits::kUserSpaceOnUse,
            region: SvgFilterRegion::default(),
            primitives: vec![primitive(
                SvgFilterInput::kSourceGraphic,
                "SourceGraphic",
                true,
            )],
            fill_paint: None,
            stroke_paint: None,
            unsupported: None,
        };
        assert!(
            !svg_filter_origin_tainted(&graph, false),
            "a result that collides with a built-in must not replace LastEffect"
        );
        let filters = PropertyFilterOperations::from_authored(
            &[PaintFilterOperation {
                r#type: PaintFilterType::kReference,
                reference_graph: Some(Arc::new(graph.clone())),
                ..Default::default()
            }],
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            Some(PaintRect {
                width: 10.0,
                height: 10.0,
                ..Default::default()
            }),
            Offset::default(),
            1.0,
        );
        assert!(
            filters.is_empty(),
            "a graph with no accepted LastEffect appends no reference operation"
        );

        let mut graph = graph;
        graph.primitives = vec![
            primitive(SvgFilterInput::kSourceGraphic, "previous", true),
            primitive(SvgFilterInput::kNamed("missing".into()), "output", false),
        ];
        assert!(
            svg_filter_origin_tainted(&graph, false),
            "an unknown named input falls back to LastEffect"
        );
    }

    #[test]
    fn box_reflection_is_an_executable_reference_filter_with_source_bounds() {
        let reflection = PaintFilterOperation {
            r#type: PaintFilterType::kBoxReflect,
            reflection_direction:
                layoutng_assembly::internal::paint_input::PaintReflectionDirection::kBelow,
            reflection_offset: 14.0,
            ..Default::default()
        };
        let filters = PropertyFilterOperations::from_authored(
            &[reflection],
            None,
            None,
            Offset::default(),
            1.0,
        );
        let operations = filters.canonical().expect("reflection is executable");
        assert_eq!(operations.size(), 1);
        assert_eq!(operations.at(0).r#type(), FilterType::REFERENCE);
        assert_eq!(
            filters.reference_output_bounds(PaintRect {
                x: 2.0,
                y: 3.0,
                width: 4.0,
                height: 5.0,
            }),
            Some(PaintRect {
                x: 2.0,
                y: 3.0,
                width: 4.0,
                height: 8.0,
            })
        );
    }

    #[test]
    fn box_reflection_bounds_map_the_offset_through_the_device_scale() {
        #[derive(Debug)]
        struct Scale(f32);
        impl FilterMapTransform for Scale {
            fn MapVector(&self, x: f32, y: f32) -> (f32, f32) {
                (x * self.0, y * self.0)
            }
        }

        let filters = PropertyFilterOperations::from_authored(
            &[PaintFilterOperation {
                r#type: PaintFilterType::kBoxReflect,
                reflection_offset: 14.0,
                ..Default::default()
            }],
            None,
            None,
            Offset::default(),
            1.0,
        );
        let operations = filters.canonical().expect("reflection is executable");
        let image_filter = operations
            .at(0)
            .image_filter()
            .as_deref()
            .expect("reflection graph");
        assert_eq!(
            image_filter.MapRect(
                Rect::new(Point::new(4, 6), Size::new(8, 10)),
                Some(&Scale(2.0)),
                MapDirection::Forward,
            ),
            Rect::new(Point::new(4, 6), Size::new(8, 16))
        );
    }

    #[test]
    fn box_reflection_keeps_resolved_bitmap_mask_layers_reachable() {
        let mask = layoutng_assembly::internal::paint_input::PaintReflectionMaskLayer {
            image: Some(image_resource::PaintImage {
                width: 1,
                height: 1,
                content: image_resource::PaintImageContent::Bitmap(Arc::new(vec![
                    0xff, 0xff, 0xff, 0xff,
                ])),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reflection = PaintFilterOperation {
            r#type: PaintFilterType::kBoxReflect,
            reflection_has_mask: true,
            reflection_mask_layers: vec![mask.clone()],
            ..Default::default()
        };
        let filters = PropertyFilterOperations::from_authored(
            &[reflection],
            None,
            None,
            Offset::default(),
            1.0,
        );
        assert_eq!(filters.reflection_mask_layers(), &[mask]);
    }

    #[test]
    fn document_reflection_mask_remains_an_explicit_typed_blocker() {
        let key = image_resource::ContainerKey::new(1, 1, 1.0, 1.0);
        let mask = layoutng_assembly::internal::paint_input::PaintReflectionMaskLayer {
            image: Some(image_resource::PaintImage {
                width: 1,
                height: 1,
                document_container_key: Some(key.clone()),
                content: image_resource::PaintImageContent::Document(Arc::new(
                    image_resource::DocumentPaintContent::new(key, Arc::new("record")),
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reflection = PaintFilterOperation {
            r#type: PaintFilterType::kBoxReflect,
            reflection_has_mask: true,
            reflection_mask_layers: vec![mask],
            ..Default::default()
        };
        let filters = PropertyFilterOperations::from_authored(
            &[reflection],
            None,
            None,
            Offset::default(),
            1.0,
        );
        assert_eq!(
            filters.blocker().map(|blocker| &blocker.reason),
            Some(&FilterInputBlockReason::ReflectionMaskRecordNotLowered)
        );
        assert_eq!(filters.reflection_mask_layers().len(), 1);
    }

    #[test]
    fn canvas_effect_classes_keep_typed_payload_bounds_and_replay() {
        use layoutng_assembly::internal::paint_input::{
            PaintComponentTransferFunction, PaintComponentTransferType, PaintConvolveEdgeMode,
            PaintTurbulenceType,
        };

        let identity = PaintComponentTransferFunction {
            r#type: PaintComponentTransferType::kIdentity,
            slope: 0.0,
            intercept: 0.0,
            amplitude: 0.0,
            exponent: 0.0,
            offset: 0.0,
            table_values: Vec::new(),
        };
        let authored = [
            PaintFilterOperation {
                r#type: PaintFilterType::kConvolveMatrix,
                canvas_filter: Some(PaintCanvasFilter::kConvolveMatrix {
                    order_x: 3,
                    order_y: 2,
                    divisor: 0.0,
                    bias: 0.25,
                    target_x: 1,
                    target_y: 0,
                    edge_mode: PaintConvolveEdgeMode::kUnknown,
                    preserve_alpha: true,
                    kernel: vec![1.0; 6],
                }),
                ..Default::default()
            },
            PaintFilterOperation {
                r#type: PaintFilterType::kComponentTransfer,
                canvas_filter: Some(PaintCanvasFilter::kComponentTransfer {
                    functions: std::array::from_fn(|_| identity.clone()),
                }),
                ..Default::default()
            },
            PaintFilterOperation {
                r#type: PaintFilterType::kTurbulence,
                canvas_filter: Some(PaintCanvasFilter::kTurbulence {
                    kind: PaintTurbulenceType::kFractalNoise,
                    base_x: 0.1,
                    base_y: 0.2,
                    octaves: 4,
                    seed: 3.0,
                    stitch: true,
                }),
                ..Default::default()
            },
        ];
        let reference_box = PaintRect {
            width: 40.0,
            height: 20.0,
            ..Default::default()
        };
        let filters = PropertyFilterOperations::from_authored(
            &authored,
            Some(reference_box),
            None,
            Offset::default(),
            1.0,
        );
        let canonical = filters.canonical().expect("typed Canvas filters");
        assert_eq!(canonical.size(), 3);
        assert!(canonical
            .operations()
            .iter()
            .all(|operation| operation.r#type() == FilterType::REFERENCE));
        assert_eq!(
            canonical
                .at(0)
                .image_filter()
                .as_deref()
                .expect("Canvas convolution")
                .MapRect(
                    Rect::new(Point::new(10, 20), Size::new(30, 40)),
                    None,
                    MapDirection::Forward,
                ),
            Rect::new(Point::new(9, 20), Size::new(33, 42))
        );
        let replay = filters.replay_operations().expect("Canvas command replay");
        for (replayed, source) in replay.iter().zip(&authored) {
            assert_eq!(replayed.r#type, source.r#type);
            assert_eq!(replayed.canvas_filter, source.canvas_filter);
        }
    }
}
