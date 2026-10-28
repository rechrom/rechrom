//! Blink-style paint property references for the supported standalone input.
//!
//! Sources: platform/graphics/paint/property_tree_state.h and the transform,
//! clip, effect and scroll_paint_property_node.h files. These are paint trees,
//! not compositor layer trees. Numeric IDs are dump-local labels; reference
//! equality, as in Blink, determines whether states share property nodes.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

use crate::paint_clip_path::PaintClipPath;
use crate::property_filter_operations::PropertyFilterOperations;
use crate::PaintRect;
use layoutng_assembly::fragment_tree::StickyPositionPaintData;
pub use layoutng_assembly::internal::layout_input::OverscrollBehavior as NativeOverscrollBehavior;
use layoutng_assembly::internal::layout_input::{Offset, TransformMatrix};
pub use layoutng_assembly::internal::paint_input::PaintBlendMode;
use layoutng_assembly::internal::paint_input::PaintCornerRadii;

/// Source: paint_property_node.h. Composited-value downgrades are deliberately
/// absent from the standalone builder: it does not directly update cc nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PaintPropertyChangeType {
    #[default]
    Unchanged,
    ChangedOnlyCompositedValues,
    ChangedOnlySimpleValues,
    ChangedOnlyValues,
    NodeAddedOrRemoved,
}

/// A separately owned, persistent node entity. Blink mutates its GC node in
/// place; Rust retains immutable frame states referring to this same entity.
/// Its allocation, rather than a client hash or dump ID, defines identity.
#[derive(Debug)]
pub struct PaintPropertyNodeIdentity {
    // Chromium: third_party/blink/renderer/platform/graphics/paint/
    // paint_property_node.h:91-97,228-230. The pair is one mutable cache slot
    // on the actual Paint property node, keyed by the cc property-tree
    // sequence. Packing both i32 values makes concurrent readers observe one
    // complete source state without adding a compositor-owned side table.
    cc_sequence_and_node_id: AtomicU64,
    // Chromium: paint_property_node.h:191-230. These are mutable members of
    // the resident Paint node even when the property state is reached through
    // a const PaintArtifact. Keeping them on the persistent identity allocation
    // preserves that ownership across immutable Rust value revisions.
    changed: AtomicU8,
    changed_sequence_number: AtomicI32,
    _private: (),
}

impl PaintPropertyNodeIdentity {
    const INVALID_CC_NODE_ID: i32 = -1;

    fn pack_cc_mapping(sequence_number: i32, node_id: i32) -> u64 {
        ((sequence_number as u32 as u64) << 32) | node_id as u32 as u64
    }

    fn cc_node_id(&self, sequence_number: i32) -> i32 {
        let mapping = self.cc_sequence_and_node_id.load(Ordering::Relaxed);
        let mapped_sequence = (mapping >> 32) as u32 as i32;
        if mapped_sequence == sequence_number {
            mapping as u32 as i32
        } else {
            Self::INVALID_CC_NODE_ID
        }
    }

    fn set_cc_node_id(&self, sequence_number: i32, node_id: i32) {
        self.cc_sequence_and_node_id.store(
            Self::pack_cc_mapping(sequence_number, node_id),
            Ordering::Relaxed,
        );
    }

    fn node_changed(&self) -> PaintPropertyChangeType {
        match self.changed.load(Ordering::Relaxed) {
            0 => PaintPropertyChangeType::Unchanged,
            1 => PaintPropertyChangeType::ChangedOnlyCompositedValues,
            2 => PaintPropertyChangeType::ChangedOnlySimpleValues,
            3 => PaintPropertyChangeType::ChangedOnlyValues,
            4 => PaintPropertyChangeType::NodeAddedOrRemoved,
            _ => unreachable!("PaintPropertyChangeType is a bounded enum"),
        }
    }

    fn add_changed(&self, changed: PaintPropertyChangeType) {
        self.changed.fetch_max(changed as u8, Ordering::Relaxed);
    }

    fn clear_changed(&self, sequence_number: i32) {
        debug_assert_ne!(
            self.changed_sequence_number.load(Ordering::Relaxed),
            sequence_number
        );
        self.changed
            .store(PaintPropertyChangeType::Unchanged as u8, Ordering::Relaxed);
        self.changed_sequence_number
            .store(sequence_number, Ordering::Relaxed);
    }
}

#[derive(Clone, Debug)]
pub struct PaintPropertyNodeLifecycle {
    pub identity: Arc<PaintPropertyNodeIdentity>,
    pub revision: u64,
    pub changed: PaintPropertyChangeType,
}
impl Default for PaintPropertyNodeLifecycle {
    fn default() -> Self {
        Self {
            identity: Arc::new(PaintPropertyNodeIdentity {
                cc_sequence_and_node_id: AtomicU64::new(
                    PaintPropertyNodeIdentity::pack_cc_mapping(
                        0,
                        PaintPropertyNodeIdentity::INVALID_CC_NODE_ID,
                    ),
                ),
                changed: AtomicU8::new(PaintPropertyChangeType::NodeAddedOrRemoved as u8),
                changed_sequence_number: AtomicI32::new(0),
                _private: (),
            }),
            revision: 0,
            changed: PaintPropertyChangeType::NodeAddedOrRemoved,
        }
    }
}
impl PartialEq for PaintPropertyNodeLifecycle {
    fn eq(&self, other: &Self) -> bool {
        self.same_node(other) && self.revision == other.revision
    }
}
impl PaintPropertyNodeLifecycle {
    pub fn same_node(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }
    pub fn cc_node_id(&self, sequence_number: i32) -> i32 {
        self.identity.cc_node_id(sequence_number)
    }
    pub fn set_cc_node_id(&self, sequence_number: i32, node_id: i32) {
        self.identity.set_cc_node_id(sequence_number, node_id);
    }
    fn root() -> Self {
        let result = Self {
            changed: PaintPropertyChangeType::Unchanged,
            ..Self::default()
        };
        result
            .identity
            .changed
            .store(PaintPropertyChangeType::Unchanged as u8, Ordering::Relaxed);
        result
    }
    fn updated(&self, changed: PaintPropertyChangeType) -> Self {
        self.identity.add_changed(changed);
        Self {
            identity: self.identity.clone(),
            revision: self.revision + u64::from(changed != PaintPropertyChangeType::Unchanged),
            changed,
        }
    }

    pub fn node_changed(&self) -> PaintPropertyChangeType {
        self.identity.node_changed()
    }
    pub fn changed_sequence_number(&self) -> i32 {
        self.identity
            .changed_sequence_number
            .load(Ordering::Relaxed)
    }
    pub fn clear_changed(&self, sequence_number: i32) {
        self.identity.clear_changed(sequence_number);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintBackfaceVisibility {
    #[default]
    Inherited,
    Visible,
    Hidden,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformPaintPropertyNode {
    /// Source: transform_paint_property_node.h:110-134. These remain source
    /// state fields even when a feature does not populate them on this node.
    pub compositor: TransformCompositorState,
    /// Source: transform_paint_property_node.h:472-485. Owns only the lazy
    /// native cache allocation; cloning a node starts a new empty cache slot.
    pub geometry_mapper_cache: crate::opaque_paint_cache_slot::OpaquePaintCacheSlot,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub matrix: TransformMatrix,
    // The standalone geometry mapper has already folded transform-origin into
    // the matrix. Keeping origin zero preserves that resolved transformation.
    pub origin: [f64; 3],
    pub flattens_inherited_transform: bool,
    pub rendering_context_id: u64,
    pub backface_visibility: PaintBackfaceVisibility,
    pub scroll: Option<Arc<ScrollPaintPropertyNode>>,
    /// Source: transform_paint_property_node.h:221-223. A fixed/overscroll
    /// paint-offset node can select a non-ancestor native scroll translation.
    pub scroll_parent_scroll_translation: Option<Arc<TransformPaintPropertyNode>>,
    /// Layout's immutable sticky geometry. The compositor resolves its scroll
    /// and ancestor identities through the property chain when applying an
    /// impl-side scroll delta.
    pub sticky_constraint: Option<StickyPositionPaintData>,
    pub direct_compositing_reasons: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformCompositorState {
    pub sticky_constraint: Option<PaintCompositorStickyConstraint>,
    pub compositor_element_id: foundation::CompositorElementId,
    pub visible_frame_element_id: foundation::CompositorElementId,
    pub in_subtree_of_page_scale: bool,
    pub animation_is_axis_aligned: bool,
    pub is_frame_paint_offset_translation: bool,
    pub is_for_svg_child: bool,
    pub anchor_position_scroll_data: Option<PaintAnchorPositionScrollData>,
}
impl Default for TransformCompositorState {
    fn default() -> Self {
        Self {
            sticky_constraint: None,
            compositor_element_id: Default::default(),
            visible_frame_element_id: Default::default(),
            in_subtree_of_page_scale: true,
            animation_is_axis_aligned: false,
            is_frame_paint_offset_translation: false,
            is_for_svg_child: false,
            anchor_position_scroll_data: None,
        }
    }
}
/// Source value: cc/trees/sticky_position_constraint.h:23-68. Paint owns the
/// builder's produced compositor constraint, distinct from Layout constraints.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintCompositorStickyConstraint {
    pub anchored: [bool; 4],
    pub offsets: [f32; 4],
    pub constraint_box_rect: PaintRect,
    pub sticky_box_rect: PaintRect,
    pub containing_block_rect: PaintRect,
    pub pixel_snap_offset: [f32; 2],
    pub x_scroll_ancestor: foundation::CompositorElementId,
    pub y_scroll_ancestor: foundation::CompositorElementId,
    pub nearest_shifting_sticky_box: foundation::CompositorElementId,
    pub nearest_shifting_containing_block: foundation::CompositorElementId,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintAnchorPositionScrollData {
    pub adjustment_container_ids: Vec<foundation::CompositorElementId>,
    pub accumulated_scroll_origin: [i32; 2],
    pub needs_scroll_adjustment_in_x: bool,
    pub needs_scroll_adjustment_in_y: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClipPaintPropertyNode {
    /// Source: clip_paint_property_node.h:249-258.
    pub geometry_mapper_cache: crate::opaque_paint_cache_slot::OpaquePaintCacheSlot,
    /// Source: clip_paint_property_node.h:88-125,188-217. Layout and paint
    /// clips are distinct source values, including compositor animation bounds.
    pub layout_clip_rects: ClipLayoutRects,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub local_transform_space: Arc<TransformPaintPropertyNode>,
    pub rect: Option<PaintRect>,
    pub radii: PaintCornerRadii,
    /// None is no path clip. A present native empty path clips everything.
    /// SVG children retain their own fill rules in the native shape owner.
    pub clip_path: Option<Arc<PaintClipPath>>,
    pub clip_path_error: Option<String>,
    /// UpdatePixelMovingFilterClipExpander retains the filter whose input
    /// bounds must be expanded; an infinite rect alone loses this relationship.
    pub pixel_moving_filter: Option<Arc<EffectPaintPropertyNode>>,
}

impl ClipPaintPropertyNode {
    /// Source: clip_paint_property_node.h:88-99,110-125. Normal native clip
    /// creation supplies equal precise/paint input until the builder supplies
    /// its unsnapped layout value or an explicit animation expansion.
    pub fn with_layout_clips(mut self) -> Self {
        self.layout_clip_rects =
            ClipLayoutRects::new(self.rect, self.radii.HasRadius(), None, None);
        self
    }
    pub fn with_layout_clip_values(
        mut self,
        precise: Option<PaintRect>,
        expanded: Option<PaintRect>,
        excluding_overlay: Option<PaintRect>,
    ) -> Self {
        self.layout_clip_rects =
            ClipLayoutRects::new(precise, self.radii.HasRadius(), expanded, excluding_overlay);
        self
    }
    pub fn clip_path_child_count(&self) -> usize {
        self.clip_path
            .as_ref()
            .map_or(0, |shape| shape.paths().len())
    }
    pub fn single_clip_path(&self) -> Option<&crate::paint_clip_path::PaintPath> {
        self.clip_path.as_ref()?.single_path().map(Arc::as_ref)
    }
    pub fn clip_path_verb_count(&self) -> usize {
        self.clip_path.as_ref().map_or(0, |shape| {
            shape.paths().iter().map(|p| p.verb_count()).sum()
        })
    }
}

/// Source: clip_paint_property_node.h:110-125,188-217. None is the source
/// infinite FloatClipRect sentinel; each optional overlay rect is unrounded.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipLayoutRect {
    pub rect: Option<PaintRect>,
    pub has_radius: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClipLayoutRects {
    pub precise: ClipLayoutRect,
    pub expanded: ClipLayoutRect,
    pub excluding_overlay: Option<ClipLayoutRect>,
}
impl ClipLayoutRects {
    pub fn new(
        precise: Option<PaintRect>,
        rounded: bool,
        expanded: Option<PaintRect>,
        excluding_overlay: Option<PaintRect>,
    ) -> Self {
        Self {
            precise: ClipLayoutRect {
                rect: precise,
                has_radius: rounded,
            },
            expanded: ClipLayoutRect {
                rect: expanded.or(precise),
                has_radius: rounded && expanded.is_none(),
            },
            excluding_overlay: excluding_overlay.map(|rect| ClipLayoutRect {
                rect: Some(rect),
                has_radius: false,
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectPaintPropertyNode {
    pub compositor: EffectCompositorState,
    /// Source: effect_paint_property_node.h:78-95,261-264.
    pub backdrop_filter_bounds: Option<Arc<crate::paint_clip_path::PaintPath>>,
    /// Source: effect_paint_property_node.h:427-429; builder.cc:2263-2264,
    /// 2493-2498,2690-2691. The producer publishes inherited participation.
    pub self_or_ancestor_participates_in_view_transition: bool,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    pub parent: Option<Arc<Self>>,
    pub local_transform_space: Arc<TransformPaintPropertyNode>,
    pub output_clip: Option<Arc<ClipPaintPropertyNode>>,
    /// Filter reference box in `local_transform_space`.  This is independent
    /// of `output_clip`: SVG filterUnits/primitiveUnits and the filter region
    /// are resolved from the owning border box, while the output clip belongs
    /// to the surrounding property tree.
    pub filter_reference_box: Option<PaintRect>,
    /// SVG userSpaceOnUse values are raw resource coordinates. HTML/CSS
    /// owners map them through EffectiveZoom before the property transform;
    /// native SVG owners use one.
    pub filter_reference_scale: f64,
    pub opacity: f32,
    pub blend_mode: PaintBlendMode,
    pub filters: PropertyFilterOperations,
    /// Filters applied to pixels already present in the parent render target.
    /// They are never folded into the foreground `filters` chain.
    pub backdrop_filters: PropertyFilterOperations,
    pub backdrop_filter_has_reference: bool,
    /// Border-box backdrop output clip. It is separate from `output_clip` so
    /// foreground descendants keep their ordinary overflow semantics.
    pub backdrop_filter_clip: Option<Arc<ClipPaintPropertyNode>>,
    pub isolates_blending: bool,
    // Legacy standalone wrapper metadata; formal Mask nodes use is_mask.
    pub has_mask: bool,
    /// ObjectPaintProperties::Mask has internal SkBlendMode::kDstIn. This is
    /// independent of the CSS blend-mode input, which cannot express DstIn.
    pub is_mask: bool,
    pub direct_compositing_reasons: Vec<&'static str>,
}

/// Source: effect_paint_property_node.h:150-178. Value fields belong to the
/// published node, not a compositor-side registry or inferred feature flags.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EffectCompositorState {
    /// Source: effect_paint_property_node.h:66-75. The producer's reference
    /// filter output rectangle; absent when this state owns no FilterInfo.
    pub filter_output_bounds: Option<PaintRect>,
    pub has_filter_info: bool,
    pub has_backdrop_filter_info: bool,
    pub compositor_element_id: foundation::CompositorElementId,
    pub backdrop_mask_element_id: foundation::CompositorElementId,
    pub view_transition_element_resource_id: Option<([u64; 2], u32, bool)>,
    pub restriction_target_id: [u64; 2],
    pub needs_effect_for_2d_scale_transform: bool,
    pub is_in_drawable_canvas_subtree: bool,
    pub canvas_child_state: Option<Arc<PaintCanvasChildState>>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PaintCanvasChildState {
    pub id: i32,
    pub reference_box_offset: Offset,
    pub content_state: PropertyTreeState,
}

impl EffectPaintPropertyNode {
    /// Current producer publishes FilterInfo only at its actual nonempty filter
    /// construction sites; retain this ownership fact on the published node.
    pub fn with_filter_presence(mut self) -> Self {
        self.compositor.has_filter_info = !self.filters.is_empty();
        self.compositor.has_backdrop_filter_info = !self.backdrop_filters.is_empty();
        self
    }
    pub fn with_inherited_view_transition(mut self) -> Self {
        self.self_or_ancestor_participates_in_view_transition |= self
            .parent
            .as_ref()
            .is_some_and(|parent| parent.self_or_ancestor_participates_in_view_transition);
        self
    }
    /// Used by the producer creating a participating view-transition effect;
    /// descendants subsequently inherit this same source-owned value.
    pub fn with_view_transition_participation(mut self, participates: bool) -> Self {
        self.self_or_ancestor_participates_in_view_transition = participates;
        self.with_inherited_view_transition()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollPaintPropertyNode {
    pub compositor: ScrollCompositorState,
    pub lifecycle: PaintPropertyNodeLifecycle,
    pub id: u64,
    /// Stable native owner identity used to connect Layout's sticky scroll
    /// ancestor to this Paint property node without exposing either owner.
    pub owner_client_id: u64,
    pub parent: Option<Arc<Self>>,
    pub overflow_clip: Option<Arc<ClipPaintPropertyNode>>,
    pub container_rect: PaintRect,
    pub contents_rect: PaintRect,
    pub minimum_scroll_offset: Offset,
    pub maximum_scroll_offset: Offset,
    pub user_scrollable_horizontal: bool,
    pub user_scrollable_vertical: bool,
}
// Source: scroll_paint_property_node.h:52-83. Native optional/feature fields
// stay on the producer node and retain their constructor defaults until set.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollCompositorState {
    pub compositor_element_id: foundation::CompositorElementId,
    pub prevent_scroll_axis_locking: bool,
    pub prevent_viewport_scrolling_from_inner: bool,
    pub max_scroll_offset_affected_by_page_scale: bool,
    pub composited_scrolling_preference: PaintCompositedScrollingPreference,
    pub main_thread_repaint_reasons: u8,
    pub overscroll_behavior: [layoutng_assembly::internal::layout_input::OverscrollBehavior; 2],
    pub scrolling_contents_cull_rect: PaintRect,
    /// The producer queried scroll-snap state and proved that no native
    /// SnapContainerData exists. False means a non-none snap payload is still
    /// required before this node can enter cc.
    pub snap_container_data_absent: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaintCompositedScrollingPreference {
    #[default]
    Default,
    Preferred,
    NotPreferred,
}
impl Default for ScrollCompositorState {
    fn default() -> Self {
        let r = foundation::infinite_int_rect::InfiniteIntRect();
        Self {
            compositor_element_id: Default::default(),
            prevent_scroll_axis_locking: false,
            prevent_viewport_scrolling_from_inner: false,
            max_scroll_offset_affected_by_page_scale: false,
            composited_scrolling_preference: Default::default(),
            main_thread_repaint_reasons: 0,
            overscroll_behavior: [Default::default(); 2],
            scrolling_contents_cull_rect: PaintRect {
                x: r.x() as f64,
                y: r.y() as f64,
                width: r.width() as f64,
                height: r.height() as f64,
            },
            snap_container_data_absent: false,
        }
    }
}

/// Complete inherited state. Scroll is reached through transform.scroll,
/// matching Blink PropertyTreeState's three references.
#[derive(Clone, Debug)]
pub struct PropertyTreeState {
    pub transform: Arc<TransformPaintPropertyNode>,
    pub clip: Arc<ClipPaintPropertyNode>,
    pub effect: Arc<EffectPaintPropertyNode>,
}

impl PartialEq for PropertyTreeState {
    fn eq(&self, other: &Self) -> bool {
        self.transform
            .lifecycle
            .same_node(&other.transform.lifecycle)
            && self.clip.lifecycle.same_node(&other.clip.lifecycle)
            && self.effect.lifecycle.same_node(&other.effect.lifecycle)
    }
}
impl Eq for PropertyTreeState {}

impl Default for PropertyTreeState {
    fn default() -> Self {
        // Blink Root() references shared sentinel nodes, rather than allocating
        // distinct identity nodes for each default state.
        static ROOT: OnceLock<PropertyTreeState> = OnceLock::new();
        ROOT.get_or_init(|| {
            let infinite = foundation::infinite_int_rect::InfiniteIntRect();
            let infinite = PaintRect {
                x: infinite.x() as f64,
                y: infinite.y() as f64,
                width: infinite.width() as f64,
                height: infinite.height() as f64,
            };
            // ScrollRoot is a sentinel, not a real scroll. Its nullable overflow
            // clip also avoids manufacturing a cyclic root clip relationship.
            let scroll = Arc::new(ScrollPaintPropertyNode {
                compositor: Default::default(),
                lifecycle: PaintPropertyNodeLifecycle::root(),
                id: 0,
                owner_client_id: 0,
                parent: None,
                overflow_clip: None,
                container_rect: infinite,
                contents_rect: infinite,
                minimum_scroll_offset: Offset::default(),
                maximum_scroll_offset: Offset::default(),
                user_scrollable_horizontal: false,
                user_scrollable_vertical: false,
            });
            let transform = Arc::new(TransformPaintPropertyNode {
                compositor: TransformCompositorState {
                    in_subtree_of_page_scale: false,
                    ..Default::default()
                },
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: None,
                lifecycle: PaintPropertyNodeLifecycle::root(),
                id: 0,
                parent: None,
                matrix: TransformMatrix::default(),
                origin: [0.0; 3],
                flattens_inherited_transform: false,
                rendering_context_id: 0,
                backface_visibility: PaintBackfaceVisibility::Visible,
                scroll: Some(scroll),
                sticky_constraint: None,
                direct_compositing_reasons: Vec::new(),
            });
            let clip = Arc::new(
                ClipPaintPropertyNode {
                    geometry_mapper_cache: Default::default(),
                    layout_clip_rects: Default::default(),
                    lifecycle: PaintPropertyNodeLifecycle::root(),
                    id: 0,
                    parent: None,
                    local_transform_space: transform.clone(),
                    rect: None,
                    radii: PaintCornerRadii::default(),
                    clip_path: None,
                    clip_path_error: None,
                    pixel_moving_filter: None,
                }
                .with_layout_clips(),
            );
            let effect = Arc::new(
                EffectPaintPropertyNode {
                    compositor: Default::default(),
                    backdrop_filter_bounds: None,
                    self_or_ancestor_participates_in_view_transition: false,
                    lifecycle: PaintPropertyNodeLifecycle::root(),
                    id: 0,
                    parent: None,
                    local_transform_space: transform.clone(),
                    output_clip: None,
                    filter_reference_box: None,
                    filter_reference_scale: 1.0,
                    opacity: 1.0,
                    blend_mode: PaintBlendMode::kNormal,
                    filters: Default::default(),
                    backdrop_filters: Default::default(),
                    backdrop_filter_has_reference: false,
                    backdrop_filter_clip: None,
                    isolates_blending: false,
                    has_mask: false,
                    is_mask: false,
                    direct_compositing_reasons: Vec::new(),
                }
                .with_inherited_view_transition(),
            );
            PropertyTreeState {
                transform,
                clip,
                effect,
            }
        })
        .clone()
    }
}

impl PropertyTreeState {
    pub fn same_nodes(&self, other: &Self) -> bool {
        self == other
    }
    /// Frame-state comparison is intentionally separate from node identity.
    /// A stable node can have new values (including changed ancestor values).
    pub fn same_values(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.transform, &other.transform)
            && Arc::ptr_eq(&self.clip, &other.clip)
            && Arc::ptr_eq(&self.effect, &other.effect)
        {
            return true;
        }
        // Compare each shared graph node once. Recursive derived equality can
        // revisit filter/clip cross-links exponentially in a nested subtree.
        let a = PaintPropertyTrees::from_states([self]);
        let b = PaintPropertyTrees::from_states([other]);
        same_directory(&a.transforms, &b.transforms, |a, b| {
            a.lifecycle == b.lifecycle
                && transform_state_change(a, b) == PaintPropertyChangeType::Unchanged
        }) && same_directory(&a.clips, &b.clips, |a, b| {
            a.lifecycle == b.lifecycle
                && clip_state_change(a, b) == PaintPropertyChangeType::Unchanged
        }) && same_directory(&a.effects, &b.effects, |a, b| {
            a.lifecycle == b.lifecycle
                && effect_state_change(a, b) == PaintPropertyChangeType::Unchanged
        }) && same_directory(&a.scrolls, &b.scrolls, |a, b| {
            a.lifecycle == b.lifecycle
                && scroll_state_change(a, b) == PaintPropertyChangeType::Unchanged
        })
    }
    pub fn changed_to_root(&self) -> PaintPropertyChangeType {
        let trees = PaintPropertyTrees::from_states([self]);
        trees
            .transforms
            .values()
            .map(|n| n.lifecycle.node_changed())
            .chain(trees.clips.values().map(|n| n.lifecycle.node_changed()))
            .chain(trees.effects.values().map(|n| n.lifecycle.node_changed()))
            .chain(trees.scrolls.values().map(|n| n.lifecycle.node_changed()))
            .max()
            .unwrap_or_default()
    }
    pub fn nearest_scroll(&self) -> Option<Arc<ScrollPaintPropertyNode>> {
        let mut transform = Some(self.transform.clone());
        while let Some(node) = transform {
            if let Some(scroll) = &node.scroll {
                return Some(scroll.clone());
            }
            transform = node.parent.clone();
        }
        None
    }
}

/// A deduplicated directory suitable for exporting the complete referenced
/// ancestry of one paint artifact. Per-type roots all have ID zero.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct PaintPropertyTrees {
    pub transforms: BTreeMap<u64, Arc<TransformPaintPropertyNode>>,
    pub clips: BTreeMap<u64, Arc<ClipPaintPropertyNode>>,
    pub effects: BTreeMap<u64, Arc<EffectPaintPropertyNode>>,
    pub scrolls: BTreeMap<u64, Arc<ScrollPaintPropertyNode>>,
    transforms_by_identity: BTreeMap<usize, Arc<TransformPaintPropertyNode>>,
    clips_by_identity: BTreeMap<usize, Arc<ClipPaintPropertyNode>>,
    effects_by_identity: BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    scrolls_by_identity: BTreeMap<usize, Arc<ScrollPaintPropertyNode>>,
}

impl PaintPropertyTrees {
    fn identity(lifecycle: &PaintPropertyNodeLifecycle) -> usize {
        Arc::as_ptr(&lifecycle.identity) as usize
    }
    pub fn transform_for(
        &self,
        node: &TransformPaintPropertyNode,
    ) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms_by_identity
            .get(&Self::identity(&node.lifecycle))
    }
    pub fn clip_for(&self, node: &ClipPaintPropertyNode) -> Option<&Arc<ClipPaintPropertyNode>> {
        self.clips_by_identity.get(&Self::identity(&node.lifecycle))
    }
    pub fn effect_for(
        &self,
        node: &EffectPaintPropertyNode,
    ) -> Option<&Arc<EffectPaintPropertyNode>> {
        self.effects_by_identity
            .get(&Self::identity(&node.lifecycle))
    }
    pub fn scroll_for(
        &self,
        node: &ScrollPaintPropertyNode,
    ) -> Option<&Arc<ScrollPaintPropertyNode>> {
        self.scrolls_by_identity
            .get(&Self::identity(&node.lifecycle))
    }
    pub fn transform_snapshots(&self) -> impl Iterator<Item = &Arc<TransformPaintPropertyNode>> {
        self.transforms_by_identity.values()
    }
    pub fn clip_snapshots(&self) -> impl Iterator<Item = &Arc<ClipPaintPropertyNode>> {
        self.clips_by_identity.values()
    }
    pub fn effect_snapshots(&self) -> impl Iterator<Item = &Arc<EffectPaintPropertyNode>> {
        self.effects_by_identity.values()
    }
    pub fn scroll_snapshots(&self) -> impl Iterator<Item = &Arc<ScrollPaintPropertyNode>> {
        self.scrolls_by_identity.values()
    }
    fn insert_transform(&mut self, node: Arc<TransformPaintPropertyNode>) {
        self.transforms_by_identity
            .insert(Self::identity(&node.lifecycle), node.clone());
        self.transforms.entry(node.id).or_insert(node);
    }
    fn insert_clip(&mut self, node: Arc<ClipPaintPropertyNode>) {
        self.clips_by_identity
            .insert(Self::identity(&node.lifecycle), node.clone());
        self.clips.entry(node.id).or_insert(node);
    }
    fn insert_effect(&mut self, node: Arc<EffectPaintPropertyNode>) {
        self.effects_by_identity
            .insert(Self::identity(&node.lifecycle), node.clone());
        self.effects.entry(node.id).or_insert(node);
    }
    fn insert_scroll(&mut self, node: Arc<ScrollPaintPropertyNode>) {
        self.scrolls_by_identity
            .insert(Self::identity(&node.lifecycle), node.clone());
        self.scrolls.entry(node.id).or_insert(node);
    }
    /// Clear every referenced ancestor, including fresh nodes without a
    /// native-owner slot. Blink's PropertyTreeState::ClearChangedToRoot walks
    /// the actual state graph; the owner directory is not the complete graph.
    pub fn clear_state_change_markers(&mut self, state: &mut PropertyTreeState) {
        let mut rebind = RetainedScrollSnapshotRebinder {
            translations: BTreeMap::new(),
            trees: std::mem::take(self),
            changed: PaintPropertyChangeType::Unchanged,
        };
        state.transform = rebind.transform(&state.transform);
        state.clip = rebind.clip(&state.clip);
        state.effect = rebind.effect(&state.effect);
        *self = rebind.trees;
    }

    pub fn from_states<'a>(states: impl IntoIterator<Item = &'a PropertyTreeState>) -> Self {
        let mut trees = Self::default();
        for state in states {
            trees.add_transform(&state.transform);
            trees.add_clip(&state.clip);
            trees.add_effect(&state.effect);
        }
        trees
    }
    fn add_transform(&mut self, node: &Arc<TransformPaintPropertyNode>) {
        if self.transform_for(node).is_some() {
            return;
        }
        self.insert_transform(node.clone());
        if let Some(parent) = &node.parent {
            self.add_transform(parent);
        }
        if let Some(scroll) = &node.scroll {
            self.add_scroll(scroll);
        }
        if let Some(parent) = &node.scroll_parent_scroll_translation {
            self.add_transform(parent);
        }
    }
    fn add_clip(&mut self, node: &Arc<ClipPaintPropertyNode>) {
        if self.clip_for(node).is_some() {
            return;
        }
        self.insert_clip(node.clone());
        if let Some(parent) = &node.parent {
            self.add_clip(parent);
        }
        if let Some(filter) = &node.pixel_moving_filter {
            self.add_effect(filter);
        }
        self.add_transform(&node.local_transform_space);
    }
    fn add_effect(&mut self, node: &Arc<EffectPaintPropertyNode>) {
        if self.effect_for(node).is_some() {
            return;
        }
        self.insert_effect(node.clone());
        if let Some(parent) = &node.parent {
            self.add_effect(parent);
        }
        self.add_transform(&node.local_transform_space);
        if let Some(clip) = &node.output_clip {
            self.add_clip(clip);
        }
        if let Some(clip) = &node.backdrop_filter_clip {
            self.add_clip(clip);
        }
    }
    fn add_scroll(&mut self, node: &Arc<ScrollPaintPropertyNode>) {
        if self.scroll_for(node).is_some() {
            return;
        }
        self.insert_scroll(node.clone());
        if let Some(parent) = &node.parent {
            self.add_scroll(parent);
        }
        if let Some(clip) = &node.overflow_clip {
            self.add_clip(clip);
        }
    }
}

/// ObjectPaintProperties owns one slot per property kind and fragment. The
/// source-specific transform slots mirror the supported ObjectPaintProperties
/// roles. Independent CSS translate/rotate/scale/offset payload is unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaintPropertyRole {
    ViewportClip,
    PaintOffsetTranslation,
    StickyTranslation,
    TextCombineTransform,
    Transform,
    Perspective,
    SvgLocalTransform,
    ClipPathClip,
    CssClip,
    BackgroundClip,
    BackdropFilterClip,
    MaskClip,
    Effect,
    Mask,
    Filter,
    PixelMovingFilterClipExpander,
    OverflowClip,
    HorizontalScrollbarEffect,
    VerticalScrollbarEffect,
    Scroll,
    ScrollTranslation,
    ReplacedContentTransform,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaintPropertyOwner {
    Viewport,
    NativeFragment { client: u64, fragment: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PaintPropertyKey {
    pub owner: PaintPropertyOwner,
    pub role: PaintPropertyRole,
}

/// Persistent Create-or-Update slots; source: ObjectPaintProperties::Update /
/// Clear in core/paint/object_paint_properties.h. The map is scoped to one
/// retained paint lifecycle, never global and never keyed by DOM order.
/// Old frames keep immutable snapshots; identity allocations are retained by
/// both snapshots. Removing a slot and later adding it creates a new entity.
#[derive(Clone, Debug)]
pub struct PaintPropertyNodeStore {
    next_id: u64,
    transforms: BTreeMap<PaintPropertyKey, Arc<TransformPaintPropertyNode>>,
    clips: BTreeMap<PaintPropertyKey, Arc<ClipPaintPropertyNode>>,
    effects: BTreeMap<PaintPropertyKey, Arc<EffectPaintPropertyNode>>,
    scrolls: BTreeMap<PaintPropertyKey, Arc<ScrollPaintPropertyNode>>,
    // A cleared published graph stays cleared until an updater marks a node.
    // This avoids scanning all resident slots on entirely clean paint passes.
    change_markers_cleared: bool,
    pub changed: PaintPropertyChangeType,
}
impl Default for PaintPropertyNodeStore {
    fn default() -> Self {
        Self {
            next_id: 1,
            transforms: BTreeMap::new(),
            clips: BTreeMap::new(),
            effects: BTreeMap::new(),
            scrolls: BTreeMap::new(),
            change_markers_cleared: true,
            changed: Default::default(),
        }
    }
}
impl PaintPropertyNodeStore {
    pub fn compositor_scroll_translations(
        &self,
    ) -> impl Iterator<Item = &Arc<TransformPaintPropertyNode>> {
        self.transforms.iter().filter_map(|(key, node)| {
            (key.role == PaintPropertyRole::ScrollTranslation).then_some(node)
        })
    }
    pub fn layout_view_scroll_translation(
        &self,
        client: u64,
        fragment: u32,
    ) -> Option<&Arc<TransformPaintPropertyNode>> {
        self.transforms.get(&PaintPropertyKey {
            owner: PaintPropertyOwner::NativeFragment { client, fragment },
            role: PaintPropertyRole::ScrollTranslation,
        })
    }
    /// The real recording coordinate anchors, indexed by the same native
    /// owner/role slots used by the property builder. No new layer identity.
    pub fn scroll_offsets(&self) -> BTreeMap<PaintPropertyOwner, Offset> {
        self.transforms
            .iter()
            .filter(|(key, _)| key.role == PaintPropertyRole::ScrollTranslation)
            .map(|(key, node)| {
                (
                    key.owner,
                    Offset {
                        x: -node.matrix.values[12],
                        y: -node.matrix.values[13],
                    },
                )
            })
            .collect()
    }

    /// Current sticky values carried by the native property owner. Recording
    /// basis changes may rebase scroll while preserving these existing slots.
    pub fn sticky_offsets(
        &self,
    ) -> BTreeMap<PaintPropertyOwner, (Offset, StickyPositionPaintData)> {
        self.transforms
            .iter()
            .filter_map(|(key, node)| {
                (key.role == PaintPropertyRole::StickyTranslation)
                    .then(|| {
                        node.sticky_constraint.map(|constraint| {
                            (
                                key.owner,
                                (
                                    Offset {
                                        x: node.matrix.values[12],
                                        y: node.matrix.values[13],
                                    },
                                    constraint,
                                ),
                            )
                        })
                    })
                    .flatten()
            })
            .collect()
    }

    /// Retained native geometry has already proved unchanged owners/topology,
    /// styles, resources and fragment geometry. Update existing real scroll
    /// translations without constructing a new PrePaint tree. The caller must
    /// prove retained recording coverage before publishing this staged store.
    ///
    /// Source: paint_property_tree_builder.cc:3769-3827 and
    /// property_tree_manager.cc:138-176. This backend retains immutable Blink
    /// snapshots; it does not claim a cc composited-value update. Rebinding a
    /// snapshot reference preserves its entity and revision. Per-pass change
    /// markers clear on unchanged values, as in the ordinary property updater.
    pub fn retained_scroll_update(
        &self,
        offsets: &BTreeMap<PaintPropertyOwner, Offset>,
        sticky: &BTreeMap<PaintPropertyOwner, (Offset, StickyPositionPaintData)>,
    ) -> Option<(Self, PaintPropertyTrees)> {
        let mut translations = BTreeMap::new();
        for (owner, offset) in offsets {
            if !offset.x.is_finite() || !offset.y.is_finite() {
                return None;
            }
            if *offset != Offset::default()
                && !self.transforms.contains_key(&PaintPropertyKey {
                    owner: *owner,
                    role: PaintPropertyRole::ScrollTranslation,
                })
            {
                return None;
            }
        }
        for (key, node) in &self.transforms {
            if key.role != PaintPropertyRole::ScrollTranslation {
                continue;
            }
            let offset = offsets.get(&key.owner)?;
            // Preserve the previous scroll-only admission: translations without
            // real ScrollNodes (e.g. hidden overflow) require ordinary PrePaint.
            if !matches!(key.owner, PaintPropertyOwner::NativeFragment { .. })
                || node.scroll.as_ref().is_none_or(|scroll| scroll.id == 0)
                || node.origin != [0.0; 3]
            {
                return None;
            }
            let mut linear = node.matrix.values;
            linear[12] = 0.0;
            linear[13] = 0.0;
            if linear != TransformMatrix::default().values {
                return None;
            }
            let mut matrix = node.matrix;
            matrix.values[12] = -offset.x;
            matrix.values[13] = -offset.y;
            translations.insert(
                PaintPropertyTrees::identity(&node.lifecycle),
                (matrix, node.sticky_constraint),
            );
        }
        // FragmentPaintPropertyTreeBuilder::UpdateStickyTranslation uses
        // Layout's resolved offset and constraint data. Do not recompute sticky
        // geometry in Paint, or treat its scroll-dependent position as relayout.
        for owner in sticky.keys() {
            if !self.transforms.contains_key(&PaintPropertyKey {
                owner: *owner,
                role: PaintPropertyRole::StickyTranslation,
            }) {
                return None;
            }
        }
        for (key, node) in &self.transforms {
            if key.role != PaintPropertyRole::StickyTranslation {
                continue;
            }
            let (offset, constraint) = sticky.get(&key.owner)?;
            let before = node.sticky_constraint?;
            if !offset.x.is_finite()
                || !offset.y.is_finite()
                || node.origin != [0.0; 3]
                || !before.same_geometry(constraint)
            {
                return None;
            }
            let mut linear = node.matrix.values;
            linear[12] = 0.0;
            linear[13] = 0.0;
            if linear != TransformMatrix::default().values {
                return None;
            }
            let mut matrix = node.matrix;
            matrix.values[12] = offset.x;
            matrix.values[13] = offset.y;
            translations.insert(
                PaintPropertyTrees::identity(&node.lifecycle),
                (matrix, Some(*constraint)),
            );
        }
        Some(self.staged_snapshots(translations))
    }

    /// Clear the preceding paint pass's change markers in new immutable
    /// snapshots. As with Blink's ClearChangedToRoot, this changes no property
    /// values, node entities or revisions; old paint artifacts retain their
    /// original markers. The caller rebinds published states using the trees.
    pub fn cleared_change_markers(&self) -> Option<(Self, PaintPropertyTrees)> {
        if self.changed == PaintPropertyChangeType::Unchanged
            && (self.change_markers_cleared
                || (self
                    .transforms
                    .values()
                    .all(|n| n.lifecycle.changed == PaintPropertyChangeType::Unchanged)
                    && self
                        .clips
                        .values()
                        .all(|n| n.lifecycle.changed == PaintPropertyChangeType::Unchanged)
                    && self
                        .effects
                        .values()
                        .all(|n| n.lifecycle.changed == PaintPropertyChangeType::Unchanged)
                    && self
                        .scrolls
                        .values()
                        .all(|n| n.lifecycle.changed == PaintPropertyChangeType::Unchanged)))
        {
            return None;
        }
        Some(self.staged_snapshots(BTreeMap::new()))
    }

    fn staged_snapshots(
        &self,
        translations: BTreeMap<usize, (TransformMatrix, Option<StickyPositionPaintData>)>,
    ) -> (Self, PaintPropertyTrees) {
        let mut rebind = RetainedScrollSnapshotRebinder {
            translations,
            trees: PaintPropertyTrees::default(),
            changed: PaintPropertyChangeType::Unchanged,
        };
        // Empty directories still expose the real shared per-type sentinels.
        let roots = PropertyTreeState::default();
        rebind.transform(&roots.transform);
        rebind.clip(&roots.clip);
        rebind.effect(&roots.effect);
        let transforms = self
            .transforms
            .iter()
            .map(|(key, node)| (*key, rebind.transform(node)))
            .collect();
        let clips = self
            .clips
            .iter()
            .map(|(key, node)| (*key, rebind.clip(node)))
            .collect();
        let effects = self
            .effects
            .iter()
            .map(|(key, node)| (*key, rebind.effect(node)))
            .collect();
        let scrolls = self
            .scrolls
            .iter()
            .map(|(key, node)| (*key, rebind.scroll(node)))
            .collect();
        let store = Self {
            next_id: self.next_id,
            transforms,
            clips,
            effects,
            scrolls,
            change_markers_cleared: rebind.changed == PaintPropertyChangeType::Unchanged,
            changed: rebind.changed,
        };
        (store, rebind.trees)
    }

    pub fn viewport_clip(&self) -> Option<&Arc<ClipPaintPropertyNode>> {
        self.clips.get(&PaintPropertyKey {
            owner: PaintPropertyOwner::Viewport,
            role: PaintPropertyRole::ViewportClip,
        })
    }

    pub fn is_scroll_only_update_from(&self, old: &Self) -> bool {
        let reject = |kind: &str,
                      key: &PaintPropertyKey,
                      before: u64,
                      after: u64,
                      same_identity: bool,
                      change: PaintPropertyChangeType| {
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                eprintln!("scroll-property-reject kind={} key={:?} old_id={} new_id={} same_identity={} change={:?}",
                    kind, key, before, after, same_identity, change);
            }
            false
        };
        if self.transforms.len() != old.transforms.len()
            || self.clips.len() != old.clips.len()
            || self.effects.len() != old.effects.len()
            || self.scrolls.len() != old.scrolls.len()
        {
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                eprintln!(
                    "scroll-property-reject kind=slot-count old={:?} new={:?}",
                    (
                        old.transforms.len(),
                        old.clips.len(),
                        old.effects.len(),
                        old.scrolls.len()
                    ),
                    (
                        self.transforms.len(),
                        self.clips.len(),
                        self.effects.len(),
                        self.scrolls.len()
                    )
                );
            }
            return false;
        }
        for (key, node) in &self.transforms {
            let Some(before) = old.transforms.get(key) else {
                return reject(
                    "transform-slot",
                    key,
                    0,
                    node.id,
                    false,
                    PaintPropertyChangeType::NodeAddedOrRemoved,
                );
            };
            let change = transform_state_change(before, node);
            let same_identity = node.lifecycle.same_node(&before.lifecycle);
            if !same_identity
                || !(change == PaintPropertyChangeType::Unchanged
                    || (key.role == PaintPropertyRole::ScrollTranslation
                        && node.scroll.as_ref().is_some_and(|scroll| scroll.id != 0)
                        && change == PaintPropertyChangeType::ChangedOnlySimpleValues))
            {
                if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                    eprintln!(
                        "scroll-transform-values old={:?} new={:?}",
                        before.matrix.values, node.matrix.values
                    );
                }
                return reject("transform", key, before.id, node.id, same_identity, change);
            }
        }
        macro_rules! check_nodes {
            ($map:ident, $change:ident) => {
                for (key, node) in &self.$map {
                    let Some(before) = old.$map.get(key) else {
                        return reject(
                            stringify!($map),
                            key,
                            0,
                            node.id,
                            false,
                            PaintPropertyChangeType::NodeAddedOrRemoved,
                        );
                    };
                    let change = $change(before, node);
                    let same_identity = node.lifecycle.same_node(&before.lifecycle);
                    if !same_identity || change != PaintPropertyChangeType::Unchanged {
                        return reject(
                            stringify!($map),
                            key,
                            before.id,
                            node.id,
                            same_identity,
                            change,
                        );
                    }
                }
            };
        }
        check_nodes!(clips, clip_state_change);
        check_nodes!(effects, effect_state_change);
        check_nodes!(scrolls, scroll_state_change);
        true
    }
    pub fn clear_owner(&mut self, owner: PaintPropertyOwner) {
        self.transforms.retain(|k, _| k.owner != owner);
        self.clips.retain(|k, _| k.owner != owner);
        self.effects.retain(|k, _| k.owner != owner);
        self.scrolls.retain(|k, _| k.owner != owner);
    }
    pub fn updater<'a>(
        &'a mut self,
        keys: &BTreeMap<u64, PaintPropertyKey>,
        prior_keys: &BTreeMap<u64, PaintPropertyKey>,
        effect_sources: &'a BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    ) -> PropertyNodeUpdater<'a> {
        // Raw builder ids name nodes created in this pass. Retained clean
        // descendants can still hold the stable id assigned to the same
        // owner/role slot in an earlier pass, so index both forms before the
        // store is mutably borrowed. Once a rebuilt ancestor updates a slot,
        // every stale descendant reference can then resolve to that slot's
        // canonical current snapshot instead of reintroducing its old value.
        let mut stable_transform_keys = BTreeMap::new();
        for (key, node) in &self.transforms {
            stable_transform_keys.insert(node.id, *key);
        }
        let mut stable_clip_keys = BTreeMap::new();
        for (key, node) in &self.clips {
            stable_clip_keys.insert(node.id, *key);
        }
        let mut stable_effect_keys = BTreeMap::new();
        for (key, node) in &self.effects {
            stable_effect_keys.insert(node.id, *key);
        }
        let mut stable_scroll_keys = BTreeMap::new();
        for (key, node) in &self.scrolls {
            stable_scroll_keys.insert(node.id, *key);
        }
        self.changed = PaintPropertyChangeType::Unchanged;
        // A partial update cannot certify that every retained graph is clean.
        self.change_markers_cleared = false;
        PropertyNodeUpdater {
            store: self,
            raw_keys: keys.clone(),
            prior_raw_keys: prior_keys.clone(),
            stable_transform_keys,
            stable_clip_keys,
            stable_effect_keys,
            stable_scroll_keys,
            effect_sources,
            seen: BTreeSet::new(),
            transforms: BTreeMap::new(),
            clips: BTreeMap::new(),
            effects: BTreeMap::new(),
            scrolls: BTreeMap::new(),
        }
    }
    pub fn next_id(&self) -> u64 {
        self.next_id
    }
}

// One pass over the resident property graph, not the fragment tree. Memoized
// per-type IDs name existing directory entries; no new node identity is made.
struct RetainedScrollSnapshotRebinder {
    translations: BTreeMap<usize, (TransformMatrix, Option<StickyPositionPaintData>)>,
    trees: PaintPropertyTrees,
    changed: PaintPropertyChangeType,
}
impl RetainedScrollSnapshotRebinder {
    fn transform(
        &mut self,
        input: &Arc<TransformPaintPropertyNode>,
    ) -> Arc<TransformPaintPropertyNode> {
        if let Some(node) = self.trees.transform_for(input) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.transform(parent));
        let scroll = input.scroll.as_ref().map(|scroll| self.scroll(scroll));
        let scroll_parent = input
            .scroll_parent_scroll_translation
            .as_ref()
            .map(|parent| self.transform(parent));
        let (matrix, sticky_constraint) = self
            .translations
            .get(&PaintPropertyTrees::identity(&input.lifecycle))
            .copied()
            .unwrap_or((input.matrix, input.sticky_constraint));
        let node = if matrix == input.matrix
            && sticky_constraint == input.sticky_constraint
            && same_snapshot_option(&parent, &input.parent)
            && same_snapshot_option(&scroll, &input.scroll)
            && same_snapshot_option(&scroll_parent, &input.scroll_parent_scroll_translation)
            && input.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            let change = if sticky_constraint != input.sticky_constraint {
                PaintPropertyChangeType::ChangedOnlyValues
            } else if matrix != input.matrix {
                PaintPropertyChangeType::ChangedOnlySimpleValues
            } else {
                PaintPropertyChangeType::Unchanged
            };
            self.changed = self.changed.max(change);
            // Clone only retained payloads. The old parent/cross-tree/lifecycle
            // references would otherwise be cloned and immediately overwritten.
            Arc::new(TransformPaintPropertyNode {
                compositor: input.compositor.clone(),
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: scroll_parent,
                lifecycle: input.lifecycle.updated(change),
                id: input.id,
                parent,
                matrix,
                origin: input.origin,
                flattens_inherited_transform: input.flattens_inherited_transform,
                rendering_context_id: input.rendering_context_id,
                backface_visibility: input.backface_visibility,
                scroll,
                sticky_constraint,
                direct_compositing_reasons: input.direct_compositing_reasons.clone(),
            })
        };
        self.trees.insert_transform(node.clone());
        node
    }
    fn clip(&mut self, input: &Arc<ClipPaintPropertyNode>) -> Arc<ClipPaintPropertyNode> {
        if let Some(node) = self.trees.clip_for(input) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.clip(parent));
        let transform = self.transform(&input.local_transform_space);
        let filter = input
            .pixel_moving_filter
            .as_ref()
            .map(|filter| self.effect(filter));
        let node = if same_snapshot_option(&parent, &input.parent)
            && Arc::ptr_eq(&transform, &input.local_transform_space)
            && same_snapshot_option(&filter, &input.pixel_moving_filter)
            && input.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            Arc::new(ClipPaintPropertyNode {
                geometry_mapper_cache: Default::default(),
                layout_clip_rects: input.layout_clip_rects,
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                parent,
                local_transform_space: transform,
                rect: input.rect,
                radii: input.radii,
                clip_path: input.clip_path.clone(),
                clip_path_error: input.clip_path_error.clone(),
                pixel_moving_filter: filter,
            })
        };
        self.trees.insert_clip(node.clone());
        node
    }
    fn effect(&mut self, input: &Arc<EffectPaintPropertyNode>) -> Arc<EffectPaintPropertyNode> {
        if let Some(node) = self.trees.effect_for(input) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.effect(parent));
        let transform = self.transform(&input.local_transform_space);
        let clip = input.output_clip.as_ref().map(|clip| self.clip(clip));
        let backdrop_clip = input
            .backdrop_filter_clip
            .as_ref()
            .map(|clip| self.clip(clip));
        let node = if same_snapshot_option(&parent, &input.parent)
            && Arc::ptr_eq(&transform, &input.local_transform_space)
            && same_snapshot_option(&clip, &input.output_clip)
            && same_snapshot_option(&backdrop_clip, &input.backdrop_filter_clip)
            && input.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            Arc::new(EffectPaintPropertyNode {
                compositor: input.compositor.clone(),
                backdrop_filter_bounds: input.backdrop_filter_bounds.clone(),
                self_or_ancestor_participates_in_view_transition: input
                    .self_or_ancestor_participates_in_view_transition,
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                parent,
                local_transform_space: transform,
                output_clip: clip,
                filter_reference_box: input.filter_reference_box,
                filter_reference_scale: input.filter_reference_scale,
                opacity: input.opacity,
                blend_mode: input.blend_mode,
                filters: input.filters.clone(),
                backdrop_filters: input.backdrop_filters.clone(),
                backdrop_filter_has_reference: input.backdrop_filter_has_reference,
                backdrop_filter_clip: backdrop_clip,
                isolates_blending: input.isolates_blending,
                has_mask: input.has_mask,
                is_mask: input.is_mask,
                direct_compositing_reasons: input.direct_compositing_reasons.clone(),
            })
        };
        self.trees.insert_effect(node.clone());
        node
    }
    fn scroll(&mut self, input: &Arc<ScrollPaintPropertyNode>) -> Arc<ScrollPaintPropertyNode> {
        if let Some(node) = self.trees.scroll_for(input) {
            return node.clone();
        }
        let parent = input.parent.as_ref().map(|parent| self.scroll(parent));
        let clip = input.overflow_clip.as_ref().map(|clip| self.clip(clip));
        let node = if same_snapshot_option(&parent, &input.parent)
            && same_snapshot_option(&clip, &input.overflow_clip)
            && input.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
        {
            input.clone()
        } else {
            Arc::new(ScrollPaintPropertyNode {
                compositor: input.compositor.clone(),
                lifecycle: input.lifecycle.updated(PaintPropertyChangeType::Unchanged),
                id: input.id,
                owner_client_id: input.owner_client_id,
                parent,
                overflow_clip: clip,
                container_rect: input.container_rect,
                contents_rect: input.contents_rect,
                minimum_scroll_offset: input.minimum_scroll_offset,
                maximum_scroll_offset: input.maximum_scroll_offset,
                user_scrollable_horizontal: input.user_scrollable_horizontal,
                user_scrollable_vertical: input.user_scrollable_vertical,
            })
        };
        self.trees.insert_scroll(node.clone());
        node
    }
}

pub struct PropertyNodeUpdater<'a> {
    store: &'a mut PaintPropertyNodeStore,
    raw_keys: BTreeMap<u64, PaintPropertyKey>,
    prior_raw_keys: BTreeMap<u64, PaintPropertyKey>,
    stable_transform_keys: BTreeMap<u64, PaintPropertyKey>,
    stable_clip_keys: BTreeMap<u64, PaintPropertyKey>,
    stable_effect_keys: BTreeMap<u64, PaintPropertyKey>,
    stable_scroll_keys: BTreeMap<u64, PaintPropertyKey>,
    effect_sources: &'a BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    seen: BTreeSet<PaintPropertyKey>,
    transforms: BTreeMap<usize, Arc<TransformPaintPropertyNode>>,
    clips: BTreeMap<usize, Arc<ClipPaintPropertyNode>>,
    effects: BTreeMap<usize, Arc<EffectPaintPropertyNode>>,
    scrolls: BTreeMap<usize, Arc<ScrollPaintPropertyNode>>,
}

fn same_directory<T>(
    a: &BTreeMap<u64, Arc<T>>,
    b: &BTreeMap<u64, Arc<T>>,
    same: impl Fn(&T, &T) -> bool,
) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|((ka, a), (kb, b))| ka == kb && same(a, b))
}
fn same_snapshot_option<T>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}
impl TransformPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && same_snapshot_option(&self.scroll, &other.scroll)
            && same_snapshot_option(
                &self.scroll_parent_scroll_translation,
                &other.scroll_parent_scroll_translation,
            )
    }
}
impl ClipPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && Arc::ptr_eq(&self.local_transform_space, &other.local_transform_space)
            && same_snapshot_option(&self.pixel_moving_filter, &other.pixel_moving_filter)
    }
}
impl EffectPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && Arc::ptr_eq(&self.local_transform_space, &other.local_transform_space)
            && same_snapshot_option(&self.output_clip, &other.output_clip)
    }
}
impl ScrollPaintPropertyNode {
    fn same_snapshot_references(&self, other: &Self) -> bool {
        same_snapshot_option(&self.parent, &other.parent)
            && same_snapshot_option(&self.overflow_clip, &other.overflow_clip)
    }
}

// Each snapshot's parent and cross-tree links are canonicalized before its
// local state is compared. Compare node entities, not old ancestor values:
// Blink SetParent/State::ComputeChange compares pointers in these fields.
fn same_optional<T>(
    a: &Option<Arc<T>>,
    b: &Option<Arc<T>>,
    identity: impl Fn(&T) -> &PaintPropertyNodeLifecycle,
) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => identity(a).same_node(identity(b)),
        _ => false,
    }
}
fn transform_state_change(
    a: &TransformPaintPropertyNode,
    b: &TransformPaintPropertyNode,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    if !same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        || !same_optional(&a.scroll, &b.scroll, |n| &n.lifecycle)
        || !same_optional(
            &a.scroll_parent_scroll_translation,
            &b.scroll_parent_scroll_translation,
            |n| &n.lifecycle,
        )
        || a.flattens_inherited_transform != b.flattens_inherited_transform
        || a.rendering_context_id != b.rendering_context_id
        || a.backface_visibility != b.backface_visibility
        || a.sticky_constraint != b.sticky_constraint
        || a.direct_compositing_reasons != b.direct_compositing_reasons
    {
        return ChangedOnlyValues;
    }
    if a.matrix == b.matrix && a.origin == b.origin {
        return Unchanged;
    }
    // Only the unambiguous 2D translation case uses the official simple-value class;
    // other matrix updates conservatively request a full property update.
    let is_translation = |m: &TransformMatrix| {
        let mut v = m.values;
        v[12] = 0.0;
        v[13] = 0.0;
        v == TransformMatrix::default().values
    };
    if is_translation(&a.matrix) && is_translation(&b.matrix) {
        ChangedOnlySimpleValues
    } else {
        ChangedOnlyValues
    }
}
fn clip_state_change(
    a: &ClipPaintPropertyNode,
    b: &ClipPaintPropertyNode,
) -> PaintPropertyChangeType {
    if same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        && a.local_transform_space
            .lifecycle
            .same_node(&b.local_transform_space.lifecycle)
        && a.rect == b.rect
        && a.radii == b.radii
        && a.layout_clip_rects == b.layout_clip_rects
        && a.clip_path == b.clip_path
        && a.clip_path_error == b.clip_path_error
        && same_optional(&a.pixel_moving_filter, &b.pixel_moving_filter, |n| {
            &n.lifecycle
        })
    {
        PaintPropertyChangeType::Unchanged
    } else {
        PaintPropertyChangeType::ChangedOnlyValues
    }
}
fn effect_state_change(
    a: &EffectPaintPropertyNode,
    b: &EffectPaintPropertyNode,
) -> PaintPropertyChangeType {
    use PaintPropertyChangeType::*;
    if !same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        || !a
            .local_transform_space
            .lifecycle
            .same_node(&b.local_transform_space.lifecycle)
        || !same_optional(&a.output_clip, &b.output_clip, |n| &n.lifecycle)
        || a.blend_mode != b.blend_mode
        || a.filters != b.filters
        || a.filter_reference_box != b.filter_reference_box
        || a.filter_reference_scale != b.filter_reference_scale
        || a.backdrop_filters != b.backdrop_filters
        || a.backdrop_filter_has_reference != b.backdrop_filter_has_reference
        || a.backdrop_filter_bounds != b.backdrop_filter_bounds
        || a.compositor != b.compositor
        || !same_optional(&a.backdrop_filter_clip, &b.backdrop_filter_clip, |n| {
            &n.lifecycle
        })
        || a.isolates_blending != b.isolates_blending
        || a.has_mask != b.has_mask
        || a.is_mask != b.is_mask
        || a.self_or_ancestor_participates_in_view_transition
            != b.self_or_ancestor_participates_in_view_transition
        || a.direct_compositing_reasons != b.direct_compositing_reasons
    {
        return ChangedOnlyValues;
    }
    if a.opacity == b.opacity {
        return Unchanged;
    }
    // EffectPaintPropertyNode::State::IsOpacityChangeSimple. No fabricated
    // running compositor animation is inferred from the animation reason.
    if (a.opacity != 1.0 && b.opacity != 1.0)
        || (a
            .direct_compositing_reasons
            .contains(&"ActiveOpacityAnimation")
            && b.direct_compositing_reasons
                .contains(&"ActiveOpacityAnimation"))
    {
        ChangedOnlySimpleValues
    } else {
        ChangedOnlyValues
    }
}
fn scroll_state_change(
    a: &ScrollPaintPropertyNode,
    b: &ScrollPaintPropertyNode,
) -> PaintPropertyChangeType {
    if same_optional(&a.parent, &b.parent, |n| &n.lifecycle)
        && same_optional(&a.overflow_clip, &b.overflow_clip, |n| &n.lifecycle)
        && a.container_rect == b.container_rect
        && a.contents_rect == b.contents_rect
        && a.owner_client_id == b.owner_client_id
        && a.minimum_scroll_offset == b.minimum_scroll_offset
        && a.maximum_scroll_offset == b.maximum_scroll_offset
        && a.user_scrollable_horizontal == b.user_scrollable_horizontal
        && a.user_scrollable_vertical == b.user_scrollable_vertical
    {
        PaintPropertyChangeType::Unchanged
    } else {
        PaintPropertyChangeType::ChangedOnlyValues
    }
}

macro_rules! update_snapshot {
    ($self:ident, $node:ident, $key:expr, $old:ident, $change:ident, $map:ident) => {{
        if let Some(key) = $key {
            $self.seen.insert(key);
            if let Some($old) = $self.store.$map.get(&key) {
                let changed = $change($old, &$node);
                $node.id = $old.id;
                $node.lifecycle = $old.lifecycle.updated(changed);
            } else {
                $node.id = $self.store.next_id;
                $self.store.next_id += 1;
                $node.lifecycle = Default::default();
            }
            $self.store.changed = $self.store.changed.max($node.lifecycle.changed);
            if $node.lifecycle.changed != PaintPropertyChangeType::Unchanged {
                $self.store.change_markers_cleared = false;
            }
            let node = if let Some(old) = $self.store.$map.get(&key).filter(|old| {
                $node.lifecycle.changed == PaintPropertyChangeType::Unchanged
                    && old.lifecycle.node_changed() == PaintPropertyChangeType::Unchanged
                    && $node.same_snapshot_references(old)
            }) {
                old.clone()
            } else {
                Arc::new($node)
            };
            $self.store.$map.insert(key, node.clone());
            node
        } else {
            // No native owner (unsupported hand-built input): keep fresh
            // entities instead of inventing cross-frame client identities.
            $node.id = $self.store.next_id;
            $self.store.next_id += 1;
            $node.lifecycle = Default::default();
            $self.store.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
            Arc::new($node)
        }
    }};
}
impl PropertyNodeUpdater<'_> {
    fn transform_key(&self, input: &TransformPaintPropertyNode) -> Option<PaintPropertyKey> {
        self.stable_transform_keys
            .get(&input.id)
            .copied()
            .filter(|key| {
                self.store
                    .transforms
                    .get(key)
                    .is_some_and(|node| input.lifecycle.same_node(&node.lifecycle))
            })
            .or_else(|| self.raw_keys.get(&input.id).copied())
            .or_else(|| self.prior_raw_keys.get(&input.id).copied())
    }

    fn clip_key(&self, input: &ClipPaintPropertyNode) -> Option<PaintPropertyKey> {
        self.stable_clip_keys
            .get(&input.id)
            .copied()
            .filter(|key| {
                self.store
                    .clips
                    .get(key)
                    .is_some_and(|node| input.lifecycle.same_node(&node.lifecycle))
            })
            .or_else(|| self.raw_keys.get(&input.id).copied())
            .or_else(|| self.prior_raw_keys.get(&input.id).copied())
    }

    fn effect_key(&self, input: &EffectPaintPropertyNode) -> Option<PaintPropertyKey> {
        self.stable_effect_keys
            .get(&input.id)
            .copied()
            .filter(|key| {
                self.store
                    .effects
                    .get(key)
                    .is_some_and(|node| input.lifecycle.same_node(&node.lifecycle))
            })
            .or_else(|| self.raw_keys.get(&input.id).copied())
            .or_else(|| self.prior_raw_keys.get(&input.id).copied())
    }

    fn scroll_key(&self, input: &ScrollPaintPropertyNode) -> Option<PaintPropertyKey> {
        self.stable_scroll_keys
            .get(&input.id)
            .copied()
            .filter(|key| {
                self.store
                    .scrolls
                    .get(key)
                    .is_some_and(|node| input.lifecycle.same_node(&node.lifecycle))
            })
            .or_else(|| self.raw_keys.get(&input.id).copied())
            .or_else(|| self.prior_raw_keys.get(&input.id).copied())
    }

    pub fn state(&mut self, state: &mut PropertyTreeState) {
        state.transform = self.transform(&state.transform);
        state.clip = self.clip(&state.clip);
        state.effect = self.effect(&state.effect);
    }
    fn transform(
        &mut self,
        input: &Arc<TransformPaintPropertyNode>,
    ) -> Arc<TransformPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.transforms.get(&identity) {
            return node.clone();
        }
        // A retained clean subtree can still reference the preceding raw
        // snapshot of an ancestor whose value was rebuilt earlier in this
        // pass (most commonly ScrollTranslation during CullRectUpdater). The
        // native owner/role is the property slot identity. Once that slot has
        // been updated, every stale raw reference must bind to the new
        // canonical snapshot instead of replaying the old value over it.
        let key = self.transform_key(input);
        if let Some(key) = key {
            if self.seen.contains(&key) {
                return self
                    .store
                    .transforms
                    .get(&key)
                    .expect("seen transform property has a canonical snapshot")
                    .clone();
            }
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.transform(p));
        node.scroll = node.scroll.as_ref().map(|s| self.scroll(s));
        node.scroll_parent_scroll_translation = node
            .scroll_parent_scroll_translation
            .as_ref()
            .map(|s| self.transform(s));
        let node = update_snapshot!(self, node, key, old, transform_state_change, transforms);
        self.transforms.insert(identity, node.clone());
        node
    }
    fn clip(&mut self, input: &Arc<ClipPaintPropertyNode>) -> Arc<ClipPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.clips.get(&identity) {
            return node.clone();
        }
        let key = self.clip_key(input);
        if let Some(key) = key {
            if self.seen.contains(&key) {
                return self
                    .store
                    .clips
                    .get(&key)
                    .expect("seen clip property has a canonical snapshot")
                    .clone();
            }
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.clip(p));
        node.local_transform_space = self.transform(&node.local_transform_space);
        node.pixel_moving_filter = node.pixel_moving_filter.as_ref().map(|f| self.effect(f));
        let node = update_snapshot!(self, node, key, old, clip_state_change, clips);
        self.clips.insert(identity, node.clone());
        node
    }
    fn effect(&mut self, input: &Arc<EffectPaintPropertyNode>) -> Arc<EffectPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.effects.get(&identity) {
            return node.clone();
        }
        let key = self.effect_key(input);
        if let Some(key) = key {
            if self.seen.contains(&key) {
                return self
                    .store
                    .effects
                    .get(&key)
                    .expect("seen effect property has a canonical snapshot")
                    .clone();
            }
        }
        // A physical clip expander may still hold a pre-rebind effect snapshot.
        // Resolve its real node entity to the final logical effect ancestry.
        let source = self.effect_sources.get(&identity).unwrap_or(input);
        let mut node = (**source).clone();
        node.parent = node.parent.as_ref().map(|p| self.effect(p));
        node.local_transform_space = self.transform(&node.local_transform_space);
        node.output_clip = node.output_clip.as_ref().map(|c| self.clip(c));
        let node = update_snapshot!(self, node, key, old, effect_state_change, effects);
        self.effects.insert(identity, node.clone());
        node
    }
    fn scroll(&mut self, input: &Arc<ScrollPaintPropertyNode>) -> Arc<ScrollPaintPropertyNode> {
        if input.id == 0 {
            return input.clone();
        }
        let identity = PaintPropertyTrees::identity(&input.lifecycle);
        if let Some(node) = self.scrolls.get(&identity) {
            return node.clone();
        }
        let key = self.scroll_key(input);
        if let Some(key) = key {
            if self.seen.contains(&key) {
                return self
                    .store
                    .scrolls
                    .get(&key)
                    .expect("seen scroll property has a canonical snapshot")
                    .clone();
            }
        }
        let mut node = (**input).clone();
        node.parent = node.parent.as_ref().map(|p| self.scroll(p));
        node.overflow_clip = node.overflow_clip.as_ref().map(|c| self.clip(c));
        let node = update_snapshot!(self, node, key, old, scroll_state_change, scrolls);
        self.scrolls.insert(identity, node.clone());
        node
    }
    pub fn finish(self) {
        let before = self.store.transforms.len()
            + self.store.clips.len()
            + self.store.effects.len()
            + self.store.scrolls.len();
        self.store.transforms.retain(|k, _| self.seen.contains(k));
        self.store.clips.retain(|k, _| self.seen.contains(k));
        self.store.effects.retain(|k, _| self.seen.contains(k));
        self.store.scrolls.retain(|k, _| self.seen.contains(k));
        let after = self.store.transforms.len()
            + self.store.clips.len()
            + self.store.effects.len()
            + self.store.scrolls.len();
        if before != after {
            self.store.changed = PaintPropertyChangeType::NodeAddedOrRemoved;
        }
        self.store.change_markers_cleared =
            self.store.changed == PaintPropertyChangeType::Unchanged;
    }
}

#[cfg(test)]
mod retained_scroll_update_tests {
    use super::*;

    #[test]
    fn rebuilt_slot_rebinds_retained_stable_id_reference() {
        fn transform(id: u64, y: f64) -> Arc<TransformPaintPropertyNode> {
            let root = PropertyTreeState::default().transform;
            let mut node = (*root).clone();
            let mut matrix = TransformMatrix::default();
            matrix.values[13] = y;
            node.lifecycle = Default::default();
            node.id = id;
            node.parent = Some(root);
            node.matrix = matrix;
            Arc::new(node)
        }

        let key = PaintPropertyKey {
            owner: PaintPropertyOwner::NativeFragment {
                client: 9,
                fragment: 0,
            },
            role: PaintPropertyRole::ScrollTranslation,
        };
        let effects = BTreeMap::new();
        let no_prior_keys = BTreeMap::new();
        let mut store = PaintPropertyNodeStore::default();

        let mut first = PropertyTreeState {
            transform: transform(10, 0.0),
            ..PropertyTreeState::default()
        };
        let mut retained_raw = first.clone();
        let mut first_keys = BTreeMap::new();
        first_keys.insert(10, key);
        let mut updater = store.updater(&first_keys, &no_prior_keys, &effects);
        updater.state(&mut first);
        updater.finish();
        let mut retained = first.clone();
        let stable_id = retained.transform.id;

        let mut rebuilt = PropertyTreeState {
            transform: transform(20, -4000.0),
            ..PropertyTreeState::default()
        };
        // Raw builder labels and stable store labels use independent
        // allocators. Exercise the collision explicitly: this fresh raw node
        // has the same number as the retained stable reference above, but it
        // belongs to a different owner slot and has a different lifecycle.
        let other_key = PaintPropertyKey {
            owner: PaintPropertyOwner::NativeFragment {
                client: 10,
                fragment: 0,
            },
            role: PaintPropertyRole::Transform,
        };
        let mut collided = PropertyTreeState {
            transform: transform(stable_id, 123.0),
            ..PropertyTreeState::default()
        };
        let mut rebuilt_keys = BTreeMap::new();
        rebuilt_keys.insert(20, key);
        rebuilt_keys.insert(stable_id, other_key);
        let mut updater = store.updater(&rebuilt_keys, &first_keys, &effects);
        updater.state(&mut rebuilt);
        updater.state(&mut collided);
        updater.state(&mut retained);
        updater.state(&mut retained_raw);
        updater.finish();

        assert_eq!(rebuilt.transform.id, stable_id);
        assert!(Arc::ptr_eq(&rebuilt.transform, &retained.transform));
        assert_eq!(retained.transform.matrix.values[13], -4000.0);
        assert!(Arc::ptr_eq(&rebuilt.transform, &retained_raw.transform));
        assert_eq!(retained_raw.transform.matrix.values[13], -4000.0);
        assert!(!Arc::ptr_eq(&rebuilt.transform, &collided.transform));
        assert_ne!(collided.transform.id, stable_id);
        assert_eq!(collided.transform.matrix.values[13], 123.0);
    }

    #[test]
    fn clearing_resident_states_includes_nodes_without_native_owner_slots() {
        let root = PropertyTreeState::default();
        let clip = Arc::new(
            ClipPaintPropertyNode {
                geometry_mapper_cache: Default::default(),
                layout_clip_rects: Default::default(),
                lifecycle: Default::default(),
                id: 17,
                parent: Some(root.clip.clone()),
                local_transform_space: root.transform.clone(),
                rect: Some(PaintRect {
                    x: 2.0,
                    y: 3.0,
                    width: 40.0,
                    height: 50.0,
                }),
                radii: Default::default(),
                clip_path: None,
                clip_path_error: None,
                pixel_moving_filter: None,
            }
            .with_layout_clips(),
        );
        let mut state = PropertyTreeState { clip, ..root };
        let mut store = PaintPropertyNodeStore::default();
        let keys = BTreeMap::new();
        let effects = BTreeMap::new();
        let prior_keys = BTreeMap::new();
        let mut updater = store.updater(&keys, &prior_keys, &effects);
        updater.state(&mut state);
        updater.finish();
        let previous = state.clone();
        let (_, mut trees) = store.cleared_change_markers().unwrap();
        assert!(!trees.clips.contains_key(&state.clip.id));
        trees.clear_state_change_markers(&mut state);
        assert_eq!(state.clip.id, previous.clip.id);
        assert_eq!(state.clip.rect, previous.clip.rect);
        assert!(state.clip.lifecycle.same_node(&previous.clip.lifecycle));
        assert_eq!(
            state.clip.lifecycle.revision,
            previous.clip.lifecycle.revision
        );
        assert_eq!(
            state.clip.lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert_eq!(
            previous.clip.lifecycle.changed,
            PaintPropertyChangeType::NodeAddedOrRemoved
        );
        let shared = state.clip.clone();
        trees.clear_state_change_markers(&mut state);
        assert!(Arc::ptr_eq(&shared, &state.clip));
    }

    #[test]
    fn direct_scroll_rebinds_cross_links_without_invalidating_static_nodes() {
        let root = PropertyTreeState::default();
        let owner = PaintPropertyOwner::NativeFragment {
            client: 42,
            fragment: 0,
        };
        let key = |role| PaintPropertyKey { owner, role };
        let clean =
            || PaintPropertyNodeLifecycle::default().updated(PaintPropertyChangeType::Unchanged);
        let scroll = Arc::new(ScrollPaintPropertyNode {
            compositor: Default::default(),
            id: 1,
            owner_client_id: 42,
            lifecycle: clean(),
            parent: root.transform.scroll.clone(),
            overflow_clip: Some(root.clip.clone()),
            container_rect: PaintRect::default(),
            contents_rect: PaintRect::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset { x: 0.0, y: 100.0 },
            user_scrollable_horizontal: false,
            user_scrollable_vertical: true,
        });
        let translation = Arc::new(TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            scroll_parent_scroll_translation: None,
            id: 2,
            lifecycle: clean(),
            parent: Some(root.transform.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: Some(scroll.clone()),
            sticky_constraint: None,
            direct_compositing_reasons: Vec::new(),
        });
        let child = Arc::new(TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            scroll_parent_scroll_translation: None,
            id: 3,
            lifecycle: clean(),
            parent: Some(translation.clone()),
            matrix: TransformMatrix::default(),
            origin: [0.0; 3],
            flattens_inherited_transform: false,
            rendering_context_id: 0,
            backface_visibility: Default::default(),
            scroll: None,
            sticky_constraint: None,
            direct_compositing_reasons: Vec::new(),
        });
        let clip = Arc::new(
            ClipPaintPropertyNode {
                geometry_mapper_cache: Default::default(),
                layout_clip_rects: Default::default(),
                id: 4,
                lifecycle: clean().updated(PaintPropertyChangeType::ChangedOnlyValues),
                parent: Some(root.clip.clone()),
                local_transform_space: child.clone(),
                rect: None,
                radii: PaintCornerRadii::default(),
                clip_path: None,
                clip_path_error: None,
                pixel_moving_filter: None,
            }
            .with_layout_clips(),
        );
        let effect = Arc::new(
            EffectPaintPropertyNode {
                compositor: Default::default(),
                backdrop_filter_bounds: None,
                self_or_ancestor_participates_in_view_transition: false,
                id: 5,
                lifecycle: clean(),
                parent: Some(root.effect.clone()),
                local_transform_space: child.clone(),
                output_clip: Some(clip.clone()),
                filter_reference_box: None,
                filter_reference_scale: 1.0,
                opacity: 0.5,
                blend_mode: PaintBlendMode::kNormal,
                filters: Default::default(),
                backdrop_filters: Default::default(),
                backdrop_filter_has_reference: false,
                backdrop_filter_clip: None,
                isolates_blending: false,
                has_mask: false,
                is_mask: false,
                direct_compositing_reasons: Vec::new(),
            }
            .with_inherited_view_transition(),
        );
        let secondary_scroll = Arc::new(ScrollPaintPropertyNode {
            compositor: Default::default(),
            id: 6,
            owner_client_id: 43,
            lifecycle: clean(),
            parent: Some(scroll.clone()),
            overflow_clip: Some(clip.clone()),
            container_rect: PaintRect::default(),
            contents_rect: PaintRect::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset::default(),
            user_scrollable_horizontal: false,
            user_scrollable_vertical: false,
        });
        let mut store = PaintPropertyNodeStore::default();
        // This test bypasses updater() to construct marked resident slots.
        store.change_markers_cleared = false;
        store.transforms.insert(
            key(PaintPropertyRole::ScrollTranslation),
            translation.clone(),
        );
        store
            .transforms
            .insert(key(PaintPropertyRole::Transform), child.clone());
        store
            .clips
            .insert(key(PaintPropertyRole::OverflowClip), clip.clone());
        store
            .effects
            .insert(key(PaintPropertyRole::Effect), effect.clone());
        store.scrolls.insert(key(PaintPropertyRole::Scroll), scroll);
        store.scrolls.insert(
            PaintPropertyKey {
                owner: PaintPropertyOwner::NativeFragment {
                    client: 43,
                    fragment: 0,
                },
                role: PaintPropertyRole::Scroll,
            },
            secondary_scroll.clone(),
        );
        let (cleared, cleared_trees) = store.cleared_change_markers().unwrap();
        assert_eq!(cleared.changed, PaintPropertyChangeType::Unchanged);
        assert_eq!(cleared.next_id, store.next_id);
        assert_eq!(
            cleared_trees.clips[&4].lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert!(cleared_trees.clips[&4].lifecycle.same_node(&clip.lifecycle));
        assert_eq!(
            cleared_trees.clips[&4].lifecycle.revision,
            clip.lifecycle.revision
        );
        assert_eq!(
            clip.lifecycle.changed,
            PaintPropertyChangeType::ChangedOnlyValues,
            "clearing published markers leaves the old artifact immutable"
        );
        assert!(Arc::ptr_eq(
            cleared_trees.effects[&5].output_clip.as_ref().unwrap(),
            &cleared_trees.clips[&4]
        ));
        assert!(cleared.cleared_change_markers().is_none());
        let offsets = BTreeMap::from([(owner, Offset { x: 0.0, y: 12.5 })]);
        let (updated, trees) = store
            .retained_scroll_update(&offsets, &BTreeMap::new())
            .unwrap();
        assert_eq!(
            translation.matrix.values[13], 0.0,
            "old snapshot stays immutable"
        );
        assert_eq!(trees.transforms[&2].matrix.values[13], -12.5);
        assert!(trees.transforms[&2]
            .lifecycle
            .same_node(&translation.lifecycle));
        assert_eq!(
            trees.transforms[&2].lifecycle.revision,
            translation.lifecycle.revision + 1
        );
        assert_eq!(
            trees.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        for (before, after) in [
            (&child.lifecycle, &trees.transforms[&3].lifecycle),
            (&clip.lifecycle, &trees.clips[&4].lifecycle),
            (&effect.lifecycle, &trees.effects[&5].lifecycle),
            (&secondary_scroll.lifecycle, &trees.scrolls[&6].lifecycle),
        ] {
            assert_eq!(before, after);
            assert_eq!(after.changed, PaintPropertyChangeType::Unchanged);
        }
        assert!(Arc::ptr_eq(
            trees.transforms[&3].parent.as_ref().unwrap(),
            &trees.transforms[&2]
        ));
        assert!(Arc::ptr_eq(
            &trees.clips[&4].local_transform_space,
            &trees.transforms[&3]
        ));
        assert!(Arc::ptr_eq(
            trees.effects[&5].output_clip.as_ref().unwrap(),
            &trees.clips[&4]
        ));
        assert!(Arc::ptr_eq(
            trees.scrolls[&6].overflow_clip.as_ref().unwrap(),
            &trees.clips[&4]
        ));
        assert!(updated.is_scroll_only_update_from(&store));
        assert!(
            trees.transforms.contains_key(&0)
                && trees.clips.contains_key(&0)
                && trees.effects.contains_key(&0)
                && trees.scrolls.contains_key(&0)
        );
        let (cleared_updated, cleared_trees) = updated.cleared_change_markers().unwrap();
        assert_eq!(
            cleared_trees.transforms[&2].matrix,
            trees.transforms[&2].matrix
        );
        assert!(cleared_trees.transforms[&2]
            .lifecycle
            .same_node(&trees.transforms[&2].lifecycle));
        assert_eq!(
            cleared_trees.transforms[&2].lifecycle.revision,
            trees.transforms[&2].lifecycle.revision
        );
        assert_eq!(
            cleared_trees.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert_eq!(
            trees.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::ChangedOnlySimpleValues
        );
        assert!(cleared_updated.cleared_change_markers().is_none());
        let (_, unchanged) = updated
            .retained_scroll_update(&offsets, &BTreeMap::new())
            .unwrap();
        assert_eq!(
            unchanged.transforms[&2].lifecycle.revision,
            trees.transforms[&2].lifecycle.revision
        );
        assert_eq!(
            unchanged.transforms[&2].lifecycle.changed,
            PaintPropertyChangeType::Unchanged
        );
        assert!(store
            .retained_scroll_update(&BTreeMap::new(), &BTreeMap::new())
            .is_none());
        let mut new_owner = offsets.clone();
        new_owner.insert(
            PaintPropertyOwner::NativeFragment {
                client: 99,
                fragment: 0,
            },
            Offset { x: 0.0, y: 1.0 },
        );
        assert!(store
            .retained_scroll_update(&new_owner, &BTreeMap::new())
            .is_none());
        let constraint = StickyPositionPaintData {
            owner_client_id: 42,
            vertical: Some(layoutng_assembly::fragment_tree::StickyAxisPaintData {
                scroll_container_client_id: 42,
                min_inset: Some(0.0),
                ..Default::default()
            }),
            ..Default::default()
        };
        let sticky_node = Arc::new(TransformPaintPropertyNode {
            compositor: Default::default(),
            geometry_mapper_cache: Default::default(),
            scroll_parent_scroll_translation: None,
            id: 7,
            parent: Some(child.clone()),
            sticky_constraint: Some(constraint),
            ..(*child).clone()
        });
        store.transforms.insert(
            key(PaintPropertyRole::StickyTranslation),
            sticky_node.clone(),
        );
        let mut current = constraint;
        current.vertical.as_mut().unwrap().current_sticky_offset = 10.0;
        current
            .vertical
            .as_mut()
            .unwrap()
            .current_total_sticky_box_offset = 10.0;
        let mut sticky = BTreeMap::from([(owner, (Offset { x: 0.0, y: 10.0 }, current))]);
        let (_, trees) = store.retained_scroll_update(&offsets, &sticky).unwrap();
        assert_eq!(trees.transforms[&7].matrix.values[13], 10.0);
        assert_eq!(trees.transforms[&7].sticky_constraint, Some(current));
        assert_eq!(sticky_node.matrix.values[13], 0.0);
        assert_eq!(sticky_node.sticky_constraint, Some(constraint));
        assert!(trees.transforms[&7]
            .lifecycle
            .same_node(&sticky_node.lifecycle));
        assert!(Arc::ptr_eq(
            trees.transforms[&7].parent.as_ref().unwrap(),
            &trees.transforms[&3]
        ));
        sticky
            .get_mut(&owner)
            .unwrap()
            .1
            .vertical
            .as_mut()
            .unwrap()
            .min_inset = Some(2.0);
        assert!(store.retained_scroll_update(&offsets, &sticky).is_none());
        assert!(store
            .retained_scroll_update(&offsets, &BTreeMap::new())
            .is_none());
    }
}
