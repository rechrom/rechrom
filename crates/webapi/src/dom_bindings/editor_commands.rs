//! Document execCommand/queryCommand IDL projection. The continuation releases
//! bindings before entering the existing synchronous Interaction transaction.
use super::*;
use interaction::{DocumentEditorCommandOperation as Op, DocumentEditorCommandResult as Out};
impl DOMJavaScriptBindings {
    pub(super) fn PrepareEditorCommand(
        &self,
        receiver: u64,
        member: &str,
        arguments: &[HostValue],
    ) -> Option<HostContinuation> {
        let operation = match member {
            "__editorCommandBrand" => None,
            "execCommand" => Some(Op::Execute),
            "queryCommandSupported" => Some(Op::Supported),
            "queryCommandEnabled" => Some(Op::Enabled),
            "queryCommandState" => Some(Op::State),
            "queryCommandIndeterm" => Some(Op::Indeterm),
            "queryCommandValue" => Some(Op::Value),
            _ => return None,
        };
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let Some(index) = document
            .FindNodeById(receiver)
            .filter(|index| document.Node(*index).Type() == DOMNodeType::kDocument)
        else {
            return Some(Box::new(|_| {
                Self::type_error("Illegal Document invocation")
            }));
        };
        if operation.is_none() {
            return Some(Box::new(|_| HostResult::default()));
        }
        let html = document.DocumentType(index) == dom::persistent_document::DOMDocumentType::kHTML
            || document
                .Node(index)
                .Children()
                .iter()
                .any(|child| document.Node(*child).IsHTMLElement("html"));
        if !html {
            return Some(Box::new(|_| {
                Self::popup_error(&dom::error::DOMException {
                    kind: dom::error::DOMExceptionKind::InvalidStateError,
                    message: "Editing commands are only supported on HTML documents.",
                })
            }));
        }
        let current = index == document.Root();
        let operation = operation.unwrap();
        let name = match arguments.first() {
            Some(value) => value.clone(),
            _ => {
                return Some(Box::new(|_| {
                    Self::type_error("Editing command requires a DOMString name")
                }))
            }
        };
        let value = arguments.get(1).cloned().unwrap_or_default();
        let handler = self.services.host.document_editor_command.clone();
        let execute = self.services.host.execute_document_editor_command.clone();
        let trusted_types = self.native_abort.clone();
        Some(Box::new(move |runtime| {
            if !current {
                return Self::value(if operation == Op::Value {
                    HostValue::String(String::new())
                } else {
                    HostValue::Boolean(false)
                });
            }
            let converted = runtime.ToString(&name);
            if converted.exception.is_some() || converted.thrown.is_some() {
                return converted;
            }
            let Some(name) = runtime.PrimitiveString(&converted.value) else {
                return Self::type_error("Editing command name conversion failed");
            };
            let name = String::from_utf16_lossy(&name);
            if operation == Op::Execute {
                let Some(execute) = execute else {
                    return Self::value(HostValue::Boolean(false));
                };
                let mut check = |html_sink, runtime: &mut dyn JavaScriptHostRuntime| {
                    trusted_types.CheckExecCommandValue(&value, html_sink, runtime)
                };
                return execute(&name, &mut check, runtime);
            }
            let Some(handler) = handler else {
                return Self::value(if operation == Op::Value {
                    HostValue::String(String::new())
                } else {
                    HostValue::Boolean(false)
                });
            };
            match handler(operation, &name, "", runtime) {
                Ok(Out::Boolean(value)) => Self::value(HostValue::Boolean(value)),
                Ok(Out::Text(value)) => Self::value(HostValue::String(value)),
                Err(error) => {
                    Self::type_error(&format!("Editing command service required: {error:?}"))
                }
            }
        }))
    }
}
