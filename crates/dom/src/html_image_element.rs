//! HTMLImageElement's loader-driven layout disposition and fallback tree.
//! Source html_image_element.cc:1104-1168,1181-1235; html_image_loader.cc:96-127.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HTMLImageLayoutDisposition {
    Primary,
    Fallback,
    Collapsed,
}
impl DOMNode {
    pub fn HTMLImageUsesFallbackContent(&self) -> bool {
        self.html_image_layout_disposition.get() == HTMLImageLayoutDisposition::Fallback
    }
    pub fn HTMLImageIsCollapsed(&self) -> bool {
        self.html_image_layout_disposition.get() == HTMLImageLayoutDisposition::Collapsed
    }
}
impl PersistentDocument {
    /// HTMLImageElement.h:178-179. This belongs to the element, not a timing record.
    pub fn SetIsLCPElement(&self, node: usize) {
        assert!(self.arena[node].IsHTMLElement("img"));
        self.arena[node].html_image_is_lcp_element.set(true);
    }
    pub fn IsLCPElement(&self, node: usize) -> bool {
        self.arena[node].html_image_is_lcp_element.get()
    }
    pub fn HasLazyLoadingAttribute(&self, node: usize) -> bool {
        self.arena[node].attributes.iter().any(|attribute| {
            attribute.namespace_uri.is_empty()
                && attribute.local_name == "loading"
                && attribute.value.eq_ignore_ascii_case("lazy")
        })
    }

    pub fn HTMLImageUsesFallbackContent(&self, node: usize) -> bool {
        self.arena[node].HTMLImageUsesFallbackContent()
    }
    pub fn HTMLImageIsCollapsed(&self, node: usize) -> bool {
        self.arena[node].HTMLImageIsCollapsed()
    }
    pub fn HTMLImageAltText(&self, node: usize) -> foundation::String {
        let element = &self.arena[node];
        for name in ["alt", "title"] {
            if let Some(index) = element.attributes.iter().position(|attribute| {
                attribute.namespace_uri.is_empty() && attribute.local_name == name
            }) {
                return foundation::String::from_utf16(&element.attribute_utf16[index].value);
            }
        }
        foundation::String::default()
    }
    fn SetHTMLImageLayoutDisposition(
        &mut self,
        node: usize,
        next: HTMLImageLayoutDisposition,
    ) -> bool {
        if matches!(
            self.arena[node].class,
            DOMNodeClass::HTMLAltTextImageElement
        ) {
            return false;
        }
        if self.arena[node].html_image_layout_disposition.get() == next {
            return false;
        }
        if let Some(root) = self
            .ShadowRoot(node)
            .filter(|root| self.ShadowRootMode(*root) == Some(DOMShadowRootMode::kUserAgent))
        {
            while let Some(child) = self.arena[root].children.first().copied() {
                self.Remove(child);
            }
        }
        self.arena[node].html_image_layout_disposition.set(next);
        self.arena[node].has_custom_style_callbacks = next == HTMLImageLayoutDisposition::Fallback;
        if next == HTMLImageLayoutDisposition::Fallback {
            let alt = self.HTMLImageAltText(node);
            self.CreateHTMLImageAltTextShadowTree(node, &alt);
        }
        self.InvalidateNodeStyle(node);
        self.ForceElementLayoutReattach(node);
        true
    }
    pub fn ResetHTMLImageLayoutDisposition(&mut self, node: usize) -> bool {
        if !self.arena[node].IsHTMLElement("img") {
            return false;
        }
        let Some(loader) = self.ExistingHTMLImageLoader(node) else {
            return false;
        };
        self.SetHTMLImageLayoutDisposition(
            node,
            if loader.ImageIsPotentiallyAvailable() {
                HTMLImageLayoutDisposition::Primary
            } else {
                if loader
                    .GetContent()
                    .is_some_and(|content| content.ShouldCollapseInitiator())
                {
                    HTMLImageLayoutDisposition::Collapsed
                } else {
                    HTMLImageLayoutDisposition::Fallback
                }
            },
        )
    }
    pub fn HTMLImageLoaderCompleted(&mut self, node: usize, no_resource: bool) -> bool {
        if !self.arena[node].IsHTMLElement("img") {
            return false;
        }
        if no_resource {
            if self.HTMLImageAltText(node).empty() {
                return false;
            }
            self.SetHTMLImageLayoutDisposition(node, HTMLImageLayoutDisposition::Fallback)
        } else {
            self.ResetHTMLImageLayoutDisposition(node)
        }
    }
}

impl PersistentDocument {
    pub fn InstallBrokenImageResources(
        &self,
        variants: image_resource::platform_resources::BrokenImageResourceVariants,
    ) {
        let mut slot = self.broken_image_resources.borrow_mut();
        assert!(slot.is_none(), "same platform image content installed once");
        *slot = Some(variants);
    }
    pub fn HasBrokenImageResources(&self) -> bool {
        self.broken_image_resources.borrow().is_some()
    }
    pub fn HTMLImageBrokenResources(
        &self,
        node: usize,
    ) -> Option<image_resource::platform_resources::BrokenImageResourceVariants> {
        if !matches!(
            self.arena[node].class,
            DOMNodeClass::HTMLAltTextImageElement
        ) {
            return None;
        }
        self.broken_image_resources.borrow().clone()
    }
    pub fn NeedsBrokenImageResources(&self, node: usize) -> bool {
        self.HTMLImageUsesFallbackContent(node) || self.ImageInputHasFallbackContent(node)
    }
}
