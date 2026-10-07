// Link the Rust layout destructor boundary retained by DOM.
use layoutng_replaced as _;

use dom::persistent_document::{DOMNamespace, DOM};
use html::html_parser::ParseHTMLFragment;

#[test]
fn fragment_from_foreign_context_keeps_its_owner_document() {
    let mut source = DOM::new();
    let context =
        source
            .GetDocumentMut()
            .CreateElement(DOMNamespace::kHTML, "div".to_owned(), Vec::new());
    let context_node = source.GetDocument().Node(context);
    let source_owner = context_node
        .OwnerDocumentNode()
        .expect("context has an owner");

    let mut target = DOM::new();
    let local =
        target
            .GetDocumentMut()
            .CreateElement(DOMNamespace::kHTML, "div".to_owned(), Vec::new());
    let local_owner = target.GetDocument().Node(local).OwnerDocumentNode();
    assert_ne!(Some(source_owner), local_owner);

    let fragment = ParseHTMLFragment(target.GetDocumentMut(), context_node, "<span>text</span>");
    let document = target.GetDocument();
    let fragment_node = document.Node(fragment);
    assert_eq!(fragment_node.OwnerDocumentNode(), Some(source_owner));
    assert!(!fragment_node.Children().is_empty());
    for &child in fragment_node.Children() {
        assert_eq!(document.Node(child).OwnerDocumentNode(), Some(source_owner));
    }
}
