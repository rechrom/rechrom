//! HTMLInputElement/HTMLTextAreaElement selection API projection.
//!
//! Chromium: core/html/forms/text_control_element.{h,cc}. WebAPI converts the
//! IDL values; Interaction retains and mutates the canonical selection owner.

use super::*;

impl DOMJavaScriptBindings {
    pub(super) fn PrepareTextControlSelection(
        &self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> Option<HostContinuation> {
        if !matches!(
            member,
            "__textControlSelectionSnapshot" | "__textControlSetSelectionRange"
        ) {
            return None;
        }
        let host = self.services.host.text_control_selection.clone();
        let result = match member {
            "__textControlSelectionSnapshot" => {
                let snapshot = host.as_ref().and_then(|host| (host.read)(receiver));
                match snapshot {
                    Some(snapshot) => Self::value(HostValue::Record(vec![
                        ("start".into(), HostValue::Number(snapshot.start as f64)),
                        ("end".into(), HostValue::Number(snapshot.end as f64)),
                        (
                            "direction".into(),
                            HostValue::String(if snapshot.backward {
                                "backward".into()
                            } else {
                                "forward".into()
                            }),
                        ),
                    ])),
                    None => Self::value(HostValue::Null(JavaScriptNull)),
                }
            }
            "__textControlSetSelectionRange" => {
                let [HostValue::Number(start), HostValue::Number(end), HostValue::String(direction)] =
                    arguments
                else {
                    return Some(Box::new(|_| {
                        Self::type_error("setSelectionRange requires start, end and direction")
                    }));
                };
                let accepted = host.as_ref().is_some_and(|host| {
                    (host.write)(
                        receiver,
                        *start as usize,
                        *end as usize,
                        direction == "backward",
                    )
                });
                if accepted {
                    Self::value(HostValue::default())
                } else {
                    Self::popup_error(&dom::error::DOMException {
                        kind: dom::error::DOMExceptionKind::InvalidStateError,
                        message: "The input element's type does not support selection.",
                    })
                }
            }
            _ => unreachable!(),
        };
        Some(Box::new(move |_| result))
    }
}
