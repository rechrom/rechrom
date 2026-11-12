//! HTML virtual leaves consumed by the rendered TextIterator.
use super::{DOMNodeClass, PersistentDocument};
pub trait InputAltTextLocale {
    fn QueryFormInputAlt(&self, node_id: u64) -> foundation::String;
}
impl PersistentDocument {
    // Source: html_element.h:262; html_image_element.cc:412-420;
    // html_input_element.cc:1101-1114. The localized IDS_FORM_INPUT_ALT value
    // belongs to the embedding's Locale owner, and is supplied unchanged.
    pub fn TextIteratorAltText(
        &self,
        node: usize,
        locale: &dyn InputAltTextLocale,
    ) -> foundation::String {
        let image = matches!(&self.arena[node].class, DOMNodeClass::HTMLImageElement);
        let input = self.arena[node].class.IsHTMLInputElement();
        if !image && !input {
            return foundation::String::default();
        }
        let element = self.Node(node);
        for name in ["alt", "title"] {
            if let Some(attribute) = element.FindAttribute(name) {
                return foundation::String::FromUtf8(attribute.value.as_bytes());
            }
        }
        if input {
            if let Some(attribute) = element.FindAttribute("value") {
                return foundation::String::FromUtf8(attribute.value.as_bytes());
            }
            return locale.QueryFormInputAlt(element.Id());
        }
        foundation::String::default()
    }
}
