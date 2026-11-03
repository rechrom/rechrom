#![allow(non_snake_case)]

use crate::internal::{
    layout_input::{ComputedStyle, ElementData, NodeKind},
    layout_object::LayoutObject,
    layout_object_builder::LayoutObjectTree,
};
use layoutng_style::style::computed_style::ComputedStyle as StyleComputedStyle;

/// Synchronous DOM attachment capability for an engine-owned source tree.
///
/// This Rust boundary corresponds to Blink's AttachLayoutTree/LayoutTreeBuilder
/// attachment work. Every operation delegates to the existing translated
/// LayoutObjectTree builder; it introduces no reconciliation or layout algorithm.
/// The engine owns Update, its cleanup and tree lifetime. Callers cannot obtain
/// the underlying tree or start/end another transaction through this handle.
pub struct LayoutTreeUpdate<'a> {
    tree: &'a mut LayoutObjectTree,
}

impl<'a> LayoutTreeUpdate<'a> {
    pub(crate) fn new(tree: &'a mut LayoutObjectTree) -> Self {
        Self { tree }
    }

    /// Consume Style's per-node reattachment decision before creating inputs.
    pub fn Reattach(&mut self, node_ids: &[u64]) {
        self.tree.Reattach(node_ids);
    }

    fn StyleInput(
        style: &StyleComputedStyle,
        resources: &crate::style_projection::StyleResourceBindings<'_>,
        image_resource_id: Option<u64>,
    ) -> ComputedStyle {
        crate::style_projection::BorrowComputedStyle(style, resources, image_resource_id)
    }

    pub fn CreateRootFromStyle(
        &mut self,
        id: u64,
        style: &StyleComputedStyle,
        resources: &crate::style_projection::StyleResourceBindings<'_>,
        image_resource_id: Option<u64>,
        debug_name: String,
        element: Option<ElementData>,
    ) -> &mut LayoutObject {
        let input = Self::StyleInput(style, resources, image_resource_id);
        self.tree
            .CreateRoot(id, &input, debug_name, element, std::ptr::null())
    }

    pub fn AddBoxFromStyle(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &StyleComputedStyle,
        resources: &crate::style_projection::StyleResourceBindings<'_>,
        image_resource_id: Option<u64>,
        debug_name: String,
        element: Option<ElementData>,
        kind: NodeKind,
        style_generated: bool,
    ) -> &mut LayoutObject {
        let input = Self::StyleInput(style, resources, image_resource_id);
        self.tree.AddBox(
            parent,
            id,
            &input,
            debug_name,
            element,
            kind,
            style_generated,
            std::ptr::null(),
        )
    }

    pub fn AddReplacedFromStyle(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &StyleComputedStyle,
        resources: &crate::style_projection::StyleResourceBindings<'_>,
        image_resource_id: Option<u64>,
        element: Option<ElementData>,
        debug_name: String,
        kind: NodeKind,
    ) -> &mut LayoutObject {
        let input = Self::StyleInput(style, resources, image_resource_id);
        self.tree
            .AddReplaced(parent, id, &input, element, debug_name, kind)
    }

    pub fn AddGeneratedReplacedFromStyle(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &StyleComputedStyle,
        resources: &crate::style_projection::StyleResourceBindings<'_>,
        image_resource_id: Option<u64>,
        element: ElementData,
        debug_name: String,
    ) -> &mut LayoutObject {
        let input = Self::StyleInput(style, resources, image_resource_id);
        self.tree
            .AddGeneratedReplaced(parent, id, &input, element, debug_name)
    }

    pub fn AddTextFromStyle(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        text: String,
        debug_name: String,
        style: &StyleComputedStyle,
        resources: &crate::style_projection::StyleResourceBindings<'_>,
        style_generated: bool,
    ) -> &mut LayoutObject {
        if std::ptr::eq(parent.StyleRef(), style) {
            return self.tree.AddText(
                parent,
                id,
                text,
                debug_name,
                std::ptr::null(),
                style_generated,
                std::ptr::null(),
            );
        }
        let input = Self::StyleInput(style, resources, None);
        self.tree.AddText(
            parent,
            id,
            text,
            debug_name,
            &input,
            style_generated,
            std::ptr::null(),
        )
    }

    pub fn CreateRoot(
        &mut self,
        id: u64,
        style: &ComputedStyle,
        debug_name: String,
        element: Option<ElementData>,
        first_line_style: *const ComputedStyle,
    ) -> &mut LayoutObject {
        self.tree
            .CreateRoot(id, style, debug_name, element, first_line_style)
    }

    pub fn CreateRootDefault(&mut self, id: u64, style: &ComputedStyle) -> &mut LayoutObject {
        self.tree.CreateRootDefault(id, style)
    }

    pub fn AddBox(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
        debug_name: String,
        element: Option<ElementData>,
        kind: NodeKind,
        style_generated: bool,
        first_line_style: *const ComputedStyle,
    ) -> &mut LayoutObject {
        self.tree.AddBox(
            parent,
            id,
            style,
            debug_name,
            element,
            kind,
            style_generated,
            first_line_style,
        )
    }

    pub fn AddBoxDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
    ) -> &mut LayoutObject {
        self.tree.AddBoxDefault(parent, id, style)
    }

    pub fn AddReplaced(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
        element: Option<ElementData>,
        debug_name: String,
        kind: NodeKind,
    ) -> &mut LayoutObject {
        self.tree
            .AddReplaced(parent, id, style, element, debug_name, kind)
    }

    pub fn AddGeneratedReplaced(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
        element: ElementData,
        debug_name: String,
    ) -> &mut LayoutObject {
        self.tree
            .AddGeneratedReplaced(parent, id, style, element, debug_name)
    }

    pub fn AddText(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        text: String,
        debug_name: String,
        style: *const ComputedStyle,
        style_generated: bool,
        source_style: *const ComputedStyle,
    ) -> &mut LayoutObject {
        self.tree.AddText(
            parent,
            id,
            text,
            debug_name,
            style,
            style_generated,
            source_style,
        )
    }

    pub fn AddTextDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        text: String,
    ) -> &mut LayoutObject {
        self.tree.AddTextDefault(parent, id, text)
    }

    pub fn NeedsCollapsibleWhitespace(&self, parent: &LayoutObject) -> bool {
        self.tree.NeedsCollapsibleWhitespace(parent)
    }
}
