//! Document-owned :hover/:active state and the frozen active chain.
//! Chromium: core/dom/document.cc:5766-5840,8957-9100;
//! core/dom/element.cc:13146-13192; html_label_element.cc:127-149.
use super::PersistentDocument;
use crate::style_state::{StyleChange, UserActionPseudoState};
use std::collections::HashMap;

/// Source Document's registered FocusedElementChangeObserver owner. The
/// observer is invoked after focus identity/options and last-user-focus state
/// are stored, before Element::SetFocused and the focus event sequence.
pub trait FocusedElementChangeObserver {
    fn DidChangeFocus(&self);
}

#[derive(Clone, Copy, Default)]
struct ElementActions {
    hovered: bool,
    dragged: bool,
    can_receive_dropped_files: bool,
    focused: bool,
    focus_within: bool,
    last_focus_from_user_gesture: bool,
    active: bool,
    in_active_chain: bool,
}
pub(super) struct UserActionElements {
    elements: HashMap<usize, ElementActions>,
    hover_element: Option<usize>,
    active_element: Option<usize>,
    focus_options: Option<std::rc::Rc<foundation::FocusOptions>>,
    last_focus_type: foundation::FocusType,
    focused_element_change_observers:
        std::cell::RefCell<HashMap<usize, std::rc::Rc<dyn FocusedElementChangeObserver>>>,
    had_keyboard_event: bool,
    accessibility_always_show_focus: foundation::shared_value::SharedValue<bool>,
    focus_without_user_activation_policy_allowed: foundation::shared_value::SharedValue<bool>,
}

impl Default for UserActionElements {
    fn default() -> Self {
        Self {
            elements: Default::default(),
            hover_element: None,
            active_element: None,
            focus_options: None,
            last_focus_type: Default::default(),
            focused_element_change_observers: Default::default(),
            had_keyboard_event: false,
            accessibility_always_show_focus: Default::default(),
            // permissions_policy_features.json5:346-350, EnableForAll.
            focus_without_user_activation_policy_allowed:
                foundation::shared_value::SharedValue::new(true),
        }
    }
}

/// Small source effects/owner IDs for Page's existing interaction-state bridge.
/// Element flags and active-chain membership remain private to this Document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserActionStateUpdate {
    pub notify_element_activated: bool,
    pub hovered_node_id: Option<u64>,
    pub active_node_id: Option<u64>,
}

impl PersistentDocument {
    // Document::Add/RemoveFocusedElementChangeObserver, document.cc:10070-10080.
    // Identity is the original observer allocation, never a copied callback.
    pub fn AddFocusedElementChangeObserver(
        &mut self,
        observer: std::rc::Rc<dyn FocusedElementChangeObserver>,
    ) {
        self.user_action_elements
            .focused_element_change_observers
            .borrow_mut()
            .insert(
                std::rc::Rc::as_ptr(&observer) as *const () as usize,
                observer,
            );
    }
    pub fn RemoveFocusedElementChangeObserver(
        &mut self,
        observer: &std::rc::Rc<dyn FocusedElementChangeObserver>,
    ) {
        let key = std::rc::Rc::as_ptr(observer) as *const () as usize;
        assert!(self
            .user_action_elements
            .focused_element_change_observers
            .borrow_mut()
            .remove(&key)
            .is_some());
    }
    /// Borrow-free handles permit observer reentry into the same live Document.
    pub fn FocusedElementChangeObservers(
        &self,
    ) -> Vec<std::rc::Rc<dyn FocusedElementChangeObserver>> {
        self.user_action_elements
            .focused_element_change_observers
            .borrow()
            .values()
            .cloned()
            .collect()
    }
    pub fn ClearFocusedElementChangeObservers(&self) {
        self.user_action_elements
            .focused_element_change_observers
            .borrow_mut()
            .clear();
    }
    /// Projection of the document's FocusWithoutUserActivation policy owner.
    /// Policy parsing/updates must write this owner, never a Page-side mirror.
    pub fn FocusWithoutUserActivationPolicyOwner(
        &self,
    ) -> foundation::shared_value::SharedValue<bool> {
        self.user_action_elements
            .focus_without_user_activation_policy_allowed
            .clone()
    }
    pub fn ElementEditContext(
        &self,
        element: usize,
    ) -> Option<std::rc::Rc<dyn foundation::focus::ElementEditContextFocus>> {
        self.arena[element].edit_context.clone()
    }
    /// Storage half of Element's rare-data EditContext attachment. The source
    /// binding supplies its real native owner; no default EditContext exists.
    pub fn StoreElementEditContext(
        &mut self,
        element: usize,
        owner: Option<std::rc::Rc<dyn foundation::focus::ElementEditContextFocus>>,
    ) {
        assert_eq!(self.arena[element].node_type, super::DOMNodeType::kElement);
        self.arena[element].edit_context = owner;
    }
    pub fn HasActiveLabelForControl(&self, control: usize) -> bool {
        self.user_action_elements
            .elements
            .iter()
            .any(|(&node, state)| {
                state.active
                    && self.Node(node).IsHTMLElement("label")
                    && self.LabelControl(node) == Some(control)
            })
    }

    pub fn GetFocusOptions(&self) -> Option<std::rc::Rc<foundation::FocusOptions>> {
        self.user_action_elements.focus_options.clone()
    }
    pub fn SetFocusOptions(&mut self, options: Option<std::rc::Rc<foundation::FocusOptions>>) {
        self.user_action_elements.focus_options = options;
    }
    pub fn LastFocusType(&self) -> foundation::FocusType {
        self.user_action_elements.last_focus_type
    }
    pub fn SetLastFocusType(&mut self, focus_type: foundation::FocusType) {
        self.user_action_elements.last_focus_type = focus_type;
    }
    pub fn HadKeyboardEvent(&self) -> bool {
        self.user_action_elements.had_keyboard_event
    }
    pub fn SetHadKeyboardEvent(&mut self, keyboard: bool) -> bool {
        let previous = self.user_action_elements.had_keyboard_event;
        self.user_action_elements.had_keyboard_event = keyboard;
        previous
    }
    pub fn SetAccessibilityAlwaysShowFocusOwner(
        &mut self,
        owner: foundation::shared_value::SharedValue<bool>,
    ) {
        self.user_action_elements.accessibility_always_show_focus = owner;
    }
    pub fn AccessibilityAlwaysShowFocus(&self) -> bool {
        self.user_action_elements
            .accessibility_always_show_focus
            .get()
    }
    pub fn FocusedUserActionElements(&self) -> impl Iterator<Item = usize> + '_ {
        self.user_action_elements
            .elements
            .iter()
            .filter_map(|(&node, state)| state.focused.then_some(node))
    }
    pub fn IsElementFocused(&self, element: usize) -> bool {
        self.user_action_elements
            .elements
            .get(&element)
            .is_some_and(|state| state.focused)
    }
    pub fn StoreElementFocused(&mut self, element: usize, focused: bool) {
        self.user_action_elements
            .elements
            .entry(element)
            .or_default()
            .focused = focused;
        self.PruneUserActionEntry(element);
        self.RefreshDisplayLockFocusAncestors(element);
    }
    pub fn WasLastFocusFromUserGesture(&self, element: usize) -> bool {
        self.user_action_elements
            .elements
            .get(&element)
            .is_some_and(|state| state.last_focus_from_user_gesture)
    }
    pub fn SetLastFocusFromUserGesture(&mut self, element: usize, user: bool) {
        self.user_action_elements
            .elements
            .entry(element)
            .or_default()
            .last_focus_from_user_gesture = user;
        self.PruneUserActionEntry(element);
    }
    pub fn HasElementFocusWithin(&self, element: usize) -> bool {
        self.user_action_elements
            .elements
            .get(&element)
            .is_some_and(|state| state.focus_within)
    }
    pub fn StoreElementFocusWithin(&mut self, element: usize, focused: bool) {
        let changed = self.HasElementFocusWithin(element) != focused;
        self.user_action_elements
            .elements
            .entry(element)
            .or_default()
            .focus_within = focused;
        self.PruneUserActionEntry(element);
        if changed {
            self.RecordEditingNodeFactsChange(element);
        }
    }
    pub fn InvalidateFocusPseudo(&mut self, element: usize, state: UserActionPseudoState) {
        self.RecordStyleChange(StyleChange::UserAction(element, state));
    }
    /// LayoutSelection::InvalidateStyleAndPaintForSelection records the same
    /// local ::selection and selector invalidation on the canonical element.
    pub fn InvalidateSelectionPseudo(&mut self, element: usize) {
        self.RecordStyleChange(StyleChange::Node(element));
        self.RecordStyleChange(StyleChange::PseudoSelection(element));
    }
    /// Element::FocusStateChanged, element.cc:9046-9101. The caller supplies
    /// the existing LayoutObject attachment fact; style remains element-owned.
    pub fn InvalidateFocusStateStyle(&mut self, element: usize, has_layout: bool) {
        use layoutng_style::style::computed_style_constants::PseudoId;
        let Some(style) = self
            .ResolvedStyleFor(element)
            .and_then(|style| style.ComputedStyle())
        else {
            return;
        };
        if !has_layout
            && !(foundation::RuntimeEnabledFeatures::HTMLAreaElementDisplayNoneEnabled()
                && self.Node(element).IsHTMLElement("area"))
        {
            return;
        }
        let subtree = style.HasPseudoElementStyle(PseudoId::kPseudoIdFirstLetter);
        let appearance = style.HasEffectiveAppearance();
        self.RecordStyleChange(StyleChange::Node(element));
        if subtree {
            self.style_state.subtree_style_recalc_roots.insert(element);
        }
        for pseudo in [
            UserActionPseudoState::Focus,
            UserActionPseudoState::FocusVisible,
            UserActionPseudoState::FocusWithin,
        ] {
            self.InvalidateFocusPseudo(element, pseudo);
        }
        if appearance {
            self.style_state.impact.paint = true;
        }
    }
    pub fn IsElementHovered(&self, element: usize) -> bool {
        self.user_action_elements
            .elements
            .get(&element)
            .is_some_and(|state| state.hovered)
    }
    // Node::IsDragged/SetDragged + UserActionElementSet, node.h:622-624,
    // node.cc:3595-3597. Non-elements have no entry in the source set.
    pub fn IsNodeDragged(&self, node: usize) -> bool {
        self.user_action_elements
            .elements
            .get(&node)
            .is_some_and(|s| s.dragged)
    }
    /// Element::SetDragged cc:8652-8690. The existing Layout attachment is
    /// carried as one source scalar; Style owns selector dependency decisions.
    pub fn SetNodeDragged(&mut self, node: usize, dragged: bool, has_layout: bool) -> bool {
        if self.Node(node).Type() != super::DOMNodeType::kElement
            || self.IsNodeDragged(node) == dragged
        {
            return false;
        }
        self.user_action_elements
            .elements
            .entry(node)
            .or_default()
            .dragged = dragged;
        self.PruneUserActionEntry(node);
        if !has_layout && dragged {
            return false;
        }
        self.RecordStyleChange(StyleChange::Dragged(node, has_layout));
        true
    }
    /// HTMLInputElement.cc:1834-1845. The input retains the source flag and
    /// its original upload button owns the active pseudo-state transition.
    pub fn SetFileInputCanReceiveDroppedFiles(&mut self, element: usize, value: bool) -> bool {
        let previous = self
            .user_action_elements
            .elements
            .get(&element)
            .is_some_and(|state| state.can_receive_dropped_files);
        if previous == value {
            return false;
        }
        self.user_action_elements
            .elements
            .entry(element)
            .or_default()
            .can_receive_dropped_files = value;
        self.PruneUserActionEntry(element);
        if let Some(button) = self.FileUploadButtonElement(element) {
            self.SetElementActive(button, value);
        }
        true
    }
    pub fn IsElementActive(&self, element: usize) -> bool {
        self.user_action_elements
            .elements
            .get(&element)
            .is_some_and(|state| state.active)
    }
    pub fn IsElementInActiveChain(&self, element: usize) -> bool {
        self.user_action_elements
            .elements
            .get(&element)
            .is_some_and(|state| state.in_active_chain)
    }
    pub fn HoverElement(&self) -> Option<usize> {
        self.user_action_elements.hover_element
    }
    pub fn ActiveElementForUserAction(&self) -> Option<usize> {
        self.user_action_elements.active_element
    }

    fn SetInActiveChain(&mut self, element: usize, value: bool) {
        self.user_action_elements
            .elements
            .entry(element)
            .or_default()
            .in_active_chain = value;
        self.PruneUserActionEntry(element);
    }
    fn PruneUserActionEntry(&mut self, element: usize) {
        if self
            .user_action_elements
            .elements
            .get(&element)
            .is_some_and(|state| {
                !state.hovered
                    && !state.dragged
                    && !state.can_receive_dropped_files
                    && !state.active
                    && !state.in_active_chain
                    && !state.focused
                    && !state.focus_within
                    && !state.last_focus_from_user_gesture
            })
        {
            self.user_action_elements.elements.remove(&element);
        }
    }
    /// Source Element::SetHovered plus HTMLLabelElement's associated-control
    /// override. This does not change Document::hover_element_.
    pub fn SetElementHovered(&mut self, element: usize, hovered: bool) -> bool {
        let mut changed = self.IsElementHovered(element) != hovered;
        if changed {
            self.user_action_elements
                .elements
                .entry(element)
                .or_default()
                .hovered = hovered;
            self.PruneUserActionEntry(element);
            self.RecordStyleChange(StyleChange::UserAction(
                element,
                UserActionPseudoState::Hover,
            ));
        }
        if self.Node(element).IsHTMLElement("label") {
            if let Some(control) = self.LabelControl(element) {
                if self.IsElementHovered(control) != hovered {
                    changed |= self.SetElementHovered(control, hovered);
                }
            }
        }
        changed
    }
    /// Source Element::SetActive plus HTMLLabelElement's associated-control
    /// override. Simulated click calls this leaf, not the Document active-chain
    /// transaction, so it cannot replace the frozen pointer-down owner.
    pub fn SetElementActive(&mut self, element: usize, active: bool) -> bool {
        let mut changed = self.IsElementActive(element) != active;
        if changed {
            self.user_action_elements
                .elements
                .entry(element)
                .or_default()
                .active = active;
            self.PruneUserActionEntry(element);
            self.RecordStyleChange(StyleChange::UserAction(
                element,
                UserActionPseudoState::Active,
            ));
        }
        if self.Node(element).IsHTMLElement("label") {
            if let Some(control) = self.LabelControl(element) {
                if self.IsElementActive(control) != active {
                    changed |= self.SetElementActive(control, active);
                }
            }
        }
        changed
    }
    fn SkipDisplayNoneUserActionAncestors(&self, mut element: Option<usize>) -> Option<usize> {
        while let Some(index) = element {
            // The resident style is Element's canonical layout-attachment
            // result. display:contents and image-map areas have no own box.
            if self
                .ResolvedStyleFor(index)
                .is_some_and(|style| style.generates_box || style.display_contents)
                || self.Node(index).IsHTMLElement("area")
            {
                return Some(index);
            }
            element = self.FlatTreeParentElement(index);
        }
        None
    }
    fn UpdateActiveState(
        &mut self,
        is_active: bool,
        update_active_chain: bool,
        new_active: Option<usize>,
        visible_target: Option<usize>,
    ) {
        let old_active = self.user_action_elements.active_element;
        if let Some(old) = old_active.filter(|_| !is_active) {
            let mut current = Some(old);
            while let Some(element) = current {
                current = self.UserActionElementParent(element);
                self.SetElementActive(element, false);
                self.SetInActiveChain(element, false);
            }
            self.user_action_elements.active_element = None;
        } else if old_active.is_none() && is_active {
            if let Some(new_active) = new_active {
                let mut current = Some(new_active);
                while let Some(element) = current {
                    self.SetInActiveChain(element, true);
                    current = self.UserActionElementParent(element);
                }
                self.user_action_elements.active_element = Some(new_active);
            }
        }
        let allow_active_changes =
            old_active.is_none() && self.user_action_elements.active_element.is_some();
        if !allow_active_changes {
            return;
        }
        debug_assert!(is_active);
        let mut current = visible_target;
        while let Some(element) = current {
            if update_active_chain || self.IsElementInActiveChain(element) {
                self.SetElementActive(element, true);
            }
            current = self.UserActionElementParent(element);
        }
    }
    fn UpdateHoverState(&mut self, new_hover: Option<usize>) {
        let old_hover = self.user_action_elements.hover_element;
        if old_hover == new_hover {
            return;
        }
        self.user_action_elements.hover_element = new_hover;
        let mut remove = Vec::new();
        let mut current = old_hover.filter(|&element| self.IsConnectedToDocument(element));
        while let Some(element) = current {
            remove.push(element);
            current = self.UserActionElementParent(element);
        }
        let mut add = Vec::new();
        current = new_hover;
        while let Some(element) = current {
            add.push(element);
            current = self.UserActionElementParent(element);
        }
        let (mut remove_index, mut add_index) = if remove.len() > add.len() {
            (remove.len() - add.len(), 0)
        } else {
            (0, add.len() - remove.len())
        };
        while add_index < add.len() {
            if remove[remove_index] == add[add_index] {
                break;
            }
            remove_index += 1;
            add_index += 1;
        }
        for &element in &remove[..remove_index] {
            self.SetElementHovered(element, false);
        }
        for &element in &add[..add_index] {
            self.SetElementHovered(element, true);
        }
        for &element in &add[add_index..] {
            if Some(element) == self.user_action_elements.hover_element {
                self.SetElementHovered(element, true);
            }
        }
    }
    // Element::DetachLayoutTree calls the Document pointer adjustments before
    // UserActionElementSet::DidDetach clears this element's retained flags.
    // Flat-tree assignment is evaluated by this DOM's existing live owner, so
    // there is no deferred/dirty slot-assignment snapshot to consult here.
    pub(super) fn UserActionElementDetached(&mut self, element: usize) {
        if self.user_action_elements.hover_element == Some(element) {
            self.user_action_elements.hover_element =
                self.SkipDisplayNoneUserActionAncestors(Some(element));
        }
        if self.user_action_elements.active_element == Some(element) {
            self.user_action_elements.active_element =
                self.SkipDisplayNoneUserActionAncestors(Some(element));
        }
        // Source DidDetach clears active/chain/hover/focus-within, retaining
        // dragged and focused: :-webkit-drag may itself remove the LayoutObject.
        if let Some(state) = self.user_action_elements.elements.get_mut(&element) {
            state.active = false;
            state.in_active_chain = false;
            state.hovered = false;
            state.focus_within = false;
        }
        self.PruneUserActionEntry(element);
    }

    fn UserActionTarget(&self, id: Option<u64>) -> Option<usize> {
        id.and_then(|id| self.FindNodeById(id)).filter(|&index| {
            self.Node(index).Type() == super::DOMNodeType::kElement
                && self.NodeDocumentHandle(index) == self.RootHandle()
                && self.IsConnectedToDocument(index)
        })
    }
    fn UserActionUpdate(&self, notify: bool) -> UserActionStateUpdate {
        UserActionStateUpdate {
            notify_element_activated: notify,
            hovered_node_id: self
                .user_action_elements
                .hover_element
                .map(|index| self.Node(index).Id()),
            active_node_id: self
                .user_action_elements
                .active_element
                .map(|index| self.Node(index).Id()),
        }
    }
    /// Source Document::UpdateHoverActiveState. A Page domain supplies only
    /// its own document's native hit target; embedded documents have independent
    /// frame/Document owners and receive their own typed input transaction.
    pub fn UpdateHoverActiveState(
        &mut self,
        is_active: bool,
        update_active_chain: bool,
        target: Option<u64>,
    ) -> UserActionStateUpdate {
        let target = self.UserActionTarget(target);
        let visible = self.SkipDisplayNoneUserActionAncestors(target);
        self.UpdateActiveState(is_active, update_active_chain, target, visible);
        self.UpdateHoverState(visible);
        self.UserActionUpdate(is_active)
    }
    /// Existing immutable interaction snapshots name already-decided source
    /// targets. Admit those values through this owner, so Style never rebuilds
    /// a competing ancestor-chain decision from scalar IDs.
    pub fn SetUserActionTargets(&mut self, hover: Option<u64>, active: Option<u64>) {
        let hover = self.UserActionTarget(hover);
        let active = self.UserActionTarget(active);
        if self.user_action_elements.active_element != active {
            self.UpdateActiveState(false, true, None, None);
            self.UpdateActiveState(active.is_some(), true, active, active);
        }
        self.UpdateHoverState(hover);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistent_document::{DOMNamespace, DOMShadowRootMode, ResolvedNodeStyle};
    fn element(document: &mut PersistentDocument, parent: usize, tag: &str) -> usize {
        let node = document.CreateElementDefault(DOMNamespace::kHTML, tag.into());
        document.AppendChild(parent, node);
        document.SetResolvedStyle(node, ResolvedNodeStyle::default());
        node
    }
    #[test]
    fn active_chain_stays_frozen_while_hover_moves_and_hidden_target_is_skipped() {
        let mut document = PersistentDocument::new();
        let root = document.Root();
        let parent = element(&mut document, root, "div");
        let first = element(&mut document, parent, "button");
        let next = element(&mut document, parent, "button");
        let first_id = document.Node(first).Id();
        let next_id = document.Node(next).Id();
        document.StyleStateMut().all_dirty = false;
        document.UpdateHoverActiveState(true, true, Some(first_id));
        let update = document.UpdateHoverActiveState(true, false, Some(next_id));
        assert_eq!(update.active_node_id, Some(first_id));
        assert_eq!(update.hovered_node_id, Some(next_id));
        assert!(document.IsElementActive(first));
        assert!(document.IsElementActive(parent));
        assert!(!document.IsElementActive(next));
        assert!(!document.IsElementHovered(first));
        assert!(document.IsElementHovered(next));
        document.UpdateHoverActiveState(false, true, Some(next_id));
        assert!(!document.IsElementInActiveChain(first));
        assert!(!document.IsElementActive(parent));
        document.SetResolvedStyle(
            next,
            ResolvedNodeStyle {
                generates_box: false,
                own_generates_box: false,
                ..Default::default()
            },
        );
        let update = document.UpdateHoverActiveState(true, true, Some(next_id));
        assert_eq!(update.active_node_id, Some(next_id));
        assert_eq!(update.hovered_node_id, Some(document.Node(parent).Id()));
        assert!(document.IsElementInActiveChain(next));
        assert!(!document.IsElementActive(next));
        assert!(document.IsElementActive(parent));
        assert!(document
            .StyleState()
            .changes
            .contains(&StyleChange::UserAction(
                parent,
                UserActionPseudoState::Active
            )));
    }
    #[test]
    fn hover_uses_assigned_slot_and_stops_at_top_layer_element() {
        let mut document = PersistentDocument::new();
        let root = document.Root();
        let ancestor = element(&mut document, root, "div");
        let dialog = element(&mut document, ancestor, "dialog");
        let host = element(&mut document, dialog, "div");
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let slot = element(&mut document, shadow, "slot");
        let button = element(&mut document, host, "button");
        document.ShowModalDialog(dialog);
        document.UpdateHoverActiveState(true, true, Some(document.Node(button).Id()));
        assert_eq!(document.FlatTreeParentElement(button), Some(slot));
        for node in [button, slot, host, dialog] {
            assert!(document.IsElementHovered(node));
            assert!(document.IsElementActive(node));
        }
        assert!(!document.IsElementHovered(ancestor));
        assert!(!document.IsElementActive(ancestor));
    }
}
