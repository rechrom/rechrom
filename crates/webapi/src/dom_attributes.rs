//! Native identity and property access for installed ordinary IDL descriptors.
//!
//! Blink: bindings/scripts/bind_gen/interface.py (member location/receiver),
//! platform/bindings/idl_member_installer.cc (SetAccessorProperty/signature),
//! core/dom/{node,element,character_data}.idl and core/html/forms/*.idl.
//! The JavaScript engine owns property resolution; the DOM remains the only
//! owner of node identity, attributes, control state and typed mutations.
use super::*;

impl DOMJavaScriptBindings {
    pub(super) fn UsesOrdinaryNodeProperties(&self, receiver: HostObjectId) -> bool {
        // Prototype registration marks the installed public binding surface.
        // Preserve the raw host protocol for embedders before bootstrap only.
        self.prototypes.contains_key("Node") && self.node(receiver).is_some()
    }

    fn ImplementsDOMInterface(&self, receiver: HostObjectId, interface: &str) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let Some(index) = document.FindNodeById(receiver) else {
            return false;
        };
        let node = document.Node(index);
        match interface {
            "Node" => true,
            "Element" => node.Type() == DOMNodeType::kElement,
            "HTMLElement" => {
                node.Type() == DOMNodeType::kElement && node.Namespace() == DOMNamespace::kHTML
            }
            "SVGElement" => {
                node.Type() == DOMNodeType::kElement && node.Namespace() == DOMNamespace::kSVG
            }
            "MathMLElement" => {
                node.Type() == DOMNodeType::kElement && node.Namespace() == DOMNamespace::kMathML
            }
            "Document" => node.Type() == DOMNodeType::kDocument,
            "DocumentFragment" => node.Type() == DOMNodeType::kDocumentFragment,
            "ShadowRoot" => document.ShadowHost(index).is_some(),
            "CharacterData" => is_character_data(node.Type()),
            "HTMLInputElement" => node.IsHTMLElement("input"),
            "HTMLTextAreaElement" => node.IsHTMLElement("textarea"),
            "HTMLSelectElement" => node.IsHTMLElement("select"),
            "HTMLImageElement" => node.IsHTMLElement("img"),
            "HTMLTemplateElement" => node.IsHTMLElement("template"),
            _ => false,
        }
    }

    pub(super) fn NativeAttributeCall(
        &mut self,
        receiver: HostObjectId,
        operation: &str,
        arguments: &[HostValue],
    ) -> HostResult {
        if operation == "__domHasInterface" {
            let implements = match arguments {
                [HostValue::Object(object), HostValue::String(interface), ..] => {
                    self.ImplementsDOMInterface(object.id, interface)
                }
                _ => false,
            };
            return Self::value(HostValue::Boolean(implements));
        }
        let [HostValue::String(interface), HostValue::String(member), rest @ ..] = arguments else {
            return Self::type_error("Invalid native attribute invocation");
        };
        if !self.ImplementsDOMInterface(receiver, interface) {
            return Self::type_error("Illegal invocation");
        }
        // No second state owner and no fallback after a selected native member:
        // use the existing getter/typed setter and preserve its exception.
        let result = if operation == "__domAttributeGet" {
            self.get(receiver, member)
        } else {
            self.set(receiver, member, rest)
        };
        if result.handled {
            result
        } else {
            Self::type_error("Native attribute is not implemented")
        }
    }
}
