//! Identity-bearing subset of the original ViewTransition supplement/tracker.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! view_transition_supplement.cc:151-159,313-349,468-504;
//! view_transition_style_tracker.cc:562-587,805-891,1023-1082,
//! 1146-1167,1209-1286,1357-1374,1956-1973;
//! view_transition.cc:1124-1145,1161-1188; utils.cc:196-226.
//!
//! Snapshot capture, CSS paint-order discovery and pseudo generation are not
//! implemented here. Their original source commit points admit identities via
//! the methods below; a CSS name alone never admits an active participant.
use super::*;
#[path = "view_transition_css_discovery.rs"]
mod css_discovery;
pub use css_discovery::{
    ViewTransitionCSSDiscoveryError, ViewTransitionCSSLayerFacts, ViewTransitionCSSPaintOrder,
    ViewTransitionRandomToken,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewTransitionTrackerState {
    Idle,
    Capturing,
    OldSnapshotFrozen,
    Captured,
    Started,
    Finished,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewTransitionTerminalState {
    Finished,
    Aborted,
    TimedOut,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewTransitionParticipationError {
    NoDocumentElement,
    DuplicateName(String),
    ElementInAnotherTransition(DOMOwnerHandle),
}
struct ViewTransitionStyleTracker {
    state: ViewTransitionTrackerState,
    // Original ElementData::target_element, keyed by the original name. Old
    // names survive CaptureResolved while their live element pointers clear.
    elements: HashMap<String, Option<DOMOwnerHandle>>,
    pending: HashMap<DOMOwnerHandle, Vec<(String, usize)>>,
    sequence: usize,
    root_transitioning: bool,
    group_state: HashMap<String, (String, Option<String>)>,
    id_to_auto_name: HashMap<String, String>,
    auto_name_token: Option<[u64; 2]>,
    apply_overflow_clip: bool,
    ordered_names: Vec<String>,
}
impl ViewTransitionStyleTracker {
    fn new() -> Self {
        Self {
            state: ViewTransitionTrackerState::Idle,
            elements: HashMap::new(),
            pending: HashMap::new(),
            sequence: 0,
            root_transitioning: false,
            group_state: HashMap::new(),
            id_to_auto_name: HashMap::new(),
            auto_name_token: None,
            apply_overflow_clip: false,
            ordered_names: Vec::new(),
        }
    }
    fn AddTransitionElement(&mut self, element: DOMOwnerHandle, name: String) {
        self.AddTransitionElementWithGroups(element, name, None, None);
    }
    fn AddTransitionElementWithGroups(
        &mut self,
        element: DOMOwnerHandle,
        name: String,
        nearest: Option<String>,
        contain: Option<String>,
    ) {
        if let Some(nearest) = nearest {
            self.group_state.insert(name.clone(), (nearest, contain));
        }
        let names = self.pending.entry(element).or_default();
        if names.iter().any(|(existing, _)| *existing == name) {
            return;
        }
        names.push((name, self.sequence));
        self.sequence += 1;
    }
    fn IsTransitionElement(&self, element: DOMOwnerHandle, root: Option<DOMOwnerHandle>) -> bool {
        if matches!(
            self.state,
            ViewTransitionTrackerState::Idle | ViewTransitionTrackerState::Captured
        ) {
            return false;
        }
        if Some(element) == root {
            return self.root_transitioning;
        }
        self.elements
            .values()
            .any(|target| *target == Some(element))
    }
    fn Targets(&self) -> Vec<DOMOwnerHandle> {
        self.elements
            .values()
            .filter_map(|target| *target)
            .collect()
    }
    fn Flatten(&self) -> Vec<(String, DOMOwnerHandle)> {
        let mut flat: Vec<_> = self
            .pending
            .iter()
            .flat_map(|(element, names)| {
                names
                    .iter()
                    .map(move |(name, order)| (*order, name.clone(), *element))
            })
            .collect();
        flat.sort_by_key(|entry| entry.0);
        flat.into_iter()
            .map(|(_, name, element)| (name, element))
            .collect()
    }
}

/// One original transition owner. Scope and target members keep DOM arena
/// identity; no LayoutObject tree or geometry is retained in this source state.
pub struct ViewTransition {
    is_preview: bool,
    document: DOMOwnerHandle,
    scope: Option<DOMOwnerHandle>,
    // StyleTracker::element_ is distinct from ViewTransition::scope_. A
    // script transition on documentElement retains that original element.
    tracker_element: Option<DOMOwnerHandle>,
    terminal: std::cell::Cell<Option<ViewTransitionTerminalState>>,
    tracker: RefCell<ViewTransitionStyleTracker>,
}
impl ViewTransition {
    /// This is the owning-state construction step of CreateFromScript/
    /// CreateFromSnapshot, not a replacement for either script entry algorithm.
    fn new(
        document: DOMOwnerHandle,
        scope: Option<DOMOwnerHandle>,
        tracker_element: Option<DOMOwnerHandle>,
    ) -> Rc<Self> {
        assert!(scope.is_none_or(|scope| scope.arena_id == document.arena_id));
        Rc::new(Self {
            is_preview: false,
            document,
            scope,
            tracker_element,
            terminal: std::cell::Cell::new(None),
            tracker: RefCell::new(ViewTransitionStyleTracker::new()),
        })
    }
    pub fn IsDone(&self) -> bool {
        self.terminal.get().is_some()
    }
    pub fn TrackerState(&self) -> ViewTransitionTrackerState {
        self.tracker.borrow().state
    }
    fn IsTransitionElementExcludingRoot(
        &self,
        node: DOMOwnerHandle,
        root: Option<DOMOwnerHandle>,
    ) -> bool {
        !self.IsDone()
            && Some(node) != root
            && self.tracker.borrow().IsTransitionElement(node, root)
    }
    fn NeedsEffectNode(
        &self,
        layout_view: bool,
        element: Option<DOMOwnerHandle>,
        root: Option<DOMOwnerHandle>,
    ) -> bool {
        if layout_view {
            return self.scope.is_none() && !self.IsDone();
        }
        if self.scope.is_some() && element == self.scope {
            return !self.IsDone();
        }
        element.is_some_and(|element| self.IsTransitionElementExcludingRoot(element, root))
    }
    fn IsRepresentedViaPseudoElements(
        &self,
        layout_view: bool,
        element: Option<DOMOwnerHandle>,
        root: Option<DOMOwnerHandle>,
    ) -> bool {
        if self.IsDone() {
            return false;
        }
        if layout_view {
            return root.is_some_and(|root_element| {
                self.tracker
                    .borrow()
                    .IsTransitionElement(root_element, root)
            });
        }
        element.is_some_and(|element| self.IsTransitionElementExcludingRoot(element, root))
    }
}

pub struct ViewTransitionSupplement {
    document: DOMOwnerHandle,
    document_transition: RefCell<Option<Rc<ViewTransition>>>,
    element_transitions: RefCell<HashMap<DOMOwnerHandle, Rc<ViewTransition>>>,
    preview_types: RefCell<Option<Vec<String>>>,
}
#[derive(Clone, Copy, Debug)]
pub enum NavigationPreviewRequired {
    CreatePreview,
    SkipExistingTransition,
}
impl ViewTransitionSupplement {
    /// source supplement.cc:214-238. Existing preview/transition owners decide
    /// whether work exists; pending unadmitted native effects remain explicit.
    pub fn StartNavigationPreviewIfNeeded(&self) -> Result<(), NavigationPreviewRequired> {
        if self.preview_types.borrow().is_none() {
            return Ok(());
        }
        if self.document_transition.borrow().is_some() {
            return Err(NavigationPreviewRequired::SkipExistingTransition);
        }
        Err(NavigationPreviewRequired::CreatePreview)
    }
    pub fn AbortNavigationPreview(&self) -> Result<(), NavigationPreviewRequired> {
        if self
            .document_transition
            .borrow()
            .as_ref()
            .is_some_and(|t| t.is_preview)
        {
            return Err(NavigationPreviewRequired::SkipExistingTransition);
        }
        Ok(())
    }

    pub fn HasDocumentTransition(&self) -> bool {
        self.document_transition.borrow().is_some()
    }
    fn new(document: DOMOwnerHandle) -> Self {
        Self {
            document,
            document_transition: RefCell::new(None),
            element_transitions: RefCell::new(HashMap::new()),
            preview_types: RefCell::new(None),
        }
    }
    fn Transitions(&self) -> Vec<Rc<ViewTransition>> {
        // Source copies just the active transition list before iteration,
        // since the visitor can finish/remove transitions during that visit.
        let mut transitions: Vec<_> = self.document_transition.borrow().iter().cloned().collect();
        transitions.extend(self.element_transitions.borrow().values().cloned());
        transitions
    }
    fn OnTransitionCreated(&self, transition: Rc<ViewTransition>) {
        assert_eq!(self.document, transition.document);
        if let Some(scope) = transition.scope {
            assert!(
                self.element_transitions
                    .borrow_mut()
                    .insert(scope, transition)
                    .is_none(),
                "source skips an existing scope transition before registering its replacement"
            );
        } else {
            assert!(
                self.document_transition
                    .borrow_mut()
                    .replace(transition)
                    .is_none(),
                "source skips an existing document transition before registering its replacement"
            );
        }
    }
    fn OnTransitionFinished(&self, transition: &Rc<ViewTransition>) {
        let is_document = self
            .document_transition
            .borrow()
            .as_ref()
            .is_some_and(|owner| Rc::ptr_eq(owner, transition));
        if is_document {
            self.document_transition.borrow_mut().take();
        } else if let Some(scope) = transition.scope {
            self.element_transitions.borrow_mut().remove(&scope);
        }
    }
}

impl PersistentDocument {
    pub fn CreateViewTransitionOwner(&self, scope: Option<usize>) -> Rc<ViewTransition> {
        let tracker_element = scope.map(|scope| self.NodeHandle(scope));
        let scope = scope.filter(|scope| Some(*scope) != self.DocumentElement());
        if let Some(scope) = scope {
            assert_eq!(self.Node(scope).Type(), DOMNodeType::kElement);
        }
        ViewTransition::new(
            self.RootHandle(),
            scope.map(|scope| self.NodeHandle(scope)),
            tracker_element,
        )
    }
    pub fn GetViewTransitionsIfExists(&self) -> Option<Rc<ViewTransitionSupplement>> {
        self.view_transitions.borrow().clone()
    }
    fn EnsureViewTransitions(&self) -> Rc<ViewTransitionSupplement> {
        self.view_transitions
            .borrow_mut()
            .get_or_insert_with(|| Rc::new(ViewTransitionSupplement::new(self.RootHandle())))
            .clone()
    }
    /// The original supplement's registration commit, after the source's
    /// creation/skip checks. The script/snapshot entry algorithms stay absent.
    pub fn RegisterViewTransition(&self, transition: Rc<ViewTransition>) {
        self.EnsureViewTransitions().OnTransitionCreated(transition);
    }
    pub fn AddViewTransitionElement(&self, transition: &ViewTransition, node: usize, name: String) {
        assert_eq!(transition.document, self.RootHandle());
        assert_eq!(self.Node(node).Type(), DOMNodeType::kElement);
        transition
            .tracker
            .borrow_mut()
            .AddTransitionElement(self.NodeHandle(node), name);
    }
    fn VerifiedTransitionElements(
        &self,
        transition: &ViewTransition,
    ) -> Result<Vec<(String, DOMOwnerHandle)>, ViewTransitionParticipationError> {
        assert_eq!(transition.document, self.RootHandle());
        if self.DocumentElement().is_none() {
            return Err(ViewTransitionParticipationError::NoDocumentElement);
        }
        let flat = transition.tracker.borrow().Flatten();
        let root = self.DocumentElement().map(|node| self.NodeHandle(node));
        let transitions = self
            .GetViewTransitionsIfExists()
            .expect("registered transition supplement")
            .Transitions();
        let mut names = HashSet::new();
        for (name, element) in &flat {
            if !names.insert(name) {
                return Err(ViewTransitionParticipationError::DuplicateName(
                    name.clone(),
                ));
            }
            if transitions
                .iter()
                .any(|other| other.IsTransitionElementExcludingRoot(*element, root))
            {
                return Err(ViewTransitionParticipationError::ElementInAnotherTransition(*element));
            }
        }
        Ok(flat)
    }
    fn InvalidateTransitionTargets(&mut self, transition: &ViewTransition) {
        let targets = transition.tracker.borrow().Targets();
        for target in targets
            .into_iter()
            .chain(transition.scope)
            .chain(self.DocumentElement().map(|node| self.NodeHandle(node)))
        {
            if let Some(node) = self.FindNodeById(target.Id()) {
                if self.NodeHandle(node) == target {
                    self.InvalidateNodeStyle(node);
                }
            }
        }
    }
    pub fn CaptureViewTransition(
        &mut self,
        transition: &ViewTransition,
    ) -> Result<(), ViewTransitionParticipationError> {
        let elements = self.VerifiedTransitionElements(transition)?;
        let root = self.DocumentElement().map(|node| self.NodeHandle(node));
        {
            let mut tracker = transition.tracker.borrow_mut();
            assert_eq!(tracker.state, ViewTransitionTrackerState::Idle);
            tracker.state = ViewTransitionTrackerState::Capturing;
            for (name, element) in elements {
                tracker.ordered_names.push(name.clone());
                tracker.elements.insert(name, Some(element));
                if Some(element) == root {
                    tracker.root_transitioning = true;
                }
            }
            tracker.pending.clear();
            tracker.sequence = 0;
        }
        self.InvalidateTransitionTargets(transition);
        Ok(())
    }
    pub fn FreezeViewTransitionOldSnapshot(&self, transition: &ViewTransition) {
        assert_eq!(transition.document, self.RootHandle());
        let mut tracker = transition.tracker.borrow_mut();
        assert_eq!(tracker.state, ViewTransitionTrackerState::Capturing);
        tracker.state = ViewTransitionTrackerState::OldSnapshotFrozen;
    }
    pub fn ResolveViewTransitionCapture(&mut self, transition: &ViewTransition) {
        assert_eq!(transition.document, self.RootHandle());
        {
            let mut tracker = transition.tracker.borrow_mut();
            assert_eq!(tracker.state, ViewTransitionTrackerState::OldSnapshotFrozen);
            tracker.state = ViewTransitionTrackerState::Captured;
        }
        // Source invalidates the old live elements BEFORE clearing pointers.
        self.InvalidateTransitionTargets(transition);
        let mut tracker = transition.tracker.borrow_mut();
        for target in tracker.elements.values_mut() {
            *target = None;
        }
        tracker.root_transitioning = false;
    }
    pub fn StartViewTransitionAnimation(
        &mut self,
        transition: &ViewTransition,
    ) -> Result<(), ViewTransitionParticipationError> {
        let elements = self.VerifiedTransitionElements(transition)?;
        let root = self.DocumentElement().map(|node| self.NodeHandle(node));
        {
            let mut tracker = transition.tracker.borrow_mut();
            assert_eq!(tracker.state, ViewTransitionTrackerState::Captured);
            tracker.state = ViewTransitionTrackerState::Started;
            for (name, element) in elements {
                if !tracker.elements.contains_key(&name) {
                    tracker.ordered_names.push(name.clone());
                }
                tracker.elements.insert(name, Some(element));
                if Some(element) == root {
                    tracker.root_transitioning = true;
                }
            }
        }
        self.InvalidateTransitionTargets(transition);
        Ok(())
    }
    pub fn FinishViewTransition(
        &mut self,
        transition: &Rc<ViewTransition>,
        reason: ViewTransitionTerminalState,
    ) {
        assert_eq!(transition.document, self.RootHandle());
        transition.terminal.set(Some(reason));
        transition.tracker.borrow_mut().state = ViewTransitionTrackerState::Finished;
        self.InvalidateTransitionTargets(transition);
        {
            let mut tracker = transition.tracker.borrow_mut();
            tracker.elements.clear();
            tracker.pending.clear();
            tracker.sequence = 0;
            tracker.root_transitioning = false;
            tracker.ordered_names.clear();
        }
        self.GetViewTransitionsIfExists()
            .expect("registered transition supplement")
            .OnTransitionFinished(transition);
    }
    pub fn LayoutObjectRepresentedViaViewTransition(
        &self,
        node: Option<u64>,
        layout_view: bool,
    ) -> bool {
        let Some(supplement) = self.GetViewTransitionsIfExists() else {
            return false;
        };
        let root = self.DocumentElement().map(|node| self.NodeHandle(node));
        let element = node
            .and_then(|node| self.FindNodeById(node))
            .filter(|node| self.Node(*node).Type() == DOMNodeType::kElement)
            .map(|node| self.NodeHandle(node));
        let mut represented = false;
        for transition in supplement.Transitions() {
            if transition.NeedsEffectNode(layout_view, element, root) {
                represented = transition.IsRepresentedViaPseudoElements(layout_view, element, root);
            }
        }
        represented
    }
}
