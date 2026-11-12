//! Node/HTMLElement facts consumed by FindBuffer, source find_buffer.cc:70-108.
//! Class and rare-data reads stay on the original DOM element.
use crate::persistent_document::DisplayLockActivationReason;
use crate::persistent_document::{DOMNamespace, DOMNodeType, PersistentDocument};
pub use foundation::find_buffer::{FindBufferNodeFacts, FindHTMLClass};
impl PersistentDocument {
    pub fn FindBufferNodeFacts(&self, node: usize) -> FindBufferNodeFacts {
        let owner = self.Node(node);
        let html_class = (owner.Type() == DOMNodeType::kElement
            && owner.Namespace() == DOMNamespace::kHTML)
            .then(|| match owner.Name() {
                "input" => FindHTMLClass::Input,
                "iframe" => FindHTMLClass::IFrame,
                "img" => FindHTMLClass::Image,
                "meter" => FindHTMLClass::Meter,
                "object" => FindHTMLClass::Object,
                "progress" => FindHTMLClass::Progress,
                "select" => FindHTMLClass::Select,
                "style" => FindHTMLClass::Style,
                "script" => FindHTMLClass::Script,
                "video" => FindHTMLClass::Video,
                "audio" => FindHTMLClass::Audio,
                "wbr" => FindHTMLClass::WBR,
                "br" => FindHTMLClass::BR,
                _ => FindHTMLClass::Other,
            });
        // HTMLElement::ShouldSerializeEndTag, html_element.cc:288-303.
        let serializes_end_tag = !matches!(
            owner.Name(),
            "area"
                | "base"
                | "basefont"
                | "bgsound"
                | "br"
                | "col"
                | "embed"
                | "frame"
                | "hr"
                | "img"
                | "input"
                | "keygen"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        );
        let text_control = self.EditingNodeFacts(node).is_text_control;
        FindBufferNodeFacts {
            is_element: owner.Type() == DOMNodeType::kElement,
            comment: owner.Type() == DOMNodeType::kComment,
            html_class,
            serializes_end_tag,
            text_control,
            suggested_value_nonempty: text_control
                && self
                    .SuggestedValue(node)
                    .is_some_and(|value| !value.is_empty()),
            select_uses_menu_list: html_class == Some(FindHTMLClass::Select)
                && self.SelectUsesMenuList(node),
            locked: self.DisplayLockIsLocked(node),
            find_activatable: self
                .DisplayLockIsActivatable(node, DisplayLockActivationReason::FindInPage),
        }
    }
}
