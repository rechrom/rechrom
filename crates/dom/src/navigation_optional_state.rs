//! Document-owned optional navigation supplements. NavigationState and
//! SkeletonLoader retain original URLs/element identity; producers and effects
//! remain typed instead of inventing absent owners or successful rendering.
use super::*;
#[derive(Clone, Copy, Debug)]
pub enum NavigationTraverseType {
    Back,
    Forward,
    Reload,
}
#[derive(Clone, Copy, Debug)]
pub enum NavigationPhase {
    Loading,
    Committed,
}
pub struct NavigationStateSupplement {
    pub old_url: String,
    pub new_url: String,
    pub source: Option<DOMOwnerHandle>,
    pub traverse: Option<NavigationTraverseType>,
    pub phase: Cell<NavigationPhase>,
}
impl NavigationStateSupplement {
    pub fn SetCommitted(&self) {
        self.phase.set(NavigationPhase::Committed);
    }
}
#[derive(Default)]
pub struct SkeletonLoaderSupplement {
    skeletons: RefCell<HashMap<String, Rc<SkeletonDocument>>>,
    current: RefCell<Option<Rc<SkeletonDocument>>>,
}
pub struct SkeletonDocument {
    pub url: String,
}
impl SkeletonLoaderSupplement {
    /// skeleton_loader.cc:35-41. Returns the actual new source resource owner
    /// for the caller's typed fetch admission; repeated links retain identity.
    pub fn AddSkeletonPrefetchLink(&self, url: String) -> Option<Rc<SkeletonDocument>> {
        use std::collections::hash_map::Entry;
        match self.skeletons.borrow_mut().entry(url.clone()) {
            Entry::Occupied(_) => None,
            Entry::Vacant(entry) => {
                let source = Rc::new(SkeletonDocument { url });
                entry.insert(source.clone());
                Some(source)
            }
        }
    }
    pub fn NavigateTo(&self, url: &str) -> Option<Rc<SkeletonDocument>> {
        assert!(self.current.borrow().is_none());
        let skeleton = self.skeletons.borrow().get(url).cloned()?;
        *self.current.borrow_mut() = Some(skeleton.clone());
        Some(skeleton)
    }
    pub fn CancelNavigation(&self) -> bool {
        self.current.borrow_mut().take().is_some()
    }
}
impl PersistentDocument {
    /// DynamicTo<HTMLFormElement>, using the class assigned at allocation.
    pub fn IsHTMLFormElement(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::HTMLFormElement)
    }
    pub fn NavigationStateIfExists(&self) -> Option<Rc<NavigationStateSupplement>> {
        self.navigation_state.borrow().clone()
    }
    pub fn StartNavigationState(
        &self,
        url: String,
        source: Option<usize>,
        direction: Option<NavigationTraverseType>,
    ) -> Rc<NavigationStateSupplement> {
        let state = Rc::new(NavigationStateSupplement {
            old_url: self.DocumentURL().to_owned(),
            new_url: url,
            source: source.map(|n| self.NodeHandle(n)),
            traverse: direction,
            phase: Cell::new(NavigationPhase::Loading),
        });
        *self.navigation_state.borrow_mut() = Some(state.clone());
        state
    }
    pub fn FinishNavigationState(&self) -> Option<Rc<NavigationStateSupplement>> {
        self.navigation_state.borrow_mut().take()
    }
    pub fn SkeletonLoaderIfExists(&self) -> Option<Rc<SkeletonLoaderSupplement>> {
        self.skeleton_loader.borrow().clone()
    }
    pub fn EnsureSkeletonLoader(&self) -> Rc<SkeletonLoaderSupplement> {
        self.skeleton_loader
            .borrow_mut()
            .get_or_insert_with(|| Rc::new(Default::default()))
            .clone()
    }
}
