//! Projection of immutable Blink computed values into Rechrom's neutral
//! DOM-to-layout snapshot. Layout keeps using the native `ComputedStyle`;
//! this projection supplies change classification and Web API snapshots.

use dom::persistent_document::ResolvedNodeStyle;
use foundation::Length;
use layoutng::internal::layout_input::{
    ExtendedStyle, FlexDirection, ShapeCoordinate, ShapeOutsideGeometry, ShapeOutsideKind,
    ShapePoint, ShapeRadius, ShapeRadiusKind, ShapeReferenceBox,
    TextDirection as LayoutTextDirection, WritingMode as LayoutWritingMode,
};
use layoutng::internal::paint_input::{
    PaintCornerRadii, PaintCornerRadius, PaintStyleData, PaintTransform, PaintTransformOperation,
    PaintTransformOperationKind,
};
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
// Undo the native storage zoom while projecting; percentages stay dimensionless.
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

/// Project the native ShapeValue into Layout's retained, renderer-neutral
/// exclusion input. The native value remains canonical; this is the minimal
/// package boundary needed by the translated float-shape algorithm.
// cpp: core/layout/shapes/shape_outside_info.cc:194-236
pub(crate) fn ProjectShapeOutside(native: &ComputedStyle, ext: &mut ExtendedStyle) {
    // shape-margin is stored zoomed in ComputedStyle while ExtendedStyle is
    // author-space and css_zoom applies EffectiveZoom exactly once.
    let margin = ProjectUnzoomedLength(native.ShapeMargin(), native.EffectiveZoom());
    ext.shape_margin = ShapeCoordinate {
        pixels: margin.pixels.unwrap_or(0.0),
        percentage: margin.percent.unwrap_or(0.0),
        calculation: margin.calculation,
    };
    ext.shape_image_threshold = native.ShapeImageThreshold() as f64;

    let Some(value) = (unsafe { native.ShapeOutside().as_ref() }) else {
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

// cpp: core/style/computed_style_base.h Border*Radius
// cpp: core/paint/border_shape_painter.cc ResolveCornerRadii
// ComputedStyle owns the canonical, zoomed LengthSize values.  The neutral
// snapshot keeps fixed components in author-space and percentage components
// unresolved so Layout can apply zoom once and Paint can resolve them against
// the final border box.
pub(crate) fn ProjectBorderRadii(native: &ComputedStyle, paint: &mut PaintStyleData) {
    let zoom = native.EffectiveZoom();
    let corners = [
        ProjectCornerRadius(native.BorderTopLeftRadius(), zoom),
        ProjectCornerRadius(native.BorderTopRightRadius(), zoom),
        ProjectCornerRadius(native.BorderBottomRightRadius(), zoom),
        ProjectCornerRadius(native.BorderBottomLeftRadius(), zoom),
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
    native: &ComputedStyle,
    paint: &mut PaintStyleData,
    resolve_color: F,
) -> Result<(), E>
where
    F: Fn(
        &layoutng_style::css::style_color::StyleColor,
    ) -> Result<layoutng::internal::layout_input_types::Color, E>,
{
    use layoutng::internal::layout_input::{Offset, PaintShadow};
    use layoutng_style::style::shadow_data::ShadowStyle;

    let zoom = native.EffectiveZoom() as f64;
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
    paint.box_shadows = project(native.BoxShadow())?;
    paint.text_shadows = project(native.TextShadow())?;
    Ok(())
}

pub(crate) fn ProjectTransform(native: &ComputedStyle) -> Option<PaintTransform> {
    use foundation::{TransformOperationType, TranslateTransformOperation};
    let mut projected = PaintTransform::default();
    for operation in native.Transform().Operations().iter() {
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
                pixels: layoutng::internal::layout_input::Offset {
                    x: x.pixels.unwrap_or(0.0),
                    y: y.pixels.unwrap_or(0.0),
                },
                percentages: layoutng::internal::layout_input::Offset {
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
            matrix: layoutng::internal::layout_input::TransformMatrix {
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

// cpp: layoutng_style/style/computed_style_base.h Width/Height/Margin*/Padding*/Inset
// cpp: core/css/resolver/style_resolver.cc ApplyProperties
pub(crate) fn ProjectGeometry(
    native: &ComputedStyle,
    resolved: &mut ResolvedNodeStyle,
    ext: &mut ExtendedStyle,
) {
    let effective_zoom = native.EffectiveZoom();
    ProjectDimension(
        native.Width(),
        effective_zoom,
        &mut resolved.style.width,
        &mut ext.width_percent,
        &mut ext.width_calculated,
    );
    ProjectDimension(
        native.Height(),
        effective_zoom,
        &mut resolved.style.height,
        &mut ext.height_percent,
        &mut ext.height_calculated,
    );
    ProjectDimension(
        native.MinWidth(),
        effective_zoom,
        &mut resolved.style.min_width,
        &mut ext.min_width_percent,
        &mut ext.min_width_calculated,
    );
    ProjectDimension(
        native.MaxWidth(),
        effective_zoom,
        &mut resolved.style.max_width,
        &mut ext.max_width_percent,
        &mut ext.max_width_calculated,
    );
    ProjectDimension(
        native.MinHeight(),
        effective_zoom,
        &mut resolved.style.min_height,
        &mut ext.min_height_percent,
        &mut ext.min_height_calculated,
    );
    ProjectDimension(
        native.MaxHeight(),
        effective_zoom,
        &mut resolved.style.max_height,
        &mut ext.max_height_percent,
        &mut ext.max_height_calculated,
    );
    ProjectDimension(
        native.FlexBasis(),
        effective_zoom,
        &mut resolved.style.flex_basis,
        &mut ext.flex_basis_percent,
        &mut ext.flex_basis_calculated,
    );

    let margins = [
        native.MarginTop(),
        native.MarginRight(),
        native.MarginBottom(),
        native.MarginLeft(),
    ];
    let margin_pixels = [
        &mut resolved.style.margin.top,
        &mut resolved.style.margin.right,
        &mut resolved.style.margin.bottom,
        &mut resolved.style.margin.left,
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
        native.PaddingTop(),
        native.PaddingRight(),
        native.PaddingBottom(),
        native.PaddingLeft(),
    ];
    let padding_pixels = [
        &mut resolved.style.padding.top,
        &mut resolved.style.padding.right,
        &mut resolved.style.padding.bottom,
        &mut resolved.style.padding.left,
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

    let insets = [native.Top(), native.Right(), native.Bottom(), native.Left()];
    let inset_pixels = [
        &mut resolved.style.top,
        &mut resolved.style.right,
        &mut resolved.style.bottom,
        &mut resolved.style.left,
    ];
    for (index, (value, pixels)) in insets.into_iter().zip(inset_pixels).enumerate() {
        let projected = ProjectUnzoomedLength(value, effective_zoom);
        *pixels = projected.pixels;
        ext.inset_percentages[index] = projected.percent;
        ext.inset_calculated[index] = projected.calculated;
    }

    resolved.style.flex_grow = native.FlexGrow();
    resolved.style.flex_shrink = native.FlexShrink();
    resolved.style.flex_direction = match native.FlexDirection() {
        foundation::EFlexDirection::kRow => FlexDirection::kRow,
        foundation::EFlexDirection::kRowReverse => FlexDirection::kRowReverse,
        foundation::EFlexDirection::kColumn => FlexDirection::kColumn,
        foundation::EFlexDirection::kColumnReverse => FlexDirection::kColumnReverse,
    };
    resolved.style.flex_wrap = native.FlexWrap().GetWrapMode() != FlexWrapMode::kNowrap;
    ext.wrap_reverse = native.FlexWrap().GetWrapMode() == FlexWrapMode::kWrapReverse;
    resolved.style.direction = match native.Direction() {
        foundation::TextDirection::kLtr => LayoutTextDirection::kLtr,
        foundation::TextDirection::kRtl => LayoutTextDirection::kRtl,
    };
    resolved.style.writing_mode = match native.GetWritingMode() {
        foundation::WritingMode::kHorizontalTb => LayoutWritingMode::kHorizontalTb,
        foundation::WritingMode::kVerticalRl | foundation::WritingMode::kSidewaysRl => {
            LayoutWritingMode::kVerticalRl
        }
        foundation::WritingMode::kVerticalLr | foundation::WritingMode::kSidewaysLr => {
            LayoutWritingMode::kVerticalLr
        }
    };
    resolved.style.column_count = native.ColumnCount() as u32;

    let line_height = ProjectUnzoomedLength(native.LineHeight(), effective_zoom);
    ext.line_height = line_height.pixels;
    ext.line_height_percent = line_height.percent;
    let text_indent = ProjectUnzoomedLength(native.TextIndent(), effective_zoom);
    ext.text_indent = text_indent.pixels.unwrap_or(0.0);
    ext.text_indent_percent = text_indent.percent;
    ext.text_indent_calculated = text_indent.calculated;

    let row_gap = native
        .RowGap()
        .as_ref()
        .map(|value| ProjectUnzoomedLength(value, effective_zoom));
    let column_gap = native
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
        resolved.style.gap = ext.row_gap.unwrap_or(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_style::style::computed_style::ComputedStyleBuilder;

    #[test]
    fn projects_fixed_percent_auto_and_calc_geometry_without_reparsing_css() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetWidthOwned(Length::Fixed(42.0));
        builder.SetHeightOwned(Length::Percent(25.0));
        builder.SetMarginTop(Length::Auto());
        builder.SetPaddingLeft(&Length::Fixed(7.0));
        builder.SetLeftOwned(Length::Percent(10.0));
        let native = unsafe { &*builder.TakeStyle() };
        let mut resolved = ResolvedNodeStyle::default();
        let mut ext = ExtendedStyle::default();
        ProjectGeometry(native, &mut resolved, &mut ext);
        assert_eq!(resolved.style.width, Some(42.0));
        assert_eq!(ext.height_percent, Some(25.0));
        assert!(ext.margin_auto[0]);
        assert_eq!(resolved.style.padding.left, 7.0);
        assert_eq!(ext.inset_percentages[3], Some(10.0));
    }

    #[test]
    fn projects_transform_operations_without_resolving_percentages_early() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        let mut operations = foundation::TransformOperations::default();
        operations.PushTranslate(Length::Percent(25.0), Length::Fixed(12.0));
        operations.PushMatrix3D(foundation::gfx::Transform::ColMajor(&[
            0.5, 0.0, 0.0, 0.0, // first column
            0.0, 0.5, 0.0, 0.0, // second column
            0.0, 0.0, 1.0, 0.0, // third column
            0.0, 0.0, 0.0, 1.0, // fourth column
        ]));
        builder.SetTransform(&operations);
        let native = unsafe { &*builder.TakeStyle() };

        let projected = ProjectTransform(native).unwrap();
        assert_eq!(projected.operations.len(), 2);
        assert_eq!(
            projected.operations[0].kind,
            PaintTransformOperationKind::kTranslate
        );
        assert_eq!(projected.operations[0].percentages.x, 25.0);
        assert_eq!(projected.operations[0].pixels.y, 12.0);
        assert_eq!(projected.operations[1].matrix.values[0], 0.5);
        assert_eq!(projected.operations[1].matrix.values[5], 0.5);
    }

    #[test]
    fn projects_native_zoomed_lengths_back_to_author_space() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetEffectiveZoom(0.5);
        // Native ComputedStyle stores this author 200px width as 100px.
        builder.SetWidthOwned(Length::Fixed(100.0));
        builder.SetPaddingLeft(&Length::Fixed(12.0));
        let native = unsafe { &*builder.TakeStyle() };
        let mut resolved = ResolvedNodeStyle::default();
        let mut ext = ExtendedStyle::default();
        ProjectGeometry(native, &mut resolved, &mut ext);
        assert_eq!(resolved.style.width, Some(200.0));
        assert_eq!(resolved.style.padding.left, 24.0);
        ext.effective_zoom = 0.5;
        resolved.style.extended = Some(ext);
        let used = layoutng::internal::css_zoom::ZoomedStyle(&resolved.style);
        assert_eq!(used.width, Some(100.0));
        assert_eq!(used.padding.left, 12.0);
    }
}
