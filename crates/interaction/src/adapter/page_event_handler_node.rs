//! EventHandler's Node leaves over the canonical DOM/event-target owner.
//! Source: core/input/event_handler.cc:901-926,962-967,1023-1028,1643,
//! 2655-2660; core/dom/node.cc:3018-3033.
//!
//! A wrapper retains identity only. DOM facts are read at the source call and
//! geometry is queried through the live Layout owner; no fragment is copied.
use super::{dom_event::DomEventTargets, event::EventTargetResolver};
use crate::{
    core::input::event_handler::{EventHandlerNode, NodeRef},
    ownership::InteractionDocument,
};
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_hit_test::core::layout::{hit_test_location::HitTestLocation, hit_test_result};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

/// Missing capabilities are rejected while assembling the binding, before an
/// input can alter retained manager state. There are deliberately no defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredEventHandlerPort {
    NodeGeometry,
    AtomicMoveState,
    ScrollControls,
    HitTestGeometry,
    HitTestLifecycle,
    FrameCoordinates,
    HoverActiveState,
    FrameDomain,
    FrameRegistry,
    FrameInput,
    FrameChrome,
    ChromeClient,
    Cursor,
    ActiveIntervalTimer,
    MiddleClickAutoscroll,
}

/// These are synchronous queries of Layout's resident node index and native
/// LayoutObject coordinate mapping, not an alternate hit-test tree.
pub trait PageEventHandlerNodeGeometry {
    fn HitTestNodeForId(&self, id: u64) -> Option<hit_test_result::NodeRef>;
    fn AbsoluteToLocalPoint(&self, id: u64, point: &HitTestLocation) -> HitTestLocation;
}
pub trait PageEventHandlerAtomicMoveState {
    fn StatePreservingAtomicMoveInProgress(&self) -> bool;
}
pub struct PageEventHandlerResidentNodeGeometry(
    pub Rc<dyn super::page_pointer_hit_test::PagePointerGeometry>,
);
impl PageEventHandlerNodeGeometry for PageEventHandlerResidentNodeGeometry {
    fn HitTestNodeForId(&self, id: u64) -> Option<hit_test_result::NodeRef> {
        match self.0.EventHandlerHitTestNode(id) {
            Ok(node) => node,
            Err(error) => std::panic::panic_any(error),
        }
    }
    fn AbsoluteToLocalPoint(&self, id: u64, point: &HitTestLocation) -> HitTestLocation {
        match self.0.EventHandlerAbsoluteToLocalPoint(id, point) {
            Ok(point) => point,
            Err(error) => std::panic::panic_any(error),
        }
    }
}

pub struct PageEventHandlerNodes {
    document: InteractionDocument,
    targets: Rc<DomEventTargets>,
    geometry: Rc<dyn PageEventHandlerNodeGeometry>,
    atomic_move: Rc<dyn PageEventHandlerAtomicMoveState>,
    wrappers: RefCell<HashMap<i32, Weak<PageEventHandlerNode>>>,
    self_: Weak<Self>,
}

impl PageEventHandlerNodes {
    pub fn new(
        document: InteractionDocument,
        targets: Rc<DomEventTargets>,
        geometry: Option<Rc<dyn PageEventHandlerNodeGeometry>>,
        atomic_move: Option<Rc<dyn PageEventHandlerAtomicMoveState>>,
    ) -> Result<Rc<Self>, RequiredEventHandlerPort> {
        let geometry = geometry.ok_or(RequiredEventHandlerPort::NodeGeometry)?;
        let atomic_move = atomic_move.ok_or(RequiredEventHandlerPort::AtomicMoveState)?;
        assert!(Rc::ptr_eq(&document, &targets.DocumentOwner()));
        Ok(Rc::new_cyclic(|self_| Self {
            document,
            targets,
            geometry,
            atomic_move,
            wrappers: RefCell::new(HashMap::new()),
            self_: self_.clone(),
        }))
    }
    pub fn Targets(&self) -> &Rc<DomEventTargets> {
        &self.targets
    }
    pub fn NodeForTarget(&self, target: &EventTargetRef) -> Option<NodeRef> {
        let id = self.targets.NodeId(target)?;
        if id == 0 {
            return None;
        }
        let id = i32::try_from(id).expect("DOMNodeId must fit Chromium's source integer");
        if let Some(node) = self.wrappers.borrow().get(&id).and_then(Weak::upgrade) {
            return Some(node);
        }
        let node = Rc::new(PageEventHandlerNode {
            id,
            target: target.clone(),
            owner: self.self_.upgrade().expect("live node registry"),
        });
        self.wrappers.borrow_mut().insert(id, Rc::downgrade(&node));
        Some(node)
    }
    pub fn NodeForId(&self, id: u64) -> Option<NodeRef> {
        self.NodeForTarget(&self.targets.ResolveTarget(id)?)
    }
    pub fn OwnsHitNode(&self, node: &hit_test_result::NodeRef) -> bool {
        let Some(id) = node.DOMNodeId() else {
            return false;
        };
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        document
            .FindNodeById(id)
            .is_some_and(|index| document.NodeHandle(index).Identity() == node.DOMIdentity())
    }
    pub fn NodeForHit(&self, node: &hit_test_result::NodeRef) -> Option<NodeRef> {
        self.OwnsHitNode(node)
            .then(|| self.NodeForId(node.DOMNodeId()?))
            .flatten()
    }
    pub fn EventTarget(&self, node: &NodeRef) -> Option<EventTargetRef> {
        let wrapper = self
            .wrappers
            .borrow()
            .get(&node.DomNodeId())
            .and_then(Weak::upgrade)?;
        let canonical: NodeRef = wrapper.clone();
        Rc::ptr_eq(&canonical, node).then(|| wrapper.target.clone())
    }
    pub fn HitTestNode(&self, node: &NodeRef) -> Option<hit_test_result::NodeRef> {
        let target = self.EventTarget(node)?;
        self.geometry
            .HitTestNodeForId(self.targets.NodeId(&target)?)
    }
}

struct PageEventHandlerNode {
    id: i32,
    target: EventTargetRef,
    owner: Rc<PageEventHandlerNodes>,
}
impl PageEventHandlerNode {
    fn Owner(&self) -> Rc<PageEventHandlerNodes> {
        self.owner.clone()
    }
    fn Read<T>(&self, read: impl FnOnce(&dom::Document, usize) -> T) -> T {
        let owner = self.Owner();
        let borrowed = owner.document.borrow();
        let document = borrowed.GetDocument();
        let index = document
            .FindNodeById(self.id as u64)
            .expect("canonical EventHandler node retains its DOM owner");
        read(document, index)
    }
}
impl EventHandlerNode for PageEventHandlerNode {
    fn DomNodeId(&self) -> i32 {
        self.id
    }
    fn StatePreservingAtomicMoveInProgress(&self) -> bool {
        self.Owner()
            .atomic_move
            .StatePreservingAtomicMoveInProgress()
    }
    fn AbsoluteToLocalPoint(&self, point: &HitTestLocation) -> HitTestLocation {
        self.Owner()
            .geometry
            .AbsoluteToLocalPoint(self.id as u64, point)
    }
    fn EnclosingLinkEventParentOrSelf(&self) -> Option<NodeRef> {
        let id = self.Read(|d, i| d.EnclosingLinkEventParentOrSelf(i).map(|i| d.Node(i).Id()))?;
        self.Owner().NodeForId(id)
    }
    fn HasParent(&self) -> bool {
        self.Read(|d, i| d.Node(i).Parent().is_some())
    }
    fn IsShadowRootWithInputHost(&self) -> bool {
        self.Read(|d, i| {
            d.ShadowHost(i)
                .is_some_and(|host| d.Node(host).IsHTMLElement("input"))
        })
    }
    fn IsElementWithTitle(&self) -> bool {
        self.Read(|document, node| {
            document.HitTestElementKind(node).is_some()
                && document.ElementTitleForHitTest(node).is_some()
        })
    }
    fn IsSVGUseElement(&self) -> bool {
        self.Read(|d, i| {
            d.Node(i)
                .IsElement("use", dom::persistent_document::DOMNamespace::kSVG)
        })
    }
    fn ContainingShadowRootHasSVGUseHost(&self) -> bool {
        self.Read(|d, i| {
            d.OwnerShadowHost(i).is_some_and(|host| {
                d.Node(host)
                    .IsElement("use", dom::persistent_document::DOMNamespace::kSVG)
            })
        })
    }
}
