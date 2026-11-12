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
    device_scale_factor: f64,
    hit_test_owner: Option<u64>,
    // AttachContext aliases to the already-attached native pseudo boxes.
    // Only this transaction retains them; this directory owns no geometry.
    overscroll_area_parents: std::collections::HashMap<u64, *mut LayoutObject>,
}

impl<'a> LayoutTreeUpdate<'a> {
    pub fn SelectedBrokenImageResourceId(
        &self,
        variants: &image_resource::platform_resources::BrokenImageResourceVariants,
    ) -> Option<u64> {
        let content = if self.device_scale_factor >= 2.0 {
            &variants.scale200
        } else {
            &variants.scale100
        };
        content.Metadata().map(|metadata| metadata.id)
    }

    pub fn UseBrokenImageResource(
        &self,
        object: &mut LayoutObject,
        variants: image_resource::platform_resources::BrokenImageResourceVariants,
    ) -> bool {
        if !object.IsLayoutImage() {
            return false;
        }
        let factories = crate::internal::layout_pass_scope::LayoutObjectFactoryScope::Objects();
        assert!(
            !factories.is_null(),
            "LayoutObjectFactoryScope is not active"
        );
        let callback = unsafe { &*factories }
            .replaced_broken_image_resource_updater
            .expect("broken LayoutImageResource updater is not installed");
        callback(
            unsafe {
                &mut *std::ptr::from_mut(object)
                    .cast::<crate::internal::layout_replaced::LayoutReplaced>()
            },
            variants,
            self.device_scale_factor,
        )
    }

    /// Install ImageLoader's same ImageResourceContent owner on the native
    /// LayoutImage created or retained by this attachment transaction.
    pub fn UpdateHTMLImageResource(
        object: &mut LayoutObject,
        update: image_resource::ImageResourceContentUpdate,
    ) -> bool {
        let node = object.GetNode();
        if !object.IsLayoutImage() || node.is_null() || unsafe { &*node }.InputIsStyleGenerated() {
            return false;
        }
        let factories = crate::internal::layout_pass_scope::LayoutObjectFactoryScope::Objects();
        assert!(
            !factories.is_null(),
            "LayoutObjectFactoryScope is not active"
        );
        let callback = unsafe { &*factories }
            .replaced_image_resource_updater
            .expect("LayoutImageResource updater is not installed");
        callback(
            unsafe {
                &mut *std::ptr::from_mut(object)
                    .cast::<crate::internal::layout_replaced::LayoutReplaced>()
            },
            update,
        )
    }

    pub fn UpdateVideoPosterResource(
        object: &mut LayoutObject,
        update: image_resource::ImageResourceContentUpdate,
    ) -> bool {
        if !object.IsVideo() {
            return false;
        }
        let factories = crate::internal::layout_pass_scope::LayoutObjectFactoryScope::Objects();
        assert!(
            !factories.is_null(),
            "LayoutObjectFactoryScope is not active"
        );
        let callback = unsafe { &*factories }
            .replaced_video_poster_resource_updater
            .expect("LayoutVideo poster resource updater is not installed");
        callback(
            unsafe {
                &mut *std::ptr::from_mut(object)
                    .cast::<crate::internal::layout_replaced::LayoutReplaced>()
            },
            update,
        )
    }

    /// Route DOM-owned editing facts onto the same native node being attached.
    /// This deliberately takes the already returned owner instead of building
    /// an id index during every element's attachment.
    pub fn SetEditingNodeFacts(object: &mut LayoutObject, facts: crate::EditingNodeFacts) {
        if let Some(node) = unsafe { object.GetNode().as_mut() } {
            node.SetInputEditingFacts(facts);
        }
    }

    /// Native attachment count, sampled around one construction call. This
    /// distinguishes a newly attached object from an in-place retained update.
    pub fn CreatedObjectCount(&self) -> usize {
        self.tree.UpdateStats().created
    }
    pub(crate) fn new(tree: &'a mut LayoutObjectTree) -> Self {
        Self::new_with_device_scale(tree, 1.0)
    }
    pub(crate) fn new_with_device_scale(
        tree: &'a mut LayoutObjectTree,
        device_scale_factor: f64,
    ) -> Self {
        Self {
            tree,
            device_scale_factor,
            hit_test_owner: None,
            overscroll_area_parents: Default::default(),
        }
    }

    /// Attachment-scoped alias to the same DOM generated-pseudo owner. Every
    /// created LayoutObject receives just its source Node identity link.
    pub fn WithGeneratedPseudoElementOwner<R>(
        &mut self,
        owner: u64,
        attach: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let previous = self.hit_test_owner.replace(owner);
        let result = attach(self);
        self.hit_test_owner = previous;
        result
    }
    pub fn BindOverscrollAreaParent(&mut self, area: u64, parent: *mut LayoutObject) {
        self.overscroll_area_parents.insert(area, parent);
    }
    pub fn OverscrollAreaParent(&self, area: u64) -> Option<*mut LayoutObject> {
        self.overscroll_area_parents.get(&area).copied()
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
        self.tree
            .AddBox(
                parent,
                id,
                &input,
                debug_name,
                element,
                kind,
                style_generated,
                std::ptr::null(),
            )
            .WithHitTestNodeOwner(self.hit_test_owner)
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
            .WithHitTestNodeOwner(self.hit_test_owner)
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
            .WithHitTestNodeOwner(self.hit_test_owner)
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
            return self
                .tree
                .AddText(
                    parent,
                    id,
                    text,
                    debug_name,
                    std::ptr::null(),
                    style_generated,
                    std::ptr::null(),
                )
                .WithHitTestNodeOwner(self.hit_test_owner);
        }
        let input = Self::StyleInput(style, resources, None);
        self.tree
            .AddText(
                parent,
                id,
                text,
                debug_name,
                &input,
                style_generated,
                std::ptr::null(),
            )
            .WithHitTestNodeOwner(self.hit_test_owner)
    }

    /// Element::RebuildLayoutTree, element.cc6142-6147. The source input
    /// projection below revisits every whitespace child of this attachment.
    pub fn BeginElementLayoutTreeAttachment(object: &mut LayoutObject) {
        if object.WhitespaceChildrenMayChange() {
            object.SetWhitespaceChildrenMayChange(false);
        }
    }
    /// Element::AttachLayoutTree/RebuildLayoutTree, cc4888/6189/6336-6339.
    /// Called by the existing element-by-element producer after its children.
    pub fn FinishElementLayoutTreeAttachment(object: &mut LayoutObject) {
        object.HandleSubtreeModifications();
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
        self.tree
            .AddBox(
                parent,
                id,
                style,
                debug_name,
                element,
                kind,
                style_generated,
                first_line_style,
            )
            .WithHitTestNodeOwner(self.hit_test_owner)
    }

    pub fn AddBoxDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
    ) -> &mut LayoutObject {
        self.tree
            .AddBoxDefault(parent, id, style)
            .WithHitTestNodeOwner(self.hit_test_owner)
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
            .WithHitTestNodeOwner(self.hit_test_owner)
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
            .WithHitTestNodeOwner(self.hit_test_owner)
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
        self.tree
            .AddText(
                parent,
                id,
                text,
                debug_name,
                style,
                style_generated,
                source_style,
            )
            .WithHitTestNodeOwner(self.hit_test_owner)
    }

    pub fn AddTextDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        text: String,
    ) -> &mut LayoutObject {
        self.tree
            .AddTextDefault(parent, id, text)
            .WithHitTestNodeOwner(self.hit_test_owner)
    }

    pub fn NeedsCollapsibleWhitespace(&self, parent: &LayoutObject) -> bool {
        self.tree.NeedsCollapsibleWhitespace(parent)
    }
}
