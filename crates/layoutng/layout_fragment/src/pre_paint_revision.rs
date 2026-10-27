//! Rust layout-output invalidation observer. Production revisions consume the
//! native LayoutObject dirty flags and add fragment-local geometry/resource
//! changes at the owned-output boundary. Hand-built fragments without native
//! identity retain an exhaustive value-comparison fallback. The resulting
//! object/descendant distinction feeds PrePaintTreeWalk (upstream
//! core/paint/pre_paint_tree_walk.cc:418-436).

use std::sync::Arc;

use super::{FragmentKind, FragmentPaintData, PaintProperties, PaintResources};
use foundation::{
    IsManagedLayoutAddress, LayoutOffset as Offset, LayoutSize as Size,
    TrustedIdentityHashMap as HashMap, TrustedIdentityHashSet as HashSet, WeakPersistent,
};
pub use layout_fragment_boundary::revision::PrePaintRevisionStats;

/// Version an in-place native scroll/sticky/control change on an already
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
}

/// A non-owning view of Chromium's native fragment topology. The pointed
/// `FragmentPaintData` is only the fragment-local Paint data attached to a
/// PhysicalFragment or FragmentItem; `parent` describes the native link and
/// never creates a second owned tree.
#[derive(Clone, Copy)]
pub struct FlatPrePaintInput {
    pub node: *mut FragmentPaintData,
    pub parent: Option<usize>,
}

#[derive(Hash, PartialEq, Eq)]
enum InputKey {
    // Production updates preserve the native FragmentData-equivalent identity.
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
    // Production fragments retain only the values Chromium's PrePaint walk
    // needs after LayoutObject dirty bits have decided whether paint changed.
    // A complete PaintProperties snapshot is reserved for hand-built inputs
    // without a native fragment identity, where no dirty-bit contract exists.
    snapshot: InputSnapshot,
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
    owner_ids: [u64; 5],
    pre_paint_revision: u64,
    pre_paint_subtree_revision: u64,
    pre_paint_owner_retained: bool,
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
    }

    pub fn last_stats(&self) -> PrePaintRevisionStats {
        self.last_stats
    }

    /// Mirror the lifecycle validation performed by the real PaintController
    /// commit for its used cacheable clients. This only advances the observed
    /// lifecycle bits; it neither validates native clients nor changes paint
    /// geometry, revisions or weak-owner proofs on its own.
    #[allow(non_snake_case)]
    pub fn CommitClients(&mut self, clients: impl IntoIterator<Item = u64>) {
        let used: HashSet<_> = clients.into_iter().filter(|id| *id != 0).collect();
        if used.is_empty() {
            return;
        }
        let committed = |id, cacheable, created| cacheable && created && used.contains(&id);
        for input in self.inputs.values_mut() {
            let Some(paint) = input.fallback_paint.as_mut() else {
                // Native clients are validated and have their dirty/lifecycle
                // flags cleared by LayoutEngine's PaintController commit.
                continue;
            };
            if committed(
                paint.display_item_client_id,
                paint.display_item_client_is_cacheable,
                paint.display_item_client_is_just_created.get(),
            ) {
                paint.display_item_client_is_just_created.set(false);
            }
            if committed(
                paint.paint_layer_client_id,
                paint.paint_layer_client_is_cacheable,
                paint.paint_layer_client_is_just_created.get(),
            ) {
                paint.paint_layer_client_is_just_created.set(false);
            }
            if let Some(bars) = &mut paint.scrollbars {
                let changes = committed(
                    bars.corner_client_id,
                    bars.corner_client_is_cacheable,
                    bars.corner_client_is_just_created.get(),
                ) || bars.horizontal.as_ref().is_some_and(|axis| {
                    committed(
                        axis.display_item_client_id,
                        axis.display_item_client_is_cacheable,
                        axis.display_item_client_is_just_created.get(),
                    )
                }) || bars.vertical.as_ref().is_some_and(|axis| {
                    committed(
                        axis.display_item_client_id,
                        axis.display_item_client_is_cacheable,
                        axis.display_item_client_is_just_created.get(),
                    )
                });
                if changes {
                    // Detach only a scrollbar payload whose actual used-client
                    // flags changed. Other shared paint payloads are untouched.
                    let bars = Arc::make_mut(bars);
                    if committed(
                        bars.corner_client_id,
                        bars.corner_client_is_cacheable,
                        bars.corner_client_is_just_created.get(),
                    ) {
                        bars.corner_client_is_just_created.set(false);
                    }
                    for axis in [&mut bars.horizontal, &mut bars.vertical]
                        .into_iter()
                        .flatten()
                    {
                        if committed(
                            axis.display_item_client_id,
                            axis.display_item_client_is_cacheable,
                            axis.display_item_client_is_just_created.get(),
                        ) {
                            axis.display_item_client_is_just_created.set(false);
                        }
                    }
                }
            }
        }
    }

    /// Certify fragment-local Paint data by following a caller-provided flat
    /// view of the native PhysicalFragment/FragmentItems graph. Entries must be
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
                // Production native inputs always have an instance id. Keep a
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
                .is_some_and(|old| old.snapshot.owner_ids == owner_ids(&node.paint));
        let native_input = node.fragment_instance_id != 0;
        let (same_identity, same_geometry, same_paint, same_resources) = old
            .as_ref()
            .map(|old| {
                let identity = snapshot_identity_equal(&old.snapshot, node);
                let geometry = snapshot_geometry_equal(&old.snapshot, node);
                // Chromium's production PrePaint walk consumes LayoutObject
                // dirty bits. Deep comparison here duplicated that work for
                // every fragment-local input on every resize. External fragments do
                // not have that contract, so retain the exhaustive fallback.
                let paint = if native_input {
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
        let children_same = old
            .as_ref()
            .is_some_and(|old| old.children.as_slice() == child_revisions);
        stats.child_changes += (!children_same) as usize;
        node.pre_paint_revision = if self_same {
            old.as_ref().unwrap().snapshot.pre_paint_revision
        } else {
            generation
        };
        node.pre_paint_subtree_revision = if self_same && children_same {
            old.as_ref().unwrap().snapshot.pre_paint_subtree_revision
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
        current.snapshot.pre_paint_revision = node.pre_paint_revision;
        current.snapshot.pre_paint_subtree_revision = node.pre_paint_subtree_revision;
        current.snapshot.pre_paint_owner_retained = node.pre_paint_owner_retained;
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
            snapshot: InputSnapshot {
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
                owner_ids: owner_ids(&node.paint),
                pre_paint_revision: 0,
                pre_paint_subtree_revision: 0,
                pre_paint_owner_retained: false,
            },
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
/// certified live-owner proof. This does not consume native dirty flags.
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
    paint.is_layout_view
        || paint.canvas_background.is_some()
        || paint
            .style
            .background_images
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
        // Native lifecycle facts drive traversal; they are not paint input.
        pre_paint_invalidation: _,
        property_fragment_unique_id,
        is_layout_view,
        requires_main_thread_scrolling_for_background_attachment_fixed,
        display_item_client_invalidation_reason,
        // Source PaintArtifact records debug names only when actually painted;
        // changing this diagnostic name alone does not invalidate painting.
        display_item_client_debug_name: _,
        paint_layer_client_invalidation_reason,
        self_ink_overflow,
        contents_ink_overflow,
        ink_overflow,
        text_glyph_ink,
        box_self_visual_overflow,
        box_fragment_is_inline_box,
        display_item_raster_effect_outset,
        display_item_client_id,
        display_item_client_owner_node_id,
        paint_layer_state_is_native,
        paint_layer_client_id,
        paint_layer_client_is_cacheable,
        paint_layer_client_is_just_created,
        paint_layer_is_self_painting,
        paint_layer_is_stacking_context,
        paint_layer_stacking_level,
        display_item_client_is_cacheable,
        display_item_client_is_just_created,
        display_item_fragment,
        has_source,
        logical_parent_node_id,
        logical_parent_fragment_instance_id,
        logical_tree_order,
        establishes_paint_state,
        hidden,
        svg_hidden_container,
        painted_atomically,
        has_collapsed_borders,
        source_kind,
        effective_zoom,
        style,
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
        scroll_size,
        client_size: _,
        scroll_container,
        writing_mode,
        direction,
        first_baseline,
        text_line_top_offset,
        text_line_height,
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
        svg_text,
        table_cell_column,
        sticky_constraint,
        background_transfers_to_view,
        canvas_background,
        background_paint_location,
        can_composite_background_attachment_fixed,
    } = a;
    property_fragment_unique_id == &b.property_fragment_unique_id
        && is_layout_view == &b.is_layout_view
        && requires_main_thread_scrolling_for_background_attachment_fixed
            == &b.requires_main_thread_scrolling_for_background_attachment_fixed
        && display_item_client_invalidation_reason == &b.display_item_client_invalidation_reason
        && paint_layer_client_invalidation_reason == &b.paint_layer_client_invalidation_reason
        && self_ink_overflow == &b.self_ink_overflow
        && contents_ink_overflow == &b.contents_ink_overflow
        && ink_overflow == &b.ink_overflow
        && text_glyph_ink == &b.text_glyph_ink
        && box_self_visual_overflow == &b.box_self_visual_overflow
        && box_fragment_is_inline_box == &b.box_fragment_is_inline_box
        && display_item_raster_effect_outset == &b.display_item_raster_effect_outset
        && display_item_client_id == &b.display_item_client_id
        && display_item_client_owner_node_id == &b.display_item_client_owner_node_id
        && paint_layer_state_is_native == &b.paint_layer_state_is_native
        && paint_layer_client_id == &b.paint_layer_client_id
        && paint_layer_client_is_cacheable == &b.paint_layer_client_is_cacheable
        && paint_layer_client_is_just_created == &b.paint_layer_client_is_just_created
        && paint_layer_is_self_painting == &b.paint_layer_is_self_painting
        && paint_layer_is_stacking_context == &b.paint_layer_is_stacking_context
        && paint_layer_stacking_level == &b.paint_layer_stacking_level
        && display_item_client_is_cacheable == &b.display_item_client_is_cacheable
        && display_item_client_is_just_created == &b.display_item_client_is_just_created
        && display_item_fragment == &b.display_item_fragment
        && has_source == &b.has_source
        && logical_parent_node_id == &b.logical_parent_node_id
        && logical_parent_fragment_instance_id == &b.logical_parent_fragment_instance_id
        && logical_tree_order == &b.logical_tree_order
        && establishes_paint_state == &b.establishes_paint_state
        && hidden == &b.hidden
        && svg_hidden_container == &b.svg_hidden_container
        && painted_atomically == &b.painted_atomically
        && has_collapsed_borders == &b.has_collapsed_borders
        && source_kind == &b.source_kind
        && effective_zoom == &b.effective_zoom
        && style == &b.style
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
        && scroll_size == &b.scroll_size
        && scroll_container == &b.scroll_container
        && writing_mode == &b.writing_mode
        && direction == &b.direction
        && first_baseline == &b.first_baseline
        && text_line_top_offset == &b.text_line_top_offset
        && text_line_height == &b.text_line_height
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
        && svg_text == &b.svg_text
        && table_cell_column == &b.table_cell_column
        && sticky_constraint == &b.sticky_constraint
        && background_transfers_to_view == &b.background_transfers_to_view
        && canvas_background == &b.canvas_background
        && background_paint_location == &b.background_paint_location
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
        || paint.paint_layer_client_is_just_created.get()
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

    fn native_fragment(instance: u64) -> FragmentPaintData {
        let mut fragment = FragmentPaintData::default();
        fragment.fragment_instance_id = instance;
        fragment
    }

    #[test]
    fn logical_parent_occurrence_changes_native_revision() {
        let mut tracker = PrePaintRevisionTracker::new();
        let mut fragment = native_fragment(1);
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
    fn child_identity_order_changes_subtree_revision() {
        let mut tracker = PrePaintRevisionTracker::new();
        let mut root = native_fragment(1);
        let mut first_child = native_fragment(2);
        let mut second_child = native_fragment(3);
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
}
