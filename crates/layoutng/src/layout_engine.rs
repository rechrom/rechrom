#![allow(non_snake_case)]

use foundation::{
    DynamicTo, InitStringStatics, LayoutHeapScope, Length, TextEmphasisMark, UnsupportedLayout,
};
use layoutng_fragment_tree::fragment_tree::pre_paint_revision::PrePaintRevisionTracker;
use layoutng_fragment_tree::fragment_tree::FragmentNode;
use layoutng_fragment_tree::layout_result::EStatus;
use std::{collections::HashSet, rc::Rc};

use crate::editing_state::LayoutEditingState;
use crate::internal::layout_object_builder::LayoutObjectTree;

use crate::internal::block_node::BlockNode;
use crate::internal::boundary::layout_boundary::ScrollPaintPropertyIndex;
use crate::internal::layout_algorithm_set::LayoutAlgorithmSet;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_input::{ConstraintSpace, NodeKind};
use crate::internal::layout_object::LayoutObject;
use crate::internal::layout_object_factory_set::LayoutObjectFactorySet;
use crate::internal::layout_pass_scope::{LayoutObjectFactoryScope, LayoutPassScope};
use crate::layout_assembly::LayoutAssembly;
use crate::layout_boundary_support::LayoutBoundarySupport;

#[path = "layout_tree_update.rs"]
mod layout_tree_update;
pub use layout_tree_update::LayoutTreeUpdate;

// Rust panic payloads retain the two standard C++ failure categories emitted
// by this entry point; unsupported modules use foundation::UnsupportedLayout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutEngineError {
    InvalidArgument(&'static str),
    RuntimeError(&'static str),
}

// cpp: layoutng/layout_engine.h:19-31
/// Mutations of the engine-owned input tree. TreeUpdate reconciles one complete
/// DOM projection using the existing native insert/style/text/remove logic;
/// the callback receives only the attachment methods for this transaction.
pub enum LayoutMutation<'a> {
    Constraints(&'a ConstraintSpace),
    TreeUpdate(&'a mut dyn FnMut(&mut LayoutTreeUpdate<'_>)),
    ReplaceTree(LayoutObjectTree),
    ScrollOffset {
        node_id: u64,
        offset: crate::internal::layout_input::Offset,
    },
    Editing {
        node_id: u64,
    },
    EditingState(Rc<LayoutEditingState>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScrollLayoutUpdate {
    RetainedGeometry,
    RefreshedLayout,
}

/// Owns the resident native input tree, its constraints and exported result.
/// DOM adapters provide mutations; Paint consumes only the fragment output.
pub struct LayoutEngine {
    /// Rust boundary output, populated by a successful Layout(). The native
    /// layout algorithm and LayoutObject lifetime remain unchanged.
    fragments: Option<Rc<FragmentNode>>,
    tree_: Option<LayoutObjectTree>,
    constraints_: ConstraintSpace,
    pre_paint_inputs_: PrePaintRevisionTracker,
    scroll_paint_index_: Option<ScrollPaintPropertyIndex>,
    pending_scroll_ids_: HashSet<u64>,
    algorithms_: LayoutAlgorithmSet,
    objects_: LayoutObjectFactorySet,
    boundary_: LayoutBoundarySupport,
}

impl LayoutEngine {
    // cpp: layoutng/layout_engine.h:21-23
    pub fn new(assembly: &LayoutAssembly) -> Self {
        Self {
            fragments: None,
            tree_: None,
            constraints_: ConstraintSpace::default(),
            pre_paint_inputs_: PrePaintRevisionTracker::new(),
            scroll_paint_index_: None,
            pending_scroll_ids_: HashSet::new(),
            algorithms_: assembly.algorithms,
            objects_: assembly.objects,
            boundary_: assembly.boundary,
        }
    }

    /// Apply input changes without running layout. Native synchronization
    /// preserves client identities and invalidates layout exactly as before.
    pub fn ApplyMutation(&mut self, mutation: LayoutMutation<'_>) -> bool {
        // Native topology, style, constraints and layout results may change at
        // all other mutation entries. Never keep borrowed native pointers or
        // exported paths across them, even if the mutation later proves a noop.
        if !matches!(&mutation, LayoutMutation::ScrollOffset { .. }) {
            self.scroll_paint_index_ = None;
            self.pending_scroll_ids_.clear();
        }
        match mutation {
            LayoutMutation::Constraints(space) => {
                self.constraints_ = space.clone();
                self.fragments = None;
                true
            }
            LayoutMutation::TreeUpdate(update) => {
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
                true
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

    pub fn GetConstraints(&self) -> &ConstraintSpace {
        &self.constraints_
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
        let mut trace = browser_tracing::span("layout", "LayoutEngine.Layout");
        self.scroll_paint_index_ = None;
        self.pending_scroll_ids_.clear();
        let root = self.RootPointer();
        self.fragments = None;
        // While layout/export runs, the engine has no certified input catalog.
        // Unwinding leaves it empty; a subsequent success receives fresh
        // globally unique revisions instead of reusing a failed pass's basis.
        let mut inputs = std::mem::take(&mut self.pre_paint_inputs_);
        if let Some(mut fragments) = self.RunLayout(unsafe { &mut *root }, &self.constraints_, true)
        {
            {
                let _trace = browser_tracing::span("prepaint", "LayoutEngine.CertifyPaintInputs");
                inputs.update(&mut fragments);
            }
            self.scroll_paint_index_ = Some({
                let _trace = browser_tracing::span("layout", "LayoutEngine.BuildScrollIndex");
                // RunLayout's temporary pass has ended. Walking native SVG
                // children still needs this engine's real virtual factories.
                let _pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
                ScrollPaintPropertyIndex::new(unsafe { &mut *root }, &fragments)
            });
            self.fragments = Some(Rc::new(fragments));
            self.pre_paint_inputs_ = inputs;
            trace.set("published", 1.0);
        }
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

    /// Release the published result when its input/lifecycle has been replaced.
    pub fn ReleaseLayoutResult(&mut self) {
        self.fragments = None;
    }

    /// Return a successfully committed immutable Paint/scroll snapshot to the
    /// engine. The caller owns the lifecycle proof; publishing performs no walk.
    pub fn PublishLayoutResult(&mut self, fragments: Rc<FragmentNode>) {
        self.fragments = Some(fragments);
    }

    /// Validate only used clients rooted by this engine, then synchronize
    /// exported lifecycle bits and the input observer after a successful Paint.
    #[doc(hidden)]
    pub fn CommitPaintClients(
        &mut self,
        clients: impl IntoIterator<Item = u64>,
        fragments: &mut Rc<FragmentNode>,
    ) {
        let used: HashSet<_> = clients.into_iter().collect();
        if let Some(tree) = &self.tree_ {
            tree.CommitPaintClientState(&used, fragments);
        }
        self.pre_paint_inputs_.CommitClients(used);
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
        let _ = self.RunLayout(unsafe { &mut *root }, &self.constraints_, false);
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
            heap.AllowUnchangedReuse();
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
        while !object.is_null() {
            let current = unsafe { &*object };
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

        let root_node = root.GetNode();
        let constraints = prepare_constraints(
            unsafe { &*root_node }.InputStyle(),
            space,
            contains_annotations,
        );
        let _layout_pass = LayoutPassScope::new(&self.algorithms_, &self.objects_);
        prepare_tree(root, space, &constraints, environment.ReusesPreparedFonts());
        let native_root = BlockNode::new(root_box);
        let algorithm = browser_tracing::span("layout", "LayoutEngine.NativeLayout");
        let result = native_root.Layout(
            &constraints,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
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
        environment.Commit(root);
        if reuse_candidate && result == previous_result {
            heap_scope.AllowUnchangedReuse();
        }
        fragments
    }
}
