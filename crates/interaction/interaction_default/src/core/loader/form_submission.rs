//! Chromium core/loader/form_submission.cc:83-210,214-435,449-488.
//! Form encoding and target lookup retain original source owners. A pending
//! auxiliary lookup resumes the same Create operation; it never becomes a
//! guessed local target or a new URL-only request.
use super::{
    frame_load_request::*,
    navigation_policy::{self, NavigationEventModifiers},
    resource_request::*,
};
use crate::core::html::{
    forms::html_form_element::{
        FormControlRef, FormDispatchScope, FormSubmission, SubmissionRef, SubmissionTargetRef,
        SubmitMethod,
    },
    html_anchor_element::AnchorFrameLoadRequest,
};
use interaction_dispatch::core::{
    dom::events::event::{EventRef, EventTargetRef},
    fileapi::{
        form_data::{EncodeStringAsFormData, FormData},
        form_data_object::FormDataObject,
    },
};
use navigation_history::FrameLoadType;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};
#[derive(Clone)]
pub struct FormSubmissionAttributes {
    pub method: SubmitMethod,
    pub action: String,
    pub target: String,
    pub encoding_type: String,
    pub accept_charset: String,
    pub relations: u32,
}
impl Default for FormSubmissionAttributes {
    fn default() -> Self {
        Self {
            method: SubmitMethod::Get,
            action: String::new(),
            target: String::new(),
            encoding_type: "application/x-www-form-urlencoded".into(),
            accept_charset: String::new(),
            relations: 0,
        }
    }
}
impl FormSubmissionAttributes {
    pub fn ParseAttribute(&mut self, name: &str, value: Option<&str>) {
        let value = value.unwrap_or("");
        match name {
            "method" => self.method = Self::ParseMethod(value),
            "action" => {
                self.action = value
                    .trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c'))
                    .into()
            }
            "target" => self.target = value.into(),
            "enctype" => self.encoding_type = Self::ParseEncodingType(value).into(),
            "accept-charset" => self.accept_charset = value.into(),
            "rel" => {
                self.relations = 0;
                for token in value.split_ascii_whitespace() {
                    match token.to_ascii_lowercase().as_str() {
                        "noreferrer" => self.relations |= 0x1000,
                        "noopener" => self.relations |= 0x40000,
                        "opener" => self.relations |= 0x80000,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    pub fn ParseMethod(value: &str) -> SubmitMethod {
        if value.eq_ignore_ascii_case("post") {
            SubmitMethod::Post
        } else if value.eq_ignore_ascii_case("dialog") {
            SubmitMethod::Dialog
        } else {
            SubmitMethod::Get
        }
    }
    pub fn ParseEncodingType(value: &str) -> &'static str {
        if value.eq_ignore_ascii_case("multipart/form-data") {
            "multipart/form-data"
        } else if value.eq_ignore_ascii_case("text/plain") {
            "text/plain"
        } else {
            "application/x-www-form-urlencoded"
        }
    }
}
pub enum SubmitterOverrides {
    /// Native HTMLFormControlElement presence distinguishes null from empty.
    Native {
        action: Option<String>,
        enctype: Option<String>,
        method: Option<String>,
        target: Option<String>,
    },
    Behavior {
        action: String,
        enctype: String,
        method: String,
        target: String,
    },
}
/// All methods project one existing Document/Window/form, at their original
/// source call sites. This collaborator is constructed internally by the form
/// factory; it is not a host-supplied submission/navigation algorithm.
pub trait FormSubmissionEnvironment {
    fn FormTarget(&self) -> EventTargetRef;
    fn Attributes(&self) -> FormSubmissionAttributes;
    fn SubmitterOverrides(&self, submitter: &FormControlRef) -> Option<SubmitterOverrides>;
    fn DialogResult(&self, submitter: Option<&FormControlRef>) -> String;
    fn DocumentURL(&self) -> String;
    fn CompleteURL(&self, value: &str) -> Option<String>;
    fn PrepareActionURL(
        &self,
        url: &str,
    ) -> foundation::content_security_policy::CSPRequestPreparation;
    fn CountUse(&self, feature: &'static str);
    fn DocumentEncoding(&self) -> String;
    fn BaseTarget(&self) -> String;
    fn ConstructEntryList(
        &self,
        submitter: Option<FormControlRef>,
        encoding: &str,
        scope: &FormDispatchScope<'_>,
    ) -> Option<Rc<FormDataObject>>;
    fn RandomBoundaryBytes(&self) -> [u8; 16];
    fn FrameEnvironment(&self) -> Rc<dyn FrameLoadRequestEnvironment>;
    fn CaptureInitiator(&self, scope: &FormDispatchScope<'_>) -> NavigationInitiator;
    fn CurrentInputModifiers(&self) -> Option<NavigationEventModifiers>;
    fn EventModifiers(&self, event: &EventRef) -> Option<NavigationEventModifiers>;
    fn IsMac(&self) -> bool;
    fn NewTabModifierSetFromIsolatedWorld(&self) -> bool;
    fn ResolveTarget(
        &self,
        request: Rc<FrameLoadRequest>,
        target: String,
        completion: Box<dyn FnOnce(Option<SubmissionTargetRef>)>,
    );
}
pub struct FormSubmissionOwner {
    environment: Rc<dyn FormSubmissionEnvironment>,
    method: SubmitMethod,
    action: String,
    target: String,
    result: String,
    resource: Option<Rc<ResourceRequest>>,
    prepared_request: Option<Rc<FrameLoadRequest>>,
    target_frame: RefCell<Option<SubmissionTargetRef>>,
    load_type: Cell<FrameLoadType>,
    /// Data remains the same owner exposed to the formdata event. Encoding
    /// freezes only the final request bytes, once that dispatch has returned.
    form_data: Option<Arc<interaction_dispatch::core::fileapi::encoded_form_data::EncodedFormData>>,
}
impl FormSubmissionOwner {
    pub fn Create(
        environment: Rc<dyn FormSubmissionEnvironment>,
        event: Option<EventRef>,
        submitter: Option<FormControlRef>,
        scope: &FormDispatchScope<'_>,
        completion: Box<dyn FnOnce(Option<SubmissionRef>)>,
    ) {
        let attributes = environment.Attributes();
        let mut copied = attributes.clone();
        if let Some(overrides) = submitter
            .as_ref()
            .and_then(|s| environment.SubmitterOverrides(s))
        {
            let (action, enctype, method, target) = match overrides {
                SubmitterOverrides::Native {
                    action,
                    enctype,
                    method,
                    target,
                } => (action, enctype, method, target),
                SubmitterOverrides::Behavior {
                    action,
                    enctype,
                    method,
                    target,
                } => (
                    if action.is_empty() {
                        None
                    } else {
                        Some(action)
                    },
                    if enctype.is_empty() {
                        None
                    } else {
                        Some(enctype)
                    },
                    if method.is_empty() {
                        None
                    } else {
                        Some(method)
                    },
                    if target.is_empty() {
                        None
                    } else {
                        Some(target)
                    },
                ),
            };
            for (name, value) in [
                ("action", action),
                ("enctype", enctype),
                ("method", method),
                ("target", target),
            ] {
                if let Some(value) = value {
                    copied.ParseAttribute(name, Some(&value));
                }
            }
        }
        if copied.method == SubmitMethod::Dialog {
            let result = environment.DialogResult(submitter.as_ref());
            completion(Some(Rc::new(Self {
                environment,
                method: SubmitMethod::Dialog,
                action: String::new(),
                target: String::new(),
                result,
                resource: None,
                prepared_request: None,
                target_frame: RefCell::new(None),
                load_type: Cell::new(FrameLoadType::Standard),
                form_data: None,
            })));
            return;
        }
        let action = if copied.action.is_empty() {
            environment.DocumentURL()
        } else {
            copied.action.clone()
        };
        // KURL retains an invalid action too: constructing the entry list and
        // its synchronous formdata event still occurs (source cc274-314).
        let action = environment.CompleteURL(&action).unwrap_or(action);
        let preparation = environment.PrepareActionURL(&action);
        if preparation.did_upgrade_insecure_request {
            environment.CountUse("UpgradeInsecureRequestsUpgradedRequestForm");
        }
        let raw_action = preparation.url;
        let mut action = url::Url::parse(&raw_action).ok();
        let mailto = action.as_ref().is_some_and(|url| url.scheme() == "mailto");
        let mut multipart =
            copied.method == SubmitMethod::Post && copied.encoding_type == "multipart/form-data";
        let mut enctype = copied.encoding_type.clone();
        if multipart && mailto {
            enctype = "application/x-www-form-urlencoded".into();
            multipart = false;
        }
        let encoding = if mailto {
            "UTF-8"
        } else {
            FormData::EncodingFromAcceptCharset(
                &copied.accept_charset,
                &environment.DocumentEncoding(),
            )
        };
        let Some(data) = environment.ConstructEntryList(submitter.clone(), encoding, scope) else {
            completion(None);
            return;
        };
        let mut boundary = String::new();
        use interaction_dispatch::core::fileapi::encoded_form_data::EncodedFormData;
        let mut encoded_data = if multipart {
            boundary = interaction_dispatch::core::fileapi::form_data::GenerateUniqueBoundaryString(
                environment.RandomBoundaryBytes(),
            );
            data.Read(|data| data.EncodeMultiPartFormData(&boundary))
        } else {
            EncodedFormData::FromBytes(data.Read(|data| {
                data.EncodeFormData(
                    environment.Attributes().method != SubmitMethod::Get && enctype == "text/plain",
                )
            }))
        };
        if copied.method == SubmitMethod::Post && mailto {
            // AppendMailtoPostFormDataToURL.cc:91-119: UTF-8/isomorphic decode.
            let mut body = String::from_utf8(
                encoded_data
                    .FlattenDataOnly()
                    .expect("mailto source form data contains no multipart files")
                    .to_vec(),
            )
            .expect("mailto FormData is UTF-8");
            if enctype.eq_ignore_ascii_case("text/plain") {
                body = body.replace('&', "\r\n").replace('+', " ") + "\r\n";
                body = DecodeURLEscapeUTF8OrIsomorphic(&body);
            }
            let mut encoded = b"body=".to_vec();
            EncodeStringAsFormData(&mut encoded, body.as_bytes(), true);
            let body = String::from_utf8(encoded)
                .expect("percent encoded ASCII")
                .replace('+', "%20");
            let action = action.as_mut().expect("mailto has a parsed source URL");
            let query = action
                .query()
                .filter(|q| !q.is_empty())
                .map_or_else(|| body.clone(), |query| format!("{query}&{body}"));
            action.set_query(Some(&query));
            encoded_data = EncodedFormData::FromBytes(Vec::new());
        }
        if let Some(action) = action
            .as_mut()
            .filter(|action| copied.method != SubmitMethod::Post && action.scheme() != "javascript")
        {
            action.set_query(Some(
                std::str::from_utf8(
                    &encoded_data
                        .FlattenDataOnly()
                        .expect("GET source form data contains no multipart files"),
                )
                .expect("URL encoded ASCII"),
            ));
        }
        let encoded_data = encoded_data.WithSubmissionMetadata(
            FormData::GenerateIdentifier(),
            data.Read(|data| data.ContainsPasswordData()),
        );
        let action = action.map(String::from).unwrap_or(raw_action);
        let resource = ResourceRequest::new(&action);
        resource.Mutate(|request| {
            request.is_form_submission = true;
            if copied.method == SubmitMethod::Post {
                request.method = "POST".into();
                request.body = Some(encoded_data.clone());
                request.headers.push((
                    "Content-Type".into(),
                    if boundary.is_empty() {
                        enctype
                    } else {
                        format!("{enctype}; boundary={boundary}")
                    },
                ));
            }
        });
        let triggering = event
            .as_ref()
            .map_or(TriggeringEventInfo::NotFromEvent, |event| {
                if event.borrow().BaseEvent().isTrusted() {
                    TriggeringEventInfo::Trusted
                } else {
                    TriggeringEventInfo::Untrusted
                }
            });
        let underlying = event
            .as_ref()
            .and_then(|event| event.borrow().BaseEvent().UnderlyingEvent())
            .or(event);
        let initiator = environment.CaptureInitiator(scope);
        let request = FrameLoadRequest::new(&resource, environment.FrameEnvironment(), initiator);
        request
            .navigation_policy
            .set(navigation_policy::NavigationPolicyFromEvent(
                underlying
                    .as_ref()
                    .and_then(|e| environment.EventModifiers(e)),
                environment.CurrentInputModifiers(),
                environment.IsMac(),
                environment.NewTabModifierSetFromIsolatedWorld(),
            ));
        request
            .client_navigation_reason
            .set(if copied.method == SubmitMethod::Post {
                ClientNavigationReason::FormSubmissionPost
            } else {
                ClientNavigationReason::FormSubmissionGet
            });
        *request.form.borrow_mut() = Some(environment.FormTarget());
        *request.source_element.borrow_mut() = Some(
            submitter
                .as_ref()
                .filter(|submitter| submitter.IsNativeFormControl())
                .map_or_else(|| environment.FormTarget(), |s| s.Target()),
        );
        request.triggering_event_info.set(triggering);
        request
            .triggering_event_trusted
            .set(triggering == TriggeringEventInfo::Trusted);
        let base_target = environment.BaseTarget();
        let target = request.CleanNavigationTarget(if copied.target.is_empty() {
            &base_target
        } else {
            &copied.target
        });
        // HasRel is read after the synchronous formdata event in source.
        request.HandleRelAttribute(&target, environment.Attributes().relations);
        let owner = Rc::new(Self {
            environment: environment.clone(),
            method: copied.method,
            action,
            target: target.clone(),
            result: String::new(),
            resource: Some(resource),
            prepared_request: Some(request.clone()),
            target_frame: RefCell::new(None),
            load_type: Cell::new(FrameLoadType::Standard),
            form_data: Some(encoded_data),
        });
        environment.ResolveTarget(
            request.clone(),
            target,
            Box::new(move |target| {
                if let Some(target) = target {
                    // Freeze history replacement before the later posted task.
                    if target.IsLocal()
                        && target.NavigationShouldReplaceCurrentHistoryEntry(
                            &request,
                            FrameLoadType::Standard,
                        )
                    {
                        owner.load_type.set(FrameLoadType::ReplaceCurrentItem);
                    }
                    *owner.target_frame.borrow_mut() = Some(target);
                }
                completion(Some(owner));
            }),
        );
    }
    pub fn Request(&self) -> Option<Rc<FrameLoadRequest>> {
        self.prepared_request.clone()
    }
    pub fn TargetName(&self) -> &str {
        &self.target
    }
    pub fn LoadType(&self) -> FrameLoadType {
        self.load_type.get()
    }
    pub fn NotifyInspector(&self) {
        if let Some(target) = self.target_frame.borrow().clone() {
            if let Some(request) = &self.prepared_request {
                target.NotifyFormNavigation(request);
            }
        }
    }
    pub fn Navigate(&self) {
        let Some(resource) = &self.resource else {
            return;
        };
        let original = self
            .prepared_request
            .as_ref()
            .expect("original form FrameLoadRequest");
        let request = FrameLoadRequest::new(
            resource,
            self.environment.FrameEnvironment(),
            original.initiator.clone(),
        );
        request
            .navigation_policy
            .set(original.navigation_policy.get());
        request
            .client_navigation_reason
            .set(original.client_navigation_reason.get());
        *request.source_element.borrow_mut() = original.source_element.borrow().clone();
        *request.form.borrow_mut() = original.form.borrow().clone();
        request
            .triggering_event_info
            .set(original.triggering_event_info.get());
        request
            .triggering_event_trusted
            .set(original.triggering_event_trusted.get());
        request
            .input_start_time
            .set(original.input_start_time.get());
        request.explicit_opener.set(original.explicit_opener.get());
        let target = self.target_frame.borrow().clone();
        if let Some(target) = target.filter(|t| t.HasPage()) {
            target.Navigate(request, self.load_type.get());
        }
    }
}
impl FormSubmission for FormSubmissionOwner {
    fn Method(&self) -> SubmitMethod {
        self.method
    }
    fn ActionIsEmpty(&self) -> bool {
        self.action.is_empty()
    }
    fn ActionIsJavaScript(&self) -> bool {
        self.action
            .get(..11)
            .is_some_and(|s| s.eq_ignore_ascii_case("javascript:"))
    }
    fn HasData(&self) -> bool {
        self.form_data.is_some()
    }
    fn TargetFrame(&self) -> Option<SubmissionTargetRef> {
        self.target_frame.borrow().clone()
    }
    fn Result(&self) -> String {
        self.result.clone()
    }
}
fn DecodeURLEscapeUTF8OrIsomorphic(value: &str) -> String {
    let mut bytes = Vec::new();
    let input = value.as_bytes();
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'%' && index + 2 < input.len() {
            if let (Some(a), Some(b)) = (
                (input[index + 1] as char).to_digit(16),
                (input[index + 2] as char).to_digit(16),
            ) {
                bytes.push(((a << 4) | b) as u8);
                index += 3;
                continue;
            }
        }
        bytes.push(input[index]);
        index += 1;
    }
    String::from_utf8(bytes.clone()).unwrap_or_else(|_| bytes.into_iter().map(char::from).collect())
}
