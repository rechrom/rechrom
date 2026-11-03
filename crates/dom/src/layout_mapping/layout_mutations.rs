#![allow(non_snake_case)]

use crate::{UserInteractionState, DOM};
use layoutng_assembly::{
    editing_state::LayoutEditingState,
    internal::layout_input::{
        CompatibilityMode as LayoutCompatibilityMode, ConstraintSpace, Offset,
    },
    layout_engine::LayoutMutation,
};
use std::rc::Rc;

// DOM owns style/input state. Chromium tree ownership and application of these
// synchronous mutation events belong to the Page's LayoutEngine.
impl DOM {
    /// Finish the document-owned SVG resource lifecycle after computed style
    /// resolution and before an assembly root chooses retained paint. This
    /// conservative invalidation refreshes every consumer on a resource change.
    pub fn UpdateSVGResources(&mut self) -> bool {
        if self.GetDocument().SVGResources().is_some() {
            return false;
        }
        let snapshot = super::svg_clip_path_graph::build_persistent(self.GetDocument());
        let filters = super::svg_filter_graph::build_persistent(self.GetDocument());
        let changed = self.GetDocumentMut().PublishSVGResources(snapshot, filters);
        if changed {
            self.GetDocumentMut()
                .StyleStateMut()
                .layout_inputs_require_tree = true;
            // Resource clients can lie outside the local changed-style batch.
            // Refresh their bindings through the ordinary export lifecycle;
            // an unrelated element's empty binding cannot certify them.
            self.GetDocumentMut()
                .StyleStateMut()
                .paint_style_requires_export = true;
            self.InvalidatePaint();
        }
        changed
    }

    pub fn EmitLayoutMutations(
        &mut self,
        interaction: &UserInteractionState,
        mut receive: impl FnMut(LayoutMutation<'_>) -> bool,
    ) {
        let state = self.GetDocument().StyleState();
        let mut trace = browser_tracing::span("layout", "DOM.EmitLayoutMutations");
        trace.set(
            "requires_tree",
            state.layout_inputs_require_tree as u8 as f64,
        );
        trace.set("attachment_roots", state.layout_tree_roots.len() as f64);
        trace.set("reattach_nodes", state.layout_reattach_nodes.len() as f64);
        trace.set("reattach", state.impact.reattach as u8 as f64);
        trace.set(
            "paint_requires_export",
            state.paint_style_requires_export as u8 as f64,
        );
        trace.set("layout_style_nodes", state.layout_style_nodes.len() as f64);
        trace.set("paint_style_nodes", state.paint_style_nodes.len() as f64);
        // Style/layout can already have consumed attachment inputs during a
        // synchronous query while the frame's paint impact remains pending.
        // The engine confirms residency; a new engine still needs attachment.
        if !state.layout_inputs_require_tree
            && state.layout_tree_roots.is_empty()
            && state.layout_reattach_nodes.is_empty()
            && state.layout_style_nodes.is_empty()
            && state.geometry_input_nodes.is_empty()
            && receive(LayoutMutation::RetainTree)
        {
            return;
        }
        // Paint impact remains pending until frame commit. Only unconsumed
        // attachment/style inputs decide whether this layout needs a rebuild.
        // Element::RecalcOwnStyle applies ordinary SetStyle updates even when
        // another element in the same lifecycle needs layout-tree attachment.
        // Keep those two work lists independent: one structural root must not
        // turn every unrelated style input into a subtree transaction.
        let can_apply_local = !state.layout_inputs_require_tree;
        let reattach_nodes = state.layout_reattach_nodes.clone();
        // Style publishes this work in its parent-first recalc order. Preserve
        // that source order instead of recovering it from the DOM afterward.
        let mut nodes = state.layout_style_nodes.iter().copied().collect::<Vec<_>>();
        let mut needs_parent_sort = false;
        for &id in &state.geometry_input_nodes {
            if !state.layout_style_nodes.contains(&id) {
                nodes.push(id);
                needs_parent_sort = true;
            }
        }
        trace.set("requires_parent_sort", needs_parent_sort as u8 as f64);
        let document = self.GetDocument();
        if needs_parent_sort {
            // Geometry-only updates can enter outside Style traversal. Mixed
            // work retains the conservative parent-first fallback.
            nodes.sort_by_cached_key(|&id| {
                let mut depth = 0;
                let mut node = document.FindNodeById(id);
                while let Some(index) = node {
                    depth += 1;
                    node = document.FlatTreeParentElement(index);
                }
                (depth, id)
            });
        }
        let mut locally_applied = Vec::new();
        let mut local_attempts = 0;
        if can_apply_local {
            for id in nodes {
                if reattach_nodes.contains(&id) {
                    continue;
                }
                local_attempts += 1;
                let applied = self.EmitPaintInputs(id, interaction, |mutation| match mutation {
                    LayoutMutation::PaintInput {
                        node_id,
                        style,
                        resources,
                        element,
                    } => receive(LayoutMutation::NodeInput {
                        node_id,
                        style,
                        resources,
                        element,
                    }),
                    _ => unreachable!("EmitPaintInputs emits typed node inputs"),
                });
                if applied {
                    locally_applied.push(id);
                }
            }
        }
        if !locally_applied.is_empty() {
            let state = self.GetDocumentMut().StyleStateMut();
            for id in &locally_applied {
                state.layout_style_nodes.remove(id);
                state.geometry_input_nodes.remove(id);
            }
        }
        trace.set("local_inputs_attempted", local_attempts as f64);
        trace.set("local_inputs_applied", locally_applied.len() as f64);
        let state = self.GetDocument().StyleState();
        let has_pending_attachment = state.layout_inputs_require_tree
            || !state.layout_tree_roots.is_empty()
            || !state.layout_reattach_nodes.is_empty()
            || !state.layout_style_nodes.is_empty()
            || !state.geometry_input_nodes.is_empty();
        let mut accepted = !locally_applied.is_empty() && !has_pending_attachment;
        if !accepted && !state.layout_inputs_require_tree {
            accepted =
                super::persistent_tree_builder::EmitLayoutSubtrees(self, interaction, &mut receive);
        }
        if !accepted {
            super::EmitLayoutMutations(self, interaction, |mutation| {
                accepted = receive(mutation);
            });
        }
        if accepted {
            let state = self.GetDocumentMut().StyleStateMut();
            state.layout_inputs_require_tree = false;
            state.layout_tree_roots.clear();
            state.layout_reattach_nodes.clear();
            state.layout_style_nodes.clear();
            state.geometry_input_nodes.clear();
        }
    }

    /// Emit the resolved style for one element whose animation sample contains
    /// only non-geometric paint properties. LayoutEngine still validates the
    /// resident source object before accepting the targeted update.
    pub fn EmitPaintStyle(
        &self,
        node_id: u64,
        mut receive: impl FnMut(LayoutMutation<'_>),
    ) -> bool {
        let document = self.GetDocument();
        let Some(resources) = document.SVGResources() else {
            return false;
        };
        let Some(index) = document.FindNodeById(node_id) else {
            return false;
        };
        let Some(resolved) = document.ResolvedStyleFor(index) else {
            return false;
        };
        let style = resolved.computed_style_owner.Get();
        let Some(style) = (unsafe { style.as_ref() }) else {
            return false;
        };
        let clip_resources = super::layout_object_tree_builder::svg_clip_resource_binding(
            style,
            resources,
            document.SVGResourceScope(index),
        );
        let resolved_resources = document.ResolvedStyleResourcesFor(index);
        let svg_filter_graphs = document.SVGFilterGraphs().unwrap();
        let replaced_image_shader = None;
        let style_resources = layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
            svg_filter_graphs,
            &clip_resources,
            &resolved_resources.svg_mask_resource,
            &resolved_resources.svg_fill_server,
            &resolved_resources.svg_stroke_server,
            &resolved_resources.svg_markers,
            &replaced_image_shader,
        );
        receive(LayoutMutation::PaintStyle {
            node_id,
            style,
            resources: &style_resources,
            clip_resource_version: if super::layout_object_tree_builder::needs_svg_clip_resources(
                style,
            ) {
                resources.version
            } else {
                Default::default()
            },
        });
        true
    }

    /// Deliver ordinary paint-only style and source theme state to resident
    /// objects. Text uses its resolved flat-tree parent's style, exactly as
    /// AppendChildren does during attachment. Generated/flattened sources keep
    /// the ordinary export path until their separate source can be retained.
    pub fn EmitPaintInputs(
        &self,
        node_id: u64,
        interaction: &UserInteractionState,
        mut receive: impl FnMut(LayoutMutation<'_>) -> bool,
    ) -> bool {
        use crate::persistent_document::{DOMNamespace, DOMNodeType};
        let document = self.GetDocument();
        let Some(resources) = document.SVGResources() else {
            return false;
        };
        let Some(index) = document.FindNodeById(node_id) else {
            return false;
        };
        let Some(resolved) = document.ResolvedStyleFor(index) else {
            return false;
        };
        if document.Node(index).Namespace() != DOMNamespace::kHTML
            || resolved.display_contents
            || !resolved.own_generates_box
            || resolved.first_letter.is_some()
            || resolved.marker.is_some()
        {
            return false;
        }
        let style = resolved.computed_style_owner.Get();
        let Some(style) = (unsafe { style.as_ref() }) else {
            return false;
        };
        let clip_resources = super::layout_object_tree_builder::svg_clip_resource_binding(
            style,
            resources,
            document.SVGResourceScope(index),
        );
        let resolved_resources = document.ResolvedStyleResourcesFor(index);
        let svg_filter_graphs = document.SVGFilterGraphs().unwrap();
        let replaced_image_shader = None;
        let style_resources = layoutng_assembly::style_projection::StyleResourceBindings::FromParts(
            svg_filter_graphs,
            &clip_resources,
            &resolved_resources.svg_mask_resource,
            &resolved_resources.svg_fill_server,
            &resolved_resources.svg_stroke_server,
            &resolved_resources.svg_markers,
            &replaced_image_shader,
        );
        let element = super::persistent_tree_builder::ElementMetadata(
            document,
            index,
            interaction,
            style.ImageOrientation()
                == foundation::RespectImageOrientationEnum::kRespectImageOrientation,
        );
        if !receive(LayoutMutation::PaintInput {
            node_id,
            style,
            resources: &style_resources,
            element: Some(element),
        }) {
            return false;
        }
        for &child in super::persistent_tree_builder::AttachmentChildren(document, index).iter() {
            let node = document.Node(child);
            if node.Type() == DOMNodeType::kText
                && !node.Data().is_empty()
                && !receive(LayoutMutation::PaintInput {
                    node_id: node.Id(),
                    style,
                    resources: &style_resources,
                    element: None,
                })
            {
                return false;
            }
        }
        true
    }

    /// Return whether a live DOM node was found and a scroll mutation emitted.
    /// A receiver can separately inspect its source engine's application result.
    pub fn EmitScrollOffset(
        &mut self,
        node_id: u64,
        offset: Offset,
        mut receive: impl FnMut(LayoutMutation<'_>),
    ) -> bool {
        if !self.SetScrollOffset(node_id, offset) {
            return false;
        }
        receive(LayoutMutation::ScrollOffset { node_id, offset });
        true
    }

    pub fn EmitEditing(&self, node_id: u64, mut receive: impl FnMut(LayoutMutation<'_>)) -> bool {
        if self.GetDocument().FindNodeById(node_id).is_none() {
            return false;
        }
        receive(LayoutMutation::Editing { node_id });
        true
    }

    pub fn EmitEditingState(
        &self,
        state: Rc<LayoutEditingState>,
        mut receive: impl FnMut(LayoutMutation<'_>),
    ) {
        receive(LayoutMutation::EditingState(state));
    }

    pub fn EmitConstraints(
        &self,
        space: &ConstraintSpace,
        mut receive: impl FnMut(LayoutMutation<'_>),
    ) {
        // Document mode is parser-owned semantic state, while viewport and
        // platform inputs come from the Page host.  Resolve them at this DOM →
        // Layout boundary so callers cannot accidentally lay out a quirks
        // document with the host's default standards-mode constraint.
        let mut resolved = space.clone();
        resolved.compatibility_mode = match self.GetDocument().GetCompatibilityMode() {
            crate::persistent_document::CompatibilityMode::kNoQuirks => {
                LayoutCompatibilityMode::kStandards
            }
            crate::persistent_document::CompatibilityMode::kLimitedQuirks => {
                LayoutCompatibilityMode::kLimitedQuirks
            }
            crate::persistent_document::CompatibilityMode::kQuirks => {
                LayoutCompatibilityMode::kQuirks
            }
        };
        receive(LayoutMutation::Constraints(&resolved));
    }
}
