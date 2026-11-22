//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! core/overscroll/overscroll_area_tracker.h:24-32,35,42-45; .cc:55-171;
//! core/dom/element.cc:5725-5752,14166-14199.
//! The Element rare-data owner retains membership. Layout supplies only the
//! original pseudo scroll area's small snap/geometry facts, never a new tree.
use super::{DOMNodeType, PersistentDocument};
pub use foundation::overscroll::{
    OverscrollAction as DOMOverscrollAction, OverscrollGeometry as DOMOverscrollGeometry,
    OverscrollOffset as DOMOverscrollOffset, OverscrollOpenBox as DOMOverscrollOpenBox,
    OverscrollRect as DOMOverscrollRect, OverscrollSnapData as DOMOverscrollSnapData,
    OverscrollToggleTargets as DOMOverscrollToggleTargets,
};
use std::collections::HashSet;
use std::{cell::RefCell, cmp::Ordering, rc::Rc};
#[derive(Debug)]
pub(super) struct OverscrollAreaTracker {
    container: usize,
    members: Vec<usize>,
    needs_dom_sort: bool,
}
impl OverscrollAreaTracker {
    fn new(container: usize) -> Self {
        Self {
            container,
            members: Vec::new(),
            needs_dom_sort: false,
        }
    }
    // .cc:94-115. The CHECK is deliberately different from OpenArea's guard.
    fn ToggleArea(geometry: Option<DOMOverscrollGeometry>) -> Option<DOMOverscrollAction> {
        let geometry = geometry?;
        let data = geometry
            .snap
            .expect("overscroll ToggleArea requires SnapContainerData");
        assert!(
            data.area_count >= 2,
            "overscroll ToggleArea requires two snap areas"
        );
        let targets = data
            .toggle_targets
            .expect("ToggleArea supplies first-area and actual target ids");
        Some(
            if targets.target_x_id == targets.first_area_id
                && targets.target_y_id == targets.first_area_id
            {
                DOMOverscrollAction::Open
            } else {
                DOMOverscrollAction::Close
            },
        )
    }
    // .cc:117-162. PhysicalContentBoxRect and snap positions remain Layout's.
    fn OpenArea(geometry: Option<DOMOverscrollGeometry>) -> Option<DOMOverscrollOffset> {
        let geometry = geometry?;
        let data = geometry.snap.filter(|data| data.area_count >= 2)?;
        let rect = data
            .second_area_rect
            .expect("two snap areas include second area");
        let box_ = geometry
            .open_box
            .expect("OpenArea supplies actual origin and content-box size");
        let min_x = (rect.x - box_.scroll_origin.x).min(0.0);
        let min_y = (rect.y - box_.scroll_origin.y).min(0.0);
        let max_x = (rect.x + rect.width - box_.content_box_width - box_.scroll_origin.x).max(0.0);
        let max_y =
            (rect.y + rect.height - box_.content_box_height - box_.scroll_origin.y).max(0.0);
        let mut offset = DOMOverscrollOffset::default();
        if (-min_x).max(max_x) > (-min_y).max(max_y) {
            offset.x = if -min_x >= max_x { min_x } else { max_x };
        } else {
            offset.y = if -min_y >= max_y { min_y } else { max_y };
        }
        Some(offset)
    }
}
#[derive(Debug, Default)]
pub(super) struct ElementOverscrollData {
    container: Option<usize>,
    tracker: Option<OverscrollAreaTracker>,
}
#[derive(Default)]
pub(super) struct OverscrollDocumentData {
    gestures_enabled: Option<bool>,
    invokers: HashSet<usize>,
    targets: HashSet<usize>,
    dirty: bool,
}
impl PersistentDocument {
    /// Runtime feature admission is immutable for this Document. Parsing may
    /// precede Page's source assembly, so the existing per-node insertion owner
    /// retains only candidate ids until this value is available. A disabled
    /// feature drops that bootstrap set and has no subsequent registry work.
    pub fn AdmitOverscrollGesturesFeature(&mut self, enabled: bool) {
        if let Some(previous) = self.overscroll_document.gestures_enabled {
            assert_eq!(
                previous, enabled,
                "one overscroll runtime feature owner per Document"
            );
            return;
        }
        self.overscroll_document.gestures_enabled = Some(enabled);
        if !enabled {
            self.overscroll_document.invokers.clear();
            self.overscroll_document.dirty = !self.overscroll_document.targets.is_empty();
        }
    }
    // HTML element.cc:886-903,3942-3946,3975-3979; Document
    // cc:10435-10495. Collect only native HTML invokers from the existing
    // insertion/removal/attribute owners. Runtime feature gating is supplied
    // by the source command resolver before target admission.
    pub(super) fn UpdateOverscrollCommandInvoker(&mut self, node: usize) {
        if self.overscroll_document.gestures_enabled == Some(false) {
            return;
        }
        let is_invoker = self.arena[node].node_type == DOMNodeType::kElement
            && self.arena[node].namespace == super::DOMNamespace::kHTML
            && self.IsConnectedToDocument(node)
            && self.arena[node].FindAttribute("command").is_some_and(|a| {
                ["toggle-overscroll", "show-overscroll", "hide-overscroll"]
                    .iter()
                    .any(|v| a.value.eq_ignore_ascii_case(v))
            });
        let changed = if is_invoker {
            self.overscroll_document.invokers.insert(node)
        } else {
            self.overscroll_document.invokers.remove(&node)
        };
        if changed {
            self.MarkOverscrollCommandTargetsDirty();
        }
    }
    pub(super) fn RemoveOverscrollCommandInvoker(&mut self, node: usize) {
        if self.overscroll_document.invokers.remove(&node) {
            self.MarkOverscrollCommandTargetsDirty();
        }
    }
    pub fn MarkOverscrollCommandTargetsDirty(&mut self) {
        if self.overscroll_document.invokers.is_empty()
            && self.overscroll_document.targets.is_empty()
        {
            return;
        }
        self.overscroll_document.dirty = true;
    }
    pub(super) fn OverscrollAttributeChanged(&mut self, node: usize, name: &str, namespace: &str) {
        if !namespace.is_empty() {
            return;
        }
        if name == "command" {
            self.UpdateOverscrollCommandInvoker(node);
        } else if (name == "commandfor" && self.overscroll_document.invokers.contains(&node))
            || (name == "id" && self.IsConnectedToDocument(node))
        {
            self.MarkOverscrollCommandTargetsDirty();
        }
    }
    /// One small cross-package batch of registered scalar identities. No DOM
    /// borrow spans lazy HTML owner construction or command target resolution.
    pub fn PendingOverscrollCommandInvokerIds(&self) -> Option<Vec<u64>> {
        self.overscroll_document.dirty.then(|| {
            self.overscroll_document
                .invokers
                .iter()
                .map(|&n| self.arena[n].id)
                .collect()
        })
    }
    pub fn UpdateOverscrollCommandTargets(
        &mut self,
        resolved_targets: impl IntoIterator<Item = u64>,
    ) -> bool {
        if !self.overscroll_document.dirty {
            return false;
        }
        let targets = resolved_targets
            .into_iter()
            .filter_map(|id| self.FindNodeById(id))
            .collect::<HashSet<_>>();
        let removed = self
            .overscroll_document
            .targets
            .difference(&targets)
            .copied()
            .collect::<Vec<_>>();
        let added = targets
            .difference(&self.overscroll_document.targets)
            .copied()
            .collect::<Vec<_>>();
        let changed = !removed.is_empty() || !added.is_empty();
        for target in removed {
            self.OverscrollTargetStateChanged(target);
        }
        for target in added {
            self.OverscrollTargetStateChanged(target);
        }
        self.overscroll_document.targets = targets;
        self.overscroll_document.dirty = false;
        changed
    }
    fn EnsureOverscrollData(&mut self, node: usize) -> Rc<RefCell<ElementOverscrollData>> {
        assert!(self.arena[node].node_type == DOMNodeType::kElement);
        self.arena[node]
            .overscroll
            .get_or_insert_with(|| Rc::new(RefCell::new(ElementOverscrollData::default())))
            .clone()
    }
    pub fn GetOverscrollContainer(&self, node: usize) -> Option<usize> {
        self.arena[node]
            .overscroll
            .as_ref()
            .and_then(|data| data.borrow().container)
    }
    pub fn HasOverscrollAreaTracker(&self, container: usize) -> bool {
        self.arena[container]
            .overscroll
            .as_ref()
            .is_some_and(|data| data.borrow().tracker.is_some())
    }
    pub(super) fn EnsureOverscrollAreaTracker(&mut self, container: usize) {
        self.EnsureOverscrollData(container)
            .borrow_mut()
            .tracker
            .get_or_insert_with(|| OverscrollAreaTracker::new(container));
    }
    // Tracker Add/Remove/RemoveAll, .cc:58-92. Members retain DOM arena identity.
    pub(super) fn AddOverscrollArea(&mut self, container: usize, node: usize) {
        assert!(self.GetOverscrollContainer(node).is_none());
        debug_assert!(self.IsConnectedToDocument(node));
        self.EnsureOverscrollAreaTracker(container);
        self.EnsureOverscrollData(node).borrow_mut().container = Some(container);
        let data = self.arena[container].overscroll.as_ref().unwrap();
        {
            let mut data = data.borrow_mut();
            let tracker = data.tracker.as_mut().unwrap();
            debug_assert_eq!(tracker.container, container);
            tracker.members.push(node);
            tracker.needs_dom_sort = tracker.members.len() > 1;
        }
        // SetNeedsReattachLayoutTree on the original container, even when
        // no ComputedStyle value changed. Geometry queries must not reuse the
        // old attachment while these source pseudo children are pending.
        self.style_state
            .layout_reattach_nodes
            .insert(self.arena[container].id);
        self.MarkOverscrollPseudoUpdate(container);
    }
    fn MarkOverscrollPseudoUpdate(&mut self, node: usize) {
        if !self.IsConnectedToDocument(node) {
            return;
        }
        self.RecordLayoutInputChange(node);
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::TREE);
        self.style_state.measurement_geometry_revision = self
            .style_state
            .measurement_geometry_revision
            .wrapping_add(1);
    }
    pub(super) fn DetachOverscroll(&mut self, node: usize) {
        let Some(container) = self.GetOverscrollContainer(node) else {
            return;
        };
        let data = self.arena[container]
            .overscroll
            .as_ref()
            .expect("overscroll container has rare data");
        {
            let mut data = data.borrow_mut();
            let tracker = data
                .tracker
                .as_mut()
                .expect("overscroll container has its tracker");
            tracker.members.retain(|&member| member != node);
            tracker.needs_dom_sort = tracker.needs_dom_sort && tracker.members.len() > 1;
        }
        self.arena[node]
            .overscroll
            .as_ref()
            .unwrap()
            .borrow_mut()
            .container = None;
    }
    pub(super) fn RemoveAllOverscrollAreas(&mut self, container: usize) {
        let Some(data) = self.arena[container].overscroll.as_ref().cloned() else {
            return;
        };
        let members = {
            let mut data = data.borrow_mut();
            let Some(tracker) = data.tracker.as_mut() else {
                return;
            };
            tracker.needs_dom_sort = false;
            std::mem::take(&mut tracker.members)
        };
        for node in members {
            self.arena[node]
                .overscroll
                .as_ref()
                .expect("member has rare data")
                .borrow_mut()
                .container = None;
        }
    }
    // Element::OverscrollTargetStateChanged, element.cc:9120-9123. The
    // Document's target set remains the sole owner of target membership.
    fn OverscrollTargetStateChanged(&mut self, node: usize) {
        self.InvalidateNodeStyle(node);
    }
    pub fn IsOverscrollCommandTarget(&self, node: usize) -> bool {
        self.overscroll_document.targets.contains(&node)
    }
    // Element::RecalcOwnStyle .cc:5725-5752, after the same ComputedStyle owner
    // has reached this element. This runs once per element, not in a Page scan.
    pub fn OverscrollDidStyleSelf(&mut self, node: usize) {
        if self.arena[node].node_type != DOMNodeType::kElement {
            return;
        }
        let parent = self.arena[node]
            .parent
            .filter(|&p| self.arena[p].node_type == DOMNodeType::kElement);
        // Element::ParentComputedStyle cc:5226-5241 uses the layout-tree-
        // builder's ordinary Parent (flat parent), not its LayoutParent which
        // skips display:contents. The relationship container remains the DOM
        // parentElement, as RecalcOwnStyle specifies.
        let parent_style_type = self.FlatTreeParentElement(node)
            .filter(|&parent| !matches!(&self.arena[parent].class,
                super::editing_node::DOMNodeClass::HTMLObjectElement(owner) if !owner.UseFallbackContent()))
            .and_then(|parent| self.ResolvedStyleFor(parent))
            .and_then(|style| style.ComputedStyle())
            .filter(|style| !style.IsEnsuredInDisplayNone())
            .map(|style| style.EffectiveOverscrollContainerType());
        let valid = self.IsOverscrollCommandTarget(node)
            && self
                .ResolvedStyleFor(node)
                .and_then(|s| s.ComputedStyle())
                .is_some_and(|s| {
                    !s.IsEnsuredInDisplayNone() && s.IsInternalOverscrollPositionAuto()
                })
            && parent_style_type.is_some_and(|type_| {
                type_ != foundation::style_constants::EOverscrollContainerType::kNone
            });
        if self
            .GetOverscrollContainer(node)
            .is_some_and(|c| !valid || Some(c) != parent)
        {
            self.DetachOverscroll(node);
            self.MarkOverscrollPseudoUpdate(node);
        }
        if self.GetOverscrollContainer(node).is_none() && valid {
            self.AddOverscrollArea(
                parent.expect("valid overscroll area has element parent"),
                node,
            );
            self.MarkOverscrollPseudoUpdate(node);
        }
    }
    pub fn ResolveOverscrollToggle(
        &self,
        node: usize,
        geometry: Option<DOMOverscrollGeometry>,
        gestures_enabled: bool,
    ) -> Option<DOMOverscrollAction> {
        assert!(gestures_enabled);
        let container = self.GetOverscrollContainer(node)?;
        if !self.HasOverscrollAreaTracker(container) {
            return None;
        }
        OverscrollAreaTracker::ToggleArea(geometry)
    }
    pub fn OverscrollAreaOffset(
        &self,
        node: usize,
        action: DOMOverscrollAction,
        geometry: Option<DOMOverscrollGeometry>,
        gestures_enabled: bool,
    ) -> Option<DOMOverscrollOffset> {
        assert!(gestures_enabled);
        let container = self.GetOverscrollContainer(node)?;
        if !self.HasOverscrollAreaTracker(container) {
            return None;
        }
        match action {
            DOMOverscrollAction::Open => OverscrollAreaTracker::OpenArea(geometry),
            DOMOverscrollAction::Close => geometry.map(|_| DOMOverscrollOffset::default()),
            DOMOverscrollAction::Toggle => {
                unreachable!("ToggleArea dispatches to OpenArea or CloseArea")
            }
        }
    }
    // DOMSortedElements .cc:67-77. Borrow the tracker's actual arena-index
    // membership slice in DOM order, without a second collection or tree.
    pub fn OverscrollMembersDOMSorted(
        &self,
        container: usize,
    ) -> Option<std::cell::Ref<'_, [usize]>> {
        let data = self.arena[container].overscroll.as_ref()?;
        {
            let mut data = data.borrow_mut();
            let tracker = data.tracker.as_mut()?;
            if tracker.needs_dom_sort {
                tracker
                    .members
                    .sort_by(|&a, &b| self.CompareOverscrollDOMOrder(a, b));
                tracker.needs_dom_sort = false;
            }
        }
        Some(std::cell::Ref::map(data.borrow(), |data| {
            data.tracker
                .as_ref()
                .expect("retained tracker")
                .members
                .as_slice()
        }))
    }
    fn CompareOverscrollDOMOrder(&self, a: usize, b: usize) -> Ordering {
        fn path(d: &PersistentDocument, mut n: usize) -> Vec<usize> {
            let mut p = vec![n];
            while let Some(parent) = d.arena[n].parent {
                p.push(parent);
                n = parent;
            }
            p.reverse();
            p
        }
        if a == b {
            return Ordering::Equal;
        }
        let a = path(self, a);
        let b = path(self, b);
        let common = a.iter().zip(&b).take_while(|(a, b)| a == b).count();
        if common == a.len() {
            return Ordering::Less;
        }
        if common == b.len() {
            return Ordering::Greater;
        }
        debug_assert!(
            common > 0,
            "overscroll members share the connected DOM tree"
        );
        let children = &self.arena[a[common - 1]].children;
        children
            .iter()
            .position(|&n| n == a[common])
            .cmp(&children.iter().position(|&n| n == b[common]))
    }
}
