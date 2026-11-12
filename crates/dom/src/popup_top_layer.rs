// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/dom/document.cc:8672-8751. These methods operate on the same
// canonical top_layer_elements and native pending-removal list as DOM queries.
use super::*;
use foundation::popup_types::{PopoverValueType, PopupTopLayerReason};
impl PersistentDocument {
    // ElementTraversal::Next within one TreeScope, HTMLCollection traversal.
    pub fn NextPopupCollectionElement(&self, root: usize, after: Option<usize>) -> Option<usize> {
        let mut current = after.unwrap_or(root);
        loop {
            let next = if let Some(&child) = self.arena[current].Children().first() {
                Some(child)
            } else {
                let mut node = current;
                loop {
                    if node == root {
                        break None;
                    }
                    let parent = self.arena[node].Parent()?;
                    let index = self.arena[parent]
                        .Children()
                        .iter()
                        .position(|&child| child == node)?;
                    if let Some(&next) = self.arena[parent].Children().get(index + 1) {
                        break Some(next);
                    }
                    node = parent;
                }
            }?;
            if self.arena[next].Type() == crate::persistent_document::DOMNodeType::kElement {
                return Some(next);
            }
            current = next;
        }
    }

    // Source: document.cc:8763-8774. Ordered top layer is already canonical.
    // Source: document.cc:9342-9345.
    pub fn ContainsNodeInTree(&self, root: usize, node: usize, flat: bool) -> bool {
        let mut current = Some(node);
        while let Some(node) = current {
            if node == root {
                return true;
            }
            current = if flat {
                self.FlatTreeParent(node)
            } else {
                self.arena[node].Parent()
            };
        }
        false
    }
    pub fn FinalizeAutofocus(&mut self) {
        self.popup_lifecycle_state.autofocus_candidates.clear();
        self.popup_lifecycle_state.autofocus_processed_flag = true;
    }
    // Source: TreeScope::AdjustedFocusedElement, using the canonical shadow
    // host/TreeScope relations rather than a second focus model.
    pub fn PopupAdjustedFocusedElement(&self, id: u64) -> Option<usize> {
        let mut element = self.FindNodeById(id)?;
        while self.TreeScopeRoot(element) != self.Root() {
            element = self.ShadowHost(self.TreeScopeRoot(element))?;
        }
        Some(element)
    }
    // Source: html_menu_item_element.cc:624-642,645-668. Called in the
    // existing native insertion/removal traversal, not a Page tree walk.
    pub fn ResetMenuItemAncestorCache(&mut self, element: usize) {
        if !self.popup_lifecycle_state.menu_elements_enabled
            || !self.arena[element].IsHTMLElement("menuitem")
        {
            return;
        }
        let mut current = self.arena[element].Parent();
        let mut owning = None;
        while let Some(ancestor) = current {
            if self.arena[ancestor].IsHTMLElement("menulist")
                || self.arena[ancestor].IsHTMLElement("menubar")
            {
                owning = Some(self.arena[ancestor].id);
                break;
            }
            current = self.arena[ancestor].Parent();
        }
        self.popup_lifecycle_state
            .menu_item_owners
            .insert(self.arena[element].id, owning);
    }
    pub fn EnsureMenuItemAncestorCache(&mut self, element: usize) {
        if !self
            .popup_lifecycle_state
            .menu_item_owners
            .contains_key(&self.arena[element].id)
        {
            self.ResetMenuItemAncestorCache(element);
        }
    }
    // Source: html_menu_item_element.cc:179-184; html_sub_menu_element.cc:25-27.
    pub fn MenuItemInvokedSubmenu(&self, element: usize) -> Option<usize> {
        if !self.popup_lifecycle_state.menu_elements_enabled {
            return None;
        }
        let parent = self.arena[element].Parent()?;
        if !self.arena[parent].IsHTMLElement("submenu") {
            return None;
        }
        self.arena[parent]
            .Children()
            .iter()
            .copied()
            .find(|&n| self.arena[n].IsHTMLElement("menulist"))
    }
    pub fn SuppressNextFocusInterest(&mut self, node: u64) {
        self.popup_lifecycle_state
            .interest_invokers
            .entry(node)
            .or_default()
            .suppress_next_focus_interest = true;
    }
    pub fn CountPopupFeature(&mut self, feature: &'static str) {
        use crate::use_counter::WebFeature::*;
        let feature = match feature {
            "PopoverTypeInvalid" => kPopoverTypeInvalid,
            "ValidPopoverAttribute" => kValidPopoverAttribute,
            "PopoverTypeAuto" => kPopoverTypeAuto,
            "PopoverTypeHint" => kPopoverTypeHint,
            "PopoverTypeManual" => kPopoverTypeManual,
            "PopoverShown" => kPopoverShown,
            "PopoverTypeAutoShown" => kPopoverTypeAutoShown,
            "PopoverTypeHintShown" => kPopoverTypeHintShown,
            "PopoverTypeManualShown" => kPopoverTypeManualShown,
            "ShowModalForElementInFullscreenStack" => kShowModalForElementInFullscreenStack,
            "DialogClosedby" => kDialogClosedby,
            _ => unreachable!("translated popup source metric"),
        };
        self.CountUse(feature);
    }
    pub fn ActiveModalDialog(&self) -> Option<usize> {
        self.top_layer_elements
            .iter()
            .rev()
            .filter_map(|&id| self.FindNodeById(id))
            .find(|&element| {
                self.DialogIsModal(element) && self.IsScheduledForTopLayerRemoval(element).is_none()
            })
    }
    pub fn AddAllOpenPopover(&mut self, element: usize) {
        self.open_popovers.insert(self.arena[element].id);
    }
    pub fn RemoveAllOpenPopover(&mut self, element: usize) {
        self.open_popovers.remove(&self.arena[element].id);
    }
    pub fn IsScheduledForTopLayerRemoval(&self, element: usize) -> Option<PopupTopLayerReason> {
        let id = self.arena[element].id;
        self.popup_lifecycle_state
            .top_layer_pending_removal
            .iter()
            .find(|(node, _)| *node == id)
            .map(|(_, reason)| *reason)
    }
    pub fn ScheduleForTopLayerRemoval(&mut self, element: usize, reason: PopupTopLayerReason) {
        if !self.IsInTopLayer(element) {
            return;
        }
        if let Some(existing) = self.IsScheduledForTopLayerRemoval(element) {
            assert_eq!(existing, reason);
        } else {
            self.popup_lifecycle_state
                .top_layer_pending_removal
                .push((self.arena[element].id, reason));
        }
        // Carries Document::ScheduleLayoutTreeUpdateIfNeeded across the DOM
        // boundary. No transition is completed or top-layer element removed.
        self.style_state
            .impact
            .Merge(crate::style_state::StyleUpdateImpact::LAYOUT);
    }
    pub fn RemoveFinishedTopLayerElements(&mut self, overlay_property_enabled: bool) -> bool {
        let mut to_remove = Vec::new();
        for &(id, _) in &self.popup_lifecycle_state.top_layer_pending_removal {
            let Some(element) = self.FindNodeById(id) else {
                continue;
            };
            let style = self
                .ResolvedStyleFor(element)
                .and_then(|style| style.ComputedStyle());
            let style = if overlay_property_enabled {
                style
            } else {
                style.filter(|style| !style.IsEnsuredInDisplayNone())
            };
            if style.is_none()
                || overlay_property_enabled
                    && style.is_some_and(|style| style.Overlay() == foundation::EOverlay::kNone)
            {
                to_remove.push(element);
            }
        }
        let removed = !to_remove.is_empty();
        for element in to_remove {
            self.RemoveFromTopLayerImmediately(element);
        }
        removed
    }
    pub fn RemoveFromTopLayerImmediately(&mut self, element: usize) {
        if !self.IsInTopLayer(element) {
            return;
        }
        let id = self.arena[element].id;
        self.top_layer_elements.retain(|&candidate| candidate != id);
        self.popup_lifecycle_state
            .top_layer_pending_removal
            .retain(|&(candidate, _)| candidate != id);
        self.ScheduleDisplayLockTopLayerCheck(element);
        if let Some(data) = self.popup_lifecycle_state.popovers.get_mut(&id) {
            if data.popover_type != PopoverValueType::None {
                data.implicit_anchor = None;
            }
        }
        self.style_state.layout_inputs_require_tree = true;
        self.style_state.measurement_geometry_revision = self
            .style_state
            .measurement_geometry_revision
            .wrapping_add(1);
    }
}
impl PersistentDocument {
    /// TreeScope::AdjustedFocusedElement projected for the specified original
    /// element's TreeScope. It follows only canonical shadow-host identities.
    pub fn AdjustedFocusedElementForTreeScope(
        &self,
        focused: u64,
        scope_element: usize,
    ) -> Option<usize> {
        let target_scope = self.TreeScopeRoot(scope_element);
        let mut focused = self.FindNodeById(focused)?;
        loop {
            let scope = self.TreeScopeRoot(focused);
            if scope == target_scope {
                return Some(focused);
            }
            focused = self.ShadowHost(scope)?;
        }
    }
}
