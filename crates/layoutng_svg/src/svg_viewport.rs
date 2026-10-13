// Copyright The Chromium Authors. See third_party/README.md and the pinned
// Chromium sources recorded in the nested-SVG evidence manifest.
#![allow(non_snake_case)]

use crate::layout_svg_root::LayoutSVGRoot;
use crate::layout_svg_viewport_container::LayoutSVGViewportContainer;
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::{gfx, DynamicTo, EOverflow, ETransformBox, UnsupportedLayout};
use layoutng_assembly::fragment_tree::{FragmentPaintRect, SvgViewportGeometry};
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_input::{Offset, Size, SvgViewBoxData, TransformMatrix};
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_style::style::style_transform::{
    ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
    ApplyTransformOrigin, TransformBoxContext,
};

pub(crate) fn Unsupported(reason: &'static str) -> ! {
    std::panic::panic_any(UnsupportedLayout::new(reason))
}

pub(crate) fn ViewBox(object: &LayoutObject) -> Option<SvgViewBoxData> {
    let element = DynamicTo::<Element>(object.GetNode());
    unsafe { element.as_ref() }
        .and_then(|e| e.InputElementData().as_ref())
        .and_then(|e| e.svg_view_box)
}

// core/svg/svg_length_functions.cc:44-70. A nested viewport's own lengths
// resolve against its parent viewport, never against its own viewBox.
pub(crate) fn ResolveViewport(object: &LayoutObject) -> gfx::SizeF {
    let mut ancestor = object.Parent();
    while let Some(parent) = unsafe { ancestor.as_ref() } {
        if parent.IsSVGRoot() || parent.IsSVGViewportContainer() {
            if let Some(view_box) = ViewBox(parent).filter(|v| v.width > 0.0 && v.height > 0.0) {
                return gfx::SizeF::new(view_box.width as f32, view_box.height as f32);
            }
            if parent.IsSVGRoot() {
                let root = unsafe { &*ancestor.cast::<LayoutSVGRoot>() };
                let size = root.ViewportSize();
                return gfx::SizeF::new(
                    size.width() / root.StyleRef().EffectiveZoom(),
                    size.height() / root.StyleRef().EffectiveZoom(),
                );
            }
            return unsafe { &*ancestor.cast::<LayoutSVGViewportContainer>() }
                .Viewport()
                .size();
        }
        ancestor = parent.Parent();
    }
    gfx::SizeF::default()
}

// core/svg/svg_preserve_aspect_ratio.cc::ComputeTransform. This is the one
// layout-owned viewBox calculation used by both outer and nested viewports.
pub(crate) fn ViewBoxTransform(
    view_box: Option<SvgViewBoxData>,
    size: gfx::SizeF,
) -> AffineTransform {
    let Some(view_box) = view_box else {
        return AffineTransform::default();
    };
    if view_box.width <= 0.0
        || view_box.height <= 0.0
        || size.width() <= 0.0
        || size.height() <= 0.0
    {
        return AffineTransform::default();
    }
    let mut sx = size.width() as f64 / view_box.width;
    let mut sy = size.height() as f64 / view_box.height;
    let mut x = 0.0;
    let mut y = 0.0;
    if !view_box.preserve_none {
        let scale = if view_box.slice {
            sx.max(sy)
        } else {
            sx.min(sy)
        };
        sx = scale;
        sy = scale;
        let align = |spare: f64, alignment: i8| {
            if alignment < 0 {
                0.0
            } else if alignment > 0 {
                spare
            } else {
                spare * 0.5
            }
        };
        x = align(
            size.width() as f64 - view_box.width * scale,
            view_box.align_x,
        );
        y = align(
            size.height() as f64 - view_box.height * scale,
            view_box.align_y,
        );
    }
    let mut transform =
        AffineTransform::new(sx, 0.0, 0.0, sy, x - view_box.x * sx, y - view_box.y * sy);
    if let Some(view) = view_box.view_transform {
        transform.PreConcat(AffineTransform::new(
            view.values[0],
            view.values[1],
            view.values[4],
            view.values[5],
            view.values[12],
            view.values[13],
        ));
    }
    transform
}

// core/layout/svg/transform_helper.cc:102-167. Reuse canonical ComputedStyle's
// transform implementation; no CSS parsing or projected-style reconstruction.
pub(crate) fn LocalTransform(object: &LayoutObject) -> AffineTransform {
    let style = object.StyleRef();
    if !style.HasTransform() {
        return AffineTransform::default();
    }
    if style.HasOffset() {
        Unsupported("SVG viewport motion-path transform is not translated");
    }
    if style.UsedTransformBox(TransformBoxContext::kSvg) != ETransformBox::kViewBox {
        Unsupported("SVG container fill-box/stroke-box transform bounds are not translated");
    }
    let viewport = ResolveViewport(object);
    let zoom = style.EffectiveZoom();
    let reference = gfx::RectF::new(
        gfx::PointF::default(),
        gfx::SizeF::new(viewport.width() * zoom, viewport.height() * zoom),
    );
    let mut transform = gfx::Transform::default();
    style.ApplyTransformRectF::<LayoutBox>(
        &mut transform,
        None,
        &reference,
        ApplyTransformOperations::kIncludeTransformOperations,
        ApplyTransformOrigin::kIncludeTransformOrigin,
        ApplyMotionPath::kIncludeMotionPath,
        ApplyIndependentTransformProperties::kIncludeIndependentTransformProperties,
    );
    if zoom != 1.0 {
        transform.Zoom(1.0 / zoom);
    }
    AffineTransform::FromTransform(&transform)
}

pub(crate) fn Matrix(transform: AffineTransform) -> TransformMatrix {
    let mut result = TransformMatrix::default();
    result.values[0] = transform.A();
    result.values[1] = transform.B();
    result.values[4] = transform.C();
    result.values[5] = transform.D();
    result.values[12] = transform.E();
    result.values[13] = transform.F();
    result
}
pub(crate) fn Rect(rect: gfx::RectF) -> FragmentPaintRect {
    FragmentPaintRect {
        offset: Offset {
            x: rect.x() as f64,
            y: rect.y() as f64,
        },
        size: Size {
            width: rect.width() as f64,
            height: rect.height() as f64,
        },
    }
}
pub(crate) fn Geometry(
    object: &LayoutObject,
    viewport: gfx::RectF,
    content: AffineTransform,
    nested: bool,
) -> SvgViewportGeometry {
    let disabled = viewport.width() <= 0.0
        || viewport.height() <= 0.0
        || ViewBox(object).is_some_and(|v| v.width <= 0.0 || v.height <= 0.0);
    // core/layout/svg/svg_layout_support.cc:224-235 and
    // core/paint/paint_property_tree_builder.cc:3308-3318. SVG uses overflow-x
    // and does not establish a CSS scroll container.
    let clips = nested
        && matches!(
            object.StyleRef().OverflowX(),
            EOverflow::kHidden | EOverflow::kClip | EOverflow::kScroll
        );
    let clip = if disabled {
        Some(FragmentPaintRect::default())
    } else if clips {
        Some(Rect(content.Inverse().MapRect(viewport)))
    } else {
        None
    };
    SvgViewportGeometry {
        viewport: Rect(viewport),
        content_transform: Matrix(content),
        clip,
        disabled,
    }
}
