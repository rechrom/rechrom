//! Canonical UA elements of the temporal input view. Source constructors:
//! multiple_fields_temporal_input_type_view.cc358-383;
//! date_time_edit_element.cc503-510,730-737; field_element.cc174-186.
use super::*;
impl PersistentDocument {
    /// Allocate the actual source element, without attaching it before the
    /// source constructor has installed its class/pseudo/ARIA state.
    pub fn CreateTemporalShadowElement(&mut self, host: usize, tag: &str) -> usize {
        assert!(matches!(
            self.InputType(host),
            Some(
                DOMInputType::kDate
                    | DOMInputType::kDateTimeLocal
                    | DOMInputType::kMonth
                    | DOMInputType::kTime
                    | DOMInputType::kWeek
            )
        ));
        self.CreateElementDefault(DOMNamespace::kHTML, tag.to_owned())
    }
    pub fn DestroyTemporalInputShadowSubtree(&mut self, host: usize) {
        self.DestroyInputTypeShadowSubtree(host);
        self.ua_shadow_pseudo_elements.remove(&host);
    }
    pub fn SetTemporalShadowPseudoId(&mut self, host: usize, element: usize, pseudo: &str) {
        self.SetUAShadowPseudoElementIdentity(host, element, pseudo);
    }
}

impl PersistentDocument {
    /// Element::ComputeInheritedLanguage, element.cc10600-10628. Empty is a
    /// specified language value; only absent attributes continue inheritance.
    pub fn ComputeInheritedLanguage(&self, element: usize) -> Rc<[u16]> {
        let mut current = Some(element);
        while let Some(node) = current {
            let n = self.Node(node);
            if n.Type() == DOMNodeType::kElement {
                let attrs = n.Attributes();
                if let Some(index) = attrs.iter().position(|a| {
                    a.namespace_uri == "http://www.w3.org/XML/1998/namespace"
                        && a.local_name == "lang"
                }) {
                    return n.AttributesUTF16()[index].value.clone();
                }
                if matches!(n.Namespace(), DOMNamespace::kHTML | DOMNamespace::kSVG) {
                    if let Some(index) = attrs
                        .iter()
                        .position(|a| a.namespace_uri.is_empty() && a.local_name == "lang")
                    {
                        return n.AttributesUTF16()[index].value.clone();
                    }
                }
            } else if n.Type() == DOMNodeType::kDocument {
                return self.ContentLanguage();
            }
            current = n.Parent().or_else(|| self.ShadowHost(node));
        }
        Rc::from([])
    }
    /// HTMLFormControlElementWithState::SetUserHasEditedTheFieldAndBlurred,
    /// cc377-383. Both user validity pseudos depend on the same retained state.
    pub fn SetUserHasEditedTheFieldAndBlurred(&mut self, node: usize) {
        let id = self.Node(node).Id();
        let state = self.text_control_runtime.entry(id).or_default();
        if state.user_interaction >= 2 {
            return;
        }
        state.user_interaction = 2;
        self.ControlPseudoStateChanged(node, crate::style_state::ControlPseudoState::UserInvalid);
        self.ControlPseudoStateChanged(node, crate::style_state::ControlPseudoState::UserValid);
    }
}
