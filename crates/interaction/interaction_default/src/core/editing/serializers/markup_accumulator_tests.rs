use super::*;
struct Node {
    kind: MarkupNodeKind,
    name: Vec<u16>,
    ns: Vec<u16>,
    data: Vec<u16>,
    children: Vec<usize>,
    attributes: Vec<(Vec<u16>, Vec<u16>, Vec<u16>, Vec<u16>)>,
}
fn Units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn Node(kind: MarkupNodeKind, name: &str, ns: &str, data: &str, children: &[usize]) -> Node {
    Node {
        kind,
        name: Units(name),
        ns: Units(ns),
        data: Units(data),
        children: children.to_vec(),
        attributes: Vec::new(),
    }
}
struct XMLDocument(Vec<Node>);
impl MarkupDocument for XMLDocument {
    type Node = usize;
    fn IsHTMLElement(&self, _: usize) -> bool {
        false
    }
    fn Kind(&self, n: usize) -> MarkupNodeKind {
        self.0[n].kind
    }
    fn QualifiedName(&self, n: usize) -> Vec<u16> {
        self.0[n].name.clone()
    }
    fn Data(&self, n: usize) -> &[u16] {
        &self.0[n].data
    }
    fn ChildCount(&self, n: usize) -> usize {
        self.0[n].children.len()
    }
    fn ChildAt(&self, n: usize, i: usize) -> usize {
        self.0[n].children[i]
    }
    fn TemplateContents(&self, _: usize) -> Option<usize> {
        None
    }
    fn IsHTMLVoidElement(&self, _: usize) -> bool {
        false
    }
    fn TextIsRaw(&self, _: usize) -> bool {
        false
    }
    fn AttributeCount(&self, n: usize) -> usize {
        self.0[n].attributes.len()
    }
    fn Attribute(&self, n: usize, i: usize) -> MarkupAttribute<'_> {
        let (prefix, local_name, namespace, value) = &self.0[n].attributes[i];
        MarkupAttribute {
            prefix,
            local_name,
            namespace,
            value,
        }
    }
    fn ResolvedAttributeValue(&self, _: usize, _: usize, _: ResolveUrls) -> Option<Vec<u16>> {
        None
    }
    fn ElementIsValue(&self, _: usize) -> Option<&[u16]> {
        None
    }
    fn SerializationType(&self) -> SerializationType {
        SerializationType::Xml
    }
    fn ElementNamespace(&self, n: usize) -> &[u16] {
        &self.0[n].ns
    }
    fn IsCDATASection(&self, n: usize) -> bool {
        n == 6
    }
    fn DocumentTypeIdentifiers(&self, n: usize) -> (&[u16], &[u16]) {
        (&self.0[n].attributes[0].3, &self.0[n].attributes[1].3)
    }
    fn XMLDeclaration(&self) -> Option<(&[u16], &[u16], Option<bool>)> {
        Some((
            &self.0[0].attributes[0].3,
            &self.0[0].attributes[1].3,
            Some(true),
        ))
    }
}
/// Previously non-HTML documents were rejected before the native serializer.
/// This single regression protects namespace context/prefix ownership and XML
/// metadata that HTML-only traversal silently discarded.
#[test]
fn xml_serialization_preserves_namespaces_and_native_metadata() {
    use MarkupNodeKind::*;
    let mut doc = XMLDocument(vec![
        Node(Document, "", "", "", &[1, 2]),
        Node(DocumentType, "root", "", "", &[]),
        Node(Element, "root", "urn:root", "", &[3, 4, 5, 6, 7]),
        Node(Element, "p:first", "urn:one", "", &[]),
        Node(Element, "p:second", "urn:two", "", &[]),
        Node(Element, "p:third", "urn:one", "", &[]),
        Node(Text, "", "", "<raw>&", &[]),
        Node(Text, "", "", "<escaped>&\u{a0}", &[]),
    ]);
    let a = |prefix: &str, name: &str, ns: &str, value: &str| {
        (Units(prefix), Units(name), Units(ns), Units(value))
    };
    doc.0[0].attributes = vec![a("", "version", "", "1.0"), a("", "encoding", "", "UTF-8")];
    doc.0[1].attributes = vec![a("", "public", "", "public"), a("", "system", "", "system")];
    doc.0[2].attributes = vec![a("xmlns", "p", "http://www.w3.org/2000/xmlns/", "urn:one")];
    doc.0[3].attributes = vec![a("", "value", "urn:attribute", "\t\n\r\"&")];
    assert_eq!(String::from_utf16(&CreateMarkup(&doc,0,false,ResolveUrls::None)).unwrap(),
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><!DOCTYPE root PUBLIC \"public\" \"system\"><root xmlns=\"urn:root\" xmlns:p=\"urn:one\"><p:first xmlns:ns1=\"urn:attribute\" ns1:value=\"&#9;&#10;&#13;&quot;&amp;\"/><p:second xmlns:p=\"urn:two\"/><p:third/><![CDATA[<raw>&]]>&lt;escaped&gt;&amp;\u{a0}</root>");
}
