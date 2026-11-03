//! Rust layout-output invalidation observer. Production revisions consume the
//! source LayoutObject dirty flags and add fragment-local geometry/resource
//! changes at the owned-output boundary. Hand-built fragments without source
//! identity retain an exhaustive value-comparison fallback. The resulting
//! object/descendant distinction feeds PrePaintTreeWalk (upstream
//! core/paint/pre_paint_tree_walk.cc:418-436).

use std::sync::Arc;

use super::{FragmentKind, FragmentPaintData, FragmentPaintRect, PaintProperties, PaintResources};
use foundation::{
    IsManagedLayoutAddress, LayoutOffset as Offset, LayoutSize as Size,
    TrustedIdentityHashMap as HashMap, TrustedIdentityHashSet as HashSet, WeakPersistent,
};
pub use layout_fragment_boundary::revision::PrePaintRevisionStats;

/// Version an in-place source scroll/sticky/control change on an already
/// certified layout output. The layout boundary must call this on every
/// changed node and its complete ancestor path. Zero remains unobserved;
/// this operation cannot certify external or previously unobserved inputs.
pub fn MarkScrollPropertyChange(node: &mut FragmentPaintData, own_changed: bool) {
    layout_fragment_boundary::revision::mark_scroll_property_change(node, own_changed);
}

#[derive(Default)]
pub struct PrePaintRevisionTracker {
    // Move stable observation handles between passes, not complete fragments.
    inputs: HashMap<InputKey, Box<Input>>,
    spare_inputs: HashMap<InputKey, Box<Input>>,
    owners: HashMap<u64, LiveOwner>,
    spare_owners: HashMap<u64, LiveOwner>,
    last_stats: PrePaintRevisionStats,
    production_update: Option<ProductionUpdate>,
}

struct ProductionUpdate {
    trace: layout_fragment_boundary::revision::RevisionUpdateTrace,
    generation: u64,
    next: HashMap<InputKey, Box<Input>>,
    previous: HashMap<InputKey, Box<Input>>,
    previous_owners: HashMap<u64, LiveOwner>,
    next_owners: HashMap<u64, LiveOwner>,
    resource_matches: HashMap<(usize, usize, bool), bool>,
    stats: PrePaintRevisionStats,
}

/// A non-owning view of Chromium's source fragment topology. The pointed
/// `FragmentPaintData` is only the fragment-local Paint data attached to a
/// PhysicalFragment or FragmentItem; `parent` describes the source link and
/// never creates a second owned tree.
#[derive(Clone, Copy)]
pub struct FlatPrePaintInput {
    pub node: *mut FragmentPaintData,
    pub parent: Option<usize>,
}

#[derive(Hash, PartialEq, Eq)]
enum InputKey {
    // Production updates preserve the source FragmentData-equivalent identity.
    // Child indices are traversal positions, not object identity.
    Instance(u64),
    // Hand-built external/test fragments may not carry a boundary identity.
    Path(Vec<usize>),
}

struct LiveOwner {
    weak: WeakPersistent<()>,
    // Whether this particular weak cell survived from the previous update.
    retained: bool,
}

struct Input {
    // Production keeps only lifecycle revisions and owner identities. A value
    // snapshot exists solely for hand-built inputs without Chromium source
    // dirty bits.
    revision: u64,
    subtree_revision: u64,
    owner_ids: [u64; 5],
    fallback_snapshot: Option<InputSnapshot>,
    fallback_paint: Option<PaintProperties>,
    resources: Option<Arc<PaintResources>>,
    uses_viewport_resource: bool,
    children: Vec<(u64, u64, u64)>,
}

#[derive(Clone, Copy)]
struct InputSnapshot {
    fragment_instance_id: u64,
    fragmentainer_instance_id: u64,
    node_id: u64,
    kind: FragmentKind,
    offset: Offset,
    size: Size,
    content_size: Size,
    text_start: Option<u32>,
    text_end: Option<u32>,
    logical_parent_node_id: Option<u64>,
    logical_parent_fragment_instance_id: Option<u64>,
    logical_tree_order: u64,
}

impl PrePaintRevisionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invalidate(&mut self) {
        self.inputs.clear();
        self.spare_inputs.clear();
        self.owners.clear();
        self.spare_owners.clear();
        self.last_stats = PrePaintRevisionStats::default();
        self.production_update = None;
    }

    /// Begin a production source walk. Layout calls ObserveProductionNode
    /// directly while following the canonical fragment cursor, so revision
    /// certification does not allocate a flat copy of the fragment topology.
    pub fn BeginProductionUpdate(&mut self) {
        assert!(self.production_update.is_none());
        let mut next = std::mem::take(&mut self.spare_inputs);
        debug_assert!(next.is_empty());
        next.reserve(self.inputs.len());
        let mut next_owners = std::mem::take(&mut self.spare_owners);
        debug_assert!(next_owners.is_empty());
        next_owners.reserve(self.owners.len());
        self.production_update = Some(ProductionUpdate {
            trace: layout_fragment_boundary::revision::update_span(),
            generation: layout_fragment_boundary::revision::next_revision(),
            next,
            previous: std::mem::take(&mut self.inputs),
            previous_owners: std::mem::take(&mut self.owners),
            next_owners,
            resource_matches: HashMap::default(),
            stats: PrePaintRevisionStats::default(),
        });
    }

    pub fn ObserveProductionNode(&mut self, node: &mut FragmentPaintData) {
        assert_ne!(node.fragment_instance_id, 0);
        let update = self
            .production_update
            .as_mut()
            .expect("production revision update has not begun");
        let key = InputKey::Instance(node.fragment_instance_id);
        Self::observe(
            node,
            key,
            &[],
            update.generation,
            &mut update.previous,
            &mut update.next,
            &mut update.resource_matches,
            &mut update.previous_owners,
            &mut update.next_owners,
            &mut update.stats,
            true,
        );
    }

    pub fn FinishProductionUpdate(&mut self) {
        let mut update = self
            .production_update
            .take()
            .expect("production revision update has not begun");
        self.inputs = update.next;
        self.owners = update.next_owners;
        update.previous.clear();
        update.previous_owners.clear();
        self.spare_inputs = update.previous;
        self.spare_owners = update.previous_owners;
        self.last_stats = update.stats;
        update.trace.finish(update.stats);
    }

    pub fn last_stats(&self) -> PrePaintRevisionStats {
        self.last_stats
    }

    /// Mirror the lifecycle validation performed by the real PaintController
    /// commit for its used cacheable clients. This only advances the observed
    /// lifecycle bits; it neither validates source clients nor changes paint
    /// geometry, revisions or weak-owner proofs on its own.
    #[allow(non_snake_case)]
    pub fn CommitClients(&mut self, clients: impl IntoIterator<Item = u64>) {
        let used: HashSet<_> = clients.into_iter().filter(|id| *id != 0).collect();
        if used.is_empty() {
            return;
        }
        let committed = |id, cacheable| cacheable && used.contains(&id);
        for input in self.inputs.values_mut() {
            let Some(paint) = input.fallback_paint.as_mut() else {
                // Chromium clients are validated and have their dirty/lifecycle
                // flags cleared by LayoutEngine's PaintController commit.
                continue;
            };
            if committed(
                paint.display_item_client_id,
                paint.display_item_client_is_cacheable,
            ) {
                paint.display_item_client_invalidation_reason =
                    Some(foundation::PaintInvalidationReason::kNone);
                paint.display_item_client_is_just_created.set(false);
            }
            if committed(
                paint.selection_client_id,
                paint.selection_client_is_cacheable,
            ) {
                paint.selection_client_invalidation_reason =
                    Some(foundation::PaintInvalidationReason::kNone);
                paint.selection_client_is_just_created.set(false);
            }
            if committed(
                paint.paint_layer_client_id,
                paint.paint_layer_client_is_cacheable,
            ) {
                paint.paint_layer_client_invalidation_reason =
                    Some(foundation::PaintInvalidationReason::kNone);
                paint.paint_layer_client_is_just_created.set(false);
            }
            if let Some(scroll) = &mut paint.scroll_container {
                if committed(
                    scroll.scrolling_background_client_id,
                    scroll.scrolling_background_client_is_cacheable,
                ) {
                    scroll.scrolling_background_client_invalidation_reason =
                        Some(foundation::PaintInvalidationReason::kNone);
                    scroll
                        .scrolling_background_client_is_just_created
                        .set(false);
                }
            }
            if let Some(bars) = &mut paint.scrollbars {
                let changes = committed(bars.corner_client_id, bars.corner_client_is_cacheable)
                    || bars.horizontal.as_ref().is_some_and(|axis| {
                        committed(
                            axis.display_item_client_id,
                            axis.display_item_client_is_cacheable,
                        )
                    })
                    || bars.vertical.as_ref().is_some_and(|axis| {
                        committed(
                            axis.display_item_client_id,
                            axis.display_item_client_is_cacheable,
                        )
                    });
                if changes {
                    // Detach only a scrollbar payload whose actual used-client
                    // flags changed. Other shared paint payloads are untouched.
                    let bars = Arc::make_mut(bars);
                    if committed(bars.corner_client_id, bars.corner_client_is_cacheable) {
                        bars.corner_client_invalidation_reason =
                            Some(foundation::PaintInvalidationReason::kNone);
                        bars.corner_client_is_just_created.set(false);
                    }
                    for axis in [&mut bars.horizontal, &mut bars.vertical]
                        .into_iter()
                        .flatten()
                    {
                        if committed(
                            axis.display_item_client_id,
                            axis.display_item_client_is_cacheable,
                        ) {
                            axis.display_item_client_invalidation_reason =
                                Some(foundation::PaintInvalidationReason::kNone);
                            axis.display_item_client_is_just_created.set(false);
                        }
                    }
                }
            }
        }
    }

    /// Certify fragment-local Paint data by following a caller-provided flat
    /// view of the source PhysicalFragment/FragmentItems graph. Entries must be
    /// in preorder, so every parent index is smaller than its children.
    pub fn update_flat(&mut self, nodes: &mut [FlatPrePaintInput]) {
        let mut trace = layout_fragment_boundary::revision::update_span();
        let generation = layout_fragment_boundary::revision::next_revision();
        let mut next = std::mem::take(&mut self.spare_inputs);
        debug_assert!(next.is_empty());
        next.reserve(self.inputs.len());
        let mut resource_matches = HashMap::default();
        let mut previous = std::mem::take(&mut self.inputs);
        let mut previous_owners = std::mem::take(&mut self.owners);
        let mut next_owners = std::mem::take(&mut self.spare_owners);
        let mut stats = PrePaintRevisionStats::default();
        debug_assert!(next_owners.is_empty());
        next_owners.reserve(previous_owners.len());

        let mut children = vec![Vec::<usize>::new(); nodes.len()];
        for (index, entry) in nodes.iter().enumerate() {
            assert!(!entry.node.is_null(), "flat pre-paint input is null");
            if let Some(parent) = entry.parent {
                assert!(parent < index, "flat pre-paint inputs must be preorder");
                children[parent].push(index);
            }
        }
        for index in (0..nodes.len()).rev() {
            let node = unsafe { &mut *nodes[index].node };
            let child_revisions: Vec<_> = children[index]
                .iter()
                .map(|&child| {
                    let child = unsafe { &*nodes[child].node };
                    (
                        child.fragment_instance_id,
                        child.pre_paint_revision,
                        child.pre_paint_subtree_revision,
                    )
                })
                .collect();
            let key = if node.fragment_instance_id == 0 {
                // Production source inputs always have an instance id. Keep a
                // deterministic fallback for diagnostics without inventing a
                // copied-tree path.
                InputKey::Path(vec![index])
            } else {
                InputKey::Instance(node.fragment_instance_id)
            };
            Self::observe(
                node,
                key,
                &child_revisions,
                generation,
                &mut previous,
                &mut next,
                &mut resource_matches,
                &mut previous_owners,
                &mut next_owners,
                &mut stats,
                false,
            );
        }
        self.inputs = next;
        self.owners = next_owners;
        previous.clear();
        previous_owners.clear();
        self.spare_inputs = previous;
        self.spare_owners = previous_owners;
        self.last_stats = stats;
        trace.finish(stats);
    }

    #[allow(clippy::too_many_arguments)]
    fn observe(
        node: &mut FragmentPaintData,
        lookup_key: InputKey,
        child_revisions: &[(u64, u64, u64)],
        generation: u64,
        previous: &mut HashMap<InputKey, Box<Input>>,
        next: &mut HashMap<InputKey, Box<Input>>,
        resource_matches: &mut HashMap<(usize, usize, bool), bool>,
        previous_owners: &mut HashMap<u64, LiveOwner>,
        next_owners: &mut HashMap<u64, LiveOwner>,
        stats: &mut PrePaintRevisionStats,
        trust_source_dirty_bits: bool,
    ) {
        let (key, old) = match previous.remove_entry(&lookup_key) {
            Some((key, old)) => (key, Some(old)),
            None => (lookup_key, None),
        };
        stats.nodes += 1;
        let invalidation = node.paint.pre_paint_invalidation;
        stats.own_invalidations += invalidation.own() as usize;
        stats.descendant_invalidations += invalidation.descendants() as usize;
        stats.needs_paint_property_update += invalidation.needs_paint_property_update as usize;
        stats.descendant_needs_paint_property_update +=
            invalidation.descendant_needs_paint_property_update as usize;
        stats.should_check_for_paint_invalidation +=
            invalidation.should_check_for_paint_invalidation as usize;
        stats.should_check_layout_for_paint_invalidation +=
            invalidation.should_check_layout_for_paint_invalidation as usize;
        stats.descendant_should_check_layout_for_paint_invalidation +=
            invalidation.descendant_should_check_layout_for_paint_invalidation as usize;
        stats.subtree_should_check_for_paint_invalidation +=
            invalidation.subtree_should_check_for_paint_invalidation as usize;
        if old.is_none() {
            stats.new_nodes += 1;
        }
        let owners_live = retain_owners(&node.paint, previous_owners, next_owners);
        node.pre_paint_owner_retained = owners_live
            && old
                .as_ref()
                .is_some_and(|old| old.owner_ids == owner_ids(&node.paint));
        let input = node.fragment_instance_id != 0;
        let (same_identity, same_geometry, same_paint, same_resources) = old
            .as_ref()
            .map(|old| {
                let identity = if input {
                    true
                } else {
                    old.fallback_snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot_identity_equal(snapshot, node))
                };
                let geometry = if input {
                    !invalidation.own()
                } else {
                    old.fallback_snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot_geometry_equal(snapshot, node))
                };
                // Chromium's production PrePaint walk consumes LayoutObject
                // dirty bits. Deep comparison here duplicated that work for
                // every fragment-local input on every resize. External fragments do
                // not have that contract, so retain the exhaustive fallback.
                let paint = if input {
                    !invalidation.own()
                } else {
                    old.fallback_paint
                        .as_ref()
                        .is_some_and(|paint| paint_equal(paint, &node.paint))
                };
                let resources = resources_equal(
                    &old.resources,
                    &node.paint.resources,
                    resource_matches,
                    old.uses_viewport_resource || paint_uses_viewport_resource(&node.paint),
                );
                (identity, geometry, paint, resources)
            })
            .unwrap_or((false, false, false, false));
        if old.is_some() {
            stats.identity_changes += (!same_identity) as usize;
            stats.geometry_changes += (!same_geometry) as usize;
            stats.paint_changes += (!same_paint) as usize;
            stats.resource_changes += (!same_resources) as usize;
        }
        let self_same = old.is_some()
            && (!just_created(&node.paint) || node.pre_paint_owner_retained)
            && same_identity
            && same_geometry
            && same_paint
            && same_resources;
        let children_same = if trust_source_dirty_bits {
            old.is_some() && !invalidation.descendants()
        } else {
            old.as_ref()
                .is_some_and(|old| old.children.as_slice() == child_revisions)
        };
        stats.child_changes += (!children_same) as usize;
        node.pre_paint_revision = if self_same {
            old.as_ref().unwrap().revision
        } else {
            generation
        };
        node.pre_paint_subtree_revision = if self_same && children_same {
            old.as_ref().unwrap().subtree_revision
        } else {
            generation
        };
        // Keep unchanged compact observations rather than rebuilding them on
        // every layout. No fragment tree ownership is retained.
        let mut current = if let Some(mut old) = old {
            if !self_same {
                let children = std::mem::take(&mut old.children);
                *old = Input::from_fragment(node);
                old.children = children;
            }
            old
        } else {
            Box::new(Input::from_fragment(node))
        };
        if !children_same {
            current.children.clear();
            current.children.extend_from_slice(child_revisions);
        }
        current.revision = node.pre_paint_revision;
        current.subtree_revision = node.pre_paint_subtree_revision;
        current.owner_ids = owner_ids(&node.paint);
        next.insert(key, current);
    }
}

impl Input {
    fn from_fragment(node: &FragmentPaintData) -> Self {
        let fallback_paint = if node.fragment_instance_id == 0 {
            let mut paint = (*node.paint).clone();
            paint.resources = None;
            Some(paint)
        } else {
            None
        };
        let resources = node.paint.resources.clone();
        let uses_viewport_resource = paint_uses_viewport_resource(&node.paint);
        // Exhaustive pattern: a new fragment self field cannot accidentally be
        // omitted from this compact observer without a compiler error here.
        let FragmentPaintData {
            // CSSOM-only inline rectangles do not affect paint recording.
            inline_geometry: _,
            // Cull hysteresis belongs to lifecycle state, not this fragment's
            // PrePaint input.
            paint_cull_scroll_offset: _,
            pre_paint_revision: _,
            pre_paint_subtree_revision: _,
            pre_paint_owner_retained: _,
            fragment_instance_id,
            fragmentainer_instance_id,
            node_id,
            kind,
            offset,
            size,
            content_size,
            text_start,
            text_end,
            paint: _,
        } = node;
        Self {
            revision: 0,
            subtree_revision: 0,
            owner_ids: owner_ids(&node.paint),
            fallback_snapshot: (node.fragment_instance_id == 0).then_some(InputSnapshot {
                fragment_instance_id: *fragment_instance_id,
                fragmentainer_instance_id: *fragmentainer_instance_id,
                node_id: *node_id,
                kind: *kind,
                offset: *offset,
                size: *size,
                content_size: *content_size,
                text_start: *text_start,
                text_end: *text_end,
                logical_parent_node_id: node.paint.logical_parent_node_id,
                logical_parent_fragment_instance_id: node.paint.logical_parent_fragment_instance_id,
                logical_tree_order: node.paint.logical_tree_order,
            }),
            fallback_paint,
            resources,
            uses_viewport_resource,
            children: Vec::new(),
        }
    }
}

fn snapshot_identity_equal(a: &InputSnapshot, b: &FragmentPaintData) -> bool {
    a.fragment_instance_id == b.fragment_instance_id
        && a.fragmentainer_instance_id == b.fragmentainer_instance_id
        && a.node_id == b.node_id
        && a.kind == b.kind
        && a.text_start == b.text_start
        && a.text_end == b.text_end
        && a.logical_parent_node_id == b.paint.logical_parent_node_id
        && a.logical_parent_fragment_instance_id == b.paint.logical_parent_fragment_instance_id
        && a.logical_tree_order == b.paint.logical_tree_order
}

fn snapshot_geometry_equal(a: &InputSnapshot, b: &FragmentPaintData) -> bool {
    a.offset == b.offset && a.size == b.size && a.content_size == b.content_size
}

/// Compare all exported self inputs, excluding children and certification
/// revisions/lifecycle metadata. Just-created clients need the boundary's
/// certified live-owner proof. This does not consume source dirty flags.
#[allow(non_snake_case)]
pub fn SamePrePaintInput(a: &FragmentPaintData, b: &FragmentPaintData) -> bool {
    (!just_created(&b.paint) || (b.pre_paint_revision != 0 && b.pre_paint_owner_retained))
        && self_equal(a, b)
        && resources_equal(
            &a.paint.resources,
            &b.paint.resources,
            &mut HashMap::default(),
            paint_uses_viewport_resource(&a.paint) || paint_uses_viewport_resource(&b.paint),
        )
}

fn paint_uses_viewport_resource(paint: &PaintProperties) -> bool {
    use layoutng::internal::paint_input::BackgroundAttachment;
    paint.IsLayoutView()
        || paint.canvas_background.is_some()
        || paint
            .style
            .background_images()
            .iter()
            .any(|layer| layer.attachment == BackgroundAttachment::kFixed)
}

fn owner_ids(paint: &PaintProperties) -> [u64; 5] {
    let mut ids = [
        paint.display_item_client_id,
        paint.paint_layer_client_id,
        0,
        0,
        0,
    ];
    if let Some(bars) = &paint.scrollbars {
        ids[2] = bars.corner_client_id;
        ids[3] = bars
            .horizontal
            .as_ref()
            .map_or(0, |axis| axis.display_item_client_id);
        ids[4] = bars
            .vertical
            .as_ref()
            .map_or(0, |axis| axis.display_item_client_id);
    }
    ids
}

fn retain_owners(
    paint: &PaintProperties,
    previous: &mut HashMap<u64, LiveOwner>,
    next: &mut HashMap<u64, LiveOwner>,
) -> bool {
    let mut retained = paint.display_item_client_id != 0
        && (!paint.paint_layer_client_is_just_created.get() || paint.paint_layer_client_id != 0);
    if let Some(bars) = &paint.scrollbars {
        retained &= (!bars.corner_client_is_just_created.get() || bars.corner_client_id != 0)
            && bars
                .horizontal
                .as_ref()
                .is_none_or(|axis| axis.display_item_client_id != 0)
            && bars
                .vertical
                .as_ref()
                .is_none_or(|axis| axis.display_item_client_id != 0);
    }
    for id in owner_ids(paint).into_iter().filter(|id| *id != 0) {
        // Evaluate every owner even after a rejection so all real clients are
        // registered for the next pass, once per id across every fragment.
        retained &= retain_owner(id, previous, next);
    }
    retained
}

fn retain_owner(
    id: u64,
    previous: &mut HashMap<u64, LiveOwner>,
    next: &mut HashMap<u64, LiveOwner>,
) -> bool {
    if let Some(owner) = next.get(&id) {
        return owner.retained;
    }
    let Ok(address) = usize::try_from(id) else {
        return false;
    };
    let pointer = address as *mut ();
    if !IsManagedLayoutAddress(pointer) {
        return false;
    }
    let old = previous.remove(&id);
    let retained = old
        .as_ref()
        .is_some_and(|owner| owner.weak.Get() == pointer);
    let weak = if retained {
        old.unwrap().weak
    } else {
        // A reclaimed owner's weak was cleared before sweep. Reusing its
        // address never revalidates that old cell. No object is dereferenced.
        WeakPersistent::from_ptr(pointer)
    };
    next.insert(id, LiveOwner { weak, retained });
    retained
}

fn paint_equal(a: &PaintProperties, b: &PaintProperties) -> bool {
    // Exhaustive field binding prevents a new paint output from being omitted.
    let PaintProperties {
        source_object_address: _,
        // Chromium lifecycle facts drive traversal; they are not paint input.
        pre_paint_invalidation: _,
        property_fragment_unique_id,
        is_first_for_node,
        can_paint_multiple_fragments,
        is_layout_view,
        is_inclusive_descendant_of_unbounded_element,
        is_printing,
        canvas_draw_element_enabled,
        is_in_canvas_subtree,
        is_overscroll_area_parent,
        is_overscroll_container,
        needs_layout,
        child_layout_blocked_by_display_lock,
        is_fragment_less_box,
        additional_compositing_reasons,
        is_box_model_object,
        is_layout_box,
        is_page_border_box,
        is_paginated_root,
        is_layout_inline,
        should_apply_paint_containment,
        should_apply_style_containment,
        should_apply_layout_containment,
        should_clip_overflow_along_either_axis,
        should_clip_overflow_along_both_axes,
        is_layout_block,
        is_layout_block_flow,
        is_inline_formatting_context,
        in_forced_colors_mode,
        is_background_attachment_fixed_object,
        requires_main_thread_scrolling_for_background_attachment_fixed,
        display_item_client_invalidation_reason,
        // Source PaintArtifact records debug names only when actually painted;
        // changing this diagnostic name alone does not invalidate painting.
        display_item_client_debug_name: _,
        paint_layer_client_invalidation_reason,
        self_ink_overflow,
        contents_ink_overflow,
        ink_overflow,
        filtered_ink_overflow,
        first_line_decoration_rect,
        text_glyph_ink,
        text_hit_test_bounds,
        box_self_visual_overflow,
        box_fragment_is_inline_box,
        display_item_raster_effect_outset,
        display_item_client_id,
        display_item_client_owner_node_id,
        paint_layer_state_is_authoritative,
        paint_layer_client_id,
        paint_layer_client_is_cacheable,
        paint_layer_client_is_just_created,
        paint_layer_is_self_painting,
        paint_layer_invisible_for_position_visibility,
        paint_layer_has_ancestor_invisible_for_position_visibility,
        paint_layer_has_visible_content,
        paint_layer_has_self_painting_layer_descendant,
        paint_layer_is_under_svg_hidden_container,
        paint_layer_supports_subsequence_caching,
        paint_layer_self_or_descendant_needs_repaint,
        paint_layer_is_fragmented,
        paint_layer_known_to_clip_subtree_to_padding_box,
        paint_layer_first_fragment_visual_rect,
        paint_layer_contents_visual_rect,
        paint_layer_filter_reference_box,
        paint_layer_filter_viewport,
        child_paint_blocked_by_display_lock,
        paint_layer_is_stacking_context,
        paint_layer_is_stacked,
        paint_layer_stacking_level,
        paint_layer_is_z_order_list_visible,
        paint_layer_is_in_top_or_view_transition_layer,
        paint_layer_is_direct_layout_view_child,
        paint_layer_can_contain_absolute_position_objects,
        paint_layer_can_contain_fixed_position_objects,
        paint_layer_has_overlay_overflow_controls,
        paint_layer_order_parent_unique_id,
        paint_layer_order,
        display_item_client_is_cacheable,
        display_item_client_is_just_created,
        selection_client_id,
        selection_client_invalidation_reason,
        selection_client_is_cacheable,
        selection_client_is_just_created,
        display_item_fragment,
        has_source,
        logical_parent_node_id,
        logical_parent_fragment_instance_id,
        logical_tree_order,
        establishes_paint_state,
        hidden,
        is_line_break,
        svg_hidden_container,
        svg_descendant_may_have_transform_related_operations,
        svg_view_box_is_empty,
        painted_atomically,
        has_collapsed_borders,
        source_kind,
        replaced_is_image,
        image_decoding_mode,
        image_last_painted_id: _,
        focused_image_map_area,
        // MutableForPainting state is output of ImagePainter, not Layout's
        // immutable pre-paint input.
        image_last_paint_rect: _,
        effective_zoom,
        style,
        selection_style,
        selection_text_decoration,
        svg_text_decoration_style,
        margin,
        border,
        border_sides,
        border_styles,
        padding,
        stitched_decoration,
        box_decoration_break,
        display,
        position,
        fixed_to_view,
        needs_anchor_position_scroll_adjustment,
        floating,
        sticky_offset,
        overflow_x,
        overflow_y,
        overscroll_behavior_x,
        overscroll_behavior_y,
        prevent_scroll_axis_locking,
        css_clip,
        overflow_clip_margin_outsets,
        scroll_offset,
        hit_test_overflow_clip,
        hit_test_scrolled_content_offset,
        scroll_size,
        scrolls_overflow,
        client_size: _,
        scroll_container,
        writing_mode,
        direction,
        first_baseline,
        text_line_top_offset,
        text_line_height,
        is_ellipsis,
        text_fit_scale,
        text_combine,
        text_combine_inline_offset,
        replaced_content,
        resources: _,
        glyph_runs,
        text_control_host,
        text_control_inner_editor,
        text_control_empty_caret,
        list_marker_symbol,
        list_marker_inside,
        mathml,
        frame_set,
        fieldset,
        column_rules,
        collapsed_table,
        table,
        scrollbars,
        form_control,
        text_control_caret_metrics,
        svg_shape,
        svg_view_box,
        svg_local_transform,
        svg_viewport,
        svg_visual_rect,
        svg_text,
        table_cell_column,
        sticky_constraint,
        background_transfers_to_view,
        canvas_background,
        view_background_invalidation,
        background_paint_location,
        background_fill_layer_count,
        has_background,
        background_should_always_be_clipped,
        background_any_local_attachment_image,
        background_any_default_attachment_image,
        background_any_layer_uses_content_box,
        scrollable_overflow,
        view_background_rect,
        can_composite_background_attachment_fixed,
    } = a;
    property_fragment_unique_id == &b.property_fragment_unique_id
        && is_first_for_node == &b.is_first_for_node
        && can_paint_multiple_fragments == &b.can_paint_multiple_fragments
        && is_layout_view == &b.is_layout_view
        && is_inclusive_descendant_of_unbounded_element
            == &b.is_inclusive_descendant_of_unbounded_element
        && is_printing == &b.is_printing
        && canvas_draw_element_enabled == &b.canvas_draw_element_enabled
        && is_in_canvas_subtree == &b.is_in_canvas_subtree
        && is_overscroll_area_parent == &b.is_overscroll_area_parent
        && is_overscroll_container == &b.is_overscroll_container
        && needs_layout == &b.needs_layout
        && child_layout_blocked_by_display_lock == &b.child_layout_blocked_by_display_lock
        && is_fragment_less_box == &b.is_fragment_less_box
        && additional_compositing_reasons == &b.additional_compositing_reasons
        && is_box_model_object == &b.is_box_model_object
        && is_layout_box == &b.is_layout_box
        && is_page_border_box == &b.is_page_border_box
        && is_paginated_root == &b.is_paginated_root
        && is_layout_inline == &b.is_layout_inline
        && should_apply_paint_containment == &b.should_apply_paint_containment
        && should_apply_style_containment == &b.should_apply_style_containment
        && should_apply_layout_containment == &b.should_apply_layout_containment
        && should_clip_overflow_along_either_axis == &b.should_clip_overflow_along_either_axis
        && should_clip_overflow_along_both_axes == &b.should_clip_overflow_along_both_axes
        && is_layout_block == &b.is_layout_block
        && is_layout_block_flow == &b.is_layout_block_flow
        && is_inline_formatting_context == &b.is_inline_formatting_context
        && in_forced_colors_mode == &b.in_forced_colors_mode
        && is_background_attachment_fixed_object == &b.is_background_attachment_fixed_object
        && requires_main_thread_scrolling_for_background_attachment_fixed
            == &b.requires_main_thread_scrolling_for_background_attachment_fixed
        && display_item_client_invalidation_reason == &b.display_item_client_invalidation_reason
        && paint_layer_client_invalidation_reason == &b.paint_layer_client_invalidation_reason
        && self_ink_overflow == &b.self_ink_overflow
        && contents_ink_overflow == &b.contents_ink_overflow
        && ink_overflow == &b.ink_overflow
        && filtered_ink_overflow == &b.filtered_ink_overflow
        && first_line_decoration_rect == &b.first_line_decoration_rect
        && text_glyph_ink == &b.text_glyph_ink
        && text_hit_test_bounds == &b.text_hit_test_bounds
        && box_self_visual_overflow == &b.box_self_visual_overflow
        && box_fragment_is_inline_box == &b.box_fragment_is_inline_box
        && display_item_raster_effect_outset == &b.display_item_raster_effect_outset
        && display_item_client_id == &b.display_item_client_id
        && display_item_client_owner_node_id == &b.display_item_client_owner_node_id
        && paint_layer_state_is_authoritative == &b.paint_layer_state_is_authoritative
        && paint_layer_client_id == &b.paint_layer_client_id
        && paint_layer_client_is_cacheable == &b.paint_layer_client_is_cacheable
        && paint_layer_client_is_just_created == &b.paint_layer_client_is_just_created
        && paint_layer_is_self_painting == &b.paint_layer_is_self_painting
        && paint_layer_invisible_for_position_visibility
            == &b.paint_layer_invisible_for_position_visibility
        && paint_layer_has_ancestor_invisible_for_position_visibility
            == &b.paint_layer_has_ancestor_invisible_for_position_visibility
        && paint_layer_has_visible_content == &b.paint_layer_has_visible_content
        && paint_layer_has_self_painting_layer_descendant
            == &b.paint_layer_has_self_painting_layer_descendant
        && paint_layer_is_under_svg_hidden_container == &b.paint_layer_is_under_svg_hidden_container
        && paint_layer_supports_subsequence_caching == &b.paint_layer_supports_subsequence_caching
        && paint_layer_self_or_descendant_needs_repaint
            == &b.paint_layer_self_or_descendant_needs_repaint
        && paint_layer_is_fragmented == &b.paint_layer_is_fragmented
        && paint_layer_known_to_clip_subtree_to_padding_box
            == &b.paint_layer_known_to_clip_subtree_to_padding_box
        && paint_layer_first_fragment_visual_rect == &b.paint_layer_first_fragment_visual_rect
        && paint_layer_contents_visual_rect == &b.paint_layer_contents_visual_rect
        && paint_layer_filter_reference_box == &b.paint_layer_filter_reference_box
        && paint_layer_filter_viewport == &b.paint_layer_filter_viewport
        && child_paint_blocked_by_display_lock == &b.child_paint_blocked_by_display_lock
        && paint_layer_is_stacking_context == &b.paint_layer_is_stacking_context
        && paint_layer_is_stacked == &b.paint_layer_is_stacked
        && paint_layer_stacking_level == &b.paint_layer_stacking_level
        && paint_layer_is_z_order_list_visible == &b.paint_layer_is_z_order_list_visible
        && paint_layer_is_in_top_or_view_transition_layer
            == &b.paint_layer_is_in_top_or_view_transition_layer
        && paint_layer_is_direct_layout_view_child == &b.paint_layer_is_direct_layout_view_child
        && paint_layer_can_contain_absolute_position_objects
            == &b.paint_layer_can_contain_absolute_position_objects
        && paint_layer_can_contain_fixed_position_objects
            == &b.paint_layer_can_contain_fixed_position_objects
        && paint_layer_has_overlay_overflow_controls == &b.paint_layer_has_overlay_overflow_controls
        && paint_layer_order_parent_unique_id == &b.paint_layer_order_parent_unique_id
        && paint_layer_order == &b.paint_layer_order
        && display_item_client_is_cacheable == &b.display_item_client_is_cacheable
        && display_item_client_is_just_created == &b.display_item_client_is_just_created
        && selection_client_id == &b.selection_client_id
        && selection_client_invalidation_reason == &b.selection_client_invalidation_reason
        && selection_client_is_cacheable == &b.selection_client_is_cacheable
        && selection_client_is_just_created == &b.selection_client_is_just_created
        && display_item_fragment == &b.display_item_fragment
        && has_source == &b.has_source
        && logical_parent_node_id == &b.logical_parent_node_id
        && logical_parent_fragment_instance_id == &b.logical_parent_fragment_instance_id
        && logical_tree_order == &b.logical_tree_order
        && establishes_paint_state == &b.establishes_paint_state
        && hidden == &b.hidden
        && is_line_break == &b.is_line_break
        && svg_hidden_container == &b.svg_hidden_container
        && svg_descendant_may_have_transform_related_operations
            == &b.svg_descendant_may_have_transform_related_operations
        && svg_view_box_is_empty == &b.svg_view_box_is_empty
        && painted_atomically == &b.painted_atomically
        && has_collapsed_borders == &b.has_collapsed_borders
        && source_kind == &b.source_kind
        && replaced_is_image == &b.replaced_is_image
        && image_decoding_mode == &b.image_decoding_mode
        && focused_image_map_area == &b.focused_image_map_area
        && effective_zoom == &b.effective_zoom
        && style == &b.style
        && selection_style == &b.selection_style
        && selection_text_decoration == &b.selection_text_decoration
        && margin == &b.margin
        && border == &b.border
        && border_sides == &b.border_sides
        && border_styles == &b.border_styles
        && padding == &b.padding
        && stitched_decoration == &b.stitched_decoration
        && box_decoration_break == &b.box_decoration_break
        && display == &b.display
        && position == &b.position
        && fixed_to_view == &b.fixed_to_view
        && needs_anchor_position_scroll_adjustment == &b.needs_anchor_position_scroll_adjustment
        && floating == &b.floating
        && sticky_offset == &b.sticky_offset
        && overflow_x == &b.overflow_x
        && overflow_y == &b.overflow_y
        && overscroll_behavior_x == &b.overscroll_behavior_x
        && overscroll_behavior_y == &b.overscroll_behavior_y
        && prevent_scroll_axis_locking == &b.prevent_scroll_axis_locking
        && css_clip == &b.css_clip
        && overflow_clip_margin_outsets == &b.overflow_clip_margin_outsets
        && scroll_offset == &b.scroll_offset
        && hit_test_overflow_clip == &b.hit_test_overflow_clip
        && hit_test_scrolled_content_offset == &b.hit_test_scrolled_content_offset
        && scroll_size == &b.scroll_size
        && scrolls_overflow == &b.scrolls_overflow
        && scroll_container == &b.scroll_container
        && writing_mode == &b.writing_mode
        && direction == &b.direction
        && first_baseline == &b.first_baseline
        && text_line_top_offset == &b.text_line_top_offset
        && text_line_height == &b.text_line_height
        && is_ellipsis == &b.is_ellipsis
        && text_fit_scale == &b.text_fit_scale
        && text_combine == &b.text_combine
        && text_combine_inline_offset == &b.text_combine_inline_offset
        && replaced_content == &b.replaced_content
        && glyph_runs == &b.glyph_runs
        && text_control_host == &b.text_control_host
        && text_control_inner_editor == &b.text_control_inner_editor
        && text_control_empty_caret == &b.text_control_empty_caret
        && list_marker_symbol == &b.list_marker_symbol
        && list_marker_inside == &b.list_marker_inside
        && mathml == &b.mathml
        && frame_set == &b.frame_set
        && fieldset == &b.fieldset
        && column_rules == &b.column_rules
        && collapsed_table == &b.collapsed_table
        && table == &b.table
        && scrollbars == &b.scrollbars
        && form_control == &b.form_control
        && text_control_caret_metrics == &b.text_control_caret_metrics
        && svg_shape == &b.svg_shape
        && svg_view_box == &b.svg_view_box
        && svg_local_transform == &b.svg_local_transform
        && svg_viewport == &b.svg_viewport
        && svg_visual_rect == &b.svg_visual_rect
        && svg_text == &b.svg_text
        && svg_text_decoration_style == &b.svg_text_decoration_style
        && table_cell_column == &b.table_cell_column
        && sticky_constraint == &b.sticky_constraint
        && background_transfers_to_view == &b.background_transfers_to_view
        && canvas_background == &b.canvas_background
        && view_background_invalidation == &b.view_background_invalidation
        && background_paint_location == &b.background_paint_location
        && background_fill_layer_count == &b.background_fill_layer_count
        && has_background == &b.has_background
        && background_should_always_be_clipped == &b.background_should_always_be_clipped
        && background_any_local_attachment_image == &b.background_any_local_attachment_image
        && background_any_default_attachment_image == &b.background_any_default_attachment_image
        && background_any_layer_uses_content_box == &b.background_any_layer_uses_content_box
        && scrollable_overflow == &b.scrollable_overflow
        && view_background_rect == &b.view_background_rect
        && can_composite_background_attachment_fixed == &b.can_composite_background_attachment_fixed
}

fn self_equal(a: &FragmentPaintData, b: &FragmentPaintData) -> bool {
    identity_equal(a, b) && geometry_equal(a, b) && paint_equal(&a.paint, &b.paint)
}

fn identity_equal(a: &FragmentPaintData, b: &FragmentPaintData) -> bool {
    a.fragment_instance_id == b.fragment_instance_id
        && a.fragmentainer_instance_id == b.fragmentainer_instance_id
        && a.node_id == b.node_id
        && a.kind == b.kind
        && a.text_start == b.text_start
        && a.text_end == b.text_end
}

fn geometry_equal(a: &FragmentPaintData, b: &FragmentPaintData) -> bool {
    a.offset == b.offset && a.size == b.size && a.content_size == b.content_size
}

fn just_created(paint: &PaintProperties) -> bool {
    paint.display_item_client_is_just_created.get()
        || paint.selection_client_is_just_created.get()
        || paint.paint_layer_client_is_just_created.get()
        || paint
            .scroll_container
            .as_ref()
            .is_some_and(|scroll| scroll.scrolling_background_client_is_just_created.get())
        || paint.scrollbars.as_ref().is_some_and(|bars| {
            bars.corner_client_is_just_created.get()
                || bars
                    .horizontal
                    .as_ref()
                    .is_some_and(|axis| axis.display_item_client_is_just_created.get())
                || bars
                    .vertical
                    .as_ref()
                    .is_some_and(|axis| axis.display_item_client_is_just_created.get())
        })
}

fn resources_equal(
    a: &Option<Arc<PaintResources>>,
    b: &Option<Arc<PaintResources>>,
    matches: &mut HashMap<(usize, usize, bool), bool>,
    compare_viewport: bool,
) -> bool {
    let (a, b) = match (a, b) {
        (None, None) => return true,
        (Some(a), Some(b)) => (a, b),
        _ => return false,
    };
    if Arc::ptr_eq(a, b) {
        return true;
    }
    let key = (
        Arc::as_ptr(a) as usize,
        Arc::as_ptr(b) as usize,
        compare_viewport,
    );
    *matches.entry(key).or_insert_with(|| {
        // These are immutable resource-provider identities, not the catalog
        // allocation rebuilt by every export. New storage conservatively dirties.
        let PaintResources {
            fonts,
            images,
            device_pixel_ratio,
            preferred_color_scheme,
            layout_zoom_factor,
            paints_opaque_base_background,
            viewport,
        } = &**a;
        *device_pixel_ratio == b.device_pixel_ratio
            && *preferred_color_scheme == b.preferred_color_scheme
            && *layout_zoom_factor == b.layout_zoom_factor
            && *paints_opaque_base_background == b.paints_opaque_base_background
            && (!compare_viewport || *viewport == b.viewport)
            && fonts.len() == b.fonts.len()
            && images.len() == b.images.len()
            && (fonts.ptr_eq(&b.fonts)
                || fonts.iter().zip(b.fonts.iter()).all(|(a, b)| {
                    font_engine::SharedFontBytes::ptr_eq(&a.bytes, &b.bytes) && a == b
                }))
            && images.iter().zip(&b.images).all(|(a, b)| a == b)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fragment(instance: u64) -> FragmentPaintData {
        let mut fragment = FragmentPaintData::default();
        fragment.fragment_instance_id = instance;
        fragment
    }

    #[test]
    fn logical_parent_occurrence_changes_source_revision() {
        let mut tracker = PrePaintRevisionTracker::new();
        let mut fragment = fragment(1);
        tracker.update_flat(&mut [FlatPrePaintInput {
            node: &mut fragment,
            parent: None,
        }]);
        let first = fragment.pre_paint_revision;

        fragment.paint.logical_parent_node_id = Some(7);
        fragment.paint.logical_parent_fragment_instance_id = Some(9);
        tracker.update_flat(&mut [FlatPrePaintInput {
            node: &mut fragment,
            parent: None,
        }]);

        assert_ne!(fragment.pre_paint_revision, first);
    }

    #[test]
    fn scrolls_overflow_changes_source_revision() {
        let first = fragment(1);
        let mut second = fragment(1);
        assert!(SamePrePaintInput(&first, &second));
        second.paint.scrolls_overflow = true;
        assert!(!SamePrePaintInput(&first, &second));
    }

    #[test]
    fn ancestor_position_visibility_changes_source_revision() {
        let first = fragment(1);
        let mut second = fragment(1);
        assert!(SamePrePaintInput(&first, &second));
        second
            .paint
            .paint_layer_has_ancestor_invisible_for_position_visibility = true;
        assert!(!SamePrePaintInput(&first, &second));
    }

    #[test]
    fn filter_reference_box_changes_source_revision() {
        let first = fragment(1);
        let mut second = fragment(1);
        assert!(SamePrePaintInput(&first, &second));
        second.paint.paint_layer_filter_reference_box = Some(FragmentPaintRect::default());
        assert!(!SamePrePaintInput(&first, &second));
    }

    #[test]
    fn filter_viewport_changes_source_revision() {
        let first = fragment(1);
        let mut second = fragment(1);
        assert!(SamePrePaintInput(&first, &second));
        second.paint.paint_layer_filter_viewport = Some(Size {
            width: 40.0,
            height: 50.0,
        });
        assert!(!SamePrePaintInput(&first, &second));
    }

    #[test]
    fn fragment_occurrence_policy_is_input_and_part_of_revision() {
        let first = fragment(1);
        let mut repeated = fragment(1);
        repeated.paint.is_first_for_node = false;
        repeated.paint.can_paint_multiple_fragments = false;
        assert!(!repeated.paint.CanPaintFragmentOccurrence());
        assert!(!SamePrePaintInput(&first, &repeated));

        repeated.paint.can_paint_multiple_fragments = true;
        assert!(repeated.paint.CanPaintFragmentOccurrence());
    }

    #[test]
    fn child_identity_order_changes_subtree_revision() {
        let mut tracker = PrePaintRevisionTracker::new();
        let mut root = fragment(1);
        let mut first_child = fragment(2);
        let mut second_child = fragment(3);
        tracker.update_flat(&mut [
            FlatPrePaintInput {
                node: &mut root,
                parent: None,
            },
            FlatPrePaintInput {
                node: &mut first_child,
                parent: Some(0),
            },
            FlatPrePaintInput {
                node: &mut second_child,
                parent: Some(0),
            },
        ]);
        let root_subtree = root.pre_paint_subtree_revision;
        let first_revision = first_child.pre_paint_revision;
        let second_revision = second_child.pre_paint_revision;

        tracker.update_flat(&mut [
            FlatPrePaintInput {
                node: &mut root,
                parent: None,
            },
            FlatPrePaintInput {
                node: &mut second_child,
                parent: Some(0),
            },
            FlatPrePaintInput {
                node: &mut first_child,
                parent: Some(0),
            },
        ]);

        assert_eq!(first_child.pre_paint_revision, first_revision);
        assert_eq!(second_child.pre_paint_revision, second_revision);
        assert_ne!(root.pre_paint_subtree_revision, root_subtree);
    }

    #[test]
    fn committed_fallback_clients_clear_invalidation_reasons() {
        let mut tracker = PrePaintRevisionTracker::new();
        let mut fragment = FragmentPaintData::default();
        fragment.paint.display_item_client_id = 11;
        fragment.paint.display_item_client_is_cacheable = true;
        fragment.paint.display_item_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kStyle);
        fragment.paint.selection_client_id = 12;
        fragment.paint.selection_client_is_cacheable = true;
        fragment.paint.selection_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kSelection);

        tracker.update_flat(&mut [FlatPrePaintInput {
            node: &mut fragment,
            parent: None,
        }]);
        tracker.CommitClients([11, 12]);

        let paint = tracker
            .inputs
            .values()
            .next()
            .unwrap()
            .fallback_paint
            .as_ref()
            .unwrap();
        assert_eq!(
            paint.display_item_client_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kNone)
        );
        assert_eq!(
            paint.selection_client_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kNone)
        );
    }
}
