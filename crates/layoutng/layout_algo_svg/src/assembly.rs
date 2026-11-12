#![allow(non_snake_case)]

use foundation::{DynamicTo, Length, MakeGarbageCollected, To};
use layoutng_assembly::fragment_items_builder::FragmentItemsBuilder;
use layoutng_assembly::internal::fragment_item_with_offset_fwd::FragmentItemWithOffsetList;
use layoutng_assembly::internal::inline_node::InlineNode;
use layoutng_assembly::internal::layout_input::NodeKind;
use layoutng_assembly::internal::layout_node_metadata::{Element, Node, Text};
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::layout_text::RegisterSvgInlineTextFontVirtuals;
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use layoutng_assembly::internal::svg_text_attributes_request::{
    SvgTextAttributesBuildRequest, SvgTextAttributesBuildResult,
};
use layoutng_assembly::layout_assembly::LayoutAssembly;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::block_style::{SquaredLocalSvgTextScale, UpdateSvgBlockStyle};
use crate::layout_svg_foreign_object::LayoutSVGForeignObject;
use crate::layout_svg_group::{LayoutSVGGroup, SvgGroupAddChild};
use crate::layout_svg_hidden_container::LayoutSVGHiddenContainer;
use crate::layout_svg_inline::LayoutSVGInline;
use crate::layout_svg_inline_text::{
    ComputeScaledSvgFont, LayoutSVGInlineText, SvgInlineTextScaledFont, SvgInlineTextScalingFactor,
    SvgInlineTextUpdateScaledFont,
};
use crate::layout_svg_resource_clipper::LayoutSVGResourceClipper;
use crate::layout_svg_resource_masker::LayoutSVGResourceMasker;
use crate::layout_svg_root::LayoutSVGRoot;
use crate::layout_svg_root::{
    QuerySvgRootSizing, SvgRootAddChild, SvgRootChildren, SvgRootLayout, SvgRootNaturalDimensions,
    SvgRootRemoveChild,
};
use crate::layout_svg_shape::LayoutSVGShape;
use crate::layout_svg_text::{
    LayoutSVGText, SvgTextAddChild, SvgTextInsertedIntoTree, SvgTextRemoveChild,
    SvgTextSetNeedsMetricsUpdate, SvgTextWillBeRemovedFromTree,
};
use crate::layout_svg_text_path::LayoutSVGTextPath;
use crate::layout_svg_tspan::LayoutSVGTSpan;
use crate::layout_svg_viewport_container::LayoutSVGViewportContainer;
use crate::svg_length_functions::ValueForLengthWithStyle;
use crate::svg_text_layout_algorithm::SvgTextLayoutAlgorithm;
use crate::svg_text_layout_attributes_builder::SvgTextLayoutAttributesBuilder;
use crate::svg_text_scale::CalculateScreenFontSizeScalingFactor;

// cpp: layoutng_svg/assembly.cc:26-57
fn CreateSvgObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    let text = DynamicTo::<Text>(node as *mut Node);
    if !text.is_null() {
        return MakeGarbageCollected(LayoutSVGInlineText::new(
            node,
            unsafe { &*text }.data().clone(),
        ))
        .cast();
    }
    let element = To::<Element>(node as *mut Node);
    match node.InputKind() {
        NodeKind::kSvgViewportContainer => {
            MakeGarbageCollected(LayoutSVGViewportContainer::new(element)).cast()
        }
        NodeKind::kSvgRoot => MakeGarbageCollected(LayoutSVGRoot::new(element)).cast(),
        NodeKind::kSvgGroup => MakeGarbageCollected(LayoutSVGGroup::new(element)).cast(),
        NodeKind::kSvgHiddenContainer => {
            MakeGarbageCollected(LayoutSVGHiddenContainer::new(element)).cast()
        }
        NodeKind::kSvgResourceGradient | NodeKind::kSvgResourcePattern => MakeGarbageCollected(
            crate::layout_svg_resource_paint_server::LayoutSVGResourcePaintServer::new(
                element,
                node.InputKind() == NodeKind::kSvgResourcePattern,
            ),
        )
        .cast(),
        NodeKind::kSvgResourceFilter | NodeKind::kSvgResourceMarker => MakeGarbageCollected(
            crate::layout_svg_resource_auxiliary::LayoutSVGResourceAuxiliary::new(
                element,
                node.InputKind() == NodeKind::kSvgResourceFilter,
            ),
        )
        .cast(),
        NodeKind::kSvgResourceMasker => {
            MakeGarbageCollected(LayoutSVGResourceMasker::new(element)).cast()
        }
        NodeKind::kSvgResourceClipper => {
            MakeGarbageCollected(LayoutSVGResourceClipper::new(element)).cast()
        }
        NodeKind::kSvgForeignObject => {
            MakeGarbageCollected(LayoutSVGForeignObject::new(element)).cast()
        }
        NodeKind::kSvgText => MakeGarbageCollected(LayoutSVGText::new(element)).cast(),
        NodeKind::kSvgInline => MakeGarbageCollected(LayoutSVGInline::new(element)).cast(),
        NodeKind::kSvgTSpan => MakeGarbageCollected(LayoutSVGTSpan::new(element)).cast(),
        NodeKind::kSvgTextPath => {
            let mut ancestor = node.parentNode().cast::<Node>();
            while !ancestor.is_null() {
                if unsafe { &*ancestor }.InputKind() == NodeKind::kSvgText {
                    return MakeGarbageCollected(LayoutSVGTextPath::new(element)).cast();
                }
                ancestor = unsafe { &*ancestor }.parentNode().cast::<Node>();
            }
            panic!("SVG textPath must be a descendant of SVG text");
        }
        NodeKind::kSvgShape => MakeGarbageCollected(LayoutSVGShape::new(element)).cast(),
        other => panic!("unsupported SVG node kind: {other:?}"),
    }
}

// cpp: layoutng_svg/assembly.cc:58-62
fn ResolveSvgLength(length: &Length, style: &ComputedStyle, dimension: f32) -> f32 {
    ValueForLengthWithStyle(length, style, dimension)
}

// cpp: layoutng_svg/assembly.cc:72-87
fn BuildSvgTextAttributes(
    request: &SvgTextAttributesBuildRequest<'_>,
    result: &mut SvgTextAttributesBuildResult,
) {
    let _trace = browser_tracing::span("layout", "layout_algo_svg.BuildTextAttributes");
    let mut builder = SvgTextLayoutAttributesBuilder::new(request.node);
    builder.Build(request.text, request.items);
    if request.include_ifc_offsets {
        result
            .ifc_text_content_offsets
            .reserve(builder.ResolvedCharacterCount());
        for index in 0..builder.ResolvedCharacterCount() {
            result
                .ifc_text_content_offsets
                .push(builder.IfcTextContentOffsetAt(index));
        }
    }
    result.data = builder.CreateSvgInlineNodeData();
}

// cpp: layoutng_svg/assembly.cc:99-104
fn LayoutSvgText(
    node: *const InlineNode,
    builder: *const FragmentItemsBuilder,
    items: *mut FragmentItemWithOffsetList,
) -> foundation::PhysicalSize {
    let _trace = browser_tracing::span("layout", "layout_algo_svg.TextLayout");
    let node = unsafe { &*node };
    let builder = unsafe { &*builder };
    let items = unsafe { &mut *items };
    SvgTextLayoutAlgorithm::new(node.clone(), builder.GetWritingMode()).Layout(builder, items)
}

// The C++ vtable owns this selection. Rust's base-first object representation
// uses the runtime class to select the translated derived method.
fn UpdateSvgObjectLayout(object: &mut LayoutObject, info: &SVGLayoutInfo) -> SVGLayoutResult {
    let _trace = browser_tracing::span("layout", "layout_algo_svg.ObjectLayout");
    match object.RuntimeClass() {
        LayoutObjectClass::SvgResourceAuxiliary => unsafe {
            &mut *(object as *mut LayoutObject)
                .cast::<crate::layout_svg_resource_auxiliary::LayoutSVGResourceAuxiliary>()
        }
        .UpdateSVGLayout(info),

        LayoutObjectClass::SvgHiddenContainer
        | LayoutObjectClass::SvgResourceClipper
        | LayoutObjectClass::SvgResourceMasker
        | LayoutObjectClass::SvgResourcePaintServer => {
            unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGHiddenContainer>() }
                .UpdateSVGLayout(info)
        }
        LayoutObjectClass::SvgViewportContainer => {
            unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGViewportContainer>() }
                .UpdateSVGLayout(info)
        }
        LayoutObjectClass::SvgGroup => {
            unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGGroup>() }
                .UpdateSVGLayout(info)
        }
        LayoutObjectClass::SvgForeignObject => {
            unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGForeignObject>() }
                .UpdateSVGLayout(info)
        }
        LayoutObjectClass::SvgShape => {
            unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGShape>() }
                .UpdateSVGLayout(info)
        }
        LayoutObjectClass::SvgText => {
            unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGText>() }
                .UpdateSVGLayout(info)
        }
        other => panic!("SVG UpdateSVGLayout is not installed for {other:?}"),
    }
}

fn SvgIsChildAllowed(
    object: &LayoutObject,
    child: *mut LayoutObject,
    style: &ComputedStyle,
) -> bool {
    match object.RuntimeClass() {
        LayoutObjectClass::SvgResourceAuxiliary => unsafe {
            &*(object as *const LayoutObject)
                .cast::<crate::layout_svg_resource_auxiliary::LayoutSVGResourceAuxiliary>()
        }
        .IsChildAllowed(child, style),

        LayoutObjectClass::SvgResourcePaintServer => unsafe {
            &*(object as *const LayoutObject)
                .cast::<crate::layout_svg_resource_paint_server::LayoutSVGResourcePaintServer>()
        }
        .IsChildAllowed(child, style),

        LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgViewportContainer
        | LayoutObjectClass::SvgHiddenContainer
        | LayoutObjectClass::SvgResourceClipper
        | LayoutObjectClass::SvgResourceMasker => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGGroup>() }
                .IsChildAllowed(child, style)
        }
        LayoutObjectClass::SvgForeignObject => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGForeignObject>() }
                .IsChildAllowed(child, style)
        }
        LayoutObjectClass::SvgRoot => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGRoot>() }
                .IsChildAllowed(child, style)
        }
        LayoutObjectClass::SvgInline => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGInline>() }
                .IsChildAllowed(child, style)
        }
        LayoutObjectClass::SvgTSpan => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGTSpan>() }
                .IsChildAllowed(child, style)
        }
        LayoutObjectClass::SvgTextPath => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGTextPath>() }
                .IsChildAllowed(child, style)
        }
        LayoutObjectClass::SvgText => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGText>() }
                .IsChildAllowed(child, style)
        }
        other => panic!("SVG IsChildAllowed is not installed for {other:?}"),
    }
}

// cpp: layoutng_svg/assembly.h:2-2
// cpp: layoutng_svg/assembly.cc:107-120
pub fn InstallSvgModule(assembly: &mut LayoutAssembly) {
    assembly.objects.svg_context_reference_box = Some(crate::svg_resources::ObjectBoundingBox);
    assembly.objects.svg_marker_paints = Some(|object| {
        if object.RuntimeClass() != LayoutObjectClass::SvgShape {
            return Vec::new();
        }
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }.MarkerPaints()
    });
    assembly.algorithms.svg_support.performance_owner = Some("layout_algo_svg");
    let svg = &mut assembly.algorithms.svg_support;
    svg.hit_test_reference_box = Some(crate::svg_resources::ReferenceBoxForEffects);
    svg.hit_test_resolve_clip = Some(|tree, object, reference| {
        crate::svg_resources::ResolveClipPathTarget(tree, object, reference)
            .map_or(std::ptr::null(), |clip| {
                clip as *const _ as *const LayoutObject
            })
    });
    svg.hit_test_clip_units = Some(|object| {
        assert!(object.RuntimeClass() == LayoutObjectClass::SvgResourceClipper);
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGResourceClipper>() }
            .UsesObjectBoundingBoxUnits()
    });
    svg.hit_test_clip_transform = Some(|object| {
        assert!(object.RuntimeClass() == LayoutObjectClass::SvgResourceClipper);
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGResourceClipper>() }
            .ElementTransformIncludingMotion()
    });
    svg.hit_test_path = Some(|object| {
        assert!(object.IsSVGShape());
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }.GetPath()
    });
    svg.hit_test_shape_empty = Some(|object| {
        assert!(object.IsSVGShape());
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }.IsShapeEmpty()
    });
    svg.hit_test_fill_geometry = Some(|object| {
        assert!(object.IsSVGShape());
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }.FillGeometry()
    });
    svg.hit_test_stroke_geometry = Some(|object| {
        assert!(object.IsSVGShape());
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }.StrokeGeometry()
    });
    svg.hit_test_stroke_bounds = Some(|object| {
        assert!(object.IsSVGShape());
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }
            .HitTestStrokeBoundingBox()
    });
    svg.hit_test_stroke_width = Some(|object| {
        assert!(object.IsSVGShape());
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }.StrokeWidth()
    });
    svg.hit_test_object_bounds_valid = Some(crate::svg_resources::IsObjectBoundingBoxValid);
    svg.hit_test_viewport = Some(|object| {
        let viewport =
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGViewportContainer>() };
        assert!(object.IsSVGViewportContainer());
        (viewport.Viewport(), viewport.LocalSVGTransform())
    });
    svg.hit_test_root_transform = Some(|object| {
        assert!(object.IsSVGRoot());
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGRoot>() }.ContentTransform()
    });
    svg.hit_test_filtered_bounds = Some(|tree, object| {
        crate::svg_resources::ApplyFiltersToRect(
            tree,
            object,
            crate::svg_resources::DecoratedBoundingBox(object),
        )
    });
    svg.hit_test_has_paint_server = Some(crate::svg_resources::HasPaintServer);
    layoutng_assembly::RegisterSvgRootChildren(SvgRootChildren);
    RegisterSvgInlineTextFontVirtuals(SvgInlineTextScaledFont, SvgInlineTextScalingFactor);
    assembly.objects.svg = Some(CreateSvgObject);
    assembly.objects.svg_block_text_scale = Some(SquaredLocalSvgTextScale);
    assembly.objects.update_svg_block_style = Some(UpdateSvgBlockStyle);
    assembly.objects.svg_group_add_child = Some(SvgGroupAddChild);
    assembly.objects.svg_shape_style_did_change =
        Some(crate::layout_svg_shape::SvgShapeStyleDidChange);
    assembly.objects.svg_container_style_did_change =
        Some(crate::layout_svg_group::SvgContainerStyleDidChange);
    assembly.objects.svg_is_child_allowed = Some(SvgIsChildAllowed);
    assembly.objects.svg_text_add_child = Some(SvgTextAddChild);
    assembly.objects.svg_text_remove_child = Some(SvgTextRemoveChild);
    assembly.objects.svg_text_inserted_into_tree = Some(SvgTextInsertedIntoTree);
    assembly.objects.svg_text_will_be_removed_from_tree = Some(SvgTextWillBeRemovedFromTree);
    assembly.objects.svg_text_needs_metrics_update = Some(SvgTextSetNeedsMetricsUpdate);
    assembly.objects.svg_inline_text_update_scaled_font = Some(SvgInlineTextUpdateScaledFont);
    assembly.objects.svg_inline_text_scaled_font = Some(SvgInlineTextScaledFont);
    assembly.objects.svg_inline_text_scaling_factor = Some(SvgInlineTextScalingFactor);
    assembly.objects.svg_root_add_child = Some(SvgRootAddChild);
    assembly.objects.svg_root_remove_child = Some(SvgRootRemoveChild);
    assembly.objects.svg_root_children = Some(SvgRootChildren);
    assembly.algorithms.svg_support.object_bounding_box =
        Some(crate::svg_resources::ObjectBoundingBox);
    assembly
        .algorithms
        .svg_support
        .resize_observer_bbox_diagnostics =
        Some(crate::svg_resources::ResizeObserverBBoxDiagnostics);
    assembly.algorithms.svg_support.stroke_bounding_box =
        Some(crate::svg_resources::StrokeBoundingBox);
    assembly.algorithms.svg_support.retained_shape = Some(|object| {
        if !object.IsSVGShape() {
            return None;
        }
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGShape>() }.RetainedShape()
    });
    assembly
        .algorithms
        .svg_support
        .foreign_object_physical_location = Some(|object| {
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGForeignObject>() }
            .PhysicalLocation()
    });
    assembly.algorithms.svg_support.decorated_bounding_box =
        Some(crate::svg_resources::DecoratedBoundingBox);
    assembly.algorithms.svg_support.root_add_visual_overflow = Some(|object| {
        unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGRoot>() }
            .AddSVGContentsVisualOverflow()
    });
    assembly
        .algorithms
        .svg_support
        .root_visual_overflow_including_filters = Some(|object| {
        unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGRoot>() }
            .VisualOverflowRectIncludingFilters()
    });
    assembly.algorithms.svg_support.update_object = Some(UpdateSvgObjectLayout);
    assembly.algorithms.svg_support.local_to_parent_transform = Some(SvgLocalToParentTransform);
    assembly.algorithms.svg_support.paint_geometry = Some(SvgPaintGeometry);
    assembly.algorithms.svg_support.text_object_bounding_box =
        Some(crate::svg_text_visual_rect::ObjectBoundingBox);
    assembly.algorithms.svg_support.text_visual_rect =
        Some(crate::svg_text_visual_rect::VisualRectInLocalSVGCoordinates);
    assembly.algorithms.svg_support.viewport_size = Some(crate::svg_viewport::ResolveViewport);
    assembly.algorithms.svg_support.root_natural_dimensions = Some(SvgRootNaturalDimensions);
    assembly.algorithms.svg_support.root_sizing_info = Some(QuerySvgRootSizing);
    assembly.algorithms.svg_support.set_root_container_size =
        Some(crate::layout_svg_root::SvgRootSetContainerSize);
    assembly.algorithms.svg_support.set_root_container_scale =
        Some(crate::layout_svg_root::SvgRootSetContainerScale);
    assembly.algorithms.svg_support.layout_root = Some(SvgRootLayout);
    assembly.algorithms.svg_support.compute_scaled_font = Some(ComputeScaledSvgFont);
    assembly.algorithms.svg_support.screen_font_scale = Some(CalculateScreenFontSizeScalingFactor);
    assembly.algorithms.svg_support.resolve_length = Some(ResolveSvgLength);
    assembly.algorithms.svg_support.build_text_attributes = Some(BuildSvgTextAttributes);
    assembly.algorithms.svg_support.layout_text = Some(LayoutSvgText);
}

fn SvgLocalToParentTransform(
    object: &LayoutObject,
) -> foundation::blink_geometry::transforms::affine_transform::AffineTransform {
    match object.RuntimeClass() {
        LayoutObjectClass::SvgResourceAuxiliary => unsafe {
            &*(object as *const LayoutObject)
                .cast::<crate::layout_svg_resource_auxiliary::LayoutSVGResourceAuxiliary>()
        }
        .LocalToSVGParentTransform(),

        LayoutObjectClass::SvgViewportContainer => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGViewportContainer>() }
                .LocalToSVGParentTransform()
        }
        LayoutObjectClass::SvgGroup => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGGroup>() }
                .LocalToSVGParentTransform()
        }
        LayoutObjectClass::SvgForeignObject => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGForeignObject>() }
                .LocalToSVGParentTransform()
        }
        LayoutObjectClass::SvgRoot => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGRoot>() }.ContentTransform()
        }
        LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText
        | LayoutObjectClass::SvgHiddenContainer
        | LayoutObjectClass::SvgResourceClipper
        | LayoutObjectClass::SvgResourceMasker
        | LayoutObjectClass::SvgResourcePaintServer => crate::svg_viewport::LocalTransform(object),
        _ => foundation::blink_geometry::transforms::affine_transform::AffineTransform::default(),
    }
}
fn SvgPaintGeometry(
    object: &LayoutObject,
) -> (
    Option<layoutng_assembly::internal::layout_input::TransformMatrix>,
    Option<layoutng_assembly::fragment_tree::SvgViewportGeometry>,
    Option<foundation::gfx::RectF>,
) {
    match object.RuntimeClass() {
        LayoutObjectClass::SvgShape => (
            None,
            None,
            Some(crate::svg_resources::DecoratedBoundingBox(object)),
        ),
        LayoutObjectClass::SvgResourceAuxiliary => (
            Some(crate::svg_viewport::Matrix(SvgLocalToParentTransform(
                object,
            ))),
            None,
            Some(crate::svg_container_layout::VisualRect(object)),
        ),
        LayoutObjectClass::SvgViewportContainer => {
            let viewport =
                unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGViewportContainer>() };
            (
                Some(crate::svg_viewport::Matrix(
                    viewport.LocalToSVGParentTransform(),
                )),
                Some(viewport.PaintGeometry()),
                Some(crate::svg_container_layout::VisualRect(object)),
            )
        }
        LayoutObjectClass::SvgGroup => (
            Some(crate::svg_viewport::Matrix(SvgLocalToParentTransform(
                object,
            ))),
            None,
            Some(crate::svg_container_layout::VisualRect(object)),
        ),
        LayoutObjectClass::SvgRoot => (
            None,
            Some(
                unsafe { &*(object as *const LayoutObject).cast::<LayoutSVGRoot>() }
                    .PaintGeometry(),
            ),
            None,
        ),
        _ => (None, None, None),
    }
}
