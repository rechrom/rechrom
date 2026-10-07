#![allow(non_snake_case)]

use crate::internal::{
    layout_input::{ComputedStyle, ElementData, NodeKind},
    layout_object::LayoutObject,
    layout_object_builder::LayoutObjectTree,
};

/// Synchronous DOM attachment capability for an engine-owned native tree.
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
