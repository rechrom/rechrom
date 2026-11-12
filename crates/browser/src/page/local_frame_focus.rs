//! Native LocalFrame focus assembly. Chromium: focus_controller.cc:1784-1914,
//! 2381-2429; local_frame_view.cc:4220-4254; document.cc:6257-6351. The FrameTree
//! owns stable frame identities; each current Document supplies its own original
//! Style/Layout/FrameSelection leaves. No root-document node id crosses a child.
use super::local_frame::{LocalFrame, LocalFrameDomain, LocalFrameRuntime};
use super::*;
use interaction::adapter::{
    page_element_focusability::PageElementFocusLayout,
    page_focus_controller::{DocumentFocusLifecycle, FocusWithinSnapUpdate},
    page_focus_navigation::{
        FocusDocumentEventTarget, FocusNavigationFrameEffect, FocusNavigationFramePort,
        FocusNavigationFrameTree, FocusNavigationLayout,
    },
};
use interaction::default::core::page::focus_controller::{
    FocusController, FocusDispatchScope, FocusDocumentRef, FocusElementRef, FocusFrame,
    FocusFrameRef,
};
use interaction::default::public::mojom::input::focus_type::FocusType;

/// An identity view of the native Frame, retained across Document replacement.
/// Relations are always resolved from the original FrameTree links.
pub(super) struct NativeFocusFrame {
    domain: Weak<LocalFrameDomain>,
    token: u64,
}
impl NativeFocusFrame {
    fn Domain(&self) -> Option<Rc<LocalFrameDomain>> {
        self.domain.upgrade()
    }
    fn Events(&self) -> Option<FocusFrameRef> {
        self.Domain()?
            .FocusInteraction(self.token)?
            .FocusLocalFrameEvents()
    }
    fn ParentToken(&self) -> Option<u64> {
        self.Domain()?.Frame(self.token).map(|frame| frame.parent)
    }
}
impl FocusFrame for NativeFocusFrame {
    fn PageIdentity(&self) -> Option<Rc<()>> {
        self.IsAttached()
            .then(|| self.Domain().unwrap().focus_identity.clone())
    }
    fn IsLocalFrame(&self) -> bool {
        true
    }
    fn IsAttached(&self) -> bool {
        self.Domain().is_some_and(|domain| {
            if self.token == domain.root {
                domain
                    .page
                    .upgrade()
                    .is_some_and(|page| page.document.borrow().GetDocument().IsActive())
            } else {
                domain.Frame(self.token).is_some_and(|frame| {
                    frame.state.Lifecycle().GetState()
                        == dom::frame_lifecycle::FrameLifecycleState::Attached
                })
            }
        })
    }
    fn IsOutermostMainFrame(&self) -> bool {
        self.Domain()
            .is_some_and(|domain| self.token == domain.root)
    }
    fn HasView(&self) -> bool {
        self.IsAttached() && self.Events().is_some_and(|frame| frame.HasView())
    }
    fn Owner(&self) -> Option<FocusElementRef> {
        let domain = self.Domain()?;
        let frame = domain.Frame(self.token)?;
        domain
            .FocusInteraction(frame.parent)?
            .FocusElementOwner(frame.state.Owner()?.Id())
    }
    fn Document(&self) -> Option<FocusDocumentRef> {
        self.Events()?.Document()
    }
    fn LocalFrameRootDocument(&self) -> FocusDocumentRef {
        self.Top().Document().expect("attached local root Document")
    }
    fn Parent(&self) -> Option<FocusFrameRef> {
        self.Domain()?.FocusFrame(self.ParentToken()?)
    }
    fn Top(&self) -> FocusFrameRef {
        let domain = self.Domain().expect("live native FrameTree");
        domain.FocusFrame(domain.root).expect("native local root")
    }
    fn LocalFrameRoot(&self) -> FocusFrameRef {
        self.Top()
    }
    fn AdvanceFocusFromRemote(&self, _: FocusType, _: &FocusFrameRef) {
        unreachable!("native LocalFrame tree has no remote frame")
    }
    fn TraverseNext(&self) -> Option<FocusFrameRef> {
        let domain = self.Domain()?;
        if let Some(child) = domain.Children(self.token).first() {
            return domain.FocusFrame(child.state.Identity());
        }
        let mut token = self.token;
        loop {
            let frame = domain.Frame(token)?;
            let siblings = domain.Children(frame.parent);
            let offset = siblings
                .iter()
                .position(|child| child.state.Identity() == token)?;
            if let Some(next) = siblings.get(offset + 1) {
                return domain.FocusFrame(next.state.Identity());
            }
            token = frame.parent;
        }
    }
    fn IsDescendantOf(&self, frame: Option<&FocusFrameRef>) -> bool {
        let Some(frame) = frame else {
            return false;
        };
        let Some(domain) = self.Domain() else {
            return false;
        };
        let mut current = domain.FocusFrame(self.token);
        while let Some(candidate) = current {
            if Rc::ptr_eq(frame, &candidate) {
                return true;
            }
            current = candidate.Parent();
        }
        false
    }
    fn SetSelectionFrameIsFocused(&self, focused: bool) {
        if let Some(events) = self.Events() {
            events.SetSelectionFrameIsFocused(focused);
        }
    }
    fn DispatchWindowEvent(&self, event_type: &'static str, scope: &FocusDispatchScope<'_>) {
        if let Some(events) = self.Events() {
            events.DispatchWindowEvent(event_type, scope);
        }
    }
    fn DidFocus(&self) {
        if let Some(events) = self.Events() {
            events.DidFocus();
        }
    }
    fn StopAutoscroll(&self) {
        if let Some(events) = self.Events() {
            events.StopAutoscroll();
        }
    }
    fn StopAutoscrollWithScope(&self, scope: &FocusDispatchScope<'_>) {
        if let Some(events) = self.Events() {
            events.StopAutoscrollWithScope(scope);
        }
    }
    fn WillChangeInputMethodFocus(&self) {
        if let Some(events) = self.Events() {
            events.WillChangeInputMethodFocus();
        }
    }
    fn SelectionPageActivationChanged(&self) {
        if let Some(events) = self.Events() {
            events.SelectionPageActivationChanged();
        }
    }
}

pub(super) struct NativeFocusFrameTree {
    domain: Weak<LocalFrameDomain>,
    token: u64,
}
impl NativeFocusFrameTree {
    pub(super) fn Root(domain: &Rc<LocalFrameDomain>) -> Rc<Self> {
        Rc::new(Self {
            domain: Rc::downgrade(domain),
            token: domain.root,
        })
    }
    fn Child(frame: &Rc<LocalFrame>) -> Rc<Self> {
        Rc::new(Self {
            domain: frame.domain.clone(),
            token: frame.state.Identity(),
        })
    }
}
impl FocusNavigationFrameTree for NativeFocusFrameTree {
    fn PageIdentity(&self) -> Rc<()> {
        self.domain
            .upgrade()
            .expect("native FrameTree")
            .focus_identity
            .clone()
    }
    fn Frame(&self) -> Option<FocusFrameRef> {
        self.domain.upgrade()?.FocusFrame(self.token)
    }
    fn Root(&self) -> Option<FocusFrameRef> {
        let domain = self.domain.upgrade()?;
        domain.FocusFrame(domain.root)
    }
    fn Parent(&self) -> Option<FocusFrameRef> {
        self.Frame()?.Parent()
    }
    fn Owner(&self) -> Option<FocusElementRef> {
        self.Frame()?.Owner()
    }
    fn TraverseNext(&self) -> Option<FocusFrameRef> {
        self.Frame()?.TraverseNext()
    }
    fn ContentFrame(&self, owner_node: u64) -> Option<FocusFrameRef> {
        let domain = self.domain.upgrade()?;
        let document = domain.Document(self.token)?;
        let document = document.borrow();
        let document = document.GetDocument();
        let node = document.FindNodeById(owner_node)?;
        let child = document.ContentFrame(node)?;
        domain.FocusFrame(child.State().Identity())
    }
    fn Controller(&self) -> Option<Rc<FocusController>> {
        self.domain.upgrade()?.focus_controller.borrow().clone()
    }
    fn DocumentEvents(&self, identity: u128) -> Option<Rc<dyn FocusDocumentEventTarget>> {
        let domain = self.domain.upgrade()?;
        let tokens = std::iter::once(domain.root)
            .chain(domain.frames.borrow().keys().copied().collect::<Vec<_>>());
        for token in tokens {
            let Some(document) = domain.Document(token) else {
                continue;
            };
            if document.borrow().GetDocument().RootHandle().Identity() == identity {
                return domain.FocusInteraction(token)?.FocusDocumentEventTarget();
            }
        }
        None
    }
}
impl LocalFrameDomain {
    pub(super) fn FocusInteraction(&self, token: u64) -> Option<interaction::Interaction<'static>> {
        if token == self.root {
            return self
                .page
                .upgrade()?
                .popup_source_interaction
                .borrow()
                .clone();
        }
        let frame = self.Frame(token)?;
        let runtime = frame.runtime.borrow().clone()?;
        Some(runtime.frame_interaction.get()?.engine.clone())
    }
    pub(super) fn FocusFrame(self: &Rc<Self>, token: u64) -> Option<FocusFrameRef> {
        if token != self.root && self.Frame(token).is_none() {
            return None;
        }
        let frame = self
            .focus_frames
            .borrow_mut()
            .entry(token)
            .or_insert_with(|| {
                Rc::new(NativeFocusFrame {
                    domain: Rc::downgrade(self),
                    token,
                })
            })
            .clone();
        Some(frame)
    }
    pub(super) fn RootPointToFrame(
        &self,
        token: u64,
        point: foundation::gfx::Point,
    ) -> Option<foundation::gfx::Point> {
        let point = self.RootPointToFrameFloat(
            token,
            foundation::gfx::PointF::new(point.x() as f32, point.y() as f32),
        )?;
        Some(foundation::gfx::Point::new(
            point.x().round() as i32,
            point.y().round() as i32,
        ))
    }
    pub(super) fn EnsureFocusLayout(&self, token: u64) -> io::Result<()> {
        let page = self
            .page
            .upgrade()
            .ok_or_else(|| io::Error::other("detached local root"))?;
        if token == self.root {
            page.EnsureLayoutOutput();
            return Ok(());
        }
        let frame = self
            .Frame(token)
            .ok_or_else(|| io::Error::other("detached LocalFrame"))?;
        self.EnsureFocusLayout(frame.parent)?;
        let runtime = frame
            .runtime
            .borrow()
            .clone()
            .ok_or_else(|| io::Error::other("detached child Document"))?;
        let size = self
            .WithParentLayout(&frame, |layout| {
                layout
                    .Output()
                    .and_then(|output| output.Node(frame.state.Owner()?.Id()))
                    .and_then(|node| node.EmbeddedChildFrameSize())
            })
            .ok_or_else(|| io::Error::other("child frame owner has no LayoutObject"))?;
        if let Some(interaction) = self.FocusInteraction(token) {
            interaction::adapter::page_form_validation::RefreshControlValidity(
                &runtime.document.Handle(),
                &interaction.FormControlState(),
            )
            .map_err(|error| {
                io::Error::other(format!(
                    "child constraint-validation service required: {error:?}"
                ))
            })?;
        }
        runtime.UpdateFocusLayout(size, &page)
    }
}

pub(super) struct ChildFocusServices {
    frame: Weak<LocalFrame>,
    runtime: Weak<LocalFrameRuntime>,
}
impl ChildFocusServices {
    pub(super) fn new(frame: &Rc<LocalFrame>, runtime: &Rc<LocalFrameRuntime>) -> Rc<Self> {
        Rc::new(Self {
            frame: Rc::downgrade(frame),
            runtime: Rc::downgrade(runtime),
        })
    }
    fn Owners(&self) -> Option<(Rc<LocalFrame>, Rc<LocalFrameRuntime>)> {
        let frame = self.frame.upgrade()?;
        let runtime = self.runtime.upgrade()?;
        frame.IsCurrentRuntime(&runtime).then_some((frame, runtime))
    }
    fn Update(&self) {
        if let Some((frame, runtime)) = self.Owners() {
            frame.EnsureFocusLayout(&runtime);
        }
    }
    fn Snap(&self, update: FocusWithinSnapUpdate) {
        let Some((_, runtime)) = self.Owners() else {
            return;
        };
        let facts = {
            let owner = runtime.document.borrow();
            let document = owner.GetDocument();
            update
                .editing_fact_nodes
                .into_iter()
                .filter_map(|id| {
                    document
                        .FindNodeById(id)
                        .map(|node| (id, document.EditingNodeFacts(node)))
                })
                .collect::<Vec<_>>()
        };
        let mut layout = runtime.layout.borrow_mut();
        for (id, facts) in facts {
            layout.UpdateEditingNodeFactsForFocus(id, facts);
        }
        for id in update.snap_container_candidates {
            if let Some(mut source) = layout.NativeScrollSource(id) {
                source.UpdateFocusDataForSnapAreas();
            }
        }
    }
}
impl PageElementFocusLayout for ChildFocusServices {
    fn HasLayoutObject(&self, node: u64) -> bool {
        self.Owners().is_some_and(|(_, runtime)| {
            runtime
                .layout
                .borrow()
                .EventRegistryLayout(Some(node))
                .is_some()
        })
    }
    fn IsUserScrollable(&self, node: u64) -> bool {
        self.Facts(node)
            .is_some_and(|facts| facts.is_user_scrollable)
    }
    fn IsCanvasLayoutObject(&self, node: u64) -> bool {
        self.Facts(node).is_some_and(|facts| facts.is_canvas)
    }
}
impl FocusNavigationLayout for ChildFocusServices {
    fn GridFocusgroup(
        &self,
        root: u64,
        query: interaction::adapter::page_focus_navigation::FocusgroupGridQuery,
    ) -> Option<interaction::adapter::page_focus_navigation::FocusgroupGridResult> {
        let (_, runtime) = self.Owners()?;
        let layout = runtime.layout.borrow();
        let document = runtime.document.borrow();
        interaction::adapter::page_focus_navigation::QueryGridFocusgroup(
            document.GetDocument(),
            layout.Output()?,
            root,
            query,
        )
    }
    fn UpdateStyleAndLayoutForFocus(&self) {
        self.Update();
    }
    fn Facts(
        &self,
        node: u64,
    ) -> Option<layoutng_assembly::layout_engine::LayoutFocusNavigationFacts> {
        let (_, runtime) = self.Owners()?;
        let layout = runtime.layout.borrow();
        layout
            .Output()?
            .Node(node)
            .map(|node| node.FocusNavigationFacts())
    }
    fn ReadingFlowNodes(&self, node: u64) -> Vec<u64> {
        let Some((_, runtime)) = self.Owners() else {
            return Vec::new();
        };
        let layout = runtime.layout.borrow();
        layout
            .Output()
            .and_then(|output| output.Node(node))
            .map_or_else(Vec::new, |node| node.ReadingFlowNodes())
    }
}
impl FocusNavigationFramePort for ChildFocusServices {
    fn FrameTree(&self) -> Option<Rc<dyn FocusNavigationFrameTree>> {
        Some(NativeFocusFrameTree::Child(&self.frame.upgrade()?))
    }
    fn HasView(&self) -> bool {
        self.Owners()
            .is_some_and(|(_, runtime)| runtime.document.borrow().GetDocument().IsActive())
    }
    fn IsPagePaused(&self) -> bool {
        false
    }
    fn CanTakeFocus(&self, _: FocusType) -> bool {
        false
    }
    fn TakeFocus(&self, _: FocusType) {
        debug_assert!(false, "TakeFocus requires CanTakeFocus");
    }
    fn FrameEffect(&self, _: FocusNavigationFrameEffect) {
        if let Some((frame, _)) = self.Owners() {
            frame.MarkNavigationDirty();
        }
    }
    fn StopAutoscroll(&self, _: &FocusDispatchScope<'_>) { /* Child has no native autoscroll producer. */
    }
    fn HandleInterestForHoverOrFocus(&self, _: u64, _: bool) { /* InterestTargetController is disabled. */
    }
    fn UpdateFocusDataForSnapAreas(&self, update: FocusWithinSnapUpdate) {
        self.Snap(update);
    }
}
impl DocumentFocusLifecycle for ChildFocusServices {
    fn PageFocusTarget(
        &self,
        node: u64,
    ) -> Option<(Rc<FocusController>, FocusFrameRef, FocusElementRef)> {
        let (frame, _) = self.Owners()?;
        let domain = frame.domain.upgrade()?;
        let controller = domain.focus_controller.borrow().clone()?;
        Some((
            controller,
            domain.FocusFrame(frame.state.Identity())?,
            domain
                .FocusInteraction(frame.state.Identity())?
                .FocusElementOwner(node)?,
        ))
    }

    fn StopClearFocusedElementTimer(&self) { /* No deferred clear-focus timer is scheduled. */
    }
    fn HasFrame(&self) -> bool {
        FocusNavigationFramePort::HasView(self)
    }
    fn IsPageFocused(&self) -> bool {
        self.frame
            .upgrade()
            .and_then(|frame| frame.domain.upgrade())
            .and_then(|domain| domain.page.upgrade())
            .is_some_and(|page| page.page_focused.get())
    }
    fn UpdateStyleAndLayoutTreeForElement(&self, _: u64) {
        self.Update();
    }
    fn UpdateStyleAndLayoutForNode(&self, _: u64) {
        self.Update();
    }
    fn UpdateStyleAndLayoutTree(&self) {
        self.Update();
    }
    fn FrameSelectionHost(
        &self,
    ) -> Rc<dyn interaction::adapter::page_frame_selection::PageFrameSelectionHost> {
        super::frame_selection_host::ResidentFrameSelectionHost::Child(
            &self.frame.upgrade().expect("retained Frame"),
            &self.runtime.upgrade().expect("retained Document"),
        )
    }
    fn IsLayoutEmbeddedContent(&self, node: u64) -> bool {
        self.Update();
        self.Facts(node)
            .is_some_and(|facts| facts.is_embedded_content)
    }
    fn ScrollBoundingBoxForFocusIntoView(&self, node: u64) {
        let Some((frame, runtime)) = self.Owners() else {
            return;
        };
        let result = runtime
            .layout
            .borrow_mut()
            .NativeScrollSource(node)
            .map(|mut source| source.ScrollFocusIntoView(true));
        if let Some(result) = result {
            frame.ApplyNativeScrollResult(&runtime, result);
        }
    }
    fn UpdateFocusDataForSnapAreas(&self, update: FocusWithinSnapUpdate) {
        self.Snap(update);
    }
    fn NotifyFocusedElementChanged(&self, old: Option<u64>, new: Option<u64>, _: FocusType) {
        let Some((frame, runtime)) = self.Owners() else {
            return;
        };
        if let Some(navigation) = runtime
            .document
            .borrow()
            .GetDocument()
            .WindowNavigationApiOwner()
        {
            navigation.FocusedElementChanged();
        }
        let _ = old;
        if let Some(page) = frame
            .domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
        {
            page.ValidationMessageFocusChanged(new);
        }
        frame.MarkNavigationDirty();
    }
}
impl LocalFrame {
    pub(super) fn IsCurrentRuntime(&self, runtime: &Rc<LocalFrameRuntime>) -> bool {
        self.state.Lifecycle().GetState() == dom::frame_lifecycle::FrameLifecycleState::Attached
            && self
                .runtime
                .borrow()
                .as_ref()
                .is_some_and(|current| Rc::ptr_eq(current, runtime))
    }
    pub(super) fn EnsureFocusLayout(&self, runtime: &Rc<LocalFrameRuntime>) {
        if !self.IsCurrentRuntime(runtime) {
            return;
        }
        let Some(domain) = self.domain.upgrade() else {
            return;
        };
        if let Err(error) = domain.EnsureFocusLayout(self.state.Identity()) {
            if let Some(page) = domain.page.upgrade() {
                page.client.borrow_mut().DidFail(&error.to_string());
            }
        }
    }
    pub(super) fn ApplyNativeScrollResult(
        &self,
        runtime: &Rc<LocalFrameRuntime>,
        result: layoutng_assembly::native_scroll_source::NativeScrollResult,
    ) {
        if !self.IsCurrentRuntime(runtime) {
            return;
        }
        for update in result.offsets {
            runtime
                .document
                .ApplyScrollOffset(update.node_id, update.offset);
        }
        for request in result.programmatic_requests {
            self.SetProgrammaticScrollOffset(
                runtime,
                request.node_id,
                Offset {
                    x: request.offset.x() as f64,
                    y: request.offset.y() as f64,
                },
                if request.behavior
                    == layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior::kSmooth
                {
                    ViewportScrollBehavior::Smooth
                } else {
                    ViewportScrollBehavior::Instant
                },
                request.source_type,
            );
        }
        if result.did_scroll {
            runtime.document.borrow_mut().InvalidatePaint();
            self.MarkNavigationDirty();
        }
    }
    pub(super) fn DetachFocus(&self) {
        let Some(domain) = self.domain.upgrade() else {
            return;
        };
        let controller = domain.focus_controller.borrow().clone();
        if let (Some(controller), Some(frame)) =
            (controller, domain.FocusFrame(self.state.Identity()))
        {
            controller.FrameDetached(&frame);
        }
    }
    pub(super) fn QueueSelectionchange(&self, runtime: &Rc<LocalFrameRuntime>, text_control: bool) {
        let Some(window) = self.navigation.borrow().window.clone() else {
            return;
        };
        if !Rc::ptr_eq(&window.document, &runtime.document) {
            return;
        }
        let target = {
            let mut owner = runtime.document.borrow_mut();
            let document = owner.GetDocumentMut();
            let target = if text_control {
                let selection =
                    interaction::adapter::page_frame_selection::CurrentDOMSelectionForFocus(
                        document,
                        &runtime.editing.selections,
                    );
                interaction::adapter::page_focus_controller::EnclosingTextControlForSelectionForFocus(document, selection)
            } else {
                Some(document.Node(document.Root()).Id())
            };
            target.filter(|&target| document.MarkSelectionchangeEventScheduled(target))
        };
        if let Some(target) = target {
            window.QueueSelectionchangeEvent(target);
        }
    }
    pub(super) fn SelectOwnerInParentIfFullySelected(&self, runtime: &Rc<LocalFrameRuntime>) {
        if !self.IsCurrentRuntime(runtime) {
            return;
        }
        let Some(domain) = self.domain.upgrade() else {
            return;
        };
        interaction::default::core::editing::frame_selection::SelectFrameElementInParentIfFullySelected(
            &NativeFrameOwnerSelection { frame: self, runtime, domain },
        );
    }
}

/// Source FrameSelection's parent-selection leaves over original native
/// owners. Only the selected owner's identity survives focus listener reentry.
struct NativeFrameOwnerSelection<'a> {
    frame: &'a LocalFrame,
    runtime: &'a Rc<LocalFrameRuntime>,
    domain: Rc<LocalFrameDomain>,
}
struct NativeParentSelectionOwner {
    token: u64,
    document: Rc<DocumentEngine>,
    document_identity: dom::persistent_document::DOMOwnerHandle,
    element: dom::persistent_document::DOMOwnerHandle,
    parent: dom::persistent_document::DOMOwnerHandle,
}
impl interaction::default::core::editing::frame_selection::FrameOwnerSelectionSource
    for NativeFrameOwnerSelection<'_>
{
    type Frame = u64;
    type Owner = NativeParentSelectionOwner;
    fn Parent(&self) -> Option<u64> {
        self.domain
            .FocusFrame(self.frame.parent)
            .map(|_| self.frame.parent)
    }
    fn HasPage(&self) -> bool {
        self.domain.page.upgrade().is_some()
    }
    fn SelectionIsRange(&self) -> bool {
        let owner = self.runtime.document.borrow();
        interaction::adapter::page_frame_selection::CurrentDOMSelectionForFocus(
            owner.GetDocument(),
            &self.runtime.editing.selections,
        )
        .is_some_and(|selection| selection.anchor != selection.focus)
    }
    fn UpdateStyleAndLayoutForSelection(&self) {
        self.frame.EnsureFocusLayout(self.runtime);
    }
    fn VisibleSelectionCoversDocument(&self) -> bool {
        if !self.frame.IsCurrentRuntime(self.runtime) {
            return false;
        }
        let layout = self.runtime.layout.borrow();
        let Some(output) = layout.Output() else {
            return false;
        };
        let owner = self.runtime.document.borrow();
        let document = owner.GetDocument();
        let selection = interaction::adapter::page_frame_selection::CurrentDOMSelectionForFocus(
            document,
            &self.runtime.editing.selections,
        );
        interaction::adapter::page_focus_controller::VisibleDOMSelectionCoversDocumentForFocus(
            document, output, selection,
        )
    }
    fn LocalParentOwner(&self, parent: &u64) -> Option<Self::Owner> {
        if !self.domain.FocusFrame(*parent)?.IsLocalFrame() {
            return None;
        }
        let document = self.domain.Document(*parent)?;
        let element = self.frame.state.Owner()?;
        let (document_identity, parent) = {
            let owner = document.borrow();
            let document = owner.GetDocument();
            let element_index = document.FindNodeById(element.Id())?;
            if document.NodeHandle(element_index) != element {
                return None;
            }
            (
                document.RootHandle(),
                document.NodeHandle(document.Node(element_index).Parent()?),
            )
        };
        Some(NativeParentSelectionOwner {
            token: self.frame.parent,
            document,
            document_identity,
            element,
            parent,
        })
    }
    fn UpdateOwnerDocumentStyleAndLayout(&self, owner: &Self::Owner) -> bool {
        match self.domain.EnsureFocusLayout(owner.token) {
            Ok(()) => true,
            Err(error) => {
                if let Some(page) = self.domain.page.upgrade() {
                    page.client.borrow_mut().DidFail(&error.to_string());
                }
                false
            }
        }
    }
    fn OwnerParentIsEditable(&self, owner: &Self::Owner) -> bool {
        let document = owner.document.borrow();
        let document = document.GetDocument();
        document
            .FindNodeById(owner.parent.Id())
            .is_some_and(|parent| {
                document.NodeHandle(parent) == owner.parent && document.IsEditableForEditing(parent)
            })
    }
    fn FocusParent(&self, parent: &u64) -> bool {
        let controller = self.domain.focus_controller.borrow().clone();
        let Some(controller) = controller else {
            return false;
        };
        let Some(parent) = self.domain.FocusFrame(*parent) else {
            return false;
        };
        let Some(events) = self
            .domain
            .FocusInteraction(self.frame.state.Identity())
            .and_then(|engine| engine.FocusDocumentEventTarget())
            .map(|owner| owner.FocusEvents())
        else {
            return false;
        };
        controller.SetFocusedFrameWithScope(
            Some(parent),
            true,
            &FocusDispatchScope {
                focus_events: Some(events.as_ref()),
                ..Default::default()
            },
        );
        true
    }
    fn OwnerIsConnectedToParentDocument(&self, owner: &Self::Owner, parent: &u64) -> bool {
        let Some(current) = self.domain.Document(*parent) else {
            return false;
        };
        if !Rc::ptr_eq(&current, &owner.document) {
            return false;
        }
        let current = current.borrow();
        let document = current.GetDocument();
        document.RootHandle() == owner.document_identity
            && document
                .FindNodeById(owner.element.Id())
                .is_some_and(|element| {
                    document.NodeHandle(element) == owner.element
                        && document.IsConnectedToDocument(element)
                        && document.NodeDocumentHandle(element) == document.RootHandle()
                })
    }
    fn SelectOwnerInParent(&self, owner: &Self::Owner, parent: &u64) {
        if let Some(selection) = self
            .domain
            .FocusInteraction(*parent)
            .and_then(|engine| engine.FrameSelectionOwner())
        {
            selection.SelectFrameOwnerElement(owner.element);
        }
    }
}

impl LocalFrameRuntime {
    pub(super) fn UpdateFocusLayout(
        &self,
        size: foundation::gfx::Size,
        page: &PageState,
    ) -> io::Result<()> {
        let pending =
            std::mem::take(&mut *self.pending_text_overflow_layout_invalidations.borrow_mut());
        let invalidated = {
            let mut layout = self.layout.borrow_mut();
            pending.into_iter().fold(false, |changed, node| {
                layout.InvalidateTextOverflowLayoutForNode(node) || changed
            })
        };
        let lifecycle = self.document.borrow().GetDocument().Lifecycle().clone();
        let changed = self.view.size.replace(size) != size;
        if changed {
            lifecycle.ScheduleVisualUpdate();
        }
        if invalidated {
            lifecycle.ScheduleLayout();
        }
        let needs_update = invalidated
            || changed
            || !self.document.borrow().GetStyleImpact().IsEmpty()
            || self.layout.borrow().Output().is_none();
        if needs_update {
            let mut constraints = self.constraints.borrow_mut();
            constraints.available_size = layoutng_assembly::internal::layout_input::Size {
                width: size.width() as f64,
                height: size.height() as f64,
            };
            constraints.viewport = Some(
                layoutng_assembly::internal::layout_input::ViewportGeometry {
                    size: IntSize {
                        width: size.width(),
                        height: size.height(),
                    },
                    ..Default::default()
                },
            );
            let mut document = self.document.borrow_mut();
            lifecycle.EnsureStateAtMost(dom::document_lifecycle::LifecycleState::StyleClean);
            lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::InStyleRecalc);
            if !crate::style_services::ResolveLayoutStylesWithEngineAndForcedColorsUsingFontMetrics(
                &mut self.style.borrow_mut(),
                &mut document,
                &constraints,
                page.preferred_color_scheme.get(),
                page.forced_colors.get(),
                page.style_font_metrics.clone(),
            ) {
                return Err(io::Error::other("child document font preparation required"));
            }
            lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::StyleClean);
            // SelectType's text/style lifecycle runs after the canonical
            // Style/DOM loans and before Layout consumes those same owners.
            drop(document);
            dom::attribute_lifecycle::DrainAttributeLifecycle(&self.document.Handle());
            let mut document = self.document.borrow_mut();
            lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::InPerformLayout);
            crate::persistent_layout::LayoutDocumentWithEngine(
                &mut self.layout.borrow_mut(),
                &mut document,
                &self.interaction_state.borrow(),
                &constraints,
            );
            lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::AfterPerformLayout);
            lifecycle.AdvanceTo(dom::document_lifecycle::LifecycleState::LayoutClean);
            self.view.DidLayout();
        }
        if needs_update {
            self.view
                .revision
                .set(self.view.revision.get().wrapping_add(1));
        }
        Ok(())
    }
}

/// LocalFrame's Document owns input and form state before a Window realm is
/// materialized. Script bindings project these same owners; the realm never
/// creates a second EventHandler, pointer, mouse or selection state.
pub(super) struct FrameInteraction {
    pub(super) engine: interaction::Interaction<'static>,
    pub(super) pending_submissions:
        Rc<RefCell<std::collections::VecDeque<interaction::PreparedFormSubmission>>>,
    pub(super) controller: Rc<interaction::adapter::page_form_controller::PageFormController>,
}
impl LocalFrameRuntime {
    pub(super) fn InstallFrameInteraction(self: &Rc<Self>, frame: &Rc<LocalFrame>) {
        let effect_frame = Rc::downgrade(frame);
        let effect_runtime = Rc::downgrade(self);
        let pending_submissions = Rc::new(RefCell::new(std::collections::VecDeque::new()));
        let effect_pending = pending_submissions.clone();
        let engine = interaction::Interaction::WithSelectionState(
            Rc::new(move |output| {
                let (Some(frame), Some(runtime)) =
                    (effect_frame.upgrade(), effect_runtime.upgrade())
                else {
                    return;
                };
                if !frame.IsCurrentRuntime(&runtime) {
                    return;
                }
                match output {
                    interaction::InteractionOutput::DocumentMutation(mutation) => {
                        runtime
                            .document
                            .ApplyDOMMutation(&mutation)
                            .unwrap_or_else(|error| std::panic::panic_any(error));
                        frame.MarkNavigationDirty();
                    }
                    interaction::InteractionOutput::StateMutation(mutation) => {
                        *runtime.interaction_state.borrow_mut() = mutation.state;
                        if let Some(window) = frame
                            .navigation
                            .borrow()
                            .window
                            .clone()
                            .filter(|window| Rc::ptr_eq(&window.document, &runtime.document))
                        {
                            if let Ok(mut bindings) = window.web_api.DOMBindings().try_borrow_mut()
                            {
                                bindings.SetFocusedNode(mutation.state.focused_node_id);
                            }
                        }
                        runtime
                            .document
                            .borrow_mut()
                            .GetDocumentMut()
                            .StyleStateMut()
                            .impact
                            .Merge(dom::style_state::StyleUpdateImpact {
                                paint: true,
                                ..Default::default()
                            });
                        frame.MarkNavigationDirty();
                    }
                    interaction::InteractionOutput::Effect(effect) => {
                        frame.ApplyFrameInteractionEffect(&runtime, &effect_pending, effect);
                    }
                }
            }),
            None,
            self.editing.selections.clone(),
            frame
                .domain
                .upgrade()
                .and_then(|domain| domain.page.upgrade())
                .expect("LocalFrame input clock")
                .InteractionEventClock(),
        );
        engine.BindDocumentInputLifecycle(&self.document.Handle());
        self.BindNativeInput(frame, &engine);
        if let Some(domain) = frame.domain.upgrade() {
            engine.SetFormNavigationPreparation(Some(Rc::new(
                super::local_frame_navigation::FormNavigationPreparation::new(
                    &domain,
                    frame.state.Identity(),
                ),
            )));
        }
        engine
            .FormControlState()
            .SetValidationPatternEngine(webapi::form_validation::CreateValidationPatternEngine());
        engine.FormControlState().SetValidationMessageClient(
            super::validation_message::PageValidationMessageClient::ForChild(frame, self),
        );
        let services = ChildFocusServices::new(frame, self);
        let focus_frame = Rc::downgrade(frame);
        assert!(engine.BindFocusNavigation(
            services.clone(),
            services.clone(),
            services,
            interaction::adapter::page_focus_navigation::FocusNavigationPolicy {
                tabs_to_links: true,
                spatial_navigation_enabled: false
            },
            Rc::new(move |_| {
                if let Some(frame) = focus_frame.upgrade() {
                    frame.MarkNavigationDirty();
                }
            })
        ));
        let controller = interaction::adapter::page_form_controller::PageFormController::Install(
            self.document.Handle(),
            engine.clone(),
        );
        let task_frame = Rc::downgrade(frame);
        let task_runtime = Rc::downgrade(self);
        controller.SetTaskRunner(Rc::new(move |task| {
            let Some(frame) = task_frame.upgrade() else {
                return;
            };
            if let Some(domain) = frame.domain.upgrade() {
                domain
                    .form_state_tasks
                    .borrow_mut()
                    .push_back((task_runtime.clone(), task));
            }
        }));
        if let Some(page) = frame
            .domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
        {
            if let Some(reader) = page.native_file_reader.borrow().clone() {
                controller.SetFileReader(reader);
            }
            if let Some(ring) = page.kill_ring.borrow().clone() {
                engine.SetKillRing(ring);
            }
        }
        assert!(self
            .frame_interaction
            .set(FrameInteraction {
                engine,
                pending_submissions,
                controller
            })
            .is_ok());
    }
}
impl LocalFrameDomain {
    pub(super) fn RunFormStateTask(&self) {
        let task = self.form_state_tasks.borrow_mut().pop_front();
        if let Some((runtime, task)) = task {
            let Some(runtime) = runtime
                .upgrade()
                .filter(|runtime| runtime.document.borrow().GetDocument().IsActive())
            else {
                return;
            };
            task();
            let frame = self
                .frames
                .borrow()
                .values()
                .find(|frame| frame.IsCurrentRuntime(&runtime))
                .cloned();
            if let Some(frame) = frame {
                if let Some(engine) = self.FocusInteraction(frame.state.Identity()) {
                    runtime.CompleteNoScriptFormSubmissions(&frame, &engine);
                }
            }
        }
    }
}

impl LocalFrameDomain {
    pub(super) fn DocumentFocusState(
        &self,
        token: u64,
        identity: u128,
    ) -> Option<webapi::dom_bindings::DocumentFocusState> {
        let page = self.page.upgrade()?;
        let document = self.Document(token)?;
        if document.borrow().GetDocument().RootHandle().Identity() != identity {
            return None;
        }
        let engine = self.FocusInteraction(token)?;
        Some(webapi::dom_bindings::DocumentFocusState {
            active_element: engine.DocumentActiveElementNode(),
            has_focus: engine.DocumentHasFocus(page.page_focused.get(), page.page_focused.get()),
        })
    }
}
