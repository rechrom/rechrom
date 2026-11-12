#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::native_path::{NativePath, ShapeFillGeometry, ShapeStrokeGeometry, StrokeData};
use foundation::{gfx, DynamicTo, Traceable, Visitor};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_input::SvgShapeData;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use layoutng_geometry::geometry::logical_size::LogicalSize;
use std::sync::Arc;

// cpp: layoutng_svg/layout_svg_shape.h:10-22
#[repr(C)]
pub struct LayoutSVGShape {
    flow: LayoutBlockFlow,
    pub(crate) effect_bounds: crate::svg_effect_bounds::SvgEffectBoundsCache,
    shape: Option<Arc<SvgShapeData>>,
    path: Option<Arc<dyn NativePath>>,
    marker_positions: Vec<crate::svg_marker_data::MarkerPosition>,
    marker_bounds: gfx::RectF,
    stroke_path_cache: std::cell::RefCell<Option<(Arc<dyn NativePath>, AffineTransform)>>,
}

impl LayoutSVGShape {
    // cpp: layoutng_svg/layout_svg_shape.cc:10-10
    pub fn new(element: *mut Element) -> Self {
        let flow = LayoutBlockFlow::new(element.cast());
        flow.SetRuntimeClass(LayoutObjectClass::SvgShape);
        Self {
            flow,
            effect_bounds: Default::default(),
            shape: None,
            path: None,
            stroke_path_cache: Default::default(),
            marker_positions: Vec::new(),
            marker_bounds: Default::default(),
        }
    }

    // cpp: layoutng_svg/layout_svg_shape.h:14-14
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGShape"
    }

    // cpp: layoutng_svg/layout_svg_shape.h:15-15
    pub fn CreatesNewFormattingContext(&self) -> bool {
        true
    }

    // cpp: layoutng_svg/layout_svg_shape.h:16-16
    pub fn CanHaveChildren(&self) -> bool {
        false
    }

    // cpp: layoutng_svg/layout_svg_shape.h:20-21
    pub fn IsSVG(&self) -> bool {
        true
    }

    pub fn IsSVGShape(&self) -> bool {
        true
    }

    /// Chromium layout_svg_shape.cc:300-330. The same retained native Path
    /// supplies fill containment and geometry; no fragment-box substitution.
    pub fn GetPath(&self) -> Option<&dyn NativePath> {
        self.path.as_deref()
    }
    pub fn IsShapeEmpty(&self) -> bool {
        use layoutng_assembly::internal::layout_input::SvgShapeGeometry;
        let Some(shape) = self.shape.as_ref() else {
            return true;
        };
        match shape.geometry {
            SvgShapeGeometry::kPath => self.GetPath().is_none_or(|p| p.IsEmpty()),
            SvgShapeGeometry::kRectangle | SvgShapeGeometry::kEllipse => {
                shape.bounds_width <= 0. || shape.bounds_height <= 0.
            }
        }
    }
    /// LayoutSVGRect::ShapeDependentFillContains (137-145) and
    /// LayoutSVGEllipse::ShapeDependentFillContains (164-172).
    pub fn FillGeometry(&self) -> ShapeFillGeometry {
        use layoutng_assembly::internal::layout_input::SvgShapeGeometry;
        let Some(shape) = self.shape.as_ref() else {
            return ShapeFillGeometry::Path;
        };
        let bounds = self.ObjectBoundingBox();
        match shape.geometry {
            SvgShapeGeometry::kRectangle
                if shape.rectangle_radius_x <= 0. && shape.rectangle_radius_y <= 0. =>
            {
                ShapeFillGeometry::Rectangle(bounds)
            }
            SvgShapeGeometry::kEllipse => ShapeFillGeometry::Ellipse {
                center: gfx::PointF::new(
                    bounds.x() + bounds.width() / 2.,
                    bounds.y() + bounds.height() / 2.,
                ),
                radii: gfx::SizeF::new(bounds.width() / 2., bounds.height() / 2.),
            },
            _ => ShapeFillGeometry::Path,
        }
    }
    pub fn RetainedShape(&self) -> Option<&Arc<SvgShapeData>> {
        self.shape.as_ref()
    }
    pub fn ObjectBoundingBox(&self) -> gfx::RectF {
        use layoutng_assembly::internal::layout_input::SvgShapeGeometry;
        if let Some(shape) = self
            .shape
            .as_ref()
            .filter(|s| s.geometry != SvgShapeGeometry::kPath)
        {
            // Rect/ellipse retain their source geometric bounds even when a
            // zero dimension makes the platform Path empty.
            return gfx::RectF::new(
                gfx::PointF::new(shape.bounds_offset.x as f32, shape.bounds_offset.y as f32),
                gfx::SizeF::new(shape.bounds_width as f32, shape.bounds_height as f32),
            );
        }
        self.path
            .as_ref()
            .map_or_else(gfx::RectF::default, |p| p.BoundingRect())
    }
    /// Chromium layout_svg_shape.cc:233-256: stroke bounds ignore dashes,
    /// union the fill bounds, and use the platform Path stroke operation.
    pub fn StrokeBoundingBox(&self) -> gfx::RectF {
        let mut bounds = self.ObjectBoundingBox();
        let Some(path) = self.GetPath() else {
            return bounds;
        };
        if !self.StyleRef().HasStroke() || self.IsShapeEmpty() {
            return bounds;
        }
        bounds.Union(path.StrokeBoundingRect(self.StrokeData()));
        bounds
    }
    pub fn HasNonScalingStroke(&self) -> bool {
        self.StyleRef().VectorEffect() == foundation::EVectorEffect::kNonScalingStroke
    }
    pub fn StrokeWidth(&self) -> f32 {
        let viewport = crate::svg_viewport::ResolveViewport(self);
        crate::svg_length_functions::ValueForUnzoomedLength(
            self.StyleRef().StrokeWidth(),
            self.StyleRef().EffectiveZoom(),
            ((viewport.width().powi(2) + viewport.height().powi(2)) / 2.).sqrt(),
        )
    }
    fn StrokeData(&self) -> StrokeData {
        let s = self.StyleRef();
        StrokeData {
            width: self.StrokeWidth(),
            miter_limit: s.StrokeMiterLimit(),
            cap: s.CapStyle(),
            join: s.JoinStyle(),
        }
    }
    pub fn ComputeRootTransform(&self) -> AffineTransform {
        let mut root = self.Parent();
        while let Some(object) = unsafe { root.as_ref() } {
            if object.IsSVGRoot() {
                break;
            }
            root = object.Parent();
        }
        assert!(!root.is_null(), "SVG shape retains its root owner");
        let mut transform = AffineTransform::FromTransform(
            &self.LocalToAncestorTransform(root.cast(), Default::default()),
        );
        let root = unsafe { &*root.cast::<crate::layout_svg_root::LayoutSVGRoot>() };
        let scale = root.ContainerScale();
        transform.Scale(scale.x() as f64, scale.y() as f64);
        transform
    }
    pub fn NonScalingStrokeTransform(&self) -> AffineTransform {
        let root = self.ComputeRootTransform();
        if foundation::RuntimeEnabledFeatures::SvgNewZoomEnabled() {
            root
        } else {
            let mut transform = AffineTransform::default();
            transform
                .Scale(
                    1. / self.StyleRef().EffectiveZoom() as f64,
                    1. / self.StyleRef().EffectiveZoom() as f64,
                )
                .PreConcat(root);
            transform
        }
    }
    pub fn ApproximateStrokeBoundingBox(&self, mut bounds: gfx::RectF) -> gfx::RectF {
        let width = self.StrokeWidth();
        if width <= 0. {
            return bounds;
        }
        let mut delta = width / 2.;
        let style = self.StyleRef();
        if self.shape.as_ref().is_some_and(|s| {
            s.geometry == layoutng_assembly::internal::layout_input::SvgShapeGeometry::kPath
        }) {
            let is_line = self.GetPath().is_some_and(|p| p.IsLine());
            if !is_line && style.JoinStyle() == foundation::LineJoin::kMiterJoin {
                let miter = style.StrokeMiterLimit();
                delta *= if miter < std::f32::consts::SQRT_2
                    && style.CapStyle() == foundation::LineCap::kSquareCap
                {
                    std::f32::consts::SQRT_2
                } else {
                    miter.max(1.)
                };
            } else if style.CapStyle() == foundation::LineCap::kSquareCap {
                delta *= std::f32::consts::SQRT_2;
            }
        }
        bounds.Inset(-delta);
        bounds
    }
    pub fn DecoratedBoundingBox(&self) -> gfx::RectF {
        let mut bounds = self.CalculateStrokeBoundingBox();
        bounds.Union(self.marker_bounds);
        bounds
    }
    fn CalculateStrokeBoundingBox(&self) -> gfx::RectF {
        let fill = self.ObjectBoundingBox();
        if !self.StyleRef().HasStroke() {
            return fill;
        }
        if self.HasNonScalingStroke() {
            let transform = self.NonScalingStrokeTransform();
            if !transform.IsInvertible() {
                return fill;
            }
            let Some(path) = self.GetPath() else {
                return fill;
            };
            let transformed = path.Transformed(&transform);
            let mut bounds = fill;
            bounds.Union(
                transform
                    .Inverse()
                    .MapRect(self.ApproximateStrokeBoundingBox(transformed.BoundingRect())),
            );
            bounds
        } else {
            self.ApproximateStrokeBoundingBox(fill)
        }
    }
    pub fn HitTestStrokeBoundingBox(&self) -> gfx::RectF {
        if self.StyleRef().HasStroke() {
            self.DecoratedBoundingBox()
        } else {
            self.ApproximateStrokeBoundingBox(self.ObjectBoundingBox())
        }
    }
    fn DashScaleFactor(&self) -> f32 {
        if self.StyleRef().StrokeDashArray().is_null() {
            return 1.;
        }
        let element = foundation::DynamicTo::<Element>(self.GetNode());
        let author = unsafe { element.as_ref() }
            .and_then(|e| e.InputElementData().as_ref())
            .and_then(|e| e.svg_path_length);
        let Some(author) = author else {
            return 1.;
        };
        let length = self.GetPath().map_or(0., |p| p.Length());
        if length == 0. {
            0.
        } else {
            (length / author.abs()).min(f32::MAX)
        }
    }
    pub fn StrokeGeometry(&self) -> ShapeStrokeGeometry {
        let style = self.StyleRef();
        let width = self.StrokeWidth();
        if !self.HasNonScalingStroke() && style.StrokeDashArray().is_null() {
            match self.FillGeometry() {
                ShapeFillGeometry::Rectangle(rect)
                    if style.JoinStyle() == foundation::LineJoin::kMiterJoin
                        && style.StrokeMiterLimit() >= 1.5 =>
                {
                    return ShapeStrokeGeometry::Rectangle { rect, width }
                }
                ShapeFillGeometry::Ellipse { center, radii } if radii.width() == radii.height() => {
                    return ShapeStrokeGeometry::Circle {
                        center,
                        radius: radii.width(),
                        width,
                    }
                }
                _ => {}
            }
        }
        if self.stroke_path_cache.borrow().is_none() {
            let path = self.GetPath().expect("nonempty native shape has its Path");
            let root = self.ComputeRootTransform();
            let host = if self.HasNonScalingStroke() {
                self.NonScalingStrokeTransform()
            } else {
                AffineTransform::default()
            };
            let transformed = self.HasNonScalingStroke().then(|| path.Transformed(&host));
            let stroke_source = transformed.as_deref().unwrap_or(path);
            let precision = ((root.A() * root.A() + root.B() * root.B())
                .max(root.C() * root.C() + root.D() * root.D()))
            .sqrt()
            .min(f32::MAX as f64) as f32;
            let viewport = crate::svg_viewport::ResolveViewport(self);
            let dimension = ((viewport.width().powi(2) + viewport.height().powi(2)) / 2.).sqrt();
            let scale = self.DashScaleFactor();
            let resolve = |length: &foundation::Length| {
                crate::svg_length_functions::ValueForLengthWithStyle(length, style, dimension)
                    * scale
            };
            let dashes = unsafe { style.StrokeDashArray().as_ref() }
                .map(|d| d.iter().map(resolve).collect::<Vec<_>>())
                .unwrap_or_default();
            let offset = resolve(style.StrokeDashOffset());
            *self.stroke_path_cache.borrow_mut() = Some((
                stroke_source.StrokePath(self.StrokeData(), &dashes, offset, precision),
                host,
            ));
        }
        let cache = self.stroke_path_cache.borrow();
        let (path, host_transform) = cache.as_ref().unwrap();
        ShapeStrokeGeometry::Path {
            path: path.clone(),
            host_transform: *host_transform,
        }
    }

    fn UpdateMarkerPositions(&mut self) {
        self.marker_positions.clear();
        if !self.StyleRef().HasMarkers()
            || !self.shape.as_ref().is_some_and(|shape| {
                shape.geometry == layoutng_assembly::internal::layout_input::SvgShapeGeometry::kPath
            })
        {
            return;
        }
        let element = DynamicTo::<Element>(self.GetNode());
        let path_element = unsafe { element.as_ref() }
            .and_then(|element| element.InputElementData().as_ref())
            .is_some_and(|data| data.svg_path_element);
        let stream = if path_element {
            unsafe { self.StyleRef().D().as_ref() }
        } else {
            None
        };
        self.marker_positions = if let Some(style_path) = stream {
            crate::svg_marker_data::BuildFromByteStream(style_path.ByteStream().Span())
        } else {
            self.GetPath()
                .map(crate::svg_marker_data::BuildFromPath)
                .unwrap_or_default()
        };
    }
    fn StrokeWidthForMarkerUnits(&self) -> f32 {
        let width = self.StrokeWidth();
        if self.HasNonScalingStroke() {
            let transform = self.NonScalingStrokeTransform();
            if !transform.IsInvertible() {
                return 0.;
            }
            let scale = (((transform.A() * transform.A() + transform.B() * transform.B())
                + (transform.C() * transform.C() + transform.D() * transform.D()))
                / 2.)
                .sqrt()
                .min(f32::MAX as f64) as f32;
            width / scale
        } else if foundation::RuntimeEnabledFeatures::SvgNewZoomEnabled() {
            width / self.StyleRef().EffectiveZoom()
        } else {
            width
        }
    }
    // cpp: svg_shape_painter.cc:256-280. Query the same retained positions,
    // native resources and source cycle owner at Paint time.
    pub(crate) fn MarkerPaints(
        &self,
    ) -> Vec<layoutng_assembly::internal::paint_input::SvgMarkerPaint> {
        if self.marker_positions.is_empty() {
            return Vec::new();
        }
        layoutng_assembly::internal::layout_pass_scope::LayoutPassScope::WithSVGResourceTree(|tree| {
            let style = self.StyleRef();
            let markers = [
                crate::svg_resources::ResolveMarker(tree, self, style.MarkerStartResource()),
                crate::svg_resources::ResolveMarker(tree, self, style.MarkerMidResource()),
                crate::svg_resources::ResolveMarker(tree, self, style.MarkerEndResource()),
            ];
            let width = self.StrokeWidthForMarkerUnits();
            self.marker_positions.iter().filter_map(|position| {
                let index = match position.kind {
                    crate::svg_marker_data::MarkerType::Start => 0,
                    crate::svg_marker_data::MarkerType::Mid => 1,
                    crate::svg_marker_data::MarkerType::End => 2,
                };
                let marker = markers[index].filter(|marker| marker.ShouldPaint())?;
                let object: &LayoutObject = marker;
                let client = unsafe { &*(object as *const LayoutObject as *const
                    foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient) };
                let viewport = marker.Viewport();
                Some(layoutng_assembly::internal::paint_input::SvgMarkerPaint {
                    resource_client_id: client.Id() as u64,
                    transform: crate::svg_viewport::Matrix(marker.MarkerTransformation(position, width)),
                    viewport_clip: matches!(marker.StyleRef().OverflowX(), foundation::EOverflow::kHidden | foundation::EOverflow::kClip | foundation::EOverflow::kScroll).then(||
                        layoutng_assembly::fragment_tree::FragmentPaintRect {
                            offset: layoutng_assembly::internal::layout_input::Offset {x: viewport.x() as f64,y:viewport.y() as f64},
                            size: layoutng_assembly::internal::layout_input::Size {width:viewport.width() as f64,height:viewport.height() as f64},
                        }),
                })
            }).collect()
        }).expect("SVG marker Paint borrows the native resource tree query scope")
    }
    fn UpdateMarkerBounds(&mut self) {
        self.marker_bounds = Default::default();
        if self.marker_positions.is_empty() || self.IsShapeEmpty() {
            return;
        }
        let bounds =
            layoutng_assembly::internal::layout_pass_scope::LayoutPassScope::WithSVGResourceTree(
                |tree| {
                    let style = self.StyleRef();
                    let markers = [
                        crate::svg_resources::ResolveMarker(
                            tree,
                            self,
                            style.MarkerStartResource(),
                        ),
                        crate::svg_resources::ResolveMarker(tree, self, style.MarkerMidResource()),
                        crate::svg_resources::ResolveMarker(tree, self, style.MarkerEndResource()),
                    ];
                    let mut bounds = gfx::RectF::default();
                    let width = self.StrokeWidthForMarkerUnits();
                    for position in &self.marker_positions {
                        let index = match position.kind {
                            crate::svg_marker_data::MarkerType::Start => 0,
                            crate::svg_marker_data::MarkerType::Mid => 1,
                            crate::svg_marker_data::MarkerType::End => 2,
                        };
                        if let Some(marker) = markers[index] {
                            bounds.Union(
                                marker.MarkerBoundaries(
                                    &marker.MarkerTransformation(position, width),
                                ),
                            );
                        }
                    }
                    bounds
                },
            )
            .expect("SVG marker bounds borrow the native resource tree scope");
        self.marker_bounds = bounds;
    }
    fn UpdateShapeFromElement(&mut self) {
        let element = DynamicTo::<Element>(self.GetNode());
        let shape = unsafe { element.as_ref() }
            .and_then(|e| e.InputElementData().as_ref())
            .and_then(|e| e.svg_shape.as_ref());
        if shape == self.shape.as_deref() {
            return;
        }
        let Some(shape) = shape else {
            self.path = None;
            self.shape = None;
            return;
        };
        let factory = unsafe { self.FactorySet().as_ref() }
            .and_then(|f| f.svg_shape_path)
            .expect("SVG Layout requires a native platform Path factory");
        self.stroke_path_cache.get_mut().take();
        self.path = Some(factory(shape));
        self.shape = Some(Arc::new(shape.clone()));
    }

    // cpp: layoutng_svg/layout_svg_shape.cc:12-19
    pub fn UpdateSVGLayout(&mut self, _info: &SVGLayoutInfo) -> SVGLayoutResult {
        self.stroke_path_cache.get_mut().take();
        self.UpdateShapeFromElement();
        self.UpdateMarkerPositions();
        let style = self.StyleRef();
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            style.GetWritingMode(),
            style.GetWritingDirection(),
            true,
            true,
            false,
        );
        builder.SetAvailableSize(LogicalSize::default());
        let space = builder.ToConstraintSpace();
        BlockNode::new((self as *mut Self).cast()).Layout(
            &space,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        self.UpdateMarkerBounds();
        SVGLayoutResult::new(true, true)
    }
}

impl Deref for LayoutSVGShape {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

impl DerefMut for LayoutSVGShape {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutSVGShape, flow) == 0);

impl Traceable for LayoutSVGShape {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}

// cpp: layoutng_svg/layout_svg_shape.h:24-29
impl foundation::DowncastFrom<LayoutObject> for LayoutSVGShape {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGShape()
    }
}

// layout_svg_shape.cc:70-132 preserves the stroke-cache invalidation owner.
pub fn SvgShapeStyleDidChange(
    flow: &mut LayoutBlockFlow,
    difference: layoutng_style::style::style_difference::StyleDifference,
    old: *const layoutng_style::style::computed_style::ComputedStyle,
    new: &layoutng_style::style::computed_style::ComputedStyle,
    context: &layoutng_assembly::internal::layout_object::StyleChangeContext,
) {
    flow.StyleDidChange(difference, old, new, context);
    let shape = unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutSVGShape>() };
    if difference.NeedsFullLayout()
        || unsafe { old.as_ref() }.is_some_and(|old| {
            old.StrokeDashOffset() != new.StrokeDashOffset()
                || old.PathLength() != new.PathLength()
                || unsafe { old.StrokeDashArray().as_ref() }
                    != unsafe { new.StrokeDashArray().as_ref() }
        })
    {
        shape.stroke_path_cache.get_mut().take();
    }
}
