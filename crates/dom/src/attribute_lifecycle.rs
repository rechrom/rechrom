//! Synchronous attribute lifecycle across the embedding's RefCell boundary.
//! Attribute storage records typed facts while borrowed. The transaction owner
//! drains them immediately after releasing that borrow, before script reentry.
use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug)]
pub enum FormStateParsingKind {
    Control,
    Form,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeModificationReason {
    Directly,
    ByParser,
    ByCloning,
    ByMoveToNewDocument,
    BySynchronizationOfLazyAttribute,
}
#[derive(Clone, Debug)]
pub struct AttributeChange {
    pub node: u64,
    pub name: String,
    pub namespace: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    /// Original AtomicString values retained at the mutation commit point.
    pub old_value_utf16: Option<Rc<[u16]>>,
    pub new_value_utf16: Option<Rc<[u16]>>,
    pub reason: AttributeModificationReason,
    /// Source ParseAttribute's connection predicate at the setter call point,
    /// before an exclusively borrowed parser continues inserting this node.
    pub was_connected: bool,
}

/// HTMLElement/Dialog source lifecycle facts, drained by the synchronous
/// Interaction transaction after releasing the document storage borrow.
#[derive(Clone, Debug)]
pub enum PopupLifecycleChange {
    Attribute(AttributeChange),
    Removed(u64),
    DialogInserted(u64, bool),
}
pub trait AttributeLifecycle {
    fn IntersectionObserverConnected(&self, node: u64);
    fn StatefulFormControlConnected(&self, node: u64);
    fn FormStateFinishedParsing(&self, node: u64, kind: FormStateParsingKind, was_finished: bool);
    fn FormStateDocumentFinishedParsing(&self);
    fn AttributeChanged(&self, change: AttributeChange);
    /// Synchronous source removal, before the canonical parent link changes.
    fn WillRemove(&self, node: u64);
    fn SetPopoverOpen(
        &self,
        node: u64,
        open: bool,
        source: Option<u64>,
    ) -> Result<(), crate::dom_mutation::DOMMutationError>;
    fn SetDialogModal(
        &self,
        node: u64,
        open: bool,
        return_value: Option<&[u16]>,
    ) -> Result<(), crate::dom_mutation::DOMMutationError>;
    fn InputValueAssigned(&self, node: u64, value: Rc<[u16]>);
    fn InputInitialized(&self, node: u64);
    fn MediaInitialized(&self, node: u64);
    fn MediaFinishedParsing(&self, node: u64);
    fn MediaSourceChanged(&self, media: u64, source: u64, added: bool);
    fn MediaInserted(&self, node: u64);
    fn AnchorInserted(&self, node: u64);
    fn AnchorRemoved(&self, node: u64);
    fn MediaRemoved(&self, node: u64);
    fn MediaAttached(&self, node: u64);
    fn InputAttached(&self, node: u64, layout_image: Option<bool>);
    fn InputFilesAssigned(&self, node: u64, files: Rc<fileapi::FileList>);
    fn InputFilesCleared(&self, node: u64);
    fn DetailsInsertedInto(&self, node: u64);
    fn DialogInsertedInto(&self, node: u64, insertion_point_connected: bool);
    fn SelectTextStyleAndContentChanged(&self, node: u64);
    fn SelectFinishedParsingChildren(&self, node: u64);
    fn SelectSlottedButtonChanged(&self, node: u64);
    fn OptionMutationObserverChanged(&self, node: u64, in_style_recalc: bool);
    fn SelectDidRecalcStyle(&self, node: u64);
    fn SelectMutationObserverChanged(&self, node: u64);
    fn SelectPickerDidRecalcStyle(&self, node: u64);
    fn ElementLayoutDetached(&self, node: u64);
}
/// SelectType's current per-Document lifecycle owner, independent of Script.
pub trait SelectControlLifecycle {
    fn TextStyleAndContentChanged(&self, select: u64);
    fn FinishedParsingChildren(&self, select: u64);
}
enum LifecycleChange {
    FrameOwnerLoading(crate::frame_owner_loading::FrameOwnerLoadChange),
    VTT(crate::persistent_document::VTTLifecycleChange),
    IntersectionObserverConnected(u64),
    StatefulFormControlConnected(u64),
    FormAssociatedCustomElementUpgraded(u64),
    FormStateFinishedParsing(u64, FormStateParsingKind, bool),
    FormStateDocumentFinishedParsing,
    Attribute(AttributeChange),
    DetailsInsertedInto(u64),
    DialogInsertedInto(u64, bool),
    SelectTextStyleAndContentChanged(u64),
    SelectFinishedParsingChildren(u64),
    SelectSlottedButtonChanged(u64),
    OptionMutationObserverChanged(u64, bool),
    SelectDidRecalcStyle(u64),
    SelectMutationObserverChanged(u64),
    SelectPickerDidRecalcStyle(u64),
    ElementLayoutDetached(u64),
    InputValueAssigned(u64, Rc<[u16]>),
    InputInitialized(u64),
    MediaInitialized(u64),
    MediaFinishedParsing(u64),
    MediaSourceChanged(u64, u64, bool),
    MediaInserted(u64),
    AnchorInserted(u64),
    AnchorRemoved(u64),
    MediaRemoved(u64),
    MediaAttached(u64),
    InputAttached(u64, Option<bool>),
    InputFilesAssigned(u64, Rc<fileapi::FileList>),
    InputFilesCleared(u64),
}
#[derive(Default)]
pub struct AttributeLifecycleQueue {
    select_control: RefCell<Option<Weak<dyn SelectControlLifecycle>>>,
    frame_owner_document: RefCell<Option<Weak<RefCell<crate::DOM>>>>,
    form_controller: RefCell<Option<Weak<dyn crate::form_controller::DocumentFormController>>>,
    owner: RefCell<Option<Weak<dyn AttributeLifecycle>>>,
    pending: RefCell<VecDeque<LifecycleChange>>,
    popup_pending: RefCell<VecDeque<PopupLifecycleChange>>,
}
impl AttributeLifecycleQueue {
    pub fn BindSelectControl(&self, owner: Weak<dyn SelectControlLifecycle>) {
        *self.select_control.borrow_mut() = Some(owner);
    }
    fn SelectControl(&self) -> Option<Rc<dyn SelectControlLifecycle>> {
        self.select_control.borrow().as_ref()?.upgrade()
    }
    pub fn BindFormController(
        &self,
        owner: Weak<dyn crate::form_controller::DocumentFormController>,
    ) {
        *self.form_controller.borrow_mut() = Some(owner);
    }
    fn FormController(&self) -> Option<Rc<dyn crate::form_controller::DocumentFormController>> {
        self.form_controller
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
    }
    pub fn TakePopupLifecycleChanges(&self) -> VecDeque<PopupLifecycleChange> {
        std::mem::take(&mut *self.popup_pending.borrow_mut())
    }
    pub(crate) fn BindFrameOwnerDocument(&self, document: Weak<RefCell<crate::DOM>>) {
        *self.frame_owner_document.borrow_mut() = Some(document);
    }
    pub(crate) fn HasFrameOwnerDocument(&self) -> bool {
        self.frame_owner_document
            .borrow()
            .as_ref()
            .is_some_and(|document| document.strong_count() != 0)
    }
    pub(crate) fn RecordFrameOwnerLoading(
        &self,
        change: crate::frame_owner_loading::FrameOwnerLoadChange,
    ) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::FrameOwnerLoading(change));
    }
    pub(crate) fn RecordVTTLifecycle(
        &self,
        change: crate::persistent_document::VTTLifecycleChange,
    ) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::VTT(change));
    }
    pub fn RecordAnchorInserted(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::AnchorInserted(node));
    }
    pub fn RecordAnchorRemoved(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::AnchorRemoved(node));
    }
    pub fn RecordIntersectionObserverConnected(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::IntersectionObserverConnected(node));
    }

    pub fn HasOwner(&self) -> bool {
        self.owner
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some()
    }
    pub fn Bind(&self, owner: Weak<dyn AttributeLifecycle>) {
        assert!(
            self.owner.borrow().is_none(),
            "one attribute lifecycle owner per document"
        );
        *self.owner.borrow_mut() = Some(owner);
    }
    fn Owner(&self) -> Result<Rc<dyn AttributeLifecycle>, crate::dom_mutation::DOMMutationError> {
        self.owner
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .ok_or(crate::error::DOMException {
                kind: crate::error::DOMExceptionKind::LogicError,
                message: "popup methods require the canonical native element lifecycle owner",
            })
    }
    pub(crate) fn SetPopoverOpen(
        &self,
        node: u64,
        open: bool,
        source: Option<u64>,
    ) -> Result<(), crate::dom_mutation::DOMMutationError> {
        self.Owner()?.SetPopoverOpen(node, open, source)
    }
    pub(crate) fn SetDialogModal(
        &self,
        node: u64,
        open: bool,
        return_value: Option<&[u16]>,
    ) -> Result<(), crate::dom_mutation::DOMMutationError> {
        self.Owner()?.SetDialogModal(node, open, return_value)
    }
    pub(crate) fn WillRemove(&self, node: u64) {
        // ChildFrameDisconnector runs before the owner's removal callbacks.
        // Borrow just this original frame alias; never hold DOM across detach.
        let frame = self
            .frame_owner_document
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .and_then(|document| {
                let owner = document.borrow();
                let doc = owner.GetDocument();
                doc.FindNodeById(node)
                    .and_then(|index| doc.ContentFrame(index))
            });
        if let Some(frame) = frame {
            frame.DetachForRemove();
        }
        self.popup_pending
            .borrow_mut()
            .push_back(PopupLifecycleChange::Removed(node));
        let owner = self.owner.borrow().as_ref().and_then(Weak::upgrade);
        if let Some(owner) = owner {
            owner.WillRemove(node);
        }
    }
    pub fn Record(&self, change: AttributeChange) {
        if change.namespace.is_empty()
            && matches!(
                change.name.as_str(),
                "popover" | "open" | "closedby" | "interestfor"
            )
        {
            self.popup_pending
                .borrow_mut()
                .push_back(PopupLifecycleChange::Attribute(change.clone()));
        }
        if self
            .owner
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some()
        {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::Attribute(change));
        }
    }
    /// HTMLDetailsElement::InsertedInto follows the canonical DOM insertion
    /// stage and shares its synchronous transaction order with attributes.
    // Dialog rare-data construction may precede native lifecycle binding.
    // Retain the source insertion identity, as InputInitialized does.
    pub fn RecordDialogInsertedInto(&self, node: u64, insertion_point_connected: bool) {
        self.popup_pending
            .borrow_mut()
            .push_back(PopupLifecycleChange::DialogInserted(
                node,
                insertion_point_connected,
            ));
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::DialogInsertedInto(
                node,
                insertion_point_connected,
            ));
    }
    pub fn RecordDetailsInsertedInto(&self, node: u64) {
        if self.HasOwner() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::DetailsInsertedInto(node));
        }
    }
    pub fn RecordSelectTextStyleAndContentChanged(&self, node: u64) {
        if self.HasOwner() || self.SelectControl().is_some() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::SelectTextStyleAndContentChanged(node));
        }
    }
    pub fn RecordSelectFinishedParsingChildren(&self, node: u64) {
        if self.HasOwner() || self.SelectControl().is_some() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::SelectFinishedParsingChildren(node));
        }
    }
    pub fn RecordSelectSlottedButtonChanged(&self, node: u64) {
        if self.HasOwner() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::SelectSlottedButtonChanged(node));
        }
    }
    pub fn RecordStatefulFormControlConnected(&self, node: u64) {
        if self.HasOwner() || self.FormController().is_some() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::StatefulFormControlConnected(node));
        }
    }
    pub fn RecordFormAssociatedCustomElementUpgraded(&self, node: u64) {
        if self.FormController().is_some() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::FormAssociatedCustomElementUpgraded(node));
        }
    }
    pub fn RecordFormStateFinishedParsing(
        &self,
        node: u64,
        kind: FormStateParsingKind,
        was_finished: bool,
    ) {
        if self.HasOwner() || self.FormController().is_some() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::FormStateFinishedParsing(
                    node,
                    kind,
                    was_finished,
                ));
        }
    }
    pub fn RecordFormStateDocumentFinishedParsing(&self) {
        if self.HasOwner() || self.FormController().is_some() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::FormStateDocumentFinishedParsing);
        }
    }
    pub fn RecordOptionMutationObserverChanged(&self, node: u64, in_style_recalc: bool) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::OptionMutationObserverChanged(
                node,
                in_style_recalc,
            ));
    }
    pub fn RecordSelectDidRecalcStyle(&self, node: u64) {
        if self.HasOwner() || self.SelectControl().is_some() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::SelectDidRecalcStyle(node));
        }
    }
    pub fn RecordSelectMutationObserverChanged(&self, node: u64) {
        if self.HasOwner() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::SelectMutationObserverChanged(node));
        }
    }
    pub fn RecordSelectPickerDidRecalcStyle(&self, node: u64) {
        if self.HasOwner() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::SelectPickerDidRecalcStyle(node));
        }
    }
    pub fn RecordElementLayoutDetached(&self, node: u64) {
        if self.HasOwner() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::ElementLayoutDetached(node));
        }
    }
    // Source: HTMLInputElement constructor / ParserDidSetAttributes ->
    // InitializeTypeInParsing. Keep initial allocations until the native
    // lifecycle owner is bound; no observer runs under the Document borrow.
    pub fn RecordInputValueAssigned(&self, node: u64, value: Rc<[u16]>) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::InputValueAssigned(node, value));
    }
    pub fn RecordInputInitialized(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::InputInitialized(node));
    }
    pub fn RecordMediaSourceChanged(&self, media: u64, source: u64, added: bool) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::MediaSourceChanged(media, source, added));
    }
    pub fn RecordMediaRemoved(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::MediaRemoved(node));
    }
    pub fn RecordMediaInserted(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::MediaInserted(node));
    }
    pub fn RecordMediaAttached(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::MediaAttached(node));
    }
    pub fn RecordMediaFinishedParsing(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::MediaFinishedParsing(node));
    }
    pub fn RecordMediaInitialized(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::MediaInitialized(node));
    }
    pub fn RecordInputAttached(&self, node: u64, layout_image: Option<bool>) {
        if self.HasOwner() {
            self.pending
                .borrow_mut()
                .push_back(LifecycleChange::InputAttached(node, layout_image));
        }
    }
    pub fn RecordInputFilesAssigned(&self, node: u64, files: Rc<fileapi::FileList>) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::InputFilesAssigned(node, files));
    }
    pub fn RecordInputFilesCleared(&self, node: u64) {
        self.pending
            .borrow_mut()
            .push_back(LifecycleChange::InputFilesCleared(node));
    }
    pub fn Drain(&self) {
        if !self.HasOwner()
            && !self.HasFrameOwnerDocument()
            && self.FormController().is_none()
            && self.SelectControl().is_none()
        {
            return;
        }
        // Each queued fact is moved out before invoking its class owner.
        // Nested setters must finish their own source lifecycle before the
        // listener returns (e.g. beforetoggle's state revalidation).
        // Move this call's existing facts into its own continuation. A nested
        // setter drains newly recorded facts before this continuation resumes,
        // rather than consuming older sibling facts from the outer batch.
        let mut continuation = std::mem::take(&mut *self.pending.borrow_mut());
        while let Some(change) = continuation.pop_front() {
            let change = match change {
                LifecycleChange::FrameOwnerLoading(change) => {
                    let document = self
                        .frame_owner_document
                        .borrow()
                        .as_ref()
                        .and_then(Weak::upgrade);
                    if let Some(document) = document {
                        crate::frame_owner_loading::DispatchFrameOwnerLoadChange(&document, change);
                    }
                    if !self.pending.borrow().is_empty() {
                        self.Drain();
                    }
                    continue;
                }
                change => change,
            };
            if let LifecycleChange::VTT(change) = &change {
                change.Dispatch();
                if !self.pending.borrow().is_empty() {
                    self.Drain();
                }
                continue;
            }
            if let Some(controller) = self.FormController() {
                match &change {
                    LifecycleChange::StatefulFormControlConnected(node) => {
                        controller.StatefulControlConnected(*node)
                    }
                    LifecycleChange::FormAssociatedCustomElementUpgraded(node) => {
                        controller.CustomElementUpgraded(*node)
                    }
                    LifecycleChange::FormStateFinishedParsing(node, kind, was_finished) => {
                        controller.FinishedParsing(*node, *kind, *was_finished)
                    }
                    LifecycleChange::FormStateDocumentFinishedParsing => {
                        controller.DocumentFinishedParsing()
                    }
                    _ => {}
                }
            }
            if let Some(select) = self.SelectControl() {
                let handled = match &change {
                    LifecycleChange::SelectTextStyleAndContentChanged(node)
                    | LifecycleChange::SelectDidRecalcStyle(node) => {
                        select.TextStyleAndContentChanged(*node);
                        true
                    }
                    LifecycleChange::SelectFinishedParsingChildren(node) => {
                        select.FinishedParsingChildren(*node);
                        true
                    }
                    _ => false,
                };
                if handled {
                    if !self.pending.borrow().is_empty() {
                        self.Drain();
                    }
                    continue;
                }
            }
            let owner = self.owner.borrow().as_ref().and_then(Weak::upgrade);
            if let Some(owner) = owner {
                match change {
                    LifecycleChange::FrameOwnerLoading(_) => {
                        unreachable!("native frame loading dispatched above")
                    }
                    LifecycleChange::VTT(_) => {
                        unreachable!("native VTT lifecycle dispatched above")
                    }
                    LifecycleChange::InputValueAssigned(node, value) => {
                        owner.InputValueAssigned(node, value)
                    }
                    LifecycleChange::InputInitialized(node) => owner.InputInitialized(node),
                    LifecycleChange::MediaInitialized(node) => owner.MediaInitialized(node),
                    LifecycleChange::MediaFinishedParsing(node) => owner.MediaFinishedParsing(node),
                    LifecycleChange::MediaSourceChanged(media, source, added) => {
                        owner.MediaSourceChanged(media, source, added)
                    }
                    LifecycleChange::MediaInserted(node) => owner.MediaInserted(node),
                    LifecycleChange::AnchorInserted(node) => owner.AnchorInserted(node),
                    LifecycleChange::AnchorRemoved(node) => owner.AnchorRemoved(node),
                    LifecycleChange::IntersectionObserverConnected(node) => {
                        owner.IntersectionObserverConnected(node)
                    }
                    LifecycleChange::MediaRemoved(node) => owner.MediaRemoved(node),
                    LifecycleChange::MediaAttached(node) => owner.MediaAttached(node),
                    LifecycleChange::InputAttached(node, layout_image) => {
                        owner.InputAttached(node, layout_image)
                    }
                    LifecycleChange::InputFilesAssigned(node, files) => {
                        owner.InputFilesAssigned(node, files)
                    }
                    LifecycleChange::StatefulFormControlConnected(node) => {
                        owner.StatefulFormControlConnected(node)
                    }
                    LifecycleChange::FormAssociatedCustomElementUpgraded(_) => {}
                    LifecycleChange::FormStateFinishedParsing(node, kind, finished) => {
                        owner.FormStateFinishedParsing(node, kind, finished)
                    }
                    LifecycleChange::FormStateDocumentFinishedParsing => {
                        owner.FormStateDocumentFinishedParsing()
                    }
                    LifecycleChange::InputFilesCleared(node) => owner.InputFilesCleared(node),
                    LifecycleChange::Attribute(change) => owner.AttributeChanged(change),
                    LifecycleChange::DetailsInsertedInto(node) => owner.DetailsInsertedInto(node),
                    LifecycleChange::DialogInsertedInto(node, connected) => {
                        owner.DialogInsertedInto(node, connected)
                    }
                    LifecycleChange::SelectTextStyleAndContentChanged(node) => {
                        owner.SelectTextStyleAndContentChanged(node)
                    }
                    LifecycleChange::SelectFinishedParsingChildren(node) => {
                        owner.SelectFinishedParsingChildren(node)
                    }
                    LifecycleChange::SelectSlottedButtonChanged(node) => {
                        owner.SelectSlottedButtonChanged(node)
                    }
                    LifecycleChange::OptionMutationObserverChanged(node, recalc) => {
                        owner.OptionMutationObserverChanged(node, recalc)
                    }
                    LifecycleChange::SelectDidRecalcStyle(node) => owner.SelectDidRecalcStyle(node),
                    LifecycleChange::SelectMutationObserverChanged(node) => {
                        owner.SelectMutationObserverChanged(node)
                    }
                    LifecycleChange::SelectPickerDidRecalcStyle(node) => {
                        owner.SelectPickerDidRecalcStyle(node)
                    }
                    LifecycleChange::ElementLayoutDetached(node) => {
                        owner.ElementLayoutDetached(node)
                    }
                }
            }
            if !self.pending.borrow().is_empty() {
                self.Drain();
            }
        }
    }
}
/// Clone the queue under the DOM borrow, then release it before source class
/// callbacks can synchronously read or mutate that same DOM owner. Nested
/// mutations drain their own facts synchronously after releasing their DOM borrow.
pub fn DrainAttributeLifecycle(document: &Rc<RefCell<crate::DOM>>) {
    let queue = document.borrow().GetDocument().AttributeLifecycle().clone();
    queue.Drain();
    // Source frame-owner loading may call the native CreateFrame client and
    // reenter DOM. Preserve the same released-borrow transaction boundary.
    // A required port is retained on Document, together with its pending
    // notification/request, rather than published as a successful no-frame.
    let _ = crate::frame_owner_loading::DrainBoundFrameOwnerLoading(document);
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Owner {
        queue: Rc<AttributeLifecycleQueue>,
        seen: RefCell<Vec<String>>,
    }
    impl AttributeLifecycle for Owner {
        fn AnchorInserted(&self, _node: u64) {}
        fn AnchorRemoved(&self, _node: u64) {}
        fn IntersectionObserverConnected(&self, _: u64) {}
        fn MediaRemoved(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }

        fn MediaSourceChanged(&self, _: u64, _: u64, _: bool) {
            unreachable!("attribute-only fixture")
        }

        fn StatefulFormControlConnected(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn FormStateFinishedParsing(&self, _: u64, _: FormStateParsingKind, _: bool) {
            unreachable!("attribute-only fixture")
        }
        fn FormStateDocumentFinishedParsing(&self) {
            unreachable!("attribute-only fixture")
        }
        fn MediaInserted(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn MediaAttached(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn MediaFinishedParsing(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn MediaInitialized(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn SelectPickerDidRecalcStyle(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn SelectMutationObserverChanged(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn ElementLayoutDetached(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn SelectDidRecalcStyle(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn OptionMutationObserverChanged(&self, _: u64, _: bool) {
            unreachable!("attribute-only fixture")
        }
        fn SelectTextStyleAndContentChanged(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }

        fn SelectFinishedParsingChildren(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }
        fn SelectSlottedButtonChanged(&self, _: u64) {
            unreachable!("attribute-only fixture")
        }

        fn SetPopoverOpen(
            &self,
            _: u64,
            _: bool,
            _: Option<u64>,
        ) -> Result<(), crate::dom_mutation::DOMMutationError> {
            unreachable!("attribute-only fixture");
        }
        fn SetDialogModal(
            &self,
            _: u64,
            _: bool,
            _: Option<&[u16]>,
        ) -> Result<(), crate::dom_mutation::DOMMutationError> {
            unreachable!("attribute-only fixture");
        }
        fn WillRemove(&self, _node: u64) {
            unreachable!("attribute-only fixture");
        }
        fn InputValueAssigned(&self, _node: u64, _value: Rc<[u16]>) {
            unreachable!("attribute-only fixture");
        }
        fn InputInitialized(&self, _node: u64) {
            unreachable!("attribute-only fixture");
        }
        fn InputAttached(&self, _node: u64, _layout_image: Option<bool>) {
            unreachable!("attribute-only fixture");
        }
        fn InputFilesAssigned(&self, _node: u64, _files: Rc<fileapi::FileList>) {
            unreachable!("attribute-only fixture");
        }
        fn InputFilesCleared(&self, _node: u64) {
            unreachable!("attribute-only fixture");
        }
        fn DialogInsertedInto(&self, _node: u64, _connected: bool) {
            unreachable!("attribute-only fixture");
        }
        fn DetailsInsertedInto(&self, _node: u64) {
            unreachable!("attribute-only fixture");
        }
        fn AttributeChanged(&self, change: AttributeChange) {
            self.seen.borrow_mut().push(change.name.clone());
            if change.name == "open" {
                self.queue.Record(change_named("type"));
                // Source setter lifecycle completes before this listener
                // returns to the outer owner's beforetoggle revalidation.
                self.queue.Drain();
                assert_eq!(&*self.seen.borrow(), &["open", "type"]);
            }
        }
    }
    fn change_named(name: &str) -> AttributeChange {
        AttributeChange {
            node: 9,
            name: name.into(),
            namespace: String::new(),
            old_value: None,
            new_value: Some(String::new()),
            old_value_utf16: None,
            new_value_utf16: Some(Rc::from([])),
            reason: AttributeModificationReason::Directly,
            was_connected: false,
        }
    }
    #[test]
    fn nested_attribute_callbacks_finish_before_outer_continuation() {
        let queue = Rc::new(AttributeLifecycleQueue::default());
        let owner = Rc::new(Owner {
            queue: queue.clone(),
            seen: RefCell::new(Vec::new()),
        });
        let source: Rc<dyn AttributeLifecycle> = owner.clone();
        queue.Bind(Rc::downgrade(&source));
        queue.Record(change_named("open"));
        queue.Record(change_named("name"));
        queue.Drain();
        assert_eq!(&*owner.seen.borrow(), &["open", "type", "name"]);
        queue.Drain();
        assert_eq!(owner.seen.borrow().len(), 3);
    }
}
