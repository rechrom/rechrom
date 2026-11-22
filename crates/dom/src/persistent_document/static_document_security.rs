//! DocumentLoader.cc:2790-2807,2851-2906,3144-3192. A static document has a
//! new DOM lifetime even when it inherits the old Window's security inputs and
//! the original DocumentToken. No node/tree or layout state is copied.
use super::PersistentDocument;
use std::sync::Arc;
pub struct JavaScriptURLDocumentSecurity {
    security: super::security_context::JavaScriptURLSecuritySnapshot,
    csp: Arc<crate::content_security_policy::DocumentContentSecurityPolicy>,
    cross_origin_isolated_capability: bool,
}
impl PersistentDocument {
    /// LocalDOMWindow::ClearForReuse keeps the original ExecutionContext;
    /// the new Document aliases that lifetime rather than reviving an old
    /// destroyed context or copying its lifecycle/observer state.
    pub fn ReuseWindowExecutionContext(&mut self, previous: &PersistentDocument) {
        assert_ne!(self.RootHandle(), previous.RootHandle());
        assert!(
            !previous.IsContextDestroyed(),
            "source reuses a live initial Window"
        );
        assert!(
            self.GetDocumentParser().is_none(),
            "Window selection precedes parser creation"
        );
        self.native_execution_context_lifetime = previous.native_execution_context_lifetime.clone();
        self.security_context
            .ReuseWindowPublicURLManager(&previous.security_context);
        self.ReuseWindowCustomElementDefinitions(previous);
    }
    pub fn CaptureJavaScriptURLDocumentSecurity(&self) -> JavaScriptURLDocumentSecurity {
        JavaScriptURLDocumentSecurity {
            security: self.security_context.CaptureForJavaScriptURLCommit(),
            csp: self.content_security_policy.clone(),
            cross_origin_isolated_capability: self
                .execution_context_isolation
                .CrossOriginIsolatedCapability(),
        }
    }
}
impl JavaScriptURLDocumentSecurity {
    pub fn InstallOnNewDocument(
        &self,
        document: &mut PersistentDocument,
        token: [u64; 2],
        url: String,
    ) {
        assert!(
            !document.navigation_security_initialized,
            "install security once on the new Document"
        );
        assert!(
            document.GetDocumentParser().is_none(),
            "InstallNewDocument precedes parser creation"
        );
        document.security_context = self.security.CreateContext();
        document.content_security_policy = self.csp.CopyPoliciesForNavigation();
        document
            .content_security_policy
            .BindSecurityOrigin(document.security_context.Origin());
        document.navigation_security_initialized = true;
        document.navigation_document_token =
            crate::navigation_tokens::DocumentTokenOwner::WithToken(token);
        document.execution_context_isolation =
            super::security_context::ExecutionContextIsolation::ForInheritedAgentCluster(
                self.cross_origin_isolated_capability,
            );
        document.SetDocumentURL(url);
        document.SetIsInitialEmptyDocument(false);
    }
}
