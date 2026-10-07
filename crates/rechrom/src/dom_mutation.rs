#![allow(non_snake_case)]

use dom::dom_mutation::DOMMutation;
use dom::error::{DOMException, DOMExceptionKind};
use dom::persistent_document::DOMNodeType;
use dom::{Document, DOM};
use html::html_parser::ParseHTMLFragment;

/// Apply the DOM-tree portion of Page::ApplyMutation. The Page owner remains
/// responsible for notification, connected-subtree resource preparation and
/// frame/style invalidation around this operation.
// cpp: browser/browser.cc:956-974
pub fn ApplyDOMTreeMutation(owner: &mut DOM, mutation: &DOMMutation) {
    owner
        .ApplyMutationWithFragmentParser(mutation, ParseInnerHTMLMutation)
        .unwrap_or_else(|error| std::panic::panic_any(error));
}

fn ParseInnerHTMLMutation(
    document: &mut Document,
    mutation: &DOMMutation,
) -> Result<(), dom::dom_mutation::DOMMutationError> {
    let Some(mut target) = document.FindNodeById(mutation.target_node_id) else {
        return Err(DOMException {
            kind: DOMExceptionKind::InvalidArgument,
            message: "invalid innerHTML target",
        });
    };
    if !matches!(
        document.Node(target).Type(),
        DOMNodeType::kElement | DOMNodeType::kDocumentFragment
    ) {
        return Err(DOMException {
            kind: DOMExceptionKind::InvalidArgument,
            message: "invalid innerHTML target",
        });
    }
    // Snapshot only the context node's immutable parser data: ParseHTMLFragment
    // reborrows the same arena mutably, and allocates the source synthetic
    // context and fragment there. No tree or node identity is cloned.
    let context = document.Node(target).clone();
    let fragment = ParseHTMLFragment(document, &context, &mutation.value);
    if let Some(contents) = document.TemplateContents(target) {
        target = contents;
    }
    while let Some(&child) = document.Node(target).Children().last() {
        document.Remove(child);
    }
    document.TakeAllChildren(fragment, target);
    Ok(())
}
