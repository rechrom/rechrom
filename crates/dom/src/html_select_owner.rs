//! Retained select/selectedcontent DOM owners and their source lifecycle.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]
use super::{editing_node::DOMNodeClass as C, DOMNamespace, PersistentDocument};

/// RuntimeEnabledFeatures admission before parser feed. The stable default is
/// the pinned revision's metadata, not an inferred DOM or appearance value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMSelectSourceFeatures {
    pub selectedcontent_spec: bool,
    pub selectedcontent_multiple: bool,
    pub selectedcontentelement_attribute: bool,
    pub filterable_select: bool,
    pub customizable_combobox: bool,
    pub appearance_base: bool,
    pub customizable_select_multiple_popup: bool,
    pub input_in_select: bool,
    pub select_autofill_popover_preview: bool,
    pub select_remove_overflow_hidden: bool,
}
impl Default for DOMSelectSourceFeatures {
    fn default() -> Self {
        Self {
            selectedcontent_spec: true,
            selectedcontent_multiple: false,
            selectedcontentelement_attribute: false,
            filterable_select: false,
            customizable_combobox: false,
            appearance_base: false,
            customizable_select_multiple_popup: false,
            input_in_select: false,
            select_autofill_popover_preview: true,
            select_remove_overflow_hidden: true,
        }
    }
}
/// Native subclasses allocated only by SelectType::CreateShadowSubtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMSelectUAPartKind {
    MenuListInner,
    Picker,
    AutofillPreview,
}
#[derive(Clone)]
pub(super) struct HTMLSelectElementData {
    pub(super) content_model_violations_count: u32,
    pub(super) option_style: layoutng_style::style::menu_list_inner_style::MenuListOptionStyleOwner,
    pub(super) size: u32,
    pub(super) is_multiple: bool,
    pub(super) uses_menu_list: bool,
    pub(super) active_option: Option<usize>,
    pub(super) native_popup_is_visible: bool,
    pub(super) is_appearance_base_picker_for_display_none: std::cell::Cell<bool>,
    pub(super) last_on_change_option: Option<usize>,
    pub(super) index_to_select_on_cancel: i32,
    pub(super) was_option_inserted: bool,
    pub(super) num_descendant_inputs: usize,
    pub(super) children_descendant_counts:
        std::collections::HashMap<usize, super::html_select_shadow::SelectDescendantCounts>,
    pub(super) shadow: super::html_select_shadow::SelectShadowData,
    // TreeOrderedList membership belongs to the native select class. Its order
    // is maintained on insertion; no descendant tree is copied or rescanned.
    descendant_selectedcontents: Vec<usize>,
    finished_parsing_children: bool,
}
impl std::fmt::Debug for HTMLSelectElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HTMLSelectElementData")
            .field("size", &self.size)
            .field("is_multiple", &self.is_multiple)
            .field("uses_menu_list", &self.uses_menu_list)
            .field(
                "option_style_owner",
                &std::rc::Rc::as_ptr(&self.option_style),
            )
            .field(
                "content_model_violations_count",
                &self.content_model_violations_count,
            )
            .finish_non_exhaustive()
    }
}
impl Default for HTMLSelectElementData {
    fn default() -> Self {
        Self {
            content_model_violations_count: 0,
            option_style: std::rc::Rc::new(std::cell::RefCell::new(
                foundation::Persistent::default(),
            )),
            size: 0,
            is_multiple: false,
            uses_menu_list: true,
            active_option: None,
            native_popup_is_visible: false,
            is_appearance_base_picker_for_display_none: std::cell::Cell::new(false),
            last_on_change_option: None,
            index_to_select_on_cancel: -1,
            was_option_inserted: false,
            num_descendant_inputs: 0,
            children_descendant_counts: Default::default(),
            shadow: Default::default(),
            descendant_selectedcontents: Vec::new(),
            finished_parsing_children: true,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub(super) struct HTMLSelectedContentElementData {
    disabled: bool,
    nearest_ancestor_select: Option<usize>,
}
impl PersistentDocument {
    pub fn InstallSelectSourceFeatures(&mut self, features: DOMSelectSourceFeatures) {
        assert_eq!(
            self.select_elements_allocated, 0,
            "bind select features before parser feed"
        );
        assert!(!features.selectedcontent_multiple || features.selectedcontent_spec);
        assert!(!features.selectedcontentelement_attribute || features.selectedcontent_spec);
        self.select_source_features = features;
    }
    // html_select_element.cc:98-105,436-455,1290-1316,219-235.
    pub(super) fn InitializeSelectElement(&mut self, select: usize) {
        if !matches!(self.arena[select].class, C::HTMLSelectElement(_)) {
            return;
        }
        self.select_elements_allocated += 1;
        self.RebuildSelectUserAgentShadowTree(select);
        self.attribute_lifecycle
            .RecordSelectTextStyleAndContentChanged(self.arena[select].id);
        let attributes = self.arena[select].attributes.clone();
        for attribute in attributes {
            if attribute.namespace_uri.is_empty() {
                if self.ParseSelectAttribute(select, &attribute.local_name) {
                    self.attribute_lifecycle
                        .RecordSelectTextStyleAndContentChanged(self.arena[select].id);
                }
            }
        }
    }
    pub(super) fn ParseSelectAttribute(&mut self, select: usize, name: &str) -> bool {
        if !matches!(name, "size" | "multiple") {
            return false;
        }
        let size = self.arena[select]
            .FindAttribute("size")
            .and_then(|a| Self::ParseHTMLNonNegativeInteger(&a.value))
            .unwrap_or(0);
        let size_present = self.arena[select].FindAttribute("size").is_some();
        let multiple = self.arena[select].FindAttribute("multiple").is_some();
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            return false;
        };
        let old_size = data.size;
        if name == "size" {
            data.size = size;
        } else {
            data.is_multiple = multiple;
        }
        data.uses_menu_list = if data.is_multiple {
            size_present && data.size == 1
        } else {
            data.size <= 1
        };
        let changed = name == "multiple" || old_size != size;
        if changed {
            self.RebuildSelectUserAgentShadowTree(select);
        }
        changed
    }
    pub fn SelectWasOptionInserted(&self, select: usize) -> bool {
        matches!(&self.arena[select].class, C::HTMLSelectElement(data) if data.was_option_inserted)
    }
    pub fn InputInSelectEnabled(&self) -> bool {
        self.select_source_features.input_in_select
    }
    fn FirstAncestorSelectClass(&self, node: usize) -> Option<usize> {
        let mut ancestor = self.arena[node].parent;
        while let Some(node) = ancestor {
            if matches!(self.arena[node].class, C::HTMLSelectElement(_)) {
                return Some(node);
            }
            ancestor = self.arena[node].parent;
        }
        None
    }
    fn SelectedContentAncestorFacts(&self, node: usize) -> (Option<usize>, bool) {
        let mut first = None;
        let mut disabled = false;
        let mut ancestor = self.arena[node].parent;
        while let Some(node) = ancestor {
            match self.arena[node].class {
                C::HTMLOptionElement { .. } | C::HTMLSelectedContentElement(_) => disabled = true,
                C::HTMLSelectElement(_) => {
                    if first.is_some() {
                        disabled = true;
                        break;
                    }
                    first = Some(node);
                }
                _ => {}
            }
            ancestor = self.arena[node].parent;
        }
        (first, disabled)
    }
    fn SelectedContentTreePath(&self, node: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut node = node;
        while let Some(parent) = self.arena[node].parent {
            result.push(
                self.arena[parent]
                    .children
                    .iter()
                    .position(|&child| child == node)
                    .expect("native parent/child relation"),
            );
            node = parent;
        }
        result.reverse();
        result
    }
    fn AddDescendantSelectedcontent(&mut self, select: usize, selectedcontent: usize) {
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            unreachable!()
        };
        let position = self.SelectedContentTreePath(selectedcontent);
        let existing = data
            .descendant_selectedcontents
            .iter()
            .position(|&node| node == selectedcontent);
        if self.select_source_features.selectedcontent_spec {
            assert!(existing.is_none());
        }
        // Legacy DidNotify runs again when an already-associated select subtree
        // reconnects; TreeOrderedList::Add preserves existing membership.
        let offset = existing.unwrap_or_else(|| {
            data.descendant_selectedcontents
                .partition_point(|&node| self.SelectedContentTreePath(node) < position)
        });
        if existing.is_none() {
            let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
                unreachable!()
            };
            data.descendant_selectedcontents
                .insert(offset, selectedcontent);
        }
        if !self.select_source_features.selectedcontent_spec && offset == 0 {
            self.UpdateIndividualSelectedcontent(select, selectedcontent);
            let C::HTMLSelectElement(data) = &self.arena[select].class else {
                unreachable!()
            };
            if let Some(&next) = data.descendant_selectedcontents.get(1) {
                self.ReplaceSelectedContentChildren(next, Vec::new());
            }
        }
    }
    fn RemoveDescendantSelectedcontent(&mut self, select: usize, selectedcontent: usize) {
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            unreachable!()
        };
        let offset = data
            .descendant_selectedcontents
            .iter()
            .position(|&node| node == selectedcontent)
            .expect("registered selectedcontent");
        data.descendant_selectedcontents.remove(offset);
        let next = (offset == 0)
            .then(|| data.descendant_selectedcontents.first().copied())
            .flatten();
        if !self.select_source_features.selectedcontent_spec {
            if let Some(next) = next {
                self.UpdateIndividualSelectedcontent(select, next);
            }
        }
    }
    // html_selected_content_element.cc:78-130. Called on the existing native
    // ContainerNode insertion walk before any post-connection callbacks.
    pub(super) fn SelectedContentInsertedInto(&mut self, node: usize) -> bool {
        if !matches!(self.arena[node].class, C::HTMLSelectedContentElement(_)) {
            return false;
        }
        if self.select_source_features.selectedcontent_spec {
            let (nearest, disabled) = self.SelectedContentAncestorFacts(node);
            let C::HTMLSelectedContentElement(data) = &mut self.arena[node].class else {
                unreachable!()
            };
            data.disabled = disabled;
            if data.nearest_ancestor_select != nearest {
                assert!(data.nearest_ancestor_select.is_none());
                data.nearest_ancestor_select = nearest;
                if let Some(select) = nearest {
                    self.AddDescendantSelectedcontent(select, node);
                }
            }
        }
        true
    }
    // html_selected_content_element.cc:132-155.
    pub(super) fn SelectedContentDidNotifySubtreeInsertions(&mut self, node: usize) {
        if self.select_source_features.selectedcontent_spec {
            self.SelectedContentUpdateFromAncestorSelect(node);
        } else {
            // The legacy branch calls the first select before a later select
            // disables the selectedcontent. Preserve that observable order.
            if let C::HTMLSelectedContentElement(data) = &mut self.arena[node].class {
                data.disabled = false;
            }
            let mut first = None;
            let mut ancestor = self.arena[node].parent;
            while let Some(parent) = ancestor {
                match self.arena[parent].class {
                    C::HTMLOptionElement { .. } | C::HTMLSelectedContentElement(_) => {
                        let C::HTMLSelectedContentElement(data) = &mut self.arena[node].class
                        else {
                            unreachable!()
                        };
                        data.disabled = true;
                    }
                    C::HTMLSelectElement(_) => {
                        if first.is_some() {
                            let C::HTMLSelectedContentElement(data) = &mut self.arena[node].class
                            else {
                                unreachable!()
                            };
                            data.disabled = true;
                            break;
                        }
                        first = Some(parent);
                        self.AddDescendantSelectedcontent(parent, node);
                    }
                    _ => {}
                }
                ancestor = self.arena[parent].parent;
            }
        }
    }
    // html_selected_content_element.cc:157-178. Internal descendants keep their
    // association when the entire select subtree is removed.
    pub(super) fn SelectedContentRemovedFrom(&mut self, node: usize, removed_from: usize) {
        if !matches!(self.arena[node].class, C::HTMLSelectedContentElement(_)) {
            return;
        }
        let nearest = self.FirstAncestorSelectClass(node);
        if self.select_source_features.selectedcontent_spec {
            let C::HTMLSelectedContentElement(data) = &self.arena[node].class else {
                unreachable!()
            };
            let old = data.nearest_ancestor_select;
            if old == nearest {
                return;
            }
            assert!(nearest.is_none());
            self.RemoveDescendantSelectedcontent(old.expect("previous ancestor select"), node);
            let C::HTMLSelectedContentElement(data) = &mut self.arena[node].class else {
                unreachable!()
            };
            data.nearest_ancestor_select = nearest;
            data.disabled = false;
        } else {
            if nearest.is_none() {
                if let Some(select) = self.FirstAncestorSelectClass(removed_from) {
                    self.RemoveDescendantSelectedcontent(select, node);
                }
            }
            let C::HTMLSelectedContentElement(data) = &mut self.arena[node].class else {
                unreachable!()
            };
            data.disabled = false;
        }
    }
    // html_selected_content_element.cc:190-194. The source atomic-move microtask
    // may call this same owner; an atomic-move producer is not synthesized here.
    pub fn SelectedContentUpdateFromAncestorSelect(&mut self, node: usize) {
        let C::HTMLSelectedContentElement(data) = &self.arena[node].class else {
            return;
        };
        if !data.disabled {
            if let Some(select) = data.nearest_ancestor_select {
                self.UpdateIndividualSelectedcontent(select, node);
            }
        }
    }
    fn ReplaceSelectedContentChildren(&mut self, target: usize, children: Vec<usize>) {
        let old = self.arena[target].children.clone();
        for child in old {
            self.Remove(child);
        }
        for child in children {
            self.AppendChild(target, child);
        }
    }
    // html_selected_content_element.cc:19-48. Clone actual descendants and
    // their UTF-16 node payload through the canonical Node::cloneNode owner.
    fn CloneSelectedContentFromOption(&mut self, target: usize, option: Option<usize>) {
        let C::HTMLSelectedContentElement(data) = &self.arena[target].class else {
            return;
        };
        if data.disabled {
            return;
        }
        let mut clones = Vec::new();
        if let Some(option) = option {
            assert!(matches!(
                self.arena[option].class,
                C::HTMLOptionElement { .. }
            ));
            assert!(self
                .OptionOwnerSelect(option)
                .is_none_or(|select| !self.SelectIsMultiple(select)));
            for child in self.arena[option].children.clone() {
                clones.push(self.CloneNode(child, true));
            }
        }
        self.ReplaceSelectedContentChildren(target, clones);
    }
    // html_selected_content_element.cc:50-76.
    fn CloneSelectedContentFromMultiple(&mut self, target: usize, select: usize) {
        assert!(
            self.select_source_features.selectedcontent_multiple && self.SelectIsMultiple(select)
        );
        let C::HTMLSelectedContentElement(data) = &self.arena[target].class else {
            return;
        };
        if data.disabled {
            return;
        }
        let mut clones = Vec::new();
        for id in self.SelectOptionNodeIds(select) {
            let option = self.FindNodeById(id).expect("native collection member");
            if !self.OptionSelected(option) {
                continue;
            }
            let container = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
            for child in self.arena[option].children.clone() {
                let clone = self.CloneNode(child, true);
                self.AppendChild(container, clone);
            }
            clones.push(container);
        }
        self.ReplaceSelectedContentChildren(target, clones);
    }
    // html_select_element.cc:2068-2088. This is GetElementAttribute, without
    // reference-target retargeting; external selectedcontent retains identity.
    pub fn SelectSelectedContentElement(&self, select: usize) -> Option<usize> {
        if !self.select_source_features.selectedcontentelement_attribute {
            return None;
        }
        self.GetElementAttribute(select, "selectedcontentelement")
            .filter(|&node| matches!(self.arena[node].class, C::HTMLSelectedContentElement(_)))
    }
    pub fn SetSelectSelectedContentElement(&mut self, select: usize, target: Option<usize>) {
        if !self.select_source_features.selectedcontentelement_attribute {
            return;
        }
        if let Some(node) = target {
            assert!(matches!(
                self.arena[node].class,
                C::HTMLSelectedContentElement(_)
            ));
        }
        let old = self.SelectSelectedContentElement(select);
        self.SetElementAttribute(select, "selectedcontentelement", target);
        if old != target {
            if let Some(target) = target {
                self.UpdateIndividualSelectedcontent(select, target);
            }
        }
    }
    // html_select_element.cc:2090-2128. Snapshot only the source enabled list
    // before clones can themselves perform native insertion/removal callbacks.
    pub fn SelectUpdateAllSelectedcontentsSingle(&mut self, select: usize, option: Option<usize>) {
        assert!(!self.SelectIsMultiple(select));
        debug_assert_eq!(option, self.FirstSelectedOption(select));
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            return;
        };
        let targets = if self.select_source_features.selectedcontent_spec {
            data.descendant_selectedcontents.iter().copied().filter(|&node| {
                matches!(&self.arena[node].class, C::HTMLSelectedContentElement(data) if !data.disabled)
            }).collect::<Vec<_>>()
        } else {
            data.descendant_selectedcontents
                .first()
                .copied()
                .into_iter()
                .collect()
        };
        for target in targets {
            self.CloneSelectedContentFromOption(target, option);
        }
        if let Some(target) = self.SelectSelectedContentElement(select) {
            self.CloneSelectedContentFromOption(target, option);
        }
    }
    // html_select_element.cc:2130-2141.
    pub fn SelectUpdateAllSelectedcontentsMultiple(&mut self, select: usize) {
        assert!(self.SelectIsMultiple(select));
        if !self.select_source_features.selectedcontent_multiple {
            return;
        }
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            return;
        };
        let targets = data.descendant_selectedcontents.clone();
        for target in targets {
            self.CloneSelectedContentFromMultiple(target, select);
        }
    }
    fn UpdateIndividualSelectedcontent(&mut self, select: usize, target: usize) {
        if self.SelectIsMultiple(select) {
            if self.select_source_features.selectedcontent_multiple {
                self.CloneSelectedContentFromMultiple(target, select);
            }
        } else {
            self.CloneSelectedContentFromOption(target, self.FirstSelectedOption(select));
        }
    }
    // html_option_element.cc:636-645,923-931.
    pub(super) fn OptionDidNotifySubtreeInsertions(&mut self, option: usize) {
        if !self.select_source_features.selectedcontent_spec || !self.OptionSelected(option) {
            return;
        }
        if let Some(select) = self.OptionOwnerSelect(option) {
            if self.SelectIsMultiple(select) {
                self.SelectUpdateAllSelectedcontentsMultiple(select);
            } else {
                self.SelectUpdateAllSelectedcontentsSingle(select, Some(option));
            }
        }
    }
    // Node/Element parser lifecycle. Script-created elements begin finished;
    // parser insertion starts parsing AFTER attachment and finishes on pop.
    pub fn BeginParsingChildren(&mut self, element: usize) {
        self.RenderBlockingBeginParsingChildren(element);
        if self.IsMediaElement(element) {
            self.MediaElementState(element)
                .finished_parsing_children
                .set(false);
        }
        if let C::HTMLSelectElement(data) = &mut self.arena[element].class {
            data.finished_parsing_children = false;
        }
    }
    pub fn FinishParsingChildren(&mut self, element: usize) {
        self.RenderBlockingFinishParsingChildren(element);
        if self.IsMediaElement(element) {
            self.MediaElementState(element)
                .finished_parsing_children
                .set(true);
            self.attribute_lifecycle
                .RecordMediaFinishedParsing(self.arena[element].id);
        }
        // HTMLFormControlElementWithState.cc:354-357 / ListedElement.cc:760-765;
        // HTMLFormElement.cc:1616-1619. The inherited stage precedes select's
        // selectedcontent and SelectType subclass stages.
        if self.ClassSupportsFormStateRestore(element) {
            self.attribute_lifecycle.RecordFormStateFinishedParsing(
                self.arena[element].id,
                crate::attribute_lifecycle::FormStateParsingKind::Control,
                self.ParsingState()
                    == crate::document_parser_lifecycle::DocumentParsingState::Finished,
            );
        } else if matches!(self.arena[element].class, C::HTMLFormElement) {
            self.attribute_lifecycle.RecordFormStateFinishedParsing(
                self.arena[element].id,
                crate::attribute_lifecycle::FormStateParsingKind::Form,
                self.ParsingState()
                    == crate::document_parser_lifecycle::DocumentParsingState::Finished,
            );
        }
        if let C::HTMLSelectElement(data) = &mut self.arena[element].class {
            data.finished_parsing_children = true;
            if self.SelectIsMultiple(element)
                && self.select_source_features.selectedcontent_multiple
            {
                self.SelectUpdateAllSelectedcontentsMultiple(element);
            }
            if !self.SelectUsesMenuList(element) {
                self.attribute_lifecycle
                    .RecordSelectFinishedParsingChildren(self.arena[element].id);
            }
        } else if matches!(self.arena[element].class, C::HTMLOptionElement { .. })
            && self.OptionSelected(element)
        {
            if let Some(select) = self.OptionOwnerSelect(element) {
                if !self.SelectIsMultiple(select) {
                    self.SelectUpdateAllSelectedcontentsSingle(select, Some(element));
                }
            }
        }
    }

    pub fn SelectIsFinishedParsingChildren(&self, select: usize) -> bool {
        matches!(&self.arena[select].class, C::HTMLSelectElement(data) if data.finished_parsing_children)
    }
}
