#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{Traceable, Visitor};
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_svg/layout_svg_group.h:11-28
#[repr(C)]
pub struct LayoutSVGGroup {
    flow: LayoutBlockFlow,
    local_transform: foundation::blink_geometry::transforms::affine_transform::AffineTransform,
}

impl LayoutSVGGroup {
    // cpp: layoutng_svg/layout_svg_group.cc:11-11
    pub fn new(element: *mut Element) -> Self {
        let flow = LayoutBlockFlow::new(element.cast());
        flow.SetRuntimeClass(LayoutObjectClass::SvgGroup);
        Self {
            flow,
            local_transform: Default::default(),
        }
    }

    // cpp: layoutng_svg/layout_svg_group.h:15-15
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGGroup"
    }

    // cpp: layoutng_svg/layout_svg_group.h:16-17
    pub fn CreatesNewFormattingContext(&self) -> bool {
        true
    }

    pub fn CanHaveChildren(&self) -> bool {
        true
    }

    // cpp: layoutng_svg/layout_svg_group.h:25-27
    pub fn IsSVG(&self) -> bool {
        true
    }

    pub fn IsSVGContainer(&self) -> bool {
        true
    }

    pub fn IsSVGTransformableContainer(&self) -> bool {
        true
    }

    // cpp: layoutng_svg/layout_svg_group.cc:12-17
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, _style: &ComputedStyle) -> bool {
        if child.is_null() {
            return false;
        }
        // Chromium's SVGContentContainer accepts every renderable SVG child
        // except inline descendants and nested SVG roots. Restricting this to
        // the three common concrete classes rejects valid SVG image/resource
        // objects used by ordinary inline icons.
        let child = unsafe { &*child };
        child.IsSVG() && !child.IsSVGInline() && !child.IsSVGInlineText() && !child.IsSVGRoot()
    }

    // cpp: layoutng_svg/layout_svg_group.cc:19-24
    pub fn AddChild(&mut self, child: *mut LayoutObject, before_child: *mut LayoutObject) {
        assert!(!child.is_null());
        let child_ref = unsafe { &*child };
        // Call the translated derived method directly. Calling through the
        // embedded LayoutBlockFlow selects its HTML child policy, unlike the
        // C++ LayoutSVGModelObject virtual dispatch used by Chromium.
        assert!(
            self.IsChildAllowed(child, child_ref.StyleRef()),
            "class={:?} svg={} inline={} inline_text={} root={}",
            child_ref.RuntimeClass(),
            child_ref.IsSVG(),
            child_ref.IsSVGInline(),
            child_ref.IsSVGInlineText(),
            child_ref.IsSVGRoot()
        );
        unsafe { &mut *(self as *mut Self).cast::<LayoutObject>() }
            .AddChildBase(child, before_child);
    }

    // cpp: layoutng_svg/layout_svg_group.cc:26-34
    pub fn LocalToSVGParentTransform(
        &self,
    ) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform {
        self.local_transform
    }
    pub fn UpdateSVGLayout(&mut self, info: &SVGLayoutInfo) -> SVGLayoutResult {
        self.local_transform = crate::svg_viewport::LocalTransform(self);
        let element = foundation::DynamicTo::<Element>(self.GetNode());
        let use_data = unsafe { element.as_ref() }
            .and_then(|element| element.InputElementData().as_ref())
            .and_then(|element| element.svg_use);
        let mut relative = false;
        if let Some(use_data) = use_data {
            use layoutng_assembly::internal::layout_input::SvgUseData;
            match use_data {
                SvgUseData::Local => {},
                SvgUseData::UnsupportedReference => crate::svg_viewport::Unsupported("SVG use external/normalized/encoded IRI is not translated"),
                SvgUseData::UnsupportedViewport => crate::svg_viewport::Unsupported("SVG use svg/symbol viewport instances are not translated"),
                SvgUseData::UnsupportedImage => crate::svg_viewport::Unsupported("SVG use image instances are not translated"),
                SvgUseData::UnsupportedSwitch => crate::svg_viewport::Unsupported("SVG use switch instances are not translated"),
                SvgUseData::UnbuiltInstance => crate::svg_viewport::Unsupported("SVG use instance lifecycle was not updated before layout"),
            }
            // LayoutSVGTransformableContainer::UpdateLocalTransform: canonical
            // transform followed by use X/Y, resolved in the parent viewport.
            let style = self.StyleRef();
            if !style.X().HasOnlyFixedAndPercent() || !style.Y().HasOnlyFixedAndPercent() {
                crate::svg_viewport::Unsupported("nonlinear SVG use X/Y is not translated");
            }
            let viewport = crate::svg_viewport::ResolveViewport(self);
            let offset = crate::svg_length_functions::VectorForLengthPair(
                style.X(), style.Y(), style.EffectiveZoom(), &viewport,
            );
            relative = style.X().HasPercent() || style.Y().HasPercent();
            self.local_transform.Translate(f64::from(offset.x()), f64::from(offset.y()));
        }
        let mut result = crate::svg_container_layout::LayoutContainer(self, info, 0.0, 0.0);
        result.has_viewport_dependence |= relative;
        result
    }
}

// C++ virtual dispatch is registered by the SVG assembly once installed.
pub fn SvgGroupAddChild(
    flow: &mut LayoutBlockFlow,
    child: *mut LayoutObject,
    before_child: *mut LayoutObject,
) {
    let group = unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutSVGGroup>() };
    if group.RuntimeClass() == LayoutObjectClass::SvgForeignObject {
        let foreign = unsafe {
            &*(flow as *const LayoutBlockFlow)
                .cast::<crate::layout_svg_foreign_object::LayoutSVGForeignObject>()
        };
        assert!(foreign.IsChildAllowed(child, unsafe { &*child }.StyleRef()));
        flow.AddChild(child, before_child);
    } else {
        group.AddChild(child, before_child);
    }
}

impl Deref for LayoutSVGGroup {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

impl DerefMut for LayoutSVGGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutSVGGroup, flow) == 0);

impl Traceable for LayoutSVGGroup {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}

// core/layout/svg/layout_svg_model_object.cc::StyleDidChange and
// layout_svg_viewport_container.cc:133-146. Native SVG transforms participate
// in SVG layout (including descendant text scaling), even when ordinary CSS
// boxes can commit the same style change as paint-only.
pub fn SvgContainerStyleDidChange(
    flow: &mut LayoutBlockFlow,
    difference: layoutng_style::style::style_difference::StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    context: &layoutng_assembly::internal::layout_object::StyleChangeContext,
) {
    flow.StyleDidChange(difference, old_style, new_style, context);
    if !old_style.is_null() && difference.transform_changed() {
        flow.SetNeedsLayout(
            &raw const layoutng_assembly::internal::layout_invalidation_reason::kStyleChange,
        );
    }
    if let Some(old) = unsafe { old_style.as_ref() } {
        if old.X() != new_style.X() || old.Y() != new_style.Y() {
            flow.SetNeedsLayout(
                &raw const layoutng_assembly::internal::layout_invalidation_reason::kStyleChange,
            );
        }
        if old.OverflowX() != new_style.OverflowX() {
            flow.SetNeedsPaintPropertyUpdate();
        }
    }
}
