//! SelectType's host leaves over the same resident Document/Layout owners.
//! Algorithms stay in Interaction/default; Page supplies lifecycle, task runner
//! and ChromeClient/optional AXObjectCache notifications only.
use super::local_frame_scroll::InteractionScrollOwner;
use super::*;
use interaction::adapter::page_select_control::{SelectAccessibilityClient, SelectControlHost};

pub(super) struct ResidentSelectControlHost(InteractionScrollOwner);
impl ResidentSelectControlHost {
    pub(super) fn Root(page: &Rc<PageState>, engine: &interaction::Interaction<'static>) {
        Self::Install(
            InteractionScrollOwner::Root(Rc::downgrade(page)),
            engine,
            &page.document.Handle(),
        );
    }
    pub(super) fn Child(
        frame: &Rc<local_frame::LocalFrame>,
        runtime: &Rc<local_frame::LocalFrameRuntime>,
        engine: &interaction::Interaction<'static>,
    ) {
        Self::Install(
            InteractionScrollOwner::Child {
                frame: Rc::downgrade(frame),
                runtime: Rc::downgrade(runtime),
            },
            engine,
            &runtime.document.Handle(),
        );
    }
    fn Install(
        owner: InteractionScrollOwner,
        engine: &interaction::Interaction<'static>,
        document: &interaction::InteractionDocument,
    ) {
        let host = Rc::new(Self(owner));
        engine
            .FormControlState()
            .SetSelectControlHost(document, host);
    }
    fn Document(&self) -> Option<Rc<DocumentEngine>> {
        match &self.0 {
            InteractionScrollOwner::Root(page) => Some(page.upgrade()?.document.clone()),
            InteractionScrollOwner::Child { .. } => Some(self.0.ChildOwners()?.1.document.clone()),
        }
    }
    fn Engine(&self) -> Option<interaction::Interaction<'static>> {
        let page = self.0.Page()?;
        let domain = page.local_frames.get()?;
        domain.FocusInteraction(match &self.0 {
            InteractionScrollOwner::Root(_) => domain.root,
            InteractionScrollOwner::Child { .. } => self.0.ChildOwners()?.0.state.Identity(),
        })
    }
    fn EnsureLayout(&self) {
        match &self.0 {
            InteractionScrollOwner::Root(page) => {
                if let Some(page) = page.upgrade() {
                    page.EnsureLayoutOutput();
                }
            }
            InteractionScrollOwner::Child { .. } => {
                if let Some((frame, runtime)) = self.0.ChildOwners() {
                    frame.EnsureFocusLayout(&runtime);
                }
            }
        }
    }
    fn ApplyScroll(&self, result: layoutng_assembly::native_scroll_source::NativeScrollResult) {
        match &self.0 {
            InteractionScrollOwner::Root(page) => {
                if let Some(page) = page.upgrade() {
                    page.ApplyNativeScrollResult(result);
                }
            }
            InteractionScrollOwner::Child { .. } => {
                if let Some((frame, runtime)) = self.0.ChildOwners() {
                    frame.ApplyNativeScrollResult(&runtime, result);
                }
            }
        }
    }
}
impl dom::attribute_lifecycle::SelectControlLifecycle for ResidentSelectControlHost {
    fn TextStyleAndContentChanged(&self, select: u64) {
        let (Some(engine), Some(document)) = (self.Engine(), self.Document()) else {
            return;
        };
        engine.RefreshSelectTextStyleAndContent(&document.Handle(), select);
    }
    fn FinishedParsingChildren(&self, select: u64) {
        let (Some(engine), Some(document)) = (self.Engine(), self.Document()) else {
            return;
        };
        engine.SelectFinishedParsingChildren(&document.Handle(), select);
    }
}
impl SelectControlHost for ResidentSelectControlHost {
    fn PostUserInteractionTask(&self, task: Box<dyn FnOnce()>) {
        let Some(page) = self.0.Page() else {
            return;
        };
        let document = page.document.borrow().GetDocument().RootHandle();
        page.popup_tasks.Post(
            document,
            0.0,
            super::popup_tasks::PageInteractionTask::FormRestore(Rc::new(RefCell::new(Some(task)))),
        );
    }
    fn ScrollOptionIntoView(&self, select: u64, option: u64) {
        self.EnsureLayout();
        let Some(document) = self.Document() else {
            return;
        };
        let container = {
            let owner = document.borrow();
            let document = owner.GetDocument();
            let Some(select) = document.FindNodeById(select) else {
                return;
            };
            let Some(option) = document.FindNodeById(option) else {
                return;
            };
            if !document.IsActive()
                || !document.IsConnectedToDocument(select)
                || document.OptionOwnerSelect(option) != Some(select)
            {
                return;
            }
            document
                .Node(document.SelectListboxElement(select).unwrap_or(select))
                .Id()
        };
        if let Some(result) = self
            .0
            .WithLayout(|layout| layout.ScrollSelectOptionIntoView(container, option))
        {
            self.ApplyScroll(result);
        }
    }
    fn HasLayoutObject(&self, node: u64) -> bool {
        self.0
            .WithLayout(|layout| layout.EventRegistryLayout(Some(node)).is_some())
            .unwrap_or(false)
    }
    fn SetInnerModifiedStyleOutsideStyleRecalc(
        &self,
        inner: u64,
        style: foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle>,
    ) {
        self.0
            .WithLayout(|layout| layout.SetInputModifiedStyleOutsideStyleRecalc(inner, style));
        if let Some(page) = self.0.Page() {
            page.dirty.set(true);
            page.scroll_only.set(false);
        }
    }
    fn HasLayoutBox(&self, node: u64) -> bool {
        self.0
            .WithLayout(|layout| {
                layout
                    .Output()
                    .and_then(|output| output.Node(node))
                    .is_some_and(|node| node.HitTestFacts().box_)
            })
            .unwrap_or(false)
    }
    fn AccessibilityClient(&self) -> Option<Rc<dyn SelectAccessibilityClient>> {
        let page = self.0.Page()?;
        let client = page.client.borrow().SelectAccessibilityClient();
        client
    }
    fn NativePopupClient(
        &self,
    ) -> Option<Rc<dyn interaction::adapter::page_select_control::NativeSelectPopupClient>> {
        let page = self.0.Page()?;
        let client = page.client.borrow().NativeSelectPopupClient();
        client
    }
    fn DidChangeSelectionInSelectControl(&self, select: u64) {
        let (Some(page), Some(document)) = (self.0.Page(), self.Document()) else {
            return;
        };
        let identity = document.borrow().GetDocument().RootHandle().Identity();
        page.client
            .borrow_mut()
            .DidChangeSelectionInSelectControl(identity, select);
    }
    fn FormToolClient(
        &self,
    ) -> Option<Rc<dyn interaction::adapter::page_select_control::FormToolClient>> {
        let page = self.0.Page()?;
        let client = page.client.borrow().FormToolClient();
        client
    }
    fn FormElementReset(&self, form: u64) {
        let (Some(page), Some(document)) = (self.0.Page(), self.Document()) else {
            return;
        };
        let identity = document.borrow().GetDocument().RootHandle().Identity();
        page.client.borrow_mut().DidResetFormElement(identity, form);
    }
    fn NotifyFormStateChanged(&self, _: u64) {
        if let Some(page) = self.0.Page() {
            page.SaveHistoryState();
        }
    }
    fn MultipleOptionsSelectedText(&self, count: usize) -> Rc<[u16]> {
        let page = self.0.Page().expect("live select locale owner");
        let text = page.client.borrow().MultipleOptionsSelectedText(count);
        text
    }
}
