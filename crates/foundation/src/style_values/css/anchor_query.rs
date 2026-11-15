#![allow(non_snake_case)]

use crate::{Member, Traceable, ValuesEquivalent, Visitor};

use super::css_anchor_query_enums::{CSSAnchorQueryType, CSSAnchorSizeValue, CSSAnchorValue};
use crate::style_values::style::anchor_specifier_value::AnchorSpecifierValue;

// cpp: foundation/style_values/css/anchor_query.h:35-35
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnchorQueryValue {
    Anchor(CSSAnchorValue),
    AnchorSize(CSSAnchorSizeValue),
}

// cpp: foundation/style_values/css/anchor_query.h:23-70
#[derive(Clone)]
pub struct AnchorQuery {
    query_type_: CSSAnchorQueryType,
    anchor_specifier_: Member<AnchorSpecifierValue>,
    percentage_: f32,
    value_: AnchorQueryValue,
}

impl AnchorQuery {
    // cpp: foundation/style_values/css/anchor_query.h:31-43
    pub fn new(
        query_type: CSSAnchorQueryType,
        anchor_specifier: *const AnchorSpecifierValue,
        percentage: f32,
        value: AnchorQueryValue,
    ) -> Self {
        assert!(!anchor_specifier.is_null());
        Self {
            query_type_: query_type,
            anchor_specifier_: Member::from_ptr(anchor_specifier as *mut AnchorSpecifierValue),
            percentage_: percentage,
            value_: value,
        }
    }

    // cpp: foundation/style_values/css/anchor_query.h:45-65
    pub fn Type(&self) -> CSSAnchorQueryType {
        self.query_type_
    }

    pub fn AnchorSpecifier(&self) -> &AnchorSpecifierValue {
        unsafe { &*self.anchor_specifier_.Get() }
    }

    pub fn AnchorSide(&self) -> CSSAnchorValue {
        debug_assert_eq!(self.query_type_, CSSAnchorQueryType::kAnchor);
        match self.value_ {
            AnchorQueryValue::Anchor(side) => side,
            AnchorQueryValue::AnchorSize(_) => panic!("anchor query contains anchor-size value"),
        }
    }

    pub fn AnchorSidePercentage(&self) -> f32 {
        debug_assert_eq!(self.query_type_, CSSAnchorQueryType::kAnchor);
        debug_assert_eq!(self.AnchorSide(), CSSAnchorValue::kPercentage);
        self.percentage_
    }

    pub fn AnchorSidePercentageOrZero(&self) -> f32 {
        debug_assert_eq!(self.query_type_, CSSAnchorQueryType::kAnchor);
        if self.AnchorSide() == CSSAnchorValue::kPercentage {
            self.percentage_
        } else {
            0.0
        }
    }

    pub fn AnchorSize(&self) -> CSSAnchorSizeValue {
        debug_assert_eq!(self.query_type_, CSSAnchorQueryType::kAnchorSize);
        match self.value_ {
            AnchorQueryValue::AnchorSize(size) => size,
            AnchorQueryValue::Anchor(_) => panic!("anchor-size query contains anchor side"),
        }
    }

    // cpp: foundation/style_values/css/anchor_query.h:68-68
    // cpp: foundation/style_values/css/anchor_query.cc:19-21
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.anchor_specifier_);
    }
}

// cpp: foundation/style_values/css/anchor_query.h:67-67
// cpp: foundation/style_values/css/anchor_query.cc:13-17
impl PartialEq for AnchorQuery {
    fn eq(&self, other: &Self) -> bool {
        self.query_type_ == other.query_type_
            && self.percentage_ == other.percentage_
            && ValuesEquivalent(&self.anchor_specifier_, &other.anchor_specifier_)
            && self.value_ == other.value_
    }
}

impl Traceable for AnchorQuery {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        AnchorQuery::Trace(self, visitor);
    }
}
