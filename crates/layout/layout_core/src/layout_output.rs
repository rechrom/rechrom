//! Borrowed access to the engine's resident layout result. No exported tree,
//! DOM ownership or source LayoutObject handle crosses this interface.
use super::*;
use crate::internal::{
    layout_box_model_object::LayoutBoxModelObject, layout_inline::LayoutInline,
    layout_node_metadata::Element,
};
use foundation::{gfx, PhysicalOffset, PhysicalRect, PhysicalSize};
use layout_fragment::fragment_tree::{
    FragmentPaintData, LayoutResultHandle, PaintPropertiesHandle,
};
use layout_fragment::fragment_data::{FragmentPaintBinding, FragmentPaintSidecar};
use layout_fragment::layout_result::LayoutResult;
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
use layout_fragment::physical_fragment::PhysicalFragment;
use layout_geometry::geometry::physical_rect::PhysicalRectExt;
use layout_style::style::style_transform::{
    ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
    ApplyTransformOrigin,
};
use std::cell::{Cell, UnsafeCell};
use std::collections::HashMap;
use std::sync::Arc;
#[path = "native_hit_test.rs"]
mod native_hit_test;
#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/spatial_navigation_geometry.rs"]
mod spatial_navigation_geometry;
pub use native_hit_test::{
    NativeHitTestChildren, NativeHitTestCursor, NativeHitTestInlineItem, NativeHitTestInlineItems,
    NativeHitTestTarget,
};

/// Opaque lifetime lease for the canonical per-result Paint sidecar. It
/// carries no Layout API and exposes no storage to Paint.
#[derive(Clone)]
pub struct PaintFragmentLease {
    owner: Arc<dyn std::any::Any>,
    generation: u64,
}

impl PaintFragmentLease {
    pub fn PtrEq(&self, other: &Self) -> bool {
        self.generation == other.generation && Arc::ptr_eq(&self.owner, &other.owner)
    }
}

#[path = "../../layout_fragment/src/fragment_client_rect_queries.rs"]
mod layout_fragment_fragment_client_rect_queries;
#[path = "../../layout_fragment/src/fragment_cursor_children.rs"]
mod layout_fragment_fragment_cursor_children;
#[path = "../../layout_fragment/src/layout_metrics.rs"]
mod layout_fragment_layout_metrics;
#[path = "../../layout_fragment/src/text_fragment_queries.rs"]
mod layout_fragment_text_fragment_queries;

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/editing_text_runs.rs"]
mod layout_algo_inline_editing_text_runs;
#[cfg(feature = "translation_in_progress")]
pub use layout_algo_inline_editing_text_runs::{
    EditingTextIteratorBehavior, EditingTextRun, EditingTextRunCursor,
};

#[path = "../../layout_algo_inline/src/layout_output_editing_queries.rs"]
mod layout_algo_inline_layout_output_editing_queries;

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/visual_caret_movement.rs"]
mod layout_algo_inline_visual_caret_movement;
#[cfg(feature = "translation_in_progress")]
pub use layout_algo_inline_visual_caret_movement::{
    EditingCaretDirections, VisualCaretMoveDirection, VisualCaretMoveResult,
};

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/editing_line_movement.rs"]
mod layout_algo_inline_editing_line_movement;
#[cfg(feature = "translation_in_progress")]
pub use layout_algo_inline_editing_line_movement::EditingLinePositionResult;

#[path = "../../layout_algo_inline/src/editing_text_offset_mapping.rs"]
mod layout_algo_inline_editing_text_offset_mapping;
pub use layout_algo_inline_editing_text_offset_mapping::EditingInlineContentsBlockIdentity;
#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/editing_text_iterator_facts.rs"]
mod layout_algo_inline_editing_text_iterator_facts;
#[cfg(feature = "translation_in_progress")]
pub use layout_algo_inline_editing_text_iterator_facts::{
    EditingAnonymousPlaceholderFacts, EditingBackwardsTextFacts, EditingBlockFlowPlaceholderFacts,
    EditingMovePageFacts, EditingParagraphLayoutFacts, EditingParagraphTextFacts,
    EditingTextIteratorObjectFacts,
};

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/editing_inner_text.rs"]
mod editing_inner_text;
#[cfg(feature = "translation_in_progress")]
pub use editing_inner_text::InnerTextLayoutFacts;

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/editing_visible_bounds.rs"]
mod editing_visible_bounds;

#[derive(Clone, Copy)]
pub struct LayoutOutput<'a> {
    pub(super) engine: &'a LayoutEngine,
}
pub struct LayoutNode<'a> {
    output: LayoutOutput<'a>,
    object: &'a LayoutObject,
}
impl LayoutEngine {
    /// EventHandler::PerformHitTest admission, event_handler.cc:374-387.
    /// The resident LayoutView's layout history survives dirty result removal.
    pub fn HasCompletedLayoutForHitTest(&self) -> bool {
        self.tree_
            .as_ref()
            .is_some_and(|tree| tree.Root().EverHadLayout())
    }
    /// Node::GetLayoutObject reads the resident attachment registry, including
    /// synchronous detach. This does not require/publish a fragment result.
    pub fn CurrentLayoutNode(&self, id: u64) -> Option<LayoutNode<'_>> {
        Some(LayoutNode {
            output: LayoutOutput { engine: self },
            object: self.tree_.as_ref()?.GetLayoutObjectForNode(id)?,
        })
    }
}

impl LayoutNode<'_> {
    /// MoveCommands::VerticalScrollDistance reads the focused native LayoutBox
    /// and the real layout viewport scroll area, move_commands.cc:65-91.
    pub fn EditingPageMovementFacts(&self) -> Option<(bool, i32, i32)> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(self.object).as_ref()
        }?;
        let root = self.output.engine.tree_.as_ref()?.Root();
        let viewport = unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(root).as_ref()
        }?;
        let scrollable = unsafe { viewport.GetScrollableArea().as_ref() }?;
        Some((
            matches!(
                self.object.StyleRef().OverflowY(),
                foundation::EOverflow::kScroll | foundation::EOverflow::kAuto
            ),
            box_.PhysicalPaddingBoxRect().size.height.ToInt(),
            scrollable
                .VisibleContentRect(
                    crate::internal::scroll_types::IncludeScrollbarsInRect::kExcludeScrollbars,
                )
                .height(),
        ))
    }
    /// LayoutEmbeddedContent::UpdateGeometry, cc:433-481. Child layout uses
    /// the pixel-snapped replaced rect, including its absolute subpixel origin.
    /// Transform/scroll mapping stays on the original native LayoutObject.
    pub fn EmbeddedChildFrameSize(&self) -> Option<gfx::Size> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let replaced = unsafe {
            DynamicTo::<crate::internal::layout_replaced::LayoutReplaced>(self.object).as_ref()
        }?;
        let mut rect = replaced.ReplacedContentRect();
        rect.offset += self
            .object
            .LocalToAbsolutePointPhysical(&PhysicalOffset::default(), Default::default());
        Some(foundation::ToPixelSnappedRect(rect).size())
    }
    /// Source: LayoutObject::IsSelectable. This narrow production query keeps
    /// label/radio selection preparation on the resident LayoutObject without
    /// enabling the unfinished general editing geometry surface.
    /// Node::GetLayoutObject's canonical source Node association. The returned
    /// owner is retained, never a copied DOM or geometry representation.
    pub fn NativeNodeOwner(
        &self,
    ) -> Option<foundation::Persistent<crate::internal::layout_node_metadata::Node>> {
        let node = self.object.GetNode();
        (!node.is_null()).then(|| foundation::Persistent::from_ptr(node))
    }
    pub fn EditingIsSelectable(&self) -> bool {
        self.object.IsSelectable()
    }
}

/// Pointer-preserving cursor over LayoutNG's canonical fragment graph. It
/// carries no projected geometry or owned descendants. Inline cursors point
/// directly into FragmentItems; physical cursors point directly at the
/// PhysicalFragment retained by LayoutResult.
#[derive(Clone, Copy)]
pub enum FragmentCursor<'a> {
    Physical {
        fragment: &'a PhysicalFragment,
        offset: PhysicalOffset,
    },
    Inline {
        item: &'a layout_fragment::fragment_item::FragmentItem,
        items: &'a layout_fragment::fragment_items::FragmentItems,
        container: &'a PhysicalBoxFragment,
        index: usize,
        offset: PhysicalOffset,
    },
}

/// Paint cursor over the same source links. Dereferencing resolves the owner's
/// FragmentData Paint sidecar (LayoutPaintState::Slot); `Children` always follows
/// PhysicalFragment/FragmentItem pointers and never a copied child vector.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum PaintFragmentOwner {
    Physical(*const PhysicalFragment),
    Inline(*const layout_fragment::fragment_items::FragmentItems, usize),
}

impl PaintFragmentOwner {
    /// The live identity of this resident owner, computed from the owner
    /// itself on every query (no publication census or identity map). Never
    /// 0: an owner without a Chromium identity pair (a line box fragment, an
    /// unnumbered or container-owned inline item, the last fragmentainer)
    /// falls back to a key derived from the owner itself.
    fn InstanceId(self) -> u64 {
        match self {
            Self::Physical(fragment) => {
                let fragment = unsafe { &*fragment };
                // A box's FragmentData::UniqueId is read from the entry of its
                // fragment index; establish the entries first
                // (GetOrCreateFragmentData, pre_paint_tree_walk.cc).
                if fragment.IsBox() {
                    if let Some(object) = unsafe { fragment.GetLayoutObject().as_ref() } {
                        crate::internal::boundary::layout_boundary::SynchronizeBoxFragmentData(
                            object,
                        );
                    }
                }
                layout_fragment::fragment_tree::PhysicalFragmentInstanceId(fragment)
            }
            Self::Inline(items, index) => {
                layout_fragment::fragment_tree::InlineItemInstanceId(unsafe { &*items }, index)
            }
        }
    }

    /// The opaque address pair a sidecar binding records for this owner.
    fn Key(self) -> (usize, usize) {
        match self {
            Self::Physical(fragment) => (fragment as usize, usize::MAX),
            Self::Inline(items, index) => (items as usize, index),
        }
    }
}

/// Process-unique serials for paint publications and resource epochs. A
/// sidecar lives on the persistent FragmentData, so a binding made through
/// one LayoutResult must compare directly with the publications of the
/// results that replace it.
fn NextPaintSerial() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// The FragmentData entry index of a box fragment whose LayoutObject was
/// destroyed while a retained LayoutResult still reaches it. LayoutBox's
/// result list is gone, so derive FragmentDataFromPhysicalFragment's index
/// (BoxFragmentIndex, fragmentation_utils.cc:1443-1447) from the fragment's
/// own break token: the token after fragment `i` has SequenceNumber `i`.
fn DestroyedBoxFragmentIndex(fragment: &PhysicalFragment, list_size: u32) -> u32 {
    let box_fragment =
        unsafe { &*(fragment as *const PhysicalFragment as *const PhysicalBoxFragment) };
    if box_fragment.IsFirstForNode() {
        return 0;
    }
    let token = box_fragment.GetBreakToken();
    let index = if token.is_null() {
        list_size.saturating_sub(1)
    } else {
        unsafe { &*token }.SequenceNumber()
    };
    index.min(list_size.saturating_sub(1))
}

/// Per-result Paint publication state. It holds no per-fragment directory:
/// each fragment occurrence's FragmentPaintData sidecar lives on the
/// FragmentData Chromium keeps that occurrence's paint state on
/// (core/paint/fragment_data.h) — a box fragment's own FragmentData
/// (PhysicalBoxFragment::GetFragmentData()), the LayoutObject's first
/// FragmentData keyed by FragmentItem::FragmentId for an inline item, the
/// fragmentation-context owner's for a fragmentainer — so it survives a
/// replacement LayoutResult exactly as Chromium's FragmentData does, with no
/// seeding. PrePaintTreeWalk brings the FragmentData of each object it
/// reaches up to date (pre_paint_tree_walk.cc, GetOrCreateFragmentData /
/// WalkInternal); likewise the first cursor of a publication to reach an
/// occurrence rebinds and refreshes its sidecar from the owner.
///
/// Lifetime: the state retains its LayoutResult. A PaintFragmentLease (or
/// handle) therefore keeps the result's fragments alive, they keep every
/// LayoutObject they reach (destroyed or not) through their Member edges,
/// and each object keeps its FragmentDataList and sidecars; a destroyed
/// object's sidecars are released by the GC only once no retained result
/// reaches it. The one way a sidecar can lose its FragmentData earlier is
/// FragmentDataList::Shrink of a fragmented box; such sidecars are pinned by
/// every state that reached them (`pinned`).
pub(crate) struct LayoutPaintState {
    /// The LayoutResult this state publishes, held for lifetime only (see
    /// the note above).
    #[allow(dead_code)]
    result: foundation::Persistent<()>,
    /// Sidecars of occurrences without a Chromium FragmentData: line box
    /// fragments, fragments and items without a LayoutObject, the last
    /// fragmentainer (no break token). Keyed by the owner-derived instance
    /// id, so such a sidecar is retained exactly while LayoutNG reuses the
    /// immutable owner. A superseded state also keeps here the private
    /// sidecars of occurrences it first reaches after its replacement was
    /// published.
    fallback:
        UnsafeCell<foundation::TrustedIdentityHashMap<u64, std::rc::Rc<FragmentPaintSidecar>>>,
    /// Sidecars reached through a non-first FragmentData entry, keyed by the
    /// sidecar's address; lifetime only, never looked up by fragment.
    pinned: UnsafeCell<HashMap<usize, std::rc::Rc<FragmentPaintSidecar>>>,
    /// A replacement LayoutResult's state was published after this one. Its
    /// handles stay readable but never rebind a shared sidecar.
    superseded: Cell<bool>,
    /// LayoutInline objects whose FragmentData list this publication already
    /// synchronized with their inline containers.
    synchronized_inlines: UnsafeCell<std::collections::HashSet<usize>>,
    /// Serial of the current publication (NextPaintSerial).
    publication: Cell<u64>,
    generation: Cell<u64>,
    /// Serial advanced whenever a publication installs document resources
    /// that are incompatible with the ones the sidecars were exported with.
    resource_epoch: Cell<u64>,
    /// The Layout boundary's PaintStyle projection cache generation lent to
    /// the current publication.
    exported_styles: Cell<Option<crate::internal::layout_input::ExportedStyleCacheKey>>,
    /// The document's paint resources (fonts, images, canvases, device scale,
    /// layout zoom, base background, viewport) for this result. Chromium
    /// reads these from the Document / LocalFrame / LocalFrameView / Settings
    /// that own the painted LayoutObjects, never from a per-fragment copy, so
    /// every fragment of the result shares this one value.
    document_resources: UnsafeCell<Option<Arc<layout_fragment::fragment_tree::PaintResources>>>,
}

impl LayoutPaintState {
    pub(crate) fn New(result: &LayoutResult) -> Arc<Self> {
        Arc::new(Self {
            result: foundation::Persistent::from_ptr(
                result as *const LayoutResult as *mut LayoutResult as *mut (),
            ),
            fallback: UnsafeCell::new(Default::default()),
            pinned: UnsafeCell::new(Default::default()),
            superseded: Cell::new(false),
            synchronized_inlines: UnsafeCell::new(Default::default()),
            publication: Cell::new(0),
            generation: Cell::new(1),
            resource_epoch: Cell::new(NextPaintSerial()),
            exported_styles: Cell::new(None),
            document_resources: UnsafeCell::new(None),
        })
    }

    /// The document paint resources this result's fragments paint with.
    pub(crate) fn DocumentResources(
        &self,
    ) -> Option<&Arc<layout_fragment::fragment_tree::PaintResources>> {
        unsafe { &*self.document_resources.get() }.as_ref()
    }

    /// Install the document's current paint resources. Returns whether the
    /// value differs from the previously installed one; a pointer-equal or
    /// value-equal catalog keeps its identity, as ImageResourceContent and
    /// Font keep theirs.
    pub(crate) fn SetDocumentResources(
        &self,
        resources: Arc<layout_fragment::fragment_tree::PaintResources>,
    ) -> bool {
        let slot = unsafe { &mut *self.document_resources.get() };
        let changed = slot.as_ref().is_none_or(|previous| {
            !Arc::ptr_eq(previous, &resources) && previous.as_ref() != resources.as_ref()
        });
        if changed {
            *slot = Some(resources);
        }
        changed
    }

    /// Continue the document-level facts of the state a replacement
    /// LayoutResult supersedes. FragmentData sidecars need nothing: they stay
    /// on their LayoutObjects across relayout, as Chromium's FragmentData /
    /// ObjectPaintProperties do. Only the owner-keyed fallback sidecars the
    /// previous publication bound are shared, since LayoutNG may reuse those
    /// immutable owners; the sidecars were last exported with the previous
    /// document resources, which the publication compares against before
    /// installing its own.
    pub(crate) fn InheritFrom(&self, previous: &Self) {
        let last = previous.publication.get();
        let fallback = unsafe { &*previous.fallback.get() };
        let inherited: foundation::TrustedIdentityHashMap<_, _> = fallback
            .iter()
            .filter(|(_, sidecar)| sidecar.Binding().publication == last)
            .map(|(instance, sidecar)| (*instance, std::rc::Rc::clone(sidecar)))
            .collect();
        *unsafe { &mut *self.fallback.get() } = inherited;
        self.resource_epoch.set(previous.resource_epoch.get());
        *unsafe { &mut *self.document_resources.get() } = previous.DocumentResources().cloned();
    }

    /// A replacement LayoutResult's state now owns the sidecar bindings.
    pub(crate) fn Supersede(&self) {
        self.superseded.set(true);
    }

    pub(crate) fn BeginPublication(&self) {
        unsafe { &mut *self.synchronized_inlines.get() }.clear();
        self.publication.set(NextPaintSerial());
    }

    /// Install this publication's document resources and PaintStyle cache
    /// generation. `compatible`: whether the sidecars exported with the
    /// previously installed resources stay valid under these.
    pub(crate) fn InstallPublication(
        &self,
        resources: Arc<layout_fragment::fragment_tree::PaintResources>,
        compatible: bool,
        exported_styles: Option<crate::internal::layout_input::ExportedStyleCacheKey>,
    ) {
        if !compatible {
            self.resource_epoch.set(NextPaintSerial());
        }
        self.SetDocumentResources(resources);
        self.exported_styles.set(exported_styles);
    }

    pub(crate) fn FinishPublication(&self, topology_retained: bool) {
        if !topology_retained {
            self.generation.set(
                self.generation
                    .get()
                    .checked_add(1)
                    .expect("paint fragment generation exhausted"),
            );
        }
    }

    /// This state's owner-keyed fallback sidecar `instance`.
    fn FallbackSidecar(&self, instance: u64) -> *const FragmentPaintSidecar {
        let sidecar = unsafe { &mut *self.fallback.get() }
            .entry(instance)
            .or_insert_with(FragmentPaintSidecar::New);
        std::rc::Rc::as_ptr(sidecar)
    }

    /// Keep a sidecar reached through a FragmentData that
    /// FragmentDataList::Shrink may drop alive for this state's leases.
    fn Pin(&self, sidecar: &std::rc::Rc<FragmentPaintSidecar>) -> *const FragmentPaintSidecar {
        let pointer = std::rc::Rc::as_ptr(sidecar);
        unsafe { &mut *self.pinned.get() }
            .entry(pointer as usize)
            .or_insert_with(|| std::rc::Rc::clone(sidecar));
        pointer
    }

    /// The sidecar `owner`'s occurrence owns on its FragmentData:
    /// a box fragment's own entry (FragmentDataFromPhysicalFragment), the
    /// first entry of a non-box fragment's object, the object's keyed table
    /// for an inline item (DisplayItemClient, FragmentId) or a fragmentainer
    /// (owner, break-token sequence), else this state's fallback.
    fn ResolveSidecar(&self, owner: PaintFragmentOwner) -> *const FragmentPaintSidecar {
        match owner {
            PaintFragmentOwner::Physical(fragment) => {
                let fragment = unsafe { &*fragment };
                if fragment.IsFragmentainerBox() {
                    let box_fragment = unsafe {
                        &*(fragment as *const PhysicalFragment as *const PhysicalBoxFragment)
                    };
                    let instance =
                        layout_fragment::fragment_tree::FragmentainerInstanceId(box_fragment);
                    // A fragmentainer is not a CSS box: GetLayoutObject()
                    // hides the fragmentation-context owner it was laid out
                    // for, which owns its (owner, sequence) identity.
                    let context = fragment.layout_object_ptr();
                    if !context.is_null() && !box_fragment.GetBreakToken().is_null() {
                        let first = unsafe { &*context }.PaintSidecarFragments().front();
                        return std::rc::Rc::as_ptr(first.KeyedPaintSidecar(instance));
                    }
                    return self.FallbackSidecar(instance);
                }
                let object = fragment.GetLayoutObject();
                if object.is_null() || fragment.IsLineBox() {
                    return self.FallbackSidecar(
                        layout_fragment::fragment_tree::PhysicalFragmentInstanceId(fragment),
                    );
                }
                let object = unsafe { &*object };
                let list = object.PaintSidecarFragments();
                let fragment_data: &layout_fragment::fragment_data::FragmentData =
                    if !fragment.IsBox() {
                        list.front()
                    } else if object.WasDestroyedForTreeUpdate() {
                        list.at(DestroyedBoxFragmentIndex(fragment, list.size()))
                    } else {
                        // GetOrCreateFragmentData (pre_paint_tree_walk.cc):
                        // one FragmentData entry per physical fragment.
                        crate::internal::boundary::layout_boundary::SynchronizeBoxFragmentData(
                            object,
                        );
                        match unsafe { fragment.GetFragmentData().as_ref() } {
                            Some(fragment_data) => fragment_data,
                            None => list.front(),
                        }
                    };
                let sidecar = fragment_data.OwnPaintSidecar();
                if std::ptr::eq(fragment_data, list.front()) {
                    std::rc::Rc::as_ptr(sidecar)
                } else {
                    self.Pin(sidecar)
                }
            }
            PaintFragmentOwner::Inline(items, index) => {
                let items = unsafe { &*items };
                let instance = layout_fragment::fragment_tree::InlineItemInstanceId(items, index);
                match items
                    .Items()
                    .get(index)
                    .and_then(|item| unsafe { item.GetLayoutObject().as_ref() })
                {
                    Some(object) => std::rc::Rc::as_ptr(
                        object.PaintSidecarFragments().front().KeyedPaintSidecar(instance),
                    ),
                    None => self.FallbackSidecar(instance),
                }
            }
        }
    }

    /// The current publication's sidecar of `owner`, rebinding and
    /// refreshing it on the publication's first access. `display_item_fragment`
    /// is the display-item fragment scope the reaching walk carries for a
    /// physical owner, when known.
    fn Slot(
        &self,
        owner: PaintFragmentOwner,
        display_item_fragment: Option<u32>,
    ) -> *mut FragmentPaintData {
        let sidecar = unsafe { &*self.ResolveSidecar(owner) };
        let binding = sidecar.Binding();
        let publication = self.publication.get();
        let key = owner.Key();
        if binding.publication == publication && binding.owner == key {
            return Self::Current(sidecar, owner, binding, display_item_fragment);
        }
        // A replacement result's publication (a newer serial) rebound this
        // FragmentData's sidecar: as with Chromium's single FragmentData, an
        // older handle reads the same, current value and never rebinds it.
        if binding.publication > publication
            || (self.superseded.get() && binding.publication == publication)
        {
            return sidecar.Data();
        }
        if self.superseded.get() {
            // Reached through this superseded handle only now: refresh a
            // private sidecar from the old owner so the shared one keeps
            // the replacement's binding.
            let private = unsafe { &*self.FallbackSidecar(owner.InstanceId()) };
            let binding = private.Binding();
            if binding.publication == publication && binding.owner == key {
                return Self::Current(private, owner, binding, display_item_fragment);
            }
            return self.Bind(owner, private, display_item_fragment);
        }
        self.Bind(owner, sidecar, display_item_fragment)
    }

    /// A sidecar this publication already bound to `owner`. A first binding
    /// through a call-scoped cursor (no walk context) left the inherited
    /// display-item fragment scope unresolved; resolve it now.
    fn Current(
        sidecar: &FragmentPaintSidecar,
        owner: PaintFragmentOwner,
        mut binding: FragmentPaintBinding,
        display_item_fragment: Option<u32>,
    ) -> *mut FragmentPaintData {
        let data = sidecar.Data();
        if !binding.scope_known && display_item_fragment.is_some() {
            binding.scope_known = true;
            sidecar.SetBinding(binding);
            if let PaintFragmentOwner::Physical(fragment) = owner {
                if let Some(value) =
                    PhysicalDisplayItemFragment(unsafe { &*fragment }, display_item_fragment)
                {
                    let data = unsafe { &mut *data };
                    if data.paint.display_item_fragment != value {
                        data.paint.display_item_fragment = value;
                    }
                }
            }
        }
        data
    }

    /// Bind `sidecar` to `owner` for this publication and refresh it from the
    /// owner: the same FragmentData / (client, FragmentId) occurrence keeps
    /// the sidecar it last published, whichever immutable fragment of a
    /// replacement LayoutResult now carries it.
    fn Bind(
        &self,
        owner: PaintFragmentOwner,
        sidecar: &FragmentPaintSidecar,
        display_item_fragment: Option<u32>,
    ) -> *mut FragmentPaintData {
        // GetOrCreateFragmentData (pre_paint_tree_walk.cc): a LayoutInline's
        // FragmentData list matches its inline containers before the sidecar
        // is refreshed.
        if let PaintFragmentOwner::Inline(items, index) = owner {
            if let Some(object) = unsafe { &*items }
                .Items()
                .get(index)
                .and_then(|item| unsafe { item.GetLayoutObject().as_ref() })
            {
                self.SynchronizeInlineObjectFragmentData(object);
            }
        }
        let previous = sidecar.Binding();
        let epoch = self.resource_epoch.get();
        let key = owner.Key();
        let retained = previous.publication != 0 && previous.resource_epoch == epoch;
        let same_owner = previous.owner == key;
        sidecar.SetBinding(FragmentPaintBinding {
            publication: self.publication.get(),
            resource_epoch: epoch,
            owner: key,
            scope_known: match owner {
                PaintFragmentOwner::Physical(_) => display_item_fragment.is_some(),
                PaintFragmentOwner::Inline(..) => true,
            },
        });
        let pointer = sidecar.Data();
        let output = unsafe { &mut *pointer };
        let resources = self.DocumentResources().cloned();
        let exported_styles = self.exported_styles.get();
        match owner {
            PaintFragmentOwner::Physical(fragment) => {
                let fragment = unsafe { &*fragment };
                output.resident_owner = layout_fragment_boundary::ResidentFragmentOwner::Physical(
                    fragment as *const PhysicalFragment as usize,
                );
                crate::internal::boundary::layout_boundary::InitializePhysicalFragmentPaintData(
                    fragment,
                    output,
                    retained,
                    same_owner,
                    PhysicalDisplayItemFragment(fragment, display_item_fragment),
                    resources.as_ref(),
                    exported_styles,
                );
            }
            PaintFragmentOwner::Inline(items, index) => {
                output.resident_owner =
                    layout_fragment_boundary::ResidentFragmentOwner::InlineItem {
                        items: items as usize,
                        index,
                    };
                crate::internal::boundary::layout_boundary::InitializeInlineItemPaintData(
                    unsafe { &*items },
                    index,
                    output,
                    retained,
                    same_owner,
                    resources.as_ref(),
                    exported_styles,
                );
            }
        }
        pointer
    }

    /// SynchronizeInlineFragmentData once per LayoutInline and publication.
    fn SynchronizeInlineObjectFragmentData(&self, object: &LayoutObject) {
        if object.IsLayoutInline()
            && unsafe { &mut *self.synchronized_inlines.get() }
                .insert(object as *const LayoutObject as usize)
        {
            crate::internal::boundary::layout_boundary::SynchronizeInlineFragmentData(object);
        }
    }

    fn Generation(&self) -> u64 {
        self.generation.get()
    }
}

/// The display-item fragment a physical fragment paints under: its own scope
/// (a fragmentainer's break-token sequence, box_fragment_painter.cc:398-403,
/// 1211-1212; a layered box's fragment index in its LayoutBox,
/// paint_layer_painter.cc:776-778), else the scope `inherited` from the walk
/// that reached it.
fn PhysicalDisplayItemFragment(
    fragment: &PhysicalFragment,
    inherited: Option<u32>,
) -> Option<u32> {
    if !fragment.IsBox() {
        return inherited;
    }
    let box_fragment =
        unsafe { &*(fragment as *const PhysicalFragment as *const PhysicalBoxFragment) };
    if box_fragment.IsFragmentainerBox() {
        let token = box_fragment.GetBreakToken();
        return Some(if token.is_null() {
            0
        } else {
            unsafe { &*token }.SequenceNumber() + 1
        });
    }
    let Some(object) = (unsafe { box_fragment.GetLayoutObject().as_ref() }) else {
        return inherited;
    };
    if !object.HasLayer() {
        return inherited;
    }
    let mut fragments = crate::fragment_data_iterator::AccompaniedFragmentIterator::new(object);
    let mut index = 0;
    while let Some(physical) = fragments.GetPhysicalBoxFragment() {
        if std::ptr::eq(physical, box_fragment) {
            return Some(index);
        }
        index += 1;
        fragments.Advance();
    }
    inherited
}

#[derive(Clone, Copy)]
pub struct PaintFragmentCursor<'a> {
    source: FragmentCursor<'a>,
    state: Option<&'a LayoutPaintState>,
    /// The display-item fragment scope inherited from the walk that reached
    /// this cursor (ScopedDisplayItemFragment of the nearest enclosing
    /// fragmentainer, layered box or atomic inline); None for a call-scoped
    /// cursor rebound without a walk.
    display_item_fragment: Option<u32>,
}

/// Opaque borrowed identity of the live LayoutObject behind a Paint client.
/// Only Layout may project it back to its native layer; no object storage or
/// geometry crosses this identity port.
#[derive(Clone, Copy)]
pub struct PaintLayoutObjectIdentity<'a> {
    pub(crate) object: &'a LayoutObject,
}

impl PaintLayoutObjectIdentity<'_> {
    pub fn PtrEq(self, other: PaintLayoutObjectIdentity<'_>) -> bool {
        std::ptr::eq(self.object, other.object)
    }
}

/// Native Layout geometry facts consumed by source PaintLayerClipper.
/// Only the two queried rectangles cross this boundary; Layout keeps ownership.
pub struct PaintLayerClipGeometry {
    pub has_non_visible_overflow: bool,
    pub clips_overflow: bool,
    pub has_border_radius: bool,
    pub local_visual_rect: Option<PhysicalRect>,
    pub overflow_clip_rect: Option<PhysicalRect>,
}
impl<'a> PaintLayoutObjectIdentity<'a> {
    pub fn PaintTimingIdentity(self) -> u64 {
        self.object.PaintTimingIdentity()
    }

    /// Narrow identity projection for call-scoped clip geometry queries. The
    /// caller already borrows this live native object from the retained owner.
    pub fn ForClipPathGeometry(object: &'a LayoutObject) -> Self {
        object.CheckIsNotDestroyed();
        Self { object }
    }

    pub fn LayerClipGeometry(
        self,
        behavior: foundation::OverlayScrollbarClipBehavior,
        include_rects: bool,
    ) -> PaintLayerClipGeometry {
        let object = self.object;
        let box_ptr = foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object);
        let box_ = unsafe { box_ptr.as_ref() };
        let non_visible = box_.is_some_and(|box_| box_.ShouldClipOverflowAlongEitherAxis());
        let radius = object.StyleRef().HasBorderRadius();
        let visual = if !include_rects || !non_visible {
            None
        } else if object.IsGlobalRootScroller() {
            Some(
                unsafe { object.View().as_ref() }
                    .expect("global root scroller has view")
                    .ViewRect(),
            )
        } else {
            box_.map(|box_| box_.VisualOverflowRect())
        };
        PaintLayerClipGeometry {
            has_non_visible_overflow: non_visible,
            clips_overflow: non_visible || (object.IsLayoutEmbeddedContent() && radius),
            has_border_radius: radius,
            local_visual_rect: visual,
            overflow_clip_rect: if include_rects
                && (non_visible || (object.IsLayoutEmbeddedContent() && radius))
            {
                box_.map(|box_| box_.OverflowClipRectWithBehavior(behavior))
            } else {
                None
            },
        }
    }

    /// Source: ClipPathClipper::BorderBoxRect, clip_path_clipper.cc:84-111.
    pub fn ClipPathBorderBox(self) -> PhysicalRect {
        assert!(
            !self.object.IsSVGChild(),
            "SVG uses SVGResources reference geometry"
        );
        if let Some(box_) = unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(self.object).as_ref()
        } {
            return if box_.PhysicalFragmentCount() == 0 {
                PhysicalRect::default()
            } else {
                box_.PhysicalBorderBoxRect()
            };
        }
        let inline = unsafe { foundation::DynamicTo::<LayoutInline>(self.object).as_ref() }
            .expect("non-SVG clip box owner");
        if inline.IsInLayoutNGInlineFormattingContext() {
            let mut cursor = crate::inline_cursor::InlineCursor::default();
            cursor.MoveToLayoutObject(self.object);
            if !cursor.IsNull() {
                return cursor.Current().RectInContainerFragment();
            }
        }
        PhysicalRect::default()
    }
    pub fn ClipPathReferenceBox(
        self,
        geometry: layout_style::style::computed_style_constants::GeometryBox,
    ) -> PhysicalRect {
        let mut rect = self.ClipPathBorderBox();
        rect.Expand(&self.ReferenceBoxBorderBoxOutsets(geometry));
        rect
    }

    /// Source: GeometryBoxUtils::ReferenceBoxBorderBoxOutsets,
    /// core/paint/geometry_box_utils.cc:13-50.
    pub fn ReferenceBoxBorderBoxOutsets(
        self,
        geometry: layout_style::style::computed_style_constants::GeometryBox,
    ) -> layout_geometry::geometry::box_strut::PhysicalBoxStrut {
        use layout_style::style::computed_style_constants::GeometryBox;
        let object = self.object;
        assert!(
            !object.IsSVGChild(),
            "SVG effects use their native reference boxes"
        );
        let model = unsafe { foundation::DynamicTo::<LayoutBoxModelObject>(object).as_ref() }
            .expect("geometry box owner");
        let border = model.BorderOutsets();
        match geometry {
            GeometryBox::kBorderBox | GeometryBox::kStrokeBox | GeometryBox::kViewBox => {
                layout_geometry::geometry::box_strut::PhysicalBoxStrut::default()
            }
            GeometryBox::kPaddingBox => {
                layout_geometry::geometry::box_strut::PhysicalBoxStrut::new(
                    -border.top,
                    -border.right,
                    -border.bottom,
                    -border.left,
                )
            }
            GeometryBox::kContentBox | GeometryBox::kFillBox => {
                let padding = model.PaddingOutsets();
                layout_geometry::geometry::box_strut::PhysicalBoxStrut::new(
                    -border.top - padding.top,
                    -border.right - padding.right,
                    -border.bottom - padding.bottom,
                    -border.left - padding.left,
                )
            }
            GeometryBox::kMarginBox => {
                if object.IsBox() {
                    unsafe {
                        &*foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object)
                    }
                    .MarginOutsets()
                } else {
                    unsafe { &*foundation::DynamicTo::<LayoutInline>(object) }.MarginOutsets()
                }
            }
            GeometryBox::kHalfBorderBox => {
                layout_geometry::geometry::box_strut::PhysicalBoxStrut::new(
                    -border.top / 2,
                    -border.right / 2,
                    -border.bottom / 2,
                    -border.left / 2,
                )
            }
        }
    }
    /// Source: ComputeBorderShapeReferenceRects/ComputeBorderShapeOuterPath,
    /// core/paint/border_shape_utils.cc:13-51. Borrow the same compiled shape.
    pub fn BorderShapeOuterReferenceRect(
        self,
        mut rect: PhysicalRect,
        style: &layout_style::style::computed_style::ComputedStyle,
    ) -> PhysicalRect {
        let object = self.object;
        if style.HasBorderShape() && object.IsBoxModelObject() && !object.IsSVGChild() {
            let shape = unsafe { style.BorderShape().Get().as_ref() }.expect("source border-shape");
            rect.Expand(&self.ReferenceBoxBorderBoxOutsets(shape.OuterBox()));
        }
        rect
    }

    /// Source: paint_layer_clipper.cc:69-90. The caller supplies the feature
    /// gate; the native layer owner determines the exact active ancestor.
    pub fn ActiveUnboundedAncestor(self) -> Option<Self> {
        if !self.object.IsInclusiveDescendantOfUnboundedElement() {
            return None;
        }
        let mut layer = unsafe { self.object.EnclosingLayer().as_ref() };
        while let Some(current) = layer {
            let object = current.GetLayoutObject();
            if object.StyleRef().IsUnboundedElementActive() {
                return Some(Self { object });
            }
            layer = unsafe { current.Parent().as_ref() };
        }
        unreachable!("unbounded descendant has an active native ancestor")
    }
}

/// Source FragmentDataIterator/AccompaniedFragmentIterator projection. Inline
/// boxes in one container share one FragmentData entry, so only its first
/// exact native item is returned, matching PaintLayer::CollectFragments.
pub struct PaintObjectFragmentCursors<'a> {
    root: PaintFragmentCursor<'a>,
    object: &'a LayoutObject,
    fragments: Option<crate::fragment_data_iterator::AccompaniedFragmentIterator<'a>>,
    previous_data: Option<usize>,
}
impl<'a> Iterator for PaintObjectFragmentCursors<'a> {
    type Item = PaintFragmentCursor<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let fragments = self.fragments.as_mut()?;
        while !fragments.IsDone() {
            let data = fragments.GetFragmentData()? as *const _ as usize;
            let cursor = if let Some(fragment) = fragments.GetPhysicalBoxFragment() {
                // The retained result lease, not the temporary iterator, owns
                // this source fragment and its LayoutObject association.
                let fragment = unsafe { &*(fragment as *const PhysicalBoxFragment) };
                Some(self.root.ForPhysicalFragment(fragment))
            } else {
                fragments
                    .Cursor()
                    .and_then(|cursor| self.root.ForNativeInlineCursor(cursor))
            };
            fragments.Advance();
            if self.previous_data == Some(data) {
                continue;
            }
            let Some(cursor) = cursor else {
                continue;
            };
            if cursor.Cursor().GetLayoutObject() != self.object as *const _ {
                continue;
            }
            if cursor.PaintDataOption().is_none() {
                continue;
            }
            self.previous_data = Some(data);
            return Some(cursor);
        }
        None
    }
}

pub(crate) fn RootForHandle(handle: &LayoutResultHandle) -> &PhysicalFragment {
    let result = unsafe { handle.LayoutResult::<LayoutResult>() };
    assert!(
        !result.is_null(),
        "empty fragment handle has no source result"
    );
    unsafe { &*result }.GetPhysicalFragment()
}

impl<'a> PaintFragmentCursor<'a> {
    /// Source: LocalFrameView::UpdateLifecyclePhasesInternal(), immediately
    /// before PrePaintTreeWalk. Layout publication can be followed by layer
    /// tree mutations (for example, document-image or editing updates), so
    /// descendant-dependent PaintLayer flags must be made current at the
    /// consumer boundary rather than only when the LayoutResult is published.
    pub fn UpdateRootLayerDescendantDependentFlags(self) {
        let Some(object) = (unsafe { self.source.GetLayoutObject().as_ref() }) else {
            return;
        };
        let enclosing = object.EnclosingLayer();
        if enclosing.is_null() {
            return;
        }
        let root = unsafe { &*enclosing }.Root();
        if !root.is_null() {
            unsafe { &mut *root }.UpdateDescendantDependentFlags();
        }
    }

    /// Source BoxFragmentPainter/InlineBoxFragmentPainter's aggregation gate.
    /// Anonymous objects deliberately do not establish a new aggregation root.
    pub fn TextPaintTimingAggregator(self) -> Option<(u64, u64)> {
        let object = unsafe { self.source.GetLayoutObject().as_ref() }?;
        if !object.IsBox() && !object.IsLayoutInline() {
            return None;
        }
        let node = unsafe { object.GetNode().as_ref() }?;
        Some((node.InputId(), object.PaintTimingIdentity()))
    }
    pub fn TextPaintShouldSkipDrawing(self) -> bool {
        let object =
            unsafe { self.source.GetLayoutObject().as_ref() }.expect("native text LayoutObject");
        let font = unsafe { object.StyleRef().GetFont().as_ref() }.expect("canonical text Font");
        font.ShouldSkipDrawing()
    }

    /// The LayoutView root fragment's canvas background selection
    /// (ViewPainter, view_painter.cc:76-449; root/body transfer per
    /// layout_box_model_object.cc:1123-1151), read live from the resident
    /// root and body fragments. None for any other fragment, for documents
    /// without root background behavior, or when nothing would be painted.
    /// The style is the selected fragment's PaintStyle projection.
    pub fn CanvasBackground(
        self,
    ) -> Option<layout_fragment::fragment_tree::CanvasBackgroundPaintView<'a>> {
        let object = unsafe { self.source.GetLayoutObject().as_ref() }?;
        if !object.IsLayoutView() {
            return None;
        }
        crate::internal::boundary::layout_boundary::CanvasBackgroundForRoot(object, self)
    }

    /// CanvasBackground including a transparent selection (LayoutView
    /// background invalidation geometry).
    pub fn ViewBackground(
        self,
    ) -> Option<layout_fragment::fragment_tree::CanvasBackgroundPaintView<'a>> {
        let object = unsafe { self.source.GetLayoutObject().as_ref() }?;
        if !object.IsLayoutView() {
            return None;
        }
        crate::internal::boundary::layout_boundary::ViewBackgroundForRoot(object, self)
    }

    /// The document element's own background facts, reached from any fragment
    /// through its LayoutObject's View(): document.GetLayoutView()->StyleRef()
    /// as BoxFragmentPainter paints it for a kPageBorderBox. The style is the
    /// root fragment's PaintStyle projection, borrowed.
    pub fn DocumentElementBackground(
        self,
    ) -> Option<layout_fragment::fragment_tree::CanvasBackgroundPaintView<'a>> {
        let object = unsafe { self.source.GetLayoutObject().as_ref() }?;
        crate::internal::boundary::layout_boundary::DocumentElementBackgroundForObject(object, self)
    }

    /// Enter the source fragment graph through the opaque owner supplied by
    /// the standalone `layout_fragment` package. Concrete type recovery stays
    /// inside the LayoutNG assembly adapter.
    pub fn TryFromHandle(handle: &'a LayoutResultHandle) -> Option<Self> {
        let source = FragmentCursor::Physical {
            fragment: RootForHandle(handle),
            offset: PhysicalOffset::default(),
        };
        let state = handle.PaintState::<LayoutPaintState>()?;
        // The root's scope is 0 (PaintLayerPainter scopes the LayoutView
        // layer's own fragment index over it).
        Some(Self {
            source,
            state: Some(state),
            display_item_fragment: Some(0),
        })
    }

    pub fn FromHandle(handle: &'a LayoutResultHandle) -> Self {
        Self::TryFromHandle(handle)
            .expect("layout result handle has no committed root paint input")
    }

    pub fn SvgContextReferenceBox(self) -> Option<foundation::gfx::RectF> {
        let object = unsafe { self.source.GetLayoutObject().as_ref() }?;
        let query = unsafe { object.FactorySet().as_ref() }?.svg_context_reference_box?;
        Some(query(object))
    }
    pub fn IsSvgUse(self) -> bool {
        unsafe { self.source.GetLayoutObject().as_ref() }
            .and_then(|object| unsafe {
                foundation::DynamicTo::<Element>(object.GetNode()).as_ref()
            })
            .and_then(|element| element.InputElementData().as_ref())
            .is_some_and(|data| data.svg_use.is_some())
    }
    /// svg_shape_painter.cc:256-280. Return only native resource identities
    /// and derived instance transforms; no resource or geometry tree is copied.
    pub fn SvgMarkerPaints(self) -> Option<Vec<crate::internal::paint_input::SvgMarkerPaint>> {
        let object = unsafe { self.source.GetLayoutObject().as_ref() }?;
        if !object.IsSVGShape() {
            return Some(Vec::new());
        }
        let query = unsafe { object.FactorySet().as_ref() }?.svg_marker_paints?;
        Some(query(object))
    }

    pub fn Cursor(self) -> FragmentCursor<'a> {
        self.source
    }

    pub fn LayerForPaintObject(
        self,
        identity: PaintLayoutObjectIdentity<'a>,
    ) -> Option<&'a crate::internal::paint_layer::PaintLayer> {
        // The token borrows the canonical source object. Check its existing
        // FragmentData Paint sidecar before projection so identities from another
        // Page/result cannot cross this domain. No document walk is involved.
        self.ForLayoutObject(identity.object)?;
        unsafe { identity.object.EnclosingLayer().as_ref() }
    }

    pub fn ForPaintObject(self, identity: PaintLayoutObjectIdentity<'a>) -> Option<Self> {
        self.ForLayoutObject(identity.object)
    }

    pub fn FragmentsForLayoutObject(
        self,
        object: &'a LayoutObject,
    ) -> PaintObjectFragmentCursors<'a> {
        // The FragmentData list is iterated directly; establish its entries
        // first (GetOrCreateFragmentData, pre_paint_tree_walk.cc).
        if let Some(state) = self.state {
            crate::internal::boundary::layout_boundary::SynchronizeBoxFragmentData(object);
            state.SynchronizeInlineObjectFragmentData(object);
        }
        let layout_box = foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object);
        let empty = object.IsFragmentLessBox()
            || unsafe { layout_box.as_ref() }
                .is_some_and(|layout_box| layout_box.PhysicalFragmentCount() == 0);
        PaintObjectFragmentCursors {
            root: self,
            object,
            fragments: (!empty)
                .then(|| crate::fragment_data_iterator::AccompaniedFragmentIterator::new(object)),
            previous_data: None,
        }
    }

    /// Selection invalidation follows native same-object and parent indices,
    /// including inline box/line sidecars, without a fragment-tree census.
    pub(crate) fn VisitSelectionInputsForObject(
        self,
        object: &'a LayoutObject,
        visitor: &mut impl FnMut(Self),
    ) {
        if let Some(box_) = unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object).as_ref()
        } {
            for fragment in box_.PhysicalFragments() {
                let cursor = self.ForPhysicalFragment(fragment);
                if cursor.PaintDataOption().is_some() {
                    visitor(cursor);
                }
            }
            return;
        }
        let mut inline = crate::inline_cursor::InlineCursor::default();
        inline.MoveToLayoutObject(object);
        while inline.IsNotNull() {
            let mut parent = inline.clone();
            while parent.IsNotNull() {
                if let Some(cursor) = self.ForNativeInlineCursor(&parent) {
                    visitor(cursor);
                }
                parent.MoveToParent();
            }
            inline.MoveToNextForSameLayoutObject();
        }
    }

    fn ForNativeInlineCursor(self, inline: &crate::inline_cursor::InlineCursor) -> Option<Self> {
        if inline.IsNull() {
            return None;
        }
        let container = unsafe { &*(inline.ContainerFragment() as *const PhysicalBoxFragment) };
        let items = inline.Items();
        let index = unsafe { inline.CurrentItem().offset_from(items.Items().as_ptr()) } as usize;
        let cursor = self.ForInlineItem(container, index);
        cursor.PaintDataOption().map(|_| cursor)
    }

    /// Resolve the first resident fragment for an existing LayoutObject using
    /// LayoutBox's retained result list or InlineCursor's same-object index.
    /// No document walk or projected fragment directory is constructed. The
    /// result-local Paint owner check also rejects objects from another lease.
    pub fn ForLayoutObject(self, object: &'a LayoutObject) -> Option<Self> {
        let layout_box = foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object);
        if let Some(layout_box) = unsafe { layout_box.as_ref() } {
            if layout_box.PhysicalFragmentCount() != 0 {
                let fragment = unsafe { &*layout_box.GetPhysicalFragment(0) };
                let cursor = self.ForPhysicalFragment(fragment);
                if cursor.PaintDataOption().is_some() {
                    return Some(cursor);
                }
            }
        }
        let mut inline = crate::inline_cursor::InlineCursor::default();
        inline.MoveToLayoutObject(object);
        self.ForNativeInlineCursor(&inline)
    }

    /// Rebind a call-scoped cursor to an existing physical owner in this
    /// result. The owner's FragmentData supplies the same canonical Paint
    /// sidecar; this never searches or copies the fragment graph.
    pub fn ForPhysicalFragment(self, fragment: &'a PhysicalFragment) -> Self {
        Self {
            source: FragmentCursor::Physical {
                fragment,
                offset: PhysicalOffset::default(),
            },
            state: self.state,
            display_item_fragment: None,
        }
    }

    /// Rebind to the canonical item occurrence in its resident container.
    pub fn ForInlineItem(self, container: &'a PhysicalBoxFragment, index: usize) -> Self {
        let items = unsafe { container.Items().as_ref() }
            .expect("inline cursor requires resident FragmentItems");
        let item = &items.Items()[index];
        Self {
            source: FragmentCursor::Inline {
                item,
                items,
                container,
                index,
                offset: *item.OffsetInContainerFragment(),
            },
            state: self.state,
            display_item_fragment: None,
        }
    }

    pub fn Children(self) -> impl Iterator<Item = Self> + 'a {
        layout_fragment_fragment_cursor_children::paint_fragment_children(self)
    }

    // Borrowed-view boundary (stage 1 of replacing the per-fragment paint
    // sidecar): geometry read straight from the resident PhysicalFragment or
    // FragmentItem and its link, in the sidecar's CSS-pixel units, the way
    // Chromium's painters read PhysicalBoxFragment::Size() and the
    // PhysicalFragmentLink offset.

    /// The fragment's size (PhysicalFragment::Size / FragmentItem::Size).
    pub fn ViewSize(self) -> foundation::LayoutSize {
        let size = self.source.Size();
        foundation::LayoutSize {
            width: size.width.ToDouble(),
            height: size.height.ToDouble(),
        }
    }

    /// The fragment's offset in its paint-tree parent: a physical
    /// fragment's child-link offset, or an inline item's offset from its
    /// parent item (from the container fragment for a top-level item), as
    /// FragmentChildren records it.
    pub fn ViewOffsetInContainer(self) -> foundation::LayoutOffset {
        let offset = self.source.Offset();
        foundation::LayoutOffset {
            x: offset.left.ToDouble(),
            y: offset.top.ToDouble(),
        }
    }

    /// The LayoutObject behind the fragment, if any.
    pub fn ViewLayoutObject(self) -> Option<&'a LayoutObject> {
        unsafe { self.source.GetLayoutObject().as_ref() }
    }

    /// Chromium identity of this occurrence, computed from the resident
    /// owner without a publication census: FragmentData::UniqueId for a box
    /// fragment (ensured on demand), (DisplayItemClient, FragmentItem::
    /// FragmentId) for an inline item, (owner client, break-token sequence)
    /// for a fragmentainer, the immutable fragment itself for a line box or
    /// the other cases without a Chromium pair (see
    /// `fragment_tree::PhysicalFragmentInstanceId` / `InlineItemInstanceId`).
    /// FragmentData-less occurrences key their fallback sidecar by it.
    pub fn FragmentInstanceId(self) -> u64 {
        self.source.PaintOwner().InstanceId()
    }

    /// The identity of the fragmentainer this cursor itself is (a column or
    /// page area), else None. Descendants inherit the nearest one through
    /// the walk that reaches them, as PrePaintTreeWalk carries
    /// `fragmentainer_idx` (pre_paint_tree_walk.cc).
    pub fn FragmentainerInstanceId(self) -> Option<u64> {
        match self.source {
            FragmentCursor::Physical { fragment, .. } if fragment.IsFragmentainerBox() => {
                Some(layout_fragment::fragment_tree::FragmentainerInstanceId(
                    unsafe { &*(fragment as *const PhysicalFragment as *const PhysicalBoxFragment) },
                ))
            }
            _ => None,
        }
    }

    pub fn PaintData(self) -> &'a FragmentPaintData {
        self.PaintDataOption()
            .expect("source fragment has no committed paint input")
    }

    /// The fragment's resident Paint sidecar, bound and brought up to date
    /// for the current publication on first access; it lives on the owner's
    /// FragmentData (LayoutPaintState::ResolveSidecar).
    pub unsafe fn PaintDataMut(self) -> &'a mut FragmentPaintData {
        let state = self.state.expect("paint cursor has no result state");
        unsafe { &mut *state.Slot(self.source.PaintOwner(), self.display_item_fragment) }
    }

    pub(crate) fn PaintDataOption(self) -> Option<&'a FragmentPaintData> {
        let state = self.state?;
        Some(unsafe { &*state.Slot(self.source.PaintOwner(), self.display_item_fragment) })
    }

    /// The display-item fragment scope this cursor's children inherit: an
    /// atomic inline's FragmentItem::FragmentId (box_fragment_painter.cc:
    /// 2025-2026,2071-2072), a box's own scope, else this cursor's inherited
    /// one.
    pub(crate) fn ChildDisplayItemFragmentScope(self) -> Option<u32> {
        match self.source {
            FragmentCursor::Physical { fragment, .. } => {
                PhysicalDisplayItemFragment(fragment, self.display_item_fragment)
            }
            FragmentCursor::Inline { item, .. } => Some(item.FragmentId()),
        }
    }

    /// The paint resources of the document this fragment belongs to. Chromium
    /// reads them from the fragment's Document / LocalFrameView, not from the
    /// fragment. Each result carries its own document's value, so a cursor
    /// into an embedded LocalFrame's result sees that frame's resources.
    pub fn DocumentResources(
        self,
    ) -> Option<&'a Arc<layout_fragment::fragment_tree::PaintResources>> {
        self.state?.DocumentResources()
    }

    /// Build the final immutable glyph payload from the canonical
    /// FragmentItem/ShapeResult only when Paint records this text item. A
    /// cached drawing never calls this method, matching Blink's
    /// TextFragmentPainter ownership and avoiding a document-wide glyph
    /// projection during Layout publication.
    pub fn PaintGlyphRuns(
        self,
    ) -> Option<std::sync::Arc<Vec<layout_fragment::fragment_tree::PaintGlyphRun>>> {
        layout_fragment_text_fragment_queries::paint_glyph_runs(self)
    }

    /// Primary-font text origin used by caret geometry. This reads the source
    /// font metrics directly and does not allocate or enumerate glyphs.
    pub fn TextPaintBaseline(self) -> Option<f64> {
        layout_fragment_text_fragment_queries::text_paint_baseline(self)
    }

    /// Query the canonical FragmentItem/ShapeResult for an insertion
    /// position. Chromium keeps this data beside FragmentItem and performs
    /// the query only for editing or hit testing; it is not projected into
    /// every paint fragment during layout publication.
    pub fn TextInlinePosition(self, offset: u32) -> Option<f64> {
        layout_fragment_text_fragment_queries::text_inline_position(self, offset)
    }

    /// Measure the next grapheme with the source effective style. This is the
    /// width source used by Chromium's block and underscore caret geometry.
    pub fn TextNextCharacterWidth(self, offset: u32) -> Option<f64> {
        layout_fragment_text_fragment_queries::text_next_character_width(self, offset)
    }

    /// Return the physical local rectangle of a logical text range directly
    /// from FragmentItem. The result is valid only while this cursor borrows
    /// its LayoutResult owner.
    pub fn TextRangeRect(self, start: u32, end: u32) -> Option<LayoutRect> {
        layout_fragment_text_fragment_queries::text_range_rect(self, start, end)
    }

    /// Return Chromium's complete local selection rect, including containing
    /// line block expansion and an explicitly selected soft line break.
    pub fn TextSelectionRangeRect(
        self,
        start: u32,
        end: u32,
        select_soft_line_break: bool,
    ) -> Option<LayoutRect> {
        layout_fragment_text_fragment_queries::text_selection_range_rect(
            self,
            start,
            end,
            select_soft_line_break,
        )
    }

    /// Return HighlightPainter's line-relative local range rectangle. This is
    /// deliberately distinct from the physical selection-background rect:
    /// Paint uses it for selected glyph/decorations clipping after applying
    /// the writing-mode rotation.
    /// TextFragmentPainter's line-top snap (core/paint/
    /// text_fragment_painter.cc:69-103 ComputeBoxRect): the rounded minus
    /// the unrounded physical line top, from the containing line's offset
    /// and the container fragment's FragmentData::PaintOffset(). `None`
    /// unless this cursor is a text item inside a line.
    pub fn TextLineSnapDelta(self) -> Option<f64> {
        match self.source {
            FragmentCursor::Inline {
                items,
                container,
                index,
                ..
            } => crate::internal::boundary::layout_boundary::LiveTextLineSnapDelta(
                container, items, index,
            ),
            FragmentCursor::Physical { .. } => None,
        }
    }

    pub fn TextHighlightRangeRect(self, start: u32, end: u32) -> Option<LayoutRect> {
        layout_fragment_text_fragment_queries::text_highlight_range_rect(self, start, end)
    }

    /// Resolve a local physical point to a UTF-16 text offset through the
    /// source FragmentItem/ShapeResult hit-test path.
    pub fn TextOffsetForPoint(self, point: crate::internal::layout_input::Offset) -> Option<u32> {
        layout_fragment_text_fragment_queries::text_offset_for_point(self, point)
    }
}

pub(crate) fn PaintLeaseForHandle(handle: &LayoutResultHandle) -> PaintFragmentLease {
    let state = handle
        .PaintState::<LayoutPaintState>()
        .expect("layout result handle has no committed paint state");
    PaintFragmentLease {
        owner: handle
            .PaintStateLease()
            .expect("layout result handle has no committed paint state"),
        generation: state.Generation(),
    }
}

impl<'a> std::ops::Deref for PaintFragmentCursor<'a> {
    type Target = FragmentPaintData;

    fn deref(&self) -> &Self::Target {
        self.PaintDataOption()
            .expect("source fragment has no committed paint input")
    }
}

impl<'a> FragmentCursor<'a> {
    fn PaintOwner(self) -> PaintFragmentOwner {
        match self {
            Self::Physical { fragment, .. } => {
                PaintFragmentOwner::Physical(fragment as *const PhysicalFragment)
            }
            Self::Inline { items, index, .. } => {
                PaintFragmentOwner::Inline(items as *const _, index)
            }
        }
    }

    pub fn PhysicalFragment(self) -> Option<&'a PhysicalFragment> {
        match self {
            Self::Physical { fragment, .. } => Some(fragment),
            Self::Inline { item, .. } => unsafe { item.BoxFragment().as_ref() }
                .map(|fragment| fragment as &PhysicalBoxFragment as &PhysicalFragment),
        }
    }

    pub fn InlineItem(self) -> Option<&'a layout_fragment::fragment_item::FragmentItem> {
        match self {
            Self::Physical { .. } => None,
            Self::Inline { item, .. } => Some(item),
        }
    }

    pub fn Offset(self) -> PhysicalOffset {
        match self {
            Self::Physical { offset, .. } | Self::Inline { offset, .. } => offset,
        }
    }

    pub fn Size(self) -> PhysicalSize {
        match self {
            Self::Physical { fragment, .. } => fragment.Size(),
            Self::Inline { item, .. } => item.Size(),
        }
    }

    pub fn GetLayoutObject(self) -> *const LayoutObject {
        match self {
            Self::Physical { fragment, .. } => fragment.GetLayoutObject(),
            Self::Inline { item, .. } => item.GetLayoutObject(),
        }
    }

    /// Direct child links in LayoutNG paint order. Iteration follows the
    /// source PhysicalFragment/FragmentItem storage without materializing a
    /// parallel child vector.
    pub fn Children(self) -> impl Iterator<Item = Self> + 'a {
        layout_fragment_fragment_cursor_children::fragment_children(self)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutScrollRange {
    pub minimum: crate::internal::layout_input::Offset,
    pub maximum: crate::internal::layout_input::Offset,
}
#[derive(Clone, Copy, Debug)]
pub enum LayoutMetric {
    OffsetWidth,
    OffsetHeight,
    OffsetLeft,
    OffsetTop,
    ClientWidth,
    ClientHeight,
    ClientLeft,
    ClientTop,
    ScrollWidth,
    ScrollHeight,
    ContentWidth,
    ContentHeight,
    BorderBoxWidth,
    BorderBoxHeight,
    MinimumScrollLeft,
    MaximumScrollLeft,
    MinimumScrollTop,
    MaximumScrollTop,
}
impl TryFrom<&str> for LayoutMetric {
    type Error = ();
    fn try_from(name: &str) -> Result<Self, ()> {
        use LayoutMetric::*;
        Ok(match name {
            "offsetWidth" => OffsetWidth,
            "offsetHeight" => OffsetHeight,
            "offsetLeft" => OffsetLeft,
            "offsetTop" => OffsetTop,
            "clientWidth" => ClientWidth,
            "clientHeight" => ClientHeight,
            "clientLeft" => ClientLeft,
            "clientTop" => ClientTop,
            "scrollWidth" => ScrollWidth,
            "scrollHeight" => ScrollHeight,
            "contentWidth" => ContentWidth,
            "contentHeight" => ContentHeight,
            "borderBoxWidth" => BorderBoxWidth,
            "borderBoxHeight" => BorderBoxHeight,
            "minimumScrollLeft" => MinimumScrollLeft,
            "maximumScrollLeft" => MaximumScrollLeft,
            "minimumScrollTop" => MinimumScrollTop,
            "maximumScrollTop" => MaximumScrollTop,
            _ => return Err(()),
        })
    }
}
impl<'a> LayoutOutput<'a> {
    /// Source ScrollableAreaTraversal over this already-borrowed resident
    /// Layout tree. Widget point queries must keep using this loan instead of
    /// reaching back through a Page callback for a second engine borrow.
    pub fn WheelScrollableAreaTraversal(
        self,
        target_node_id: u64,
    ) -> crate::native_wheel_scrollable_traversal::NativeWheelScrollableAreaTraversal {
        self.engine.WheelScrollableAreaTraversal(target_node_id)
    }

    /// TextControlElement's hard-wrap value from the resident InlineCursor and
    /// OffsetMapping owner. No geometry or layout tree crosses this boundary.
    pub fn TextControlValueWithHardLineBreaks(
        self,
        inner_editor: u64,
    ) -> Option<std::rc::Rc<[u16]>> {
        self.engine.TextControlValueWithHardLineBreaks(inner_editor)
    }

    pub fn FragmentRoot(self) -> Option<FragmentCursor<'a>> {
        let root = DynamicTo::<LayoutBox>(self.engine.tree_.as_ref()?.Root() as *const _);
        let root = unsafe { root.as_ref() }?;
        let result = root.GetLayoutResult(0);
        let result = unsafe { result.as_ref() }?;
        Some(FragmentCursor::Physical {
            fragment: result.GetPhysicalFragment(),
            offset: PhysicalOffset::default(),
        })
    }

    pub fn Node(self, id: u64) -> Option<LayoutNode<'a>> {
        Some(LayoutNode {
            object: self.engine.tree_.as_ref()?.OutputNode(id)?,
            output: self,
        })
    }
    pub fn Root(self) -> LayoutNode<'a> {
        LayoutNode {
            object: self.engine.tree_.as_ref().unwrap().Root(),
            output: self,
        }
    }
    /// EventHandler::SelectCursor, event_handler.cc:729-755. This is
    /// LocalFrame::ContentLayoutObject, with the original native coordinate
    /// conversion and LayoutView visual-rect owner under the normal pass scope.
    pub fn CursorContentLocalToRootAncestorPoint(self, point: gfx::PointF) -> gfx::PointF {
        use crate::internal::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
        let _scope = self.engine.NativeScrollPassScope();
        let root = self
            .engine
            .tree_
            .as_ref()
            .expect("resident frame tree")
            .Root();
        root.LocalToAncestorPoint(
            &point,
            std::ptr::null(),
            MapCoordinatesFlags::from_mode(MapCoordinatesMode::kTraverseDocumentBoundaries)
                | MapCoordinatesFlags::from_mode(
                    MapCoordinatesMode::kApplyRemoteMainFrameTransform,
                ),
        )
    }

    pub fn CursorMapContentRectToRootAncestorSpace(self, rect: gfx::RectF) -> gfx::RectF {
        let _scope = self.engine.NativeScrollPassScope();
        let root = self
            .engine
            .tree_
            .as_ref()
            .expect("resident frame tree")
            .Root();
        let mut rect = PhysicalRect::EnclosingRect(&rect);
        root.MapToVisualRectInAncestorSpacePhysical(
            std::ptr::null(),
            &mut rect,
            foundation::graphics_types::graphics::visual_rect_flags::VisualRectFlags::default(),
        );
        gfx::RectF::from(rect)
    }

    /// VisualViewport::VisibleRect, visual_viewport.cc:440-453. Rechrom's
    /// independent Page uses the inactive VisualViewport branch: its retained
    /// host size is returned at origin, even for kExcludeScrollbars. Do not
    /// substitute PaintLayerScrollableArea::VisibleContentRect's scroll offset.
    pub fn CursorVisibleContentRectExcludingScrollbars(self) -> Option<gfx::RectF> {
        let root = self.engine.tree_.as_ref()?.Root();
        let viewport = root.InputOwnerForLayout().InputViewport().as_ref()?;
        assert!(
            !viewport.visual_viewport_is_active,
            "active VisualViewport must provide its native visible rect owner"
        );
        Some(gfx::RectF::new(
            gfx::PointF::default(),
            gfx::SizeF::new(viewport.size.width as f32, viewport.size.height as f32),
        ))
    }
    /// HitTestResult's LayoutObject::PositionForPoint query. The caller borrows
    /// the current result for this call; retained hit nodes never retain this
    /// LayoutOutput. Native Node/LayoutObject identity handles pseudo/anonymous
    /// owners without substituting a DOM-id or materializing another tree.
    /// Source layout_object.cc:4504-4512, layout_block.cc:405-441,
    /// layout_inline.cc:699-708, layout_text.cc:661-713.
    #[cfg(feature = "translation_in_progress")]
    pub fn PositionForNativeNode(
        self,
        node: &crate::internal::layout_node_metadata::Node,
        point: &PhysicalOffset,
        options: crate::EditingPositionOptions,
    ) -> Option<crate::EditingPositionWithAffinity> {
        let root = self.engine.tree_.as_ref()?.Root();
        if node.InputLayoutView() as *const LayoutObject != root as *const LayoutObject {
            return None;
        }
        let object = unsafe { node.GetLayoutObject().as_ref() }?;
        if object.WasDestroyedForTreeUpdate() {
            return None;
        }
        LayoutNode {
            output: self,
            object,
        }
        .PositionForPoint(point, options)
    }
    pub fn ScrollOffsets(self) -> Vec<(u64, crate::internal::layout_input::Offset)> {
        self.engine.tree_.as_ref().unwrap().OutputScrollOffsets()
    }
}
impl<'a> LayoutNode<'a> {
    /// Source: context_menu_controller.cc:317-321. Read the original box's
    /// background opacity proof for exactly the hit-test point rectangle.
    pub fn BackgroundIsKnownToBeOpaqueInRect(&self, rect: &PhysicalRect) -> bool {
        let _scope = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(self.object).as_ref()
        }
        .is_some_and(|box_| box_.BackgroundIsKnownToBeOpaqueInRect(rect))
    }
    pub fn PaintObjectIdentity(&self) -> PaintLayoutObjectIdentity<'a> {
        PaintLayoutObjectIdentity {
            object: self.object,
        }
    }
    /// LayoutEmbeddedContent::BorderBoxFromEmbeddedContent + the original
    /// LayoutObject ancestor mapping. Frame assembly only composes the same
    /// native mappings across the actual FrameTree owner links.
    pub fn EmbeddedContentPointToDocument(&self, point: gfx::PointF) -> gfx::PointF {
        let _scope = self.output.engine.NativeScrollPassScope();
        let box_ =
            unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }.expect("embedded LayoutBox");
        let offset = box_.PhysicalContentBoxRect().offset;
        self.object.LocalToAbsolutePointF(
            &gfx::PointF::new(
                point.x() + offset.left.ToFloat(),
                point.y() + offset.top.ToFloat(),
            ),
            Default::default(),
        )
    }
    /// LocalFrameView::ConvertFromRootFrame: inverse of the embedded owner's
    /// native content-box mapping, including transforms and ancestor scroll.
    pub fn DocumentPointToEmbeddedContent(&self, point: gfx::PointF) -> Option<gfx::PointF> {
        let _scope = self.output.engine.NativeScrollPassScope();
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }?;
        let point = self
            .object
            .AbsoluteToLocalPointF(&point, Default::default());
        let origin = box_.PhysicalContentBoxRect().offset;
        Some(gfx::PointF::new(
            point.x() - origin.left.ToFloat(),
            point.y() - origin.top.ToFloat(),
        ))
    }
    pub fn EmbeddedContentRectToDocument(&self, rect: gfx::RectF) -> gfx::RectF {
        let _scope = self.output.engine.NativeScrollPassScope();
        let box_ =
            unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }.expect("embedded LayoutBox");
        let offset = box_.PhysicalContentBoxRect().offset;
        let rect = gfx::RectF::new(
            gfx::PointF::new(
                rect.x() + offset.left.ToFloat(),
                rect.y() + offset.top.ToFloat(),
            ),
            rect.size(),
        );
        self.object
            .LocalToAbsoluteQuad(&gfx::QuadF::from(rect), Default::default())
            .BoundingBox()
    }

    /// Read the same resident document-order index used by fragment selection
    /// paint. Editing never reconstructs a parallel DOM/Layout tree order.
    pub fn LogicalTreeOrder(&self) -> Option<u64> {
        self.output
            .engine
            .boundary_cache_
            .sources
            .get(&(self.object as *const LayoutObject))
            .map(|source| source.logical_tree_order)
    }
    pub fn PaintClientId(&self) -> u64 {
        // LayoutObject's base-first representation retains DisplayItemClient.
        unsafe { &*(self.object as *const _ as *const foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient) }.Id() as u64
    }
    /// Source: LayoutObject::CanBeSelectionLeaf, layout_object.cc:5582-5588.
    pub fn CanBeSelectionLeaf(&self) -> bool {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        self.object.CanBeSelectionLeaf()
    }

    /// Source: LayoutObject::AbsoluteToLocalPoint. Coordinate mapping keeps
    /// the canonical object and its installed virtual providers in this scope.
    pub fn AbsoluteToLocalPoint(&self, point: &PhysicalOffset) -> PhysicalOffset {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        self.object.AbsoluteToLocalPointPhysical(
            point,
            crate::internal::map_coordinates_flags::MapCoordinatesFlags::default(),
        )
    }

    /// html_anchor_element.cc:149-165. Borrow the image's native LayoutBox;
    /// preserve transformed coordinates, content origin and effective zoom.
    pub fn ServerMapContentPoint(&self, absolute: &gfx::PointF) -> Option<(f32, f32, f32)> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object as *const _).as_ref() }?;
        let point = self.object.AbsoluteToLocalPointF(
            absolute,
            crate::internal::map_coordinates_flags::MapCoordinatesFlags::default(),
        );
        let origin = box_.PhysicalContentBoxRect().offset;
        Some((
            point.x() - origin.left.ToFloat(),
            point.y() - origin.top.ToFloat(),
            self.object.StyleRef().EffectiveZoom(),
        ))
    }

    /// Source: LayoutObject::AbsoluteQuads. Enumerate native fragments with
    /// the object's installed virtual owner, never an enclosing client rect.
    pub fn AbsoluteQuads(&self) -> Vec<gfx::QuadF> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let mut quads = Vec::new();
        self.object.AbsoluteQuads(
            &mut quads,
            crate::internal::map_coordinates_flags::MapCoordinatesFlags::default(),
            crate::internal::layout_object::BoxQuadType::kBorder,
        );
        quads
    }

    /// Source: LayoutText::AbsoluteQuadsForRange. The UTF-16 range uses the
    /// canonical OffsetMapping and shaped resident FragmentItems.
    pub fn AbsoluteQuadsForRange(&self, range: std::ops::Range<u32>) -> Vec<gfx::QuadF> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let mut quads = Vec::new();
        let text = unsafe {
            foundation::DynamicTo::<crate::internal::layout_text::LayoutText>(
                self.object as *const _,
            )
            .as_ref()
        };
        if let Some(text) = text {
            text.AbsoluteQuadsForRange(&mut quads, range.start, range.end);
        }
        quads
    }

    /// Element::ClientQuads cc3337-3372 (HTML branch), and
    /// GetBoundingClientRectNoLifecycleUpdate cc3443-3467. This query borrows
    /// the original object's virtual AbsoluteQuads; it does not walk/copy the
    /// fragment tree through the legacy CSSOM ClientRects projection.
    pub fn HTMLBoundingClientRectNoLifecycleUpdate(&self, absolute_zoom: f32) -> gfx::RectF {
        let mut bounds = gfx::RectF::default();
        if self.object.IsBoxModelObject() || self.object.IsBR() {
            for quad in self.AbsoluteQuads() {
                bounds.Union(quad.BoundingBox());
            }
        }
        if bounds != gfx::RectF::default() && absolute_zoom != 1.0 {
            bounds.Scale(1.0 / absolute_zoom, 1.0 / absolute_zoom);
        }
        bounds
    }

    /// Document::AdjustRectForScrollAndAbsoluteZoom, document.cc:8939-8948;
    /// AdjustForAbsoluteZoom::GetAbsoluteZoom, adjust_for_absolute_zoom.h:44-50.
    /// The frame supplies its layout zoom and Document feature policy; the
    /// native LayoutObject remains the owner of effective CSS zoom.
    pub fn HTMLBoundingClientRectForDocument(
        &self,
        frame_layout_zoom: f32,
        standardized_browser_zoom: bool,
    ) -> gfx::RectF {
        let zoom = if standardized_browser_zoom {
            frame_layout_zoom
        } else {
            self.object.StyleRef().EffectiveZoom()
        };
        self.HTMLBoundingClientRectNoLifecycleUpdate(zoom)
    }

    /// Element::getClientRects and
    /// Document::AdjustRectForScrollAndAbsoluteZoom, element.cc:3373-3384.
    /// ClientRects remains the native, unadjusted geometry query for Layout
    /// consumers; this projection is the CSSOM-facing value after Chromium's
    /// absolute-zoom adjustment.
    pub fn HTMLClientRectsForDocument(
        &self,
        frame_layout_zoom: f32,
        standardized_browser_zoom: bool,
    ) -> Vec<LayoutRect> {
        let zoom = if standardized_browser_zoom {
            frame_layout_zoom
        } else {
            self.object.StyleRef().EffectiveZoom()
        };
        let scale = if zoom.is_finite() && zoom > 0.0 {
            1.0 / f64::from(zoom)
        } else {
            1.0
        };
        self.ClientRects()
            .into_iter()
            .map(|rect| LayoutRect {
                x: rect.x * scale,
                y: rect.y * scale,
                width: rect.width * scale,
                height: rect.height * scale,
            })
            .collect()
    }

    /// Element::GetBoundingClientRect cc3443-3473 and
    /// DOMRectReadOnly::IsPointInside h50-52, on the resident client quads.
    pub fn BoundingClientRectContainsPoint(&self, x: f64, y: f64) -> bool {
        let mut bounds = gfx::RectF::default();
        for rect in self.ClientRects() {
            bounds.Union(gfx::RectF::new(
                gfx::PointF::new(rect.x as f32, rect.y as f32),
                gfx::SizeF::new(rect.width as f32, rect.height as f32),
            ));
        }
        x >= bounds.x() as f64
            && x < bounds.right() as f64
            && y >= bounds.y() as f64
            && y < bounds.bottom() as f64
    }
    /// Element::VisibleBoundsRespectingClipsInLocalRoot, element.cc:3313-3336.
    /// Stop at this engine's native LayoutView: the embedding supplies each
    /// actual LocalFrame owner separately, rather than fabricating a native
    /// pointer from a different engine. Scroll, transforms and overflow clips
    /// are evaluated by the existing visual-rect slow path in this scope.
    pub fn VisibleBoundsInFrame(&self) -> Option<gfx::RectF> {
        let _scope = self.output.engine.NativeScrollPassScope();
        let box_ = unsafe { foundation::DynamicTo::<LayoutBox>(self.object).as_ref() }?;
        let rect = gfx::RectF::from(box_.PhysicalBorderBoxRect());
        self.MapLocalVisualRectInFrame(rect)
    }

    /// LocalFrameView::MapToVisualRectInAncestorSpace's embedded-content
    /// boundary. Input is already clipped in the child viewport; offset it to
    /// this owner's content box, then use the same native ancestor clip chain.
    pub fn EmbeddedContentVisibleRectInFrame(&self, mut rect: gfx::RectF) -> Option<gfx::RectF> {
        let _scope = self.output.engine.NativeScrollPassScope();
        let box_ = unsafe { foundation::DynamicTo::<LayoutBox>(self.object).as_ref() }?;
        let content = box_.PhysicalContentBoxRect();
        rect.Intersect(gfx::RectF::new(
            gfx::PointF::default(),
            gfx::SizeF::new(content.size.width.ToFloat(), content.size.height.ToFloat()),
        ));
        rect.Offset(content.offset.left.ToFloat(), content.offset.top.ToFloat());
        self.MapLocalVisualRectInFrame(rect)
    }

    fn MapLocalVisualRectInFrame(&self, mut rect: gfx::RectF) -> Option<gfx::RectF> {
        use foundation::graphics_types::graphics::visual_rect_flags::VisualRectFlags;
        use foundation::TransformState;
        if self.object.HasLockedAncestorPreventingPaint() || rect.IsEmpty() {
            return None;
        }
        let root = self.output.engine.tree_.as_ref()?.Root();
        let root_box = unsafe { foundation::DynamicTo::<LayoutBox>(root).as_ref() }?;
        if !self.object.MapToVisualRectInAncestorSpaceFloat(
            root as *const _ as *const LayoutBoxModelObject,
            &mut rect,
            VisualRectFlags::default(),
        ) {
            return None;
        }
        // Mapping to an explicit ancestor intentionally leaves that ancestor's
        // content scroll/clip unapplied. Apply LayoutView's own native owner
        // once, including the fixed-position exemption in ContainedContentsScroll.
        let mut state = TransformState::new(
            TransformState::kApplyTransformDirection,
            gfx::QuadF::from(rect),
        );
        if !root_box.MapContentsRectToBoxSpace(
            &mut state,
            TransformState::kFlattenTransform,
            self.object,
            VisualRectFlags::default(),
        ) {
            return None;
        }
        state.Flatten();
        let mut rect = state.LastPlanarQuad().BoundingBox();
        // LocalFrameView clipping also applies when CSS overflow propagation
        // leaves LayoutView without a HasClipRelatedProperty bit.
        rect.Intersect(gfx::RectF::from(root_box.OverflowClipRect()));
        (!rect.IsEmpty()).then_some(rect)
    }

    pub fn ClientRects(&self) -> Vec<LayoutRect> {
        layout_fragment_fragment_client_rect_queries::client_rects(self)
    }
    pub fn ScrollRange(&self) -> LayoutScrollRange {
        layout_fragment_layout_metrics::scroll_range(self)
    }
    pub fn NativeOffsetWidth(&self) -> Option<foundation::LayoutUnit> {
        layout_fragment_layout_metrics::native_offset_width(self)
    }
    pub fn Metric(&self, metric: LayoutMetric) -> Option<f64> {
        layout_fragment_layout_metrics::metric(self, metric)
    }
}

/// Minimal source Layout facts consumed by sequential focus navigation.
#[derive(Clone, Copy, Debug)]
pub struct LayoutFocusNavigationFacts {
    pub is_box: bool,
    pub is_canvas: bool,
    pub is_embedded_content: bool,
    pub is_snap_container: bool,
    pub is_table: bool,
    pub is_table_cell: bool,
    pub is_user_scrollable: bool,
    pub is_reading_flow_container: bool,
}
impl LayoutNode<'_> {
    pub fn FocusNavigationFacts(&self) -> LayoutFocusNavigationFacts {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object as *const _).as_ref() };
        LayoutFocusNavigationFacts {
            is_box: box_.is_some(),
            is_canvas: self.object.IsCanvas(),
            is_embedded_content: self.object.IsLayoutEmbeddedContent(),
            is_table: self.object.IsTable(),
            is_table_cell: self.object.IsTableCell(),
            is_snap_container: box_
                .is_some_and(|box_| !box_.StyleRef().GetScrollSnapType().is_none),
            is_user_scrollable: box_.is_some_and(|box_| box_.IsUserScrollable()),
            is_reading_flow_container: box_.is_some_and(|box_| box_.IsReadingFlowContainer()),
        }
    }
    pub fn ReadingFlowNodes(&self) -> Vec<u64> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let Some(box_) = (unsafe { DynamicTo::<LayoutBox>(self.object as *const _).as_ref() })
        else {
            return Vec::new();
        };
        box_.ReadingFlowNodes()
            .iter()
            .filter_map(|node| unsafe { node.Get().as_ref() })
            .map(|node| node.InputId())
            .collect()
    }
}

#[path = "../../layout_fragment/src/grid_focusgroup_cursor.rs"]
mod layout_fragment_grid_focusgroup_cursor;
pub use layout_fragment_grid_focusgroup_cursor::TableFocusgroupCursor;

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/editing_bidi_range.rs"]
mod editing_bidi_range;
#[cfg(feature = "translation_in_progress")]
pub use editing_bidi_range::EditingBidiRangeAdjustment;

/// Scalar source virtual facts, borrowed from this node's current native owner.
pub struct LayoutHitTestFacts {
    pub image: bool,
    pub svg_image: bool,
    pub box_: bool,
    pub media: bool,
    pub locked_ancestor_preventing_paint: bool,
    pub child_paint_blocked: bool,
    pub direction: foundation::TextDirection,
}
impl LayoutNode<'_> {
    /// Source touch_action_util.cc:14-59 and touch_adjustment.cc:132-136.
    /// These values are read from the same resident ComputedStyle owner.
    pub fn PointerTouchActionFacts(&self) -> (foundation::TouchAction, bool, bool) {
        (
            self.object.StyleRef().EffectiveTouchAction(),
            self.object.StyleRef().AffectedByActive(),
            self.object.StyleRef().AffectedByHover(),
        )
    }
    pub fn PointerHasHorizontallyScrollableAncestor(&self) -> bool {
        let _scope = self.output.engine.NativeScrollPassScope();
        crate::internal::layout_box::LayoutBox::HasHorizontallyScrollableAncestor(
            self.object as *const _ as *mut _,
        )
    }
    pub fn HitTestFacts(&self) -> LayoutHitTestFacts {
        LayoutHitTestFacts {
            image: self.object.IsImage(),
            svg_image: self.object.IsSVGImage(),
            box_: self.object.IsBox(),
            media: self.object.IsMedia(),
            locked_ancestor_preventing_paint: self.object.HasLockedAncestorPreventingPaint(),
            child_paint_blocked: self.object.ChildPaintBlockedByDisplayLock(),
            direction: self.object.StyleRef().Direction(),
        }
    }
    pub fn AbsoluteContentQuadForHitTest(&self) -> gfx::QuadF {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(self.object).as_ref()
        }
        .expect("HitTestResult::ImageRect uses the owning LayoutBox");
        box_.AbsoluteContentQuad(Default::default())
    }
    /// Source LayoutObject::UpdateHitTestResult cc:4440-4448;
    /// LayoutInline cc:834-844 and LayoutView cc:916-932.
    pub fn UpdateHitTestNodeAndPoint(
        &self,
        point: PhysicalOffset,
        document_element: Option<u64>,
    ) -> Option<(u64, PhysicalOffset)> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        if self.object.IsLayoutView() {
            let node = document_element?;
            let mut point = point;
            if let Some(object) = self
                .output
                .engine
                .GetLayoutTree()
                .and_then(|tree| tree.GetLayoutObjectForNode(node))
            {
                if let Some(box_) = unsafe {
                    foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(object).as_ref()
                } {
                    point -= box_.PhysicalLocation();
                }
            }
            if self.object.IsScrollContainer() {
                let box_ = unsafe {
                    &*(self.object as *const LayoutObject)
                        .cast::<crate::internal::layout_box::LayoutBox>()
                };
                let scroll = box_.PixelSnappedScrolledContentOffset();
                point += PhysicalOffset::new(
                    foundation::LayoutUnit::from_signed(scroll.x()),
                    foundation::LayoutUnit::from_signed(scroll.y()),
                );
            }
            return Some((node, point));
        }
        let pointer = if self.object.IsLayoutInline() {
            self.object.GetNode()
        } else {
            self.object.NodeForHitTest()
        };
        let node = unsafe { pointer.as_ref()? };
        Some((
            node.InputHitTestNodeOwner()
                .unwrap_or_else(|| node.InputId()),
            point,
        ))
    }
    pub fn IsSelected(&self) -> bool {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        self.object.IsSelected()
    }
}

impl LayoutNode<'_> {
    pub fn LayoutSelectionRange(&self) -> std::ops::Range<u32> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let text = unsafe {
            foundation::DynamicTo::<crate::internal::layout_text::LayoutText>(self.object).as_ref()
        }
        .expect("touch selected range belongs to LayoutText");
        let editing = self
            .output
            .engine
            .GetLayoutTree()
            .expect("live layout tree")
            .EditingState();
        let range = editing
            .selections
            .layout_selection
            .borrow()
            .TextSelectionRange(text);
        range
    }
    pub fn ImageMapContainer(&self) -> foundation::native_path::ImageMapContainer {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(self.object).as_ref()
        };
        foundation::native_path::ImageMapContainer {
            effective_zoom: self.object.StyleRef().EffectiveZoom(),
            border_box: box_.map(|b| {
                let rect = b.PhysicalBorderBoxRect();
                gfx::RectF::new(
                    gfx::PointF::new(rect.offset.left.ToFloat(), rect.offset.top.ToFloat()),
                    gfx::SizeF::new(rect.size.width.ToFloat(), rect.size.height.ToFloat()),
                )
            }),
        }
    }
}

#[path = "input_control_geometry.rs"]
mod input_control_geometry;

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/editing_find_buffer.rs"]
mod layout_algo_inline_editing_find_buffer;
#[cfg(feature = "translation_in_progress")]
pub use layout_algo_inline_editing_find_buffer::{
    EditingFindBlockIdentity, EditingFindMapping, EditingFindStyleFacts,
};

#[path = "resize_observer_geometry.rs"]
mod resize_observer_geometry;

#[path = "native_range_geometry.rs"]
mod native_range_geometry;

#[cfg(feature = "translation_in_progress")]
#[path = "../../layout_algo_inline/src/editing_apply_style_facts.rs"]
mod editing_apply_style_facts;

impl LayoutOutput<'_> {
    pub fn BlockZoomBoundsForNode(self, node: u64) -> Option<(bool, gfx::Rect)> {
        crate::native_page_scale_geometry::BlockZoomBoundsForNode(self.engine, node)
    }
    pub fn PageScaleGeometry(
        self,
    ) -> Option<crate::native_page_scale_geometry::NativePageScaleGeometry> {
        self.engine.PageScaleGeometry()
    }
}
