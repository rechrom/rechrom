//! Document-owned CSS telemetry shared by parser contexts and CSSOM wrappers.
//! Source: UseCounter::Count / CountCSSProperty and CSSParserContext.cc220-242.
use crate::CSSPropertyID;
use std::{cell::RefCell, collections::HashSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CSSParserFeature {
    Rule,
    Property,
    AtRule(i32),
    ContainerRangeSyntax,
    ContainerStyleQuery,
}
#[derive(Default)]
pub struct CSSUseCounter {
    properties: RefCell<HashSet<CSSPropertyID>>,
    features: RefCell<HashSet<CSSParserFeature>>,
    webdx: RefCell<HashSet<CSSParserFeature>>,
    deprecations: RefCell<HashSet<CSSParserFeature>>,
}
#[allow(non_snake_case)]
impl CSSUseCounter {
    pub fn CountProperty(&self, property: CSSPropertyID) {
        self.properties.borrow_mut().insert(property);
    }
    pub fn IsPropertyCounted(&self, property: CSSPropertyID) -> bool {
        self.properties.borrow().contains(&property)
    }
    pub fn Count(&self, feature: CSSParserFeature) {
        self.features.borrow_mut().insert(feature);
    }
    pub fn IsCounted(&self, feature: CSSParserFeature) -> bool {
        self.features.borrow().contains(&feature)
    }
    pub fn CountWebDX(&self, feature: CSSParserFeature) {
        self.webdx.borrow_mut().insert(feature);
    }
    pub fn CountDeprecation(&self, feature: CSSParserFeature) {
        self.deprecations.borrow_mut().insert(feature);
        self.Count(feature);
    }
}
