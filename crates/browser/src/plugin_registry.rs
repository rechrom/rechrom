//! Browser product plugin admission and per-Page instance lookup.
//!
//! Rechrom has no plugin loader, process or WebPluginContainer implementation.
//! This is an explicit product capability, rather than an injected ContextMenu
//! callback or an empty plugin implementation. A future supported product must
//! add its real instance owner and lookup result here before admitting plugins.
use dom::persistent_document::DOMOwnerHandle;

enum BrowserPluginCapability {
    Unsupported,
}
pub(crate) struct PagePluginRegistry {
    capability: BrowserPluginCapability,
}
impl Default for PagePluginRegistry {
    fn default() -> Self {
        Self::ForBrowserProduct()
    }
}
pub(crate) enum PluginInstanceLookup {
    Unsupported,
}
impl PagePluginRegistry {
    pub(crate) fn ForBrowserProduct() -> Self {
        Self {
            capability: BrowserPluginCapability::Unsupported,
        }
    }
    /// Source LayoutEmbeddedContent::Plugin() is nullable. The unsupported
    /// product cannot admit any instance for this canonical element identity.
    pub(crate) fn LookupPlugin(&self, _element: DOMOwnerHandle) -> PluginInstanceLookup {
        match self.capability {
            BrowserPluginCapability::Unsupported => PluginInstanceLookup::Unsupported,
        }
    }
}
