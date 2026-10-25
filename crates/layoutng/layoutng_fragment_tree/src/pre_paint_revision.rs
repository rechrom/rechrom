//! Rust layout-output invalidation observer. These revisions describe exported
//! inputs, not Blink paint properties or native dirty flags. They supply the
//! object/descendant change distinction used by PrePaintTreeWalk (upstream
//! core/paint/pre_paint_tree_walk.cc:418-436) at this owned-output boundary.

use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use super::{FragmentNode, PaintProperties, PaintResources};
use foundation::{IsManagedLayoutAddress, WeakPersistent};

// Across engines and failed passes, certified revisions must never alias.
static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

fn next_revision() -> u64 {
    NEXT_REVISION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |revision| {
            revision.checked_add(1)
        })
        .expect("pre-paint revision exhausted")
}

/// Version an in-place native scroll/sticky/control change on an already
/// certified layout output. The layout boundary must call this on every
/// changed node and its complete ancestor path. Zero remains unobserved;
/// this operation cannot certify external or previously unobserved inputs.
pub fn MarkScrollPropertyChange(node: &mut FragmentNode, own_changed: bool) {
    let generation = next_revision();
    if own_changed && node.pre_paint_revision != 0 {
        node.pre_paint_revision = generation;
    }
    if node.pre_paint_revision != 0 && node.pre_paint_subtree_revision != 0 {
        node.pre_paint_subtree_revision = generation;
    } else {
        node.pre_paint_subtree_revision = 0;
    }
}

#[derive(Default)]
pub struct PrePaintRevisionTracker {
    // Move stable observation handles between passes, not complete fragments.
    inputs: HashMap<InputKey, Box<Input>>,
    spare_inputs: HashMap<InputKey, Box<Input>>,
    owners: HashMap<u64, LiveOwner>,
    spare_owners: HashMap<u64, LiveOwner>,
}

#[derive(Hash, PartialEq, Eq)]
enum InputKey {
    // Production exports preserve the native FragmentData-equivalent identity
    // across cull projections. Child indices may shift as an offscreen sibling
    // enters or leaves the projection, so they are not object identity.
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
    // Children are deliberately absent: no second retained fragment tree and
    // no Rc ownership that would force copy-on-write in scroll updates.
    fragment: FragmentNode,
    resources: Option<Arc<PaintResources>>,
    children: Vec<(u64, u64)>,
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
            let paint = &mut input.fragment.paint;
            if committed(
                paint.display_item_client_id,
                paint.display_item_client_is_cacheable,
                paint.display_item_client_is_just_created,
            ) {
                paint.display_item_client_is_just_created = false;
            }
            if committed(
                paint.paint_layer_client_id,
                paint.paint_layer_client_is_cacheable,
                paint.paint_layer_client_is_just_created,
            ) {
                paint.paint_layer_client_is_just_created = false;
            }
            if let Some(bars) = &mut paint.scrollbars {
                let changes = committed(
                    bars.corner_client_id,
                    bars.corner_client_is_cacheable,
                    bars.corner_client_is_just_created,
                ) || bars.horizontal.as_ref().is_some_and(|axis| {
                    committed(
                        axis.display_item_client_id,
                        axis.display_item_client_is_cacheable,
                        axis.display_item_client_is_just_created,
                    )
                }) || bars.vertical.as_ref().is_some_and(|axis| {
                    committed(
                        axis.display_item_client_id,
                        axis.display_item_client_is_cacheable,
                        axis.display_item_client_is_just_created,
                    )
                });
                if changes {
                    // Detach only a scrollbar payload whose actual used-client
                    // flags changed. Other shared paint payloads are untouched.
                    let bars = Arc::make_mut(bars);
                    if committed(
                        bars.corner_client_id,
                        bars.corner_client_is_cacheable,
                        bars.corner_client_is_just_created,
                    ) {
                        bars.corner_client_is_just_created = false;
                    }
                    for axis in [&mut bars.horizontal, &mut bars.vertical]
                        .into_iter()
                        .flatten()
                    {
                        if committed(
                            axis.display_item_client_id,
                            axis.display_item_client_is_cacheable,
                            axis.display_item_client_is_just_created,
                        ) {
                            axis.display_item_client_is_just_created = false;
                        }
                    }
                }
            }
        }
    }

    /// Certify a complete freshly exported tree. Only the layout boundary may
    /// make this assertion; mutable external inputs default to revision zero.
    pub fn update(&mut self, root: &mut FragmentNode) {
        let generation = next_revision();
        let mut next = std::mem::take(&mut self.spare_inputs);
        debug_assert!(next.is_empty());
        next.reserve(self.inputs.len());
        let mut resource_matches = HashMap::new();
        let mut previous = std::mem::take(&mut self.inputs);
        let mut previous_owners = std::mem::take(&mut self.owners);
        let mut next_owners = std::mem::take(&mut self.spare_owners);
        debug_assert!(next_owners.is_empty());
        next_owners.reserve(previous_owners.len());
        Self::visit(
            root,
            &mut Vec::new(),
            generation,
            &mut previous,
            &mut next,
            &mut resource_matches,
            &mut previous_owners,
            &mut next_owners,
        );
        self.inputs = next;
        self.owners = next_owners;
        // Removed fragments/clients retire now. Keep only empty bucket arrays;
        // no previous observation or weak-owner proof survives in the spares.
        previous.clear();
        previous_owners.clear();
        self.spare_inputs = previous;
        self.spare_owners = previous_owners;
    }

    fn visit(
        node: &mut FragmentNode,
        path: &mut Vec<usize>,
        generation: u64,
        previous: &mut HashMap<InputKey, Box<Input>>,
        next: &mut HashMap<InputKey, Box<Input>>,
        resource_matches: &mut HashMap<(usize, usize, bool), bool>,
        previous_owners: &mut HashMap<u64, LiveOwner>,
        next_owners: &mut HashMap<u64, LiveOwner>,
    ) {
        for (index, child) in node.children.iter_mut().enumerate() {
            path.push(index);
            Self::visit(
                child,
                path,
                generation,
                previous,
                next,
                resource_matches,
                previous_owners,
                next_owners,
            );
            path.pop();
        }
        let lookup_key = if node.fragment_instance_id == 0 {
            InputKey::Path(path.clone())
        } else {
            InputKey::Instance(node.fragment_instance_id)
        };
        let (key, old) = match previous.remove_entry(&lookup_key) {
            Some((key, old)) => (key, Some(old)),
            None => (lookup_key, None),
        };
        let owners_live = retain_owners(&node.paint, previous_owners, next_owners);
        node.pre_paint_owner_retained = owners_live
            && old
                .as_ref()
                .is_some_and(|old| owner_ids(&old.fragment.paint) == owner_ids(&node.paint));
        let self_same = old.as_ref().is_some_and(|old| {
            (!just_created(&node.paint) || node.pre_paint_owner_retained)
                && self_equal(&old.fragment, node)
                && resources_equal(
                    &old.resources,
                    &node.paint.resources,
                    resource_matches,
                    paint_uses_viewport_resource(&old.fragment.paint)
                        || paint_uses_viewport_resource(&node.paint),
                )
        });
        let children_same = old.as_ref().is_some_and(|old| {
            old.children.iter().copied().eq(node
                .children
                .iter()
                .map(|child| (child.pre_paint_revision, child.pre_paint_subtree_revision)))
        });
        node.pre_paint_revision = if self_same {
            old.as_ref().unwrap().fragment.pre_paint_revision
        } else {
            generation
        };
        node.pre_paint_subtree_revision = if self_same && children_same {
            old.as_ref().unwrap().fragment.pre_paint_subtree_revision
        } else {
            generation
        };
        // Keep unchanged shallow payloads, including glyph vectors, rather than
        // cloning them on every layout. No fragment tree ownership is retained.
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
            current.children.extend(
                node.children
                    .iter()
                    .map(|child| (child.pre_paint_revision, child.pre_paint_subtree_revision)),
            );
        }
        current.fragment.pre_paint_revision = node.pre_paint_revision;
        current.fragment.pre_paint_subtree_revision = node.pre_paint_subtree_revision;
        current.fragment.pre_paint_owner_retained = node.pre_paint_owner_retained;
        next.insert(key, current);
    }
}

impl Input {
    fn from_fragment(node: &FragmentNode) -> Self {
        // Exhaustive pattern: a new exported self field cannot accidentally be
        // omitted from the observer without a compiler error here.
        let FragmentNode {
            // CSSOM-only inline rectangles do not affect paint recording.
            inline_geometry: _,
            // Cull hysteresis belongs to the projection lifecycle, not this
            // fragment's PrePaint input.
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
            paint,
            children: _,
        } = node;
        let mut paint = paint.clone();
        let resources = paint.resources.take();
        Self {
            fragment: FragmentNode {
                inline_geometry: Vec::new(),
                paint_cull_scroll_offset: None,
                pre_paint_revision: 0,
                pre_paint_subtree_revision: 0,
                pre_paint_owner_retained: false,
                fragment_instance_id: *fragment_instance_id,
                fragmentainer_instance_id: *fragmentainer_instance_id,
                node_id: *node_id,
                kind: *kind,
                offset: *offset,
                size: *size,
                content_size: *content_size,
                text_start: *text_start,
                text_end: *text_end,
                paint,
                children: Vec::new(),
            },
            resources,
            children: Vec::new(),
        }
    }
}

/// Compare all exported self inputs, excluding children and certification
/// revisions/lifecycle metadata. Just-created clients need the boundary's
/// certified live-owner proof. This does not consume native dirty flags.
#[allow(non_snake_case)]
pub fn SamePrePaintInput(a: &FragmentNode, b: &FragmentNode) -> bool {
    (!just_created(&b.paint) || (b.pre_paint_revision != 0 && b.pre_paint_owner_retained))
        && self_equal(a, b)
        && resources_equal(
            &a.paint.resources,
            &b.paint.resources,
            &mut HashMap::new(),
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
        && (!paint.paint_layer_client_is_just_created || paint.paint_layer_client_id != 0);
    if let Some(bars) = &paint.scrollbars {
        retained &= (!bars.corner_client_is_just_created || bars.corner_client_id != 0)
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
        text_caret_positions,
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
        && text_caret_positions == &b.text_caret_positions
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

fn self_equal(a: &FragmentNode, b: &FragmentNode) -> bool {
    a.fragment_instance_id == b.fragment_instance_id
        && a.fragmentainer_instance_id == b.fragmentainer_instance_id
        && a.node_id == b.node_id
        && a.kind == b.kind
        && a.offset == b.offset
        && a.size == b.size
        && a.content_size == b.content_size
        && a.text_start == b.text_start
        && a.text_end == b.text_end
        && paint_equal(&a.paint, &b.paint)
}

fn just_created(paint: &PaintProperties) -> bool {
    paint.display_item_client_is_just_created
        || paint.paint_layer_client_is_just_created
        || paint.scrollbars.as_ref().is_some_and(|bars| {
            bars.corner_client_is_just_created
                || bars
                    .horizontal
                    .as_ref()
                    .is_some_and(|axis| axis.display_item_client_is_just_created)
                || bars
                    .vertical
                    .as_ref()
                    .is_some_and(|axis| axis.display_item_client_is_just_created)
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

    #[test]
    fn hidden_svg_container_kind_changes_pre_paint_revision() {
        let mut fragment = FragmentNode::default();
        let mut tracker = PrePaintRevisionTracker::new();
        tracker.update(&mut fragment);
        let visible = fragment.clone();
        fragment.paint.svg_hidden_container = true;
        assert!(!SamePrePaintInput(&visible, &fragment));
        tracker.update(&mut fragment);
        assert_ne!(fragment.pre_paint_revision, visible.pre_paint_revision);
        assert_ne!(
            fragment.pre_paint_subtree_revision,
            visible.pre_paint_subtree_revision
        );
    }

    #[test]
    fn viewport_resource_change_only_dirties_viewport_dependent_fragments() {
        let resources = |width| {
            let mut resources = PaintResources::default();
            resources.viewport = Some(layoutng::internal::layout_input::ViewportGeometry {
                size: layoutng::internal::layout_input_types::IntSize { width, height: 600 },
                ..Default::default()
            });
            Arc::new(resources)
        };
        let mut ordinary = FragmentNode::default();
        ordinary.paint.resources = Some(resources(800));
        let mut resized = ordinary.clone();
        resized.paint.resources = Some(resources(700));
        assert!(SamePrePaintInput(&ordinary, &resized));

        ordinary.paint.is_layout_view = true;
        resized.paint.is_layout_view = true;
        assert!(!SamePrePaintInput(&ordinary, &resized));

        ordinary.paint.is_layout_view = false;
        resized.paint.is_layout_view = false;
        ordinary.paint.style.background_images.push(
            layoutng::internal::paint_input::BackgroundImageLayer {
                attachment: layoutng::internal::paint_input::BackgroundAttachment::kFixed,
                ..Default::default()
            },
        );
        resized.paint.style = ordinary.paint.style.clone();
        assert!(!SamePrePaintInput(&ordinary, &resized));
    }

    #[test]
    fn revisions_observe_complete_input_and_order_without_retaining_children() {
        let mut root = FragmentNode::default();
        let mut resources = PaintResources::default();
        resources
            .images
            .push(layoutng::internal::layout_input::PaintImage {
                id: 7,
                revision: 1,
                width: 1,
                height: 1,
                content: image_resource::PaintImageContent::Bitmap(Arc::new(vec![1, 2, 3, 255])),
                ..Default::default()
            });
        root.paint.resources = Some(Arc::new(resources));
        root.children = vec![
            FragmentNode {
                node_id: 1,
                ..Default::default()
            },
            FragmentNode {
                node_id: 2,
                ..Default::default()
            },
        ];
        assert_eq!(root.pre_paint_revision, 0);
        let mut tracker = PrePaintRevisionTracker::new();
        tracker.update(&mut root);
        let self_revision = root.pre_paint_revision;
        let subtree_revision = root.pre_paint_subtree_revision;
        // Recreated catalog shell and releasing engine.fragments are irrelevant.
        let mut next = root.clone();
        next.paint.resources = Some(Arc::new((**root.paint.resources.as_ref().unwrap()).clone()));
        assert!(SamePrePaintInput(&root, &next));
        let mut other_engine = PrePaintRevisionTracker::new();
        let mut other = next.clone();
        other_engine.update(&mut other);
        assert_ne!(other.pre_paint_revision, self_revision);
        drop(root);
        tracker.update(&mut next);
        assert_eq!(next.pre_paint_revision, self_revision);
        assert_eq!(next.pre_paint_subtree_revision, subtree_revision);
        let mut resources = (**next.paint.resources.as_ref().unwrap()).clone();
        resources.images[0].revision = 2;
        resources.images[0].content =
            image_resource::PaintImageContent::Bitmap(Arc::new(vec![1, 2, 3, 255]));
        let old = next.clone();
        next.paint.resources = Some(Arc::new(resources));
        assert!(!SamePrePaintInput(&old, &next));
        tracker.update(&mut next);
        assert_ne!(next.pre_paint_revision, self_revision);
        let self_revision = next.pre_paint_revision;
        let subtree_revision = next.pre_paint_subtree_revision;
        next.children[0].paint.sticky_offset.x = 0.5;
        tracker.update(&mut next);
        assert_eq!(next.pre_paint_revision, self_revision);
        assert_ne!(next.pre_paint_subtree_revision, subtree_revision);
        let changed = next.pre_paint_subtree_revision;
        next.children.swap(0, 1);
        tracker.update(&mut next);
        assert_ne!(next.pre_paint_subtree_revision, changed);
        let changed = next.pre_paint_subtree_revision;
        next.children.pop();
        tracker.update(&mut next);
        assert_ne!(next.pre_paint_subtree_revision, changed);
        next.paint.display_item_client_is_just_created = true;
        tracker.update(&mut next);
        let created = next.pre_paint_revision;
        tracker.update(&mut next);
        assert_ne!(next.pre_paint_revision, created);
        next.paint.display_item_client_is_just_created = false;
        let changed = next.pre_paint_revision;
        tracker.invalidate();
        tracker.update(&mut next);
        assert_ne!(next.pre_paint_revision, changed);
        assert!(tracker
            .inputs
            .values()
            .all(|input| input.fragment.children.is_empty()));

        // Real native-heap liveness, including a distinct paint layer. The
        // tracker's weak handles must neither retain either allocation nor
        // multiply with the number of fragments of one native owner.
        let mut managed = FragmentNode::default();
        let mut owner;
        let mut layer;
        let mut offscreen_owner;
        {
            let _scope = foundation::LayoutHeapScope::new();
            owner = foundation::Persistent::from_ptr(foundation::MakeGarbageCollected(11u32));
            layer = foundation::Persistent::from_ptr(foundation::MakeGarbageCollected(22u32));
            offscreen_owner =
                foundation::Persistent::from_ptr(foundation::MakeGarbageCollected(44u32));
            managed.paint.display_item_client_id = owner.Get() as usize as u64;
            managed.paint.paint_layer_client_id = layer.Get() as usize as u64;
            managed.paint.display_item_client_is_just_created = true;
            managed.paint.paint_layer_client_is_just_created = true;
            managed.paint.display_item_client_is_cacheable = true;
            managed.paint.paint_layer_client_is_cacheable = true;
            managed.paint.scrollbars = Some(Arc::new(super::super::ScrollbarPaintData {
                corner_client_id: managed.paint.paint_layer_client_id,
                corner_client_is_cacheable: true,
                corner_client_is_just_created: true,
                vertical: Some(super::super::ScrollbarPaintAxis {
                    display_item_client_id: managed.paint.display_item_client_id,
                    display_item_client_is_cacheable: true,
                    display_item_client_is_just_created: true,
                    ..Default::default()
                }),
                ..Default::default()
            }));
            managed.children = vec![managed.clone(), managed.clone()];
            let mut offscreen = FragmentNode::default();
            offscreen.paint.display_item_client_id = offscreen_owner.Get() as usize as u64;
            offscreen.paint.display_item_client_is_cacheable = true;
            offscreen.paint.display_item_client_is_just_created = true;
            managed.children.push(offscreen);
            tracker.invalidate();
            tracker.update(&mut managed);
            assert!(!managed.pre_paint_owner_retained);
            let first = managed.pre_paint_revision;
            tracker.update(&mut managed);
            assert!(managed.pre_paint_owner_retained);
            assert!(managed
                .children
                .iter()
                .all(|child| child.pre_paint_owner_retained));
            assert_eq!(managed.pre_paint_revision, first);
            assert_eq!(tracker.owners.len(), 3);
            assert!(SamePrePaintInput(&managed, &managed));

            let offscreen_revision = managed.children[2].pre_paint_revision;
            tracker.CommitClients([
                managed.paint.display_item_client_id,
                managed.paint.paint_layer_client_id,
            ]);
            // Simulate the next native export after actual PaintController
            // validation. Unused offscreen clients are never validated.
            managed.paint.display_item_client_is_just_created = false;
            managed.paint.paint_layer_client_is_just_created = false;
            for node in std::iter::once(&mut managed.paint).chain(
                managed.children[..2]
                    .iter_mut()
                    .map(|child| &mut child.paint),
            ) {
                node.display_item_client_is_just_created = false;
                node.paint_layer_client_is_just_created = false;
                let bars = Arc::make_mut(node.scrollbars.as_mut().unwrap());
                bars.corner_client_is_just_created = false;
                bars.vertical
                    .as_mut()
                    .unwrap()
                    .display_item_client_is_just_created = false;
            }
            tracker.update(&mut managed);
            assert_eq!(managed.pre_paint_revision, first);
            assert_eq!(managed.children[2].pre_paint_revision, offscreen_revision);
            assert!(
                managed.children[2]
                    .paint
                    .display_item_client_is_just_created
            );
            assert!(managed.children[2].pre_paint_owner_retained);
        }
        let old_layer = managed.paint.paint_layer_client_id;
        {
            let _scope = foundation::LayoutHeapScope::new();
            layer.Clear();
        }
        assert!(tracker.owners[&old_layer].weak.Get().is_null());
        // A newly created native layer starts JustCreated again, including
        // after a prior layer had been validated by PaintController.
        managed.paint.paint_layer_client_is_just_created = true;
        for child in &mut managed.children[..2] {
            child.paint.paint_layer_client_is_just_created = true;
        }
        let before_reclaim = managed.pre_paint_revision;
        tracker.update(&mut managed);
        assert!(!managed.pre_paint_owner_retained);
        assert_ne!(managed.pre_paint_revision, before_reclaim);
        {
            let _scope = foundation::LayoutHeapScope::new();
            layer = foundation::Persistent::from_ptr(foundation::MakeGarbageCollected(33u32));
            managed.paint.paint_layer_client_id = layer.Get() as usize as u64;
            for child in &mut managed.children {
                if child.paint.paint_layer_client_id == old_layer {
                    child.paint.paint_layer_client_id = managed.paint.paint_layer_client_id;
                }
            }
            for paint in std::iter::once(&mut managed.paint).chain(
                managed.children[..2]
                    .iter_mut()
                    .map(|child| &mut child.paint),
            ) {
                let bars = Arc::make_mut(paint.scrollbars.as_mut().unwrap());
                bars.corner_client_id = paint.paint_layer_client_id;
                bars.corner_client_is_just_created = true;
            }
            tracker.update(&mut managed);
            // This also covers the allocator reusing the reclaimed address:
            // the old cell was cleared, so the new object gets no old proof.
            assert!(!managed.pre_paint_owner_retained);
            tracker.update(&mut managed);
            assert!(managed.pre_paint_owner_retained);
        }
        {
            let _scope = foundation::LayoutHeapScope::new();
            owner.Clear();
            layer.Clear();
            offscreen_owner.Clear();
        }
        assert!(tracker
            .owners
            .values()
            .all(|owner| owner.weak.Get().is_null()));
    }
}
