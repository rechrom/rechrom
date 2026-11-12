//! HTMLSelectElement / SelectType native UA shadow and manual slot owners.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]
use super::{
    editing_node::DOMNodeClass as C, html_select_owner::DOMSelectUAPartKind as K, DOMAttribute,
    DOMNamespace, DOMNodeType, DOMShadowRootMode, DOMSlotAssignmentMode, PersistentDocument,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SelectRelatedParts {
    select: usize,
    child: usize,
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SelectDescendantCounts {
    inputs: usize,
    options: usize,
}
#[derive(Clone, Debug, Default)]
pub(super) struct SelectShadowData {
    inner: Option<usize>,
    button_slot: Option<usize>,
    options_slot: Option<usize>,
    picker: Option<usize>,
    input_slot: Option<usize>,
    listbox: Option<usize>,
    preview: Option<usize>,
    preview_text: Option<usize>,
}
impl PersistentDocument {
    pub fn SelectIsInDialogMode(&self, select: usize) -> bool {
        if !self.SelectHasBaseAppearance(select) {
            return false;
        }
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            unreachable!("native select")
        };
        self.select_source_features.filterable_select && data.num_descendant_inputs != 0
            || data.content_model_violations_count != 0
    }
    pub fn SelectPickerAppearanceState(&self, picker: usize) -> Option<bool> {
        match self.arena[picker].class {
            C::HTMLSelectUAPart {
                kind: K::Picker,
                appearance_state,
                ..
            } => appearance_state,
            _ => unreachable!("native picker"),
        }
    }
    pub fn SetSelectPickerAppearanceState(&mut self, picker: usize, value: Option<bool>) {
        let C::HTMLSelectUAPart {
            kind: K::Picker,
            appearance_state,
            ..
        } = &mut self.arena[picker].class
        else {
            unreachable!("native picker")
        };
        *appearance_state = value;
    }
    pub fn SelectContentModelViolationsCount(&self, select: usize) -> u32 {
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            unreachable!("native select")
        };
        data.content_model_violations_count
    }
    // HTMLSelectElement cc:849-873, retained on its original native class.
    pub fn ChangeSelectContentModelViolationCount(
        &mut self,
        select: usize,
        increase: bool,
    ) -> bool {
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            unreachable!("native select")
        };
        if increase {
            let changed = data.content_model_violations_count == 0;
            data.content_model_violations_count += 1;
            changed
        } else {
            let changed = data.content_model_violations_count == 1;
            if data.content_model_violations_count > 0 {
                data.content_model_violations_count -= 1;
            }
            changed
        }
    }
    pub fn SelectContentNodeKind(
        &self,
        node: usize,
    ) -> foundation::select_content_types::SelectContentNodeKind {
        use foundation::select_content_types::SelectContentNodeKind as K;
        // CDATASection retains the Text node type and is distinguished by its
        // dynamic DOMNodeClass, exactly like Chromium's Text subclass.
        if self.arena[node].node_type == DOMNodeType::kText {
            return K::Text;
        }
        if self.arena[node].node_type == DOMNodeType::kComment {
            return K::Comment;
        }
        if self.arena[node].node_type != DOMNodeType::kElement {
            return K::Other;
        }
        if self.arena[node].Namespace() == DOMNamespace::kSVG {
            return K::SVG;
        }
        if self.arena[node].Namespace() != DOMNamespace::kHTML {
            return K::Other;
        }
        match self.arena[node].class {
            C::HTMLSelectElement(_) => K::Select,
            C::HTMLOptionElement { .. } => K::Option,
            C::HTMLSelectedContentElement(_) => K::SelectedContent,
            C::HTMLOptGroupElement => K::OptGroup,
            C::HTMLButtonElement => K::Button,
            C::HTMLDivElement | C::HTMLSelectUAPart { .. } => K::Div,
            C::HTMLSpanElement => K::Span,
            C::HTMLNoScriptElement => K::NoScript,
            C::ScriptElement => K::Script,
            C::HTMLTemplateElement => K::Template,
            C::HTMLLegendElement => K::Legend,
            C::HTMLHRElement => K::HR,
            C::HTMLDataListElement(_) => K::DataList,
            C::HTMLObjectElement(_) => K::Object,
            _ => K::HTML,
        }
    }
    pub fn SelectRemoveOverflowHiddenEnabled(&self) -> bool {
        self.select_source_features.select_remove_overflow_hidden
    }
    pub fn InstallSelectOptionStyleOwner(
        &mut self,
        select: usize,
    ) -> layoutng_style::style::menu_list_inner_style::MenuListOptionStyleOwner {
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            unreachable!("native select")
        };
        data.option_style =
            std::rc::Rc::new(std::cell::RefCell::new(foundation::Persistent::default()));
        data.option_style.clone()
    }
    pub fn SelectOptionStyleOwner(
        &self,
        select: usize,
    ) -> layoutng_style::style::menu_list_inner_style::MenuListOptionStyleOwner {
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            unreachable!("native select")
        };
        data.option_style.clone()
    }
    pub fn OptionWasElementInserted(&self, option: usize) -> bool {
        matches!(
            self.arena[option].class,
            C::HTMLOptionElement {
                was_element_inserted: true,
                ..
            }
        )
    }
    // html_option_element.cc:432-444. This historic bit never resets when the
    // inserted element is removed; future descendant edits still need observing.
    pub(super) fn OptionChildrenChanged(&mut self, option: usize) {
        if !matches!(self.arena[option].class, C::HTMLOptionElement { .. }) {
            return;
        }
        self.OptionDidChangeTextContent(option);
        if self.OptionWasElementInserted(option) {
            return;
        }
        if self.arena[option]
            .Children()
            .iter()
            .any(|&node| self.arena[node].node_type == DOMNodeType::kElement)
        {
            let C::HTMLOptionElement {
                was_element_inserted,
                ..
            } = &mut self.arena[option].class
            else {
                unreachable!()
            };
            *was_element_inserted = true;
            self.attribute_lifecycle
                .RecordOptionMutationObserverChanged(self.arena[option].id, false);
        }
    }
    // html_option_element.cc:95-98,572-590.
    pub(super) fn EnsureOptionUserAgentShadowTree(&mut self, option: usize) {
        let C::HTMLOptionElement {
            label_container, ..
        } = &self.arena[option].class
        else {
            return;
        };
        if label_container.is_some() {
            return;
        }
        let root = self
            .ShadowRoot(option)
            .unwrap_or_else(|| self.AttachShadowRoot(option, DOMShadowRootMode::kUserAgent));
        let label = self.CreateElementDefault(DOMNamespace::kHTML, "span".into());
        self.SetUAShadowPseudoElementIdentity(option, label, "-internal-option-label-container");
        self.SetSelectShadowAttribute(label, "aria-hidden", "true");
        self.AppendChild(root, label);
        let slot = self.CreateElementDefault(DOMNamespace::kHTML, "slot".into());
        self.SetUAShadowPseudoElementIdentity(option, slot, "-internal-option-slot");
        self.AppendChild(root, slot);
        let C::HTMLOptionElement {
            label_container, ..
        } = &mut self.arena[option].class
        else {
            unreachable!()
        };
        *label_container = Some(label);
        self.UpdateOptionShadowLabel(option);
    }
    pub fn OptionLabelContainerElement(&self, option: usize) -> Option<usize> {
        match self.arena[option].class {
            C::HTMLOptionElement {
                label_container, ..
            } => label_container,
            _ => None,
        }
    }
    fn UpdateOptionShadowLabel(&mut self, option: usize) {
        if let Some(label) = self.OptionLabelContainerElement(option) {
            let text = self.OptionDisplayLabelUTF16(option);
            self.SetTextContentUTF16(label, &text);
        }
    }
    // html_option_element.cc:432-444,496-504. Direct ChildrenChanged invokes
    // this immediately; OptionTextObserver invokes the same owner on delivery
    // for descendant mutations, preserving its separate microtask semantics.
    pub fn OptionDidChangeTextContent(&mut self, option: usize) {
        if !matches!(self.arena[option].class, C::HTMLOptionElement { .. }) {
            return;
        }
        if self.OptionSelected(option) {
            if let Some(select) = self.OptionOwnerSelect(option) {
                self.attribute_lifecycle
                    .RecordSelectTextStyleAndContentChanged(self.arena[select].id);
            }
        }
        if let Some(select) = self.OptionOwnerSelect(option) {
            self.SetControlNeedsValidityCheck(select);
        }
        self.UpdateOptionShadowLabel(option);
    }
    fn CreateSelectShadowElement(
        &mut self,
        select: usize,
        tag: &str,
        kind: Option<K>,
        pseudo: Option<&str>,
    ) -> usize {
        let node = self.CreateElementDefault(DOMNamespace::kHTML, tag.into());
        if let Some(kind) = kind {
            self.arena[node].class = C::HTMLSelectUAPart {
                select,
                kind,
                appearance_state: None,
            };
        }
        if let Some(pseudo) = pseudo {
            self.SetUAShadowPseudoElementIdentity(select, node, pseudo);
        }
        node
    }
    fn SetSelectShadowAttribute(&mut self, node: usize, name: &str, value: &str) {
        self.SetAttribute(
            node,
            DOMAttribute {
                local_name: name.into(),
                value: value.into(),
                ..Default::default()
            },
        );
    }
    // html_select_element.cc:1594-1613. Constructor creates a default MenuList
    // first; parser attributes then rebuild the same root in source order.
    pub fn RebuildSelectUserAgentShadowTree(&mut self, select: usize) {
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            return;
        };
        let old = std::mem::take(&mut data.shadow);
        let menu = data.uses_menu_list;
        let has_inputs = data.num_descendant_inputs != 0;
        assert!(!has_inputs || self.select_source_features.filterable_select);
        let root = self.ShadowRoot(select).unwrap_or_else(|| {
            self.AttachShadowRootWithSlotAssignment(
                select,
                DOMShadowRootMode::kUserAgent,
                DOMSlotAssignmentMode::kManual,
            )
        });
        assert_eq!(
            self.ShadowRootMode(root),
            Some(DOMShadowRootMode::kUserAgent)
        );
        assert_eq!(
            self.ShadowRootSlotAssignment(root),
            Some(DOMSlotAssignmentMode::kManual)
        );
        for slot in [old.button_slot, old.options_slot, old.input_slot]
            .into_iter()
            .flatten()
        {
            self.AssignSlotNodes(slot, Vec::new());
        }
        for node in [
            old.inner,
            old.button_slot,
            old.picker,
            old.listbox,
            old.preview,
            old.preview_text,
        ]
        .into_iter()
        .flatten()
        {
            self.RemoveUAShadowPseudoElementIdentity(select, node);
        }
        for child in self.arena[root].children.clone() {
            self.Remove(child);
        }
        let mut shadow = SelectShadowData::default();
        // MenuListSelectType::CreateShadowSubtree, select_type.cc:593-652.
        let options_parent = if menu {
            let inner = self.CreateSelectShadowElement(
                select,
                "div",
                Some(K::MenuListInner),
                Some("-internal-select-inner-element"),
            );
            self.SetSelectShadowAttribute(inner, "aria-hidden", "true");
            let text = self.CreateText(String::new());
            self.AppendChild(inner, text);
            self.AppendChild(root, inner);
            shadow.inner = Some(inner);
            let button_slot = self.CreateSelectShadowElement(
                select,
                "slot",
                None,
                Some("-internal-select-button-slot"),
            );
            self.AppendChild(root, button_slot);
            shadow.button_slot = Some(button_slot);
            let picker = self.CreateSelectShadowElement(
                select,
                "div",
                Some(K::Picker),
                Some("picker(select)"),
            );
            self.AppendChild(root, picker);
            self.SetSelectShadowAttribute(picker, "popover", "auto");
            shadow.picker = Some(picker);
            picker
        } else {
            root
        };
        let options = self.CreateSelectShadowElement(select, "slot", None, None);
        self.SetSelectShadowAttribute(options, "id", "select-options");
        shadow.options_slot = Some(options);
        if menu && !self.select_source_features.select_autofill_popover_preview {
            self.CreateSelectAutofillPopover(select, root, &mut shadow);
        }
        // ListBoxSelectType::CreateShadowSubtree, select_type.cc:1948-1973.
        if has_inputs {
            let input = self.CreateSelectShadowElement(select, "slot", None, None);
            self.SetSelectShadowAttribute(input, "id", "select-input");
            self.AppendChild(options_parent, input);
            shadow.input_slot = Some(input);
            let listbox =
                self.CreateSelectShadowElement(select, "div", None, Some("select-listbox"));
            self.SetSelectShadowAttribute(listbox, "role", "listbox");
            self.AppendChild(options_parent, listbox);
            self.AppendChild(listbox, options);
            shadow.listbox = Some(listbox);
        } else {
            self.AppendChild(options_parent, options);
        }
        if !menu || self.select_source_features.select_autofill_popover_preview {
            self.CreateSelectAutofillPopover(select, root, &mut shadow);
        }
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            unreachable!()
        };
        data.shadow = shadow;
        self.AssignSelectSlots(select);
    }
    // select_type.cc:2083-2102. Nodes and implicit-anchor identity are real;
    // popup visibility and custom style remain on their original owners.
    fn CreateSelectAutofillPopover(
        &mut self,
        select: usize,
        root: usize,
        shadow: &mut SelectShadowData,
    ) {
        if !self.select_source_features.select_autofill_popover_preview
            && !self.SelectHasBaseAppearance(select)
        {
            return;
        }
        self.SelectSetMayBeImplicitAnchor(select);
        let preview = self.CreateSelectShadowElement(
            select,
            "div",
            Some(K::AutofillPreview),
            Some("-internal-select-autofill-preview"),
        );
        self.AppendChild(root, preview);
        self.SetSelectShadowAttribute(preview, "popover", "manual");
        let text = self.CreateSelectShadowElement(
            select,
            "div",
            None,
            Some("-internal-select-autofill-preview-text"),
        );
        self.AppendChild(preview, text);
        shadow.preview = Some(preview);
        shadow.preview_text = Some(text);
    }
    pub fn SelectHasBaseAppearance(&self, select: usize) -> bool {
        self.ResolvedStyleFor(select)
            .and_then(|style| style.ComputedStyle())
            .is_some_and(|style| {
                self.SelectSupportsBaseAppearance(select, style.EffectiveAppearance())
            })
    }
    // HTMLSelectElement::SupportsBaseAppearanceInternal cc:2324-2337.
    pub fn SelectSupportsBaseAppearance(
        &self,
        select: usize,
        appearance: foundation::AppearanceValue,
    ) -> bool {
        use foundation::AppearanceValue;
        match appearance {
            AppearanceValue::kBaseSelect => (),
            AppearanceValue::kBase if self.select_source_features.appearance_base => (),
            _ => return false,
        }
        self.select_source_features
            .customizable_select_multiple_popup
            || !(self.SelectUsesMenuList(select) && self.SelectIsMultiple(select))
    }

    pub fn SelectSetMayBeImplicitAnchor(&mut self, select: usize) {
        assert!(matches!(self.arena[select].class, C::HTMLSelectElement(_)));
        self.popup_lifecycle_state
            .may_be_implicit_anchors
            .insert(self.arena[select].id);
    }
    pub fn SelectUAPart(&self, node: usize) -> Option<(usize, K)> {
        match self.arena[node].class {
            C::HTMLSelectUAPart { select, kind, .. } => Some((select, kind)),
            _ => None,
        }
    }
    pub fn SelectListboxElement(&self, select: usize) -> Option<usize> {
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            unreachable!("native select")
        };
        data.shadow.listbox
    }
    pub fn SelectInnerElement(&self, select: usize) -> Option<usize> {
        match &self.arena[select].class {
            C::HTMLSelectElement(data) => data.shadow.inner,
            _ => None,
        }
    }
    pub fn SelectPickerElement(&self, select: usize) -> Option<usize> {
        match &self.arena[select].class {
            C::HTMLSelectElement(data) => data.shadow.picker,
            _ => None,
        }
    }
    pub fn SelectAutofillPreviewElement(&self, select: usize) -> Option<usize> {
        match &self.arena[select].class {
            C::HTMLSelectElement(data) => data.shadow.preview,
            _ => None,
        }
    }
    pub fn SelectAutofillPreviewTextElement(&self, select: usize) -> Option<usize> {
        match &self.arena[select].class {
            C::HTMLSelectElement(data) => data.shadow.preview_text,
            _ => None,
        }
    }
    pub fn SelectSlottedButton(&self, select: usize) -> Option<usize> {
        if !self.SelectUsesMenuList(select) {
            return None;
        }
        self.arena[select]
            .children
            .iter()
            .copied()
            .find(|&node| self.arena[node].node_type == DOMNodeType::kElement)
            .filter(|&node| matches!(self.arena[node].class, C::HTMLButtonElement))
    }
    // select_type.cc:654-715,1975-2006. Only manual assignment's existing
    // forward/reverse owner is mutated; flat traversal consumes those maps.
    pub(super) fn AssignSelectSlots(&mut self, select: usize) {
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            return;
        };
        let Some(options_slot) = data.shadow.options_slot else {
            return;
        };
        let button_slot = data.shadow.button_slot;
        let input_slot = data.shadow.input_slot;
        let has_inputs = data.num_descendant_inputs != 0;
        let mut button = None;
        let mut after_first_element = false;
        let mut options = Vec::new();
        let mut inputs = Vec::new();
        for &child in &self.arena[select].children {
            let slotable = matches!(
                self.arena[child].node_type,
                DOMNodeType::kElement | DOMNodeType::kText
            );
            if button_slot.is_some() {
                if !slotable {
                    continue;
                }
                if !after_first_element && self.arena[child].node_type == DOMNodeType::kElement {
                    after_first_element = true;
                    if matches!(self.arena[child].class, C::HTMLButtonElement) {
                        button = Some(child);
                        continue;
                    }
                }
            }
            if has_inputs
                && (self.arena[child].class.IsHTMLInputElement()
                    || data
                        .children_descendant_counts
                        .get(&child)
                        .is_some_and(|counts| counts.inputs != 0 && counts.options == 0))
            {
                inputs.push(child);
                continue;
            }
            if button_slot.is_some()
                || slotable
                    && matches!(
                        self.arena[child].class,
                        C::HTMLOptionElement { .. }
                            | C::HTMLOptGroupElement
                            | C::HTMLHRElement
                            | C::HTMLSpanElement
                            | C::HTMLDivElement
                            | C::HTMLSelectUAPart { .. }
                            | C::HTMLAltTextContainerElement
                    )
            {
                options.push(child);
            }
        }
        if let Some(slot) = button_slot {
            let had_button = self
                .manually_assigned_nodes
                .get(&slot)
                .is_some_and(|nodes| !nodes.is_empty());
            if had_button != button.is_some() {
                self.attribute_lifecycle
                    .RecordSelectSlottedButtonChanged(self.arena[select].id);
            }
            self.AssignSlotNodes(slot, button.into_iter().collect());
        }
        self.AssignSlotNodes(options_slot, options);
        if let Some(slot) = input_slot {
            self.AssignSlotNodes(slot, inputs);
        }
    }
    pub fn InputOwnerSelectForFiltering(&self, input: usize) -> Option<usize> {
        if !self.arena[input].class.IsHTMLInputElement() {
            return None;
        }
        self.arena[input]
            .select_related_parts
            .map(|parts| parts.select)
    }
    pub fn OptionOwnerDataList(&self, option: usize) -> Option<usize> {
        if !self.select_source_features.customizable_combobox {
            let mut ancestor = self.arena[option].parent;
            while let Some(node) = ancestor {
                if matches!(self.arena[node].class, C::HTMLDataListElement(_)) {
                    return Some(node);
                }
                ancestor = self.arena[node].parent;
            }
            return None;
        }
        match self.arena[option].class {
            C::HTMLOptionElement {
                nearest_ancestor_datalist,
                ..
            } => nearest_ancestor_datalist,
            _ => None,
        }
    }
    // html_select_element.cc:2157-2195. The stored association belongs to each
    // input/option; ancestor traversal is only done at its native lifecycle.
    fn SelectRelatedPartsFor(&self, element: usize) -> (Option<SelectRelatedParts>, Option<usize>) {
        let mut optgroup = false;
        let mut child = element;
        let mut ancestor = self.arena[element].parent;
        while let Some(node) = ancestor {
            match self.arena[node].class {
                C::HTMLOptionElement { .. } | C::HTMLHRElement => return (None, None),
                C::HTMLDataListElement(_) => return (None, Some(node)),
                C::HTMLOptGroupElement => {
                    if optgroup || matches!(self.arena[element].class, C::HTMLOptGroupElement) {
                        return (None, None);
                    }
                    optgroup = true;
                }
                C::HTMLSelectElement(_) => {
                    return (
                        Some(SelectRelatedParts {
                            select: node,
                            child,
                        }),
                        None,
                    );
                }
                _ => {}
            }
            child = node;
            ancestor = self.arena[node].parent;
        }
        (None, None)
    }
    pub(super) fn SelectCountedElementInserted(&mut self, node: usize) {
        let input = self.arena[node].class.IsHTMLInputElement();
        let option = matches!(self.arena[node].class, C::HTMLOptionElement { .. });
        let optgroup = matches!(self.arena[node].class, C::HTMLOptGroupElement);
        if !input && !option && !optgroup {
            return;
        }
        let related = if option || optgroup || self.select_source_features.filterable_select {
            self.SelectRelatedPartsFor(node)
        } else {
            (None, None)
        };
        if option || optgroup {
            self.arena[node].select_owner = related.0.map(|p| p.select);
        }
        if optgroup {
            return;
        }
        if option {
            if let C::HTMLOptionElement {
                nearest_ancestor_datalist,
                ..
            } = &mut self.arena[node].class
            {
                *nearest_ancestor_datalist = related.1;
            }
            if let Some(parts) = related.0 {
                let select = parts.select;
                let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
                    unreachable!()
                };
                data.was_option_inserted = true;
            }
        }
        if !self.select_source_features.filterable_select {
            return;
        }
        let new = related.0;
        let old = self.arena[node].select_related_parts;
        if old.map(|p| p.select) == new.map(|p| p.select) {
            return;
        }
        assert!(old.is_none());
        self.arena[node].select_related_parts = new;
        let Some(parts) = new else {
            return;
        };
        let C::HTMLSelectElement(data) = &mut self.arena[parts.select].class else {
            unreachable!()
        };
        if parts.child != node {
            let counts = data
                .children_descendant_counts
                .entry(parts.child)
                .or_default();
            if input {
                counts.inputs += 1;
            } else {
                counts.options += 1;
            }
        }
        if input {
            data.num_descendant_inputs += 1;
        }
        let rebuild = input && data.num_descendant_inputs == 1;
        if rebuild {
            self.RebuildSelectUserAgentShadowTree(parts.select);
            self.InvalidateNodeStyle(parts.select);
        } else {
            self.AssignSelectSlots(parts.select);
        }
    }
    // html_select_element.cc:2208-2284; input_element.cc:1988-1996.
    pub(super) fn SelectCountedElementRemoved(&mut self, node: usize) {
        if matches!(self.arena[node].class, C::HTMLSelectElement(_)) {
            self.attribute_lifecycle
                .RecordSelectMutationObserverChanged(self.arena[node].id);
        }
        let option = matches!(self.arena[node].class, C::HTMLOptionElement { .. });
        let optgroup = matches!(self.arena[node].class, C::HTMLOptGroupElement);
        let related = if option || optgroup || self.arena[node].select_related_parts.is_some() {
            self.SelectRelatedPartsFor(node)
        } else {
            (None, None)
        };
        if option || optgroup {
            self.arena[node].select_owner = related.0.map(|p| p.select);
        }
        if optgroup {
            return;
        }
        if let C::HTMLOptionElement {
            nearest_ancestor_datalist,
            ..
        } = &mut self.arena[node].class
        {
            *nearest_ancestor_datalist = related.1;
        }
        let Some(old) = self.arena[node].select_related_parts else {
            return;
        };
        let new = related.0;
        if new.map(|p| p.select) == Some(old.select) {
            return;
        }
        assert!(new.is_none());
        self.arena[node].select_related_parts = None;
        let input = self.arena[node].class.IsHTMLInputElement();
        let C::HTMLSelectElement(data) = &mut self.arena[old.select].class else {
            unreachable!()
        };
        if old.child != node {
            let counts = data
                .children_descendant_counts
                .get_mut(&old.child)
                .expect("source counted child");
            if input {
                counts.inputs = counts.inputs.checked_sub(1).expect("counted input");
            } else {
                counts.options = counts.options.checked_sub(1).expect("counted option");
            }
            if counts.inputs == 0 && counts.options == 0 {
                data.children_descendant_counts.remove(&old.child);
            }
        }
        if input {
            data.num_descendant_inputs = data
                .num_descendant_inputs
                .checked_sub(1)
                .expect("counted input");
        }
        let rebuild = input && data.num_descendant_inputs == 0;
        if rebuild {
            self.RebuildSelectUserAgentShadowTree(old.select);
            self.InvalidateNodeStyle(old.select);
        } else {
            self.AssignSelectSlots(old.select);
        }
    }
}
