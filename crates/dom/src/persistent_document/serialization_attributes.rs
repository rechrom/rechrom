// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Retained typed values of EditingStyle's HTML attribute equivalents.
//! Chromium editing_style.cc:337-416,1198-1227; html_font_element.cc:53-156;
//! styled_markup_serializer.cc:278-304. Parse once per original attribute
//! mutation, in the DOM owner; serialization borrows/clones canonical values.
use super::*;
use foundation::{CSSPropertyID, CSSValueID};
use style_parser::{
    css_parser_context::{CSSParserContext, SecureContextMode},
    production_css_value::{self, ProductionCSSValueDispatch, Value},
    production_property_parser::{
        ParsePropertyWithContext, ProductionParserPlatform, PropertyParseError,
        PropertyParseErrorKind,
    },
};
use style_parser::{
    css_parser_mode::CSSParserMode,
    css_property_value_set::{CSSPropertyValueSetRuleHandle, MutableCSSPropertyValueSet},
};

#[derive(Clone)]
pub(super) struct SerializationAttributeValues {
    secure: bool,
    implicit: Option<Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>>,
    background: Option<Rc<Value>>,
    // HtmlAttributeEquivalent parses dir independently as direction and
    // unicode-bidi. Keep these separate from serializer wrapping style.
    direction: Option<Rc<Value>>,
    unicode_bidi: Option<Rc<Value>>,
}

// HTMLFontElement::ParseFontSize preserves the source prefix-integer grammar,
// including relative +/- values and checked integer-overflow behavior.
fn font_size(units: &[u16]) -> Option<CSSValueID> {
    let mut i = 0;
    while units
        .get(i)
        .is_some_and(|c| matches!(*c, 9 | 10 | 12 | 13 | 32))
    {
        i += 1;
    }
    let mode = match units.get(i) {
        Some(43) => {
            i += 1;
            1
        }
        Some(45) => {
            i += 1;
            -1
        }
        _ => 0,
    };
    let start = i;
    while units.get(i).is_some_and(|c| (48..=57).contains(c)) {
        i += 1;
    }
    if start == i {
        return None;
    }
    let mut value = units[start..i]
        .iter()
        .try_fold(0i32, |n, c| {
            n.checked_mul(10)?.checked_add(i32::from(*c - 48))
        })
        .unwrap_or(0);
    value = match mode {
        1 => value.checked_add(3).unwrap_or(value),
        -1 => 3i32.checked_sub(value).unwrap_or(value),
        _ => value,
    };
    Some(match value.clamp(1, 7) {
        1 => CSSValueID::kXSmall,
        2 => CSSValueID::kSmall,
        3 => CSSValueID::kMedium,
        4 => CSSValueID::kLarge,
        5 => CSSValueID::kXLarge,
        6 => CSSValueID::kXxLarge,
        _ => CSSValueID::kXxxLarge,
    })
}

impl PersistentDocument {
    fn EnsureSerializationAttributeValues(&self, element: usize) -> Result<(), PropertyParseError> {
        let node = &self.arena[element];
        let secure = self.SecurityContext().IsSecureContext();
        if node
            .serialization_attribute_values
            .borrow()
            .as_ref()
            .is_some_and(|values| values.secure == secure)
        {
            return Ok(());
        }
        let context = CSSParserContext::<ProductionParserPlatform>::FromMode(
            CSSParserMode::kHTMLQuirksMode,
            if secure {
                SecureContextMode::kSecureContext
            } else {
                SecureContextMode::kInsecureContext
            },
            None,
        );
        let mut implicit = MutableCSSPropertyValueSet::new(CSSParserMode::kHTMLQuirksMode);
        let mut background = None;
        let mut direction = None;
        let mut unicode_bidi = None;
        for (attribute, utf16) in node.attributes.iter().zip(&node.attribute_utf16) {
            if !attribute.namespace_uri.is_empty() {
                continue;
            }
            let property = if node.IsHTMLElement("font") {
                match attribute.local_name.as_str() {
                    "color" => Some(CSSPropertyID::kColor),
                    "face" => Some(CSSPropertyID::kFontFamily),
                    "size" => {
                        if let Some(id) = font_size(&utf16.value) {
                            implicit.SetProperty(
                                CSSPropertyID::kFontSize,
                                production_css_value::identifier(id),
                                false,
                            );
                        }
                        continue;
                    }
                    _ => None,
                }
            } else {
                None
            };
            if let Some(property) = property {
                match ParsePropertyWithContext(
                    property,
                    &foundation::String::from_utf16(&utf16.value),
                    false,
                    &context,
                ) {
                    Ok(properties) => {
                        for property in properties {
                            implicit.SetLonghandProperty(property);
                        }
                    }
                    Err(error) if error.kind == PropertyParseErrorKind::Invalid => {}
                    Err(error) => return Err(error),
                }
            }
            if attribute.local_name == "dir" {
                for property in [CSSPropertyID::kDirection, CSSPropertyID::kUnicodeBidi] {
                    match ParsePropertyWithContext(
                        property,
                        &foundation::String::from_utf16(&utf16.value),
                        false,
                        &context,
                    ) {
                        Ok(properties) => {
                            let value = properties
                                .into_iter()
                                .find(|value| value.PropertyID() == property)
                                .map(|value| value.ValueRef());
                            match property {
                                CSSPropertyID::kDirection => direction = value,
                                CSSPropertyID::kUnicodeBidi => unicode_bidi = value,
                                _ => unreachable!(),
                            }
                        }
                        Err(error) if error.kind == PropertyParseErrorKind::Invalid => {}
                        Err(error) => return Err(error),
                    }
                }
            }
            if attribute.local_name == "background" {
                // Exact source ParseAndSetProperty input. The raw attribute is
                // intentionally not escaped: malformed quote/backslash syntax
                // has the same CSS parser invalid-value result as Blink.
                let mut units: Vec<u16> = "url('".encode_utf16().collect();
                units.extend_from_slice(&utf16.value);
                units.extend("')".encode_utf16());
                match ParsePropertyWithContext(
                    CSSPropertyID::kBackgroundImage,
                    &foundation::String::from_utf16(&units),
                    false,
                    &context,
                ) {
                    Ok(properties) => {
                        background = properties
                            .into_iter()
                            .find(|property| {
                                property.PropertyID() == CSSPropertyID::kBackgroundImage
                            })
                            .map(|property| property.ValueRef())
                    }
                    Err(error) if error.kind == PropertyParseErrorKind::Invalid => {}
                    Err(error) => return Err(error),
                }
            }
        }
        let implicit = (!implicit.Properties().is_empty()).then(|| {
            Rc::new(CSSPropertyValueSetRuleHandle::Mutable(Rc::new(
                RefCell::new(implicit),
            )))
        });
        *node.serialization_attribute_values.borrow_mut() = Some(SerializationAttributeValues {
            secure,
            implicit,
            background,
            direction,
            unicode_bidi,
        });
        Ok(())
    }
    pub fn ImplicitSerializationStyle(
        &self,
        element: usize,
    ) -> Result<
        Option<Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>>,
        PropertyParseError,
    > {
        self.EnsureSerializationAttributeValues(element)?;
        Ok(self.arena[element]
            .serialization_attribute_values
            .borrow()
            .as_ref()
            .unwrap()
            .implicit
            .clone())
    }
    pub fn BackgroundAttributeImage(
        &self,
        element: usize,
    ) -> Result<Option<Rc<Value>>, PropertyParseError> {
        self.EnsureSerializationAttributeValues(element)?;
        Ok(self.arena[element]
            .serialization_attribute_values
            .borrow()
            .as_ref()
            .unwrap()
            .background
            .clone())
    }
    /// editing_style.cc:337-416. Reuses the original attribute mutation's
    /// compiled CSSValue (including HTMLFontElement's legacy size parser).
    pub fn EditingStyleAttributeCSSValue(
        &self,
        element: usize,
        property: CSSPropertyID,
    ) -> Result<Option<Rc<Value>>, PropertyParseError> {
        self.EnsureSerializationAttributeValues(element)?;
        let values = self.arena[element].serialization_attribute_values.borrow();
        let values = values.as_ref().unwrap();
        Ok(match property {
            CSSPropertyID::kDirection => values.direction.clone(),
            CSSPropertyID::kUnicodeBidi => values.unicode_bidi.clone(),
            CSSPropertyID::kFontSize | CSSPropertyID::kFontFamily | CSSPropertyID::kColor => {
                values.implicit.as_ref().and_then(|style| {
                    style.WithPropertySet(|style| style.GetPropertyValueRef(property))
                })
            }
            _ => unreachable!("HtmlAttributeEquivalent's five source properties"),
        })
    }
    /// Element.cc:4138-4150, ElementData.cc:98-112 and Attribute::Matches.
    /// The canonical UTF-16 attribute values are compared by qualified local
    /// name and namespace; prefixes and iteration order do not affect equality.
    pub fn HasEquivalentAttributesForEditing(&self, first: usize, second: usize) -> bool {
        if first == second {
            return true;
        }
        let first = self.Node(first).AttributesUTF16();
        let second = self.Node(second).AttributesUTF16();
        first.len() == second.len()
            && first.iter().all(|attribute| {
                second.iter().any(|other| {
                    attribute.name.local_name == other.name.local_name
                        && attribute.name.namespace_uri == other.name.namespace_uri
                        && attribute.value == other.value
                })
            })
    }
}
