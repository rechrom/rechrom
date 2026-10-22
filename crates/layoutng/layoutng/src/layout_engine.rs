#![allow(non_snake_case)]

use foundation::{
    DynamicTo, InitStringStatics, LayoutHeapScope, Length, TextEmphasisMark, UnsupportedLayout,
};
use layoutng_fragment_tree::fragment_tree::pre_paint_revision::PrePaintRevisionTracker;
use layoutng_fragment_tree::fragment_tree::FragmentNode;
use layoutng_fragment_tree::layout_result::EStatus;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use crate::internal::layout_object_builder::LayoutObjectTree;
use crate::{caret::FrameCaret, editing_state::LayoutEditingState};
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;

use crate::internal::block_node::BlockNode;
use crate::internal::boundary::layout_boundary::{
    ResolveLogicalParentOccurrences, ScrollPaintPropertyIndex,
};
use crate::internal::layout_algorithm_set::LayoutAlgorithmSet;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_input::{ConstraintSpace, NodeKind};
use crate::internal::layout_object::LayoutObject;
use crate::internal::layout_object_factory_set::LayoutObjectFactorySet;
use crate::internal::layout_pass_scope::{LayoutObjectFactoryScope, LayoutPassScope};
use crate::layout_assembly::LayoutAssembly;
use crate::layout_boundary_support::{LayoutBoundarySupport, LayoutUpdateKind};

#[path = "layout_tree_update.rs"]
mod layout_tree_update;
pub use layout_tree_update::LayoutTreeUpdate;
#[path = "layout_output.rs"]
mod layout_output;
pub use layout_output::{LayoutMetric, LayoutNode, LayoutOutput, LayoutRect, LayoutScrollRange};

// Rust panic payloads retain the two standard C++ failure categories emitted
// by this entry point; unsupported modules use foundation::UnsupportedLayout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutEngineError {
    InvalidArgument(&'static str),
    RuntimeError(&'static str),
}

fn UpdateFragmentPaintStyle(
    fragment: &mut FragmentNode,
    node_id: u64,
    style: &crate::internal::layout_input::PaintStyle,
    found: &mut bool,
) {
    if fragment.node_id == node_id {
        let mut current = style.clone();
        if let Some(clip) = current
            .clip_path
            .as_mut()
            .filter(|clip| clip.reference_target_id.is_some())
        {
            let Some(previous) = fragment.paint.style.clip_path.as_ref().filter(|previous| {
                previous.reference_target_id == clip.reference_target_id
                    && previous.reference_scope == clip.reference_scope
            }) else {
                // A newly selected resource needs the ordinary native export
                // to establish its own viewport, even if box geometry is clean.
                return;
            };
            clip.reference_viewport = previous.reference_viewport;
        }
        for (index, layer) in current.mask_images.iter_mut().enumerate() {
            let Some(reference) = layer
                .svg_reference
                .as_mut()
                .filter(|r| r.target_id.is_some())
            else {
                continue;
            };
            let Some(previous) = fragment
                .paint
                .style
                .mask_images
                .get(index)
                .and_then(|l| l.svg_reference.as_ref())
                .filter(|p| {
                    p.target_id == reference.target_id
                        && p.scope == reference.scope
                        && p.id == reference.id
                })
            else {
                return;
            };
            reference.viewport = previous.viewport;
        }
        fragment.paint.style = current;
        *found = true;
    }
    for child in &mut fragment.children {
        UpdateFragmentPaintStyle(child, node_id, style, found);
    }
}

fn SameOptionalArc<T: ?Sized>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn SameImageGeometry(
    a: &crate::internal::layout_input::PaintImage,
    b: &crate::internal::layout_input::PaintImage,
) -> bool {
    a.id == b.id
        && a.paint_image_id == b.paint_image_id
        && a.width == b.width
        && a.height == b.height
        && a.resolution_scale == b.resolution_scale
        && a.orientation == b.orientation
        && a.document_container_key == b.document_container_key
}

fn SameImageCatalogGeometry(
    a: &[crate::internal::layout_input::PaintImage],
    b: &[crate::internal::layout_input::PaintImage],
) -> bool {
    a.len() == b.len()
        && a.iter().all(|left| {
            b.iter()
                .find(|right| right.id == left.id)
                .is_some_and(|right| SameImageGeometry(left, right))
        })
}

/// ConstraintSpace carries both geometry and shared host services. DOM emits
/// the resolved value before every layout pass, so compare it with the
/// resident value before invalidating a retained fragment snapshot. Provider
/// identity is part of the comparison: replacing a service is a real input
/// change even when its current answer happens to match.
fn SameConstraintScalars(a: &ConstraintSpace, b: &ConstraintSpace) -> bool {
    a.available_size == b.available_size
        && a.fragmentainer_block_size == b.fragmentainer_block_size
        && a.compatibility_mode == b.compatibility_mode
        && a.printing == b.printing
        && a.vertical_scroll_enforced == b.vertical_scroll_enforced
        && a.scrollbar_theme == b.scrollbar_theme
        && a.minimum_font_physical_size == b.minimum_font_physical_size
        && a.device_pixel_ratio == b.device_pixel_ratio
        && a.preferred_color_scheme == b.preferred_color_scheme
        && a.document_image_container == b.document_image_container
        && a.layout_zoom_factor == b.layout_zoom_factor
        && a.canvas_draw_element_enabled == b.canvas_draw_element_enabled
        && a.prefer_non_composited_scrolling_enabled == b.prefer_non_composited_scrolling_enabled
        && a.viewport == b.viewport
        && a.control_theme == b.control_theme
}

fn SameConstraints(a: &ConstraintSpace, b: &ConstraintSpace) -> bool {
    SameConstraintScalars(a, b)
        && SameOptionalArc(&a.hyphenation, &b.hyphenation)
        && SameOptionalArc(&a.phrase_break, &b.phrase_break)
        && SameOptionalArc(&a.text_transform, &b.text_transform)
        && SameOptionalArc(&a.font_backend_factory, &b.font_backend_factory)
        && a.font_queries == b.font_queries
        && a.fonts == b.fonts
        // Decoded pixels, animation metadata and revision are Paint resource
        // inputs. Chromium invalidates layout only when intrinsic image
        // geometry changes; Page publishes content-only mutations separately.
        && SameImageCatalogGeometry(&a.images, &b.images)
}

fn CountFragmentNode(node: &FragmentNode, node_id: u64, count: &mut usize) {
    *count += usize::from(node.node_id == node_id);
    for child in &node.children {
        CountFragmentNode(child, node_id, count);
    }
}

fn MaxFragmentInstance(node: &FragmentNode) -> u64 {
    node.children
        .iter()
        .map(MaxFragmentInstance)
        .fold(node.fragment_instance_id, u64::max)
}

fn MaxFragmentainerInstance(node: &FragmentNode) -> u64 {
    node.children
        .iter()
        .map(MaxFragmentainerInstance)
        .fold(node.fragmentainer_instance_id, u64::max)
}

fn SameFragmentIdentity(a: &FragmentNode, b: &FragmentNode) -> bool {
    a.node_id == b.node_id
        && a.kind == b.kind
        && a.text_start == b.text_start
        && a.text_end == b.text_end
}

fn AdoptRetainedFragmentIds(
    old: &FragmentNode,
    new: &mut FragmentNode,
    ids: &mut HashMap<u64, u64>,
    next_fragment: &mut u64,
    next_fragmentainer: &mut u64,
) {
    let exported_id = new.fragment_instance_id;
    if SameFragmentIdentity(old, new) {
        new.fragment_instance_id = old.fragment_instance_id;
        new.fragmentainer_instance_id = old.fragmentainer_instance_id;
        new.paint.logical_tree_order = old.paint.logical_tree_order;
    } else {
        new.fragment_instance_id = *next_fragment;
        *next_fragment = next_fragment.saturating_add(1);
        if new.fragmentainer_instance_id != 0 {
            new.fragmentainer_instance_id = *next_fragmentainer;
            *next_fragmentainer = next_fragmentainer.saturating_add(1);
        }
    }
    ids.insert(exported_id, new.fragment_instance_id);

    let mut claimed = vec![false; old.children.len()];
    for (index, child) in new.children.iter_mut().enumerate() {
        let old_index = old
            .children
            .get(index)
            .filter(|old_child| SameFragmentIdentity(old_child, child))
            .map(|_| index)
            .or_else(|| {
                old.children
                    .iter()
                    .enumerate()
                    .position(|(old_index, old_child)| {
                        !claimed[old_index] && SameFragmentIdentity(old_child, child)
                    })
            });
        if let Some(old_index) = old_index {
            claimed[old_index] = true;
            AdoptRetainedFragmentIds(
                &old.children[old_index],
                child,
                ids,
                next_fragment,
                next_fragmentainer,
            );
        } else {
            AssignFreshFragmentIds(child, ids, next_fragment, next_fragmentainer);
        }
    }
}

fn AssignFreshFragmentIds(
    node: &mut FragmentNode,
    ids: &mut HashMap<u64, u64>,
    next_fragment: &mut u64,
    next_fragmentainer: &mut u64,
) {
    let exported_id = node.fragment_instance_id;
    node.fragment_instance_id = *next_fragment;
    *next_fragment = next_fragment.saturating_add(1);
    if node.fragmentainer_instance_id != 0 {
        node.fragmentainer_instance_id = *next_fragmentainer;
        *next_fragmentainer = next_fragmentainer.saturating_add(1);
    }
    ids.insert(exported_id, node.fragment_instance_id);
    for child in &mut node.children {
        AssignFreshFragmentIds(child, ids, next_fragment, next_fragmentainer);
    }
}

fn RemapLogicalParents(node: &mut FragmentNode, ids: &HashMap<u64, u64>) {
    if let Some(parent) = node.paint.logical_parent_fragment_instance_id {
        if let Some(remapped) = ids.get(&parent) {
            node.paint.logical_parent_fragment_instance_id = Some(*remapped);
        }
    }
    for child in &mut node.children {
        RemapLogicalParents(child, ids);
    }
}

fn ReplaceFragmentNode(
    node: &mut FragmentNode,
    node_id: u64,
    replacement: &mut FragmentNode,
    next_fragment: &mut u64,
    next_fragmentainer: &mut u64,
) -> bool {
    if node.node_id == node_id {
        let old_offset = node.offset;
        let old_logical_parent_node = node.paint.logical_parent_node_id;
        let old_logical_parent_fragment = node.paint.logical_parent_fragment_instance_id;
        let mut ids = HashMap::new();
        AdoptRetainedFragmentIds(
            node,
            replacement,
            &mut ids,
            next_fragment,
            next_fragmentainer,
        );
        RemapLogicalParents(replacement, &ids);
        replacement.offset = old_offset;
        replacement.paint.logical_parent_node_id = old_logical_parent_node;
        replacement.paint.logical_parent_fragment_instance_id = old_logical_parent_fragment;
        *node = std::mem::take(replacement);
        return true;
    }
    for child in &mut node.children {
        if ReplaceFragmentNode(
            child,
            node_id,
            replacement,
            next_fragment,
            next_fragmentainer,
        ) {
            return true;
        }
    }
    false
}

// cpp: layoutng/layout_engine.h:19-31
/// Mutations of the engine-owned input tree. TreeUpdate reconciles one complete
/// DOM projection using the existing native insert/style/text/remove logic;
/// the callback receives only the attachment methods for this transaction.
pub enum LayoutMutation<'a> {
    Constraints(&'a ConstraintSpace),
    TreeUpdate(&'a mut dyn FnMut(&mut LayoutTreeUpdate<'_>)),
    /// No pending DOM attachment inputs. A fresh engine rejects this so the
    /// caller still supplies a complete projection when no resident tree exists.
    RetainTree,
    /// Element::RebuildLayoutTree attachment rooted at one dirty element.
    /// The parent is borrowed only for this synchronous attachment callback.
    SubtreeUpdate {
        node_id: u64,
        parent_id: u64,
        update: &'a mut dyn FnMut(
            &mut LayoutTreeUpdate<'_>,
            &mut super::internal::layout_object::LayoutObject,
        ),
    },
    NodeInput {
        node_id: u64,
        style: &'a crate::internal::layout_input::ComputedStyle,
        element: Option<crate::internal::layout_input::ElementData>,
    },
    PaintStyle {
        node_id: u64,
        style: &'a NativeComputedStyle,
        projected: &'a crate::internal::paint_input::PaintStyleData,
        clip_resource_version: crate::internal::paint_input::SvgClipResourceVersion,
    },
    /// Complete local paint-only input, including native theme metadata.
    PaintInput {
        node_id: u64,
        style: &'a crate::internal::layout_input::ComputedStyle,
        element: Option<crate::internal::layout_input::ElementData>,
    },
    ReplaceTree(LayoutObjectTree),
    ScrollOffset {
        node_id: u64,
        offset: crate::internal::layout_input::Offset,
    },
    Editing {
        node_id: u64,
    },
    /// Selection/caret range paint changed while layout geometry stayed valid.
    SelectionPaint {
        node_id: u64,
    },
    EditingState(Rc<LayoutEditingState>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScrollLayoutUpdate {
    RetainedGeometry,
    RefreshedLayout,
}

/// Immutable native content-box geometry. This layout output does not depend
/// on StyleEngine or its query AST.
#[derive(Clone, Debug, PartialEq)]
pub struct ContainerQuerySizeSnapshot {
    pub node_id: u64,
    pub width: f64,
    pub height: f64,
}

/// Move-only access to Layout's resident fragment result for one Paint pass.
/// Moving this value transfers only an `Rc`; it never clones the fragment tree.
pub struct LayoutPaintOutput {
    fragments: Rc<FragmentNode>,
}

impl LayoutPaintOutput {
    pub fn Fragments(&self) -> &FragmentNode {
        &self.fragments
    }

    /// Page-level resource preparation may update the resident fragment value
    /// before Paint consumes it. Copy-on-write occurs only when a genuinely
    /// older published frame still retains the same tree.
    pub fn FragmentsMut(&mut self) -> &mut Rc<FragmentNode> {
        &mut self.fragments
    }
}

/// Opaque client set produced by Paint after its artifact is complete.
/// Layout consumes it to advance native DisplayItemClient state.
pub struct PaintClientCommit {
    used_clients: HashSet<u64>,
}

impl PaintClientCommit {
    pub fn FromClientSet(used_clients: HashSet<u64>) -> Self {
        Self { used_clients }
    }
}

/// Interface implemented by a Paint engine output. It lets Layout consume the
/// opaque commit receipt without depending on the Paint crate or its artifact.
pub trait PaintCommitOutput {
    type Artifact;

    fn IntoLayoutCommit(self) -> (Self::Artifact, PaintClientCommit);
}

/// Owns the resident native input tree, its constraints and exported result.
/// DOM adapters provide mutations; Paint consumes only the fragment output.
pub struct LayoutEngine {
    /// Rust boundary output, populated by a successful Layout(). The native
    /// layout algorithm and LayoutObject lifetime remain unchanged.
    fragments: Option<Rc<FragmentNode>>,
    output_kind: LayoutUpdateKind,
    tree_: Option<LayoutObjectTree>,
    constraints_: ConstraintSpace,
    pre_paint_inputs_: PrePaintRevisionTracker,
    scroll_paint_index_: Option<ScrollPaintPropertyIndex>,
    pending_scroll_ids_: HashSet<u64>,
    pending_paint_ids_: HashSet<u64>,
    pending_inline_ink_ids_: HashSet<u64>,
    // A single Element::RebuildLayoutTree root may reuse the rest of the
    // exported fragment tree after native layout. Multiple or unrelated
    // mutations clear this proof and take the ordinary complete export.
    pending_subtree_export_id_: Option<u64>,
    algorithms_: LayoutAlgorithmSet,
    objects_: LayoutObjectFactorySet,
    boundary_: LayoutBoundarySupport,
}

impl LayoutEngine {
    // cpp: layoutng/layout_engine.h:21-23
    pub fn new(assembly: &LayoutAssembly) -> Self {
        Self {
            fragments: None,
            output_kind: LayoutUpdateKind::Paint,
            tree_: None,
            constraints_: ConstraintSpace::default(),
            pre_paint_inputs_: PrePaintRevisionTracker::new(),
            scroll_paint_index_: None,
            pending_scroll_ids_: HashSet::new(),
            pending_paint_ids_: HashSet::new(),
            pending_inline_ink_ids_: HashSet::new(),
            pending_subtree_export_id_: None,
            algorithms_: assembly.algorithms,
            objects_: assembly.objects,
            boundary_: assembly.boundary,
        }
    }

    /// A clean resident tree may prove that an attribute changed no geometry.
    /// Uses the same canonical style and metadata admission as paint updates,
    /// without changing native objects or exported paint/resource bindings.
    pub fn GeometryInputMatches(
        &self,
        id: u64,
        style: &crate::internal::layout_input::ComputedStyle,
        element: &Option<crate::internal::layout_input::ElementData>,
    ) -> bool {
        self.tree_.as_ref().is_some_and(|tree| {
            !tree.Root().NeedsLayout() && tree.GeometryInputMatches(id, style, element)
        })
    }

    /// Apply input changes without running layout. Native synchronization
    /// preserves client identities and invalidates layout exactly as before.
    pub fn ApplyMutation(&mut self, mutation: LayoutMutation<'_>) -> bool {
        // EmitConstraints is deliberately unconditional at the DOM boundary:
        // compatibility mode is resolved there. Do not let an identical
        // value erase a valid local-layout export proof on every frame.
        if let LayoutMutation::Constraints(space) = &mutation {
            let same = SameConstraints(&self.constraints_, space);
            let mut trace = browser_tracing::span("layout", "LayoutEngine.Constraints");
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
        // Native topology, style, constraints and layout results may change at
        // all other mutation entries. Never keep borrowed native pointers or
        // exported paths across them, even if the mutation later proves a noop.
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
            self.scroll_paint_index_ = None;
            self.pending_scroll_ids_.clear();
        }
        if !matches!(
            &mutation,
            LayoutMutation::SubtreeUpdate { .. }
                | LayoutMutation::SelectionPaint { .. }
                | LayoutMutation::EditingState(_)
        ) {
            self.pending_subtree_export_id_ = None;
        }
        match mutation {
            LayoutMutation::Constraints(space) => {
                self.constraints_ = space.clone();
                self.fragments = None;
                true
            }
            LayoutMutation::TreeUpdate(update) => {
                let mut trace = browser_tracing::span("layout", "LayoutEngine.TreeUpdate");
                self.fragments = None;
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
                true
            }
            LayoutMutation::RetainTree => {
                let mut trace = browser_tracing::span("layout", "LayoutEngine.RetainTree");
                let retained = self.tree_.as_ref().is_some_and(|tree| tree.HasRoot());
                trace.set("retained", retained as u8 as f64);
                retained
            }
            LayoutMutation::SubtreeUpdate {
                node_id,
                parent_id,
                update,
            } => {
                let mut trace = browser_tracing::span("layout", "LayoutEngine.SubtreeUpdate");
                trace.set("node_id", node_id as f64);
                trace.set("had_fragments", self.fragments.is_some() as u8 as f64);
                let applied = self.tree_.as_mut().is_some_and(|tree| {
                    tree.UpdateSubtree(node_id, parent_id, |tree, parent| {
                        update(&mut LayoutTreeUpdate::new(tree), parent);
                    })
                });
                if applied {
                    self.pending_subtree_export_id_ =
                        match (self.pending_subtree_export_id_, self.fragments.is_some()) {
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
                }
                trace.set(
                    "pending_export",
                    self.pending_subtree_export_id_.is_some() as u8 as f64,
                );
                trace.set("applied", applied as u8 as f64);
                applied
            }
            LayoutMutation::NodeInput {
                node_id,
                style,
                element,
            } => {
                let mut trace = browser_tracing::span("layout", "LayoutEngine.NodeInput");
                trace.set("node_id", node_id as f64);
                let applied = self
                    .tree_
                    .as_mut()
                    .is_some_and(|tree| tree.UpdateNodeInput(node_id, style, element));
                if applied {
                    self.fragments = None;
                }
                trace.set("applied", applied as u8 as f64);
                applied
            }
            LayoutMutation::PaintStyle {
                node_id,
                style,
                projected,
                clip_resource_version,
            } => {
                let binding = self
                    .tree_
                    .as_mut()
                    .and_then(|tree| tree.UpdatePaintStyle(node_id, style, clip_resource_version));
                if let Some(binding) = binding.as_ref() {
                    if let Some(fragments) = &mut self.fragments {
                        let mut found = false;
                        let paint_style = crate::internal::boundary::native_paint_style::ExportStyleWithProjectedSvg(
                                style,
                                binding.image_resource_id,
                                projected,
                                &binding.svg_clip_resources,
                            );
                        UpdateFragmentPaintStyle(
                            Rc::make_mut(fragments),
                            node_id,
                            &paint_style,
                            &mut found,
                        );
                        if !found {
                            self.fragments = None;
                            return false;
                        }
                    }
                }
                binding.is_some()
            }
            LayoutMutation::PaintInput {
                node_id,
                style,
                element,
            } => {
                if self.fragments.is_none() {
                    return false;
                }
                let update = self
                    .tree_
                    .as_mut()
                    .and_then(|tree| tree.UpdatePaintInput(node_id, style, element));
                if let Some(update) = update {
                    self.pending_paint_ids_.extend(update.sources);
                    self.pending_inline_ink_ids_
                        .extend(update.inline_ink_changes);
                    true
                } else {
                    false
                }
            }
            LayoutMutation::ReplaceTree(tree) => {
                self.fragments = None;
                self.pre_paint_inputs_.invalidate();
                self.tree_ = Some(tree);
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
                self.fragments = None;
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

    /// Read-only access for native lifecycle consumers. The engine keeps sole
    /// ownership; callers cannot retain an Rc that extends the tree lifetime.
    pub fn GetLayoutTree(&self) -> Option<&LayoutObjectTree> {
        self.tree_.as_ref()
    }

    /// Chromium supplies laid-out content sizes to SizeContainerChanged.
    /// Native content boxes exclude borders, padding and scrollbars;
    /// FragmentNode::content_size instead describes scrollable overflow.
    pub fn ContainerQuerySizes(&self) -> Vec<ContainerQuerySizeSnapshot> {
        let Some(tree) = self.tree_.as_ref() else {
            return Vec::new();
        };
        // This traversal uses LayoutObject's translated virtual child dispatch
        // through NextInPreOrder(). Keep the same installed object factories
        // available here as during Layout(), including SVG root children.
        let _object_factory_scope = LayoutObjectFactoryScope::new(&self.objects_);
        let root = tree.Root() as *const LayoutObject;
        let mut current = root as *mut LayoutObject;
        let mut result = Vec::new();
        while let Some(object) = unsafe { current.as_ref() } {
            let real_element = unsafe { object.GetNode().as_ref() }
                .is_some_and(|node| !node.InputIsStyleGenerated());
            if real_element && object.StyleRef().ContainerType() & 3 != 0 && !object.NeedsLayout() {
                let box_ = DynamicTo::<LayoutBox>(current);
                if let Some(box_) =
                    unsafe { box_.as_ref() }.filter(|b| b.PhysicalFragmentCount() != 0)
                {
                    let size = box_.PhysicalContentBoxRect().size;
                    // style_engine.cc:3888-3890 adjusts layout's physical size
                    // back to CSS pixels before evaluating container features.
                    let zoom = object.StyleRef().EffectiveZoom() as f64;
                    result.push(ContainerQuerySizeSnapshot {
                        node_id: object.OwnerNodeIdDefault() as u64,
                        width: size.width.ToDouble().max(0.0) / zoom,
                        height: size.height.ToDouble().max(0.0) / zoom,
                    });
                }
            }
            current = object.NextInPreOrder(root);
        }
        result
    }

    pub fn GetConstraints(&self) -> &ConstraintSpace {
        &self.constraints_
    }

    pub fn HasPendingSubtreeExport(&self) -> bool {
        self.pending_subtree_export_id_.is_some() && self.fragments.is_some()
    }

    fn RootPointer(&mut self) -> *mut LayoutObject {
        self.tree_
            .as_mut()
            .expect("apply a tree mutation before Layout")
            .RootMut() as *mut LayoutObject
    }

    /// Publish the owned fragment snapshot only after native layout and export
    /// succeed. A rejected pass cannot leave the previous snapshot visible.
    pub fn Layout(&mut self) {
        self.UpdateLayoutOutput(LayoutUpdateKind::Paint);
    }

    /// Flush native layout and publish only CSSOM geometry. This output has
    /// no paint certification and cannot enter retained paint update paths.
    pub fn UpdateLayout(&mut self) {
        self.UpdateLayoutOutput(LayoutUpdateKind::Layout);
    }
    /// A borrow of resident output. Consumers receive values and scoped
    /// fragment access, never the engine's native object ownership.
    pub fn Output(&self) -> Option<LayoutOutput<'_>> {
        let tree = self.tree_.as_ref()?;
        let root = tree.Root();
        if root.NeedsLayout() {
            return None;
        }
        let box_ = DynamicTo::<LayoutBox>(root as *const _);
        let box_ = unsafe { box_.as_ref() }?;
        (!box_.GetLayoutResults().is_empty()).then_some(LayoutOutput { engine: self })
    }

    fn UpdateLayoutOutput(&mut self, purpose: LayoutUpdateKind) {
        let mut trace = browser_tracing::span("layout", "LayoutEngine.Layout");
        self.scroll_paint_index_ = None;
        self.pending_scroll_ids_.clear();
        let root = self.RootPointer();
        let subtree_id = (purpose == LayoutUpdateKind::Paint)
            .then_some(self.pending_subtree_export_id_)
            .flatten();
        self.pending_subtree_export_id_ = None;
        let mut retained_fragments = subtree_id.and_then(|_| self.fragments.take());
        self.fragments = None;
        self.output_kind = purpose;
        trace.set(
            "measurement",
            (purpose == LayoutUpdateKind::Layout) as u8 as f64,
        );
        // While layout/export runs, the engine has no certified input catalog.
        // Unwinding leaves it empty; a subsequent success receives fresh
        // globally unique revisions instead of reusing a failed pass's basis.
        let mut inputs = std::mem::take(&mut self.pre_paint_inputs_);
        let exported = retained_fragments.is_none();
        let layout_result = self.RunLayout(
            unsafe { &mut *root },
            &self.constraints_,
            purpose == LayoutUpdateKind::Paint && exported,
            purpose,
        );
        let mut fragments = if exported {
            layout_result
        } else if layout_result.is_none() {
            let target = subtree_id.unwrap();
            retained_fragments.take().and_then(|mut retained| {
                if self.ReplaceExportedSubtree(root, target, Rc::make_mut(&mut retained)) {
                    trace.set("targeted_subtree_export", 1.0);
                    Some(Rc::try_unwrap(retained).unwrap_or_else(|shared| (*shared).clone()))
                } else {
                    trace.set("targeted_subtree_fallback", 1.0);
                    let assembly = LayoutAssembly {
                        algorithms: self.algorithms_,
                        objects: self.objects_,
                        boundary: self.boundary_,
                    };
                    Some(assembly.ExportFragments(unsafe { &mut *root }, &self.constraints_))
                }
            })
        } else {
            layout_result
        };
        if let Some(mut fragments) = fragments.take() {
            if purpose == LayoutUpdateKind::Paint {
                let _trace = browser_tracing::span("prepaint", "LayoutEngine.CertifyPaintInputs");
                inputs.update(&mut fragments);
            }
            if purpose == LayoutUpdateKind::Paint {
                self.scroll_paint_index_ = Some({
                    let _trace = browser_tracing::span("layout", "LayoutEngine.BuildScrollIndex");
                    // RunLayout's temporary pass has ended. Walking native SVG
                    // children still needs this engine's real virtual factories.
                    let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
                    ScrollPaintPropertyIndex::new(unsafe { &mut *root }, &fragments)
                });
            }
            self.fragments = Some(Rc::new(fragments));
            self.pre_paint_inputs_ = inputs;
            trace.set("published", 1.0);
        }
    }

    fn ReplaceExportedSubtree(
        &mut self,
        document_root: *mut LayoutObject,
        node_id: u64,
        retained: &mut FragmentNode,
    ) -> bool {
        let Some(tree) = self.tree_.as_mut() else {
            return false;
        };
        let Some(object) = tree.OutputNodeMut(node_id) else {
            return false;
        };
        let box_ = DynamicTo::<LayoutBox>(object as *mut LayoutObject);
        if box_.is_null()
            || unsafe { &*box_ }.GetLayoutResults().is_empty()
            || unsafe { &*box_ }.GetLayoutResult(0).is_null()
            || object.NeedsLayout()
        {
            return false;
        }
        let mut occurrences = 0usize;
        CountFragmentNode(retained, node_id, &mut occurrences);
        if occurrences != 1 {
            return false;
        }
        let assembly = LayoutAssembly {
            algorithms: self.algorithms_,
            objects: self.objects_,
            boundary: self.boundary_,
        };
        let mut replacement = unsafe {
            assembly.ExportFragmentSubtree(
                document_root,
                object as *mut LayoutObject,
                &self.constraints_,
            )
        };
        let mut next_fragment = MaxFragmentInstance(retained).saturating_add(1);
        let mut next_fragmentainer = MaxFragmentainerInstance(retained).saturating_add(1);
        let replaced = ReplaceFragmentNode(
            retained,
            node_id,
            &mut replacement,
            &mut next_fragment,
            &mut next_fragmentainer,
        );
        if replaced {
            // ExportFragmentSubtree resolves logical parents only within its
            // temporary subtree. Re-resolve every occurrence after insertion
            // so retained and replacement nodes refer to instance ids in the
            // same complete snapshot.
            ResolveLogicalParentOccurrences(retained);
        }
        replaced
    }

    /// Re-export paint inputs from the resident layout result without running
    /// layout. Blink's paint-only style path updates ObjectPaintProperties from
    /// the existing layout geometry; transform/opacity animation samples must
    /// not turn into a full layout solely to refresh the fragment snapshot.
    ///
    /// The DOM adapter may conservatively classify an update as paint-only.
    /// The native tree is the final guard: if synchronization marked any
    /// layout object dirty, reject this path and let the caller run Layout().
    pub fn ExportPaintOnly(&mut self) -> bool {
        let mut trace = browser_tracing::span("layout", "LayoutEngine.ExportPaintOnly");
        if self.tree_.is_none() {
            trace.set("rejected", 1.0);
            return false;
        }
        let root = self.RootPointer();
        let root_box = DynamicTo::<LayoutBox>(root);
        if root_box.is_null()
            || unsafe { &*root }.NeedsLayout()
            || unsafe { &*root_box }.GetLayoutResults().is_empty()
            || unsafe { &*root_box }.GetLayoutResult(0).is_null()
        {
            trace.set("native_needs_layout", 1.0);
            return false;
        }
        let assembly = LayoutAssembly {
            algorithms: self.algorithms_,
            objects: self.objects_,
            boundary: self.boundary_,
        };
        let mut fragments = assembly.ExportFragments(unsafe { &mut *root }, &self.constraints_);
        let mut inputs = std::mem::take(&mut self.pre_paint_inputs_);
        {
            let _trace = browser_tracing::span("prepaint", "LayoutEngine.CertifyPaintInputs");
            inputs.update(&mut fragments);
        }
        self.scroll_paint_index_ = Some({
            let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
            ScrollPaintPropertyIndex::new(unsafe { &mut *root }, &fragments)
        });
        self.pending_scroll_ids_.clear();
        self.fragments = Some(Rc::new(fragments));
        self.output_kind = LayoutUpdateKind::Paint;
        self.pre_paint_inputs_ = inputs;
        trace.set("published", 1.0);
        true
    }

    /// Publish paint-only values already applied to the retained fragment
    /// snapshot. Chromium updates FragmentData/ObjectPaintProperties in place;
    /// this immutable Rust boundary uses copy-on-write once per frame, then
    /// refreshes the same subtree revision proof consumed by PrePaint.
    pub fn ExportTargetedPaintOnly(&mut self) -> bool {
        if self.output_kind != LayoutUpdateKind::Paint {
            return false;
        }
        let mut trace = browser_tracing::span("layout", "LayoutEngine.ExportTargetedPaintOnly");
        let Some(fragments) = &mut self.fragments else {
            trace.set("rejected", 1.0);
            return false;
        };
        let root = self
            .tree_
            .as_ref()
            .map(LayoutObjectTree::Root)
            .expect("targeted paint update requires a layout tree");
        let root_box = DynamicTo::<LayoutBox>(root as *const _ as *mut LayoutObject);
        if root_box.is_null()
            || root.NeedsLayout()
            || unsafe { &*root_box }.GetLayoutResults().is_empty()
            || unsafe { &*root_box }.GetLayoutResult(0).is_null()
        {
            trace.set("native_needs_layout", 1.0);
            return false;
        }
        self.pre_paint_inputs_.update(Rc::make_mut(fragments));
        trace.set("published", 1.0);
        true
    }

    /// Publish ordinary local paint updates from resident geometry. The
    /// boundary refreshes paint inputs only; shaped runs and fragment IDs stay
    /// owned by the last layout, like Blink's FragmentData/ObjectPaintProperties.
    pub fn ExportTargetedPaintInputs(&mut self) -> bool {
        if self.output_kind != LayoutUpdateKind::Paint {
            return false;
        }
        let mut trace = browser_tracing::span("layout", "LayoutEngine.ExportTargetedPaintInputs");
        if self.tree_.is_none() {
            return false;
        }
        let root = self.RootPointer();
        if unsafe { &*root }.NeedsLayout() {
            return false;
        }
        let Some(fragments) = &mut self.fragments else {
            return false;
        };
        let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        if !crate::internal::boundary::layout_boundary::RefreshRetainedPaintInputs(
            unsafe { &mut *root },
            Rc::make_mut(fragments),
            &self.pending_paint_ids_,
            &self.pending_inline_ink_ids_,
        ) {
            trace.set("rejected", 1.0);
            return false;
        }
        self.pre_paint_inputs_.update(Rc::make_mut(fragments));
        trace.set("updated_sources", self.pending_paint_ids_.len() as f64);
        self.pending_paint_ids_.clear();
        self.pending_inline_ink_ids_.clear();
        trace.set("published", 1.0);
        true
    }

    /// The latest successful exported layout snapshot. Reading it does not
    /// clone the tree or run layout.
    pub fn GetLayoutResult(&self) -> Option<&Rc<FragmentNode>> {
        self.fragments.as_ref()
    }

    /// Transfer the published snapshot for an exclusive Paint/scroll update.
    /// This does not clone fragments or mutate the native tree.
    pub fn TakeLayoutResult(&mut self) -> Option<Rc<FragmentNode>> {
        self.fragments.take()
    }

    /// Transfer the resident result into the zero-copy Layout -> Paint path.
    pub fn TakePaintOutput(&mut self) -> Option<LayoutPaintOutput> {
        self.fragments
            .take()
            .map(|fragments| LayoutPaintOutput { fragments })
    }

    /// Transfer Page's currently published frame back into the Layout -> Paint
    /// path. The engine's matching retained `Rc` is dropped before mutation so
    /// it cannot itself trigger fragment-tree copy-on-write.
    pub fn TakePublishedPaintOutput(&mut self, fragments: Rc<FragmentNode>) -> LayoutPaintOutput {
        if let Some(resident) = self.fragments.take() {
            debug_assert!(
                Rc::ptr_eq(&resident, &fragments),
                "published Page frame must match LayoutEngine output"
            );
        }
        LayoutPaintOutput { fragments }
    }

    /// Commit one completed Paint output and republish the same fragment owner.
    /// No fragment or artifact snapshot is created here; only `Rc`/`Arc`
    /// handles cross the interface.
    pub fn CommitPaint<P: PaintCommitOutput>(
        &mut self,
        mut layout: LayoutPaintOutput,
        paint: P,
        caret: Option<&FrameCaret>,
    ) -> (Rc<FragmentNode>, P::Artifact) {
        let (artifact, commit) = paint.IntoLayoutCommit();
        if let Some(caret) = caret {
            let caret_id = caret.display_item_client().Id();
            if commit.used_clients.contains(&caret_id) {
                caret.ValidateForCommittedPaint(caret_id);
            }
        }
        if let Some(tree) = &self.tree_ {
            tree.CommitPaintClientState(&commit.used_clients, &mut layout.fragments);
        }
        self.pre_paint_inputs_
            .CommitClients(commit.used_clients.iter().copied());
        self.output_kind = LayoutUpdateKind::Paint;
        self.fragments = Some(layout.fragments.clone());
        (layout.fragments, artifact)
    }

    /// Republish a zero-copy Layout output after Paint proved that its existing
    /// display-item recording remains valid.
    pub fn CommitRetainedPaint(&mut self, layout: LayoutPaintOutput) -> Rc<FragmentNode> {
        self.output_kind = LayoutUpdateKind::Paint;
        self.fragments = Some(layout.fragments.clone());
        layout.fragments
    }

    /// Release the published result when its input/lifecycle has been replaced.
    pub fn ReleaseLayoutResult(&mut self) {
        self.fragments = None;
    }

    /// Return a successfully committed immutable Paint/scroll snapshot to the
    /// engine. The caller owns the lifecycle proof; publishing performs no walk.
    pub fn PublishLayoutResult(&mut self, fragments: Rc<FragmentNode>) {
        self.output_kind = LayoutUpdateKind::Paint;
        self.fragments = Some(fragments);
    }

    /// Internal retained-scroll path. The caller must already own fragments and
    /// prove that styles are unchanged. It compares the stored native result
    /// before/after layout, then exports a changed result or refreshes the
    /// retained snapshot's scroll paint properties. This method publishes no
    /// snapshot and is not the ordinary Page layout entry point.
    #[doc(hidden)]
    pub fn LayoutWithoutExport(&mut self) {
        let _trace = browser_tracing::span("layout", "LayoutEngine.LayoutWithoutExport");
        self.scroll_paint_index_ = None;
        self.pending_scroll_ids_.clear();
        let root = self.RootPointer();
        self.fragments = None;
        // Retained scroll refresh is owned by the caller and can mutate inputs
        // without exporting. Its next ordinary layout must establish a basis.
        self.pre_paint_inputs_.invalidate();
        let _ = self.RunLayout(
            unsafe { &mut *root },
            &self.constraints_,
            false,
            LayoutUpdateKind::Paint,
        );
    }

    /// The caller admits only unchanged styles and known scroll owners. Keep
    /// the resident scroll fast path and its result-identity fallback inside
    /// the engine that owns the native objects.
    #[doc(hidden)]
    pub fn UpdateScrollLayout(
        &mut self,
        fragments: &mut FragmentNode,
        scroll_only: bool,
    ) -> Option<ScrollLayoutUpdate> {
        let mut trace = browser_tracing::span("layout", "LayoutEngine.UpdateScrollLayout");
        trace.set("offset_only", scroll_only as u8 as f64);
        trace.set("pending_scrolls", self.pending_scroll_ids_.len() as f64);
        if self.tree_.is_none() {
            return None;
        }
        let root = self.RootPointer();
        let box_ptr = DynamicTo::<LayoutBox>(root);
        if box_ptr.is_null() {
            return None;
        }
        let before = unsafe { &*box_ptr }.GetLayoutResult(0);
        trace.set(
            "native_needs_layout",
            unsafe { &*root }.NeedsLayout() as u8 as f64,
        );
        let assembly = LayoutAssembly {
            algorithms: self.algorithms_,
            objects: self.objects_,
            boundary: self.boundary_,
        };
        if scroll_only && !unsafe { &*root }.NeedsLayout() && !before.is_null() {
            let mut heap = LayoutHeapScope::new();
            let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
            let refreshed = self.scroll_paint_index_.as_ref().is_some_and(|index| {
                index.refresh(unsafe { &mut *root }, fragments, &self.pending_scroll_ids_)
            });
            trace.set("indexed_refresh", refreshed as u8 as f64);
            if !refreshed {
                assembly.RefreshScrollPaintProperties(unsafe { &mut *root }, fragments, true);
                self.scroll_paint_index_ = Some(ScrollPaintPropertyIndex::new(
                    unsafe { &mut *root },
                    fragments,
                ));
            }
            self.pending_scroll_ids_.clear();
            heap.DeferCollection();
            trace.set("retained_geometry", 1.0);
            return Some(ScrollLayoutUpdate::RetainedGeometry);
        }
        self.LayoutWithoutExport();
        let after = unsafe { &*box_ptr }.GetLayoutResult(0);
        trace.set("layout_result_changed", (before != after) as u8 as f64);
        if before != after {
            *fragments = assembly.ExportFragments(unsafe { &mut *root }, &self.constraints_);
        } else {
            assembly.RefreshScrollPaintProperties(unsafe { &mut *root }, fragments, false);
        }
        Some(ScrollLayoutUpdate::RefreshedLayout)
    }

    // cpp: layoutng/layout_engine.h:24-25
    // cpp: layoutng/layout_engine.cc:20-73
    fn RunLayout(
        &self,
        root: &mut LayoutObject,
        space: &ConstraintSpace,
        export: bool,
        purpose: LayoutUpdateKind,
    ) -> Option<FragmentNode> {
        let mut trace = browser_tracing::span("layout", "LayoutEngine.RunLayout");
        trace.set("export", export as u8 as f64);
        let mut heap_scope = LayoutHeapScope::new();
        let reuse_candidate = !root.NeedsLayout();
        InitStringStatics();
        Length::Initialize();

        let (Some(create_environment), Some(prepare_constraints), Some(prepare_tree)) = (
            self.boundary_.create_environment,
            self.boundary_.prepare_constraints,
            self.boundary_.prepare_tree,
        ) else {
            std::panic::panic_any(LayoutEngineError::InvalidArgument(
                "LayoutObject boundary is not installed",
            ));
        };
        let export_tree = if export {
            Some(self.boundary_.export_tree.unwrap_or_else(|| {
                std::panic::panic_any(LayoutEngineError::InvalidArgument(
                    "fragment snapshot exporter is not installed",
                ));
            }))
        } else {
            None
        };
        let mut environment = create_environment(root, space);
        let root_box = DynamicTo::<LayoutBox>(root as *mut LayoutObject);
        if root_box.is_null() {
            std::panic::panic_any(LayoutEngineError::InvalidArgument(
                "LayoutObject root must be a LayoutBox",
            ));
        }

        let previous_result = if unsafe { &*root_box }.GetLayoutResults().is_empty() {
            std::ptr::null()
        } else {
            unsafe { &*root_box }.GetLayoutResult(0)
        };
        let mut contains_annotations = false;
        let missing_forms = self
            .algorithms_
            .forms_support
            .intrinsic_inline_size
            .is_none()
            || self
                .algorithms_
                .forms_support
                .intrinsic_block_size
                .is_none();
        let missing_float = self
            .algorithms_
            .float_support
            .margin_box_inline_size
            .is_none()
            || self.algorithms_.float_support.position.is_none()
            || self.algorithms_.float_support.create_shape.is_none();

        // C++ virtual dispatch remains available while walking the tree before
        // LayoutPassScope. Rust's explicit SVG vtable needs the same factory
        // callbacks during this walk.
        let _object_factory_scope = LayoutObjectFactoryScope::new(&self.objects_);
        let mut object: *mut LayoutObject = root;
        let mut object_count = 0usize;
        let mut needs_layout_count = 0usize;
        let mut self_needs_full_layout_count = 0usize;
        let mut child_needs_full_layout_count = 0usize;
        while !object.is_null() {
            let current = unsafe { &*object };
            object_count += 1;
            needs_layout_count += current.NeedsLayout() as usize;
            self_needs_full_layout_count += current.SelfNeedsFullLayout() as usize;
            child_needs_full_layout_count += current.ChildNeedsFullLayout() as usize;
            if missing_float && current.IsFloating() {
                std::panic::panic_any(UnsupportedLayout::new(
                    "float layout module is not installed",
                ));
            }
            let node = current.GetNode();
            if missing_forms
                && !node.is_null()
                && unsafe { &*node }.InputKind() == NodeKind::kFormControl
            {
                std::panic::panic_any(UnsupportedLayout::new(
                    "forms layout module is not installed",
                ));
            }
            if current.IsInlineRubyText()
                || current.StyleRef().GetTextEmphasisMark() != TextEmphasisMark::kNone
            {
                contains_annotations = true;
                if !missing_forms {
                    break;
                }
            }
            object = current.NextInPreOrder(root);
        }
        trace.set("objects", object_count as f64);
        trace.set("needs_layout_objects", needs_layout_count as f64);
        trace.set(
            "self_needs_full_layout_objects",
            self_needs_full_layout_count as f64,
        );
        trace.set(
            "child_needs_full_layout_objects",
            child_needs_full_layout_count as f64,
        );

        let root_node = root.GetNode();
        let constraints = prepare_constraints(
            unsafe { &*root_node }.InputStyle(),
            space,
            contains_annotations,
        );
        let _layout_pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        let reuses_prepared_fonts = environment.ReusesPreparedFonts();
        trace.set("reuses_prepared_fonts", reuses_prepared_fonts as u8 as f64);
        prepare_tree(root, space, &constraints, reuses_prepared_fonts);
        let native_root = BlockNode::new(root_box);
        trace.set(
            "root_cached_results",
            unsafe { &*root_box }.GetLayoutResults().len() as f64,
        );
        trace.set(
            "root_skips_layout_cache",
            unsafe { &*root_box }.ShouldSkipLayoutCache() as u8 as f64,
        );
        let algorithm = browser_tracing::span("layout", "LayoutEngine.NativeLayout");
        // The extracted host publishes one complete root layout transaction.
        // Delay provisional scrollbar clamps until all nested relayouts settle,
        // then resolve them before sticky properties and fragments are exported.
        let clamp_scope = crate::internal::scroll_layout_scope::DelayScrollOffsetClampScope::new();
        let result = native_root.Layout(
            &constraints,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        drop(clamp_scope);
        drop(algorithm);
        trace.set(
            "layout_result_reused",
            (result == previous_result) as u8 as f64,
        );
        if result.is_null() || unsafe { &*result }.Status() != EStatus::kSuccess {
            std::panic::panic_any(LayoutEngineError::RuntimeError(
                "BlockNode::Layout did not produce a successful result",
            ));
        }
        // BlockNode stores the result on the LayoutBox; the tree owns its lifetime.
        // Export uses the same assembly callbacks, font bindings and layout
        // pass as the native result, before committing the environment.
        let fragments = export_tree.map(|export_tree| {
            let _trace = browser_tracing::span("layout", "LayoutEngine.ExportFragments");
            export_tree(root, unsafe { &*result }, space)
        });
        if purpose == LayoutUpdateKind::Layout {
            let mut objects = Vec::new();
            let mut object = root as *mut LayoutObject;
            while !object.is_null() {
                objects.push(object);
                object = unsafe { &*object }.NextInPreOrder(root);
            }
            crate::internal::boundary::layout_boundary::UpdateStickyOffsets(objects, false);
        }
        environment.Commit(root);
        if reuse_candidate && result == previous_result {
            heap_scope.AllowUnchangedReuse();
        }
        fragments
    }
}

#[cfg(test)]
mod incremental_fragment_tests {
    use super::*;

    fn fragment(node_id: u64, instance_id: u64) -> FragmentNode {
        let mut node = FragmentNode::default();
        node.node_id = node_id;
        node.fragment_instance_id = instance_id;
        node.paint.has_source = true;
        node.paint.establishes_paint_state = true;
        node
    }

    #[test]
    fn subtree_replacement_resolves_logical_parent_in_retained_tree() {
        let mut root = fragment(1, 1);
        let logical_parent = fragment(200, 20);
        let mut old_target = fragment(300, 30);
        let mut old_child = fragment(400, 40);
        old_child.paint.logical_parent_node_id = Some(200);
        old_child.paint.logical_parent_fragment_instance_id = Some(20);
        old_target.children.push(old_child);
        root.children = vec![logical_parent, old_target];

        // A standalone subtree export cannot see its logical parent in the
        // retained sibling and therefore carries a stale occurrence id.
        let mut replacement = fragment(300, 1);
        let mut replacement_child = fragment(400, 2);
        replacement_child.paint.logical_parent_node_id = Some(200);
        replacement_child.paint.logical_parent_fragment_instance_id = Some(999);
        replacement.children.push(replacement_child);

        let mut next_fragment = 41;
        let mut next_fragmentainer = 1;
        assert!(ReplaceFragmentNode(
            &mut root,
            300,
            &mut replacement,
            &mut next_fragment,
            &mut next_fragmentainer,
        ));
        ResolveLogicalParentOccurrences(&mut root);

        assert_eq!(
            root.children[1].children[0]
                .paint
                .logical_parent_fragment_instance_id,
            Some(20)
        );
    }
}
