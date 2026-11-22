//! Settings and Document font-size inputs used by native editing style.
//! Chromium settings.json5:118-125,559-565; document.h:2062-2064;
//! document.cc:8367-8415 / html_meta_element.cc TextScaleMetaChanged callers.
use super::*;
pub(super) struct EditingFontSettings {
    minimum_logical_font_size: i32,
    accessibility_scale_factor: f64,
    text_scale_meta_present: bool,
}
impl EditingFontSettings {
    pub fn new() -> Self {
        Self {
            minimum_logical_font_size: 0,
            accessibility_scale_factor: 1.0,
            text_scale_meta_present: false,
        }
    }
}
impl PersistentDocument {
    pub fn MinimumLogicalFontSize(&self) -> i32 {
        self.editing_font_settings.minimum_logical_font_size
    }
    pub fn SetMinimumLogicalFontSize(&mut self, value: i32) {
        if self.editing_font_settings.minimum_logical_font_size == value {
            return;
        }
        self.editing_font_settings.minimum_logical_font_size = value;
        // Settings invalidate:Style; consume the existing source all-style input.
        self.InvalidateAllStyles();
    }
    pub fn AccessibilityFontScaleFactor(&self) -> f64 {
        self.editing_font_settings.accessibility_scale_factor
    }
    pub fn SetAccessibilityFontScaleFactor(&mut self, value: f64) {
        if self.editing_font_settings.accessibility_scale_factor == value {
            return;
        }
        self.editing_font_settings.accessibility_scale_factor = value;
        self.InvalidateAllStyles();
    }
    pub fn TextScaleMetaTagPresent(&self) -> bool {
        foundation::RuntimeEnabledFeatures::TextScaleMetaTagEnabled()
            && self.editing_font_settings.text_scale_meta_present
    }
    fn SetTextScaleMetaTagPresent(&mut self, present: bool) {
        if self.editing_font_settings.text_scale_meta_present == present {
            return;
        }
        self.editing_font_settings.text_scale_meta_present = present;
        if present {
            self.CountWebDX(crate::use_counter::WebDXFeature::kMetaTextScale);
        }
        // Document::SetTextScaleMetaTagPresent invokes InitialStyleChanged.
        self.InvalidateAllStyles();
    }
    pub(super) fn TextScaleMetaChanged(&mut self) {
        let mut current = self.DocumentElement();
        let mut present = false;
        while let Some(node) = current {
            if self.arena[node].IsHTMLElement("meta")
                && self.arena[node]
                    .AttributeValueUTF16("name")
                    .is_some_and(|v| AsciiEqual(&v, b"text-scale"))
            {
                present = self.arena[node]
                    .AttributeValueUTF16("content")
                    .is_some_and(|v| AsciiEqual(&v, b"scale"));
                break;
            }
            // Source Traversal<HTMLMetaElement>::DescendantsOf(documentElement),
            // performed at the mutation, never by the serialization query.
            if let Some(&child) = self.arena[node].children.first() {
                current = Some(child);
                continue;
            }
            let mut ancestor = node;
            loop {
                if let Some(next) = self.arena[ancestor].next_sibling {
                    current = Some(next);
                    break;
                }
                let Some(parent) = self.arena[ancestor].parent else {
                    current = None;
                    break;
                };
                if self.arena[parent].node_type == DOMNodeType::kDocument {
                    current = None;
                    break;
                }
                ancestor = parent;
            }
        }
        self.SetTextScaleMetaTagPresent(present);
    }
    pub(super) fn IsTextScaleMetaElement(&self, node: usize) -> bool {
        self.arena[node].IsHTMLElement("meta")
            && self.arena[node]
                .AttributeValueUTF16("name")
                .is_some_and(|v| AsciiEqual(&v, b"text-scale"))
    }
    pub(super) fn EditingMetaAttributeChanged(
        &mut self,
        node: usize,
        change: &crate::attribute_lifecycle::AttributeChange,
    ) {
        if !change.namespace.is_empty()
            || !self.arena[node].IsHTMLElement("meta")
            || !self.IsConnectedToDocument(node)
        {
            return;
        }
        let changed_text_scale_name = change.name == "name"
            && change
                .old_value
                .as_ref()
                .is_some_and(|v| v.eq_ignore_ascii_case("text-scale"));
        if matches!(change.name.as_str(), "name" | "content")
            && (self.IsTextScaleMetaElement(node) || changed_text_scale_name)
        {
            self.TextScaleMetaChanged();
        }
    }
}

fn AsciiEqual(value: &[u16], expected: &[u8]) -> bool {
    value.len() == expected.len()
        && value.iter().zip(expected).all(|(&unit, &ascii)| {
            let folded = if (65..=90).contains(&unit) {
                unit + 32
            } else {
                unit
            };
            folded == ascii as u16
        })
}
