//! Live form/control projections shared by bindings and default handlers.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: core/html/forms/html_form_element.cc:1302-1379,1400-1428;
//! html_select_element.cc:709-776,2339-2364;
//! html_option_element.cc:386-412.
use super::{DOMNamespace, PersistentDocument};
use std::{collections::HashMap, rc::Rc};

/// Native listed/control class selected by the DOM allocation owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMFormControlClass {
    Input,
    TextArea,
    Select,
    Button,
    Fieldset,
    Output,
    Object,
}

#[derive(Default)]
pub(super) struct FormCollectionState {
    forms: HashMap<u64, (u64, Rc<Vec<u64>>)>,
    selects: HashMap<u64, (u64, Rc<DOMSelectCollection>)>,
    datalists: HashMap<u64, (u64, Rc<Vec<u64>>)>,
}

/// HTMLSelectElement's source ListItems/GetOptionList share this retained DOM
/// collection. Consumers index the same source owner rather than cloning a
/// document tree or rebuilding the list for every NextValidOption step.
pub struct DOMSelectCollection {
    list_items: Vec<u64>,
    options: Vec<u64>,
}
impl DOMSelectCollection {
    pub fn ListItemNodeIds(&self) -> &[u64] {
        &self.list_items
    }
    pub fn OptionNodeIds(&self) -> &[u64] {
        &self.options
    }
}

impl PersistentDocument {
    /// Element's selector virtuals read the same ListedElement cache prepared
    /// by the document lifecycle and synchronous selector-query boundary.
    pub fn MatchesValidityPseudoClasses(&self, node: usize) -> bool {
        self.arena[node].IsHTMLElement("form")
            || self.arena[node].IsHTMLElement("fieldset")
            || self.IsFormAssociatedCustomElement(node)
            || (self.IsFormControlElement(node) && self.ControlWillValidate(node))
    }
    pub fn IsValidElementForStyle(&self, node: usize) -> bool {
        if self.arena[node].IsHTMLElement("form") {
            return self.ListedFormControlNodeIds(node).iter().all(|&id| {
                self.FindNodeById(id)
                    .is_none_or(|i| self.ControlIsNotCandidateOrValid(i))
            });
        }
        if self.arena[node].IsHTMLElement("fieldset") {
            fn visit(d: &PersistentDocument, node: usize) -> bool {
                d.arena[node].children.iter().all(|&child| {
                    (!(d.IsFormControlElement(child) || d.IsFormAssociatedCustomElement(child))
                        || d.ControlIsNotCandidateOrValid(child))
                        && visit(d, child)
                })
            }
            return visit(self, node);
        }
        self.ControlIsNotCandidateOrValid(node)
    }
    fn ControlIsNotCandidateOrValid(&self, node: usize) -> bool {
        !self.ControlWillValidate(node)
            || if self.IsFormAssociatedCustomElement(node) {
                self.ElementInternalsValidityFlags(node) == 0
            } else {
                // Native allocation constructs this ListedElement owner.
                // Its pending work is completed before the immutable Style loan.
                self.text_control_runtime[&self.arena[node].id].cached_validity
            }
    }
    pub fn MatchesUserValidityPseudoClass(&self, node: usize, valid: bool) -> bool {
        matches!(
            self.FormControlClass(node),
            Some(
                DOMFormControlClass::Input
                    | DOMFormControlClass::TextArea
                    | DOMFormControlClass::Select
            )
        ) && self.ControlUserValidityIsExposed(node)
            && self.MatchesValidityPseudoClasses(node)
            && self.IsValidElementForStyle(node) == valid
    }
    pub fn ClassSupportsFormStateRestore(&self, node: usize) -> bool {
        self.IsFormAssociatedCustomElement(node)
            || matches!(
                self.FormControlClass(node),
                Some(
                    DOMFormControlClass::Input
                        | DOMFormControlClass::TextArea
                        | DOMFormControlClass::Select
                )
            )
    }
    pub fn StatefulFormControlRevision(&self) -> u64 {
        self.stateful_form_control_revision
    }
    pub fn ControlUserHasEditedTheField(&self, node: usize) -> bool {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .is_some_and(|s| s.user_interaction >= 1)
    }
    /// ElementTraversal::Next in the document's ordinary DOM tree. Shadow
    /// roots are separate trees and therefore are not visited here.
    pub fn NextFormStateElement(&self, node: usize) -> Option<usize> {
        let mut current = node;
        loop {
            let mut next = self.arena[current].children.first().copied();
            if next.is_none() {
                let mut ancestor = Some(current);
                while let Some(index) = ancestor {
                    if index == self.Root() {
                        return None;
                    }
                    if let Some(sibling) = self.arena[index].next_sibling {
                        next = Some(sibling);
                        break;
                    }
                    ancestor = self.arena[index].parent;
                }
            }
            current = next?;
            if self.arena[current].node_type == super::DOMNodeType::kElement {
                return Some(current);
            }
        }
    }
    pub fn FormSignatureActionURL(&self, form: usize) -> Rc<[u16]> {
        // Element::GetURLAttributeAsKURL cc11639-11642; FormSignature cc307-322.
        let Some(attribute) = self.arena[form]
            .FindAttribute("action")
            .map(|a| a.value.as_str())
        else {
            return Rc::from([]);
        };
        let attribute = attribute.trim_matches(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' '));
        let mut action = self.BaseURL().CompleteURL(attribute);
        if let Some(action) = action.as_mut() {
            action.set_query(None);
        }
        action.map_or_else(
            || Rc::from([]),
            |url| url.as_str().encode_utf16().collect::<Vec<_>>().into(),
        )
    }
    pub fn FormStateControlType(&self, node: usize) -> Rc<[u16]> {
        if self.IsFormAssociatedCustomElement(node) {
            return self.arena[node]
                .Name()
                .encode_utf16()
                .collect::<Vec<_>>()
                .into();
        }
        let name = match self.FormControlClass(node) {
            Some(DOMFormControlClass::Select) => {
                if self.SelectIsMultiple(node) {
                    "select-multiple"
                } else {
                    "select-one"
                }
            }
            Some(DOMFormControlClass::TextArea) => "textarea",
            Some(DOMFormControlClass::Input) => {
                match self.InputType(node).expect("input type owner") {
                    super::DOMInputType::kButton => "button",
                    super::DOMInputType::kColor => "color",
                    super::DOMInputType::kFile => "file",
                    super::DOMInputType::kHidden => "hidden",
                    super::DOMInputType::kImage => "image",
                    super::DOMInputType::kNumber => "number",
                    super::DOMInputType::kRange => "range",
                    super::DOMInputType::kReset => "reset",
                    super::DOMInputType::kSubmit => "submit",
                    super::DOMInputType::kRadio => "radio",
                    super::DOMInputType::kCheckbox => "checkbox",
                    super::DOMInputType::kDate => "date",
                    super::DOMInputType::kDateTimeLocal => "datetime-local",
                    super::DOMInputType::kMonth => "month",
                    super::DOMInputType::kTime => "time",
                    super::DOMInputType::kWeek => "week",
                    super::DOMInputType::kEmail => "email",
                    super::DOMInputType::kPassword => "password",
                    super::DOMInputType::kSearch => "search",
                    super::DOMInputType::kTelephone => "tel",
                    super::DOMInputType::kURL => "url",
                    super::DOMInputType::kText => "text",
                }
            }
            _ => unreachable!("only source stateful classes serialize a control type"),
        };
        name.encode_utf16().collect::<Vec<_>>().into()
    }
    /// HTMLInputElement::ValueOrDefaultLabel. Rechrom's current English UA
    /// button labels are shared by form data and native control painting.
    pub fn InputValueOrDefaultLabelUTF16(&self, node: usize) -> Rc<[u16]> {
        if let Some(value) = self.InputValueAttributeUTF16(node) {
            return value;
        }
        let label = match self.InputType(node) {
            Some(super::DOMInputType::kSubmit) => "Submit",
            Some(super::DOMInputType::kReset) => "Reset",
            _ => "",
        };
        label.encode_utf16().collect::<Vec<_>>().into()
    }
    pub fn FormControlClass(&self, node: usize) -> Option<DOMFormControlClass> {
        use super::editing_node::DOMNodeClass as C;
        Some(match &self.arena[node].class {
            C::HTMLInputElement | C::MediaControlInput { .. } => DOMFormControlClass::Input,
            C::HTMLTextAreaElement => DOMFormControlClass::TextArea,
            C::HTMLSelectElement(_) => DOMFormControlClass::Select,
            C::HTMLButtonElement => DOMFormControlClass::Button,
            C::HTMLFieldSetElement => DOMFormControlClass::Fieldset,
            C::HTMLOutputElement(_) => DOMFormControlClass::Output,
            C::HTMLObjectElement(_) => DOMFormControlClass::Object,
            _ => return None,
        })
    }
    // HTMLFormControlElement h:153-154, constructor cc:71.
    pub fn ControlBlocksFormSubmission(&self, node: usize) -> bool {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .is_some_and(|state| state.blocks_form_submission)
    }
    pub fn SetControlBlocksFormSubmission(&mut self, node: usize, blocks: bool) {
        self.text_control_runtime
            .entry(self.arena[node].id)
            .or_default()
            .blocks_form_submission = blocks;
    }
    // HTMLFormControlElementWithState cc:386-400. Both pseudo states change.
    pub fn ForceControlUserValid(&mut self, node: usize) {
        self.text_control_runtime
            .entry(self.arena[node].id)
            .or_default()
            .force_user_valid = true;
        self.ControlPseudoStateChanged(node, crate::style_state::ControlPseudoState::UserValid);
        self.ControlPseudoStateChanged(node, crate::style_state::ControlPseudoState::UserInvalid);
    }
    pub fn ControlUserValidityIsExposed(&self, node: usize) -> bool {
        self.text_control_runtime
            .get(&self.arena[node].id)
            .is_some_and(|state| state.user_interaction >= 2 || state.force_user_valid)
    }

    // HTMLOptionElement::DisplayLabel/CollectOptionInnerText,
    // html_option_element.cc:248-258,550-562. Read exact UTF-16 and skip the
    // native script class's descendants; this is not layout innerText.
    // Source: html_option_element.cc:513-519. Unlike DisplayLabel, label()
    // preserves an explicitly present empty label attribute.
    pub fn OptionLabelUTF16(&self, option: usize) -> Rc<[u16]> {
        if let Some(label) = self.arena[option].AttributeValueUTF16("label") {
            return label;
        }
        self.OptionDisplayLabelUTF16(option)
    }
    pub fn OptionDisplayLabelUTF16(&self, option: usize) -> Rc<[u16]> {
        if let Some(label) = self.arena[option].AttributeValueUTF16("label") {
            if !label.is_empty() {
                return label;
            }
        }
        self.OptionInnerTextUTF16(option)
    }
    // HTMLOptionElement::value, cc:339-345. Value does not use label=.
    pub fn OptionValueUTF16(&self, option: usize) -> Rc<[u16]> {
        if let Some(value) = self.arena[option].AttributeValueUTF16("value") {
            return value;
        }
        self.OptionInnerTextUTF16(option)
    }
    pub fn OptionInnerTextUTF16(&self, option: usize) -> Rc<[u16]> {
        fn collect(document: &PersistentDocument, node: usize, text: &mut Vec<u16>) {
            if matches!(
                document.arena[node].class,
                super::editing_node::DOMNodeClass::ScriptElement
            ) {
                return;
            }
            if document.arena[node].node_type == super::DOMNodeType::kText {
                text.extend_from_slice(&document.CharacterDataUTF16(node));
            }
            for &child in &document.arena[node].children {
                collect(document, child, text);
            }
        }
        let mut text = Vec::new();
        for &child in &self.arena[option].children {
            collect(self, child, &mut text);
        }
        let mut result = Vec::new();
        let mut pending_space = false;
        for unit in text {
            if matches!(unit, 0x09 | 0x0a | 0x0c | 0x0d | 0x20) {
                pending_space = !result.is_empty();
            } else {
                if pending_space {
                    result.push(0x20);
                }
                pending_space = false;
                result.push(unit);
            }
        }
        Rc::from(result)
    }

    // The optgroup projection of HTMLSelectElement::WalkAncestorsForRelatedParts,
    // select.cc:2157-2195. A first optgroup remains the result at every later
    // related-part barrier; unlike :disabled this ignores a disabled select.
    pub fn OptionOwnerOptgroup(&self, option: usize) -> Option<usize> {
        use super::editing_node::DOMNodeClass as C;
        let mut parent = self.arena[option].parent;
        while let Some(index) = parent {
            match self.arena[index].class {
                C::HTMLOptGroupElement => return Some(index),
                C::HTMLOptionElement { .. }
                | C::HTMLSelectElement(_)
                | C::HTMLDataListElement(_)
                | C::HTMLHRElement => return None,
                _ => parent = self.arena[index].parent,
            }
        }
        None
    }
    // HTMLFormElement::SubmitDialog, html_form_element.cc:875-883.
    pub fn FirstAncestorDialog(&self, node: usize) -> Option<usize> {
        let mut parent = self.arena[node].parent;
        while let Some(index) = parent {
            if matches!(
                self.arena[index].class,
                super::editing_node::DOMNodeClass::HTMLDialogElement
            ) {
                return Some(index);
            }
            parent = self.arena[index].parent;
        }
        None
    }
    // HTMLOptionElement cc:414-426. This is per-option retained state, not
    // another selected-index or a Page interaction snapshot.
    pub fn SetOptionMultiSelectFocused(&mut self, option: usize, focused: bool) {
        if self.OptionMultiSelectFocused(option) == focused {
            return;
        }
        if let Some(select) = self.OptionOwnerSelect(option) {
            debug_assert!(self.SelectIsMultiple(select));
            let super::editing_node::DOMNodeClass::HTMLOptionElement {
                multi_select_focused,
                ..
            } = &mut self.arena[option].class
            else {
                unreachable!("native option class")
            };
            *multi_select_focused = focused;
            self.InvalidateNodeStyle(option);
        }
    }
    pub fn OptionMultiSelectFocused(&self, option: usize) -> bool {
        matches!(
            self.arena[option].class,
            super::editing_node::DOMNodeClass::HTMLOptionElement {
                multi_select_focused: true,
                ..
            }
        )
    }
    // HTMLInputElement::ParseAttribute / HTMLTextAreaElement::ParseAttribute
    // constraint changes, and select.cc:443,1294/base RequiredAttributeChanged.
    // The input type transition stays on its original subclass lifecycle.
    pub(super) fn ControlValidationAttributeChanged(
        &mut self,
        node: usize,
        name: &str,
        namespace: &str,
    ) {
        if !namespace.is_empty() {
            return;
        }
        match self.FormControlClass(node) {
            Some(DOMFormControlClass::Input | DOMFormControlClass::TextArea)
                if matches!(
                    name,
                    "value"
                        | "checked"
                        | "required"
                        | "readonly"
                        | "disabled"
                        | "pattern"
                        | "min"
                        | "max"
                        | "step"
                        | "multiple"
                        | "minlength"
                        | "maxlength"
                ) =>
            {
                self.SetControlNeedsValidityCheck(node);
                if matches!(name, "readonly" | "disabled") {
                    self.UpdateControlWillValidate(node);
                }
            }
            Some(DOMFormControlClass::Select)
                if matches!(name, "size" | "multiple" | "required" | "disabled") =>
            {
                self.SetControlNeedsValidityCheck(node)
            }
            _ if self.IsFormAssociatedCustomElement(node)
                && matches!(name, "disabled" | "readonly") =>
            {
                self.SetControlNeedsValidityCheck(node)
            }
            _ => {}
        }
    }
    // Source: Node::IsShadowIncludingInclusiveAncestorOf.
    pub fn IsShadowIncludingInclusiveAncestor(&self, ancestor: usize, mut node: usize) -> bool {
        loop {
            if node == ancestor {
                return true;
            }
            match self.Node(node).Parent().or_else(|| self.ShadowHost(node)) {
                Some(parent) => node = parent,
                None => return false,
            }
        }
    }

    /// Source virtual IsFormControlElement (html_form_control_element.h:184).
    /// This describes concrete DOM classes, not form association or default
    /// action policy: optgroup/option/object are not HTMLFormControlElement.
    pub fn IsFormControlElement(&self, node: usize) -> bool {
        self.FormControlClass(node)
            .is_some_and(|class| class != DOMFormControlClass::Object)
    }

    /// The listed-elements collection is tree ordered and includes controls
    /// associated by `form`, including controls outside the form subtree.
    pub fn ListedFormControlNodeIds(&self, form: usize) -> Rc<Vec<u64>> {
        let id = self.Node(form).Id();
        let revision = self.collection_membership_revision;
        if revision != u64::MAX {
            if let Some((cached_revision, value)) =
                self.form_collection_state.borrow().forms.get(&id)
            {
                if *cached_revision == revision {
                    return value.clone();
                }
            }
        }
        fn visit(document: &PersistentDocument, node: usize, form: usize, result: &mut Vec<u64>) {
            let element = document.Node(node);
            if (document.FormControlClass(node).is_some()
                || document.IsFormAssociatedCustomElement(node))
                && document.FormOwner(node) == Some(form)
            {
                result.push(element.Id());
            }
            for &child in element.Children() {
                visit(document, child, form, result);
            }
        }
        let mut result = Vec::new();
        visit(self, self.TreeScopeRoot(form), form, &mut result);
        let result = Rc::new(result);
        if revision != u64::MAX {
            self.form_collection_state
                .borrow_mut()
                .forms
                .insert(id, (revision, result.clone()));
        }
        result
    }
    // Source: html_select_element.cc:709-776,2339-2364.
    pub fn SelectCollection(&self, select: usize) -> Rc<DOMSelectCollection> {
        let id = self.Node(select).Id();
        let revision = self.collection_membership_revision;
        if revision != u64::MAX {
            if let Some((cached_revision, value)) =
                self.form_collection_state.borrow().selects.get(&id)
            {
                if *cached_revision == revision {
                    return value.clone();
                }
            }
        }
        fn collect(
            document: &PersistentDocument,
            parent: usize,
            select: usize,
            in_optgroup: bool,
            items: &mut Vec<u64>,
            options: &mut Vec<u64>,
        ) {
            for &child in document.Node(parent).Children() {
                let element = document.Node(child);
                if element.IsHTMLElement("select") || element.IsHTMLElement("datalist") {
                    continue;
                }
                if element.IsHTMLElement("optgroup") {
                    if in_optgroup {
                        continue;
                    }
                    if items.len() < 100_000 {
                        items.push(element.Id());
                    }
                    collect(document, child, select, true, items, options);
                } else if element.IsHTMLElement("option") {
                    if items.len() < 100_000 {
                        items.push(element.Id());
                    }
                    if document.OptionOwnerSelect(child) == Some(select) {
                        options.push(element.Id());
                    }
                } else if element.IsHTMLElement("hr") {
                    if items.len() < 100_000 {
                        items.push(element.Id());
                    }
                } else {
                    collect(document, child, select, in_optgroup, items, options);
                }
            }
        }
        let mut list_items = Vec::new();
        let mut options = Vec::new();
        // The option range is unbounded even though ListItems has a popup
        // safety limit. Both ranges derive in the same source-order walk.
        collect(self, select, select, false, &mut list_items, &mut options);
        let result = Rc::new(DOMSelectCollection {
            list_items,
            options,
        });
        if revision != u64::MAX {
            self.form_collection_state
                .borrow_mut()
                .selects
                .insert(id, (revision, result.clone()));
        }
        result
    }
    /// HTMLDataListElement::options/HTMLDataListOptionsCollection. The live
    /// DOM owner retains the tree-ordered collection at its membership epoch.
    pub fn InputDataListOptionNodeIds(&self, input: usize) -> Option<Rc<Vec<u64>>> {
        Some(self.DataListOptionNodeIds(self.InputDataList(input)?))
    }
    pub fn DataListOptionNodeIds(&self, list: usize) -> Rc<Vec<u64>> {
        let key = self.Node(list).Id();
        let revision = self.collection_membership_revision;
        if revision != u64::MAX {
            if let Some((cached_revision, nodes)) =
                self.form_collection_state.borrow().datalists.get(&key)
            {
                if *cached_revision == revision {
                    return nodes.clone();
                }
            }
        }
        fn append(document: &PersistentDocument, node: usize, result: &mut Vec<u64>) {
            for &child in document.Node(node).Children() {
                if document.Node(child).IsHTMLElement("option") {
                    result.push(document.Node(child).Id());
                }
                append(document, child, result);
            }
        }
        let mut nodes = Vec::new();
        append(self, list, &mut nodes);
        let nodes = Rc::new(nodes);
        if revision != u64::MAX {
            self.form_collection_state
                .borrow_mut()
                .datalists
                .insert(key, (revision, nodes.clone()));
        }
        nodes
    }
    pub fn SetOptionSelectionDirty(&mut self, option: usize, dirty: bool) {
        if dirty {
            self.option_dirty_state.insert(self.Node(option).Id());
        } else {
            self.option_dirty_state.remove(&self.Node(option).Id());
        }
    }
}
