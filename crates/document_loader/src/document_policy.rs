//! DocumentLoader.cc:1632-1682,3170-3175; SecurityContextInit.cc:65-102.
//! Header parsing is retained once by the Loader; the Document's SecurityContext
//! receives the same enforce/report-only policy identities at commit.
use dom::document_policy::{
    self, DocumentPolicy, DocumentPolicyFeatureState, DocumentPolicyMessage, ParsedDocumentPolicy,
    PolicyMessageLevel,
};
use std::rc::Rc;
#[derive(Default)]
pub(super) struct DocumentPolicyLoader {
    required: Rc<DocumentPolicyFeatureState>,
    enforce: Option<Rc<DocumentPolicy>>,
    report_only: Option<Rc<DocumentPolicy>>,
    required_for_subtree: Option<Rc<DocumentPolicyFeatureState>>,
    blocked: bool,
    messages: Vec<DocumentPolicyMessage>,
}
impl DocumentPolicyLoader {
    pub fn BindRequiredPolicy(&mut self, required: Rc<DocumentPolicyFeatureState>) {
        assert!(
            self.enforce.is_none(),
            "FramePolicy precedes response policy parsing"
        );
        self.required = required;
    }
    pub fn Initialized(&self) -> bool {
        self.enforce.is_some()
    }
    pub fn WasBlocked(&self) -> bool {
        self.blocked
    }
    pub fn Initialize(
        &mut self,
        url: &str,
        header: &str,
        report_only: &str,
        required_header: &str,
    ) {
        assert!(
            self.enforce.is_none(),
            "one retained policy compilation per committed response"
        );
        let local = url.is_empty()
            || url::Url::parse(url)
                .is_ok_and(|url| matches!(url.scheme(), "about" | "data" | "blob" | "filesystem"));
        let parsed = if local {
            ParsedDocumentPolicy {
                feature_state: (*self.required).clone(),
                ..Default::default()
            }
        } else {
            let mut parsed = document_policy::ParseDocumentPolicy(
                header,
                "Document-Policy HTTP header: ",
                &mut self.messages,
            )
            .unwrap_or_default();
            self.blocked =
                !document_policy::IsPolicyCompatible(&self.required, &parsed.feature_state);
            if self.blocked {
                parsed = ParsedDocumentPolicy {
                    feature_state: (*self.required).clone(),
                    ..Default::default()
                };
                self.messages.push(DocumentPolicyMessage{level:PolicyMessageLevel::Error,message:format!("Refused to load '{url}' because it violates the following document policy required by its embedder: '{}'.", document_policy::SerializeDocumentPolicy(&self.required).unwrap_or_else(||"[Serialization Error]".into()))});
            } else {
                let required = document_policy::ParseDocumentPolicy(
                    required_header,
                    "Require-Document-Policy HTTP header: ",
                    &mut self.messages,
                )
                .unwrap_or_default();
                self.required_for_subtree = Some(Rc::new(document_policy::MergeFeatureState(
                    &required.feature_state,
                    &self.required,
                )));
            }
            parsed
        };
        self.enforce = Some(DocumentPolicy::CreateWithHeaderPolicy(&parsed));
        // SecurityContextInit intentionally discards report-only parse messages.
        let parsed = document_policy::ParseDocumentPolicy(report_only, "", &mut vec![]);
        self.report_only = parsed
            .filter(|parsed| !parsed.feature_state.is_empty())
            .map(|parsed| DocumentPolicy::CreateWithHeaderPolicy(&parsed));
    }
    pub fn Commit(&mut self, document: &mut dom::Document) {
        let policy = self
            .enforce
            .as_ref()
            .expect("CreateDocumentPolicy precedes commit")
            .clone();
        document.SecurityContext().SetDocumentPolicy(policy);
        document
            .SecurityContext()
            .SetReportOnlyDocumentPolicy(self.report_only.clone());
        if let Some(required) = &self.required_for_subtree {
            document.SetRequiredDocumentPolicy(required.clone());
        }
        document.QueueDocumentPolicyMessages(self.messages.drain(..));
    }
}
