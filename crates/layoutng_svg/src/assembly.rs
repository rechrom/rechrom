#![allow(non_snake_case)]

use foundation::{DynamicTo, Length, MakeGarbageCollected, To};
use layoutng_assembly::fragment_items_builder::FragmentItemsBuilder;
use layoutng_assembly::internal::fragment_item_with_offset_fwd::FragmentItemWithOffsetList;
use layoutng_assembly::internal::inline_node::InlineNode;
use layoutng_assembly::internal::layout_input::NodeKind;
use layoutng_assembly::internal::layout_node_metadata::{Element, Node, Text};
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use layoutng_assembly::internal::svg_text_attributes_request::{
    SvgTextAttributesBuildRequest, SvgTextAttributesBuildResult,
};
use layoutng_assembly::layout_assembly::LayoutAssembly;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::block_style::{SquaredLocalSvgTextScale, UpdateSvgBlockStyle};
use crate::layout_svg_foreign_object::LayoutSVGForeignObject;
use crate::layout_svg_group::{LayoutSVGGroup, SvgGroupAddChild};
use crate::layout_svg_inline::LayoutSVGInline;
use crate::layout_svg_inline_text::{
    ComputeScaledSvgFont, LayoutSVGInlineText, SvgInlineTextScaledFont, SvgInlineTextScalingFactor,
    SvgInlineTextUpdateScaledFont,
};
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
        NodeKind::kSvgRoot => MakeGarbageCollected(LayoutSVGRoot::new(element)).cast(),
        NodeKind::kSvgGroup => MakeGarbageCollected(LayoutSVGGroup::new(element)).cast(),
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
    let node = unsafe { &*node };
    let builder = unsafe { &*builder };
    let items = unsafe { &mut *items };
    SvgTextLayoutAlgorithm::new(node.clone(), builder.GetWritingMode()).Layout(builder, items)
}

// The C++ vtable owns this selection. Rust's base-first object representation
// uses the runtime class to select the translated derived method.
fn UpdateSvgObjectLayout(object: &mut LayoutObject, info: &SVGLayoutInfo) -> SVGLayoutResult {
    match object.RuntimeClass() {
        LayoutObjectClass::SvgGroup | LayoutObjectClass::SvgForeignObject => {
            unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGGroup>() }
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
        LayoutObjectClass::SvgGroup => {
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
    assembly.objects.svg = Some(CreateSvgObject);
    assembly.objects.svg_block_text_scale = Some(SquaredLocalSvgTextScale);
    assembly.objects.update_svg_block_style = Some(UpdateSvgBlockStyle);
    assembly.objects.svg_group_add_child = Some(SvgGroupAddChild);
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
    assembly.algorithms.svg_support.update_object = Some(UpdateSvgObjectLayout);
    assembly.algorithms.svg_support.root_natural_dimensions = Some(SvgRootNaturalDimensions);
    assembly.algorithms.svg_support.root_sizing_info = Some(QuerySvgRootSizing);
    assembly.algorithms.svg_support.layout_root = Some(SvgRootLayout);
    assembly.algorithms.svg_support.compute_scaled_font = Some(ComputeScaledSvgFont);
    assembly.algorithms.svg_support.screen_font_scale = Some(CalculateScreenFontSizeScalingFactor);
    assembly.algorithms.svg_support.resolve_length = Some(ResolveSvgLength);
    assembly.algorithms.svg_support.build_text_attributes = Some(BuildSvgTextAttributes);
    assembly.algorithms.svg_support.layout_text = Some(LayoutSvgText);
}
