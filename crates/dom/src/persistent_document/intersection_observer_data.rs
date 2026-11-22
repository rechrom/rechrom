//! ElementIntersectionObserverData and Document's canonical controller slot.
use super::*;
#[derive(Default)]
pub(super) struct ElementIntersectionObserverState {
    data: Rc<intersection_observer::ElementIntersectionObserverData>,
    target: RefCell<Option<Rc<dyn intersection_observer::IntersectionObserverTarget>>>,
}
impl PersistentDocument {
    /// Factory-retained HTMLImageElement class, shared by LazyMediaHelper.
    pub fn IsHTMLImageElement(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::HTMLImageElement)
    }

    fn EnsureIntersectionObserverState(&self, node: usize) -> Rc<ElementIntersectionObserverState> {
        assert!(matches!(
            self.Node(node).Type(),
            DOMNodeType::kElement | DOMNodeType::kDocument
        ));
        self.arena[node]
            .intersection_observer
            .borrow_mut()
            .get_or_insert_with(|| Rc::new(ElementIntersectionObserverState::default()))
            .clone()
    }
    pub fn EnsureIntersectionObserverData(
        &self,
        node: usize,
    ) -> Rc<intersection_observer::ElementIntersectionObserverData> {
        self.EnsureIntersectionObserverState(node).data.clone()
    }
    pub fn IntersectionObserverTarget(
        &self,
        node: usize,
    ) -> Option<Rc<dyn intersection_observer::IntersectionObserverTarget>> {
        let state = self.arena[node].intersection_observer.borrow().clone()?;
        let target = state.target.borrow().clone();
        target
    }
    pub fn SetIntersectionObserverTarget(
        &self,
        node: usize,
        target: Rc<dyn intersection_observer::IntersectionObserverTarget>,
    ) {
        let state = self.EnsureIntersectionObserverState(node);
        assert!(Rc::ptr_eq(&target.Data(), &state.data));
        assert!(
            state.target.borrow().is_none(),
            "one canonical Node intersection target"
        );
        *state.target.borrow_mut() = Some(target);
    }
    pub fn IntersectionController(
        &self,
        document: DOMOwnerHandle,
    ) -> Option<Rc<intersection_observer::IntersectionObserverController>> {
        let index = self.FindNodeById(document.Id())?;
        if self.NodeHandle(index) != document {
            return None;
        }
        self.document_metadata
            .get(&document.Id())?
            .intersection_observer_controller
            .get()
            .cloned()
    }
    pub fn SetIntersectionController(
        &self,
        document: DOMOwnerHandle,
        controller: Rc<intersection_observer::IntersectionObserverController>,
    ) {
        let index = self
            .FindNodeById(document.Id())
            .expect("same Document identity");
        assert_eq!(self.NodeHandle(index), document);
        self.document_metadata
            .get(&document.Id())
            .expect("Document metadata allocated with owner")
            .intersection_observer_controller
            .set(controller)
            .unwrap_or_else(|_| panic!("one Document intersection controller"));
    }
    pub fn IntersectionObserverOwnerDocument(&self, node: usize) -> DOMOwnerHandle {
        if self.Node(node).Type() == DOMNodeType::kDocument {
            self.NodeHandle(node)
        } else {
            self.Node(node)
                .OwnerDocumentNode()
                .expect("canonical Node owner document")
        }
    }
    pub(super) fn RecordIntersectionObserverConnected(&self, node: usize) {
        if self.arena[node]
            .intersection_observer
            .borrow()
            .as_ref()
            .is_some_and(|state| !state.data.IsEmpty())
        {
            self.AttributeLifecycle()
                .RecordIntersectionObserverConnected(self.Node(node).Id());
        }
    }
}
