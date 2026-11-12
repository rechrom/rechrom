//! HTMLLinkElement/LinkLoader's font-preload producer journal. The existing
//! insertion/attribute/removal walk records each original source transition;
//! ResourceFetcher consumes it at the synchronous DOM boundary or parser token.
use crate::persistent_document::PersistentDocument;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FontPreloadReason {
    Attribute,
    MediaChange,
    Removed,
}
pub struct FontPreloadMutation {
    pub node: u64,
    pub reason: FontPreloadReason,
    pub active: bool,
    pub rel: String,
    pub as_: String,
    pub href: Option<String>,
    pub media: String,
    pub type_: String,
    pub cross_origin: Option<String>,
    pub referrer_policy: String,
    pub integrity: String,
    pub has_body: bool,
}
impl PersistentDocument {
    pub(super) fn FontPreloadAttributeChanged(&self, node: usize, name: &str, namespace: &str) {
        if namespace.is_empty()
            && matches!(
                name,
                "rel"
                    | "as"
                    | "href"
                    | "type"
                    | "media"
                    | "crossorigin"
                    | "referrerpolicy"
                    | "integrity"
                    | "fetchpriority"
            )
            && self.IsConnectedToDocument(node)
        {
            self.RecordFontPreload(
                node,
                if name == "media" {
                    FontPreloadReason::MediaChange
                } else {
                    FontPreloadReason::Attribute
                },
            );
        }
    }
    pub(super) fn RecordFontPreload(&self, node: usize, reason: FontPreloadReason) {
        let element = self.Node(node);
        if !element.IsHTMLElement("link") {
            return;
        }
        let attribute = |name| {
            element
                .FindAttribute(name)
                .map(|attribute| attribute.value.clone())
        };
        // HTMLLinkElement::ShouldLoadLink cc242-263. The preload branch only
        // loads active documents; disconnected/removal unconditionally aborts.
        let active = reason != FontPreloadReason::Removed
            && self.IsActive()
            && self.IsConnectedToDocument(node);
        let href = attribute("href")
            .filter(|value| !value.is_empty())
            .and_then(|value| self.BaseURL().CompleteURL(&value))
            .map(|url| url.to_string());
        let has_body = self
            .GetRenderBlockingResourceManager()
            .is_some_and(|owner| owner.borrow().DocumentHasBody());
        self.FontLoading().RecordPreload(FontPreloadMutation {
            node: element.Id(),
            reason,
            active,
            rel: attribute("rel").unwrap_or_default(),
            as_: attribute("as").unwrap_or_default(),
            href,
            media: attribute("media").unwrap_or_default(),
            type_: attribute("type").unwrap_or_default(),
            cross_origin: attribute("crossorigin"),
            referrer_policy: attribute("referrerpolicy").unwrap_or_default(),
            integrity: attribute("integrity").unwrap_or_default(),
            has_body,
        });
    }
}
