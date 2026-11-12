// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Parser/serialization support at the canonical DOM owner.
use super::*;
fn html_space(c: &u16) -> bool {
    matches!(*c, 9 | 10 | 12 | 13 | 32)
}
fn trim_html(mut value: &[u16]) -> &[u16] {
    while value.first().is_some_and(html_space) {
        value = &value[1..];
    }
    while value.last().is_some_and(html_space) {
        value = &value[..value.len() - 1];
    }
    value
}
/// kurl.cc:177–179,880–890 / url_util.cc FindAndCompareScheme. Scheme
/// comparison ignores URL whitespace; no Unicode scalar conversion is needed.
fn javascript_url(value: &[u16]) -> bool {
    let mut chars = value
        .iter()
        .copied()
        .skip_while(|c| *c <= 32)
        .filter(|c| !matches!(*c, 9 | 10 | 13));
    b"javascript:".iter().all(|b| {
        chars
            .next()
            .is_some_and(|c| c <= 127 && (c as u8).eq_ignore_ascii_case(b))
    })
}
impl DOMNode {
    /// Element::IsScriptingAttribute, element.cc:4054–4095. The virtual URL
    /// classifier is shared with ordinary serialization, not duplicated here.
    pub fn IsScriptingAttributeValue(&self, attribute: &DOMAttribute, value: &[u16]) -> bool {
        let plain = attribute.namespace_uri.is_empty();
        if plain && attribute.local_name.starts_with("on") {
            return true;
        }
        if self.IsURLAttributeValue(attribute, value) && javascript_url(trim_html(value)) {
            return true;
        }
        // HTMLFrameElementBase::IsHTMLContentAttribute cc:251–255.
        if self.namespace == DOMNamespace::kHTML
            && matches!(self.name.as_str(), "iframe" | "frame")
            && plain
            && attribute.local_name == "srcdoc"
        {
            return true;
        }
        // SVGAnimateElement cc:212–231; SVGSetElement and
        // SVGAnimateTransformElement inherit this override. Other SVG classes
        // inherit Element's false virtual (element.h:1221–1224).
        if self.namespace != DOMNamespace::kSVG
            || !matches!(self.name.as_str(), "animate" | "animateTransform" | "set")
            || !plain
        {
            return false;
        }
        if matches!(attribute.local_name.as_str(), "from" | "to") {
            return javascript_url(trim_html(value));
        }
        if attribute.local_name == "values" {
            // SVGAnimationElement::ParseValues cc:58–83. Reject an empty
            // interior component, tolerate the final empty component.
            let mut parts = value.split(|c| *c == 59).peekable();
            while let Some(part) = parts.next() {
                let part = trim_html(part);
                if part.is_empty() {
                    if parts.peek().is_some() {
                        return true;
                    }
                } else if javascript_url(part) {
                    return true;
                }
            }
        }
        false
    }
}
impl PersistentDocument {
    /// ParserSetAttributes' original values and parser modification reason.
    pub fn ParserSetQualifiedAttributeUTF16(
        &mut self,
        element: usize,
        mut attribute: DOMAttribute,
        name: &foundation::QualifiedName,
        units: &[u16],
    ) {
        attribute.value = String::from_utf16_lossy(units);
        self.SetAttributeWithReasonAndInlineStyleChange(
            element,
            attribute,
            false,
            crate::attribute_lifecycle::AttributeModificationReason::ByParser,
            Some(Rc::from(units)),
            Some(name),
        );
    }
    /// HTMLConstructionSite text coalescing, preserving WTF-16 in the same
    /// CharacterData storage used by DOM, ranges, serialization and bindings.
    pub fn AppendTextUTF16(&mut self, parent: usize, data: &[u16]) -> usize {
        if let Some(&last) = self.arena[parent].children.last() {
            if self.arena[last].node_type == DOMNodeType::kText
                && !self.arena[last].IsCDATASection()
            {
                self.ReplaceDataUTF16(last, self.CharacterDataUTF16Slice(last).len(), 0, data);
                return last;
            }
        }
        let text = self.CreateTextUTF16(data);
        self.AppendChild(parent, text);
        text
    }
    pub fn InsertTextBeforeUTF16(&mut self, parent: usize, before: usize, data: &[u16]) -> usize {
        let position = self.arena[parent]
            .children
            .iter()
            .position(|c| *c == before)
            .expect("parser insertion point is a child");
        if position > 0 {
            let previous = self.arena[parent].children[position - 1];
            if self.arena[previous].node_type == DOMNodeType::kText {
                self.ReplaceDataUTF16(
                    previous,
                    self.CharacterDataUTF16Slice(previous).len(),
                    0,
                    data,
                );
                return previous;
            }
        }
        let text = self.CreateTextUTF16(data);
        self.InsertBefore(parent, text, before);
        text
    }
}
impl PersistentDocument {
    /// serialization.cc:494–501: generic temporary Document and null-name
    /// Element, sharing this arena/agent but owning the tagged subtree.
    pub fn CreateMarkupContextDocument(&mut self) -> (usize, usize) {
        let document = self.Allocate(
            DOMNodeType::kDocument,
            DOMNamespace::kNone,
            "#document".into(),
            String::new(),
            Vec::new(),
        );
        let root = self.CreateElement(DOMNamespace::kNone, String::new(), Vec::new());
        self.AppendChild(document, root);
        (document, root)
    }
    /// serialization.cc:153–181. Attribute changes are collected before any
    /// setter runs, preserving original traversal and setter ordering.
    pub fn CompleteMarkupURLs(&mut self, fragment: usize, base_url: &str) {
        let Ok(base) = url::Url::parse(base_url) else {
            return;
        };
        if base.scheme() == "javascript" {
            return;
        }
        let mut changes = Vec::new();
        let mut node = self.arena[fragment].children.first().copied();
        while let Some(current) = node {
            let element = &self.arena[current];
            if element.node_type == DOMNodeType::kElement {
                for (index, attribute) in element.attribute_utf16.iter().enumerate() {
                    if !element.IsURLAttribute(index) || attribute.value.is_empty() {
                        continue;
                    }
                    // KURL consumes scalar-value input; original WTF-16 remains
                    // untouched unless this URL attribute is committed below.
                    let scalar = String::from_utf16_lossy(&attribute.value);
                    let Ok(completed) = base.join(&scalar) else {
                        continue;
                    };
                    if completed.scheme() == "javascript" {
                        continue;
                    }
                    changes.push((
                        current,
                        index,
                        completed.as_str().encode_utf16().collect::<Vec<_>>(),
                    ));
                }
            }
            node = if let Some(first) = self.arena[current].children.first() {
                Some(*first)
            } else {
                let mut cursor = current;
                loop {
                    if cursor == fragment {
                        break None;
                    }
                    let Some(parent) = self.arena[cursor].parent else {
                        break None;
                    };
                    let children = &self.arena[parent].children;
                    let position = children
                        .iter()
                        .position(|c| *c == cursor)
                        .expect("parent contains node");
                    if let Some(next) = children.get(position + 1) {
                        break Some(*next);
                    }
                    cursor = parent;
                }
            };
        }
        for (element, index, units) in changes {
            let attribute = self.arena[element].attributes[index].clone();
            self.SetAttributeUTF16(element, attribute, &units);
        }
    }
}
impl DOMNode {
    pub fn IsSVGUseElement(&self) -> bool {
        self.svg_element_kind == Some(svg_element_kind::SVGElementKind::Use)
    }
}
impl PersistentDocument {
    /// SVGURLReferenceResolver cc:89–104. HrefString prefers SVG2 href,
    /// including an explicitly empty attribute, before legacy xlink:href.
    pub fn MarkupSvgUseReferenceIsLocalAndNotData(&self, node: usize) -> bool {
        assert!(self.arena[node].IsSVGUseElement());
        let attributes = self.arena[node].AttributesUTF16();
        let equal = |value: &[u16], ascii: &str| value.iter().copied().eq(ascii.encode_utf16());
        let href = attributes
            .iter()
            .find(|a| a.name.namespace_uri.is_empty() && equal(&a.name.local_name, "href"))
            .or_else(|| {
                attributes.iter().find(|a| {
                    equal(&a.name.namespace_uri, "http://www.w3.org/1999/xlink")
                        && equal(&a.name.local_name, "href")
                })
            })
            .map_or(&[][..], |a| a.value.as_ref());
        let scalar = String::from_utf16_lossy(href);
        let absolute = self.BaseURL().CompleteURL(&scalar);
        if absolute.as_ref().is_some_and(|url| url.scheme() == "data") {
            return false;
        }
        if href.first() == Some(&35) {
            return true;
        }
        let Some(mut absolute) = absolute else {
            return false;
        };
        let Ok(mut document) = url::Url::parse(self.DocumentURL()) else {
            return false;
        };
        absolute.set_fragment(None);
        document.set_fragment(None);
        absolute == document
    }
}
impl PersistentDocument {
    /// StyleElement::ChildrenChanged -> Element::TextFromChildren. This is
    /// the stylesheet's original WTF-16 input, not serialized DOM markup.
    pub fn StyleElementTextForParsing(&self, node: usize) -> Rc<[u16]> {
        assert!(self.arena[node].IsStyleElement());
        fn append(document: &PersistentDocument, node: usize, out: &mut Vec<u16>) {
            if document.arena[node].node_type == DOMNodeType::kText {
                out.extend_from_slice(document.CharacterDataUTF16Slice(node));
            }
            for child in &document.arena[node].children {
                append(document, *child, out);
            }
        }
        let mut text = Vec::new();
        append(self, node, &mut text);
        text.into()
    }
}
