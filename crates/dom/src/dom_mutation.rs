#![allow(non_snake_case)]

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
    kAttachShadowRoot,
    kCloneNode,
    kImportNode,
    kAppendChild,
    kInsertBefore,
    kRemoveChild,
    #[default]
    kSetAttribute,
    kRemoveAttribute,
    kSetTextContent,
    kSetInnerHTML,
    kSetControlChecked,
    kSetControlValue,
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
}

// cpp: dom/dom_mutation.h:40-51
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DOMMutation {
    pub mutation_type: DOMMutationType,
    pub target_node_id: u64,
    pub name: String,
    pub value: String,
    pub namespace_uri: String,
    pub bool_value: bool,
    /// CSSOM's existing-declaration mutation classification for a null-namespace
    /// style attribute. Generic setAttribute/cssText writes leave this false.
    pub inline_style_independent: bool,
    /// Parsed declaration ownership travels with a CSSOM style write, before
    /// mutation callbacks can reenter the document. Concrete types stay private.
    pub inline_style: Option<cssom::CSSInlineStyle>,
    pub child_node_id: u64,
    /// Source node for an import into target_node_id (the destination document).
    pub source_node_id: u64,
    pub before_node_id: u64,
    /// Ordered native identities for atomic HTMLSlotElement.assign().
    pub node_ids: Vec<u64>,
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
                let mut ancestor = Some(target);
                while let Some(index) = ancestor {
                    if index == child {
                        return Err(DOMException {
                            kind: DOMExceptionKind::InvalidArgument,
                            message: "insertion would create a cycle",
                        });
                    }
                    ancestor = document.Node(index).Parent();
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
        | DOMMutationType::kNotifyInlineStyleUnchanged => {
            if document.Node(target).Type() != DOMNodeType::kElement {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "DOM attribute mutation target is not an element",
                });
            }
            if mutation.name.is_empty() {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "DOM attribute mutation name is empty",
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
            if !document.IsPopover(target) {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "popover state mutation target is not a popover",
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
        DOMMutationType::kSetOptionSelected => {
            if !document.Node(target).IsHTMLElement("option") {
                return Err(DOMException {
                    kind: DOMExceptionKind::InvalidArgument,
                    message: "selected mutation target is not an option",
                });
            }
        }
        DOMMutationType::kSetControlValue => {
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
                let namespace = match mutation.namespace_uri.as_str() {
                    "http://www.w3.org/2000/svg" => DOMNamespace::kSVG,
                    "http://www.w3.org/1998/Math/MathML" => DOMNamespace::kMathML,
                    _ => DOMNamespace::kHTML,
                };
                document.CreateElementDefault(namespace, mutation.name.clone());
            }
            DOMMutationType::kCreateText => {
                document.CreateText(mutation.value.clone());
            }
            DOMMutationType::kCreateComment => {
                document.CreateComment(mutation.value.clone());
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
                debug_assert_eq!(document.Node(root).Id(), mutation.child_node_id);
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
                document.SetAttributeWithInlineStyleChange(
                    target,
                    DOMAttribute {
                        local_name: mutation.name.clone(),
                        namespace_uri: mutation.namespace_uri.clone(),
                        value: mutation.value.clone(),
                        ..DOMAttribute::default()
                    },
                    mutation.inline_style_independent,
                );
                if mutation.name == "style" && mutation.namespace_uri.is_empty() {
                    if let Some(style) = &mutation.inline_style {
                        document.Node(target).RetainInlineStyle(style.clone());
                    }
                }
            }
            DOMMutationType::kRemoveAttribute => {
                document.RemoveAttribute(target, &mutation.name, &mutation.namespace_uri);
            }
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
            DOMMutationType::kSetTextContent => {
                document.SetTextContent(target, mutation.value.clone());
            }
            DOMMutationType::kSetInnerHTML => {}
            DOMMutationType::kSetControlChecked => {
                document.SetControlChecked(target, mutation.bool_value);
            }
            DOMMutationType::kSetControlIndeterminate => {
                document.SetControlIndeterminate(target, mutation.bool_value);
            }
            DOMMutationType::kSetDialogModal => {
                if mutation.bool_value {
                    document.ShowModalDialog(target);
                } else {
                    document.CloseDialog(target);
                }
            }
            DOMMutationType::kSetPopoverOpen => {
                if mutation.bool_value {
                    document.ShowPopover(target);
                } else {
                    document.HidePopover(target);
                }
            }
            DOMMutationType::kRequestFullscreen => {
                document.RequestFullscreen(target);
            }
            DOMMutationType::kExitFullscreen => {
                document.ExitFullscreen();
            }
            DOMMutationType::kSetOptionSelected => {
                document.SetOptionSelected(target, mutation.bool_value);
            }
            DOMMutationType::kSetControlValue => {
                document.SetControlValue(target, mutation.value.clone());
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
        DOMMutationType::kAttachShadowRoot => "attach-shadow-root",
        DOMMutationType::kCloneNode => "clone-node",
        DOMMutationType::kImportNode => "import-node",
        DOMMutationType::kAppendChild => "append-child",
        DOMMutationType::kInsertBefore => "insert-before",
        DOMMutationType::kRemoveChild => "remove-child",
        DOMMutationType::kSetAttribute => "set-attribute",
        DOMMutationType::kRemoveAttribute => "remove-attribute",
        DOMMutationType::kSetTextContent => "set-text-content",
        DOMMutationType::kSetInnerHTML => "set-inner-html",
        DOMMutationType::kSetControlChecked => "set-control-checked",
        DOMMutationType::kSetControlIndeterminate => "set-control-indeterminate",
        DOMMutationType::kSetControlValue => "set-control-value",
        DOMMutationType::kSetOptionSelected => "set-option-selected",
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
        DOMMutationType::kSetControlIndeterminate,
        DOMMutationType::kSetDialogModal,
        DOMMutationType::kSetPopoverOpen,
        DOMMutationType::kRequestFullscreen,
        DOMMutationType::kExitFullscreen,
        DOMMutationType::kAssignSlotNodes,
        DOMMutationType::kNotifyInlineStyleUnchanged,
        DOMMutationType::kSetOptionSelected,
    ]
    .into_iter()
    .find(|kind| *kind as u8 == value)
    .map_or("unknown", DOMMutationTypeName)
}
