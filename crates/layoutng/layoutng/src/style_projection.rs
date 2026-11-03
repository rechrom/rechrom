// Canonical ComputedStyle access and legacy Layout input conversion.

#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug)]
pub struct StyleProjectionError;

// Production attachment borrows Blink's immutable ComputedStyle. The complete
// conversion below remains only for the legacy Layout input API and its tests.

use crate::internal::layout_input::{
    ComputedStyle as LayoutComputedStyle, ExtendedStyle, FlexBasisSizing, FlexDirection,
    IntrinsicSizing, ShapeCoordinate, ShapeOutsideGeometry, ShapeOutsideKind, ShapePoint,
    ShapeRadius, ShapeRadiusKind, ShapeReferenceBox, TextDirection as LayoutTextDirection,
    WritingMode as LayoutWritingMode,
};
use crate::internal::paint_input::{
    PaintCornerRadii, PaintCornerRadius, PaintStyleData, PaintTransform, PaintTransformOperation,
    PaintTransformOperationKind,
};
use foundation::{EDisplay, Length};
use layoutng_style::style::{
    basic_shapes::{
        BasicShape, BasicShapeCircle, BasicShapeEllipse, BasicShapeInset, BasicShapePolygon,
        BasicShapeRadius, BasicShapeWithCenterAndRadii, RadiusType, ShapeType,
    },
    computed_style::ComputedStyle,
    computed_style_constants::{FlexWrapMode, ShapeBox},
    shape_value::ShapeValueType,
    style_fetched_image::FetchedImage,
};

/// Resource graph bindings resolved by the document after computed style.
/// These values have no canonical home in ComputedStyle; keeping them separate
/// prevents the package boundary from growing into a second general style.
#[derive(Clone, Copy)]
pub struct StyleResourceBindings<'a> {
    pub(crate) svg_filter_graphs:
        &'a std::sync::Arc<Vec<crate::internal::paint_input::SvgFilterGraph>>,
    pub(crate) svg_clip_resources: &'a crate::internal::paint_input::SvgClipResourceBinding,
    pub(crate) svg_mask_resource:
        &'a Option<std::sync::Arc<crate::internal::paint_input::SvgMaskGraph>>,
    pub(crate) svg_fill_server: &'a Option<crate::internal::paint_input::SvgPaintServer>,
    pub(crate) svg_stroke_server: &'a Option<crate::internal::paint_input::SvgPaintServer>,
    pub(crate) svg_markers: &'a [crate::internal::paint_input::SvgMarkerInstance],
    pub(crate) replaced_image_shader:
        &'a Option<std::sync::Arc<crate::internal::paint_input::PaintShader>>,
}

impl<'a> StyleResourceBindings<'a> {
    pub(crate) fn FromData(
        resources: &'a crate::internal::layout_input::StyleResourceData,
    ) -> Self {
        Self {
            svg_filter_graphs: &resources.svg_filter_graphs,
            svg_clip_resources: &resources.svg_clip_resources,
            svg_mask_resource: &resources.svg_mask_resource,
            svg_fill_server: &resources.svg_fill_server,
            svg_stroke_server: &resources.svg_stroke_server,
            svg_markers: resources.svg_markers.as_ref(),
            replaced_image_shader: &resources.replaced_image_shader,
        }
    }

    pub fn FromParts(
        svg_filter_graphs: &'a std::sync::Arc<Vec<crate::internal::paint_input::SvgFilterGraph>>,
        svg_clip_resources: &'a crate::internal::paint_input::SvgClipResourceBinding,
        svg_mask_resource: &'a Option<std::sync::Arc<crate::internal::paint_input::SvgMaskGraph>>,
        svg_fill_server: &'a Option<crate::internal::paint_input::SvgPaintServer>,
        svg_stroke_server: &'a Option<crate::internal::paint_input::SvgPaintServer>,
        svg_markers: &'a [crate::internal::paint_input::SvgMarkerInstance],
        replaced_image_shader: &'a Option<
            std::sync::Arc<crate::internal::paint_input::PaintShader>,
        >,
    ) -> Self {
        Self {
            svg_filter_graphs,
            svg_clip_resources,
            svg_mask_resource,
            svg_fill_server,
            svg_stroke_server,
            svg_markers,
            replaced_image_shader,
        }
    }

    pub(crate) fn FromPaint(
        paint: &'a PaintStyleData,
        svg_clip_resources: &'a crate::internal::paint_input::SvgClipResourceBinding,
    ) -> Self {
        Self {
            svg_filter_graphs: &paint.svg_filter_graphs,
            svg_clip_resources,
            svg_mask_resource: &paint.svg_mask_resource,
            svg_fill_server: &paint.svg_fill_server,
            svg_stroke_server: &paint.svg_stroke_server,
            svg_markers: &paint.svg_markers,
            replaced_image_shader: &paint.replaced_image_shader,
        }
    }

    pub(crate) fn ToData(
        &self,
        image_resource_id: Option<u64>,
    ) -> crate::internal::layout_input::StyleResourceData {
        crate::internal::layout_input::StyleResourceData {
            image_resource_id,
            svg_filter_graphs: std::sync::Arc::clone(self.svg_filter_graphs),
            svg_clip_resources: self.svg_clip_resources.clone(),
            svg_mask_resource: self.svg_mask_resource.clone(),
            svg_fill_server: self.svg_fill_server.clone(),
            svg_stroke_server: self.svg_stroke_server.clone(),
            svg_markers: self.svg_markers.to_vec().into(),
            replaced_image_shader: self.replaced_image_shader.clone(),
        }
    }

    pub(crate) fn ApplyTo(&self, style: &mut LayoutComputedStyle) {
        style.paint.svg_filter_graphs = std::sync::Arc::clone(self.svg_filter_graphs);
        style.svg_clip_resources = self.svg_clip_resources.clone();
        style.paint.svg_mask_resource = self.svg_mask_resource.clone();
        style.paint.svg_fill_server = self.svg_fill_server.clone();
        style.paint.svg_stroke_server = self.svg_stroke_server.clone();
        style.paint.svg_markers = self.svg_markers.to_vec();
        style.paint.replaced_image_shader = self.replaced_image_shader.clone();
    }
}

pub(crate) fn BorrowComputedStyle(
    computed_style: &ComputedStyle,
    resources: &StyleResourceBindings<'_>,
    image_resource_id: Option<u64>,
) -> LayoutComputedStyle {
    let mut input = LayoutComputedStyle {
        style: computed_style,
        ..Default::default()
    };
    resources.ApplyTo(&mut input);
    input.paint.image_resource_id = image_resource_id;
    input
}

#[derive(Default)]
struct ProjectedLength {
    pixels: Option<f64>,
    percent: Option<f64>,
    calculation: Option<std::sync::Arc<foundation::CalculationValue>>,
    calculated: bool,
    auto: bool,
}

// cpp: foundation/blink_geometry/geometry/length.h:213-280
// cpp: foundation/blink_geometry/geometry/length.cc:138-157
fn ProjectLength(value: &Length) -> ProjectedLength {
    if value.IsAuto() {
        return ProjectedLength {
            auto: true,
            ..Default::default()
        };
    }
    if value.IsFixed() {
        return ProjectedLength {
            pixels: Some(value.Pixels() as f64),
            ..Default::default()
        };
    }
    if value.IsPercent() {
        return ProjectedLength {
            percent: Some(value.PercentValue() as f64),
            ..Default::default()
        };
    }
    if value.IsCalculated() {
        let calculation = value.GetCalculationValue();
        if !calculation.IsExpression() {
            let pair = calculation.GetPixelsAndPercent();
            return ProjectedLength {
                pixels: pair.has_explicit_pixels.then_some(pair.pixels as f64),
                percent: pair.has_explicit_percent.then_some(pair.percent as f64),
                calculated: true,
                auto: false,
                calculation: None,
            };
        }
        return ProjectedLength {
            calculated: true,
            calculation: Some(calculation),
            ..Default::default()
        };
    }
    ProjectedLength::default()
}

// Blink stores ordinary computed lengths in zoomed CSS pixels. Rechrom's
// renderer-neutral layout snapshot stores author-space CSS pixels and applies
// `effective_zoom` once at the Layout boundary (see `css_zoom::ZoomedStyle`).
// Undo the computed style storage zoom while projecting; percentages stay dimensionless.
fn ProjectUnzoomedLength(value: &Length, effective_zoom: f32) -> ProjectedLength {
    let mut projected = ProjectLength(value);
    if effective_zoom > 0.0 && effective_zoom != 1.0 {
        if let Some(pixels) = &mut projected.pixels {
            *pixels /= effective_zoom as f64;
        }
        if let Some(calculation) = &mut projected.calculation {
            *calculation = calculation.Zoom(1.0 / effective_zoom as f64);
        }
    }
    projected
}

fn ProjectShapeCoordinate(value: &Length) -> ShapeCoordinate {
    // Basic-shape computed lengths already include EffectiveZoom in Blink.
    // Unlike the author-space box geometry below, Layout consumes this shape
    // payload directly and therefore must not apply zoom a second time.
    let value = ProjectLength(value);
    ShapeCoordinate {
        pixels: value.pixels.unwrap_or(0.0),
        percentage: value.percent.unwrap_or(0.0),
        calculation: value.calculation,
    }
}

fn ProjectShapePoint(value: &foundation::LengthPoint) -> ShapePoint {
    ShapePoint {
        x: ProjectShapeCoordinate(value.X()),
        y: ProjectShapeCoordinate(value.Y()),
    }
}

fn ProjectShapeRadius(value: &BasicShapeRadius) -> ShapeRadius {
    ShapeRadius {
        kind: match value.GetType() {
            RadiusType::kValue => ShapeRadiusKind::kLengthPercentage,
            RadiusType::kClosestSide => ShapeRadiusKind::kClosestSide,
            RadiusType::kFarthestSide => ShapeRadiusKind::kFarthestSide,
            RadiusType::kClosestCorner => ShapeRadiusKind::kClosestCorner,
            RadiusType::kFarthestCorner => ShapeRadiusKind::kFarthestCorner,
        },
        value: ProjectShapeCoordinate(value.Value()),
    }
}

fn ProjectShapeReferenceBox(value: ShapeBox) -> ShapeReferenceBox {
    match value {
        ShapeBox::kMarginBox => ShapeReferenceBox::kMarginBox,
        ShapeBox::kBorderBox => ShapeReferenceBox::kBorderBox,
        ShapeBox::kPaddingBox => ShapeReferenceBox::kPaddingBox,
        ShapeBox::kContentBox => ShapeReferenceBox::kContentBox,
    }
}

/// Project the computed style ShapeValue into Layout's retained, renderer-neutral
/// exclusion input. The computed style value remains canonical; this is the minimal
/// package boundary needed by the translated float-shape algorithm.
// cpp: core/layout/shapes/shape_outside_info.cc:194-236
pub(crate) fn ProjectShapeOutside(computed_style: &ComputedStyle, ext: &mut ExtendedStyle) {
    // shape-margin is stored zoomed in ComputedStyle while ExtendedStyle is
    // author-space and css_zoom applies EffectiveZoom exactly once.
    let margin =
        ProjectUnzoomedLength(computed_style.ShapeMargin(), computed_style.EffectiveZoom());
    ext.shape_margin = ShapeCoordinate {
        pixels: margin.pixels.unwrap_or(0.0),
        percentage: margin.percent.unwrap_or(0.0),
        calculation: margin.calculation,
    };
    ext.shape_image_threshold = computed_style.ShapeImageThreshold() as f64;

    let Some(value) = (unsafe { computed_style.ShapeOutside().as_ref() }) else {
        ext.shape_outside = None;
        return;
    };
    let mut output = ShapeOutsideGeometry {
        reference_box: ProjectShapeReferenceBox(value.CssBox()),
        ..ShapeOutsideGeometry::default()
    };
    match value.GetType() {
        ShapeValueType::kBox => {}
        ShapeValueType::kImage => {
            // Pending/failed and generated images are not usable shape images
            // in Chromium's ShapeOutsideInfo::IsEnabledFor. Do not publish an
            // invalid resource id into Layout's immutable image lookup.
            let Some(image) = (unsafe { value.GetImage().as_ref() }).and_then(FetchedImage) else {
                ext.shape_outside = None;
                return;
            };
            if image.ResourceId() == 0 {
                ext.shape_outside = None;
                return;
            }
            output.kind = ShapeOutsideKind::kImage;
            output.resource_id = image.ResourceId();
        }
        ShapeValueType::kShape => {
            let shape = value.Shape();
            match shape.GetType() {
                ShapeType::kBasicShapePolygonType => {
                    let polygon =
                        unsafe { &*(shape as *const dyn BasicShape as *const BasicShapePolygon) };
                    output.kind = ShapeOutsideKind::kPolygon;
                    output.contour = polygon
                        .Values()
                        .as_slice()
                        .chunks_exact(2)
                        .map(|point| ShapePoint {
                            x: ProjectShapeCoordinate(&point[0]),
                            y: ProjectShapeCoordinate(&point[1]),
                        })
                        .collect();
                }
                ShapeType::kBasicShapeCircleType => {
                    let circle =
                        unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeCircle) };
                    output.kind = ShapeOutsideKind::kCircle;
                    output.center = ProjectShapePoint(circle.Center());
                    output.radius_x = ProjectShapeRadius(circle.Radius());
                    output.radius_y = output.radius_x.clone();
                }
                ShapeType::kBasicShapeEllipseType => {
                    let ellipse =
                        unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeEllipse) };
                    output.kind = ShapeOutsideKind::kEllipse;
                    output.center = ProjectShapePoint(ellipse.Center());
                    output.radius_x = ProjectShapeRadius(ellipse.RadiusX());
                    output.radius_y = ProjectShapeRadius(ellipse.RadiusY());
                }
                ShapeType::kBasicShapeInsetType => {
                    let inset =
                        unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeInset) };
                    output.kind = ShapeOutsideKind::kInset;
                    output.insets = [
                        ProjectShapeCoordinate(inset.Top()),
                        ProjectShapeCoordinate(inset.Right()),
                        ProjectShapeCoordinate(inset.Bottom()),
                        ProjectShapeCoordinate(inset.Left()),
                    ];
                    output.corner_radii = [
                        ShapePoint {
                            x: ProjectShapeCoordinate(inset.TopLeftRadius().Width()),
                            y: ProjectShapeCoordinate(inset.TopLeftRadius().Height()),
                        },
                        ShapePoint {
                            x: ProjectShapeCoordinate(inset.TopRightRadius().Width()),
                            y: ProjectShapeCoordinate(inset.TopRightRadius().Height()),
                        },
                        ShapePoint {
                            x: ProjectShapeCoordinate(inset.BottomRightRadius().Width()),
                            y: ProjectShapeCoordinate(inset.BottomRightRadius().Height()),
                        },
                        ShapePoint {
                            x: ProjectShapeCoordinate(inset.BottomLeftRadius().Width()),
                            y: ProjectShapeCoordinate(inset.BottomLeftRadius().Height()),
                        },
                    ];
                }
                // Layout's typed exclusion boundary does not yet carry the
                // path/ray/shape command stream. Keeping it disabled matches
                // Chromium's unusable-image behavior instead of substituting
                // an incorrect reference-box exclusion.
                ShapeType::kStylePathType
                | ShapeType::kStyleRayType
                | ShapeType::kStyleShapeType => {
                    ext.shape_outside = None;
                    return;
                }
            }
        }
    }
    ext.shape_outside = Some(output);
}

pub fn ReadShapeOutside(computed_style: &ComputedStyle) -> Option<ShapeOutsideGeometry> {
    let mut derived = ExtendedStyle::default();
    ProjectShapeOutside(computed_style, &mut derived);
    derived.shape_outside
}

fn ProjectCornerRadius(
    value: &foundation::LengthSize,
    effective_zoom: f32,
) -> (PaintCornerRadius, PaintCornerRadius) {
    let x = ProjectUnzoomedLength(value.Width(), effective_zoom);
    let y = ProjectUnzoomedLength(value.Height(), effective_zoom);
    (
        PaintCornerRadius {
            x: x.pixels.unwrap_or(0.0),
            y: y.pixels.unwrap_or(0.0),
        },
        PaintCornerRadius {
            x: x.percent.unwrap_or(0.0),
            y: y.percent.unwrap_or(0.0),
        },
    )
}

// cpp: core/style/computed style_base.h Border*Radius
// cpp: core/paint/border_shape_painter.cc ResolveCornerRadii
// ComputedStyle owns the canonical, zoomed LengthSize values.  The neutral
// snapshot keeps fixed components in author-space and percentage components
// unresolved so Layout can apply zoom once and Paint can resolve them against
// the final border box.
pub(crate) fn ProjectBorderRadii(computed_style: &ComputedStyle, paint: &mut PaintStyleData) {
    let zoom = computed_style.EffectiveZoom();
    let corners = [
        ProjectCornerRadius(computed_style.BorderTopLeftRadius(), zoom),
        ProjectCornerRadius(computed_style.BorderTopRightRadius(), zoom),
        ProjectCornerRadius(computed_style.BorderBottomRightRadius(), zoom),
        ProjectCornerRadius(computed_style.BorderBottomLeftRadius(), zoom),
    ];
    let pixels = PaintCornerRadii {
        top_left: corners[0].0,
        top_right: corners[1].0,
        bottom_right: corners[2].0,
        bottom_left: corners[3].0,
    };
    paint.border_radii_percentages = PaintCornerRadii {
        top_left: corners[0].1,
        top_right: corners[1].1,
        bottom_right: corners[2].1,
        bottom_left: corners[3].1,
    };
    paint.border_radii =
        (pixels.HasRadius() || paint.border_radii_percentages.HasRadius()).then_some(pixels);
    paint.border_radius = 0.0;
}

// cpp: core/style/shadow_data.h
// cpp: core/paint/box_painter_base.cc PaintBoxShadow
pub(crate) fn ProjectShadows<E, F>(
    computed_style: &ComputedStyle,
    paint: &mut PaintStyleData,
    resolve_color: F,
) -> Result<(), E>
where
    F: Fn(
        &layoutng_style::css::style_color::StyleColor,
    ) -> Result<crate::internal::layout_input_types::Color, E>,
{
    use crate::internal::layout_input::{Offset, PaintShadow};
    use layoutng_style::style::shadow_data::ShadowStyle;

    let zoom = computed_style.EffectiveZoom() as f64;
    let unzoom = |value: f32| {
        if zoom > 0.0 {
            value as f64 / zoom
        } else {
            value as f64
        }
    };
    let project =
        |list: *mut layoutng_style::style::shadow_list::ShadowList| -> Result<Vec<PaintShadow>, E> {
            if list.is_null() {
                return Ok(Vec::new());
            }
            unsafe { &*list }
                .Shadows()
                .iter()
                .map(|shadow| {
                    Ok(PaintShadow {
                        offset: Offset {
                            x: unzoom(shadow.X()),
                            y: unzoom(shadow.Y()),
                        },
                        blur_radius: unzoom(shadow.BlurRadius()),
                        spread: unzoom(shadow.Spread()),
                        color: resolve_color(shadow.GetColor())?,
                        inset: shadow.Style() == ShadowStyle::kInset,
                    })
                })
                .collect()
        };
    paint.box_shadows = project(computed_style.BoxShadow())?;
    paint.text_shadows = project(computed_style.TextShadow())?;
    Ok(())
}

pub(crate) fn ProjectTransform(computed_style: &ComputedStyle) -> Option<PaintTransform> {
    use foundation::{TransformOperationType, TranslateTransformOperation};
    let mut projected = PaintTransform::default();
    for operation in computed_style.Transform().Operations().iter() {
        let operation = operation.GetNonNull().expect("transform operation is null");
        let operation = unsafe { operation.as_ref() };
        if matches!(
            operation.GetType(),
            TransformOperationType::kTranslate
                | TransformOperationType::kTranslateX
                | TransformOperationType::kTranslateY
                | TransformOperationType::kTranslateZ
                | TransformOperationType::kTranslate3D
        ) {
            // TransformOperation's discriminant is the same checked downcast
            // used by Blink's concrete operation visitors.
            let translate = unsafe {
                &*(operation as *const dyn foundation::TransformOperation
                    as *const TranslateTransformOperation)
            };
            let x = ProjectLength(translate.XLength());
            let y = ProjectLength(translate.YLength());
            projected.operations.push(PaintTransformOperation {
                kind: if matches!(
                    operation.GetType(),
                    TransformOperationType::kTranslateZ | TransformOperationType::kTranslate3D
                ) {
                    PaintTransformOperationKind::kTranslate3D
                } else {
                    PaintTransformOperationKind::kTranslate
                },
                pixels: crate::internal::layout_input::Offset {
                    x: x.pixels.unwrap_or(0.0),
                    y: y.pixels.unwrap_or(0.0),
                },
                percentages: crate::internal::layout_input::Offset {
                    x: x.percent.unwrap_or(0.0),
                    y: y.percent.unwrap_or(0.0),
                },
                z: translate.Z(),
                ..PaintTransformOperation::default()
            });
            continue;
        }
        let mut matrix = foundation::gfx::Transform::default();
        operation.Apply(&mut matrix, &foundation::gfx::SizeF::new(0.0, 0.0));
        projected.operations.push(PaintTransformOperation {
            kind: PaintTransformOperationKind::kMatrix,
            matrix: crate::internal::layout_input::TransformMatrix {
                values: matrix.GetColMajor(),
            },
            ..PaintTransformOperation::default()
        });
    }
    (!projected.operations.is_empty()).then_some(projected)
}

fn ProjectDimension(
    value: &Length,
    effective_zoom: f32,
    pixels: &mut Option<f64>,
    percent: &mut Option<f64>,
    calculated: &mut bool,
) {
    let value = ProjectUnzoomedLength(value, effective_zoom);
    *pixels = value.pixels;
    *percent = value.percent;
    *calculated = value.calculated;
}

// cpp: foundation/blink_geometry/geometry/length.h:116-130
// cpp: layoutng/internal/boundary/input.cc:241-262
// Intrinsic sizing keywords have no pixel/percentage payload. Keep their
// discriminant across the Style -> Layout package boundary instead of
// collapsing them to `auto` when ProjectLength intentionally returns an empty
// numeric value.
fn ProjectIntrinsicSizing(value: &Length) -> IntrinsicSizing {
    if value.IsMinContent() {
        IntrinsicSizing::kMinContent
    } else if value.IsMaxContent() {
        IntrinsicSizing::kMaxContent
    } else if value.IsFitContent() {
        IntrinsicSizing::kFitContent
    } else {
        IntrinsicSizing::kAuto
    }
}

fn ProjectFlexBasisSizing(value: &Length) -> FlexBasisSizing {
    if value.GetType() == foundation::LengthType::kContent {
        FlexBasisSizing::kContent
    } else if value.IsMinContent() {
        FlexBasisSizing::kMinContent
    } else if value.IsMaxContent() {
        FlexBasisSizing::kMaxContent
    } else if value.IsFitContent() {
        FlexBasisSizing::kFitContent
    } else {
        FlexBasisSizing::kAuto
    }
}

fn ProjectEdge(
    value: &Length,
    effective_zoom: f32,
    pixels: &mut f64,
    percent: &mut Option<f64>,
    calculated: &mut bool,
    auto: Option<&mut bool>,
) {
    let value = ProjectUnzoomedLength(value, effective_zoom);
    *pixels = value.pixels.unwrap_or(0.0);
    *percent = value.percent;
    *calculated = value.calculated;
    if let Some(auto) = auto {
        *auto = value.auto;
    }
}

// cpp: layoutng_style/style/computed style_base.h Width/Height/Margin*/Padding*/Inset
// cpp: core/css/resolver/style_resolver.cc ApplyProperties
pub(crate) fn ProjectGeometry(
    computed_style: &ComputedStyle,
    resolved: &mut LayoutComputedStyle,
    ext: &mut ExtendedStyle,
) {
    let effective_zoom = computed_style.EffectiveZoom();
    ProjectDimension(
        computed_style.Width(),
        effective_zoom,
        &mut resolved.width,
        &mut ext.width_percent,
        &mut ext.width_calculated,
    );
    ext.width_sizing = ProjectIntrinsicSizing(computed_style.Width());
    ProjectDimension(
        computed_style.Height(),
        effective_zoom,
        &mut resolved.height,
        &mut ext.height_percent,
        &mut ext.height_calculated,
    );
    ext.height_sizing = ProjectIntrinsicSizing(computed_style.Height());
    ProjectDimension(
        computed_style.MinWidth(),
        effective_zoom,
        &mut resolved.min_width,
        &mut ext.min_width_percent,
        &mut ext.min_width_calculated,
    );
    ext.min_width_sizing = ProjectIntrinsicSizing(computed_style.MinWidth());
    ProjectDimension(
        computed_style.MaxWidth(),
        effective_zoom,
        &mut resolved.max_width,
        &mut ext.max_width_percent,
        &mut ext.max_width_calculated,
    );
    ext.max_width_sizing = ProjectIntrinsicSizing(computed_style.MaxWidth());
    ProjectDimension(
        computed_style.MinHeight(),
        effective_zoom,
        &mut resolved.min_height,
        &mut ext.min_height_percent,
        &mut ext.min_height_calculated,
    );
    ext.min_height_sizing = ProjectIntrinsicSizing(computed_style.MinHeight());
    ProjectDimension(
        computed_style.MaxHeight(),
        effective_zoom,
        &mut resolved.max_height,
        &mut ext.max_height_percent,
        &mut ext.max_height_calculated,
    );
    ext.max_height_sizing = ProjectIntrinsicSizing(computed_style.MaxHeight());
    ProjectDimension(
        computed_style.FlexBasis(),
        effective_zoom,
        &mut resolved.flex_basis,
        &mut ext.flex_basis_percent,
        &mut ext.flex_basis_calculated,
    );
    ext.flex_basis_sizing = ProjectFlexBasisSizing(computed_style.FlexBasis());

    let margins = [
        computed_style.MarginTop(),
        computed_style.MarginRight(),
        computed_style.MarginBottom(),
        computed_style.MarginLeft(),
    ];
    let margin_pixels = [
        &mut resolved.margin.top,
        &mut resolved.margin.right,
        &mut resolved.margin.bottom,
        &mut resolved.margin.left,
    ];
    for (index, (value, pixels)) in margins.into_iter().zip(margin_pixels).enumerate() {
        ProjectEdge(
            value,
            effective_zoom,
            pixels,
            &mut ext.margin_percentages[index],
            &mut ext.margin_calculated[index],
            Some(&mut ext.margin_auto[index]),
        );
    }

    let paddings = [
        computed_style.PaddingTop(),
        computed_style.PaddingRight(),
        computed_style.PaddingBottom(),
        computed_style.PaddingLeft(),
    ];
    let padding_pixels = [
        &mut resolved.padding.top,
        &mut resolved.padding.right,
        &mut resolved.padding.bottom,
        &mut resolved.padding.left,
    ];
    for (index, (value, pixels)) in paddings.into_iter().zip(padding_pixels).enumerate() {
        ProjectEdge(
            value,
            effective_zoom,
            pixels,
            &mut ext.padding_percentages[index],
            &mut ext.padding_calculated[index],
            None,
        );
    }

    let insets = [
        computed_style.Top(),
        computed_style.Right(),
        computed_style.Bottom(),
        computed_style.Left(),
    ];
    let inset_pixels = [
        &mut resolved.top,
        &mut resolved.right,
        &mut resolved.bottom,
        &mut resolved.left,
    ];
    for (index, (value, pixels)) in insets.into_iter().zip(inset_pixels).enumerate() {
        let projected = ProjectUnzoomedLength(value, effective_zoom);
        *pixels = projected.pixels;
        ext.inset_percentages[index] = projected.percent;
        ext.inset_calculated[index] = projected.calculated;
    }

    resolved.flex_grow = computed_style.FlexGrow();
    resolved.flex_shrink = computed_style.FlexShrink();
    resolved.flex_direction = match computed_style.FlexDirection() {
        foundation::EFlexDirection::kRow => FlexDirection::kRow,
        foundation::EFlexDirection::kRowReverse => FlexDirection::kRowReverse,
        foundation::EFlexDirection::kColumn => FlexDirection::kColumn,
        foundation::EFlexDirection::kColumnReverse => FlexDirection::kColumnReverse,
    };
    resolved.flex_wrap = computed_style.FlexWrap().GetWrapMode() != FlexWrapMode::kNowrap;
    ext.wrap_reverse = computed_style.FlexWrap().GetWrapMode() == FlexWrapMode::kWrapReverse;
    resolved.direction = match computed_style.Direction() {
        foundation::TextDirection::kLtr => LayoutTextDirection::kLtr,
        foundation::TextDirection::kRtl => LayoutTextDirection::kRtl,
    };
    resolved.writing_mode = match computed_style.GetWritingMode() {
        foundation::WritingMode::kHorizontalTb => LayoutWritingMode::kHorizontalTb,
        foundation::WritingMode::kVerticalRl | foundation::WritingMode::kSidewaysRl => {
            LayoutWritingMode::kVerticalRl
        }
        foundation::WritingMode::kVerticalLr | foundation::WritingMode::kSidewaysLr => {
            LayoutWritingMode::kVerticalLr
        }
    };
    resolved.column_count = computed_style.ColumnCount() as u32;

    let line_height = ProjectUnzoomedLength(computed_style.LineHeight(), effective_zoom);
    ext.line_height = line_height.pixels;
    ext.line_height_percent = line_height.percent;
    let text_indent = ProjectUnzoomedLength(computed_style.TextIndent(), effective_zoom);
    ext.text_indent = text_indent.pixels.unwrap_or(0.0);
    ext.text_indent_percent = text_indent.percent;
    ext.text_indent_calculated = text_indent.calculated;

    let row_gap = computed_style
        .RowGap()
        .as_ref()
        .map(|value| ProjectUnzoomedLength(value, effective_zoom));
    let column_gap = computed_style
        .ColumnGap()
        .as_ref()
        .map(|value| ProjectUnzoomedLength(value, effective_zoom));
    if let Some(value) = row_gap {
        ext.row_gap = value.pixels;
        ext.row_gap_percent = value.percent;
        ext.row_gap_calculated = value.calculated;
    }
    if let Some(value) = column_gap {
        ext.column_gap = value.pixels;
        ext.column_gap_percent = value.percent;
        ext.column_gap_calculated = value.calculated;
    }
    if ext.row_gap == ext.column_gap && ext.row_gap_percent == ext.column_gap_percent {
        resolved.gap = ext.row_gap.unwrap_or(0.0);
    }
}

pub fn ProjectListMarkerImage(
    computed_style: &ComputedStyle,
) -> Option<crate::internal::layout_input::ListMarkerImage> {
    let image = unsafe { computed_style.ListStyleImage().Get().as_ref() }?;
    if image.ErrorOccurred() {
        return None;
    }
    let (resource_id, shader, resolution_scale) = if let Some(fetched) = FetchedImage(image) {
        if fetched.ResourceId() == 0 {
            // Until the resource supplies an intrinsic size and paint id, use
            // list-style-type. Resource invalidation rebuilds this projection.
            return None;
        }
        (fetched.ResourceId(), None, fetched.ResolutionScale())
    } else {
        let shader = crate::internal::boundary::paint_style_export::ExportGeneratedImageShader(
            computed_style,
            image,
        )?;
        (0, Some(shader), 1.0)
    };

    let zoom = computed_style.EffectiveZoom();
    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }
    let side = unsafe { computed_style.GetFont().as_ref() }
        .and_then(|font| unsafe { font.PrimaryFont().as_ref() })
        .map_or_else(
            || computed_style.GetFontDescription().ComputedSize() / 2.0,
            |primary| primary.GetFontMetrics().Ascent() as f32 / 2.0,
        );
    let default_size = foundation::gfx::SizeF::new(side, side);
    let respect_orientation = computed_style.ImageOrientation();
    let size = image.ImageSize(zoom, &default_size, respect_orientation);
    let natural_width = f64::from(size.width()) / f64::from(zoom);
    let natural_height = f64::from(size.height()) / f64::from(zoom);
    if !natural_width.is_finite()
        || !natural_height.is_finite()
        || natural_width <= 0.0
        || natural_height <= 0.0
    {
        return None;
    }
    Some(crate::internal::layout_input::ListMarkerImage {
        resource_id,
        shader,
        natural_width,
        natural_height,
        resolution_scale,
        known_to_be_opaque: resource_id != 0 && image.KnownToBeOpaque(computed_style),
    })
}

pub(crate) fn ReadListStyleType(
    computed_style: &ComputedStyle,
) -> crate::internal::layout_input::ListStyleType {
    use crate::internal::layout_input::ListStyleType;
    let value = computed_style.ListStyleType().Get();
    if value.is_null() {
        return ListStyleType::kNone;
    }
    let value = unsafe { &*value };
    if !value.IsCounterStyle() {
        return ListStyleType::kDisc;
    }
    match value.GetCounterStyleName().Utf8().as_str() {
        "none" => ListStyleType::kNone,
        "circle" => ListStyleType::kCircle,
        "square" => ListStyleType::kSquare,
        "decimal" => ListStyleType::kDecimal,
        "lower-alpha" | "lower-latin" => ListStyleType::kLowerAlpha,
        _ => ListStyleType::kDisc,
    }
}

pub(crate) struct FragmentStyleFacts {
    pub margin: crate::internal::layout_input::Edges,
    pub border: crate::internal::layout_input::Edges,
    pub border_styles: [crate::internal::layout_input::BorderLineStyle; 4],
    pub padding: crate::internal::layout_input::Edges,
    pub display: crate::internal::layout_input::Display,
    pub position: crate::internal::layout_input::Position,
    pub floating: crate::internal::layout_input::FloatSide,
    pub writing_mode: crate::internal::layout_input::WritingMode,
    pub direction: crate::internal::layout_input::TextDirection,
    pub overflow_x: crate::internal::layout_input::Overflow,
    pub overflow_y: crate::internal::layout_input::Overflow,
}

pub(crate) fn ReadFragmentStyleFacts(style: &ComputedStyle) -> FragmentStyleFacts {
    use crate::internal::layout_input::{
        Display as D, FloatSide as F, Overflow as O, Position as P,
    };
    let zoom = style.EffectiveZoom();
    let edge = |value: &Length| ProjectUnzoomedLength(value, zoom).pixels.unwrap_or(0.0);
    let border = |value: foundation::EBorderStyle| {
        crate::internal::boundary::paint_style_export::BorderStyle(value)
    };
    let overflow = |value| match value {
        foundation::EOverflow::kVisible => O::kVisible,
        foundation::EOverflow::kHidden => O::kHidden,
        foundation::EOverflow::kScroll => O::kScroll,
        foundation::EOverflow::kAuto | foundation::EOverflow::kOverlay => O::kAuto,
        foundation::EOverflow::kClip => O::kClip,
    };
    FragmentStyleFacts {
        margin: crate::internal::layout_input::Edges {
            top: edge(style.MarginTop()),
            right: edge(style.MarginRight()),
            bottom: edge(style.MarginBottom()),
            left: edge(style.MarginLeft()),
        },
        border: crate::internal::layout_input::Edges {
            top: f64::from(style.BorderTopWidth()) / f64::from(zoom),
            right: f64::from(style.BorderRightWidth()) / f64::from(zoom),
            bottom: f64::from(style.BorderBottomWidth()) / f64::from(zoom),
            left: f64::from(style.BorderLeftWidth()) / f64::from(zoom),
        },
        border_styles: [
            border(style.BorderTopStyle()),
            border(style.BorderRightStyle()),
            border(style.BorderBottomStyle()),
            border(style.BorderLeftStyle()),
        ],
        padding: crate::internal::layout_input::Edges {
            top: edge(style.PaddingTop()),
            right: edge(style.PaddingRight()),
            bottom: edge(style.PaddingBottom()),
            left: edge(style.PaddingLeft()),
        },
        display: match style.Display() {
            EDisplay::kInline => D::kInline,
            EDisplay::kBlock => D::kBlock,
            EDisplay::kInlineBlock => D::kInlineBlock,
            EDisplay::kListItem
            | EDisplay::kInlineListItem
            | EDisplay::kFlowRootListItem
            | EDisplay::kInlineFlowRootListItem => D::kListItem,
            EDisplay::kFlex | EDisplay::kWebkitBox => D::kFlex,
            EDisplay::kInlineFlex | EDisplay::kWebkitInlineBox => D::kInlineFlex,
            EDisplay::kGrid => D::kGrid,
            EDisplay::kInlineGrid => D::kInlineGrid,
            EDisplay::kTable => D::kTable,
            EDisplay::kInlineTable => D::kInlineTable,
            EDisplay::kTableRowGroup => D::kTableSection,
            EDisplay::kTableHeaderGroup => D::kTableHeaderGroup,
            EDisplay::kTableFooterGroup => D::kTableFooterGroup,
            EDisplay::kTableRow => D::kTableRow,
            EDisplay::kTableColumn => D::kTableColumn,
            EDisplay::kTableColumnGroup => D::kTableColumnGroup,
            EDisplay::kTableCell => D::kTableCell,
            EDisplay::kTableCaption => D::kTableCaption,
            EDisplay::kFlowRoot => D::kFlowRoot,
            EDisplay::kMath => D::kMath,
            EDisplay::kBlockMath => D::kBlockMath,
            EDisplay::kRuby => D::kRuby,
            EDisplay::kBlockRuby => D::kBlockRuby,
            EDisplay::kRubyText => D::kRubyText,
            EDisplay::kGridLanes | EDisplay::kInlineGridLanes => D::kGridLanes,
            EDisplay::kLayoutCustom | EDisplay::kInlineLayoutCustom => D::kCustom,
            EDisplay::kNone | EDisplay::kContents => D::kInline,
        },
        position: match style.GetPosition() {
            foundation::EPosition::kStatic => P::kStatic,
            foundation::EPosition::kRelative => P::kRelative,
            foundation::EPosition::kAbsolute => P::kAbsolute,
            foundation::EPosition::kFixed => P::kFixed,
            foundation::EPosition::kSticky => P::kSticky,
        },
        floating: match style.Floating() {
            foundation::EFloat::kNone => F::kNone,
            foundation::EFloat::kLeft => F::kLeft,
            foundation::EFloat::kRight => F::kRight,
            foundation::EFloat::kInlineStart => F::kInlineStart,
            foundation::EFloat::kInlineEnd => F::kInlineEnd,
        },
        writing_mode: match style.GetWritingMode() {
            foundation::WritingMode::kHorizontalTb => {
                crate::internal::layout_input::WritingMode::kHorizontalTb
            }
            foundation::WritingMode::kVerticalRl | foundation::WritingMode::kSidewaysRl => {
                crate::internal::layout_input::WritingMode::kVerticalRl
            }
            foundation::WritingMode::kVerticalLr | foundation::WritingMode::kSidewaysLr => {
                crate::internal::layout_input::WritingMode::kVerticalLr
            }
        },
        direction: match style.Direction() {
            foundation::TextDirection::kLtr => crate::internal::layout_input::TextDirection::kLtr,
            foundation::TextDirection::kRtl => crate::internal::layout_input::TextDirection::kRtl,
        },
        overflow_x: overflow(style.OverflowX()),
        overflow_y: overflow(style.OverflowY()),
    }
}

/// Convert the older direct Layout input API. Production DOM attachment uses
/// `BorrowComputedStyle` and never calls this full conversion.
pub fn ProjectComputedStyle(
    computed_style: &ComputedStyle,
) -> Result<LayoutComputedStyle, StyleProjectionError> {
    let _trace = browser_tracing::span("layout_input", "Layout.ProjectComputedStyle");
    let mut resolved = LayoutComputedStyle::default();
    let font = computed_style.GetFontDescription();
    let mut ext = crate::internal::layout_input::ExtendedStyle::default();
    use crate::internal::layout_input::{
        Display, FloatSide, ListStylePosition as LayoutListStylePosition,
        ListStyleType as LayoutListStyleType, Overflow, Position, WhiteSpace,
    };
    resolved.display = match computed_style.Display() {
        EDisplay::kInline => Display::kInline,
        EDisplay::kBlock => Display::kBlock,
        EDisplay::kInlineBlock => Display::kInlineBlock,
        EDisplay::kListItem
        | EDisplay::kInlineListItem
        | EDisplay::kFlowRootListItem
        | EDisplay::kInlineFlowRootListItem => Display::kListItem,
        EDisplay::kFlex | EDisplay::kWebkitBox => Display::kFlex,
        EDisplay::kInlineFlex | EDisplay::kWebkitInlineBox => Display::kInlineFlex,
        EDisplay::kGrid => Display::kGrid,
        EDisplay::kInlineGrid => Display::kInlineGrid,
        EDisplay::kTable => Display::kTable,
        EDisplay::kInlineTable => Display::kInlineTable,
        EDisplay::kTableRowGroup => Display::kTableSection,
        EDisplay::kTableHeaderGroup => Display::kTableHeaderGroup,
        EDisplay::kTableFooterGroup => Display::kTableFooterGroup,
        EDisplay::kTableRow => Display::kTableRow,
        EDisplay::kTableColumn => Display::kTableColumn,
        EDisplay::kTableColumnGroup => Display::kTableColumnGroup,
        EDisplay::kTableCell => Display::kTableCell,
        EDisplay::kTableCaption => Display::kTableCaption,
        EDisplay::kFlowRoot => Display::kFlowRoot,
        EDisplay::kMath => Display::kMath,
        EDisplay::kBlockMath => Display::kBlockMath,
        EDisplay::kRuby => Display::kRuby,
        EDisplay::kBlockRuby => Display::kBlockRuby,
        EDisplay::kRubyText => Display::kRubyText,
        EDisplay::kGridLanes | EDisplay::kInlineGridLanes => Display::kGridLanes,
        EDisplay::kLayoutCustom | EDisplay::kInlineLayoutCustom => Display::kCustom,
        // Box generation/contents flags carry the structural distinction;
        // SVG's display:none g uses its dedicated hidden-container object.
        EDisplay::kNone | EDisplay::kContents => Display::kInline,
    };
    resolved.position = match computed_style.GetPosition() {
        foundation::EPosition::kStatic => Position::kStatic,
        foundation::EPosition::kRelative => Position::kRelative,
        foundation::EPosition::kAbsolute => Position::kAbsolute,
        foundation::EPosition::kFixed => Position::kFixed,
        foundation::EPosition::kSticky => Position::kSticky,
    };
    resolved.floating = match computed_style.Floating() {
        foundation::EFloat::kNone => FloatSide::kNone,
        foundation::EFloat::kLeft => FloatSide::kLeft,
        foundation::EFloat::kRight => FloatSide::kRight,
        foundation::EFloat::kInlineStart => FloatSide::kInlineStart,
        foundation::EFloat::kInlineEnd => FloatSide::kInlineEnd,
    };
    use layoutng_style::css::white_space::EWhiteSpace;
    ext.white_space = match computed_style.WhiteSpace() {
        EWhiteSpace::kNormal => WhiteSpace::kNormal,
        EWhiteSpace::kNowrap => WhiteSpace::kNowrap,
        EWhiteSpace::kPre => WhiteSpace::kPre,
        EWhiteSpace::kPreLine => WhiteSpace::kPreLine,
        EWhiteSpace::kPreWrap => WhiteSpace::kPreWrap,
        EWhiteSpace::kBreakSpaces => WhiteSpace::kBreakSpaces,
        value if value.bits() == 2 => WhiteSpace::kPreserveBreaksNowrap,
        value if value.bits() == 3 => WhiteSpace::kBreakSpacesNowrap,
        _ => unreachable!("white-space collapse/wrap fields exceeded their bit widths"),
    };
    let overflow = |value| match value {
        foundation::EOverflow::kVisible => Overflow::kVisible,
        foundation::EOverflow::kHidden => Overflow::kHidden,
        foundation::EOverflow::kScroll => Overflow::kScroll,
        foundation::EOverflow::kAuto | foundation::EOverflow::kOverlay => Overflow::kAuto,
        foundation::EOverflow::kClip => Overflow::kClip,
    };
    ext.overflow_x = overflow(computed_style.OverflowX());
    ext.overflow_y = overflow(computed_style.OverflowY());
    ext.list_style_position = match computed_style.ListStylePosition() {
        foundation::EListStylePosition::kOutside => LayoutListStylePosition::kOutside,
        foundation::EListStylePosition::kInside => LayoutListStylePosition::kInside,
    };
    ext.list_style_image = ProjectListMarkerImage(computed_style);
    ext.list_style_type = ReadListStyleType(computed_style);

    // FontDescription::ComputedSize already includes EffectiveZoom. The
    // neutral layout snapshot is author-space and ZoomedStyle applies that
    // factor at the boundary exactly once.
    ext.font_size = font.SpecifiedSize() as f64;
    ext.font_weight = font.Weight().ToFloat() as f64;
    ext.font_italic = font.Style().ToFloat() != 0.0;
    ext.font_smoothing = match font.FontSmoothing() {
        font_engine::FontSmoothingMode::kNoSmoothing => {
            crate::internal::layout_input::FontSmoothing::kNone
        }
        font_engine::FontSmoothingMode::kAntialiased => {
            crate::internal::layout_input::FontSmoothing::kAntialiased
        }
        font_engine::FontSmoothingMode::kSubpixelAntialiased => {
            crate::internal::layout_input::FontSmoothing::kSubpixelAntialiased
        }
        font_engine::FontSmoothingMode::kAutoSmoothing => {
            crate::internal::layout_input::FontSmoothing::kAuto
        }
    };
    // cpp: font_description_data.cc:127-145. The computed CSS value keeps a
    // Length/percentage, while Layout consumes the used spacing resolved from
    // the current computed font size.
    let effective_zoom = computed_style.EffectiveZoom();
    let unzoom = |value: f32| {
        if effective_zoom > 0.0 {
            value as f64 / effective_zoom as f64
        } else {
            value as f64
        }
    };
    ext.letter_spacing = unzoom(font.LetterSpacing());
    ext.word_spacing = unzoom(font.WordSpacing());
    ext.effective_zoom = effective_zoom;
    ext.zoom = computed_style.Zoom();
    ext.has_author_background = computed_style.HasAuthorBackground();
    ext.has_author_border = computed_style.HasAuthorBorder();
    ext.has_author_border_radius = computed_style.HasAuthorBorderRadius();
    let mut family = font.Family() as *const font_engine::FontFamily;
    while !family.is_null() {
        let f = unsafe { &*family };
        if !f.FamilyName().empty() {
            ext.font_families.push(f.FamilyName().Utf8())
        }
        family = f.Next()
    }

    // cpp: ComputedStyle::GetCurrentColor/BorderWidth; StyleColor::Resolve.
    // Paint's neutral projection shares these scalar results with the computed style
    // object used by layout; complex paint payloads stay in their own mapping.
    let current = computed_style.GetCurrentColor(None);
    let color=|value:&layoutng_style::css::style_color::StyleColor|->Result<crate::internal::layout_input_types::Color,StyleProjectionError>{
        let value=value.Resolve(current,computed_style.UsedColorScheme(),None);
        let (red,green,blue)=value.ToSRGB();
        Ok(crate::internal::layout_input_types::Color{red,green,blue,alpha:value.Alpha()})
    };
    resolved.paint.color = color(computed_style.Color())?;
    resolved.paint.background_color = color(computed_style.BackgroundColor())?;
    resolved.paint.respect_image_orientation = computed_style.ImageOrientation()
        == foundation::RespectImageOrientationEnum::kRespectImageOrientation;
    ProjectBorderRadii(computed_style, &mut resolved.paint);
    ProjectShadows(computed_style, &mut resolved.paint, color)?;
    // cpp: computed style.h SVG fill/stroke and style_transform.cc.  Layout
    // consumes a renderer-neutral projection, but the computed style ComputedStyle is
    // the single source of truth for both CSS declarations and SVG
    // presentation attributes.  Do not leave these fields at PaintStyleData's
    // defaults after the computed style cascade has resolved them.
    use layoutng_style::style::svg_paint::SVGPaintType;
    let project_svg_paint = |paint: &layoutng_style::style::svg_paint::SVGPaint,
                             opacity: f32|
     -> Result<
        Option<crate::internal::layout_input_types::Color>,
        StyleProjectionError,
    > {
        if paint.paint_type == SVGPaintType::kNone {
            return Ok(None);
        }
        if !paint.HasColor() {
            return Ok(None);
        }
        let mut projected = color(paint.GetColor())?;
        projected.alpha *= opacity;
        Ok(Some(projected))
    };
    resolved.paint.svg_fill =
        project_svg_paint(computed_style.FillPaint(), computed_style.FillOpacity())?;
    resolved.paint.svg_stroke =
        project_svg_paint(computed_style.StrokePaint(), computed_style.StrokeOpacity())?;
    resolved.paint.svg_fill_reference = computed_style
        .FillPaint()
        .HasUrl()
        .then(|| computed_style.FillPaint().GetUrl().Utf8());
    resolved.paint.svg_stroke_reference = computed_style
        .StrokePaint()
        .HasUrl()
        .then(|| computed_style.StrokePaint().GetUrl().Utf8());
    resolved.paint.svg_fill_current_color = computed_style.FillPaint().HasCurrentColor();
    resolved.paint.svg_stroke_current_color = computed_style.StrokePaint().HasCurrentColor();

    // SVGResources freezes pattern/marker content before Layout's ordinary
    // fragment export. Project the same canonical ComputedStyle stroke facts
    // here that paint_style_export::SvgStrokeStyle later publishes for normal
    // SVG shapes; otherwise resource primitives silently retain
    // PaintStyleData defaults (notably stroke-width:1).
    let length_parts = |value: &foundation::Length| -> (f64, f64) {
        if value.IsFixed() {
            return (value.Pixels() as f64, 0.0);
        }
        if value.IsPercent() {
            return (0.0, value.PercentValue() as f64);
        }
        if value.IsCalculated() {
            let calculation = value.GetCalculationValue();
            if !calculation.IsExpression() {
                let pair = calculation.GetPixelsAndPercent();
                return (
                    if pair.has_explicit_pixels {
                        pair.pixels as f64
                    } else {
                        0.0
                    },
                    if pair.has_explicit_percent {
                        pair.percent as f64
                    } else {
                        0.0
                    },
                );
            }
        }
        (0.0, 0.0)
    };
    resolved.paint.svg_stop_color = color(computed_style.StopColor())?;
    resolved.paint.svg_stop_current_color = computed_style.StopColor().IsCurrentColor();
    resolved.paint.svg_stop_opacity = computed_style.StopOpacity();
    resolved.paint.svg_stroke_width = length_parts(computed_style.StrokeWidth().length()).0;
    resolved.paint.svg_stroke_dash_offset = length_parts(computed_style.StrokeDashOffset()).0;
    let dash_array = computed_style.StrokeDashArray();
    resolved.paint.svg_stroke_dash_array = if dash_array.is_null() {
        Vec::new()
    } else {
        unsafe { &*dash_array }
            .iter()
            .map(|length| length_parts(length).0)
            .collect()
    };
    resolved.paint.svg_stroke_line_cap = match computed_style.CapStyle() {
        foundation::LineCap::kButtCap => crate::internal::paint_input::SvgStrokeLineCap::kButt,
        foundation::LineCap::kRoundCap => crate::internal::paint_input::SvgStrokeLineCap::kRound,
        foundation::LineCap::kSquareCap => crate::internal::paint_input::SvgStrokeLineCap::kSquare,
    };
    resolved.paint.svg_stroke_line_join = match computed_style.JoinStyle() {
        foundation::LineJoin::kMiterJoin => crate::internal::paint_input::SvgStrokeLineJoin::kMiter,
        foundation::LineJoin::kRoundJoin => crate::internal::paint_input::SvgStrokeLineJoin::kRound,
        foundation::LineJoin::kBevelJoin => crate::internal::paint_input::SvgStrokeLineJoin::kBevel,
    };
    resolved.paint.svg_stroke_miter_limit = computed_style.StrokeMiterLimit() as f64;
    resolved.paint.svg_fill_even_odd =
        computed_style.FillRule() == foundation::WindRule::RULE_EVENODD;
    resolved.paint.svg_non_scaling_stroke =
        computed_style.VectorEffect() == foundation::EVectorEffect::kNonScalingStroke;
    resolved.paint.svg_shape_antialias = !matches!(
        computed_style.ShapeRendering(),
        foundation::EShapeRendering::kCrispedges | foundation::EShapeRendering::kOptimizespeed
    );
    use crate::internal::paint_input::SvgPaintComponent::{
        kFill as SvgFill, kMarkers as SvgMarkers, kStroke as SvgStroke,
    };
    resolved.paint.svg_paint_order = match computed_style.PaintOrder() {
        layoutng_style::style::computed_style_constants::EPaintOrder::kPaintOrderNormal
        | layoutng_style::style::computed_style_constants::EPaintOrder::kPaintOrderFillStrokeMarkers => {
            [SvgFill, SvgStroke, SvgMarkers]
        }
        layoutng_style::style::computed_style_constants::EPaintOrder::kPaintOrderFillMarkersStroke => {
            [SvgFill, SvgMarkers, SvgStroke]
        }
        layoutng_style::style::computed_style_constants::EPaintOrder::kPaintOrderStrokeFillMarkers => {
            [SvgStroke, SvgFill, SvgMarkers]
        }
        layoutng_style::style::computed_style_constants::EPaintOrder::kPaintOrderStrokeMarkersFill => {
            [SvgStroke, SvgMarkers, SvgFill]
        }
        layoutng_style::style::computed_style_constants::EPaintOrder::kPaintOrderMarkersFillStroke => {
            [SvgMarkers, SvgFill, SvgStroke]
        }
        layoutng_style::style::computed_style_constants::EPaintOrder::kPaintOrderMarkersStrokeFill => {
            [SvgMarkers, SvgStroke, SvgFill]
        }
    };

    resolved.paint.transform = ProjectTransform(computed_style);
    resolved.paint.border_colors = [
        color(computed_style.BorderTopColor())?,
        color(computed_style.BorderRightColor())?,
        color(computed_style.BorderBottomColor())?,
        color(computed_style.BorderLeftColor())?,
    ];
    resolved.paint.outline_color = color(computed_style.OutlineColor())?;
    resolved.paint.outline_width =
        if computed_style.OutlineStyle() == foundation::EBorderStyle::kNone {
            0.0
        } else {
            unzoom(*computed_style.OutlineWidth() as f32)
        };
    resolved.paint.outline_offset = unzoom(*computed_style.OutlineOffset() as f32);
    resolved.paint.opacity = computed_style.Opacity();
    resolved.paint.visible = computed_style.Visibility() == foundation::EVisibility::kVisible;
    resolved.paint.pointer_events_none =
        computed_style.PointerEvents() == foundation::EPointerEvents::kNone;
    resolved.paint.cursor = computed_style.Cursor();
    resolved.paint.z_index = (!computed_style.HasAutoZIndex()).then(|| computed_style.ZIndex());
    resolved.border.top = unzoom(computed_style.BorderTopWidth() as f32);
    resolved.border.right = unzoom(computed_style.BorderRightWidth() as f32);
    resolved.border.bottom = unzoom(computed_style.BorderBottomWidth() as f32);
    resolved.border.left = unzoom(computed_style.BorderLeftWidth() as f32);
    fn border(style: foundation::EBorderStyle) -> crate::internal::layout_input::BorderLineStyle {
        use crate::internal::layout_input::BorderLineStyle as P;
        use foundation::EBorderStyle as N;
        match style {
            N::kNone => P::kNone,
            N::kHidden => P::kHidden,
            N::kSolid => P::kSolid,
            N::kDashed => P::kDashed,
            N::kDotted => P::kDotted,
            N::kDouble => P::kDouble,
            N::kGroove => P::kGroove,
            N::kRidge => P::kRidge,
            N::kInset => P::kInset,
            N::kOutset => P::kOutset,
        }
    }
    resolved.border_styles = [
        border(computed_style.BorderTopStyle()),
        border(computed_style.BorderRightStyle()),
        border(computed_style.BorderBottomStyle()),
        border(computed_style.BorderLeftStyle()),
    ];
    resolved.paint.outline_style = border(computed_style.OutlineStyle());
    ProjectGeometry(computed_style, &mut resolved, &mut ext);
    ProjectShapeOutside(computed_style, &mut ext);
    resolved.extended = Some(ext.into());
    resolved.style = computed_style as *const ComputedStyle;
    Ok(resolved)
}
