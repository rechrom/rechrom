#![allow(non_snake_case)]

use std::rc::Rc;

use crate::error::{DOMException, DOMExceptionKind};
use crate::persistent_document::{
    DOMAttribute, DOMNamespace, DOMNodeType, DOMShadowRootMode, DOMSlotAssignmentMode,
    PersistentDocument,
};

/// Existing DOM exception categories exposed through the mutation facade.
pub type DOMMutationError = DOMException;

// cpp: dom/dom_mutation.h:15-35
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum DOMMutationType {
    kParseDocument,
    kCreateDocument,
    kCreateElement,
    kCreateText,
    kCreateComment,
    kCreateFragment,
    kAppendTextOrBr,
    kAttachShadowRoot,
    kSetShadowReferenceTarget,
    kCloneNode,
    kImportNode,
    kAppendChild,
    kInsertBefore,
    kRemoveChild,
    #[default]
    kSetAttribute,
    kRemoveAttribute,
    kRemoveAllAttributes,
    kSetTextContent,
    kSetInnerHTML,
    kSetControlChecked,
    kSetControlValue,
    kSetInputFiles,
    kSetControlIndeterminate,
    kSetDialogModal,
    kSetPopoverOpen,
    kRequestFullscreen,
    kExitFullscreen,
    kAssignSlotNodes,
    kSetOptionSelected,
    /// CSSOM mutation scope completed without a changed declaration. Retain
    /// custom-element notification, without attribute/observer/style mutation.
    kNotifyInlineStyleUnchanged,
    /// TextControlElement::SetInnerEditorValue, for programmatic view updates.
    kSetTextControlInnerEditorValue,
    /// Editable-content notification after commands mutate the resident editor.
    kSyncTextControlValueFromInnerEditor,
    kReplaceCharacterData,
    kDeleteRangeContents,
    kInsertRangeNode,
    /// Source HTMLFormControlElementWithState user-interaction flag.
    kSetControlUserEdited,
    /// Source HTMLFormControlElement's canonical autofill state.
    kSetControlAutofillState,
    /// Source SetNonDirtyValue commits value and dirtyness atomically.
    kSetNonDirtyControlValue,
    kSetControlCheckednessDirty,
    kSetControlNeedsValidityCheck,
    kResetOutput,
    /// HTMLOptionElement source state update without IDL selection-change behavior.
    kSetOptionSelectedState,
    kSetOptionDirty,
    kUpdateOptionMutationObserver,
    kSetSelectLastOnChangeOption,
    kUpdateSelectSelectedcontents,
}

#[derive(Clone)]
pub struct DOMFileListValue(pub Rc<fileapi::FileList>);
impl PartialEq for DOMFileListValue {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DOMFileListValue {}
impl std::fmt::Debug for DOMFileListValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("FileList")
            .field(&self.0.length())
            .finish()
    }
}

// cpp: dom/dom_mutation.h:40-51
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DOMMutation {
    pub mutation_type: DOMMutationType,
    pub target_node_id: u64,
    pub name: String,
    /// Canonical qualified name for source element/attribute edit commands. Scalar
    /// name fields remain notification metadata only when this is present.
    pub qualified_name: Option<foundation::QualifiedName>,
    /// Namespace prefix retained separately from the local name.  Ordinary
    /// HTML attributes leave this empty; namespace-aware DOM entry points
    /// populate it from the qualified name.
    pub prefix: String,
    pub value: String,
    pub file_list: Option<DOMFileListValue>,
    /// Exact JavaScript/WTF-16 value. None uses the UTF-8 convenience field.
    pub utf16_value: Option<std::rc::Rc<[u16]>>,
    /// ElementCreationOptions.is (one-shot ElementRareData, not an attribute).
    pub is_value: Option<std::rc::Rc<[u16]>>,
    pub namespace_uri: String,
    pub bool_value: bool,
    /// Source SetSelectedState skip_mutation_observer parameter.
    pub option_skip_mutation_observer: bool,
    /// Typed source autofill state for kSetControlAutofillState.
    pub autofill_state: crate::DOMAutofillState,
    /// CSSOM's existing-declaration mutation classification for a null-namespace
    /// style attribute. Generic setAttribute/cssText writes leave this false.
    pub inline_style_independent: bool,
    /// Parsed declaration ownership travels with a CSSOM style write, before
    /// mutation callbacks can reenter the document. Concrete types stay private.
    pub inline_style: Option<cssom::CSSInlineStyle>,
    /// ShadowRoot::referenceTarget nullable DOMString (also attachShadow init).
    pub reference_target: Option<String>,
    pub child_node_id: u64,
    /// Source node for an import into target_node_id (the destination document).
    pub source_node_id: u64,
    pub before_node_id: u64,
    /// Ordered source identities for atomic HTMLSlotElement.assign().
    pub node_ids: Vec<u64>,
    /// CharacterData::replaceData offsets/counts use UTF-16 code units.
    pub offset: usize,
    pub count: usize,
    /// A live DOM-owned range, never a browser-owned endpoint map.
    pub range: Option<crate::range::DOMRangeHandle>,
}

// cpp: dom/dom_mutation.h:53-54
pub type DOMMutationList = Vec<DOMMutation>;
pub type DOMMutationEmitter = Box<dyn FnMut(&DOMMutation)>;

// cpp: dom/dom_mutation.cc:12-86
pub(crate) fn ValidateDOMMutation(
    document: &PersistentDocument,
    mutation: &DOMMutation,
    allow_fragment_parser: bool,
) -> Result<(), DOMMutationError> {
    if mutation.mutation_type == DOMMutationType::kParseDocument {
        return Err(DOMException {
            kind: DOMExceptionKind::InvalidArgument,
            message: "parse-document mutation must be applied through Page",
        });
    }
    let Some(target) = document.FindNodeById(mutation.target_node_id) else {
        return Err(DOMException {
            kind: DOMExceptionKind::InvalidArgument,
            message: "DOM mutation target does not exist",
        });
    };
    match mutation.mutation_type {
        DOMMutationType::kCreateDocument
        | DOMMutationType::kCreateElement
        | DOMMutationType::kCreateText
        | DOMMutationType::kCreateComment
        | DOMMutationType::kCreateFragment => {
            if document.Node(target).Type() != DOMNodeType::kDocument
                || mutation.child_node_id != document.NextNodeId()
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "invalid node allocation mutation",
                });
            }
        }
        DOMMutationType::kSetShadowReferenceTarget => {
            if document.ShadowHost(target).is_none() {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "referenceTarget requires a ShadowRoot",
                });
            }
        }
        DOMMutationType::kAttachShadowRoot => {
            if document.Node(target).Type() != DOMNodeType::kElement
                || mutation.child_node_id != document.NextNodeId()
                || document.ShadowRoot(target).is_some()
                || !matches!(mutation.value.as_str(), "open" | "closed")
                || !matches!(mutation.name.as_str(), "named" | "manual")
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "invalid shadow root attachment",
                });
            }
        }
        DOMMutationType::kAssignSlotNodes => {
            if !document.Node(target).IsHTMLElement("slot")
                || mutation.node_ids.iter().any(|&id| {
                    document.FindNodeById(id).is_none_or(|node| {
                        !matches!(
                            document.Node(node).Type(),
                            DOMNodeType::kElement | DOMNodeType::kText
                        )
                    })
                })
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "invalid manual slot assignment",
                });
            }
        }
        DOMMutationType::kImportNode => {
            let source = document.FindNodeById(mutation.source_node_id);
            if document.Node(target).Type() != DOMNodeType::kDocument
                || mutation.child_node_id != document.NextNodeId()
                || source.is_none_or(|source| {
                    document.Node(source).Type() == DOMNodeType::kDocument
                        || document.ShadowHost(source).is_some()
                })
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "invalid node import mutation",
                });
            }
        }
        DOMMutationType::kCloneNode => {
            if mutation.child_node_id != document.NextNodeId()
                || document.Node(target).Type() == DOMNodeType::kDocument
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "invalid node clone mutation",
                });
            }
        }
        DOMMutationType::kAppendChild
        | DOMMutationType::kInsertBefore
        | DOMMutationType::kRemoveChild => {
            let Some(child) = document.FindNodeById(mutation.child_node_id) else {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "missing child",
                });
            };
            if mutation.mutation_type == DOMMutationType::kRemoveChild {
                if document.Node(child).Parent() != Some(target) {
                    return Err(DOMException {
                        kind: DOMExceptionKind::InvalidArgument,
                        message: "child is not in parent",
                    });
                }
            } else {
                if !matches!(
                    document.Node(target).Type(),
                    DOMNodeType::kElement | DOMNodeType::kDocument | DOMNodeType::kDocumentFragment
                ) {
                    return Err(DOMException {
                        kind: DOMExceptionKind::InvalidArgument,
                        message: "parent cannot contain children",
                    });
                }
                if document.IsShadowIncludingInclusiveAncestor(child, target) {
                    return Err(DOMException {
                        kind: DOMExceptionKind::InvalidArgument,
                        message: "insertion would create a cycle",
                    });
                }
                if mutation.mutation_type == DOMMutationType::kInsertBefore
                    && mutation.before_node_id != 0
                {
                    let before = document.FindNodeById(mutation.before_node_id);
                    if before.is_none_or(|before| document.Node(before).Parent() != Some(target)) {
                        return Err(DOMException {
                            kind: DOMExceptionKind::InvalidArgument,
                            message: "reference is not in parent",
                        });
                    }
                }
            }
        }
        DOMMutationType::kParseDocument => {}
        DOMMutationType::kSetAttribute
        | DOMMutationType::kRemoveAttribute
        | DOMMutationType::kRemoveAllAttributes
        | DOMMutationType::kNotifyInlineStyleUnchanged => {
            if document.Node(target).Type() != DOMNodeType::kElement {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "DOM attribute mutation target is not an element",
                });
            }
            if mutation.name.is_empty()
                && mutation.mutation_type != DOMMutationType::kRemoveAllAttributes
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "DOM attribute mutation name is empty",
                });
            }
        }
        DOMMutationType::kReplaceCharacterData => {
            if !matches!(
                document.Node(target).Type(),
                DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
            ) {
                return Err(crate::range::DOMRangeError::InvalidNodeType.into());
            }
            if mutation.offset > document.Node(target).Data().encode_utf16().count() {
                return Err(crate::range::DOMRangeError::InvalidOffset.into());
            }
        }
        DOMMutationType::kDeleteRangeContents | DOMMutationType::kInsertRangeNode => {
            let range = mutation.range.as_ref().ok_or_else(|| DOMMutationError {
                kind: DOMExceptionKind::InvalidArgument,
                message: "range mutation requires a live range",
            })?;
            if document.Range(range).is_none() {
                return Err(crate::range::DOMRangeError::WrongDocument.into());
            }
            if mutation.mutation_type == DOMMutationType::kInsertRangeNode {
                document.ValidateRangeInsertion(range, mutation.child_node_id)?;
            }
        }
        DOMMutationType::kAppendTextOrBr => {
            if !matches!(
                document.Node(target).Type(),
                DOMNodeType::kElement | DOMNodeType::kDocumentFragment
            ) || mutation.utf16_value.is_none()
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "AppendTextOrBr requires a container and retained UTF-16 text",
                });
            }
        }
        DOMMutationType::kSetTextContent => {
            if !matches!(
                document.Node(target).Type(),
                DOMNodeType::kElement
                    | DOMNodeType::kDocumentFragment
                    | DOMNodeType::kText
                    | DOMNodeType::kComment
                    | DOMNodeType::kProcessingInstruction
            ) {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "DOM text mutation target cannot contain text",
                });
            }
        }
        DOMMutationType::kSetInnerHTML => {
            if !matches!(
                document.Node(target).Type(),
                DOMNodeType::kElement | DOMNodeType::kDocumentFragment
            ) {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "innerHTML mutation target cannot contain children",
                });
            }
            if !allow_fragment_parser {
                return Err(DOMException {
                    kind: DOMExceptionKind::LogicError,
                    message: "innerHTML mutation requires the Page HTML parser service",
                });
            }
        }
        DOMMutationType::kSetControlChecked => {
            if !document.Node(target).IsHTMLElement("input") {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "checked state mutation target is not an input",
                });
            }
        }
        DOMMutationType::kSetControlIndeterminate => {
            if !document.Node(target).IsHTMLElement("input") {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "indeterminate state mutation target is not an input",
                });
            }
        }
        DOMMutationType::kSetDialogModal => {
            if !document.Node(target).IsHTMLElement("dialog") {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "modal state mutation target is not a dialog",
                });
            }
        }
        DOMMutationType::kSetPopoverOpen => {
            if document.Node(target).Type() != DOMNodeType::kElement {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "popover method requires an element receiver",
                });
            }
        }
        DOMMutationType::kRequestFullscreen => {
            if document.Node(target).Type() != DOMNodeType::kElement
                || !document.IsConnectedToDocument(target)
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "fullscreen request target is not a connected element",
                });
            }
        }
        DOMMutationType::kExitFullscreen => {
            if document.Node(target).Type() != DOMNodeType::kDocument {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "exitFullscreen target is not a document",
                });
            }
        }
        DOMMutationType::kSetSelectLastOnChangeOption
        | DOMMutationType::kUpdateSelectSelectedcontents => {
            if !document.Node(target).IsHTMLElement("select")
                || mutation.child_node_id != 0
                    && document
                        .FindNodeById(mutation.child_node_id)
                        .is_none_or(|option| document.OptionOwnerSelect(option) != Some(target))
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "select option mutation requires its owning select",
                });
            }
        }
        DOMMutationType::kSetOptionSelected
        | DOMMutationType::kSetOptionSelectedState
        | DOMMutationType::kSetOptionDirty
        | DOMMutationType::kUpdateOptionMutationObserver => {
            if !document.Node(target).IsHTMLElement("option") {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "selected mutation target is not an option",
                });
            }
        }
        DOMMutationType::kSetTextControlInnerEditorValue
        | DOMMutationType::kSyncTextControlValueFromInnerEditor => {
            if document.TextControlInnerEditorElement(target).is_none() {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "inner editor mutation target is not a text control",
                });
            }
        }
        DOMMutationType::kSetInputFiles => {
            if !document.Node(target).IsHTMLElement("input") || mutation.file_list.is_none() {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "files mutation requires an input and FileList",
                });
            }
        }
        DOMMutationType::kSetControlCheckednessDirty => {
            if !document.Node(target).IsHTMLElement("input") {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "checkedness dirty state requires an input",
                });
            }
        }
        DOMMutationType::kResetOutput => {
            if !document.Node(target).IsHTMLElement("output") {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "output reset requires an output element",
                });
            }
        }
        DOMMutationType::kSetControlNeedsValidityCheck => {
            if document.FormControlClass(target).is_none()
                && !document.ElementInternalsTargetIsFormAssociated(target)
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "validity state requires a native listed form control",
                });
            }
        }
        DOMMutationType::kSetControlUserEdited | DOMMutationType::kSetControlAutofillState => {
            if !document.ClassSupportsFormStateRestore(target) {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "control state mutation target has no native form state",
                });
            }
        }
        DOMMutationType::kSetControlValue | DOMMutationType::kSetNonDirtyControlValue => {
            if document.InputType(target) == Some(crate::persistent_document::DOMInputType::kFile)
                && mutation
                    .utf16_value
                    .as_ref()
                    .map_or(!mutation.value.is_empty(), |value| !value.is_empty())
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "a file input value can only be cleared",
                });
            }
            if !document.Node(target).IsHTMLElement("input")
                && !document.Node(target).IsHTMLElement("textarea")
                && !document.Node(target).IsHTMLElement("select")
            {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "control value mutation target is not a value control",
                });
            }
        }
    }
    Ok(())
}

// cpp: dom/dom_mutation.h:58-58
// cpp: dom/dom_mutation.cc:90-154
pub fn ApplyDOMMutations(document: &mut PersistentDocument, mutations: &[DOMMutation]) {
    TryApplyDOMMutations(document, mutations).unwrap_or_else(|error| std::panic::panic_any(error));
}

/// Fallible entry to the same validation and mutation algorithm. The legacy
/// entry above retains its C++ exception-style panic payloads.
pub fn TryApplyDOMMutations(
    document: &mut PersistentDocument,
    mutations: &[DOMMutation],
) -> Result<(), DOMMutationError> {
    for mutation in mutations {
        ValidateDOMMutation(document, mutation, false)?;
    }
    for mutation in mutations {
        let target = document
            .FindNodeById(mutation.target_node_id)
            .expect("validated DOM mutation target must exist");
        match mutation.mutation_type {
            DOMMutationType::kCreateDocument => {
                document.CreateHTMLDocument(if mutation.bool_value {
                    Some(mutation.value.clone())
                } else {
                    None
                });
            }
            DOMMutationType::kCreateElement => {
                // bool_value distinguishes Document::createElementNS from the
                // ordinary HTML creation convenience when namespace is null.
                let namespace = match mutation.namespace_uri.as_str() {
                    "http://www.w3.org/2000/svg" => DOMNamespace::kSVG,
                    "http://www.w3.org/1998/Math/MathML" => DOMNamespace::kMathML,
                    "http://www.w3.org/1999/xhtml" => DOMNamespace::kHTML,
                    _ if mutation.bool_value => DOMNamespace::kNone,
                    _ => DOMNamespace::kHTML,
                };
                let element = if let Some(name) = &mutation.qualified_name {
                    document.CreateElementWithQualifiedName(namespace, name.clone(), Vec::new())
                } else if !mutation.bool_value
                    && mutation.prefix.is_empty()
                    && mutation.namespace_uri.is_empty()
                {
                    document.CreateElementDefault(namespace, mutation.name.clone())
                } else {
                    document.CreateElementWithQualifiedName(
                        namespace,
                        foundation::QualifiedName::new(
                            if mutation.prefix.is_empty() {
                                foundation::AtomicString::default()
                            } else {
                                foundation::AtomicString::from_str(&mutation.prefix)
                            },
                            foundation::AtomicString::from_str(&mutation.name),
                            if mutation.namespace_uri.is_empty() {
                                foundation::AtomicString::default()
                            } else {
                                foundation::AtomicString::from_str(&mutation.namespace_uri)
                            },
                        ),
                        Vec::new(),
                    )
                };
                if let Some(is_value) = mutation.is_value.as_ref() {
                    document.SetElementIsValueUTF16(element, is_value.clone());
                }
            }
            DOMMutationType::kAppendTextOrBr => {
                document.AppendTextOrBrUTF16(
                    target,
                    mutation
                        .utf16_value
                        .as_deref()
                        .expect("AppendTextOrBr retains exact source text"),
                );
            }
            DOMMutationType::kCreateText => {
                if let Some(units) = &mutation.utf16_value {
                    document.CreateTextUTF16(units);
                } else {
                    document.CreateText(mutation.value.clone());
                }
            }
            DOMMutationType::kCreateComment => {
                if let Some(units) = &mutation.utf16_value {
                    document.CreateCommentUTF16(units);
                } else {
                    document.CreateComment(mutation.value.clone());
                }
            }
            DOMMutationType::kCreateFragment => {
                document.CreateDocumentFragment();
            }
            DOMMutationType::kAttachShadowRoot => {
                let mode = if mutation.value == "open" {
                    DOMShadowRootMode::kOpen
                } else {
                    DOMShadowRootMode::kClosed
                };
                let slot_assignment = if mutation.name == "named" {
                    DOMSlotAssignmentMode::kNamed
                } else {
                    DOMSlotAssignmentMode::kManual
                };
                let root =
                    document.AttachShadowRootWithSlotAssignment(target, mode, slot_assignment);
                document.SetShadowRootDelegatesFocus(root, mutation.bool_value);
                document.SetShadowReferenceTarget(root, mutation.reference_target.clone());
                debug_assert_eq!(document.Node(root).Id(), mutation.child_node_id);
            }
            DOMMutationType::kSetShadowReferenceTarget => {
                document.SetShadowReferenceTarget(target, mutation.reference_target.clone());
            }
            DOMMutationType::kAssignSlotNodes => {
                let nodes = mutation
                    .node_ids
                    .iter()
                    .map(|&id| {
                        document
                            .FindNodeById(id)
                            .expect("validated assigned node must exist")
                    })
                    .collect();
                document.AssignSlotNodes(target, nodes);
            }
            DOMMutationType::kCloneNode => {
                document.CloneNode(target, mutation.bool_value);
            }
            DOMMutationType::kImportNode => {
                let source = document
                    .FindNodeById(mutation.source_node_id)
                    .expect("validated import source must exist");
                document.ImportNode(source, target, mutation.bool_value);
            }
            DOMMutationType::kAppendChild | DOMMutationType::kInsertBefore => {
                let child = document
                    .FindNodeById(mutation.child_node_id)
                    .expect("validated DOM mutation child must exist");
                let before = document.FindNodeById(mutation.before_node_id);
                let children = if document.Node(child).Type() == DOMNodeType::kDocumentFragment {
                    document.Node(child).Children().to_vec()
                } else {
                    vec![child]
                };
                for item in children {
                    if Some(item) == before {
                        continue;
                    }
                    if let Some(before) = before {
                        document.InsertBefore(target, item, before);
                    } else {
                        document.AppendChild(target, item);
                    }
                }
            }
            DOMMutationType::kRemoveChild => {
                let child = document
                    .FindNodeById(mutation.child_node_id)
                    .expect("validated DOM mutation child must exist");
                document.Remove(child);
            }
            DOMMutationType::kParseDocument => {}
            DOMMutationType::kSetAttribute => {
                let attribute = DOMAttribute {
                    prefix: mutation.prefix.clone(),
                    local_name: mutation.name.clone(),
                    namespace_uri: mutation.namespace_uri.clone(),
                    value: mutation.value.clone(),
                    ..DOMAttribute::default()
                };
                if let Some(name) = &mutation.qualified_name {
                    document.SetAttributeByQualifiedNameUTF16(
                        target,
                        name,
                        mutation
                            .utf16_value
                            .clone()
                            .expect("qualified attribute write retains UTF-16 value"),
                        mutation.inline_style_independent,
                    );
                } else if let Some(units) = mutation.utf16_value.clone().or_else(|| {
                    mutation
                        .inline_style
                        .as_ref()
                        .map(|style| Rc::from(style.SourceTextUTF16()))
                }) {
                    // The typed CSSOM source owns exact UTF-16 text. Its legacy
                    // UTF-8 projection must not replace that value or discard
                    // the independent-property invalidation classification.
                    document.SetAttributeUTF16WithInlineStyleChange(
                        target,
                        attribute,
                        units,
                        mutation.inline_style_independent,
                    );
                } else {
                    document.SetAttributeWithInlineStyleChange(
                        target,
                        attribute,
                        mutation.inline_style_independent,
                    );
                }
                if mutation.name == "style" && mutation.namespace_uri.is_empty() {
                    if let Some(style) = &mutation.inline_style {
                        document.Node(target).RetainInlineStyle(style.clone());
                    }
                }
            }
            DOMMutationType::kRemoveAttribute => {
                if let Some(name) = &mutation.qualified_name {
                    document.RemoveAttributeByQualifiedName(target, name);
                } else {
                    document.RemoveAttribute(target, &mutation.name, &mutation.namespace_uri);
                }
            }
            DOMMutationType::kRemoveAllAttributes => document.RemoveAllAttributesForCloning(target),
            DOMMutationType::kNotifyInlineStyleUnchanged => {
                // cpp: StyleAttributeMutationScope destructor notifies a custom
                // element even when SetPropertyInternal returns kNoChanges.
                let value = document
                    .Node(target)
                    .FindAttribute("style")
                    .map(|a| a.value.clone());
                document.CustomElementAttributeWillChange(
                    target,
                    "style",
                    "",
                    value.clone(),
                    value,
                );
            }
            DOMMutationType::kReplaceCharacterData => {
                if let Some(units) = &mutation.utf16_value {
                    document.ReplaceDataUTF16(target, mutation.offset, mutation.count, units)?;
                } else {
                    document.ReplaceData(
                        target,
                        mutation.offset,
                        mutation.count,
                        &mutation.value,
                    )?;
                }
            }
            DOMMutationType::kDeleteRangeContents => {
                document.DeleteRangeContents(
                    mutation.range.as_ref().expect("validated range mutation"),
                )?;
            }
            DOMMutationType::kInsertRangeNode => {
                document.InsertRangeNode(
                    mutation.range.as_ref().expect("validated range mutation"),
                    mutation.child_node_id,
                )?;
            }
            DOMMutationType::kSetTextContent => {
                if let Some(units) = &mutation.utf16_value {
                    document.SetTextContentUTF16(target, units);
                } else {
                    document.SetTextContent(target, mutation.value.clone());
                }
            }
            DOMMutationType::kSetInnerHTML => {}
            DOMMutationType::kSetControlChecked => {
                document.SetControlChecked(target, mutation.bool_value);
            }
            DOMMutationType::kSetControlIndeterminate => {
                document.SetControlIndeterminate(target, mutation.bool_value);
            }
            DOMMutationType::kSetDialogModal | DOMMutationType::kSetPopoverOpen => {
                return Err(DOMException {
                    kind: DOMExceptionKind::LogicError,
                    message: "popup methods require the shared native lifecycle transaction",
                });
            }
            DOMMutationType::kRequestFullscreen => {
                document.RequestFullscreen(target);
            }
            DOMMutationType::kExitFullscreen => {
                document.ExitFullscreen();
            }
            DOMMutationType::kSetSelectLastOnChangeOption => {
                let option = (mutation.child_node_id != 0)
                    .then(|| document.FindNodeById(mutation.child_node_id))
                    .flatten();
                document.SetSelectLastOnChangeOption(target, option);
            }
            DOMMutationType::kUpdateSelectSelectedcontents => {
                let option = (mutation.child_node_id != 0)
                    .then(|| document.FindNodeById(mutation.child_node_id))
                    .flatten();
                if mutation.bool_value {
                    document.SelectUpdateAllSelectedcontentsMultiple(target);
                } else {
                    document.SelectUpdateAllSelectedcontentsSingle(target, option);
                }
            }
            DOMMutationType::kSetOptionSelectedState => {
                document.SetOptionSelectedStateWithObserverUpdate(
                    target,
                    mutation.bool_value,
                    mutation.option_skip_mutation_observer,
                );
            }
            DOMMutationType::kUpdateOptionMutationObserver => {
                document.UpdateOptionMutationObserver(target, mutation.bool_value);
            }
            DOMMutationType::kSetOptionDirty => {
                document.SetOptionDirty(target, mutation.bool_value);
            }
            DOMMutationType::kSetOptionSelected => {
                document.SetOptionSelected(target, mutation.bool_value);
            }
            DOMMutationType::kSetInputFiles => {
                document.AssignInputFiles(
                    target,
                    mutation
                        .file_list
                        .as_ref()
                        .expect("validated FileList mutation")
                        .0
                        .clone(),
                );
            }
            DOMMutationType::kSetControlCheckednessDirty => {
                if mutation.bool_value {
                    document.SetControlCheckednessDirty(target);
                } else {
                    document.ClearControlCheckednessDirty(target);
                }
            }
            DOMMutationType::kSetControlNeedsValidityCheck => {
                document.SetControlNeedsValidityCheck(target);
            }
            DOMMutationType::kResetOutput => {
                document.ResetOutput(target);
            }
            DOMMutationType::kSetNonDirtyControlValue => {
                if let Some(units) = &mutation.utf16_value {
                    document.SetControlValueUTF16(target, units);
                } else {
                    document.SetControlValue(target, mutation.value.clone());
                }
                document.SetControlValueDirty(target, false);
            }
            DOMMutationType::kSetControlUserEdited => {
                document.TextControlSetUserEdited(target, mutation.bool_value);
            }
            DOMMutationType::kSetControlAutofillState => {
                document.SetAutofillState(target, mutation.autofill_state);
            }
            DOMMutationType::kSetControlValue => {
                if let Some(units) = &mutation.utf16_value {
                    document.SetControlValueUTF16(target, units);
                } else {
                    document.SetControlValue(target, mutation.value.clone());
                }
            }
            DOMMutationType::kSetTextControlInnerEditorValue => {
                if let Some(units) = &mutation.utf16_value {
                    document.SetTextControlInnerEditorValueUTF16(target, units);
                } else {
                    document.SetTextControlInnerEditorValue(target, mutation.value.clone());
                }
            }
            DOMMutationType::kSyncTextControlValueFromInnerEditor => {
                document.SyncTextControlValueFromInnerEditor(target);
            }
        }
        if matches!(
            mutation.mutation_type,
            DOMMutationType::kCreateElement
                | DOMMutationType::kCreateText
                | DOMMutationType::kCreateComment
                | DOMMutationType::kCreateFragment
        ) {
            let child = document
                .FindNodeById(mutation.child_node_id)
                .expect("new DOM node must have the declared ID");
            document.AdoptNode(child, target);
        }
    }
    Ok(())
}

// cpp: dom/dom_mutation.h:60-60
// cpp: dom/dom_mutation.cc:156-176
pub fn DOMMutationTypeName(mutation_type: DOMMutationType) -> &'static str {
    match mutation_type {
        DOMMutationType::kParseDocument => "parse-document",
        DOMMutationType::kCreateDocument => "create-document",
        DOMMutationType::kCreateElement => "create-element",
        DOMMutationType::kCreateText => "create-text",
        DOMMutationType::kCreateComment => "create-comment",
        DOMMutationType::kCreateFragment => "create-fragment",
        DOMMutationType::kAppendTextOrBr => "append-text-or-br",
        DOMMutationType::kAttachShadowRoot => "attach-shadow-root",
        DOMMutationType::kSetShadowReferenceTarget => "set-shadow-reference-target",
        DOMMutationType::kCloneNode => "clone-node",
        DOMMutationType::kImportNode => "import-node",
        DOMMutationType::kAppendChild => "append-child",
        DOMMutationType::kInsertBefore => "insert-before",
        DOMMutationType::kRemoveChild => "remove-child",
        DOMMutationType::kSetAttribute => "set-attribute",
        DOMMutationType::kRemoveAttribute => "remove-attribute",
        DOMMutationType::kRemoveAllAttributes => "remove-all-attributes",
        DOMMutationType::kSetTextContent => "set-text-content",
        DOMMutationType::kReplaceCharacterData => "replace-character-data",
        DOMMutationType::kDeleteRangeContents => "delete-range-contents",
        DOMMutationType::kInsertRangeNode => "insert-range-node",
        DOMMutationType::kSetInnerHTML => "set-inner-html",
        DOMMutationType::kSetControlChecked => "set-control-checked",
        DOMMutationType::kSetControlIndeterminate => "set-control-indeterminate",
        DOMMutationType::kSetControlValue => "set-control-value",
        DOMMutationType::kSetControlUserEdited => "set-control-user-edited",
        DOMMutationType::kSetControlAutofillState => "set-control-autofill-state",
        DOMMutationType::kSetNonDirtyControlValue => "set-non-dirty-control-value",
        DOMMutationType::kSetControlCheckednessDirty => "set-control-checkedness-dirty",
        DOMMutationType::kSetControlNeedsValidityCheck => "set-control-needs-validity-check",
        DOMMutationType::kResetOutput => "reset-output",
        DOMMutationType::kSetInputFiles => "set-input-files",
        DOMMutationType::kSetTextControlInnerEditorValue => "set-text-control-inner-editor-value",
        DOMMutationType::kSyncTextControlValueFromInnerEditor => {
            "sync-text-control-value-from-inner-editor"
        }
        DOMMutationType::kSetOptionSelected => "set-option-selected",
        DOMMutationType::kSetOptionSelectedState => "set-option-selected-state",
        DOMMutationType::kSetOptionDirty => "set-option-dirty",
        DOMMutationType::kUpdateOptionMutationObserver => "update-option-mutation-observer",
        DOMMutationType::kSetSelectLastOnChangeOption => "set-select-last-on-change-option",
        DOMMutationType::kUpdateSelectSelectedcontents => "update-select-selectedcontents",
        DOMMutationType::kSetDialogModal => "set-dialog-modal",
        DOMMutationType::kSetPopoverOpen => "set-popover-open",
        DOMMutationType::kRequestFullscreen => "request-fullscreen",
        DOMMutationType::kExitFullscreen => "exit-fullscreen",
        DOMMutationType::kAssignSlotNodes => "assign-slot-nodes",
        DOMMutationType::kNotifyInlineStyleUnchanged => "notify-inline-style-unchanged",
    }
}

// cpp: dom/dom_mutation.cc:175-175
pub fn DOMMutationTypeNameRaw(value: u8) -> &'static str {
    [
        DOMMutationType::kParseDocument,
        DOMMutationType::kCreateDocument,
        DOMMutationType::kCreateElement,
        DOMMutationType::kCreateText,
        DOMMutationType::kCreateComment,
        DOMMutationType::kCreateFragment,
        DOMMutationType::kAttachShadowRoot,
        DOMMutationType::kSetShadowReferenceTarget,
        DOMMutationType::kCloneNode,
        DOMMutationType::kImportNode,
        DOMMutationType::kAppendChild,
        DOMMutationType::kInsertBefore,
        DOMMutationType::kRemoveChild,
        DOMMutationType::kSetAttribute,
        DOMMutationType::kRemoveAttribute,
        DOMMutationType::kSetTextContent,
        DOMMutationType::kSetInnerHTML,
        DOMMutationType::kSetControlChecked,
        DOMMutationType::kSetControlValue,
        DOMMutationType::kSetInputFiles,
        DOMMutationType::kSetControlIndeterminate,
        DOMMutationType::kSetDialogModal,
        DOMMutationType::kSetPopoverOpen,
        DOMMutationType::kRequestFullscreen,
        DOMMutationType::kExitFullscreen,
        DOMMutationType::kAssignSlotNodes,
        DOMMutationType::kNotifyInlineStyleUnchanged,
        DOMMutationType::kSetTextControlInnerEditorValue,
        DOMMutationType::kSyncTextControlValueFromInnerEditor,
        DOMMutationType::kReplaceCharacterData,
        DOMMutationType::kDeleteRangeContents,
        DOMMutationType::kInsertRangeNode,
        DOMMutationType::kSetOptionSelected,
        DOMMutationType::kSetControlUserEdited,
        DOMMutationType::kSetControlAutofillState,
        DOMMutationType::kSetNonDirtyControlValue,
        DOMMutationType::kSetControlCheckednessDirty,
        DOMMutationType::kSetControlNeedsValidityCheck,
        DOMMutationType::kResetOutput,
        DOMMutationType::kSetOptionSelectedState,
        DOMMutationType::kSetOptionDirty,
        DOMMutationType::kUpdateOptionMutationObserver,
        DOMMutationType::kSetSelectLastOnChangeOption,
        DOMMutationType::kUpdateSelectSelectedcontents,
    ]
    .into_iter()
    .find(|kind| *kind as u8 == value)
    .map_or("unknown", DOMMutationTypeName)
}
