// Link the Rust layout destructor boundary retained by DOM.
use layout_algo_replaced as _;

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

#[test]
fn paste_policy_strips_before_setting_and_preserves_wtf16() {
    use html::{
        markup_fragment::CreateFragmentFromMarkup,
        parser::parser_content_policy::ParserContentPolicy,
    };
    let mut tree = DOM::new();
    let document = tree.GetDocumentMut();
    let mut markup: Vec<u16> = "<div onclick=x title='".encode_utf16().collect();
    markup.extend_from_slice(&[0xd800]);
    markup.extend("'>".encode_utf16());
    markup.extend_from_slice(&[0xdc00]);
    markup.extend("<script/>secret</script><object>fallback</object><embed><a href='javascript:x' href='safe'>link</a></div>".encode_utf16());
    let fragment = CreateFragmentFromMarkup(
        document,
        &markup,
        "",
        ParserContentPolicy::DisallowScriptingAndPluginContent,
        false,
    );
    let div = document.Node(fragment).Children()[0];
    assert!(document.Node(div).AttributeValueUTF16("onclick").is_none());
    assert_eq!(
        document
            .Node(div)
            .AttributeValueUTF16("title")
            .unwrap()
            .as_ref(),
        &[0xd800]
    );
    let children = document.Node(div).Children();
    assert_eq!(
        document.CharacterDataUTF16Slice(children[0]),
        &[0xdc00, 102, 97, 108, 108, 98, 97, 99, 107]
    );
    assert_eq!(children.len(), 2);
    let anchor = children[1];
    assert!(document.Node(anchor).IsHTMLElement("a"));
    // Token duplicates are removed before script attributes are stripped;
    // a later duplicate never replaces the discarded first attribute.
    assert!(document.Node(anchor).AttributeValueUTF16("href").is_none());

    let mut formatting: Vec<u16> = "<p><b title='".encode_utf16().collect();
    formatting.push(0xd800);
    formatting.extend("'>first</p>second".encode_utf16());
    let fragment = CreateFragmentFromMarkup(
        document,
        &formatting,
        "",
        ParserContentPolicy::DisallowScriptingAndPluginContent,
        false,
    );
    let reconstructed = *document.Node(fragment).Children().last().unwrap();
    assert!(document.Node(reconstructed).IsHTMLElement("b"));
    assert_eq!(
        document
            .Node(reconstructed)
            .AttributeValueUTF16("title")
            .unwrap()
            .as_ref(),
        &[0xd800]
    );
}

#[test]
fn context_fragment_retains_table_and_uses_code_unit_offsets() {
    use html::{
        markup_fragment::CreateFragmentFromMarkupWithContext,
        parser::parser_content_policy::ParserContentPolicy,
    };
    let mut tree = DOM::new();
    let document = tree.GetDocumentMut();
    let prefix = "<table><tr>";
    let suffix = "<td>discard</td></tr></table>";
    let mut markup: Vec<u16> = prefix.encode_utf16().collect();
    let start = markup.len();
    markup.extend("<td>".encode_utf16());
    markup.extend_from_slice(&[0xd800, 120]);
    markup.extend("</td>".encode_utf16());
    let end = markup.len();
    markup.extend(suffix.encode_utf16());
    let fragment = CreateFragmentFromMarkupWithContext(
        document,
        &markup,
        start,
        end,
        "",
        ParserContentPolicy::DisallowScriptingAndPluginContent,
        false,
    )
    .unwrap()
    .unwrap();
    assert!(document
        .Node(document.Node(fragment).Children()[0])
        .IsHTMLElement("table"));
    // Markers and material outside the original code-unit range are gone.
    fn text(document: &dom::Document, node: usize, out: &mut Vec<u16>) {
        if document.Node(node).Type() == dom::persistent_document::DOMNodeType::kText {
            out.extend_from_slice(document.CharacterDataUTF16Slice(node));
        }
        for child in document.Node(node).Children() {
            text(document, *child, out);
        }
    }
    let mut result = Vec::new();
    text(document, fragment, &mut result);
    assert_eq!(result, &[0xd800, 120]);
}
