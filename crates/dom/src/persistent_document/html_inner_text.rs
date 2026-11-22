//! HTMLElement::TextToFragment/setInnerText source cc:972-1008,1033-1052.
//! The caller supplies its original EventQueueScope. UTF-16 code units and
//! native Text/HTMLBRElement identities never pass through a UTF-8 projection.
use super::{DOMNamespace, DOMNodeType, PersistentDocument};
impl PersistentDocument {
    pub fn HTMLElementTextToFragment(&mut self, text: &[u16]) -> usize {
        let fragment = self.CreateDocumentFragment();
        let mut start = 0;
        while start < text.len() {
            let mut end = start;
            while end < text.len() && text[end] != b'\r' as u16 && text[end] != b'\n' as u16 {
                end += 1;
            }
            if end > start {
                let child = self.CreateTextUTF16(&text[start..end]);
                self.AppendChild(fragment, child);
            }
            if end == text.len() {
                break;
            }
            let br = self.CreateElementDefault(DOMNamespace::kHTML, "br".into());
            self.AppendChild(fragment, br);
            if text[end] == b'\r' as u16 && end + 1 < text.len() && text[end + 1] == b'\n' as u16 {
                end += 1;
            }
            start = end + 1;
        }
        fragment
    }
    pub fn SetHTMLElementInnerTextUTF16(&mut self, element: usize, text: &[u16]) {
        assert_eq!(self.Node(element).Type(), DOMNodeType::kElement);
        assert_eq!(self.Node(element).Namespace(), DOMNamespace::kHTML);
        if !text
            .iter()
            .any(|&unit| unit == b'\n' as u16 || unit == b'\r' as u16)
        {
            if text.is_empty() {
                self.RemoveAllChildrenForInnerText(element);
            } else {
                self.ReplaceChildrenWithTextUTF16(element, text);
            }
            return;
        }
        let fragment = self.HTMLElementTextToFragment(text);
        self.ReplaceChildrenWithFragment(element, fragment);
    }
}
