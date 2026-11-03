#![allow(non_snake_case)]

use foundation::{
    DynamicTo, InitStringStatics, LayoutHeapScope, Length, PrepareLayoutHeapForPointerQueries,
    UnsupportedLayout,
};
use layout_fragment::fragment_tree::pre_paint_revision::PrePaintRevisionTracker;
use layout_fragment::fragment_tree::{FragmentPaintData, LayoutResultHandle};
use layout_fragment::layout_result::EStatus;
use std::{collections::HashSet, rc::Rc, sync::Arc};

use crate::internal::layout_object_builder::LayoutObjectTree;
use crate::{caret::FrameCaret, editing_state::LayoutEditingState};
use layoutng_style::style::computed_style::ComputedStyle as StyleComputedStyle;

use crate::internal::block_node::BlockNode;
use crate::internal::boundary::layout_boundary::{
    RefreshCompatibleNodeInputCache, RefreshCompatiblePaintCache, ScrollPaintPropertyIndex,
};
use crate::internal::layout_algorithm_set::LayoutAlgorithmSet;
use crate::internal::layout_block_flow::LayoutBlockFlow;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_input::{ConstraintSpace, Offset};
use crate::internal::layout_invalidation_reason::kFontsChanged;
use crate::internal::layout_object::LayoutObject;
use crate::internal::layout_object_factory_set::LayoutObjectFactorySet;
use crate::internal::layout_pass_scope::{LayoutObjectFactoryScope, LayoutPassScope};
use crate::internal::layout_text::LayoutText;
use crate::internal::scroll_anchor::ScrollAnchoringAdjustmentScope;
use crate::layout_assembly::LayoutAssembly;
use crate::layout_boundary_support::{LayoutBoundarySupport, LayoutUpdateKind};
pub use layout_commit::{PaintClientCommit, PaintCommitOutput};

#[path = "../../layout_invalidation/src/tree_facts.rs"]
mod layout_invalidation_tree_facts;
use layout_invalidation_tree_facts::{collect_layout_tree_facts, LayoutTreeFacts};

#[path = "layout_tree_update.rs"]
mod layout_tree_update;
pub use layout_tree_update::LayoutTreeUpdate;
#[path = "layout_output.rs"]
mod layout_output;
pub(crate) use layout_output::LayoutPaintState;
pub use layout_output::{
    FragmentCursor, LayoutMetric, LayoutNode, LayoutOutput, LayoutRect, LayoutScrollRange,
    PaintFragmentCursor, PaintFragmentLease,
};

// Rust panic payloads retain the two standard C++ failure categories emitted
// by this entry point; unsupported modules use foundation::UnsupportedLayout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutEngineError {
    InvalidArgument(&'static str),
    RuntimeError(&'static str),
}

#[path = "../../layout_fragment/src/prepaint_input_update.rs"]
mod layout_fragment_prepaint_input_update;
use layout_fragment_prepaint_input_update::{
    UpdateFragmentPaintStyle, UpdatePrePaintInputs, VisitMutablePaintInputs,
};
#[path = "../../layout_fragment/src/engine_paint_update.rs"]
mod layout_fragment_engine_paint_update;
use layout_fragment_engine_paint_update::EnginePaintUpdate;
#[path = "../../layout_fragment/src/layout_paint_output.rs"]
mod layout_fragment_layout_paint_output;
#[path = "../../layout_fragment/src/retained_scroll_refresh.rs"]
mod layout_fragment_retained_scroll_refresh;
#[path = "../../layout_fragment/src/sticky_layout_commit.rs"]
mod layout_fragment_sticky_layout_commit;
#[path = "../../layout_invalidation/src/constraint_equivalence.rs"]
mod layout_invalidation_constraint_equivalence;
use layout_invalidation_constraint_equivalence::{
    SameConstraintScalars, SameConstraints, SameImageCatalogGeometry, SameOptionalArc,
    SameShapeImageCatalog,
};
#[path = "../../layout_invalidation/src/engine_mutation.rs"]
mod layout_invalidation_engine_mutation;
use layout_invalidation_engine_mutation::EngineMutation;
#[path = "../../layout_commit/src/engine_paint_commit.rs"]
mod layout_commit_engine_paint_commit;
use layout_commit_engine_paint_commit::PaintCommit;
#[path = "../../layout_commit/src/container_query_sizes.rs"]
mod layout_commit_container_query_sizes;
#[path = "../../layout_commit/src/engine_layout_publish.rs"]
mod layout_commit_engine_layout_publish;
#[path = "../../layout_reuse/src/engine_output_reuse.rs"]
mod layout_reuse_engine_output_reuse;
// cpp: layoutng/layout_engine.h:19-31
/// Mutations of the engine-owned input tree. TreeUpdate reconciles one complete
/// DOM projection using the existing source insert/style/text/remove logic;
/// the callback receives only the attachment methods for this transaction.
pub enum LayoutMutation<'a> {
    Constraints(&'a ConstraintSpace),
    /// A contextual platform fallback association changed while the primary
    /// font catalog and every element's ComputedStyle binding stayed stable.
    /// Layout invalidates shaped text on its resident objects in place.
    FontFallbackChanged,
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
        style: &'a StyleComputedStyle,
        resources: &'a crate::style_projection::StyleResourceBindings<'a>,
        element: Option<crate::internal::layout_input::ElementData>,
    },
    PaintStyle {
        node_id: u64,
        style: &'a StyleComputedStyle,
        resources: &'a crate::style_projection::StyleResourceBindings<'a>,
        clip_resource_version: crate::internal::paint_input::SvgClipResourceVersion,
    },
    /// Complete local paint-only input, including source theme metadata.
    PaintInput {
        node_id: u64,
        style: &'a StyleComputedStyle,
        resources: &'a crate::style_projection::StyleResourceBindings<'a>,
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

/// Immutable source content-box geometry. This layout output does not depend
/// on StyleEngine or its query AST.
#[derive(Clone, Debug, PartialEq)]
pub struct ContainerQuerySizeSnapshot {
    pub node_id: u64,
    pub width: f64,
    pub height: f64,
}

/// Move-only access to Layout's resident source result. Geometry and topology
/// come directly from LayoutResult; mutable Paint data stays in the result's
/// flat source-owner sidecar.
pub struct LayoutPaintOutput {
    fragments: LayoutResultHandle,
}

impl LayoutPaintOutput {
    /// Canonical pointer-preserving LayoutNG output.
    pub fn FragmentRoot(&self) -> &layout_fragment::physical_fragment::PhysicalFragment {
        layout_output::RootForHandle(&self.fragments)
    }

    pub fn FragmentCursor(&self) -> FragmentCursor<'_> {
        FragmentCursor::Physical {
            fragment: self.FragmentRoot(),
            offset: foundation::PhysicalOffset::default(),
        }
    }

    pub fn PaintCursor(&self) -> PaintFragmentCursor<'_> {
        PaintFragmentCursor::FromHandle(&self.fragments)
    }

    pub fn PaintLease(&self) -> PaintFragmentLease {
        layout_output::PaintLeaseForHandle(&self.fragments)
    }

    pub fn PaintCullRectHasScrolledEnough(&self, id: u64, offset: Offset) -> bool {
        layout_fragment_layout_paint_output::paint_cull_rect_has_scrolled_enough(self, id, offset)
    }

    pub fn VisitPaintInputsMut(&mut self, visitor: impl FnMut(&mut FragmentPaintData)) {
        layout_fragment_layout_paint_output::visit_paint_inputs_mut(self, visitor)
    }

    #[doc(hidden)]
    pub fn ResultHandleMut(&mut self) -> &mut LayoutResultHandle {
        &mut self.fragments
    }
}

/// Owns the resident source input tree, its constraints and published result.
/// DOM adapters provide mutations; Paint consumes only the fragment output.
pub struct LayoutEngine {
    /// Rust boundary output, populated by a successful Layout(). The source
    /// layout algorithm and LayoutObject lifetime remain unchanged.
    fragments: Option<LayoutResultHandle>,
    /// Last committed source result after its geometry was invalidated. Keep
    /// the owner alive while LayoutNG may reuse child PhysicalFragments.
    /// This is never exposed as current output or used as paint geometry.
    invalidated_fragments_: Option<LayoutResultHandle>,
    output_kind: LayoutUpdateKind,
    tree_: Option<LayoutObjectTree>,
    constraints_: ConstraintSpace,
    pre_paint_inputs_: PrePaintRevisionTracker,
    scroll_paint_index_: Option<ScrollPaintPropertyIndex>,
    // Chromium retains FragmentData/property directories across lifecycle
    // updates. A stale directory is never queried, but its allocation storage
    // can seed the replacement built after the next successful layout.
    scroll_paint_index_spare_: Option<ScrollPaintPropertyIndex>,
    pending_scroll_ids_: HashSet<u64>,
    pending_paint_ids_: HashSet<u64>,
    pending_inline_ink_ids_: HashSet<u64>,
    // A single Element::RebuildLayoutTree root can retain the rest of the
    // source LayoutObject tree. Multiple or unrelated mutations clear this
    // local-layout hint.
    pending_subtree_layout_id_: Option<u64>,
    algorithms_: LayoutAlgorithmSet,
    objects_: LayoutObjectFactorySet,
    boundary_: LayoutBoundarySupport,
    boundary_cache_: crate::layout_boundary_support::LayoutBoundaryCache,
    tree_facts_: Option<LayoutTreeFacts>,
    tree_relations_dirty_: bool,
    // ShapeOutsideInfo and document-image SVG containers observe external
    // resource inputs. Chromium invalidates them when those inputs change;
    // keep the same dirty boundary instead of scanning every LayoutObject on
    // an unrelated viewport-only layout.
    shape_image_inputs_dirty_: bool,
    document_image_container_dirty_: bool,
    targeted_font_objects_: Option<Vec<*mut LayoutObject>>,
}

impl LayoutEngine {
    fn CachedScrollIndexObjects(
        &self,
    ) -> Option<(Arc<Vec<*mut LayoutObject>>, Arc<Vec<*mut LayoutObject>>)> {
        (!self.boundary_cache_.source_objects.is_empty()).then(|| {
            (
                Arc::clone(&self.boundary_cache_.source_objects),
                Arc::clone(&self.boundary_cache_.sticky_objects),
            )
        })
    }

    // cpp: layoutng/layout_engine.h:21-23
    pub fn new(assembly: &LayoutAssembly) -> Self {
        assembly.algorithms.validate_performance_ownership();
        Self {
            fragments: None,
            invalidated_fragments_: None,
            output_kind: LayoutUpdateKind::Paint,
            tree_: None,
            constraints_: ConstraintSpace::default(),
            pre_paint_inputs_: PrePaintRevisionTracker::new(),
            scroll_paint_index_: None,
            scroll_paint_index_spare_: None,
            pending_scroll_ids_: HashSet::new(),
            pending_paint_ids_: HashSet::new(),
            pending_inline_ink_ids_: HashSet::new(),
            pending_subtree_layout_id_: None,
            algorithms_: assembly.algorithms,
            objects_: assembly.objects,
            boundary_: assembly.boundary,
            boundary_cache_: Default::default(),
            tree_facts_: None,
            tree_relations_dirty_: true,
            shape_image_inputs_dirty_: true,
            document_image_container_dirty_: true,
            targeted_font_objects_: None,
        }
    }

    /// Move Page's published fragment owner back to Layout before a full
    /// lifecycle. Chromium has one resident FragmentData graph shared by the
    /// lifecycle and compositor; this transfers the published pointer/lifetime
    /// handle so a constraint-only layout can update resident state in place.
    /// A synchronous geometry query may already have published a newer
    /// layout-only result before Page enters the next Paint lifecycle. In that
    /// case the still-presented Page frame is stale and must not replace the
    /// newer resident result.
    pub fn AdoptPublishedLayoutResult(&mut self, fragments: LayoutResultHandle) {
        if let Some(resident) = self.fragments.take() {
            self.fragments = Some(if resident.PtrEq(&fragments) {
                fragments
            } else {
                resident
            });
            return;
        } else if let Some(invalidated) = self.invalidated_fragments_.take() {
            if !invalidated.PtrEq(&fragments) {
                self.invalidated_fragments_ = Some(invalidated);
                return;
            }
        }
        self.fragments = Some(fragments);
    }

    /// A clean resident tree may prove that an attribute changed no geometry.
    /// Uses the same canonical style and metadata admission as paint updates,
    /// without changing source objects or fragment-local paint/resource data.
    pub fn GeometryInputMatches(
        &self,
        id: u64,
        style: &StyleComputedStyle,
        resources: &crate::style_projection::StyleResourceBindings<'_>,
        element: &Option<crate::internal::layout_input::ElementData>,
    ) -> bool {
        EngineMutation::geometry_input_matches(self, id, style, resources, element)
    }

    /// Apply input changes without running layout. Chromium synchronization
    /// preserves client identities and invalidates layout exactly as before.
    pub fn ApplyMutation(&mut self, mutation: LayoutMutation<'_>) -> bool {
        EngineMutation::apply_mutation(self, mutation)
    }

    /// Read-only access for source lifecycle consumers. The engine keeps sole
    /// ownership; callers cannot retain an Rc that extends the tree lifetime.
    pub fn GetLayoutTree(&self) -> Option<&LayoutObjectTree> {
        self.tree_.as_ref()
    }

    /// Chromium supplies laid-out content sizes to SizeContainerChanged.
    /// Chromium content boxes exclude borders, padding and scrollbars;
    /// FragmentPaintData::content_size instead describes scrollable overflow.
    pub fn ContainerQuerySizes(&self) -> Vec<ContainerQuerySizeSnapshot> {
        layout_commit_container_query_sizes::collect_container_query_sizes(self)
    }

    pub fn GetConstraints(&self) -> &ConstraintSpace {
        &self.constraints_
    }

    fn RootPointer(&mut self) -> *mut LayoutObject {
        self.tree_
            .as_mut()
            .expect("apply a tree mutation before Layout")
            .RootMut() as *mut LayoutObject
    }

    /// Publish the source result only after layout and Paint-data population
    /// succeed. A rejected pass cannot leave the previous result visible.
    pub fn Layout(&mut self) {
        self.UpdateLayoutOutput(LayoutUpdateKind::Paint);
    }

    /// Advance Paint layout when source inputs are dirty. The boolean mirrors
    /// LayoutView::UpdateLayout's lifecycle result for the host: `false` means
    /// the resident source result and its committed Paint artifact still
    /// describe the current inputs.
    pub fn LayoutIfNeeded(&mut self) -> bool {
        self.UpdateLayoutOutput(LayoutUpdateKind::Paint)
    }

    /// Flush source layout and publish only CSSOM geometry. This output has
    /// no paint certification and cannot enter retained paint update paths.
    pub fn UpdateLayout(&mut self) {
        self.UpdateLayoutOutput(LayoutUpdateKind::Layout);
    }

    /// Flush CSSOM geometry and report whether source layout/output advanced.
    pub fn UpdateLayoutIfNeeded(&mut self) -> bool {
        self.UpdateLayoutOutput(LayoutUpdateKind::Layout)
    }
    /// A borrow of resident output. Consumers receive values and scoped
    /// fragment access, never the engine's source object ownership.
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

    fn UpdateLayoutOutput(&mut self, purpose: LayoutUpdateKind) -> bool {
        let mut trace = browser_tracing::span("layout", "LayoutEngine.Layout");
        // LayoutView::UpdateLayout returns immediately once Blink's lifecycle
        // is layout-clean. Keep the same boundary here: a coalesced resize can
        // deliver the already-active constraints again, and RetainTree leaves
        // both the source result and its fragment-local Paint data current.
        // Paint output is a strict superset of layout-only output, while the
        // reverse still requires Paint-data population.
        let has_current_output =
            layout_reuse_engine_output_reuse::has_current_output(self, purpose);
        if has_current_output {
            trace.set("clean_output_reused", 1.0);
            return false;
        }
        let preparation = layout_commit_engine_layout_publish::begin_layout_output(self, purpose);
        trace.set(
            "measurement",
            (purpose == LayoutUpdateKind::Layout) as u8 as f64,
        );
        // While layout and Paint-data population run, the engine has no
        // certified input catalog.
        // Unwinding leaves it empty; a subsequent success receives fresh
        // globally unique revisions instead of reusing a failed pass's basis.
        // Chromium republishes the LayoutResult owner and keeps paint-local
        // data on FragmentData. A targeted relayout cannot splice a copied
        // Rust fragment subtree because no such tree exists.
        trace.set("targeted_layout", preparation.targeted_layout as u8 as f64);
        let constraints = self.constraints_.clone();
        let layout_result = self.RunLayout(
            unsafe { &mut *preparation.root },
            &constraints,
            true,
            purpose,
        );
        if let Some(fragments) = layout_result {
            layout_commit_engine_layout_publish::publish_layout_output(
                self,
                preparation.root,
                purpose,
                preparation.inputs,
                fragments,
            );
            trace.set("published", 1.0);
        }
        true
    }

    /// Refresh Paint inputs on the resident layout result without running
    /// layout. Blink's paint-only style path updates ObjectPaintProperties from
    /// the existing layout geometry; transform/opacity animation samples must
    /// not turn into a full layout solely to refresh the fragment snapshot.
    ///
    /// The DOM adapter may conservatively classify an update as paint-only.
    /// The source tree is the final guard: if synchronization marked any
    /// layout object dirty, reject this path and let the caller run Layout().
    pub fn PopulatePaintData(&mut self) -> bool {
        EnginePaintUpdate::populate_paint_data(self)
    }

    /// Publish paint-only values already applied to the retained fragment
    /// snapshot. Chromium updates FragmentData/ObjectPaintProperties in place;
    /// this immutable Rust boundary uses copy-on-write once per frame, then
    /// refreshes the same subtree revision proof consumed by PrePaint.
    pub fn RefreshTargetedPaintData(&mut self) -> bool {
        EnginePaintUpdate::refresh_targeted_paint_data(self)
    }

    /// Publish ordinary local paint updates from resident geometry. The
    /// boundary refreshes paint inputs only; shaped runs and fragment IDs stay
    /// owned by the last layout, like Blink's FragmentData/ObjectPaintProperties.
    pub fn RefreshTargetedPaintInputs(&mut self) -> bool {
        EnginePaintUpdate::refresh_targeted_paint_inputs(self)
    }

    /// Deliver the resource-observer notification for document images whose
    /// embedding environment changed while source geometry stayed resident.
    pub fn NotifyDocumentImageEnvironmentChanged(
        &mut self,
        images: &[crate::internal::layout_input::PaintImage],
    ) -> usize {
        let invalidated = self
            .tree_
            .as_mut()
            .map_or(0, |tree| tree.InvalidateDocumentImageClients(images));
        let mut trace =
            layout_invalidation::trace_span(layout_invalidation::TracePhase::ImageInvalidation);
        trace.set("document_image_clients", invalidated as f64);
        invalidated
    }

    /// The latest successful source layout result. Reading it does not clone
    /// source fragments or run layout.
    pub fn GetLayoutResult(&self) -> Option<&LayoutResultHandle> {
        self.fragments.as_ref()
    }

    /// Transfer the published snapshot for an exclusive Paint/scroll update.
    /// This does not clone fragments or mutate the source tree.
    pub fn TakeLayoutResult(&mut self) -> Option<LayoutResultHandle> {
        self.fragments.take()
    }

    /// Transfer the resident result into Paint. Moving this owner is
    /// allocation-free; Paint traverses the source fragment cursor.
    pub fn TakePaintOutput(&mut self) -> Option<LayoutPaintOutput> {
        self.fragments
            .take()
            .map(|fragments| LayoutPaintOutput { fragments })
    }

    /// Transfer Page's currently published frame back into the Layout -> Paint
    /// path. The engine's matching retained owner is dropped before mutation.
    pub fn TakePublishedPaintOutput(&mut self, fragments: LayoutResultHandle) -> LayoutPaintOutput {
        if let Some(resident) = self.fragments.take() {
            debug_assert!(
                resident.PtrEq(&fragments),
                "published Page frame must match LayoutEngine output"
            );
        }
        LayoutPaintOutput { fragments }
    }

    /// Commit one completed Paint output and republish the same fragment owner.
    /// No fragment or artifact snapshot is created here; only source fragment
    /// and artifact handles cross the interface.
    pub fn CommitPaint<P: PaintCommitOutput>(
        &mut self,
        layout: LayoutPaintOutput,
        paint: P,
        caret: Option<&FrameCaret>,
    ) -> (LayoutResultHandle, P::Artifact) {
        layout_commit::CommitPaintResult(
            &mut PaintCommit {
                engine: self,
                caret,
            },
            layout,
            paint,
        )
    }

    /// Republish the same source/result owner after Paint proved that its
    /// existing display-item recording remains valid. This fast path performs
    /// no source-fragment walk.
    pub fn CommitRetainedPaint(&mut self, layout: LayoutPaintOutput) -> LayoutResultHandle {
        self.output_kind = LayoutUpdateKind::Paint;
        self.invalidated_fragments_ = None;
        self.fragments = Some(layout.fragments.clone());
        layout.fragments
    }

    /// Release the published result when its input/lifecycle has been replaced.
    pub fn ReleaseLayoutResult(&mut self) {
        self.fragments = None;
        self.invalidated_fragments_ = None;
    }

    /// Return a successfully committed immutable Paint/scroll snapshot to the
    /// engine. The caller owns the lifecycle proof; publishing performs no walk.
    pub fn PublishLayoutResult(&mut self, fragments: LayoutResultHandle) {
        self.output_kind = LayoutUpdateKind::Paint;
        self.invalidated_fragments_ = None;
        self.fragments = Some(fragments);
    }

    /// Internal retained-scroll path. The caller must already own fragments and
    /// prove that styles are unchanged. It compares the stored source result
    /// before/after layout, then publishes a changed result or refreshes the
    /// retained result's scroll paint properties. This method publishes no
    /// result and is not the ordinary Page layout entry point.
    #[doc(hidden)]
    pub fn RunLayoutWithoutPublication(&mut self) {
        layout_commit_engine_layout_publish::run_layout_without_publication(self)
    }

    /// Apply ScrollingCoordinator's compositor-owned overlay visibility input
    /// to the resident source scroll owner without rebuilding the layout tree.
    pub fn UpdateScrollbarsHiddenIfOverlay(&mut self, id: u64, hidden: bool) -> bool {
        let changed = self
            .tree_
            .as_mut()
            .is_some_and(|tree| tree.UpdateScrollbarsHiddenIfOverlay(id, hidden));
        if changed {
            self.pending_scroll_ids_.insert(id);
        }
        changed
    }

    /// The caller admits only unchanged styles and known scroll owners. Keep
    /// the resident scroll fast path and its result-identity fallback inside
    /// the engine that owns the source objects.
    #[doc(hidden)]
    pub fn UpdateScrollLayout(
        &mut self,
        fragments: &mut LayoutResultHandle,
        scroll_only: bool,
        refresh_cull_rect: bool,
    ) -> Option<ScrollLayoutUpdate> {
        let mut trace = browser_tracing::span("layout", "LayoutEngine.UpdateScrollLayout");
        trace.set("offset_only", scroll_only as u8 as f64);
        trace.set("refresh_cull_rect", refresh_cull_rect as u8 as f64);
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
        trace.set("needs_layout", unsafe { &*root }.NeedsLayout() as u8 as f64);
        let assembly = LayoutAssembly {
            algorithms: self.algorithms_,
            objects: self.objects_,
            boundary: self.boundary_,
        };
        if scroll_only && !unsafe { &*root }.NeedsLayout() && !before.is_null() {
            let refreshed =
                layout_fragment_retained_scroll_refresh::refresh_retained_scroll_geometry(
                    self,
                    root,
                    fragments,
                    refresh_cull_rect,
                )?;
            if let Some(indexed) = refreshed.indexed_refresh {
                trace.set("indexed_refresh", indexed as u8 as f64);
            } else {
                trace.set("scroll_cull_data_refreshed", 1.0);
            }
            trace.set("retained_geometry", 1.0);
            return Some(refreshed.update);
        }
        self.RunLayoutWithoutPublication();
        let after = unsafe { &*box_ptr }.GetLayoutResult(0);
        trace.set("layout_result_changed", (before != after) as u8 as f64);
        if before != after {
            *fragments = assembly.PublishLayoutResult(
                unsafe { &mut *root },
                &self.constraints_,
                &mut self.boundary_cache_,
            );
        } else {
            assembly.RefreshScrollPaintProperties(unsafe { &mut *root }, fragments, false);
        }
        Some(ScrollLayoutUpdate::RefreshedLayout)
    }

    /// Commit Blink's main-thread CullRectUpdater scroll baseline without
    /// changing source layout geometry or republishing the fragment graph.
    /// Page uses this both at a phased wheel boundary and as the correctness
    /// fallback when retained Paint coverage is spent.
    #[doc(hidden)]
    pub fn RefreshScrollCullData(&mut self, fragments: &mut LayoutResultHandle) -> bool {
        EnginePaintUpdate::refresh_scroll_cull_data(self, fragments)
    }

    // cpp: layoutng/layout_engine.h:24-25
    // cpp: layoutng/layout_engine.cc:20-73
    fn RunLayout(
        &mut self,
        root: &mut LayoutObject,
        space: &ConstraintSpace,
        publish: bool,
        purpose: LayoutUpdateKind,
    ) -> Option<LayoutResultHandle> {
        let mut trace = browser_tracing::span("layout", "LayoutEngine.RunLayout");
        trace.set("publish", publish as u8 as f64);
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
        let publish_layout_result = if publish {
            Some(self.boundary_.publish_layout_result.unwrap_or_else(|| {
                std::panic::panic_any(LayoutEngineError::InvalidArgument(
                    "layout-result publisher is not installed",
                ));
            }))
        } else {
            None
        };
        let transaction =
            layout_commit::LayoutCommitTransaction::Begin(|| create_environment(root, space));
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

        // These facts change with LayoutObject topology or style ownership,
        // both of which invalidate tree_facts_ at the mutation boundary. A
        // viewport constraint update keeps that ownership stable, so do not
        // walk the entire resident tree before every incremental layout.
        let facts = *self
            .tree_facts_
            .get_or_insert_with(|| collect_layout_tree_facts(root, &self.objects_));
        trace.set("objects", facts.object_count as f64);
        if missing_float && facts.has_floating_object {
            std::panic::panic_any(UnsupportedLayout::new(
                "float layout module is not installed",
            ));
        }
        if missing_forms && facts.has_form_control {
            std::panic::panic_any(UnsupportedLayout::new(
                "forms layout module is not installed",
            ));
        }

        let root_node = root.GetNode();
        let constraints = {
            let _constraints = layout_invalidation::trace_span(
                layout_invalidation::TracePhase::PrepareConstraints,
            );
            prepare_constraints(root.StyleRef(), space, facts.contains_annotations)
        };
        let _layout_pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        let reuses_prepared_fonts = transaction.Environment().ReusesPreparedFonts();
        trace.set("reuses_prepared_fonts", reuses_prepared_fonts as u8 as f64);
        trace.set(
            "tree_relations_dirty",
            self.tree_relations_dirty_ as u8 as f64,
        );
        {
            let _prepare =
                layout_invalidation::trace_span(layout_invalidation::TracePhase::PrepareTree);
            prepare_tree(
                root,
                space,
                &constraints,
                reuses_prepared_fonts,
                self.tree_relations_dirty_,
                self.shape_image_inputs_dirty_,
                self.document_image_container_dirty_,
                self.targeted_font_objects_.as_deref(),
            );
        }
        self.tree_relations_dirty_ = false;
        self.targeted_font_objects_ = Some(Vec::new());
        let block_node = BlockNode::new(root_box);
        trace.set(
            "root_cached_results",
            unsafe { &*root_box }.GetLayoutResults().len() as f64,
        );
        trace.set(
            "root_skips_layout_cache",
            unsafe { &*root_box }.ShouldSkipLayoutCache() as u8 as f64,
        );
        let mut algorithm = browser_tracing::span("layout", "LayoutEngine.AlgorithmDispatch");
        let reuse_trace = layout_reuse::LayoutReuseTraceSession::begin(browser_tracing::enabled());
        let mut scroll_anchor_adjustments = ScrollAnchoringAdjustmentScope::new();
        // The extracted host publishes one complete root layout transaction.
        // Delay provisional scrollbar clamps until all nested relayouts settle,
        // then resolve them before sticky properties and the result are published.
        let clamp_scope = crate::internal::scroll_layout_scope::DelayScrollOffsetClampScope::new();
        let result = block_node.Layout(
            &constraints,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        drop(clamp_scope);
        let reuse_stats = reuse_trace.finish();
        algorithm.set("reuse_lookups", reuse_stats.lookups as f64);
        algorithm.set("reuse_hits", reuse_stats.hits as f64);
        algorithm.set("reuse_needs_layout", reuse_stats.needs_layout as f64);
        algorithm.set("reuse_simplified", reuse_stats.simplified_layouts as f64);
        algorithm.set("reuse_lines", reuse_stats.line_reuses as f64);
        algorithm.set("reuse_lookup_us", reuse_stats.lookup_ns as f64 / 1_000.0);
        drop(algorithm);
        PrepareLayoutHeapForPointerQueries();
        // LocalFrameView performs this queue at the end of the animation-frame
        // layout phase in Chromium. Do it before publishing fragments so the
        // one canonical scroll owner feeds PrePaint and the compositor.
        scroll_anchor_adjustments.Perform();
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
        // Publication uses the same assembly callbacks, font bindings and
        // layout pass as the source result, before committing the environment.
        let fragments = if let Some(publish_layout_result) = publish_layout_result {
            Some(transaction.Publish(
                root,
                |root| {
                    publish_layout_result(
                        root,
                        unsafe { &*result },
                        space,
                        &mut self.boundary_cache_,
                    )
                },
                |root| {
                    if purpose == LayoutUpdateKind::Layout {
                        layout_fragment_sticky_layout_commit::update_sticky_offsets_after_layout(
                            root,
                        );
                    }
                },
                |mut environment, root| environment.Commit(root),
            ))
        } else {
            transaction.Commit(root, |mut environment, root| environment.Commit(root));
            None
        };
        self.shape_image_inputs_dirty_ = false;
        self.document_image_container_dirty_ = false;
        if reuse_candidate && result == previous_result {
            heap_scope.AllowUnchangedReuse();
        }
        fragments
    }
}
