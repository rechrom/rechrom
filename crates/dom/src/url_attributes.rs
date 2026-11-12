// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Element::IsURLAttribute virtual dispatch, retained at the DOM owner.
//! Source element.h:1218 and the HTML/SVG/MathML element overrides below.
use super::*;
impl DOMNode {
    pub fn IsURLAttribute(&self, index: usize) -> bool {
        let Some(attribute) = self.attributes.get(index) else {
            return false;
        };
        self.IsURLAttributeValue(attribute, &self.attribute_utf16[index].value)
    }
    pub(crate) fn IsURLAttributeValue(&self, attribute: &DOMAttribute, value: &[u16]) -> bool {
        let local = attribute.local_name.as_str();
        let qualified = attribute.namespace_uri.is_empty() && attribute.prefix.is_empty();
        match self.namespace {
            DOMNamespace::kHTML => match self.name.as_str() {
                // html_{anchor,link,base}_element.cc:274,477,59: LocalName.
                "a" | "area" | "link" | "base" => local == "href",
                // html_frame_element_base.cc:239-243.
                "frame" | "iframe" => qualified && matches!(local, "longdesc" | "src"),
                // html_object_element.cc:266-270; html_embed_element.cc:211.
                "object" => qualified && matches!(local, "codebase" | "data"),
                "embed" | "source" | "track" | "script" | "audio" => qualified && local == "src",
                // html_video_element.cc:387 + html_media_element.cc:4661.
                "video" => qualified && matches!(local, "poster" | "src"),
                // html_image_element.cc:787-794, usemap excludes a local fragment.
                "img" => {
                    qualified
                        && (matches!(local, "src" | "lowsrc" | "longdesc")
                            || local == "usemap" && value.first() != Some(&35))
                }
                "body" | "table" | "td" | "th" => qualified && local == "background",
                "button" => qualified && local == "formaction",
                "input" => qualified && matches!(local, "src" | "formaction"),
                "form" => qualified && local == "action",
                "blockquote" | "q" | "del" | "ins" => qualified && local == "cite",
                "html" => qualified && local == "manifest",
                "credential" => qualified && local == "configurl",
                "install" => qualified && matches!(local, "manifestid" | "installurl" | "manifest"),
                _ => false,
            },
            DOMNamespace::kSVG => match self.name.as_str() {
                "a" => local == "href", // svg_a_element.cc:264-267.
                "script" => {
                    // svg_script_element.cc:111-113,150-152 compares LocalName
                    // to LegacyHrefString's VALUE (not a generic href test).
                    let href = self
                        .attributes
                        .iter()
                        .find(|a| a.namespace_uri.is_empty() && a.local_name == "href")
                        .or_else(|| {
                            self.attributes.iter().find(|a| {
                                a.namespace_uri == "http://www.w3.org/1999/xlink"
                                    && a.local_name == "href"
                            })
                        });
                    href.is_some_and(|href| local == href.value)
                }
                _ => false,
            },
            DOMNamespace::kMathML => self.name == "a" && qualified && local == "href",
            DOMNamespace::kNone => false,
        }
    }
}
