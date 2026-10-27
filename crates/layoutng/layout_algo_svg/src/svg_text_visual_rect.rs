// Copyright The Chromium Authors. See third_party/NOTICE.chromium-svg-text-bounds.
#![allow(non_snake_case)]

use foundation::{gfx, To};
use layoutng_assembly::internal::layout_object::LayoutObject;

use crate::layout_svg_inline::LayoutSVGInline;
use crate::layout_svg_text::LayoutSVGText;

// core/layout/svg/layout_svg_inline.cc:115-120, layout_svg_text.cc:328-337.
// The query is a pure read of the completed layout snapshot. The existing
// SvgLayoutSupport assembly slot carries this derived-class operation.
pub(crate) fn VisualRectInLocalSVGCoordinates(object: &LayoutObject) -> gfx::RectF {
    let mut bounds = if object.IsSVGText() {
        unsafe { &*To::<LayoutSVGText>(object) }.ObjectBoundingBox()
    } else {
        assert!(object.IsSVGInline());
        unsafe { &*To::<LayoutSVGInline>(object) }.ObjectBoundingBox()
    };
    if bounds.IsEmpty() {
        return bounds;
    }
    // core/layout/svg/svg_layout_support.cc:254-278. Stroke and shadow
    // expansion belongs to SVG Layout, before the immutable Paint projection.
    let style = object.StyleRef();
    if style.HasStroke() {
        let viewport = crate::svg_viewport::ResolveViewport(object);
        let dimension =
            ((viewport.width() * viewport.width() + viewport.height() * viewport.height()) / 2.0)
                .sqrt();
        let stroke = crate::svg_length_functions::ValueForLength(
            style.StrokeWidth().length(),
            style.EffectiveZoom(),
            dimension,
        );
        bounds.Inset(-stroke);
    }
    if let Some(shadow) = unsafe { style.TextShadow().as_ref() } {
        shadow.AdjustRectForShadow(&mut bounds);
    }
    bounds
}
