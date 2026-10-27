#![allow(non_snake_case)]

use super::boundary::layout_boundary::{
    ConstructionFontResolver, CreateLayoutObject, RequiresPaintBackedInlineFragment,
};
use super::boundary::native_input::{
    InvalidLayoutInput, NativeFontResolverScope, PrepareNativeStyle,
};
use super::boundary::node_metadata_input::CreateNativeElementMetadata;
use super::constraint_space::FragmentationType;
use super::layout_font_resolver::NativeFontResolver;
use super::layout_inline::LayoutInline;
use super::layout_input::{
    ComputedStyle, ConstraintSpace, DocumentRole, ElementData, NativeNodeConstructionData, NodeKind,
};
use super::layout_node_metadata::{
    ContainerNode, NativeNodeMetadataRelations, Node, Text as TextNode,
};
use super::layout_object::{AllowDestroyingLayoutObjectInFinalizerScope, LayoutObject};
use super::layout_object_factory_set::LayoutObjectFactorySet;
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::layout_text::LayoutText;
use foundation::{
    DynamicTo, InitStringStatics, LayoutHeapScope, Length, MakeGarbageCollected, Persistent,
    RuntimeEnabledFeatures, String as BlinkString,
};
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;
use std::{collections::HashMap, sync::Arc};

#[path = "../../../layout_algo_block/src/native_fragmentation_context.rs"]
mod layout_algo_block_fragmentation_context;
#[path = "../../../layout_algo_inline/src/native_collapsible_whitespace.rs"]
mod layout_algo_inline_collapsible_whitespace;
#[path = "../../../layout_commit/src/native_output_node_lookup.rs"]
mod layout_commit_native_output_node_lookup;
#[path = "../../../layout_commit/src/native_scroll_offset_snapshot.rs"]
mod layout_commit_scroll_offset_snapshot;
#[path = "../../../layout_fragment/src/native_paint_input_update.rs"]
mod layout_fragment_paint_input;
#[path = "../../../layout_fragment/src/native_scroll_input_update.rs"]
mod layout_fragment_scroll_input;
#[path = "../../../layout_invalidation/src/native_input_invalidation.rs"]
mod layout_invalidation_input_updates;
#[path = "../../../layout_invalidation/src/native_managed_input_admission.rs"]
mod layout_invalidation_native_managed_input_admission;
#[path = "../../../layout_invalidation/src/native_managed_object_update.rs"]
mod layout_invalidation_native_managed_object_update;
#[path = "../../../layout_invalidation/src/native_node_input_update.rs"]
mod layout_invalidation_node_input;
#[path = "../../../layout_invalidation/src/native_tree_update.rs"]
mod layout_invalidation_package;
#[path = "../../../layout_reuse/src/native_managed_input_index.rs"]
mod layout_reuse_native_managed_input_index;
#[path = "../../../layout_reuse/src/native_managed_object_compatibility.rs"]
mod layout_reuse_native_managed_object_compatibility;
#[path = "../../../layout_reuse/src/native_paint_only_font_binding.rs"]
mod layout_reuse_native_paint_only_font_binding;
use layout_invalidation_native_managed_input_admission::{
    ControlPaintStateChanged, GeometryMetadataEqual, NonStyleInputsEqual, RetainInputStyles,
};
use layout_reuse_native_managed_input_index::BuildManagedInputIndex;
use layout_reuse_native_paint_only_font_binding::PreservePaintOnlyFontBinding;

// The one-argument constructor is defined by //src/main/default_layout_engine.cc
// and supplies FullLayoutAssembly().objects from outside this Bazel package.
unsafe extern "Rust" {
    fn LayoutObjectTreeNew(space: &ConstraintSpace) -> LayoutObjectTree;
}

// cpp: layoutng/internal/layout_object_builder.h:17-78
pub struct LayoutObjectTree {
    storage_: *mut LayoutObjectTreeStorage,
}

pub use layout_reuse::LayoutTreeUpdateStats;

#[derive(Clone, PartialEq, Eq, Hash)]
struct InputKey(u64, i32, bool, Arc<str>, usize);

struct ManagedObject {
    object: *mut LayoutObject,
    node: *mut Node,
    parent: *mut LayoutObject,
    input: NativeNodeConstructionData,
    effective_style: Arc<ComputedStyle>,
    // These descriptors survive DOM style replacement and are read during the
    // next incremental comparison. LayoutObject::StyleRef can be a distinct
    // font-bound copy; tracing it does not retain the canonical source styles.
    input_style_roots: Vec<Persistent<NativeComputedStyle>>,
}

/// Non-CSS data attached by DOM/resource loading and needed when a native
/// `ComputedStyle` is frozen for Paint.  This stays separate from Blink's
/// style object, matching the resource/client relationship in Chromium.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintStyleBinding {
    pub image_resource_id: Option<u64>,
    pub svg_clip_resources: super::paint_input::SvgClipResourceBinding,
}

pub struct PaintInputUpdate {
    pub sources: Vec<u64>,
    pub inline_ink_changes: Vec<u64>,
}

pub(crate) struct NodeInputObjectUpdate {
    pub(crate) object: *mut LayoutObject,
}

pub(crate) struct NodeInputUpdate {
    pub(crate) objects: Vec<NodeInputObjectUpdate>,
    pub(crate) relations_changed: bool,
}

// cpp: layoutng/internal/layout_object_builder.h:75-77
// cpp: layoutng/internal/boundary/layout_boundary.cc:2181-2191
// cpp: layoutng/internal/boundary/layout_boundary.cc:2355-2362
// Input identities and parent attachments are stable between attachment
// transactions. This is the boundary equivalent of Node::GetLayoutObject;
// local updates must not search every resident object for one DOM identity.
#[derive(Default)]
struct ManagedInputIndex {
    by_id: HashMap<u64, Vec<InputKey>>,
    children: HashMap<*mut LayoutObject, Vec<InputKey>>,
}

pub struct LayoutObjectTreeStorage {
    fonts: Box<dyn NativeFontResolver>,
    factories: LayoutObjectFactorySet,
    // Canonical allocation roots, indexed for removal by the exact retired
    // object. Subtree updates must not census every live allocation to unpin one.
    metadata: HashMap<*mut Node, Persistent<Node>>,
    objects: HashMap<*mut LayoutObject, Persistent<LayoutObject>>,
    retired: Vec<(*mut LayoutObject, *mut Node)>,
    root: Persistent<LayoutObject>,
    // The native objects and their input owners have stable storage. Hash
    // bucket updates move handles, never the large style/metadata descriptors.
    managed: HashMap<InputKey, Box<ManagedObject>>,
    input_index: std::cell::OnceCell<ManagedInputIndex>,
    partial_index: Option<ManagedInputIndex>,
    attached_keys: Vec<InputKey>,
    pending: HashMap<InputKey, Box<ManagedObject>>,
    occurrences: HashMap<(u64, i32, bool, Arc<str>), usize>,
    last_child: HashMap<*mut LayoutObject, *mut Node>,
    updating: bool,
    reattach: std::collections::HashSet<u64>,
    stats: LayoutTreeUpdateStats,
    editing: std::rc::Rc<crate::editing_state::LayoutEditingState>,
}

impl LayoutObjectTreeStorage {
    fn new(space: &ConstraintSpace, factories: &LayoutObjectFactorySet) -> Self {
        let _heap_scope = LayoutHeapScope::new();
        let fonts = ConstructionFontResolver(space);
        Self {
            fonts,
            factories: *factories,
            metadata: HashMap::new(),
            objects: HashMap::new(),
            retired: Vec::new(),
            root: Persistent::from_ptr(std::ptr::null_mut()),
            managed: HashMap::new(),
            input_index: std::cell::OnceCell::new(),
            partial_index: None,
            attached_keys: Vec::new(),
            pending: HashMap::new(),
            occurrences: HashMap::new(),
            last_child: HashMap::new(),
            updating: false,
            reattach: Default::default(),
            stats: LayoutTreeUpdateStats::default(),
            editing: std::rc::Rc::new(Default::default()),
        }
    }

    fn InputIndex(&self) -> &ManagedInputIndex {
        if let Some(index) = self.input_index.get() {
            index
        } else {
            BuildManagedInputIndex(self)
        }
    }
    fn InputKeys(&self, id: u64) -> &[InputKey] {
        self.InputIndex()
            .by_id
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    fn ChildKeys(&self, parent: *mut LayoutObject) -> &[InputKey] {
        self.InputIndex()
            .children
            .get(&parent)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn Place(&mut self, parent: *mut LayoutObject, object: *mut LayoutObject, node: *mut Node) {
        layout_invalidation_package::place(self, parent, object, node)
    }
}

impl Drop for LayoutObjectTreeStorage {
    fn drop(&mut self) {
        let root = self.root.Get();
        if !root.is_null() && !unsafe { &*root }.WasDestroyedForTreeUpdate() {
            let _object_factory_scope = LayoutObjectFactoryScope::new(&self.factories);
            let _allow_destroy = AllowDestroyingLayoutObjectInFinalizerScope::new();
            unsafe { &mut *root }.Destroy();
        }
    }
}

impl Drop for LayoutObjectTree {
    // cpp: layoutng/internal/layout_object_builder.h:24-24
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2376-2376
    fn drop(&mut self) {
        if !self.storage_.is_null() {
            let _heap_scope = LayoutHeapScope::new();
            unsafe { drop(Box::from_raw(self.storage_)) };
            self.storage_ = std::ptr::null_mut();
        }
    }
}

impl LayoutObjectTree {
    #[inline(always)]
    pub(crate) fn OutputNode(&self, id: u64) -> Option<&LayoutObject> {
        layout_commit_native_output_node_lookup::output_node(self, id)
    }

    #[inline(always)]
    pub(crate) fn OutputNodeMut(&mut self, id: u64) -> Option<&mut LayoutObject> {
        layout_commit_native_output_node_lookup::output_node_mut(self, id)
    }
    pub(crate) fn OutputScrollOffsets(&self) -> Vec<(u64, super::layout_input::Offset)> {
        layout_commit_scroll_offset_snapshot::output_scroll_offsets(self)
    }
    // cpp: layoutng/internal/layout_object_builder.h:22-23
    pub fn new(space: &ConstraintSpace) -> Self {
        unsafe { LayoutObjectTreeNew(space) }
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:2364-2374
    pub fn new_with_factories(space: &ConstraintSpace, factories: &LayoutObjectFactorySet) -> Self {
        InitStringStatics();
        Length::Initialize();
        Self {
            storage_: Box::into_raw(Box::new(LayoutObjectTreeStorage::new(space, factories))),
        }
    }

    /// Reconcile one projection while keeping the tree reusable after a panic.
    pub fn Update<T>(&mut self, build: impl FnOnce(&mut Self) -> T) -> T {
        self.BeginUpdate();
        self.CompleteUpdate(build)
    }

    // Element::RebuildLayoutTree keeps clean branches outside the attachment
    // transaction. Validate the existing parent/anchor before changing anything;
    // anonymous or flattened attachment contexts need a wider dirty root.
    pub fn UpdateSubtree(
        &mut self,
        id: u64,
        parent_id: u64,
        build: impl FnOnce(&mut Self, &mut LayoutObject),
    ) -> bool {
        layout_invalidation_package::update_subtree(self, id, parent_id, build)
    }

    fn CompleteUpdate<T>(&mut self, build: impl FnOnce(&mut Self) -> T) -> T {
        layout_invalidation_package::complete_update(self, build)
    }

    /// Give a one-shot input adapter only attachment operations. The host
    /// retains ownership and transaction control; no additional Update begins.
    pub fn WithAttachment<T>(
        &mut self,
        build: impl FnOnce(&mut crate::layout_engine::LayoutTreeUpdate<'_>) -> T,
    ) -> T {
        build(&mut crate::layout_engine::LayoutTreeUpdate::new(self))
    }

    pub fn BeginUpdate(&mut self) {
        layout_invalidation_package::begin_update(self)
    }

    pub fn FinishUpdate(&mut self) {
        layout_invalidation_package::finish_update(self)
    }

    pub(crate) fn Reattach(&mut self, node_ids: &[u64]) {
        layout_invalidation_package::reattach(self, node_ids)
    }

    pub fn UpdateStats(&self) -> &LayoutTreeUpdateStats {
        &unsafe { &*self.storage_ }.stats
    }

    pub(crate) fn HasRoot(&self) -> bool {
        let root = unsafe { &*self.storage_ }.root.Get();
        !root.is_null() && !unsafe { &*root }.WasDestroyedForTreeUpdate()
    }

    // Rust's ownership move preserves the source's move-only tree contract;
    // Clone and Copy are deliberately absent.
    // cpp: layoutng/internal/layout_object_builder.h:25-28
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2378-2381
    pub fn Move(self) -> Self {
        self
    }

    /// Dispatch native virtuals with the factories retained by this tree.
    /// Keep the scope synchronous so its thread-local pointer cannot outlive
    /// the storage which owns the actual SVG/table assembly callbacks.
    pub(crate) fn WithObjectFactoryScope<T>(&self, operation: impl FnOnce() -> T) -> T {
        let storage = unsafe { &*self.storage_ };
        let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
        operation()
    }

    // cpp: layoutng/internal/layout_object_builder.h:30-35
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2383-2392
    pub fn CreateRoot(
        &mut self,
        id: u64,
        style: &ComputedStyle,
        debug_name: String,
        element: Option<ElementData>,
        first_line_style: *const ComputedStyle,
    ) -> &mut LayoutObject {
        let object = unsafe { &mut *self.storage_ }.Create(
            std::ptr::null_mut(),
            id,
            NodeKind::kBox,
            style,
            element,
            debug_name,
            String::new(),
            std::ptr::null(),
            false,
            std::ptr::null(),
            first_line_style,
            None,
        );
        unsafe { &mut *object }
    }

    pub fn CreateRootDefault(&mut self, id: u64, style: &ComputedStyle) -> &mut LayoutObject {
        self.CreateRoot(id, style, String::new(), None, std::ptr::null())
    }

    // cpp: layoutng/internal/layout_object_builder.h:39-47
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2394-2408
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
        if kind == NodeKind::kText || kind == NodeKind::kReplaced {
            panic!("AddBox requires a box-producing NodeKind");
        }
        let object = unsafe { &mut *self.storage_ }.Create(
            parent,
            id,
            kind,
            style,
            element,
            debug_name,
            String::new(),
            std::ptr::null(),
            style_generated,
            std::ptr::null(),
            first_line_style,
            None,
        );
        unsafe { &mut *object }
    }

    pub fn AddBoxDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
    ) -> &mut LayoutObject {
        self.AddBox(
            parent,
            id,
            style,
            String::new(),
            None,
            NodeKind::kBox,
            false,
            std::ptr::null(),
        )
    }

    // cpp: layoutng/internal/layout_object_builder.h:49-55
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2410-2421
    pub fn AddReplaced(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
        element: Option<ElementData>,
        debug_name: String,
        kind: NodeKind,
    ) -> &mut LayoutObject {
        if kind != NodeKind::kReplaced && kind != NodeKind::kFrame {
            panic!("AddReplaced requires a replaced NodeKind");
        }
        let object = unsafe { &mut *self.storage_ }.Create(
            parent,
            id,
            kind,
            style,
            element,
            debug_name,
            String::new(),
            std::ptr::null(),
            false,
            std::ptr::null(),
            std::ptr::null(),
            None,
        );
        unsafe { &mut *object }
    }

    /// Anonymous replaced child created by generated ContentData. Chromium's
    /// ImageContentData::CreateLayoutObject creates a LayoutImage and applies
    /// the originating pseudo style; keep that identity distinct from a real
    /// DOM replaced element while using the same translated replaced factory.
    pub fn AddGeneratedReplaced(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
        element: ElementData,
        debug_name: String,
    ) -> &mut LayoutObject {
        let object = unsafe { &mut *self.storage_ }.Create(
            parent,
            id,
            NodeKind::kReplaced,
            style,
            Some(element),
            debug_name,
            String::new(),
            std::ptr::null(),
            true,
            std::ptr::null(),
            std::ptr::null(),
            None,
        );
        unsafe { &mut *object }
    }

    pub fn AddReplacedDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
    ) -> &mut LayoutObject {
        self.AddReplaced(parent, id, style, None, String::new(), NodeKind::kReplaced)
    }

    // cpp: layoutng/internal/layout_object_builder.h:61-67
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2423-2433
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
        let object = unsafe { &mut *self.storage_ }.Create(
            parent,
            id,
            NodeKind::kText,
            &ComputedStyle::default(),
            None,
            debug_name,
            text,
            style,
            style_generated,
            source_style,
            std::ptr::null(),
            None,
        );
        unsafe { &mut *object }
    }

    pub fn AddTextDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        text: String,
    ) -> &mut LayoutObject {
        self.AddText(
            parent,
            id,
            text,
            String::new(),
            std::ptr::null(),
            false,
            std::ptr::null(),
        )
    }

    // cpp: layoutng/internal/layout_object_builder.h:69-71
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2435-2451
    pub fn NeedsCollapsibleWhitespace(&self, parent: &LayoutObject) -> bool {
        layout_algo_inline_collapsible_whitespace::needs_collapsible_whitespace(self, parent)
    }

    /// Attach the shared editing state to the resident layout tree.
    pub fn SetEditingState(
        &mut self,
        state: std::rc::Rc<crate::editing_state::LayoutEditingState>,
    ) {
        unsafe { &mut *self.storage_ }.editing = state;
    }
    pub fn EditingState(&self) -> std::rc::Rc<crate::editing_state::LayoutEditingState> {
        unsafe { &*self.storage_ }.editing.clone()
    }
    /// Invalidate the affected control and its container chain, retaining siblings.
    pub fn InvalidateEditing(&mut self, id: u64) -> bool {
        layout_invalidation_input_updates::invalidate_editing(self, id)
    }

    /// Invalidate selection drawing without invalidating text-control geometry.
    ///
    /// Chromium's LayoutSelection marks every LayoutObject whose selection
    /// state or boundary changed, and ObjectPaintInvalidator converts that bit
    /// to kSelection invalidation. Rechrom records a control's selection with
    /// the text object's ordinary display-item client, so invalidate the
    /// resident selection leaves below the control input before the paint-only
    /// export. Text controls use generated editor/text inputs with derived ids,
    /// so looking up only the host id would miss the client that records glyphs.
    ///
    /// Source: third_party/blink/renderer/core/editing/layout_selection.cc:
    ///         217-334, 954-977;
    ///         core/paint/object_paint_invalidator.cc:96-116.
    pub fn InvalidateSelectionPaint(&mut self, id: u64) -> bool {
        layout_invalidation_input_updates::invalidate_selection_paint(self, id)
    }

    /// Synchronize ordinary scroll paint metadata while retaining geometry,
    /// node identity and the owning tree's source descriptor.
    pub fn UpdateScrollOffset(&mut self, id: u64, offset: super::layout_input::Offset) -> bool {
        layout_fragment_scroll_input::update_scroll_offset(self, id, offset)
    }

    /// Update the source style for one real element after style resolution has
    /// proved that geometry and tree attachment are unchanged. This is the
    /// targeted counterpart of the paint-only branch in Create(): the native
    /// layout style/result remain resident, while fragment-local paint data reads the new
    /// paint data from the element metadata.
    pub fn UpdatePaintStyle(
        &mut self,
        id: u64,
        style: &NativeComputedStyle,
        clip_resource_version: super::paint_input::SvgClipResourceVersion,
    ) -> Option<PaintStyleBinding> {
        layout_fragment_paint_input::update_paint_style(self, id, style, clip_resource_version)
    }

    /// Element::RecalcOwnStyle updates an attached LayoutObject through
    /// SetStyle; ordinary geometry changes do not reattach every sibling.
    /// Reuse Create's style/metadata implementation after proving the native
    /// object kind and attachment are unchanged. Complex generated sources
    /// retain the ordinary attachment transaction.
    pub fn UpdateNodeInput(
        &mut self,
        id: u64,
        style: &ComputedStyle,
        element: Option<ElementData>,
    ) -> Option<NodeInputUpdate> {
        layout_invalidation_node_input::update_node_input(self, id, style, element)
    }

    /// Element::RecalcOwnStyle installs local style/theme inputs on existing
    /// LayoutObjects. Admission retains native geometry and rejects changes
    /// requiring the fragment/overflow lifecycle (ComputedStyle::VisualInvalidationDiff).
    fn PaintInputPlan(
        &self,
        id: u64,
        style: &ComputedStyle,
        element: &Option<ElementData>,
    ) -> Option<(std::collections::HashSet<u64>, Vec<u64>)> {
        layout_fragment_paint_input::paint_input_plan(self, id, style, element)
    }

    /// Query geometry validation must not install paint/resource bindings.
    pub(crate) fn GeometryInputMatches(
        &self,
        id: u64,
        style: &ComputedStyle,
        element: &Option<ElementData>,
    ) -> bool {
        self.PaintInputPlan(id, style, element).is_some()
    }

    pub fn UpdatePaintInput(
        &mut self,
        id: u64,
        style: &ComputedStyle,
        element: Option<ElementData>,
    ) -> Option<PaintInputUpdate> {
        layout_fragment_paint_input::update_paint_input(self, id, style, element)
    }

    /// Exclusive access to the tree-owned root for the layout entry point.
    pub fn RootMut(&mut self) -> &mut LayoutObject {
        let root = unsafe { &mut *self.storage_ }.root.Get();
        assert!(!root.is_null(), "LayoutObjectTree has no root");
        unsafe { &mut *root }
    }

    // cpp: layoutng/internal/layout_object_builder.h:73-73
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2453-2457
    pub fn Root(&self) -> &LayoutObject {
        let root = unsafe { &*self.storage_ }.root.Get();
        if root.is_null() {
            panic!("LayoutObjectTree has no root");
        }
        unsafe { &*root }
    }
}
