//! Page cursor projections over canonical DOM/Layout/Paint owners.
//!
//! Resident queries execute translated PaintLayer and EventHandler policy.
//! Production CSS/auto selection consumes the actual resident hit cursor.
//! The full source entry still requires frame, scroll, capture and geometry
//! ports; overflow controls and locked-selection preparation stay explicit.
//! Native window cursor creation stays with the host.
use dom::{persistent_document::DOMNodeType, Document};
use layoutng_assembly::internal::paint_input::ECursor as StyleCursor;

pub(crate) use interaction_hit_test::resident::point_queries::ResidentHitTarget as ResidentCursorHit;

/// A resident query requires the translated source owners. A missing owner is
/// reported explicitly and never falls through to compatibility hit testing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidentCursorError {
    PrePaintNotReady,
    MissingSourceLayer,
    MissingSourceCursor,
    LayerIdentityMismatch,
    HitTest(interaction_hit_test::resident::query_services::ResidentQueryError),
    MissingScrollOwner,
    MissingFrameGeometry,
    MissingHostCursorMetrics,
    Unsupported(ResidentCursorCapability),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidentCursorCapability {
    CursorImage,
    ScrollbarCursor,
    DetachedNode,
    EditingPositionOwner,
}

pub type ResidentCursorHitResult =
    interaction_hit_test::core::layout::hit_test_result::HitTestResult<'static, ()>;

/// Call-scoped source-owner binding. Implementations project canonical DOM
/// nodes and existing Paint/manager owners; they must not select targets,
/// choose cursor policy, or materialize another layer/fragment graph.
///
/// The source HitTestNode resolver and geometry services remain explicit
/// integration requirements. The historical node-id cursor selector cannot
/// satisfy this contract because it drops image-map, pseudo-node, scrollbar,
/// local-position, capture, selection and custom-image state.
pub trait ResidentCursorResolver {
    fn CheckQuery(&self) -> Result<(), ResidentCursorError> {
        Ok(())
    }

    fn BindRootLayer<'a>(
        &'a self,
        document: &'a crate::ownership::InteractionDocument,
        paint_root: paint::resident_hit_test::ResidentPaintHitTest<'a>,
        fragments: layoutng_assembly::layout_engine::PaintFragmentCursor<'a>,
    ) -> Result<
        Box<
            dyn interaction_hit_test::core::paint::paint_layer_hit_test::PaintLayerHitTest<
                    ResidentCursorHitResult,
                > + 'a,
        >,
        ResidentCursorError,
    >;
    fn CursorHandler(
        &self,
    ) -> Result<
        crate::core::input::event_handler_cursor::EventHandlerCursorPolicy<'_>,
        ResidentCursorError,
    >;
    fn CursorResult<'a>(
        &'a self,
        document: &'a crate::ownership::InteractionDocument,
        result: &'a ResidentCursorHitResult,
    ) -> Result<
        Box<dyn crate::core::input::event_handler_cursor::CursorHitTestResult + 'a>,
        ResidentCursorError,
    >;
}

struct ResidentCursorLocation(foundation::PhysicalOffset);
impl crate::core::input::event_handler_cursor::CursorHitTestLocation for ResidentCursorLocation {
    fn Point(&self) -> crate::core::input::event_handler_cursor::PointF {
        crate::core::input::event_handler_cursor::PointF {
            x: self.0.left.ToFloat(),
            y: self.0.top.ToFloat(),
        }
    }
}

/// Execute the source PaintLayer hit test followed by EventHandler's source
/// cursor policy. `None` retains the host's current cursor, matching Blink's
/// resize/autoscroll/LayoutObject "do not set cursor" branches.
///
/// This entry deliberately requires source-owner bindings. Until they are
/// assembled, callers must treat that as an incomplete integration, not a
/// successful query against an approximate fragment walk.
pub fn HitTestAtResident<'a>(
    document: &'a crate::ownership::InteractionDocument,
    point: layoutng_assembly::internal::layout_input::Offset,
    paint_engine: &'a paint::PaintEngine,
    fragments: layoutng_assembly::layout_engine::PaintFragmentCursor<'a>,
    resolver: &'a dyn ResidentCursorResolver,
) -> Result<ResidentCursorHitResult, ResidentCursorError> {
    HitTestAtResidentWithFlags(document, point, paint_engine, fragments, resolver, 0)
}
fn HitTestAtResidentWithFlags<'a>(
    document: &'a crate::ownership::InteractionDocument,
    point: layoutng_assembly::internal::layout_input::Offset,
    paint_engine: &'a paint::PaintEngine,
    fragments: layoutng_assembly::layout_engine::PaintFragmentCursor<'a>,
    resolver: &'a dyn ResidentCursorResolver,
    extra_flags: u32,
) -> Result<ResidentCursorHitResult, ResidentCursorError> {
    use foundation::{LayoutUnit, PhysicalOffset, PhysicalRect};
    use interaction_hit_test::core::{
        layout::{
            hit_test_location::HitTestLocation,
            hit_test_request::RequestType,
            hit_test_result::{HitTestResult, HitTestResultRequest},
        },
        paint::paint_layer_hit_test,
    };
    // Borrow and prepare the canonical source stacking owner once. All layer
    // descendants must come from this same retained Paint domain.
    let order = paint_engine
        .ResidentLayerOrder(
            fragments,
            paint::resident_hit_test::ResidentLayerSelection {
                negative_z: true,
                normal_flow: true,
                positive_z: true,
            },
            true,
        )
        .ok_or(ResidentCursorError::PrePaintNotReady)?;
    drop(order);
    let paint_root = paint_engine
        .ResidentHitTest(fragments)
        .ok_or(ResidentCursorError::PrePaintNotReady)?;
    let identity = paint_root.Identity();
    let layer = resolver.BindRootLayer(document, paint_root, fragments)?;
    if layer.Identity() != identity {
        return Err(ResidentCursorError::LayerIdentityMismatch);
    }
    let zoom = fragments
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.layout_zoom_factor)
        .max(f64::MIN_POSITIVE);
    let point = PhysicalOffset::new(
        LayoutUnit::FromDoubleRound(point.x * zoom),
        LayoutUnit::FromDoubleRound(point.y * zoom),
    );
    let location = HitTestLocation::FromPhysicalOffset(point);
    // Source: event_handler.cc:525-535 (read-only cursor query),
    // paint_layer.cc:1039-1094, event_handler.cc:573-854 (selection).
    let request = HitTestResultRequest::<()>::new(
        RequestType::kReadOnly as u32 | RequestType::kAllowChildFrameContent as u32 | extra_flags,
        None,
        None,
    );
    let mut result = HitTestResult::new(&request, &location);
    let area = PhysicalRect::new(PhysicalOffset::default(), fragments.Cursor().Size());
    paint_layer_hit_test::HitTest(layer.as_ref(), &location, &mut result, &area);
    resolver.CheckQuery()?;
    Ok(result)
}

pub fn CursorAtResident<'a>(
    document: &'a crate::ownership::InteractionDocument,
    point: layoutng_assembly::internal::layout_input::Offset,
    paint_engine: &'a paint::PaintEngine,
    fragments: layoutng_assembly::layout_engine::PaintFragmentCursor<'a>,
    resolver: &'a dyn ResidentCursorResolver,
) -> Result<Option<crate::cursor::Cursor>, ResidentCursorError> {
    let result = HitTestAtResident(document, point, paint_engine, fragments, resolver)?;
    let zoom = fragments
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.layout_zoom_factor)
        .max(f64::MIN_POSITIVE);
    let location = ResidentCursorLocation(foundation::PhysicalOffset::new(
        foundation::LayoutUnit::FromDoubleRound(point.x * zoom),
        foundation::LayoutUnit::FromDoubleRound(point.y * zoom),
    ));
    let cursor_result = resolver.CursorResult(document, &result)?;
    let selected = resolver
        .CursorHandler()?
        .SelectCursor(&location, cursor_result.as_ref());
    resolver.CheckQuery()?;
    Ok(selected)
}

/// Canonical typed DOM result owners. The binding retains the same node
/// identities used by source HitTestResult; it does not build a node tree.
pub trait ResidentDOMNodeOwners {
    fn NodeForId(
        &self,
        id: u64,
    ) -> Option<interaction_hit_test::core::layout::hit_test_result::NodeRef>;
}

/// The Document side of the resident hit-test bridge. Node admission and the
/// document-element fact are read from the live InteractionDocument for each
/// query; typed result owners come from the existing DOM binding.
pub struct DOMHitTestNodeResolver<'a> {
    document: &'a crate::ownership::InteractionDocument,
    owners: &'a dyn ResidentDOMNodeOwners,
}
impl<'a> DOMHitTestNodeResolver<'a> {
    pub fn new(
        document: &'a crate::ownership::InteractionDocument,
        owners: &'a dyn ResidentDOMNodeOwners,
    ) -> Self {
        Self { document, owners }
    }
    pub fn ResolveNode(
        &self,
        id: u64,
    ) -> Option<interaction_hit_test::core::layout::hit_test_result::NodeRef> {
        self.document.borrow().GetDocument().FindNodeById(id)?;
        self.owners.NodeForId(id)
    }
    pub fn DocumentElement(
        &self,
    ) -> Option<interaction_hit_test::core::layout::hit_test_result::NodeRef> {
        let id = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            document
                .Node(document.Root())
                .Children()
                .iter()
                .copied()
                .find(|&child| document.Node(child).Type() == DOMNodeType::kElement)
                .map(|child| document.Node(child).Id())?
        };
        self.owners.NodeForId(id)
    }
}

// Package-owned DOM/result projections. Persistent<Node> roots the exact native
// identity and its traced LayoutObject; only the query's LayoutOutput is borrowed.
use crate::core::input::event_handler_cursor as source_cursor;
use foundation::{gfx, Persistent};
use interaction_hit_test::core::{
    dom::node_hit_test::{self, SelectionStartNode},
    layout::hit_test_result as hit,
};
pub use interaction_hit_test::resident::child_frame_query::{
    PreparedLocalChildFrameQueries, UnavailableLocalChildFrameQueries,
};
use layoutng_assembly::{
    internal::{
        layout_box::LayoutBox, layout_box_model_object::LayoutBoxModelObject,
        layout_node_metadata::Node as NativeNode, layout_object::LayoutObject,
    },
    layout_engine::{LayoutOutput, PaintFragmentCursor},
};
use layoutng_style::style::computed_style::ComputedStyle;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct ResidentCursorImageFailure {
    pub resource_id: u64,
    pub kind: std::io::ErrorKind,
    pub message: String,
}
#[derive(Default)]
pub(super) struct CursorDiagnostics {
    capability: Cell<Option<ResidentCursorCapability>>,
    image_failures: RefCell<Vec<ResidentCursorImageFailure>>,
}
impl CursorDiagnostics {
    fn get(&self) -> Option<ResidentCursorCapability> {
        self.capability.get()
    }
    fn set(&self, value: Option<ResidentCursorCapability>) {
        self.capability.set(value)
    }
    pub(super) fn ImageFailure(&self, failure: ResidentCursorImageFailure) {
        // A query-local list, drained when the caller receives its cursor.
        self.image_failures.borrow_mut().push(failure);
    }
}
pub(super) type CursorGap = Rc<CursorDiagnostics>;
#[derive(Clone)]
struct CursorNodeValue {
    document: crate::ownership::InteractionDocument,
    id: u64,
    identity: u128,
    native: Option<Persistent<NativeNode>>,
    gap: CursorGap,
    editing_positions:
        Option<Rc<dyn super::resident_editing_positions::ResidentEditingPositionQueries>>,
    native_paths: Option<std::sync::Arc<dyn foundation::native_path::NativePathFactory>>,
}
impl CursorNodeValue {
    fn ForNative(
        document: &crate::ownership::InteractionDocument,
        native: &NativeNode,
        gap: CursorGap,
        editing_positions: Option<
            Rc<dyn super::resident_editing_positions::ResidentEditingPositionQueries>,
        >,
        native_paths: Option<std::sync::Arc<dyn foundation::native_path::NativePathFactory>>,
    ) -> Option<Self> {
        let (id, identity) = {
            let owner = document.borrow();
            let doc = owner.GetDocument();
            let id = if doc.FindNodeById(native.InputId()).is_some() {
                native.InputId()
            } else {
                native.InputHitTestNodeOwner()?
            };
            (id, doc.NodeHandle(doc.FindNodeById(id)?).Identity())
        };
        Some(Self {
            document: document.clone(),
            id,
            identity,
            native: Some(unsafe { Persistent::from_ptr(native as *const _ as *mut NativeNode) }),
            gap,
            editing_positions,
            native_paths,
        })
    }
    fn Gap(&self, capability: ResidentCursorCapability) {
        if self.gap.get().is_none() {
            self.gap.set(Some(capability));
        }
    }
    fn Read<T>(&self, read: impl FnOnce(&Document, usize) -> T) -> Option<T> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let index = document.FindNodeById(self.id)?;
        (document.NodeHandle(index).Identity() == self.identity).then(|| read(document, index))
    }
    fn Object(&self) -> Option<&LayoutObject> {
        let node = self
            .native
            .as_ref()
            .and_then(|node| unsafe { node.Get().as_ref() })?;
        unsafe { node.GetLayoutObject().as_ref() }
    }
    fn WithStyle<T>(&self, read: impl FnOnce(&ComputedStyle) -> T) -> Option<T> {
        if let Some(object) = self.Object() {
            return Some(read(object.StyleRef()));
        }
        self.Read(|document, index| {
            document
                .ResolvedStyleFor(index)
                .and_then(|style| style.ComputedStyle())
                .map(read)
        })
        .flatten()
    }
    fn Attribute(&self, name: &str) -> Option<String> {
        self.Read(|document, index| {
            document
                .Node(index)
                .FindAttribute(name)
                .map(|a| a.value.clone())
        })
        .flatten()
    }
    fn IsHTML(&self, name: &str) -> bool {
        self.Read(|document, index| document.Node(index).IsHTMLElement(name))
            .unwrap_or(false)
    }
    fn ForDOMId(&self, id: u64) -> Option<Self> {
        let identity = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            document.NodeHandle(document.FindNodeById(id)?).Identity()
        };
        let mut native = self
            .native
            .as_ref()
            .and_then(|node| unsafe { node.Get().as_ref() });
        let native = loop {
            let Some(node) = native else { break None };
            if node.InputId() == id {
                break Some(unsafe { Persistent::from_ptr(node as *const _ as *mut NativeNode) });
            }
            native = unsafe { (node.parentNode() as *mut NativeNode).as_ref() };
        };
        let native = native.or_else(|| self.editing_positions.as_ref()?.NodeOwner(id));
        Some(Self {
            id,
            identity,
            native,
            ..self.clone()
        })
    }
    fn Parent(&self, element: bool) -> Option<Self> {
        let id = self
            .Read(|document, index| {
                let parent = if element {
                    document.FlatTreeParentElement(index)
                } else {
                    document.FlatTreeParent(index)
                }?;
                Some(document.Node(parent).Id())
            })
            .flatten()?;
        self.ForDOMId(id)
    }
    fn Editable(&self) -> bool {
        self.Read(|document, index| document.IsEditableForEditing(index))
            .unwrap_or(false)
    }
}

/// No lookup directory is kept. A native owner received from source traversal
/// is retained directly; id-only DOM queries use Layout's existing node index.
pub struct ResidentDOMCursorNodes<'a> {
    document: crate::ownership::InteractionDocument,
    layout: LayoutOutput<'a>,
    fragments: PaintFragmentCursor<'a>,
    child_frames: &'a dyn PreparedLocalChildFrameQueries,
    gap: CursorGap,
    editing_positions: Rc<dyn super::resident_editing_positions::ResidentEditingPositionQueries>,
    native_paths: std::sync::Arc<dyn foundation::native_path::NativePathFactory>,
}
impl<'a> ResidentDOMCursorNodes<'a> {
    pub fn new(
        document: crate::ownership::InteractionDocument,
        layout: LayoutOutput<'a>,
        fragments: PaintFragmentCursor<'a>,
        child_frames: &'a dyn PreparedLocalChildFrameQueries,
        editing_positions: Rc<
            dyn super::resident_editing_positions::ResidentEditingPositionQueries,
        >,
        native_paths: std::sync::Arc<dyn foundation::native_path::NativePathFactory>,
    ) -> Self {
        Self {
            document,
            layout,
            fragments,
            child_frames,
            gap: Rc::new(CursorDiagnostics::default()),
            editing_positions,
            native_paths,
        }
    }
    pub fn CheckQuery(&self) -> Result<(), ResidentCursorError> {
        self.gap
            .get()
            .map_or(Ok(()), |gap| Err(ResidentCursorError::Unsupported(gap)))
    }
    fn Value(&self, id: u64, native: Option<Persistent<NativeNode>>) -> Option<CursorNodeValue> {
        let identity = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            document.NodeHandle(document.FindNodeById(id)?).Identity()
        };
        Some(CursorNodeValue {
            document: self.document.clone(),
            id,
            identity,
            native,
            gap: self.gap.clone(),
            editing_positions: Some(self.editing_positions.clone()),
            native_paths: Some(self.native_paths.clone()),
        })
    }
    fn NativeForId(&self, id: u64) -> Option<Persistent<NativeNode>> {
        let node = self.layout.Node(id)?;
        let cursor = self.fragments.ForPaintObject(node.PaintObjectIdentity())?;
        let object = unsafe { cursor.Cursor().GetLayoutObject().as_ref() }?;
        let native = object.GetNode();
        (!native.is_null()).then(|| Persistent::from_ptr(native))
    }
    pub fn NodeForNative(&self, native: &NativeNode) -> Option<hit::NodeRef> {
        CursorNodeValue::ForNative(
            &self.document,
            native,
            self.gap.clone(),
            Some(self.editing_positions.clone()),
            Some(self.native_paths.clone()),
        )
        .map(|value| Rc::new(ResidentDOMCursorNode::new(value)) as hit::NodeRef)
    }
    pub fn CursorResult<'b>(
        &'b self,
        result: &'b ResidentCursorHitResult,
    ) -> Result<ResidentDOMCursorResult, ResidentCursorError> {
        let node = result
            .InnerPossiblyPseudoNode()
            .map(|node| {
                let id = node.DOMNodeId().ok_or(ResidentCursorError::Unsupported(
                    ResidentCursorCapability::DetachedNode,
                ))?;
                self.Value(id, node.NativeNodeOwner().cloned())
                    .map(ResidentDOMCursorNode::new)
                    .ok_or(ResidentCursorError::Unsupported(
                        ResidentCursorCapability::DetachedNode,
                    ))
            })
            .transpose()?;
        let scrollbar = result
            .GetScrollbar()
            .map(|bar| {
                let owner = bar
                    .NativeScrollbarOwner()
                    .ok_or(ResidentCursorError::Unsupported(
                        ResidentCursorCapability::ScrollbarCursor,
                    ))?
                    .clone();
                Ok::<_, ResidentCursorError>(ResidentCursorScrollbar {
                    owner,
                    part: Cell::new(0),
                    gap: self.gap.clone(),
                })
            })
            .transpose()?;
        Ok(ResidentDOMCursorResult {
            scrollbar,
            node,
            local: *result.LocalPoint(),
            over_link: result.IsOverLink(),
            over_corner: result.IsOverScrollCorner(),
        })
    }
}
impl ResidentDOMNodeOwners for ResidentDOMCursorNodes<'_> {
    fn NodeForId(&self, id: u64) -> Option<hit::NodeRef> {
        self.Value(id, self.NativeForId(id))
            .map(|value| Rc::new(ResidentDOMCursorNode::new(value)) as hit::NodeRef)
    }
}

#[derive(Clone, Copy)]
enum NativeCursorAreaKind {
    Own,
    Enclosing,
    View,
}
struct NativeCursorArea {
    value: CursorNodeValue,
    kind: NativeCursorAreaKind,
}
impl NativeCursorArea {
    fn Native(
        &self,
    ) -> Option<&layoutng_assembly::internal::layout_scrollable_area::PaintLayerScrollableArea>
    {
        let object = self.value.Object()?;
        let object: &LayoutBoxModelObject = match self.kind {
            NativeCursorAreaKind::Enclosing => {
                unsafe { object.EnclosingLayer().as_ref() }?.GetLayoutObject()
            }
            NativeCursorAreaKind::Own => {
                unsafe { foundation::DynamicTo::<LayoutBoxModelObject>(object).as_ref() }?
            }
            NativeCursorAreaKind::View => {
                let view: &LayoutObject = unsafe { object.View().as_ref() }?;
                unsafe { foundation::DynamicTo::<LayoutBoxModelObject>(view).as_ref() }?
            }
        };
        unsafe { object.GetScrollableArea().as_ref() }
    }
}
impl source_cursor::CursorScrollableArea for NativeCursorArea {
    fn IsAbsolutePointInResizeControlForPointer(&self, point: source_cursor::Point) -> bool {
        self.Native().is_some_and(|area| area.IsAbsolutePointInResizeControl(&gfx::Point::new(point.x, point.y),
            layoutng_assembly::internal::layout_scrollable_area::ResizerHitTestType::kResizerForPointer))
    }
    fn ScrollCornerStyle(&self) -> Option<&dyn source_cursor::CursorComputedStyle> {
        self.Native()?.ScrollCornerStyle().map(|_| self as _)
    }
}
struct NativeCursorLayer {
    value: CursorNodeValue,
    area: NativeCursorArea,
}
impl NativeCursorLayer {
    fn Box(&self) -> Option<&LayoutBox> {
        unsafe { self.value.Object()?.EnclosingLayer().as_ref() }
            .and_then(|layer| unsafe { layer.GetLayoutBox().as_ref() })
    }
}
impl source_cursor::CursorPaintLayer for NativeCursorLayer {
    fn GetScrollableArea(&self) -> Option<&dyn source_cursor::CursorScrollableArea> {
        self.area.Native().map(|_| &self.area as _)
    }
    fn GetLayoutBox(&self) -> &dyn source_cursor::CursorLayoutBox {
        self
    }
}
impl source_cursor::CursorLayoutBox for NativeCursorLayer {
    fn StyleRef(&self) -> &dyn source_cursor::CursorComputedStyle {
        self
    }
    fn ShouldPlaceBlockDirectionScrollbarOnLogicalLeft(&self) -> bool {
        self.Box()
            .is_some_and(|box_| box_.ShouldPlaceBlockDirectionScrollbarOnLogicalLeft())
    }
}
struct ResidentDOMCursorNode {
    value: CursorNodeValue,
    layer: NativeCursorLayer,
    area: NativeCursorArea,
    view_area: NativeCursorArea,
}
impl ResidentDOMCursorNode {
    fn new(value: CursorNodeValue) -> Self {
        Self {
            layer: NativeCursorLayer {
                value: value.clone(),
                area: NativeCursorArea {
                    value: value.clone(),
                    kind: NativeCursorAreaKind::Enclosing,
                },
            },
            area: NativeCursorArea {
                value: value.clone(),
                kind: NativeCursorAreaKind::Own,
            },
            view_area: NativeCursorArea {
                value: value.clone(),
                kind: NativeCursorAreaKind::View,
            },
            value,
        }
    }
    fn Node(value: CursorNodeValue) -> hit::NodeRef {
        Rc::new(Self::new(value))
    }
    fn PseudoFacts(&self) -> Option<dom::persistent_document::DOMPseudoElementFacts> {
        self.value
            .Read(|document, index| document.PseudoElementFacts(index))
            .flatten()
    }
}
impl hit::HitTestNode for ResidentDOMCursorNode {
    fn DOMIdentity(&self) -> u128 {
        self.value.identity
    }
    fn DOMNodeId(&self) -> Option<u64> {
        Some(self.value.id)
    }
    fn NativeNodeOwner(&self) -> Option<&Persistent<NativeNode>> {
        self.value.native.as_ref()
    }
    fn Element(&self) -> Option<&dyn hit::HitTestElement> {
        self.value
            .Read(|document, index| document.Node(index).Type() == DOMNodeType::kElement)
            .unwrap_or(false)
            .then_some(self as _)
    }
    fn Layout(&self) -> Option<&dyn hit::HitTestLayout> {
        self.value.Object().map(|_| self as _)
    }
    fn Document(&self) -> &dyn hit::HitTestDocument {
        self
    }
    fn FlatTreeParent(&self) -> Option<hit::NodeRef> {
        self.value.Parent(false).map(Self::Node)
    }
    fn FlatTreeParentElement(&self) -> Option<hit::NodeRef> {
        self.value.Parent(true).map(Self::Node)
    }
    fn Pseudo(&self) -> Option<&dyn hit::HitTestPseudoElement> {
        self.PseudoFacts().map(|_| self as _)
    }
    fn ContainingShadowRoot(&self) -> Option<Rc<dyn hit::HitTestShadowRoot>> {
        let (id, user_agent) = self
            .value
            .Read(|document, index| {
                let root = document.ContainingShadowRoot(index)?;
                Some((
                    document.Node(root).Id(),
                    document.ShadowRootMode(root)
                        == Some(dom::persistent_document::DOMShadowRootMode::kUserAgent),
                ))
            })
            .flatten()?;
        Some(Rc::new(ResidentShadowRoot {
            value: self.value.ForDOMId(id)?,
            user_agent,
        }))
    }
    fn OwnerShadowHost(&self) -> Option<hit::NodeRef> {
        let id = self
            .value
            .Read(|document, index| {
                document
                    .OwnerShadowHost(index)
                    .map(|host| document.Node(host).Id())
            })
            .flatten()?;
        self.value.ForDOMId(id).map(Self::Node)
    }
    fn IsInShadowTree(&self) -> bool {
        self.value
            .Read(|document, index| document.IsInShadowTree(index))
            .unwrap_or(false)
    }
    fn IsEditable(&self) -> bool {
        self.value.Editable()
    }
    fn CaretPosition(&self, query: hit::CaretPositionQuery) -> hit::PositionWithAffinity {
        self.value.Read(|_, _| ())?;
        let Some(queries) = self.value.editing_positions.as_ref() else {
            self.value
                .gap
                .set(Some(ResidentCursorCapability::EditingPositionOwner));
            return None;
        };
        super::resident_editing_positions::Retain(queries.CaretPosition(self.value.id, query))
    }
    fn ScrollableAreaIds(&self) -> Box<dyn Iterator<Item = foundation::CompositorElementId> + '_> {
        let Some(object) = self.value.Object() else {
            return Box::new(std::iter::empty());
        };
        Box::new(
            layoutng_assembly::layout_engine::NativeHitTestCursor::ForObject(object)
                .ScrollableAreas()
                .map(|area| area.GetScrollElementId()),
        )
    }
}
struct ResidentImageMap {
    value: CursorNodeValue,
    map: u64,
}
impl hit::HitTestImageMap for ResidentImageMap {
    fn AreaForPoint(
        &self,
        point: &foundation::PhysicalOffset,
        layout: &dyn hit::HitTestLayout,
    ) -> Option<hit::NodeRef> {
        let factory = self
            .value
            .native_paths
            .as_ref()
            .expect("resident image-map path backend");
        let id = self
            .value
            .Read(|document, _| {
                document.ImageMapAreaForPoint(
                    self.map,
                    *point,
                    layout.ImageMapContainer(),
                    factory.as_ref(),
                )
            })
            .flatten()?;
        self.value.ForDOMId(id).map(ResidentDOMCursorNode::Node)
    }
}
struct ResidentShadowRoot {
    value: CursorNodeValue,
    user_agent: bool,
}
impl hit::HitTestShadowRoot for ResidentShadowRoot {
    fn IsUserAgent(&self) -> bool {
        self.user_agent
    }
    fn Host(&self) -> hit::NodeRef {
        let id = self
            .value
            .Read(|document, index| {
                document
                    .ShadowHost(index)
                    .map(|host| document.Node(host).Id())
            })
            .flatten()
            .expect("resident shadow host");
        ResidentDOMCursorNode::Node(self.value.ForDOMId(id).expect("live shadow host"))
    }
}
impl hit::HitTestPseudoElement for ResidentDOMCursorNode {
    fn Kind(&self) -> hit::PseudoKind {
        use layoutng_style::style::computed_style_constants::PseudoId;
        match self.PseudoFacts().expect("source pseudo node").pseudo_id {
            PseudoId::kPseudoIdBefore => hit::PseudoKind::Before,
            PseudoId::kPseudoIdAfter => hit::PseudoKind::After,
            PseudoId::kPseudoIdMarker => hit::PseudoKind::Marker,
            PseudoId::kPseudoIdCheckMark => hit::PseudoKind::CheckMark,
            _ => hit::PseudoKind::Other,
        }
    }
    fn UltimateOriginatingElement(&self) -> hit::NodeRef {
        let id = self
            .value
            .Read(|document, _| {
                document
                    .Node(self.PseudoFacts().expect("pseudo").originating_element)
                    .Id()
            })
            .expect("originating element");
        Self::Node(self.value.ForDOMId(id).expect("live originating element"))
    }
    fn InnerNodeForHitTesting(&self) -> hit::NodeRef {
        self.UltimateOriginatingElement()
    }
    fn SupportsHitTesting(&self) -> bool {
        self.PseudoFacts()
            .is_some_and(|facts| facts.SupportsHitTesting())
    }
}
impl hit::HitTestElement for ResidentDOMCursorNode {
    fn Kind(&self) -> hit::ElementKind {
        use dom::persistent_document::DOMHitTestElementKind as Native;
        use hit::ElementKind as Kind;
        self.value
            .Read(|document, index| match document.HitTestElementKind(index) {
                Some(Native::Html) => Kind::Html,
                Some(Native::Image) => Kind::Image,
                Some(Native::InputImage) => Kind::InputImage,
                Some(Native::Input) => Kind::Input,
                Some(Native::TextArea) => Kind::TextArea,
                Some(Native::Embed) => Kind::Embed,
                Some(Native::Object) => Kind::Object,
                Some(Native::Area) => Kind::Area,
                Some(Native::Map) => Kind::Map,
                Some(Native::Media) => Kind::Media,
                _ => Kind::Other,
            })
            .unwrap_or(Kind::Other)
    }
    fn DirectionalTitle(&self) -> Option<(String, foundation::TextDirection)> {
        self.value
            .Read(|document, index| {
                document
                    .HitTestDirectionalTitle(index, self.value.WithStyle(|style| style.Direction()))
            })
            .flatten()
    }
    fn Title(&self) -> Option<String> {
        self.value
            .Read(|document, index| document.ElementTitleForHitTest(index))
            .flatten()
    }
    fn Alt(&self) -> Option<String> {
        self.value.Attribute("alt")
    }
    fn ImageSourceURL(&self) -> String {
        self.value
            .Read(|document, index| document.ImageSourceURLForHitTest(index))
            .unwrap_or_default()
    }
    fn HrefURL(&self) -> hit::Url {
        self.value
            .Read(|document, index| {
                document.HrefAttributeForHitTest(index).map(|value| {
                    document.CompleteURLValue(
                        value.trim_matches(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' ')),
                    )
                })
            })
            .flatten()
            .and_then(super::resident_hit_metadata::DocumentURL)
    }
    fn TextContent(&self) -> String {
        self.value
            .Read(|document, index| document.NodeTextContentForHitTest(index))
            .unwrap_or_default()
    }
    fn IsLiveLink(&self) -> bool {
        hit::HitTestElement::IsLink(self)
    }
    fn IsLink(&self) -> bool {
        self.value
            .Read(|document, index| document.IsLinkForEditing(index))
            .unwrap_or(false)
    }
    fn IsDisabledOrReadOnly(&self) -> bool {
        self.value
            .Read(|document, index| document.IsDisabledOrReadOnlyForHitTest(index))
            .unwrap_or(false)
    }
    fn IsTextField(&self) -> bool {
        crate::input_type::IsTextField(&self.value.document, self.value.id)
    }
    fn ImageElement(&self) -> Option<hit::NodeRef> {
        let id = self
            .value
            .Read(|document, _| document.ImageElementForMapOrArea(self.value.id))
            .flatten()?;
        self.value.ForDOMId(id).map(Self::Node)
    }
    fn ImageMap(&self) -> Option<Rc<dyn hit::HitTestImageMap>> {
        let map = self
            .value
            .Read(|document, _| document.ImageMapForImage(self.value.id))
            .flatten()?;
        Some(Rc::new(ResidentImageMap {
            value: self.value.clone(),
            map,
        }))
    }
    fn Media(&self) -> Option<Rc<dyn hit::HitTestMedia>> {
        if !(self.value.IsHTML("video") || self.value.IsHTML("audio")) {
            return None;
        }
        self.value.Read(|document, index| {
            Rc::new(super::resident_hit_metadata::ResidentMedia(
                document.MediaElementState(index),
            )) as Rc<dyn hit::HitTestMedia>
        })
    }
}
impl hit::HitTestLayout for ResidentDOMCursorNode {
    fn IsImage(&self) -> bool {
        self.value.Object().is_some_and(|object| object.IsImage())
    }
    fn IsSVGImage(&self) -> bool {
        self.value
            .Object()
            .is_some_and(|object| object.IsSVGImage())
    }
    fn IsBox(&self) -> bool {
        self.value.Object().is_some_and(|object| object.IsBox())
    }
    fn IsMedia(&self) -> bool {
        self.value.Object().is_some_and(|object| object.IsMedia())
    }
    fn LockedAncestorPreventingPaint(&self) -> bool {
        self.value
            .Object()
            .is_some_and(|object| object.HasLockedAncestorPreventingPaint())
    }
    fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        self.value
            .Object()
            .is_some_and(|object| object.ChildPaintBlockedByDisplayLock())
    }
    fn PositionForPoint(&self, point: &foundation::PhysicalOffset) -> hit::PositionWithAffinity {
        self.value.Read(|_, _| ())?;
        let node = self.value.native.as_ref()?;
        let Some(queries) = self.value.editing_positions.as_ref() else {
            self.value
                .gap
                .set(Some(ResidentCursorCapability::EditingPositionOwner));
            return None;
        };
        super::resident_editing_positions::Retain(queries.PositionForPoint(node, point))
    }
    fn Direction(&self) -> foundation::TextDirection {
        self.value
            .WithStyle(|style| style.Direction())
            .unwrap_or(foundation::TextDirection::kLtr)
    }
    fn AbsoluteContentQuad(&self) -> gfx::QuadF {
        self.value
            .Object()
            .and_then(|object| unsafe { foundation::DynamicTo::<LayoutBox>(object).as_ref() })
            .map(|box_| box_.AbsoluteContentQuad(Default::default()))
            .unwrap_or_default()
    }
    fn ImageMapContainer(&self) -> foundation::native_path::ImageMapContainer {
        foundation::native_path::ImageMapContainer {
            effective_zoom: self
                .value
                .WithStyle(|style| style.EffectiveZoom())
                .unwrap_or(1.0),
            border_box: self
                .value
                .Object()
                .and_then(|object| unsafe { foundation::DynamicTo::<LayoutBox>(object).as_ref() })
                .map(|box_| gfx::RectF::from(box_.PhysicalBorderBoxRect())),
        }
    }
    fn ImageContent(&self) -> Option<hit::ImageContentRef> {
        let object = self.value.Object()?;
        if !object.IsLayoutImage() || object.IsSVGImage() {
            return None;
        }
        let image = unsafe {
            foundation::DynamicTo::<layoutng_assembly::internal::layout_replaced::LayoutInputReplaced>(object).as_ref()
        }?;
        let content = image.ImageResource().CachedImage()?;
        Some(Rc::new(super::resident_hit_metadata::ResidentImageContent(
            content,
        )))
    }
    fn UpdateHitTestResult(
        &self,
        point: foundation::PhysicalOffset,
    ) -> Option<hit::HitTestNodeAndPosition> {
        if self.value.Object()?.NodeForHitTest().is_null() {
            return None;
        }
        Some(hit::HitTestNodeAndPosition {
            node: Self::Node(self.value.clone()),
            point,
        })
    }
}
impl hit::HitTestDocument for ResidentDOMCursorNode {
    fn IsPrePaintClean(&self) -> bool {
        self.value
            .Object()
            .is_none_or(|object| !object.NeedsLayout())
    }
    fn Frame(&self) -> Option<hit::FrameRef> {
        if self.value.editing_positions.is_none() {
            self.value
                .gap
                .set(Some(ResidentCursorCapability::EditingPositionOwner));
        }
        self.value
            .editing_positions
            .as_ref()
            .map(|queries| Rc::new(ResidentHitFrame(queries.clone())) as hit::FrameRef)
    }
    fn CompleteURL(&self, value: &str) -> hit::Url {
        self.value
            .Read(|document, _| document.CompleteURLValue(value))
            .and_then(super::resident_hit_metadata::DocumentURL)
    }
    fn DisallowInvalidation(&self) -> Box<dyn hit::HitTestInvalidationGuard + '_> {
        Box::new(ResidentReadQuery)
    }
}
struct ResidentHitFrame(Rc<dyn super::resident_editing_positions::ResidentEditingPositionQueries>);
impl hit::HitTestFrame for ResidentHitFrame {
    fn SelectionContains(&self, point: &foundation::PhysicalOffset) -> bool {
        self.0.SelectionContains(point)
    }
}
struct ResidentReadQuery;
impl hit::HitTestInvalidationGuard for ResidentReadQuery {}
impl SelectionStartNode for ResidentDOMCursorNode {
    fn LockedAncestorPreventingPaint(&self) -> bool {
        self.value
            .Read(|document, index| document.LockedAncestorPreventingPaintForEditing(index))
            .unwrap_or(false)
    }
    fn IsEditable(&self) -> bool {
        self.value.Editable()
    }
    fn LayoutSelectionStyle(&self) -> Option<(foundation::EUserSelect, foundation::EUserDrag)> {
        self.value.Object().map(|object| {
            (
                object.StyleRef().UsedUserSelect(),
                object.StyleRef().UserDrag(),
            )
        })
    }
    fn FlatTreeParentForSelection(&self) -> Option<Box<dyn SelectionStartNode + '_>> {
        self.value
            .Parent(false)
            .map(|value| Box::new(Self::new(value)) as _)
    }
}
impl source_cursor::CursorNode for ResidentDOMCursorNode {
    fn IsLink(&self) -> bool {
        hit::HitTestElement::IsLink(self)
    }
    fn AsHTMLInputElement(&self) -> Option<&dyn source_cursor::CursorInputElement> {
        self.value.IsHTML("input").then_some(self as _)
    }
    fn IsEditable(&self) -> bool {
        self.value.Editable()
    }
    fn IsTextNode(&self) -> bool {
        self.value
            .Read(|document, index| document.Node(index).Type() == DOMNodeType::kText)
            .unwrap_or(false)
    }
    fn CanStartSelection(&self) -> bool {
        node_hit_test::CanStartSelection(self).unwrap_or_else(|_| {
            let element = if self
                .value
                .Read(|document, index| document.Node(index).Type() == DOMNodeType::kElement)
                .unwrap_or(false)
            {
                Some(self.value.clone())
            } else {
                self.value.Parent(true)
            };
            if let Some(element) = element {
                self.value
                    .editing_positions
                    .as_ref()
                    .expect("resident selection lifecycle")
                    .UpdateSelectionStyleTree(element.id);
            }
            node_hit_test::CanStartSelectionAfterLayoutTreeUpdate(self)
        })
    }
    fn IsHTMLAreaElement(&self) -> bool {
        self.value.IsHTML("area")
    }
    fn GetLayoutObject(&self) -> Option<&dyn source_cursor::CursorLayoutObject> {
        self.value.Object().map(|_| self as _)
    }
    fn GetComputedStyle(&self) -> Option<&dyn source_cursor::CursorComputedStyle> {
        self.value.WithStyle(|_| ()).map(|_| self as _)
    }
    fn ImageElement(&self) -> Option<Rc<dyn source_cursor::CursorNode>> {
        let id = self
            .value
            .Read(|document, _| document.ImageElementForMapOrArea(self.value.id))
            .flatten()?;
        self.value
            .ForDOMId(id)
            .map(|value| Rc::new(Self::new(value)) as Rc<dyn source_cursor::CursorNode>)
    }
}
impl source_cursor::CursorInputElement for ResidentDOMCursorNode {
    fn FormControlType(&self) -> source_cursor::CursorFormControlType {
        if self
            .value
            .Attribute("type")
            .is_some_and(|kind| kind.eq_ignore_ascii_case("image"))
        {
            source_cursor::CursorFormControlType::kInputImage
        } else {
            source_cursor::CursorFormControlType::kOther
        }
    }
}
impl source_cursor::CursorLayoutObject for ResidentDOMCursorNode {
    fn EnclosingLayer(&self) -> &dyn source_cursor::CursorPaintLayer {
        &self.layer
    }
    fn GetScrollableArea(&self) -> Option<&dyn source_cursor::CursorScrollableArea> {
        self.area.Native().map(|_| &self.area as _)
    }
    fn IsDocumentElement(&self) -> bool {
        self.value
            .Object()
            .is_some_and(|object| object.IsDocumentElement())
    }
    fn ViewScrollableArea(&self) -> Option<&dyn source_cursor::CursorScrollableArea> {
        self.view_area.Native().map(|_| &self.view_area as _)
    }
    fn GetCursor(&self, point: source_cursor::PointF) -> source_cursor::CursorDirective {
        use layoutng_assembly::internal::layout_object::CursorDirective as Native;
        match self
            .value
            .Object()
            .expect("native cursor LayoutObject")
            .CursorForLocalPoint(gfx::PointF::new(point.x, point.y))
        {
            Native::kSetCursorBasedOnStyle => {
                source_cursor::CursorDirective::kSetCursorBasedOnStyle
            }
            Native::kDoNotSetCursor => source_cursor::CursorDirective::kDoNotSetCursor,
            Native::kSetCursor => {
                self.value.Gap(ResidentCursorCapability::CursorImage);
                source_cursor::CursorDirective::kDoNotSetCursor
            }
        }
    }
    fn StyleRef(&self) -> &dyn source_cursor::CursorComputedStyle {
        self
    }
}

trait NativeCursorStyleAccess {
    fn WithCompiledStyle<T>(&self, read: impl FnOnce(&ComputedStyle) -> T) -> Option<T>;
    fn ImageGap(&self) -> CursorGap;
}
impl NativeCursorStyleAccess for ResidentDOMCursorNode {
    fn WithCompiledStyle<T>(&self, read: impl FnOnce(&ComputedStyle) -> T) -> Option<T> {
        self.value.WithStyle(read)
    }
    fn ImageGap(&self) -> CursorGap {
        self.value.gap.clone()
    }
}
impl NativeCursorStyleAccess for NativeCursorLayer {
    fn WithCompiledStyle<T>(&self, read: impl FnOnce(&ComputedStyle) -> T) -> Option<T> {
        self.Box().map(|box_| read(box_.StyleRef()))
    }
    fn ImageGap(&self) -> CursorGap {
        self.value.gap.clone()
    }
}
// Source event_handler.cc:215-252; custom_scrollbar.cc:426-449. Each
// style read borrows the canonical part style; no cursor list is copied.
struct ResidentCursorScrollbar {
    owner: Persistent<layoutng_assembly::internal::scrollbar::Scrollbar>,
    part: Cell<u32>,
    gap: CursorGap,
}
impl ResidentCursorScrollbar {
    fn Native(&self) -> &layoutng_assembly::internal::scrollbar::Scrollbar {
        unsafe { &*self.owner.Get() }
    }
}
impl source_cursor::CursorScrollbar for ResidentCursorScrollbar {
    fn IsCustomScrollbar(&self) -> bool {
        self.Native().IsCustomScrollbar()
    }
    fn HoveredPart(&self) -> u32 {
        self.Native().HoveredPart() as u32
    }
    fn GetScrollbarPartStyleForCursor(
        &self,
        part: u32,
    ) -> Option<&dyn source_cursor::CursorComputedStyle> {
        self.Native().GetScrollbarPartStyleForCursor(part)?;
        self.part.set(part);
        Some(self)
    }
}
impl NativeCursorStyleAccess for ResidentCursorScrollbar {
    fn WithCompiledStyle<T>(&self, read: impl FnOnce(&ComputedStyle) -> T) -> Option<T> {
        self.Native()
            .GetScrollbarPartStyleForCursor(self.part.get())
            .map(read)
    }
    fn ImageGap(&self) -> CursorGap {
        self.gap.clone()
    }
}
impl NativeCursorStyleAccess for NativeCursorArea {
    fn WithCompiledStyle<T>(&self, read: impl FnOnce(&ComputedStyle) -> T) -> Option<T> {
        self.Native()?.ScrollCornerStyle().map(read)
    }
    fn ImageGap(&self) -> CursorGap {
        self.value.gap.clone()
    }
}
fn SourceCursor(style: &ComputedStyle) -> source_cursor::ECursor {
    SourceCursorValue(style.Cursor())
}
fn SourceCursorValue(cursor: StyleCursor) -> source_cursor::ECursor {
    use source_cursor::ECursor as Cursor;
    match cursor {
        StyleCursor::kAuto => Cursor::kAuto,
        StyleCursor::kDefault => Cursor::kDefault,
        StyleCursor::kNone => Cursor::kNone,
        StyleCursor::kContextMenu => Cursor::kContextMenu,
        StyleCursor::kHelp => Cursor::kHelp,
        StyleCursor::kPointer => Cursor::kPointer,
        StyleCursor::kProgress => Cursor::kProgress,
        StyleCursor::kWait => Cursor::kWait,
        StyleCursor::kCell => Cursor::kCell,
        StyleCursor::kCrosshair => Cursor::kCrosshair,
        StyleCursor::kText => Cursor::kText,
        StyleCursor::kVerticalText => Cursor::kVerticalText,
        StyleCursor::kAlias => Cursor::kAlias,
        StyleCursor::kCopy => Cursor::kCopy,
        StyleCursor::kMove => Cursor::kMove,
        StyleCursor::kNoDrop => Cursor::kNoDrop,
        StyleCursor::kNotAllowed => Cursor::kNotAllowed,
        StyleCursor::kEResize => Cursor::kEResize,
        StyleCursor::kNResize => Cursor::kNResize,
        StyleCursor::kNeResize => Cursor::kNeResize,
        StyleCursor::kNwResize => Cursor::kNwResize,
        StyleCursor::kSResize => Cursor::kSResize,
        StyleCursor::kSeResize => Cursor::kSeResize,
        StyleCursor::kSwResize => Cursor::kSwResize,
        StyleCursor::kWResize => Cursor::kWResize,
        StyleCursor::kEwResize => Cursor::kEwResize,
        StyleCursor::kNsResize => Cursor::kNsResize,
        StyleCursor::kNeswResize => Cursor::kNeswResize,
        StyleCursor::kNwseResize => Cursor::kNwseResize,
        StyleCursor::kColResize => Cursor::kColResize,
        StyleCursor::kRowResize => Cursor::kRowResize,
        StyleCursor::kAllScroll => Cursor::kAllScroll,
        StyleCursor::kZoomIn => Cursor::kZoomIn,
        StyleCursor::kZoomOut => Cursor::kZoomOut,
        StyleCursor::kGrab => Cursor::kGrab,
        StyleCursor::kGrabbing => Cursor::kGrabbing,
    }
}
impl<T: NativeCursorStyleAccess> source_cursor::CursorComputedStyle for T {
    fn Cursors(
        &self,
    ) -> Option<Box<dyn Iterator<Item = Box<dyn source_cursor::CursorData + '_>> + '_>> {
        let cursors = self.WithCompiledStyle(|style| style.Cursors())?;
        // The canonical list is retained by the native ComputedStyle owner
        // borrowed for this synchronous query. No style/layout update occurs
        // while EventHandler iterates the list; each projection borrows the
        // original CursorData and only records missing capabilities if visited.
        let cursors = unsafe { cursors.as_ref() }?;
        let gap = self.ImageGap();
        Some(Box::new(cursors.iter().map(move |cursor| {
            Box::new(super::cursor_image::CursorDataProjection::new(
                cursor,
                gap.clone(),
            )) as Box<dyn source_cursor::CursorData>
        })))
    }
    fn Cursor(&self) -> source_cursor::ECursor {
        self.WithCompiledStyle(SourceCursor)
            .unwrap_or(source_cursor::ECursor::kAuto)
    }
    fn IsHorizontalWritingMode(&self) -> bool {
        self.WithCompiledStyle(|style| style.IsHorizontalWritingMode())
            .unwrap_or(true)
    }
    fn UsedResize(&self) -> source_cursor::EResize {
        use foundation::EResize as Native;
        self.WithCompiledStyle(|style| match style.UsedResize() {
            Native::kNone => source_cursor::EResize::kNone,
            Native::kBoth => source_cursor::EResize::kBoth,
            Native::kHorizontal => source_cursor::EResize::kHorizontal,
            Native::kVertical => source_cursor::EResize::kVertical,
            Native::kBlock => source_cursor::EResize::kBlock,
            Native::kInline => source_cursor::EResize::kInline,
        })
        .unwrap_or(source_cursor::EResize::kNone)
    }
    fn ColorSchemeFlagsIsNormal(&self) -> bool {
        self.WithCompiledStyle(|style| style.ColorSchemeFlagsIsNormal())
            .unwrap_or(true)
    }
    fn DarkColorScheme(&self) -> bool {
        self.WithCompiledStyle(|style| style.DarkColorScheme())
            .unwrap_or(false)
    }
}

pub struct ResidentDOMCursorResult {
    scrollbar: Option<ResidentCursorScrollbar>,
    node: Option<ResidentDOMCursorNode>,
    local: foundation::PhysicalOffset,
    over_link: bool,
    over_corner: bool,
}
impl source_cursor::CursorHitTestResult for ResidentDOMCursorResult {
    fn InnerPossiblyPseudoNode(&self) -> Option<&dyn source_cursor::CursorNode> {
        self.node.as_ref().map(|node| node as _)
    }
    fn GetScrollbar(&self) -> Option<&dyn source_cursor::CursorScrollbar> {
        self.scrollbar.as_ref().map(|bar| bar as _)
    }
    fn IsOverScrollCorner(&self) -> bool {
        self.over_corner
    }
    fn IsOverLink(&self) -> bool {
        self.over_link
    }
    fn LocalPoint(&self) -> source_cursor::PointF {
        source_cursor::PointF {
            x: self.local.left.ToFloat(),
            y: self.local.top.ToFloat(),
        }
    }
}

/// Bind original manager state to the canonical source layer and DOM result
/// views. Frame, scroll/autoscroll and capture owners are required live ports.
/// Construct this value for a prepared query; it retains no Page or engine.
pub struct ResidentCursorBinding<'a> {
    pub paint: &'a paint::PaintEngine,
    pub fragments: PaintFragmentCursor<'a>,
    pub nodes: &'a ResidentDOMCursorNodes<'a>,
    pub layer_services: &'a dyn interaction_hit_test::resident::ResidentLayerServices,
    pub owners: &'a super::source_input_managers::ResidentCursorOwners,
    pub frame: &'a dyn source_cursor::CursorPolicyFrame,
    pub scroll: &'a dyn source_cursor::CursorScrollManager,
    pub state: &'a dyn source_cursor::CursorEventHandlerState,
}
impl ResidentCursorResolver for ResidentCursorBinding<'_> {
    fn CheckQuery(&self) -> Result<(), ResidentCursorError> {
        if let Some(error) = self.layer_services.QueryError() {
            return Err(ResidentCursorError::HitTest(error));
        }
        self.nodes.CheckQuery()
    }
    fn BindRootLayer<'a>(
        &'a self,
        _: &'a crate::ownership::InteractionDocument,
        paint_root: paint::resident_hit_test::ResidentPaintHitTest<'a>,
        fragments: PaintFragmentCursor<'a>,
    ) -> Result<
        Box<
            dyn interaction_hit_test::core::paint::paint_layer_hit_test::PaintLayerHitTest<
                    ResidentCursorHitResult,
                > + 'a,
        >,
        ResidentCursorError,
    > {
        let layer = interaction_hit_test::resident::ResidentLayer::new(
            self.paint,
            self.fragments,
            fragments,
            self.layer_services,
        )
        .ok_or(ResidentCursorError::PrePaintNotReady)?;
        if layer.resident.Identity() != paint_root.Identity() {
            return Err(ResidentCursorError::LayerIdentityMismatch);
        }
        Ok(Box::new(layer))
    }
    fn CursorHandler(
        &self,
    ) -> Result<source_cursor::EventHandlerCursorPolicy<'_>, ResidentCursorError> {
        Ok(self
            .owners
            .CursorPolicy(self.frame, self.scroll, self.state))
    }
    fn CursorResult<'a>(
        &'a self,
        _: &'a crate::ownership::InteractionDocument,
        result: &'a ResidentCursorHitResult,
    ) -> Result<Box<dyn source_cursor::CursorHitTestResult + 'a>, ResidentCursorError> {
        Ok(Box::new(self.nodes.CursorResult(result)?))
    }
}

/// Borrow the original EventHandler capture/accessibility slot. This adapter
/// cannot be constructed from compatibility hover/pressed state.
pub struct ResidentCursorCapture<
    'a,
    'scope,
    R: crate::core::input::event_handler::EventHandlerHitResult,
> {
    pub handler: &'a crate::core::input::event_handler::EventHandler<'scope, R>,
}
impl<R: crate::core::input::event_handler::EventHandlerHitResult>
    source_cursor::CursorEventHandlerState for ResidentCursorCapture<'_, '_, R>
{
    fn HasCapturingMouseEventsElement(&self) -> bool {
        self.handler.HasCapturingMouseEventsElement()
    }
    fn cursor_accessibility_scale_factor(&self) -> f32 {
        self.handler.CursorAccessibilityScaleFactor()
    }
}

// Production facade assembly and a future full EventHandler borrow this exact
// source-state Rc. It is not inferred from pointer capture, hover, or pressed
// compatibility state.
impl source_cursor::CursorEventHandlerState
    for crate::core::input::event_handler::EventHandlerCursorState
{
    fn HasCapturingMouseEventsElement(&self) -> bool {
        crate::core::input::event_handler::EventHandlerCursorState::HasCapturingMouseEventsElement(
            self,
        )
    }
    fn cursor_accessibility_scale_factor(&self) -> f32 {
        crate::core::input::event_handler::EventHandlerCursorState::CursorAccessibilityScaleFactor(
            self,
        )
    }
}

/// ScrollManager already implements CursorScrollManager by reading its actual
/// resize-area and frame AutoscrollController owners. Borrow that manager
/// directly; no compatibility boolean or replacement scroll state is accepted.
pub type ResidentCursorScrollOwner = interaction_scroll::core::input::scroll_manager::ScrollManager;

/// ChromeClient::GetScreenInfo/GetScreenInfos values. The embedding supplies
/// the native cursor size in DIP; it is independent of CSS and Layout geometry.
/// Source event_handler.cc:668-687; render_widget_host_view_base.cc:526-530,948.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResidentCursorHostMetrics {
    pub device_scale_factor: f32,
    pub system_cursor_size_dip: source_cursor::SizeF,
}

/// Hit traversal borrows canonical resident owners. Lifecycle must be clean
/// before traversal; the returned hit drops every loan before cursor policy.
pub struct PreparedResidentCursorQuery<'a> {
    pub layout: LayoutOutput<'a>,
    pub fragments: PaintFragmentCursor<'a>,
    pub paint: &'a paint::PaintEngine,
    pub scroll: &'a ResidentCursorScrollOwner,
    pub paths: &'a dyn paint::path_hit_test::PaintPathHitTestFactory,
    pub selections: &'a layoutng_assembly::editing_state::SelectionState,
    pub lifecycle_clean: bool,
    pub child_frames: &'a dyn PreparedLocalChildFrameQueries,
    pub host_metrics: ResidentCursorHostMetrics,
    pub preferred_color_scheme: source_cursor::ColorScheme,
    pub image_failures: &'a mut Vec<ResidentCursorImageFailure>,
    pub native_paths: std::sync::Arc<dyn foundation::native_path::NativePathFactory>,
    pub editing_positions:
        Rc<dyn super::resident_editing_positions::ResidentEditingPositionQueries>,
}

/// Source hit preparation borrows resident geometry. Cursor policy uses only
/// the short-query provider, allowing its source lifecycle barriers to run.
struct PreparedCursorFrame<'a> {
    selections: &'a layoutng_assembly::editing_state::SelectionState,
    layout: Option<LayoutOutput<'a>>,
    queries: Option<&'a dyn super::resident_editing_positions::ResidentEditingPositionQueries>,
    host_metrics: ResidentCursorHostMetrics,
    preferred_color_scheme: source_cursor::ColorScheme,
    visible_content_rect: Option<source_cursor::Rect>,
}
impl interaction_hit_test::resident::query_services::ResidentQueryViewport
    for PreparedCursorFrame<'_>
{
    fn RootFrameToVisualViewport(&self, point: gfx::Point) -> gfx::Point {
        // The current Page has no VisualViewport area, matching the same
        // PageScrollFrameState used by its canonical ScrollManager.
        point
    }
}
impl source_cursor::CursorPolicyFrame for PreparedCursorFrame<'_> {
    fn Page(&self) -> Option<&dyn source_cursor::CursorPage> {
        Some(self)
    }
    fn UpdateStyleAndLayout(&self, _: source_cursor::CursorDocumentUpdateReason) {
        if let Some(queries) = self.queries {
            queries.UpdateCursorStyleAndLayout();
        }
    }
    fn HTMLAreaElementDisplayNoneEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::HTMLAreaElementDisplayNoneEnabled()
    }
    fn ResolveColorSchemeForEmbedding(
        &self,
        style: &dyn source_cursor::CursorComputedStyle,
    ) -> source_cursor::ColorScheme {
        // Source StyleEngine::ResolveColorSchemeForEmbedding, cc:4659-4671.
        if style.ColorSchemeFlagsIsNormal() {
            self.preferred_color_scheme
        } else if style.DarkColorScheme() {
            source_cursor::ColorScheme::kDark
        } else {
            source_cursor::ColorScheme::kLight
        }
    }
    fn VisibleSelectionInDomTreeIsNone(&self) -> bool {
        self.selections.DOMSelection().is_none()
            && self.selections.Document().is_none()
            && self.selections.ActiveControl().is_none()
    }
    fn ContentLocalToRootAncestorPoint(
        &self,
        point: source_cursor::PointF,
    ) -> source_cursor::PointF {
        let point = gfx::PointF::new(point.x, point.y);
        let point = match self.layout {
            Some(layout) => layout.CursorContentLocalToRootAncestorPoint(point),
            None => self
                .queries
                .expect("cursor frame queries")
                .ContentLocalToRootAncestorPoint(point),
        };
        source_cursor::PointF {
            x: point.x(),
            y: point.y(),
        }
    }
    fn MapContentRectToVisualRectInRootAncestorSpace(
        &self,
        rect: source_cursor::Rect,
    ) -> source_cursor::Rect {
        let rect = gfx::RectF::new(
            gfx::PointF::new(rect.origin.x, rect.origin.y),
            gfx::SizeF::new(rect.size.width, rect.size.height),
        );
        let rect = match self.layout {
            Some(layout) => layout.CursorMapContentRectToRootAncestorSpace(rect),
            None => self
                .queries
                .expect("cursor frame queries")
                .MapContentRectToRootAncestorSpace(rect),
        };
        source_cursor::Rect {
            origin: source_cursor::PointF {
                x: rect.x(),
                y: rect.y(),
            },
            size: source_cursor::SizeF {
                width: rect.width(),
                height: rect.height(),
            },
        }
    }
}
impl source_cursor::CursorPage for PreparedCursorFrame<'_> {
    fn DeviceScaleFactor(&self, _: &dyn source_cursor::CursorPolicyFrame) -> f32 {
        self.host_metrics.device_scale_factor
    }
    fn SystemCursorSize(&self, _: &dyn source_cursor::CursorPolicyFrame) -> source_cursor::SizeF {
        self.host_metrics.system_cursor_size_dip
    }
    fn VisibleContentRectExcludingScrollbars(&self) -> source_cursor::Rect {
        if let Some(queries) = self.queries {
            let rect = queries.VisibleContentRectExcludingScrollbars();
            source_cursor::Rect {
                origin: source_cursor::PointF {
                    x: rect.x(),
                    y: rect.y(),
                },
                size: source_cursor::SizeF {
                    width: rect.width(),
                    height: rect.height(),
                },
            }
        } else {
            self.visible_content_rect
                .expect("borrowed hit frame geometry")
        }
    }
}

impl interaction_hit_test::resident::query_services::ResidentQueryNodes
    for ResidentDOMCursorNodes<'_>
{
    fn EmbeddedChildFrameView(
        &self,
        native: &NativeNode,
    ) -> Result<
        interaction_hit_test::core::layout::layout_embedded_content_hit_test::ChildFrameView<'_>,
        interaction_hit_test::core::layout::native_hit_test::NativeHitTestRequiredPort,
    > {
        use interaction_hit_test::core::layout::{
            layout_embedded_content_hit_test::ChildFrameView,
            native_hit_test::NativeHitTestRequiredPort,
        };
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let index = document
            .FindNodeById(native.InputId())
            .ok_or(NativeHitTestRequiredPort::EmbeddedContent)?;
        // Source layout_embedded_content.cc:236: only a genuine absent or
        // remote Frame lacks a LocalFrameView. An iframe tag alone is no Frame.
        let Some(frame) = document.ContentFrame(index) else {
            return Ok(ChildFrameView::NoLocalFrame);
        };
        if frame.IsRemoteFrame() {
            return Ok(ChildFrameView::NoLocalFrame);
        }
        let frame_identity = frame.Identity();
        // Release the parent's document borrow before entering the prepared
        // child query owner. It only borrows same-local-root resident owners.
        drop(owner);
        self.child_frames
            .ForFrame(frame_identity)
            .map(ChildFrameView::Local)
    }
    fn ResolveNode(&self, id: u64) -> Option<hit::NodeRef> {
        self.NodeForId(id)
    }
    fn ResolveNativeNode(&self, node: &NativeNode) -> Option<hit::NodeRef> {
        self.NodeForNative(node)
    }
    fn DocumentElement(&self) -> Option<hit::NodeRef> {
        let id = {
            let owner = self.document.borrow();
            let document = owner.GetDocument();
            document
                .DocumentElement()
                .map(|index| document.Node(index).Id())?
        };
        self.NodeForId(id)
    }
    fn TransitionFacts(&self, cursor: PaintFragmentCursor<'_>) -> (bool, bool) {
        let object = unsafe { cursor.Cursor().GetLayoutObject().as_ref() }
            .expect("resident layer LayoutObject");
        let node = unsafe { object.GetNode().as_ref() }.map(|node| node.InputId());
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let represented =
            document.LayoutObjectRepresentedViaViewTransition(node, object.IsLayoutView());
        let has_pseudo = node.and_then(|id| document.FindNodeById(id)).is_some_and(|index| {
            document.GeneratedPseudoElement(index, layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdViewTransition).is_some()
        });
        (represented, has_pseudo)
    }
}

fn QueryPreparedResident(
    interaction: &super::interaction::Interaction<'_>,
    document: &crate::ownership::InteractionDocument,
    point: layoutng_assembly::internal::layout_input::Offset,
    query: PreparedResidentCursorQuery<'_>,
    scroll_query: bool,
    block_query: bool,
) -> Result<PreparedQueryResult, ResidentCursorError> {
    if !query.lifecycle_clean {
        return Err(ResidentCursorError::PrePaintNotReady);
    }
    let nodes = ResidentDOMCursorNodes::new(
        document.clone(),
        query.layout,
        query.fragments,
        query.child_frames,
        query.editing_positions.clone(),
        query.native_paths.clone(),
    );
    let visible = query
        .layout
        .CursorVisibleContentRectExcludingScrollbars()
        .ok_or(ResidentCursorError::MissingFrameGeometry)?;
    let frame = PreparedCursorFrame {
        selections: query.selections,
        layout: Some(query.layout),
        queries: None,
        host_metrics: query.host_metrics,
        preferred_color_scheme: query.preferred_color_scheme,
        visible_content_rect: Some(source_cursor::Rect {
            origin: source_cursor::PointF {
                x: visible.x(),
                y: visible.y(),
            },
            size: source_cursor::SizeF {
                width: visible.width(),
                height: visible.height(),
            },
        }),
    };
    let services = interaction_hit_test::resident::query_services::ResidentQueryServices::new(
        query.paint,
        query.fragments,
        &nodes,
        &frame,
        query.paths,
    );
    let owners = interaction.SourceInputOwners();
    let binding = ResidentCursorBinding {
        paint: query.paint,
        fragments: query.fragments,
        nodes: &nodes,
        layer_services: &services,
        owners,
        frame: &frame,
        scroll: query.scroll,
        state: owners.event_handler.as_ref(),
    };
    let selected = if scroll_query {
        HitTestAtResidentWithFlags(
            document,
            point,
            query.paint,
            query.fragments,
            &binding,
            if block_query {
                interaction_hit_test::core::layout::hit_test_request::RequestType::kActive as u32
            } else {
                0
            },
        )
        .map(|mut result| {
            if block_query {
                result.SetToShadowHostIfInUAShadowRoot();
                PreparedQueryResult::ScrollTarget(
                    result
                        .InnerNodeOrImageMapImage()
                        .and_then(|node| node.DOMNodeId()),
                )
            } else {
                PreparedQueryResult::ScrollTarget(
                    result.InnerNode().and_then(|node| node.DOMNodeId()),
                )
            }
        })
    } else {
        let result = HitTestAtResident(document, point, query.paint, query.fragments, &binding)?;
        let result = nodes.CursorResult(&result)?;
        let zoom = query
            .fragments
            .paint
            .resources
            .as_ref()
            .map_or(1.0, |resources| resources.layout_zoom_factor)
            .max(f64::MIN_POSITIVE);
        Ok(PreparedQueryResult::Cursor(PreparedResidentCursorHit {
            result,
            gap: nodes.gap.clone(),
            location: ResidentCursorLocation(foundation::PhysicalOffset::new(
                foundation::LayoutUnit::FromDoubleRound(point.x * zoom),
                foundation::LayoutUnit::FromDoubleRound(point.y * zoom),
            )),
        }))
    };
    query
        .image_failures
        .extend(std::mem::take(&mut *nodes.gap.image_failures.borrow_mut()));
    selected
}

enum PreparedQueryResult {
    Cursor(PreparedResidentCursorHit),
    ScrollTarget(Option<u64>),
}
/// Owned hit facts retain only canonical DOM/native identities and source local
/// hit coordinates. No LayoutOutput, Paint borrow or fragment directory survives.
pub struct PreparedResidentCursorHit {
    result: ResidentDOMCursorResult,
    location: ResidentCursorLocation,
    gap: CursorGap,
}
pub fn PrepareResidentCursor(
    interaction: &super::interaction::Interaction<'_>,
    document: &crate::ownership::InteractionDocument,
    point: layoutng_assembly::internal::layout_input::Offset,
    query: PreparedResidentCursorQuery<'_>,
) -> Result<PreparedResidentCursorHit, ResidentCursorError> {
    let PreparedQueryResult::Cursor(hit) =
        QueryPreparedResident(interaction, document, point, query, false, false)?
    else {
        unreachable!()
    };
    Ok(hit)
}
/// Source event_handler.cc:573-854 runs after the hit-test loans are released.
/// Providers borrow resident frame geometry only for each individual query.
pub fn SelectPreparedResidentCursor(
    interaction: &super::interaction::Interaction<'_>,
    hit: PreparedResidentCursorHit,
    scroll: &ResidentCursorScrollOwner,
    selections: &layoutng_assembly::editing_state::SelectionState,
    queries: &dyn super::resident_editing_positions::ResidentEditingPositionQueries,
    host_metrics: ResidentCursorHostMetrics,
    preferred_color_scheme: source_cursor::ColorScheme,
    image_failures: &mut Vec<ResidentCursorImageFailure>,
) -> Result<Option<crate::cursor::Cursor>, ResidentCursorError> {
    let frame = PreparedCursorFrame {
        selections,
        layout: None,
        queries: Some(queries),
        host_metrics,
        preferred_color_scheme,
        visible_content_rect: None,
    };
    let owners = interaction.SourceInputOwners();
    let selected = owners
        .CursorPolicy(&frame, scroll, owners.event_handler.as_ref())
        .SelectCursor(&hit.location, &hit.result);
    image_failures.extend(std::mem::take(&mut *hit.gap.image_failures.borrow_mut()));
    hit.gap.get().map_or(Ok(selected), |gap| {
        Err(ResidentCursorError::Unsupported(gap))
    })
}

/// EventHandler already owns the canonical hit. Project that same result into
/// its existing cursor policy without hit testing again or borrowing Layout.
pub fn SelectEventHandlerCursor(
    interaction: &super::interaction::Interaction<'_>,
    document: &crate::ownership::InteractionDocument,
    location: &interaction_hit_test::core::layout::hit_test_location::HitTestLocation,
    result: &ResidentCursorHitResult,
    selections: &layoutng_assembly::editing_state::SelectionState,
    queries: Rc<dyn super::resident_editing_positions::ResidentEditingPositionQueries>,
    host_metrics: ResidentCursorHostMetrics,
    preferred_color_scheme: source_cursor::ColorScheme,
    native_paths: std::sync::Arc<dyn foundation::native_path::NativePathFactory>,
    image_failures: &mut Vec<ResidentCursorImageFailure>,
) -> Result<Option<crate::cursor::Cursor>, ResidentCursorError> {
    let gap = Rc::new(CursorDiagnostics::default());
    let node = result
        .InnerPossiblyPseudoNode()
        .map(|node| {
            let native = node
                .NativeNodeOwner()
                .and_then(|node| unsafe { node.Get().as_ref() })
                .ok_or(ResidentCursorError::Unsupported(
                    ResidentCursorCapability::DetachedNode,
                ))?;
            CursorNodeValue::ForNative(
                document,
                native,
                gap.clone(),
                Some(queries.clone()),
                Some(native_paths.clone()),
            )
            .map(ResidentDOMCursorNode::new)
            .ok_or(ResidentCursorError::Unsupported(
                ResidentCursorCapability::DetachedNode,
            ))
        })
        .transpose()?;
    let scrollbar = result
        .GetScrollbar()
        .map(|bar| {
            let owner =
                bar.NativeScrollbarOwner()
                    .cloned()
                    .ok_or(ResidentCursorError::Unsupported(
                        ResidentCursorCapability::ScrollbarCursor,
                    ))?;
            Ok::<_, ResidentCursorError>(ResidentCursorScrollbar {
                owner,
                part: Cell::new(0),
                gap: gap.clone(),
            })
        })
        .transpose()?;
    let hit = PreparedResidentCursorHit {
        result: ResidentDOMCursorResult {
            scrollbar,
            node,
            local: *result.LocalPoint(),
            over_link: result.IsOverLink(),
            over_corner: result.IsOverScrollCorner(),
        },
        location: ResidentCursorLocation(*location.Point()),
        gap,
    };
    let scroll = interaction
        .native_scroll
        .borrow()
        .clone()
        .ok_or(ResidentCursorError::MissingScrollOwner)?;
    SelectPreparedResidentCursor(
        interaction,
        hit,
        &scroll.ScrollManager(),
        selections,
        queries.as_ref(),
        host_metrics,
        preferred_color_scheme,
        image_failures,
    )
}
/// WebFrameWidgetImpl::GetScrollableContainerIdAt uses the same read-only
/// PaintLayer hit result as other widget point queries, then the source live
/// ScrollableAreaTraversal. No alternate fragment walk or scroll tree exists.
pub fn QueryPreparedResidentScrollContainer(
    interaction: &super::interaction::Interaction<'_>,
    document: &crate::ownership::InteractionDocument,
    point: layoutng_assembly::internal::layout_input::Offset,
    query: PreparedResidentCursorQuery<'_>,
) -> Result<Option<foundation::CompositorElementId>, ResidentCursorError> {
    let layout = query.layout;
    let PreparedQueryResult::ScrollTarget(target) =
        QueryPreparedResident(interaction, document, point, query, true, false)?
    else {
        unreachable!()
    };
    let selected = target.and_then(|id| {
        layout
            .WheelScrollableAreaTraversal(id)
            .next()
            .map(|area| area.GetScrollElementId())
    });
    if std::env::var_os("RECHROM_DIAG_INPUT_HIT_TEST").is_some() {
        eprintln!(
            "interaction-scroll-query point=({}, {}) target={target:?} selected={selected:?}",
            point.x, point.y,
        );
    }
    Ok(selected)
}

/// WebFrameWidgetImpl::ComputeBlockBound, cc:616-651. Uses the same source
/// read-only layer query; DOM supplies its live LayoutTreeBuilder parent and
/// Layout supplies the original object's inline classification and bounds.
pub fn QueryPreparedResidentBlockBounds(
    interaction: &super::interaction::Interaction<'_>,
    document: &crate::ownership::InteractionDocument,
    point: layoutng_assembly::internal::layout_input::Offset,
    query: PreparedResidentCursorQuery<'_>,
) -> Result<gfx::Rect, ResidentCursorError> {
    let layout = query.layout;
    let PreparedQueryResult::ScrollTarget(mut node) =
        QueryPreparedResident(interaction, document, point, query, true, true)?
    else {
        unreachable!()
    };
    let owner = document.borrow();
    let tree = owner.GetDocument();
    while let Some(id) = node {
        if let Some((is_inline, bounds)) = layout.BlockZoomBoundsForNode(id) {
            if !is_inline {
                return Ok(bounds);
            }
        }
        node = tree
            .FindNodeById(id)
            .and_then(|index| tree.LayoutTreeBuilderParent(index))
            .map(|index| tree.Node(index).Id());
    }
    Ok(gfx::Rect::default())
}
