//! DOM subclass values used by HitTestResult. Chromium hit_test_result.cc:396-435;
//! HTMLElement::GetDirectionalAttribute cc:3903-3928, SVGElement::title:156-176.
use super::*;
#[derive(Clone, Copy)]
pub enum DOMHitTestElementKind {
    Html,
    Image,
    InputImage,
    Input,
    TextArea,
    Embed,
    Object,
    Area,
    Map,
    Media,
    Other,
}
impl PersistentDocument {
    pub fn HitTestElementKind(&self, node: usize) -> Option<DOMHitTestElementKind> {
        use DOMHitTestElementKind::*;
        let element = self.Node(node);
        if element.Type() != DOMNodeType::kElement {
            return None;
        }
        if element.Namespace() != DOMNamespace::kHTML {
            return Some(Other);
        }
        Some(match element.Name() {
            "img" => Image,
            "input" if self.InputType(node) == Some(DOMInputType::kImage) => InputImage,
            "input" => Input,
            "textarea" => TextArea,
            "embed" => Embed,
            "object" => Object,
            "area" => Area,
            "map" => Map,
            "audio" | "video" => Media,
            _ => Html,
        })
    }
    pub fn HitTestDirectionalTitle(
        &self,
        node: usize,
        layout_direction: Option<TextDirection>,
    ) -> Option<(String, TextDirection)> {
        let title = self.Node(node).FindAttribute("title")?.value.clone();
        let mut direction = layout_direction.unwrap_or_else(|| self.CachedDirectionality(node));
        if self.HasDirectionAuto(node) {
            if let Some(value) =
                BidiParagraph::BaseDirectionForString(&StringView::from(title.as_str()), None)
            {
                direction = value;
            }
        }
        Some((title, direction))
    }
    pub fn NodeTextContentForHitTest(&self, node: usize) -> String {
        fn append(d: &PersistentDocument, n: usize, out: &mut String) {
            let node = d.Node(n);
            // The persistent arena normalizes XML CDATA to the same text-node
            // representation consumed by layout and hit-test textContent.
            if node.Type() == DOMNodeType::kText {
                out.push_str(node.Data());
                return;
            }
            for child in node.Children() {
                append(d, *child, out);
            }
        }
        let mut text = String::new();
        append(self, node, &mut text);
        text
    }
    pub fn ElementTitleForHitTest(&self, node: usize) -> Option<String> {
        let element = self.Node(node);
        if element.Namespace() == DOMNamespace::kHTML {
            return element.FindAttribute("title").map(|a| a.value.clone());
        }
        if element.Namespace() != DOMNamespace::kSVG {
            return None;
        }
        if element.Name() == "svg" && self.DocumentElement() == Some(node) {
            return None;
        }
        if self.CorrespondingSVGElement(node).is_some() {
            if let Some(host) = self
                .ContainingShadowRoot(node)
                .and_then(|root| self.ShadowHost(root))
            {
                if let Some(title) = self
                    .ElementTitleForHitTest(host)
                    .filter(|title| !title.is_empty())
                {
                    return Some(title);
                }
            }
        }
        element
            .Children()
            .iter()
            .copied()
            .find(|n| self.Node(*n).IsElement("title", DOMNamespace::kSVG))
            .map(|title| self.NodeTextContentForHitTest(title))
    }
    pub fn ImageSourceURLForHitTest(&self, node: usize) -> String {
        let element = self.Node(node);
        if element.IsHTMLElement("img") {
            return self.CurrentImageSource(node).unwrap_or_default().to_owned();
        }
        let name = if element.IsHTMLElement("object") {
            "data"
        } else if element.IsElement("image", DOMNamespace::kSVG) {
            "href"
        } else {
            "src"
        };
        element
            .FindAttribute(name)
            .or_else(|| {
                (name == "href")
                    .then(|| {
                        element.Attributes().iter().find(|a| {
                            a.local_name == "href"
                                && a.namespace_uri == "http://www.w3.org/1999/xlink"
                        })
                    })
                    .flatten()
            })
            .map_or_else(String::new, |a| a.value.clone())
    }
    pub fn HrefAttributeForHitTest(&self, node: usize) -> Option<&str> {
        let element = self.Node(node);
        if !self.IsLinkForEditing(node) {
            return None;
        }
        element
            .FindAttribute("href")
            .or_else(|| {
                element.Attributes().iter().find(|a| {
                    a.local_name == "href" && a.namespace_uri == "http://www.w3.org/1999/xlink"
                })
            })
            .map(|a| a.value.as_str())
    }
    pub fn IsDisabledOrReadOnlyForHitTest(&self, node: usize) -> bool {
        self.IsDisabledFormControl(node)
            || (self.Node(node).FindAttribute("readonly").is_some()
                && (self.Node(node).IsHTMLElement("textarea")
                    || self
                        .InputType(node)
                        .is_some_and(DOMInputType::SupportsReadOnly)))
    }
}
