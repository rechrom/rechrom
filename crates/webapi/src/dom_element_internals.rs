//! Web IDL projection of the target-owned ElementInternals data.
use super::*;
impl DOMJavaScriptBindings {
    pub(super) fn CallElementInternals(
        &mut self,
        receiver: u64,
        member: &str,
        arguments: &[HostValue],
    ) -> HostResult {
        let mut owner = self.document.borrow_mut();
        let d = owner.GetDocumentMut();
        let Some(i) = d.FindNodeById(receiver) else {
            return Self::type_error("Illegal invocation");
        };
        if member == "__internalsAttach" {
            return match d.AttachElementInternals(i) {
                Ok(()) => HostResult::default(),
                Err(error) => Self::popup_error(&error),
            };
        }
        if !member.starts_with("__internalsState") && !d.ElementInternalsTargetIsFormAssociated(i) {
            return Self::popup_error(&dom::error::DOMException {
                kind: dom::error::DOMExceptionKind::NotSupportedError,
                message: "The target is not a form-associated custom element.",
            });
        }
        match member {
            "__internalsBrand" => HostResult::default(),
            "__internalsForm" => Self::value(d.FormOwner(i).map_or(
                HostValue::Null(JavaScriptNull),
                |form| {
                    HostValue::Object(HostObjectRef {
                        id: d.Node(form).Id(),
                    })
                },
            )),
            "__internalsLabels" => {
                drop(owner);
                Self::value(HostValue::Object(self.collection(receiver, "labels", "")))
            }
            "__internalsSetFormValue" => {
                let Some(value) = arguments
                    .first()
                    .and_then(|v| self.native_abort.ElementInternalsValue(v))
                else {
                    return Self::type_error("Invalid form value");
                };
                let state = if let Some(state) = arguments.get(1) {
                    let Some(state) = self.native_abort.ElementInternalsValue(state) else {
                        return Self::type_error("Invalid form state");
                    };
                    state
                } else {
                    value.clone()
                };
                d.SetElementInternalsFormValue(i, value, state);
                HostResult::default()
            }
            "__internalsSetValidity" => {
                let [HostValue::Number(flags), message, anchor] = arguments else {
                    return Self::type_error("Invalid validity arguments");
                };
                let message: Rc<[u16]> = match message {
                    HostValue::String16(message) => message.clone(),
                    HostValue::String(message) => message.encode_utf16().collect::<Vec<_>>().into(),
                    _ => return Self::type_error("Validity message conversion required"),
                };
                let flags = *flags as u16;
                if flags != 0 && message.is_empty() {
                    return Self::type_error(
                        "A nonempty validation message is required for invalid flags",
                    );
                }
                let anchor = match anchor {
                    HostValue::Null(_) => None,
                    HostValue::Object(anchor) => match d.FindNodeById(anchor.id) {
                        Some(anchor)
                            if d.Node(anchor).Type() == DOMNodeType::kElement
                                && d.Node(anchor).Namespace() == DOMNamespace::kHTML =>
                        {
                            Some(anchor)
                        }
                        _ => return Self::type_error("Validation anchor requires an HTMLElement"),
                    },
                    _ => return Self::type_error("Validation anchor requires an HTMLElement"),
                };
                match d.SetElementInternalsValidity(i, flags, message, anchor) {
                    Ok(()) => HostResult::default(),
                    Err(error) => Self::popup_error(&error),
                }
            }
            "__internalsStateAt" => {
                let Some(HostValue::Number(index)) = arguments.first() else {
                    return Self::type_error("State index required");
                };
                Self::value(
                    d.ElementInternalsStateAt(i, *index as usize)
                        .map_or(HostValue::Null(JavaScriptNull), HostValue::String),
                )
            }
            "__internalsStateHas" | "__internalsStateAdd" | "__internalsStateDelete" => {
                let Some(HostValue::String(state)) = arguments.first() else {
                    return Self::type_error("State string required");
                };
                if member == "__internalsStateHas" {
                    return Self::value(HostValue::Boolean(d.ElementInternalsHasState(i, state)));
                }
                let index = d.ElementInternalsStateIndex(i, state);
                let changed = d.ChangeElementInternalsState(
                    i,
                    Some(state.clone()),
                    member == "__internalsStateAdd",
                );
                Self::value(if member == "__internalsStateDelete" {
                    HostValue::Number(index.map_or(-1.0, |index| index as f64))
                } else {
                    HostValue::Boolean(changed)
                })
            }
            "__internalsStateClear" => {
                d.ChangeElementInternalsState(i, None, false);
                HostResult::default()
            }
            _ => Self::type_error("Unknown ElementInternals operation"),
        }
    }
}
