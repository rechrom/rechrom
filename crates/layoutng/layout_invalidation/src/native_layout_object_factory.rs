//! Native LayoutObject factory selection owned by tree invalidation.
//!
//! The factory dispatch is part of the update transaction: it materializes
//! invalidated nodes and keeps creation cost under an invalidation symbol.

#![allow(non_snake_case)]

use super::*;

use layoutng_style::style::computed_style::ComputedStyle as LayoutComputedStyle;

unsafe extern "Rust" {
    fn LayoutObjectCreateObjectProvider(
        element: *mut Element,
        style: &LayoutComputedStyle,
    ) -> *mut LayoutObject;
    fn LayoutObjectCreateBlockFlowOrListItemProvider(
        element: *mut Element,
        style: &LayoutComputedStyle,
    ) -> *mut LayoutBlockFlow;
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:827-827
    #[inline(never)]
    pub fn CreateObject(element: *mut Element, style: &LayoutComputedStyle) -> *mut LayoutObject {
        unsafe { LayoutObjectCreateObjectProvider(element, style) }
    }

    // cpp: layoutng/internal/layout_object.h:828-829
    #[inline(never)]
    pub fn CreateBlockFlowOrListItem(
        element: *mut Element,
        style: &LayoutComputedStyle,
    ) -> *mut LayoutBlockFlow {
        unsafe { LayoutObjectCreateBlockFlowOrListItemProvider(element, style) }
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:567-693
#[inline(never)]
pub(crate) fn CreateLayoutObject(
    node: &mut Node,
    style: &NativeComputedStyle,
    factories: &LayoutObjectFactorySet,
) -> *mut LayoutObject {
    let require = |factory: Option<LayoutObjectFactory>, capability: &str, node: &mut Node| {
        let Some(factory) = factory else {
            panic!("{capability} layout module is not installed");
        };
        let object = factory(node, style);
        if object.is_null() {
            panic!("{capability} layout factory rejected its selected node");
        }
        object
    };

    if node.IsTextNode() {
        let mut ancestor = node.parentNode();
        while !ancestor.is_null() {
            if unsafe { &*ancestor }.node.InputKind() == NodeKind::kSvgText {
                return require(factories.svg, "svg", node);
            }
            ancestor = unsafe { &*ancestor }.node.parentNode();
        }
        return require(factories.inline_text, "inline", node);
    }

    assert!(
        node.IsElementNode(),
        "CreateLayoutObject requires an element"
    );
    let element = node as *mut Node as *mut Element;
    if node.parentNode().is_null() {
        if node.InputKind() != NodeKind::kBox || style.Display() != EDisplay::kBlock {
            panic!("Layout root must be a block box suitable for LayoutView");
        }
        let container = unsafe { &mut (*element).container };
        return MakeGarbageCollected(LayoutView::new(container)) as *mut LayoutObject;
    }
    if style.StyleType() == PseudoId::kPseudoIdFirstLetter && !style.InitialLetter().IsNormal() {
        let container = unsafe { &mut (*element).container };
        return MakeGarbageCollected(LayoutBlockFlow::new(container)) as *mut LayoutObject;
    }
    let element_data = unsafe { &*element }.InputElementData().as_ref();
    if element_data.is_some_and(|data| data.text_control_inner_editor) {
        return require(factories.forms, "forms", node);
    }
    match node.InputKind() {
        NodeKind::kLineBreak => return require(factories.inline_line_break, "inline", node),
        NodeKind::kWordBreak => return require(factories.inline_word_break, "inline", node),
        NodeKind::kListMarker => return require(factories.list, "list", node),
        NodeKind::kReplaced | NodeKind::kFrame => {
            return require(factories.replaced, "replaced", node);
        }
        NodeKind::kFormControl => {
            let Some(factory) = factories.forms else {
                panic!("forms layout module is not installed");
            };
            let control = factory(node, style);
            if !control.is_null() {
                return control;
            }
            if element_data
                .and_then(|data| data.select_uses_menu_list)
                .unwrap_or(false)
            {
                return require(factories.flex, "flex", node);
            }
        }
        NodeKind::kFieldset => return require(factories.forms, "forms", node),
        NodeKind::kFrameSet => return require(factories.frameset, "frameset", node),
        NodeKind::kSvgRoot
        | NodeKind::kSvgViewportContainer
        | NodeKind::kSvgGroup
        | NodeKind::kSvgHiddenContainer
        | NodeKind::kSvgResourceClipper
        | NodeKind::kSvgResourceMasker
        | NodeKind::kSvgForeignObject
        | NodeKind::kSvgText
        | NodeKind::kSvgInline
        | NodeKind::kSvgTSpan
        | NodeKind::kSvgTextPath
        | NodeKind::kSvgShape => return require(factories.svg, "svg", node),
        _ => {}
    }

    match style.Display() {
        EDisplay::kInline | EDisplay::kRuby | EDisplay::kRubyText => {
            require(factories.inline_box, "inline", node)
        }
        EDisplay::kBlock | EDisplay::kFlowRoot | EDisplay::kInlineBlock => {
            let container = unsafe { &mut (*element).container };
            MakeGarbageCollected(LayoutBlockFlow::new(container)) as *mut LayoutObject
        }
        EDisplay::kListItem | EDisplay::kFlowRootListItem | EDisplay::kInlineFlowRootListItem => {
            require(factories.list, "list", node)
        }
        EDisplay::kTable
        | EDisplay::kInlineTable
        | EDisplay::kTableRowGroup
        | EDisplay::kTableHeaderGroup
        | EDisplay::kTableFooterGroup
        | EDisplay::kTableRow
        | EDisplay::kTableColumnGroup
        | EDisplay::kTableColumn
        | EDisplay::kTableCell
        | EDisplay::kTableCaption => require(factories.table, "table", node),
        EDisplay::kFlex | EDisplay::kInlineFlex => require(factories.flex, "flex", node),
        EDisplay::kGrid | EDisplay::kInlineGrid => require(factories.grid, "grid", node),
        EDisplay::kGridLanes => require(factories.grid_lanes, "grid-lanes", node),
        EDisplay::kMath | EDisplay::kBlockMath => require(factories.mathml, "mathml", node),
        EDisplay::kBlockRuby => require(factories.block_ruby, "inline", node),
        EDisplay::kLayoutCustom => require(factories.custom, "custom", node),
        _ => panic!("Display value has no standalone native object"),
    }
}
