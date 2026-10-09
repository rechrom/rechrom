// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Neutral rule boundaries and validated property-registration data.
//! Runtime activation/registration ownership belongs to Document, not CSSOM.
#![allow(non_camel_case_types)]
use crate::{
    CSSContainerListSeparator, CSSContainerNumber, CSSContainerUnit, CSSContainerVariableData,
};
use foundation::{AtomicString, CSSValueID};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSSScopeRoot {
    ImplicitStylesheetOwner,
    ExplicitSelectorList(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSStyleScope {
    pub root: CSSScopeRoot,
    pub limit: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSScopeRule {
    pub scope: CSSStyleScope,
    pub prelude_text: String,
    pub source_order: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSPropertySyntaxType {
    kTokenStream,
    kIdent,
    kLength,
    kNumber,
    kPercentage,
    kLengthPercentage,
    kColor,
    kImage,
    kUrl,
    kInteger,
    kAngle,
    kTime,
    kResolution,
    kTransformFunction,
    kTransformList,
    kCustomIdent,
    kString,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSPropertySyntaxRepeat {
    None,
    SpaceSeparated,
    CommaSeparated,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSPropertySyntaxComponent {
    pub kind: CSSPropertySyntaxType,
    pub ident: AtomicString,
    pub repeat: CSSPropertySyntaxRepeat,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSPropertySyntax {
    pub components: Vec<CSSPropertySyntaxComponent>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSSRegisteredPropertyValue {
    Numeric(CSSContainerNumber, CSSContainerUnit),
    Identifier(CSSValueID),
    CustomIdent(AtomicString),
    String(AtomicString),
    Color {
        space: foundation::ColorSpace,
        components: [CSSContainerNumber; 3],
        alpha: CSSContainerNumber,
        none: [bool; 4],
    },
    Unparsed(CSSContainerVariableData),
    List(Vec<Self>, CSSContainerListSeparator),
}
/// A validated stylesheet registration request. Consumers must apply cascade
/// origin/layer/order and media activation before changing a Document registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSPropertyRule {
    pub name: AtomicString,
    pub syntax: CSSPropertySyntax,
    pub syntax_text: String,
    pub inherits: bool,
    pub initial_value: Option<CSSRegisteredPropertyValue>,
    pub initial_value_text: Option<String>,
    /// Retained parser tokens; the Document consumes these without retokenizing text.
    pub initial_value_data: Option<CSSContainerVariableData>,
    pub declaration_text: String,
    pub media_conditions: Vec<String>,
    pub layer_name: String,
    pub source_order: u32,
}
