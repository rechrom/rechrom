//! Resolved SVG filter geometry and Blink-compatible forward bounds.
//!
//! Layout and Paint borrow the same immutable authored resource graph. This
//! shared source geometry owner resolves reference box/viewport/filter scale;
//! Raster consumes these same values when building source image filters.

use crate::internal::paint_input::{
    SvgCompositeOperator, SvgFilterGraph, SvgFilterInput, SvgFilterLength, SvgFilterPrimitive,
    SvgFilterPrimitiveKind, SvgFilterRegion, SvgFilterUnits, SvgTransferFunction,
};
use foundation::paint_rect::PaintRect;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgFilterGeometryError {
    MissingViewport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedSvgFilterPrimitiveGeometry {
    pub source_index: usize,
    pub region: PaintRect,
    pub tile_source_region: Option<PaintRect>,
    local_region: BlinkRectF,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedSvgFilterGeometry {
    pub filter_region: PaintRect,
    pub primitives: Vec<ResolvedSvgFilterPrimitiveGeometry>,
    local_filter_region: BlinkRectF,
    primitive_reference_box: Option<BlinkRectF>,
    filter_scale: f32,
}

impl ResolvedSvgFilterGeometry {
    /// Source: filter.cc:57-63.
    pub fn map_local_rect_to_absolute(&self, rect: PaintRect) -> PaintRect {
        BlinkRectF::from_paint(rect)
            .scale(self.filter_scale, self.filter_scale)
            .to_paint()
    }

    pub fn map_absolute_rect_to_local(&self, rect: PaintRect) -> PaintRect {
        let inverse = 1.0_f32 / self.filter_scale;
        BlinkRectF::from_paint(rect)
            .scale(inverse, inverse)
            .to_paint()
    }

    /// `Filter::ApplyHorizontalScale/ApplyVerticalScale` multiply in this
    /// order, using `float` at each step (filter.cc:65-75).
    pub fn scale_primitive_x(&self, value: f64) -> f32 {
        let mut value = value as f32;
        if let Some(reference) = self.primitive_reference_box {
            value *= reference.width;
        }
        self.filter_scale * value
    }

    pub fn scale_primitive_y(&self, value: f64) -> f32 {
        let mut value = value as f32;
        if let Some(reference) = self.primitive_reference_box {
            value *= reference.height;
        }
        self.filter_scale * value
    }

    /// Source: filter.cc:77-87 and vector2d_f.cc:39-41.
    pub fn resolve_3d_point(&self, x: f64, y: f64, z: f64) -> [f32; 3] {
        let (mut x, mut y, mut z) = (x as f32, y as f32, z as f32);
        if let Some(reference) = self.primitive_reference_box {
            x = x * reference.width + reference.x;
            y = y * reference.height + reference.y;
            let squared = f64::from(reference.width) * f64::from(reference.width)
                + f64::from(reference.height) * f64::from(reference.height);
            z *= ((squared / 2.0) as f32).sqrt();
        }
        [
            x * self.filter_scale,
            y * self.filter_scale,
            z * self.filter_scale,
        ]
    }

    fn absolute_crop(&self, primitive: BlinkRectF) -> BlinkRectF {
        self.local_filter_region
            .intersect(primitive)
            .scale(self.filter_scale, self.filter_scale)
    }
}

// gfx::RectF stores a PointF plus a SizeF. SizeF clamps non-positive, NaN and
// sub-epsilon dimensions to zero, and RectF::Union may advance a rounded-down
// extent to the next representable float. Keep those semantics private to the
// Chromium boundary while the package-facing PaintRect remains f64.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BlinkRectF {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl BlinkRectF {
    const TRIVIAL: f32 = 8.0 * f32::EPSILON;

    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: Self::clamp_size(width),
            height: Self::clamp_size(height),
        }
    }

    fn from_paint(rect: PaintRect) -> Self {
        Self::new(
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        )
    }

    fn to_paint(self) -> PaintRect {
        PaintRect {
            x: f64::from(self.x),
            y: f64::from(self.y),
            width: f64::from(self.width),
            height: f64::from(self.height),
        }
    }

    fn clamp_size(value: f32) -> f32 {
        if value > Self::TRIVIAL {
            value
        } else {
            0.0
        }
    }

    fn is_empty(self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }

    fn right(self) -> f32 {
        self.x + self.width
    }

    fn bottom(self) -> f32 {
        self.y + self.height
    }

    fn set_x(&mut self, x: f32) {
        self.x = x;
    }

    fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    fn set_width(&mut self, width: f32) {
        self.width = Self::clamp_size(width);
    }

    fn set_height(&mut self, height: f32) {
        self.height = Self::clamp_size(height);
    }

    fn scale(self, x_scale: f32, y_scale: f32) -> Self {
        Self::new(
            self.x * x_scale,
            self.y * y_scale,
            self.width * x_scale,
            self.height * y_scale,
        )
    }

    fn intersect(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::default();
        }
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if x >= right || y >= bottom {
            Self::default()
        } else {
            Self::new(x, y, right - x, bottom - y)
        }
    }

    fn union(&mut self, other: Self) {
        if self.is_empty() {
            *self = other;
            return;
        }
        if other.is_empty() {
            return;
        }
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        *self = Self::new(x, y, right - x, bottom - y);
        if self.right() < right && self.width < f32::MAX {
            self.width = next_greater_float(self.width.max(Self::TRIVIAL));
        }
        if self.bottom() < bottom && self.height < f32::MAX {
            self.height = next_greater_float(self.height.max(Self::TRIVIAL));
        }
    }

    fn offset(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }

    fn outset(&mut self, x: f32, y: f32) {
        self.x += -x;
        self.y += -y;
        self.set_width(self.width - (-x + -x));
        self.set_height(self.height - (-y + -y));
    }
}

fn next_greater_float(value: f32) -> f32 {
    if value == f32::INFINITY {
        value
    } else if value == -0.0 {
        f32::from_bits(1)
    } else if value >= 0.0 {
        f32::from_bits(value.to_bits() + 1)
    } else {
        f32::from_bits(value.to_bits() - 1)
    }
}

fn region_uses_viewport(region: &SvgFilterRegion, defaults_are_percentages: bool) -> bool {
    [region.x, region.y, region.width, region.height]
        .iter()
        .any(|value| match value {
            Some(value) => value.percentage,
            None => defaults_are_percentages,
        })
}

pub fn graph_requires_viewport(graph: &SvgFilterGraph) -> bool {
    (graph.filter_units == SvgFilterUnits::kUserSpaceOnUse
        && region_uses_viewport(&graph.region, true))
        || (graph.primitive_units == SvgFilterUnits::kUserSpaceOnUse
            && graph
                .primitives
                .iter()
                .any(|primitive| region_uses_viewport(&primitive.region, false)))
}

// `None` denotes a standard source input. Per
// SVGFilterPrimitiveStandardAttributes::DefaultFilterPrimitiveSubregion, one
// such input makes the whole default subregion equal to the filter region.
fn input_subregion(
    input: &SvgFilterInput,
    previous: Option<BlinkRectF>,
    named: &HashMap<&str, BlinkRectF>,
    fill_paint_installed: bool,
    stroke_paint_installed: bool,
) -> Option<BlinkRectF> {
    match input {
        SvgFilterInput::kSourceGraphic | SvgFilterInput::kSourceAlpha => None,
        SvgFilterInput::kFillPaint if fill_paint_installed => None,
        SvgFilterInput::kStrokePaint if stroke_paint_installed => None,
        SvgFilterInput::kNamed(name) if !name.is_empty() => {
            named.get(name.as_str()).copied().or(previous)
        }
        SvgFilterInput::kFillPaint | SvgFilterInput::kStrokePaint | SvgFilterInput::kNamed(_) => {
            previous
        }
    }
}

fn default_primitive_subregion(
    primitive: &SvgFilterPrimitive,
    filter: BlinkRectF,
    previous: Option<BlinkRectF>,
    named: &HashMap<&str, BlinkRectF>,
    fill_paint_installed: bool,
    stroke_paint_installed: bool,
) -> BlinkRectF {
    use SvgFilterPrimitiveKind::*;
    if matches!(
        primitive.kind,
        kFlood { .. } | kTurbulence { .. } | kImage { .. } | kTile
    ) {
        return filter;
    }
    let mut union = BlinkRectF::default();
    let mut count = 0;
    let mut add = |input: &SvgFilterInput| {
        count += 1;
        let Some(region) = input_subregion(
            input,
            previous,
            named,
            fill_paint_installed,
            stroke_paint_installed,
        ) else {
            return false;
        };
        union.union(region);
        true
    };
    let inherited = match &primitive.kind {
        kMerge { inputs } => inputs.iter().all(&mut add),
        kBlend { input2, .. } | kComposite { input2, .. } | kDisplacementMap { input2, .. } => {
            add(&primitive.input) && add(input2)
        }
        _ => add(&primitive.input),
    };
    if inherited && count != 0 {
        union
    } else {
        filter
    }
}

/// Sources: layout_svg_resource_container.cc:94-137,
/// svg_filter_primitive_standard_attributes.cc:116-171 and filter.cc:57-74.
pub fn resolve_svg_filter_geometry(
    graph: &SvgFilterGraph,
    reference: PaintRect,
    reference_viewport: Option<PaintRect>,
    filter_scale: f64,
) -> Result<ResolvedSvgFilterGeometry, SvgFilterGeometryError> {
    if reference_viewport.is_none() && graph_requires_viewport(graph) {
        return Err(SvgFilterGeometryError::MissingViewport);
    }
    let reference = BlinkRectF::from_paint(reference);
    let viewport = reference_viewport.map(BlinkRectF::from_paint);
    let (vw, vh) = viewport.map_or((0.0, 0.0), |viewport| (viewport.width, viewport.height));
    let resolve_position = |value: Option<SvgFilterLength>,
                            reference_origin: f32,
                            axis: f32,
                            units: SvgFilterUnits,
                            default: f32| {
        value.map_or(default, |value| {
            if value.percentage {
                reference_origin + axis * value.value as f32 / 100.0
            } else if units == SvgFilterUnits::kObjectBoundingBox {
                reference_origin + value.value as f32 * axis
            } else {
                value.value as f32
            }
        })
    };
    let resolve_length =
        |value: Option<SvgFilterLength>, axis: f32, units: SvgFilterUnits, default: f32| {
            value.map_or(default, |value| {
                if value.percentage {
                    axis * value.value as f32 / 100.0
                } else if units == SvgFilterUnits::kObjectBoundingBox {
                    value.value as f32 * axis
                } else {
                    value.value as f32
                }
            })
        };
    let resolve_region = |region: &SvgFilterRegion, units: SvgFilterUnits, defaults: BlinkRectF| {
        let (origin_x, origin_y, axis_width, axis_height) = match units {
            SvgFilterUnits::kObjectBoundingBox => {
                (reference.x, reference.y, reference.width, reference.height)
            }
            SvgFilterUnits::kUserSpaceOnUse => (0.0, 0.0, vw, vh),
        };
        BlinkRectF::new(
            resolve_position(region.x, origin_x, axis_width, units, defaults.x),
            resolve_position(region.y, origin_y, axis_height, units, defaults.y),
            resolve_length(region.width, axis_width, units, defaults.width),
            resolve_length(region.height, axis_height, units, defaults.height),
        )
    };
    let (filter_x, filter_y, filter_width, filter_height) = match graph.filter_units {
        SvgFilterUnits::kObjectBoundingBox => {
            (reference.x, reference.y, reference.width, reference.height)
        }
        SvgFilterUnits::kUserSpaceOnUse => (0.0, 0.0, vw, vh),
    };
    // The SVG initial values are -10%, -10%, 120%, 120%. Preserve the same
    // multiply-then-divide float evaluation as FloatValueForLength.
    let local_filter_region = resolve_region(
        &graph.region,
        graph.filter_units,
        BlinkRectF::new(
            filter_x + filter_width * -10.0 / 100.0,
            filter_y + filter_height * -10.0 / 100.0,
            filter_width * 120.0 / 100.0,
            filter_height * 120.0 / 100.0,
        ),
    );
    let filter_scale = filter_scale as f32;
    let filter_region = local_filter_region.scale(filter_scale, filter_scale);
    let mut primitives = Vec::with_capacity(graph.primitives.len());
    let mut named_regions = HashMap::new();
    let mut previous_region = None;
    for (source_index, primitive) in graph.primitives.iter().enumerate() {
        let defaults = default_primitive_subregion(
            primitive,
            local_filter_region,
            previous_region,
            &named_regions,
            graph.fill_paint.is_some(),
            graph.stroke_paint.is_some(),
        );
        let region = resolve_region(&primitive.region, graph.primitive_units, defaults);
        let tile_source_region =
            matches!(primitive.kind, SvgFilterPrimitiveKind::kTile).then(|| {
                input_subregion(
                    &primitive.input,
                    previous_region,
                    &named_regions,
                    graph.fill_paint.is_some(),
                    graph.stroke_paint.is_some(),
                )
                .unwrap_or(local_filter_region)
            });
        let result_is_builtin = primitive.result == "SourceGraphic"
            || primitive.result == "SourceAlpha"
            || (primitive.result == "FillPaint" && graph.fill_paint.is_some())
            || (primitive.result == "StrokePaint" && graph.stroke_paint.is_some());
        if result_is_builtin {
            continue;
        }
        if !primitive.result.is_empty() {
            named_regions.insert(primitive.result.as_str(), region);
        }
        previous_region = Some(region);
        primitives.push(ResolvedSvgFilterPrimitiveGeometry {
            source_index,
            region: region.scale(filter_scale, filter_scale).to_paint(),
            tile_source_region: tile_source_region
                .map(|region| region.scale(filter_scale, filter_scale).to_paint()),
            local_region: region,
        });
    }
    let primitive_reference_box =
        (graph.primitive_units == SvgFilterUnits::kObjectBoundingBox).then_some(reference);
    Ok(ResolvedSvgFilterGeometry {
        filter_region: filter_region.to_paint(),
        primitives,
        local_filter_region,
        primitive_reference_box,
        filter_scale,
    })
}

fn blur_outsets(sigma_x: f32, sigma_y: f32) -> (f32, f32) {
    // Source: fe_gaussian_blur.cc:38-76. Preserve its float calculation.
    let kernel = |sigma: f32| {
        if sigma == 0.0 {
            0.0
        } else {
            ((sigma * (3.0 / 4.0 * std::f32::consts::TAU.sqrt()) + 0.5).floor()).max(2.0)
        }
    };
    (1.5 * kernel(sigma_x), 1.5 * kernel(sigma_y))
}

fn transfer_affects_transparent(function: &SvgTransferFunction) -> bool {
    let intercept = match function {
        SvgTransferFunction::kIdentity => 0.0,
        SvgTransferFunction::kTable(values) | SvgTransferFunction::kDiscrete(values) => {
            values.first().copied().unwrap_or(0.0)
        }
        SvgTransferFunction::kLinear { intercept, .. } => *intercept,
        SvgTransferFunction::kGamma { offset, .. } => *offset,
    };
    255.0 * intercept >= 1.0
}

fn lookup_bounds(
    input: &SvgFilterInput,
    source: BlinkRectF,
    filter_region: BlinkRectF,
    previous: Option<BlinkRectF>,
    named: &HashMap<&str, BlinkRectF>,
    fill_paint_installed: bool,
    stroke_paint_installed: bool,
) -> BlinkRectF {
    match input {
        SvgFilterInput::kSourceGraphic | SvgFilterInput::kSourceAlpha => source,
        SvgFilterInput::kFillPaint if fill_paint_installed => filter_region,
        SvgFilterInput::kStrokePaint if stroke_paint_installed => filter_region,
        SvgFilterInput::kNamed(name) if !name.is_empty() => named
            .get(name.as_str())
            .copied()
            .unwrap_or_else(|| previous.unwrap_or(source)),
        SvgFilterInput::kFillPaint | SvgFilterInput::kStrokePaint | SvgFilterInput::kNamed(_) => {
            previous.unwrap_or(source)
        }
    }
}

/// Sources: filter_effect.cc:55-82 and the concrete FE* MapInputs/MapEffect
/// implementations. This is Blink's forward local-space bounds graph, not a
/// filter-region substitute.
pub fn map_svg_filter_forward_bounds(
    graph: &SvgFilterGraph,
    geometry: &ResolvedSvgFilterGeometry,
    input: PaintRect,
) -> PaintRect {
    use SvgFilterPrimitiveKind::*;
    let filter_region = geometry
        .local_filter_region
        .scale(geometry.filter_scale, geometry.filter_scale);
    let source = BlinkRectF::from_paint(input).intersect(filter_region);
    let mut previous = None;
    let mut named = HashMap::new();
    for resolved in &geometry.primitives {
        let primitive = &graph.primitives[resolved.source_index];
        let first = lookup_bounds(
            &primitive.input,
            source,
            filter_region,
            previous,
            &named,
            graph.fill_paint.is_some(),
            graph.stroke_paint.is_some(),
        );
        let second = |input: &SvgFilterInput| {
            lookup_bounds(
                input,
                source,
                filter_region,
                previous,
                &named,
                graph.fill_paint.is_some(),
                graph.stroke_paint.is_some(),
            )
        };
        let mut bounds = match &primitive.kind {
            kFlood { .. } | kTurbulence { .. } | kImage { .. } | kTile => {
                geometry.absolute_crop(resolved.local_region)
            }
            kMerge { inputs } => {
                let mut result = BlinkRectF::default();
                for input in inputs {
                    result.union(second(input));
                }
                result
            }
            kBlend { input2, .. } => {
                let mut result = first;
                result.union(second(input2));
                result
            }
            kComposite {
                operator,
                input2,
                k,
                ..
            } => {
                let other = second(input2);
                match operator {
                    SvgCompositeOperator::kIn => first.intersect(other),
                    SvgCompositeOperator::kAtop => other,
                    SvgCompositeOperator::kArithmetic if k[3] <= 0.0 => {
                        if k[1] > 0.0 && k[2] > 0.0 {
                            let mut result = first;
                            result.union(other);
                            result
                        } else if k[1] > 0.0 {
                            first
                        } else if k[2] > 0.0 {
                            other
                        } else if k[0] > 0.0 {
                            first.intersect(other)
                        } else {
                            BlinkRectF::default()
                        }
                    }
                    _ => {
                        let mut result = first;
                        result.union(other);
                        result
                    }
                }
            }
            kDisplacementMap { .. } => first,
            _ => first,
        };
        match &primitive.kind {
            kGaussianBlur { sigma_x, sigma_y } => {
                let (x, y) = blur_outsets(
                    geometry.scale_primitive_x(*sigma_x),
                    geometry.scale_primitive_y(*sigma_y),
                );
                bounds.outset(x, y);
            }
            kOffset { dx, dy } => {
                bounds.offset(
                    geometry.scale_primitive_x(*dx),
                    geometry.scale_primitive_y(*dy),
                );
            }
            kDropShadow {
                sigma_x,
                sigma_y,
                dx,
                dy,
                ..
            } => {
                let (x, y) = blur_outsets(
                    geometry.scale_primitive_x(*sigma_x),
                    geometry.scale_primitive_y(*sigma_y),
                );
                let mut shadow = bounds;
                shadow.offset(
                    geometry.scale_primitive_x(*dx),
                    geometry.scale_primitive_y(*dy),
                );
                shadow.outset(x, y);
                bounds.union(shadow);
            }
            kMorphology {
                radius_x, radius_y, ..
            } => {
                bounds.outset(
                    geometry.scale_primitive_x(*radius_x),
                    geometry.scale_primitive_y(*radius_y),
                );
            }
            kDisplacementMap { scale, .. } => {
                // Chromium divides the authored float before applying the
                // Filter scaling (fe_displacement_map.cc:43-47).
                bounds.outset(
                    geometry.scale_primitive_x(f64::from((*scale as f32).abs() / 2.0)),
                    geometry.scale_primitive_y(f64::from((*scale as f32).abs() / 2.0)),
                );
            }
            kConvolveMatrix {
                order_x,
                order_y,
                kernel,
                target_x,
                target_y,
                ..
            } => {
                let area = order_x.checked_mul(*order_y);
                let valid = *order_x > 0
                    && *order_y > 0
                    && area.is_some_and(|area| area <= i32::MAX as u32)
                    && area.map(|area| area as usize) == Some(kernel.len())
                    && *target_x >= 0
                    && (*target_x as u32) < *order_x
                    && *target_y >= 0
                    && (*target_y as u32) < *order_y;
                if valid {
                    bounds.offset(-(*target_x as f32), -(*target_y as f32));
                    bounds.set_width(bounds.width + *order_x as f32);
                    bounds.set_height(bounds.height + *order_y as f32);
                }
            }
            _ => {}
        }
        let affects_transparent = match &primitive.kind {
            kColorMatrix { kind, values } => {
                *kind == crate::internal::paint_input::SvgColorMatrixType::kMatrix
                    && values.len() >= 20
                    && values[19] > 0.0
            }
            kComponentTransfer { functions } => transfer_affects_transparent(&functions[3]),
            kComposite {
                operator: SvgCompositeOperator::kArithmetic,
                k,
                ..
            } => k[3] > 0.0,
            kLighting { .. } => true,
            _ => false,
        };
        let crop = geometry.absolute_crop(resolved.local_region);
        bounds = if affects_transparent {
            crop
        } else {
            bounds.intersect(crop)
        };
        if !primitive.result.is_empty() {
            named.insert(primitive.result.as_str(), bounds);
        }
        previous = Some(bounds);
    }
    previous.unwrap_or(source).to_paint()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::paint_input::{SvgColorMatrixType, SvgFilterPrimitive};

    fn absolute(value: f64) -> Option<SvgFilterLength> {
        Some(SvgFilterLength {
            value,
            percentage: false,
        })
    }

    fn graph(kind: SvgFilterPrimitiveKind) -> SvgFilterGraph {
        SvgFilterGraph {
            resource_node_id: 0,
            id: "bounds".into(),
            filter_units: SvgFilterUnits::kUserSpaceOnUse,
            primitive_units: SvgFilterUnits::kUserSpaceOnUse,
            region: SvgFilterRegion {
                x: absolute(0.0),
                y: absolute(0.0),
                width: absolute(200.0),
                height: absolute(200.0),
            },
            primitives: vec![SvgFilterPrimitive {
                input: SvgFilterInput::kSourceGraphic,
                result: String::new(),
                region: SvgFilterRegion::default(),
                linear_rgb: false,
                origin_tainted: false,
                kind,
            }],
            fill_paint: None,
            stroke_paint: None,
            unsupported: None,
        }
    }

    fn bounds(kind: SvgFilterPrimitiveKind) -> PaintRect {
        let graph = graph(kind);
        let geometry = resolve_svg_filter_geometry(
            &graph,
            PaintRect {
                width: 100.0,
                height: 100.0,
                ..Default::default()
            },
            None,
            1.0,
        )
        .expect("absolute user-space graph");
        map_svg_filter_forward_bounds(
            &graph,
            &geometry,
            PaintRect {
                x: 20.0,
                y: 30.0,
                width: 10.0,
                height: 10.0,
            },
        )
    }

    #[test]
    fn offset_and_gaussian_bounds_match_filter_effect_map_effect() {
        assert_eq!(
            bounds(SvgFilterPrimitiveKind::kOffset { dx: 5.0, dy: -7.0 }),
            PaintRect {
                x: 25.0,
                y: 23.0,
                width: 10.0,
                height: 10.0,
            }
        );
        assert_eq!(
            bounds(SvgFilterPrimitiveKind::kGaussianBlur {
                sigma_x: 2.0,
                sigma_y: 2.0,
            }),
            PaintRect {
                x: 14.0,
                y: 24.0,
                width: 22.0,
                height: 22.0,
            }
        );
    }

    #[test]
    fn convolve_and_displacement_bounds_keep_their_distinct_source_rules() {
        assert_eq!(
            bounds(SvgFilterPrimitiveKind::kConvolveMatrix {
                order_x: 3,
                order_y: 5,
                kernel: vec![0.0; 15],
                divisor: 1.0,
                bias: 0.0,
                target_x: 1,
                target_y: 2,
                edge_mode: crate::internal::paint_input::SvgFilterEdgeMode::kDuplicate,
                preserve_alpha: false,
            }),
            PaintRect {
                x: 19.0,
                y: 28.0,
                width: 13.0,
                height: 15.0,
            }
        );
        assert_eq!(
            bounds(SvgFilterPrimitiveKind::kDisplacementMap {
                input2: SvgFilterInput::kSourceGraphic,
                scale: 10.0,
                x_channel: crate::internal::paint_input::SvgFilterChannel::kA,
                y_channel: crate::internal::paint_input::SvgFilterChannel::kA,
            }),
            PaintRect {
                x: 15.0,
                y: 25.0,
                width: 20.0,
                height: 20.0,
            }
        );
    }

    #[test]
    fn alpha_additive_color_matrix_uses_the_whole_primitive_crop() {
        let mut values = vec![0.0; 20];
        values[19] = 0.01;
        assert_eq!(
            bounds(SvgFilterPrimitiveKind::kColorMatrix {
                kind: SvgColorMatrixType::kMatrix,
                values,
            }),
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            }
        );
    }

    #[test]
    fn tile_map_inputs_returns_its_absolute_bounds() {
        assert_eq!(
            bounds(SvgFilterPrimitiveKind::kTile),
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            }
        );
    }

    #[test]
    fn geometry_keeps_chromium_float_evaluation_order() {
        let mut graph = graph(SvgFilterPrimitiveKind::kOffset { dx: 0.1, dy: 0.0 });
        graph.filter_units = SvgFilterUnits::kObjectBoundingBox;
        graph.primitive_units = SvgFilterUnits::kObjectBoundingBox;
        graph.region.x = Some(SvgFilterLength {
            value: 0.2,
            percentage: true,
        });
        graph.region.y = absolute(0.0);
        graph.region.width = Some(SvgFilterLength {
            value: 100.0,
            percentage: true,
        });
        graph.region.height = graph.region.width;
        let geometry = resolve_svg_filter_geometry(
            &graph,
            PaintRect {
                x: 0.1,
                y: 0.0,
                width: 0.1,
                height: 1.0,
            },
            None,
            0.3,
        )
        .expect("object-box graph");

        let local_x = 0.1_f32 + 0.1_f32 * 0.2_f32 / 100.0_f32;
        assert_eq!(geometry.filter_region.x, f64::from(local_x * 0.3_f32));
        assert_eq!(
            geometry.scale_primitive_x(0.1),
            (0.1_f32 * 0.1_f32) * 0.3_f32
        );
        let absolute = PaintRect {
            x: 3.0,
            y: 6.0,
            width: 9.0,
            height: 12.0,
        };
        let local = geometry.map_absolute_rect_to_local(absolute);
        let inverse = 1.0_f32 / 0.3_f32;
        assert_eq!(
            local,
            PaintRect {
                x: f64::from(3.0_f32 * inverse),
                y: f64::from(6.0_f32 * inverse),
                width: f64::from(9.0_f32 * inverse),
                height: f64::from(12.0_f32 * inverse),
            }
        );
    }

    #[test]
    fn rect_union_advances_a_rounded_down_right_edge() {
        let mut result = BlinkRectF::new(0.3, 0.0, 0.1, 1.0);
        let other = BlinkRectF::new(0.7, 0.0, 0.7, 1.0);
        let required_right = other.right();
        result.union(other);
        assert!(result.right() >= required_right);
        assert_eq!(result.width, next_greater_float(1.099_999_9));
    }
}
