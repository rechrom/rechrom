//! Borrowed source StyleResolver::StyleRulesForElement query and individual
//! computed-property reads. No RuleSet, selector service or resolver cache is
//! exposed at the package boundary. Canonical declaration handles remain shared.
//! Chromium style_resolver.cc:2445-2458; editing_style.cc:1568-1587,1606-1651.
use crate::css_computed_style_declaration::{
    CSSComputedStyleDeclaration, ComputedStyleValueError, ComputedStyleValueOptions,
};
use crate::resolver::cascade_origin::CascadeOrigin;
use foundation::CSSPropertyID;
use layoutng_style::style::computed_style::ComputedStyle;
use std::rc::Rc;
use style_model::css_property_value_set::{CSSPropertyValueSet, CSSPropertyValueSetRuleHandle};
use style_parser::production_css_value::{ProductionCSSValueDispatch, Value};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerializationRuleScope {
    Author,
    UserAgentAndUser,
    All,
}
impl SerializationRuleScope {
    pub(crate) fn Includes(self, origin: CascadeOrigin) -> bool {
        match self {
            Self::Author => origin == CascadeOrigin::kAuthor,
            Self::UserAgentAndUser => {
                matches!(origin, CascadeOrigin::kUserAgent | CascadeOrigin::kUser)
            }
            Self::All => matches!(
                origin,
                CascadeOrigin::kUserAgent | CascadeOrigin::kUser | CascadeOrigin::kAuthor
            ),
        }
    }
}
pub(crate) struct SerializationDeclaration {
    pub origin: CascadeOrigin,
    pub properties: Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>,
}
/// The immutable matched list pins original rule declarations, like Blink's
/// StyleRuleList. Computed values borrow the Element's original ComputedStyle.
pub struct SerializationStyleQuery<'a> {
    pub(crate) computed: Option<&'a ComputedStyle>,
    pub(crate) declarations: Vec<SerializationDeclaration>,
}
impl SerializationStyleQuery<'_> {
    pub fn ForEachMatchedDeclaration(
        &self,
        mut read: impl FnMut(&CSSPropertyValueSet<ProductionCSSValueDispatch>),
    ) {
        for declaration in &self.declarations {
            declaration.properties.WithPropertySet(|set| read(set));
        }
    }
    pub fn GetComputedProperty(
        &self,
        property: CSSPropertyID,
    ) -> Result<Option<Rc<Value>>, ComputedStyleValueError> {
        CSSComputedStyleDeclaration::new(self.computed, ComputedStyleValueOptions::default())
            .GetPropertyCSSValue(property)
    }
}

#[derive(Debug)]
pub enum SerializationValueError {
    Style(crate::document_style_engine::DocumentStyleError),
    Computed(ComputedStyleValueError),
    Cascade {
        property: CSSPropertyID,
        operations: Vec<std::string::String>,
    },
    MissingComputedStyle,
}
impl std::fmt::Display for SerializationValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SerializationValueError {}
impl From<crate::document_style_engine::DocumentStyleError> for SerializationValueError {
    fn from(error: crate::document_style_engine::DocumentStyleError) -> Self {
        Self::Style(error)
    }
}

/// CSSPrimitiveValue::IsPercentage (css_primitive_value.cc:194-199), with
/// CSSMathFunctionValue's category check rather than looking for a '%' token.
pub fn IsSerializationPercentage(value: &Value) -> bool {
    match value.Payload() {
        style_model::css_value::CSSValuePayload::kNumericLiteralClass(value) => {
            value.GetType() == style_model::css_primitive_value::UnitType::kPercentage
        }
        style_model::css_value::CSSValuePayload::kMathFunctionClass(value) => {
            value.Category()
                == style_parser::css_math_expression_node::CalculationResultCategory::Percent
        }
        _ => false,
    }
}

/// Typed HTML attribute compilation error used by the serializer boundary.
pub use style_parser::production_property_parser::PropertyParseError as SerializationAttributeError;
