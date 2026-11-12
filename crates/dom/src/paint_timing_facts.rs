//! Node.cc2663-2690 and TextElementTiming.cc27-34. These are canonical DOM
//! facts consumed by timing, not a Browser selector/HTML serialization path.
use crate::persistent_document::{DOMNamespace, DOMNodeType, PersistentDocument};
#[derive(Debug)]
pub enum PaintTimingDOMError {
    ContainerTimingAttributionOwnerUnavailable,
}
impl PersistentDocument {
    pub fn TextPaintNeedsElementTiming(&self, node: usize) -> Result<bool, PaintTimingDOMError> {
        let element = self.Node(node);
        if element.Type() != DOMNodeType::kElement || self.IsInShadowTree(node) {
            return Ok(false);
        }
        if element.AttributeValueUTF16("elementtiming").is_some() {
            return Ok(true);
        }
        // ContainerTiming is a distinct origin-trial/experimental client.
        // Its attribution owner must be installed before admitting that
        // feature; a missing tracker is not a negative attribution result.
        if foundation::RuntimeEnabledFeatures::ContainerTimingEnabled() {
            return Err(PaintTimingDOMError::ContainerTimingAttributionOwnerUnavailable);
        }
        Ok(false)
    }
    pub fn PaintTimingDebugName(&self, node: usize) -> String {
        let source = self.Node(node);
        let mut name: Vec<u16> = if source.IsCDATASection() {
            "#cdata-section".encode_utf16().collect()
        } else {
            match source.Type() {
                DOMNodeType::kDocument => "#document".encode_utf16().collect(),
                DOMNodeType::kText => "#text".encode_utf16().collect(),
                DOMNodeType::kComment => "#comment".encode_utf16().collect(),
                DOMNodeType::kDocumentFragment => "#document-fragment".encode_utf16().collect(),
                DOMNodeType::kElement
                    if source.Namespace() == DOMNamespace::kHTML
                        && self.DocumentType(self.Root())
                            == crate::persistent_document::DOMDocumentType::kHTML =>
                {
                    source
                        .NameUTF16()
                        .iter()
                        .map(|unit| {
                            if (b'a' as u16..=b'z' as u16).contains(unit) {
                                unit - 32
                            } else {
                                *unit
                            }
                        })
                        .collect()
                }
                _ => source.NameUTF16().to_vec(),
            }
        };
        if source.Type() == DOMNodeType::kElement {
            if let Some(id) = source.AttributeValueUTF16("id").filter(|id| !id.is_empty()) {
                name.extend(" id='".encode_utf16());
                name.extend(id.iter().copied());
                name.push(39);
            }
            if let Some(class) = source.AttributeValueUTF16("class") {
                let mut tokens: Vec<&[u16]> = Vec::new();
                for token in class
                    .split(|unit| matches!(*unit, 9 | 10 | 12 | 13 | 32))
                    .filter(|token| !token.is_empty())
                {
                    if !tokens.contains(&token) {
                        tokens.push(token);
                    }
                }
                if !tokens.is_empty() {
                    name.extend(" class='".encode_utf16());
                    for (index, token) in tokens.iter().enumerate() {
                        if index != 0 {
                            name.push(32);
                        }
                        name.extend(token.iter().copied());
                    }
                    name.push(39);
                }
            }
        }
        // TracedValue exports UTF-8, while source DOM attributes above retain
        // their exact UTF-16 owners throughout the native query.
        String::from_utf16_lossy(&name)
    }
}
