//! Element::EnsureResizeObserverData: the canonical Element strongly retains
//! observations. Observer/Controller target links in the shared owner are weak.
use super::*;
#[derive(Default)]
pub(super) struct ElementResizeObserverState {
    data: Rc<resize_observer::ResizeObserverData>,
    target: RefCell<Option<Rc<dyn resize_observer::ResizeObserverTarget>>>,
}
impl PersistentDocument {
    fn EnsureResizeObserverState(&self, node: usize) -> Rc<ElementResizeObserverState> {
        assert_eq!(self.arena[node].node_type, DOMNodeType::kElement);
        self.arena[node]
            .resize_observer
            .borrow_mut()
            .get_or_insert_with(|| Rc::new(ElementResizeObserverState::default()))
            .clone()
    }
    pub fn EnsureResizeObserverData(&self, node: usize) -> Rc<resize_observer::ResizeObserverData> {
        self.EnsureResizeObserverState(node).data.clone()
    }
    pub fn ResizeObserverTarget(
        &self,
        node: usize,
    ) -> Option<Rc<dyn resize_observer::ResizeObserverTarget>> {
        let state = self.arena[node].resize_observer.borrow().clone()?;
        let target = state.target.borrow().clone();
        target
    }
    pub fn SetResizeObserverTarget(
        &self,
        node: usize,
        target: Rc<dyn resize_observer::ResizeObserverTarget>,
    ) {
        let state = self.EnsureResizeObserverState(node);
        assert!(
            Rc::ptr_eq(&target.Data(), &state.data),
            "same Element observation map"
        );
        let mut slot = state.target.borrow_mut();
        assert!(slot.is_none(), "one canonical Element target identity");
        *slot = Some(target);
    }
    // DisplayLockUtilities.cc:666-692 for the current independent local frame.
    pub fn ResizeObserverIsInLockedSubtree(&self, node: usize) -> bool {
        if self.DisplayLockLockedCount() == 0 {
            return false;
        }
        let mut ancestor = Some(node);
        while let Some(node) = ancestor {
            if self.DisplayLockIsLocked(node) {
                return true;
            }
            ancestor = self.FlatTreeParent(node);
        }
        false
    }
    // ResizeObservation::TargetDepth, cc:113-121: flat-tree Element ancestry.
    pub fn ResizeObserverTargetDepth(&self, node: usize) -> usize {
        let mut depth = 0;
        let mut parent = Some(node);
        while let Some(node) = parent {
            depth += 1;
            parent = self.FlatTreeParentElement(node);
        }
        depth
    }
}
