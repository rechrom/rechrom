//! Native dynamic classes of WebVTT cue nodes, vtt_element.h/cc,
//! vtt_cue_box.h/cc and VTTCueBackgroundBox in vtt_cue.h/cc.
//! Only aliases of the original translated source owners cross this boundary.
use super::{editing_node::DOMNodeClass, PersistentDocument};
use foundation::{AtomicString, LayoutUnit};
use std::{any::Any, rc::Rc};
pub trait VTTTextStyleSource: std::fmt::Debug {
    fn HasTrack(&self) -> bool;
    fn Owner(&self) -> Rc<dyn Any>;
    fn VisitStyleSheets(&self, visitor: &mut dyn FnMut(&Rc<cssom::CSSStyleSheet>));
}
pub trait VTTElementSource: VTTTextStyleSource {
    fn IsPastNode(&self) -> bool;
    fn Language(&self) -> AtomicString;
    fn CloneToNode(&self, node: super::DOMOwnerHandle);
}
pub trait VTTCueBoxSource: std::fmt::Debug {
    fn Owner(&self) -> Rc<dyn Any>;
    fn InsertedInto(&self, connected: bool);
    fn RemovedFrom(&self);
    fn IsInRegion(&self, parent_is_text_track_container: Option<bool>) -> bool;
    fn ForcesBlockFlowLayout(&self, parent_is_text_track_container: Option<bool>) -> bool;
    fn AdjustedPosition(&self, full_dimension: LayoutUnit) -> LayoutUnit;
}
pub trait VTTCueBackgroundSource: VTTTextStyleSource {
    fn DidRecalcStyle(&self);
}
pub(crate) enum VTTLifecycleChange {
    Inserted(Rc<dyn VTTCueBoxSource>, bool),
    Removed(Rc<dyn VTTCueBoxSource>),
    DidRecalcStyle(Rc<dyn VTTCueBackgroundSource>),
    ClonedElement(Rc<dyn VTTElementSource>, super::DOMOwnerHandle),
}
impl VTTLifecycleChange {
    pub(crate) fn Dispatch(&self) {
        match self {
            Self::Inserted(source, connected) => source.InsertedInto(*connected),
            Self::Removed(source) => source.RemovedFrom(),
            Self::DidRecalcStyle(source) => source.DidRecalcStyle(),
            Self::ClonedElement(source, node) => source.CloneToNode(*node),
        }
    }
}
impl PersistentDocument {
    pub fn SetVTTPastNeedsStyleRecalc(&mut self, node: usize) {
        // VTTElement::SetIsPastNode requests a local style change, rather than
        // Element::PseudoStateChanged's separate selector invalidation path.
        self.RecordStyleChange(crate::style_state::StyleChange::Node(node));
    }
    pub fn CloneVTTChildNodesFrom(&mut self, to: usize, from: usize) {
        let mut child = self.Node(from).Children().first().copied();
        while let Some(node) = child {
            child = self.Node(node).NextSibling();
            let cloned = self.CloneNode(node, true);
            self.AppendChild(to, cloned);
        }
    }
    pub fn RemoveVTTChildren(&mut self, node: usize) {
        self.SetTextContentUTF16(node, &[]);
    }
    /// Exact histogram inputs, retained until the document's UMA sink consumes
    /// them. Do not substitute tracing or change the source sample decision.
    pub fn RecordVTTContainsStyleBlock(&self, value: bool) {
        self.vtt_contains_style_block_samples
            .borrow_mut()
            .push_back(value);
    }
    pub fn TakeVTTContainsStyleBlockSamples(&self) -> std::collections::VecDeque<bool> {
        std::mem::take(&mut *self.vtt_contains_style_block_samples.borrow_mut())
    }
    pub fn VTTOriginatingElementForStyle(&self) -> Option<usize> {
        self.vtt_originating_element.get()
    }
    pub fn EnsureVTTOriginatingElementForStyle(&mut self) -> usize {
        if let Some(node) = self.vtt_originating_element.get() {
            return node;
        }
        let node = self.CreateElementWithQualifiedName(
            super::DOMNamespace::kNone,
            foundation::QualifiedName::new(
                AtomicString::default(),
                AtomicString::from_str(""),
                AtomicString::from_str(""),
            ),
            Vec::new(),
        );
        self.vtt_originating_element.set(Some(node));
        node
    }
    pub fn BindVTTElementSource(&mut self, node: usize, owner: Rc<dyn VTTElementSource>) {
        self.arena[node].class = DOMNodeClass::VTTElement(owner);
    }
    pub fn BindVTTCueBoxSource(&mut self, node: usize, owner: Rc<dyn VTTCueBoxSource>) {
        self.arena[node].class = DOMNodeClass::VTTCueBox(owner);
    }
    pub fn BindVTTCueBackgroundSource(
        &mut self,
        node: usize,
        owner: Rc<dyn VTTCueBackgroundSource>,
    ) {
        self.arena[node].class = DOMNodeClass::VTTCueBackgroundBox(owner);
    }
    pub fn IsVTTElement(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::VTTElement(_))
    }
    pub fn IsVTTCueBox(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::VTTCueBox(_))
    }
    pub fn IsVTTCueBackgroundBox(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::VTTCueBackgroundBox(_))
    }
    pub fn IsTextTrackContainer(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::TextTrackContainer(_))
    }
    pub fn VTTIsPastNode(&self, node: usize) -> bool {
        match &self.arena[node].class {
            DOMNodeClass::VTTElement(source) => source.IsPastNode(),
            _ => false,
        }
    }
    pub fn VTTLanguage(&self, node: usize) -> AtomicString {
        match &self.arena[node].class {
            DOMNodeClass::VTTElement(source) => source.Language(),
            _ => AtomicString::default(),
        }
    }
    pub fn VTTHasTrack(&self, node: usize) -> bool {
        match &self.arena[node].class {
            DOMNodeClass::VTTElement(s) => s.HasTrack(),
            DOMNodeClass::VTTCueBackgroundBox(s) => s.HasTrack(),
            _ => false,
        }
    }
    pub fn VisitVTTStyleSheets(
        &self,
        node: usize,
        visitor: &mut dyn FnMut(&Rc<cssom::CSSStyleSheet>),
    ) {
        match &self.arena[node].class {
            DOMNodeClass::VTTElement(source) => source.VisitStyleSheets(visitor),
            DOMNodeClass::VTTCueBackgroundBox(source) => source.VisitStyleSheets(visitor),
            _ => {}
        }
    }
    pub fn VTTElementOwner(&self, node: usize) -> Option<Rc<dyn Any>> {
        match &self.arena[node].class {
            DOMNodeClass::VTTElement(source) => Some(source.Owner()),
            _ => None,
        }
    }
    pub fn VTTCueBoxOwner(&self, node: usize) -> Option<Rc<dyn Any>> {
        match &self.arena[node].class {
            DOMNodeClass::VTTCueBox(source) => Some(source.Owner()),
            _ => None,
        }
    }
    pub fn VTTCueBackgroundOwner(&self, node: usize) -> Option<Rc<dyn Any>> {
        match &self.arena[node].class {
            DOMNodeClass::VTTCueBackgroundBox(source) => Some(source.Owner()),
            _ => None,
        }
    }
    pub fn VTTCueBoxAdjustedPosition(&self, node: usize, full: LayoutUnit) -> Option<LayoutUnit> {
        match &self.arena[node].class {
            DOMNodeClass::VTTCueBox(source) => Some(source.AdjustedPosition(full)),
            _ => None,
        }
    }
    pub fn VTTCueBoxForcesBlockFlowLayout(&self, node: usize) -> bool {
        match &self.arena[node].class {
            DOMNodeClass::VTTCueBox(source) => source.ForcesBlockFlowLayout(
                self.Node(node)
                    .Parent()
                    .map(|p| self.IsTextTrackContainer(p)),
            ),
            _ => false,
        }
    }
    pub fn VTTCueBoxIsInRegion(&self, node: usize) -> bool {
        match &self.arena[node].class {
            DOMNodeClass::VTTCueBox(source) => source.IsInRegion(
                self.Node(node)
                    .Parent()
                    .map(|p| self.IsTextTrackContainer(p)),
            ),
            _ => false,
        }
    }
    pub(super) fn VTTElementCloned(&self, source: usize, node: usize) {
        if let DOMNodeClass::VTTElement(owner) = &self.arena[source].class {
            self.attribute_lifecycle
                .RecordVTTLifecycle(VTTLifecycleChange::ClonedElement(
                    owner.clone(),
                    self.NodeHandle(node),
                ));
        }
    }
    pub(super) fn VTTNodeInserted(&self, node: usize) {
        if let DOMNodeClass::VTTCueBox(source) = &self.arena[node].class {
            self.attribute_lifecycle
                .RecordVTTLifecycle(VTTLifecycleChange::Inserted(
                    source.clone(),
                    self.IsConnectedToDocument(node),
                ));
        }
    }
    pub(super) fn VTTNodeRemoved(&self, node: usize) {
        if let DOMNodeClass::VTTCueBox(source) = &self.arena[node].class {
            self.attribute_lifecycle
                .RecordVTTLifecycle(VTTLifecycleChange::Removed(source.clone()));
        }
    }
    pub(super) fn VTTNodeDidRecalcStyle(&self, node: usize) {
        if !self.HasCustomStyleCallbacks(node) {
            return;
        }
        if let DOMNodeClass::VTTCueBackgroundBox(source) = &self.arena[node].class {
            self.attribute_lifecycle
                .RecordVTTLifecycle(VTTLifecycleChange::DidRecalcStyle(source.clone()));
        }
    }
    pub fn SetVTTShadowPseudoId(&mut self, node: usize, id: &str) {
        self.shadow_pseudo_ids.insert(node, id.into());
    }
}
