// Layout mutation admission and invalidation policy live outside the engine facade.
// LayoutEngine keeps stable public entry points while sampling resolves here.

use super::*;

pub(super) trait NativeEngineMutation {
    fn geometry_input_matches(
        &self,
        id: u64,
        style: &crate::internal::layout_input::ComputedStyle,
        element: &Option<crate::internal::layout_input::ElementData>,
    ) -> bool;
    fn apply_mutation(&mut self, mutation: LayoutMutation<'_>) -> bool;
}

fn retain_invalidated_layout_result(engine: &mut LayoutEngine) {
    if let Some(fragments) = engine.fragments.take() {
        engine.invalidated_fragments_ = Some(fragments);
    }
}

impl NativeEngineMutation for LayoutEngine {
    #[inline(never)]
    fn geometry_input_matches(
        &self,
        id: u64,
        style: &crate::internal::layout_input::ComputedStyle,
        element: &Option<crate::internal::layout_input::ElementData>,
    ) -> bool {
        self.tree_.as_ref().is_some_and(|tree| {
            !tree.Root().NeedsLayout() && tree.GeometryInputMatches(id, style, element)
        })
    }

    #[inline(never)]
    fn apply_mutation(&mut self, mutation: LayoutMutation<'_>) -> bool {
        // EmitConstraints is deliberately unconditional at the DOM boundary:
        // compatibility mode is resolved there. Do not let an identical
        // value erase a valid local-layout hint on every frame.
        if let LayoutMutation::Constraints(space) = &mutation {
            let same = SameConstraints(&self.constraints_, space);
            let mut trace =
                layout_invalidation::trace_span(layout_invalidation::TracePhase::Constraints);
            trace.set("same", same as u8 as f64);
            trace.set(
                "scalars_same",
                SameConstraintScalars(&self.constraints_, space) as u8 as f64,
            );
            trace.set(
                "fonts_same",
                (self.constraints_.fonts == space.fonts) as u8 as f64,
            );
            trace.set(
                "font_queries_same",
                (self.constraints_.font_queries == space.font_queries) as u8 as f64,
            );
            trace.set(
                "image_geometry_same",
                SameImageCatalogGeometry(&self.constraints_.images, &space.images) as u8 as f64,
            );
            trace.set(
                "providers_same",
                (SameOptionalArc(&self.constraints_.hyphenation, &space.hyphenation)
                    && SameOptionalArc(&self.constraints_.phrase_break, &space.phrase_break)
                    && SameOptionalArc(&self.constraints_.text_transform, &space.text_transform)
                    && SameOptionalArc(
                        &self.constraints_.font_backend_factory,
                        &space.font_backend_factory,
                    )) as u8 as f64,
            );
            if same {
                return true;
            }
        }
        // Invalidation follows the mutation result below. Compatible SetStyle
        // updates keep native object identities, and rejected/no-op attachment
        // transactions must not discard their retained native state.
        if !matches!(&mutation, LayoutMutation::PaintInput { .. }) {
            self.pending_paint_ids_.clear();
            self.pending_inline_ink_ids_.clear();
        }
        if !matches!(
            &mutation,
            LayoutMutation::ScrollOffset { .. }
                | LayoutMutation::PaintStyle { .. }
                | LayoutMutation::PaintInput { .. }
        ) {
            if let Some(index) = self.scroll_paint_index_.take() {
                self.scroll_paint_index_spare_ = Some(index);
            }
            self.pending_scroll_ids_.clear();
        }
        if !matches!(
            &mutation,
            LayoutMutation::SubtreeUpdate { .. }
                | LayoutMutation::SelectionPaint { .. }
                | LayoutMutation::EditingState(_)
        ) {
            self.pending_subtree_layout_id_ = None;
        }
        match mutation {
            LayoutMutation::Constraints(space) => {
                self.shape_image_inputs_dirty_ |=
                    !SameShapeImageCatalog(&self.constraints_.images, &space.images);
                self.document_image_container_dirty_ |=
                    self.constraints_.document_image_container != space.document_image_container;
                self.constraints_ = space.clone();
                retain_invalidated_layout_result(self);
                true
            }
            LayoutMutation::TreeUpdate(update) => {
                let mut trace =
                    layout_invalidation::trace_span(layout_invalidation::TracePhase::TreeUpdate);
                retain_invalidated_layout_result(self);
                // DOM projection has always used a deferred construction font
                // resolver. Layout's boundary installs the current resources;
                // do not build a duplicate font catalog while creating inputs.
                let tree = self.tree_.get_or_insert_with(|| {
                    LayoutObjectTree::new_with_factories(
                        &ConstraintSpace::default(),
                        &self.objects_,
                    )
                });
                tree.Update(|tree| {
                    let mut attachment = LayoutTreeUpdate::new(tree);
                    update(&mut attachment);
                });
                let stats = tree.UpdateStats();
                trace.set("created", stats.created as f64);
                trace.set("reused", stats.reused as f64);
                trace.set("updated", stats.updated as f64);
                trace.set("removed", stats.removed as f64);
                trace.set("moved", stats.moved as f64);
                self.boundary_cache_.Clear();
                self.tree_facts_ = None;
                self.tree_relations_dirty_ = true;
                self.targeted_font_objects_ = None;
                true
            }
            LayoutMutation::RetainTree => {
                let mut trace = layout_reuse::retain_tree_span();
                let retained = self.tree_.as_ref().is_some_and(|tree| tree.HasRoot());
                trace.set("retained", retained as u8 as f64);
                retained
            }
            LayoutMutation::SubtreeUpdate {
                node_id,
                parent_id,
                update,
            } => {
                let mut trace =
                    layout_invalidation::trace_span(layout_invalidation::TracePhase::SubtreeUpdate);
                trace.set("node_id", node_id as f64);
                trace.set("had_fragments", self.fragments.is_some() as u8 as f64);
                let applied = self.tree_.as_mut().is_some_and(|tree| {
                    tree.UpdateSubtree(node_id, parent_id, |tree, parent| {
                        update(&mut LayoutTreeUpdate::new(tree), parent);
                    })
                });
                if applied {
                    self.pending_subtree_layout_id_ =
                        match (self.pending_subtree_layout_id_, self.fragments.is_some()) {
                            (None, true) => Some(node_id),
                            (Some(previous), true) if previous == node_id => Some(node_id),
                            _ => {
                                self.fragments = None;
                                None
                            }
                        };
                    let stats = self.tree_.as_ref().unwrap().UpdateStats();
                    trace.set("created", stats.created as f64);
                    trace.set("reused", stats.reused as f64);
                    trace.set("updated", stats.updated as f64);
                    trace.set("removed", stats.removed as f64);
                    trace.set("moved", stats.moved as f64);
                    if stats.created != 0
                        || stats.updated != 0
                        || stats.removed != 0
                        || stats.moved != 0
                    {
                        retain_invalidated_layout_result(self);
                        self.boundary_cache_.Clear();
                        self.tree_facts_ = None;
                        self.tree_relations_dirty_ = true;
                        self.targeted_font_objects_ = None;
                    }
                }
                trace.set(
                    "pending_subtree_layout",
                    self.pending_subtree_layout_id_.is_some() as u8 as f64,
                );
                trace.set("applied", applied as u8 as f64);
                applied
            }
            LayoutMutation::NodeInput {
                node_id,
                style,
                element,
            } => {
                let mut trace =
                    layout_invalidation::trace_span(layout_invalidation::TracePhase::NodeInput);
                trace.set("node_id", node_id as f64);
                let update = self
                    .tree_
                    .as_mut()
                    .and_then(|tree| tree.UpdateNodeInput(node_id, style, element));
                if let Some(update) = update.as_ref() {
                    RefreshCompatibleNodeInputCache(&mut self.boundary_cache_, update);
                    self.tree_facts_ = None;
                    if update.relations_changed {
                        retain_invalidated_layout_result(self);
                    }
                    self.tree_relations_dirty_ |= update.relations_changed;
                    trace.set("objects", update.objects.len() as f64);
                    trace.set("relations_changed", update.relations_changed as u8 as f64);
                    if let Some(targeted) = &mut self.targeted_font_objects_ {
                        for change in &update.objects {
                            if !targeted.contains(&change.object) {
                                targeted.push(change.object);
                            }
                        }
                    }
                }
                let applied = update.is_some();
                trace.set("applied", applied as u8 as f64);
                applied
            }
            LayoutMutation::PaintStyle {
                node_id,
                style,
                projected,
                clip_resource_version,
            } => {
                let mut trace = layout_fragment_boundary::trace::span(
                    layout_fragment_boundary::trace::TracePhase::PaintStyle,
                );
                trace.set(
                    "cached_objects_before",
                    self.boundary_cache_.objects.len() as f64,
                );
                let binding = self
                    .tree_
                    .as_mut()
                    .and_then(|tree| tree.UpdatePaintStyle(node_id, style, clip_resource_version));
                if let Some(binding) = binding.as_ref() {
                    RefreshCompatiblePaintCache(&mut self.boundary_cache_);
                    self.tree_facts_ = None;
                    if let Some(fragments) = &mut self.fragments {
                        let mut found = false;
                        let paint_style = crate::internal::boundary::native_paint_style::ExportStyleWithProjectedSvg(
                                style,
                                binding.image_resource_id,
                                projected,
                                &binding.svg_clip_resources,
                            );
                        VisitNativePaintInputsMut(fragments, &self.objects_, &mut |fragment| {
                            UpdateFragmentPaintStyle(fragment, node_id, &paint_style, &mut found)
                        });
                        if !found {
                            self.fragments = None;
                            return false;
                        }
                    }
                }
                trace.set(
                    "cached_objects_after",
                    self.boundary_cache_.objects.len() as f64,
                );
                binding.is_some()
            }
            LayoutMutation::PaintInput {
                node_id,
                style,
                element,
            } => {
                let mut trace = layout_fragment_boundary::trace::span(
                    layout_fragment_boundary::trace::TracePhase::PaintInput,
                );
                trace.set(
                    "cached_objects_before",
                    self.boundary_cache_.objects.len() as f64,
                );
                if self.fragments.is_none() {
                    return false;
                }
                let update = self
                    .tree_
                    .as_mut()
                    .and_then(|tree| tree.UpdatePaintInput(node_id, style, element));
                if let Some(update) = update {
                    RefreshCompatiblePaintCache(&mut self.boundary_cache_);
                    self.tree_facts_ = None;
                    self.pending_paint_ids_.extend(update.sources);
                    self.pending_inline_ink_ids_
                        .extend(update.inline_ink_changes);
                    trace.set(
                        "cached_objects_after",
                        self.boundary_cache_.objects.len() as f64,
                    );
                    true
                } else {
                    false
                }
            }
            LayoutMutation::ReplaceTree(tree) => {
                self.boundary_cache_.Clear();
                self.tree_facts_ = None;
                self.fragments = None;
                self.invalidated_fragments_ = None;
                self.pre_paint_inputs_.invalidate();
                self.tree_ = Some(tree);
                self.tree_relations_dirty_ = true;
                self.targeted_font_objects_ = None;
                true
            }
            LayoutMutation::ScrollOffset { node_id, offset } => {
                self.fragments = None;
                let applied = self
                    .tree_
                    .as_mut()
                    .is_some_and(|tree| tree.UpdateScrollOffset(node_id, offset));
                if applied {
                    self.pending_scroll_ids_.insert(node_id);
                }
                applied
            }
            LayoutMutation::Editing { node_id } => {
                retain_invalidated_layout_result(self);
                self.tree_
                    .as_mut()
                    .is_some_and(|tree| tree.InvalidateEditing(node_id))
            }
            LayoutMutation::SelectionPaint { node_id } => self
                .tree_
                .as_mut()
                .is_some_and(|tree| tree.InvalidateSelectionPaint(node_id)),
            LayoutMutation::EditingState(state) => {
                if let Some(tree) = &mut self.tree_ {
                    tree.SetEditingState(state);
                    true
                } else {
                    false
                }
            }
        }
    }
}
