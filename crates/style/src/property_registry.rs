// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Document-owned registration state. CSSOM stores immutable rule requests only.
//! cpp: property_registry.h:16-114/.cc:9-140; property_registration.cc:31-77,165-285;
//! style_engine.cc:3391-3399,3499-3511,3621-3643.
#![allow(non_snake_case)]
use crate::{
    css_primitive_value::{UnitType, UnitTypeToString},
    css_syntax_component::{CSSSyntaxComponent, CSSSyntaxRepeat as R, CSSSyntaxType as T},
    css_syntax_definition::{CSSSyntaxDefinition, SyntaxValueError},
    css_value::CSSValuePayload,
    media_queries::{
        media_query_backend::{MediaQueryValueBackend, ParseMediaQuerySet},
        media_query_evaluator::MediaQueryEvaluator,
        media_values::MediaValues,
        production_container_query::{
            DocumentMediaValues, RegisteredContainerPropertyResolver,
            RegisteredContainerPropertyResult,
        },
    },
    parser::css_parser_token::{
        BlockType, CSSParserToken, CSSParserTokenType::*, NumericSign, NumericValueType,
    },
    production_container_projection::RestoreContainerVariableData,
    production_css_value::{self as values, CSSVariableData, Value, VariableToken},
    resolver::{
        cascade_origin::CascadeOrigin,
        production_style_builder::custom_properties::{
            CustomProperties, SubstitutionContext, VariableResolutionError,
        },
    },
};
use cssom::{CSSPropertyRule, CSSPropertySyntaxRepeat as NR, CSSPropertySyntaxType as NT};
use foundation::{AtomicString, String, StringView};
use std::{cell::Cell, collections::HashMap, rc::Rc};

#[derive(Clone)]
pub struct PropertyRegistration {
    pub(crate) syntax: CSSSyntaxDefinition,
    pub(crate) inherits: bool,
    pub(crate) initial: Option<Rc<Value>>,
    pub(crate) initial_data: Option<Rc<CSSVariableData>>,
    referenced: Cell<bool>,
}
impl PropertyRegistration {
    pub fn Syntax(&self) -> &CSSSyntaxDefinition {
        &self.syntax
    }
    pub fn Inherits(&self) -> bool {
        self.inherits
    }
    pub fn Initial(&self) -> Option<&Rc<Value>> {
        self.initial.as_ref()
    }
    /// The typed CSS.registerProperty entry point validates through the same
    /// syntax and independence owners as stylesheet descriptor conversion.
    pub fn new(
        syntax: CSSSyntaxDefinition,
        inherits: bool,
        initial_data: Option<Rc<CSSVariableData>>,
    ) -> Result<Self, SyntaxValueError> {
        let initial = initial_data
            .as_ref()
            .map(|data| syntax.ParseTokens(data))
            .transpose()?;
        if initial
            .as_ref()
            .is_some_and(|v| !crate::property_registration::ComputationallyIndependent(v))
            || initial.is_none() && !syntax.IsUniversal()
        {
            return Err(SyntaxValueError::Invalid);
        }
        Ok(Self {
            syntax,
            inherits,
            initial,
            initial_data,
            referenced: Cell::new(false),
        })
    }
    fn FromRule(rule: &CSSPropertyRule) -> Self {
        let syntax = CSSSyntaxDefinition::new(
            rule.syntax
                .components
                .iter()
                .map(|c| {
                    let kind = match c.kind {
                        NT::kTokenStream => T::kTokenStream,
                        NT::kIdent => T::kIdent,
                        NT::kLength => T::kLength,
                        NT::kNumber => T::kNumber,
                        NT::kPercentage => T::kPercentage,
                        NT::kLengthPercentage => T::kLengthPercentage,
                        NT::kColor => T::kColor,
                        NT::kImage => T::kImage,
                        NT::kUrl => T::kUrl,
                        NT::kInteger => T::kInteger,
                        NT::kAngle => T::kAngle,
                        NT::kTime => T::kTime,
                        NT::kResolution => T::kResolution,
                        NT::kTransformFunction => T::kTransformFunction,
                        NT::kTransformList => T::kTransformList,
                        NT::kCustomIdent => T::kCustomIdent,
                        NT::kString => T::kString,
                    };
                    CSSSyntaxComponent::new(
                        kind,
                        &String::from(c.ident.Utf8().as_str()),
                        match c.repeat {
                            NR::None => R::kNone,
                            NR::SpaceSeparated => R::kSpaceSeparated,
                            NR::CommaSeparated => R::kCommaSeparated,
                        },
                    )
                })
                .collect(),
        );
        let data = rule
            .initial_value_data
            .as_ref()
            .map(|data| Rc::new(RestoreContainerVariableData(data)));
        // These are admitted, validated descriptor tokens, not stylesheet text.
        Self::new(syntax, rule.inherits, data)
            .expect("CSSOM property rule retains validated descriptor tokens")
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyRegistryError {
    AlreadyRegistered(AtomicString),
}
#[derive(Default)]
pub struct PropertyRegistry {
    registered_properties: HashMap<std::string::String, PropertyRegistration>,
    declared_properties: HashMap<std::string::String, PropertyRegistration>,
    declared_rules: HashMap<std::string::String, CSSPropertyRule>,
    version: u64,
}
impl PropertyRegistry {
    pub fn Version(&self) -> u64 {
        self.version
    }
    pub fn IsEmpty(&self) -> bool {
        self.registered_properties.is_empty() && self.declared_properties.is_empty()
    }
    pub fn IsInRegisteredPropertySet(&self, name: &str) -> bool {
        self.registered_properties.contains_key(name)
    }
    pub fn MarkReferenced(&self, name: &str) {
        if let Some(r) = self.Registration(name) {
            r.referenced.set(true);
        }
    }
    pub fn WasReferenced(&self, name: &str) -> bool {
        self.Registration(name).is_some_and(|r| r.referenced.get())
    }
    pub fn Registration(&self, name: &str) -> Option<&PropertyRegistration> {
        self.registered_properties
            .get(name)
            .or_else(|| self.declared_properties.get(name))
    }
    pub fn Registrations(&self) -> impl Iterator<Item = (&str, &PropertyRegistration)> {
        self.registered_properties
            .iter()
            .chain(
                self.declared_properties
                    .iter()
                    .filter(|(n, _)| !self.registered_properties.contains_key(*n)),
            )
            .map(|(n, r)| (n.as_str(), r))
    }
    pub fn RegisterProperty(
        &mut self,
        name: AtomicString,
        registration: PropertyRegistration,
    ) -> Result<(), PropertyRegistryError> {
        if self.registered_properties.contains_key(&name.Utf8()) {
            return Err(PropertyRegistryError::AlreadyRegistered(name));
        }
        let name = name.Utf8();
        self.registered_properties.insert(name, registration);
        self.version = self.version.wrapping_add(1);
        Ok(())
    }
    /// AtRuleCascadeMap: normal origin/layer priority, then last source wins.
    /// Replaces only declarations; script registrations survive every rebuild.
    pub fn UpdateDeclaredProperties(
        &mut self,
        sheets: &[(cssom::css_style_sheet::CSSStyleSheet, CascadeOrigin)],
        values: &DocumentMediaValues<'_>,
    ) -> bool {
        let evaluator = MediaQueryEvaluator::<MediaQueryValueBackend>::ForMediaValues(values);
        let mut layers = HashMap::new();
        for (sheet, origin) in sheets {
            for layer in &sheet.layer_order {
                let next = layers.len() + 1;
                layers.entry((*origin as u8, layer.clone())).or_insert(next);
            }
        }
        let mut winners: HashMap<std::string::String, ((u8, usize, usize, u32), CSSPropertyRule)> =
            HashMap::new();
        for (sheet_index, (sheet, origin)) in sheets.iter().enumerate() {
            // AtRuleCascadeMap only collects user and document author sheets.
            if *origin == CascadeOrigin::kUserAgent {
                continue;
            }
            for rule in &sheet.property_rules {
                if !rule
                    .media_conditions
                    .iter()
                    .all(|condition| evaluator.Eval(&ParseMediaQuerySet(condition)))
                {
                    continue;
                }
                let layer = if rule.layer_name.is_empty() {
                    usize::MAX
                } else {
                    *layers
                        .get(&(*origin as u8, rule.layer_name.clone()))
                        .expect("property layer has declared order")
                };
                let priority = (*origin as u8, layer, sheet_index, rule.source_order);
                let entry = winners
                    .entry(rule.name.Utf8())
                    .or_insert_with(|| (priority, rule.clone()));
                if priority >= entry.0 {
                    *entry = (priority, rule.clone());
                }
            }
        }
        let rules = winners
            .into_iter()
            .map(|(name, (_, rule))| (name, rule))
            .collect::<HashMap<_, _>>();
        if self.declared_rules == rules {
            return false;
        }
        self.declared_properties = rules
            .iter()
            .map(|(name, rule)| (name.clone(), PropertyRegistration::FromRule(rule)))
            .collect();
        self.declared_rules = rules;
        self.version = self.version.wrapping_add(1);
        true
    }
}
/// Concrete represented values use the current cascade's length owner. No
/// parser is duplicated, and emitted substitution tokens are built directly.
// cpp: style_builder_converter.cc:3548-3600,3686-3693.
pub fn ComputeRegisteredValue(
    value: &Rc<Value>,
    length: &dyn Fn(f64, UnitType) -> Result<f64, VariableResolutionError>,
) -> Result<Rc<Value>, VariableResolutionError> {
    Ok(match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(n) => {
            use UnitType::*;
            let number = n.DoubleValue();
            let unit = n.GetType();
            let (number, unit) = if crate::css_numeric_literal_value::IsLength(unit) {
                (length(number, unit)?, kPixels)
            } else {
                match unit {
                    kRadians => (number * 180.0 / std::f64::consts::PI, kDegrees),
                    kGradians => (number * 0.9, kDegrees),
                    kTurns => (number * 360.0, kDegrees),
                    kMilliseconds => (number / 1000.0, kSeconds),
                    kDotsPerInch => (number / 96.0, kDotsPerPixel),
                    kDotsPerCentimeter => (number * 2.54 / 96.0, kDotsPerPixel),
                    kX => (number, kDotsPerPixel),
                    _ => (number, unit),
                }
            };
            values::numeric(number, unit)
        }
        CSSValuePayload::kIdentifierClass(id) if id.0 == foundation::CSSValueID::kCurrentcolor => {
            value.clone()
        }
        CSSValuePayload::kIdentifierClass(id) => {
            values::NamedColor(id.0).map(values::color).ok_or(
                VariableResolutionError::Unsupported("registered system color conversion owner"),
            )?
        }
        CSSValuePayload::kValueListClass(list) => values::list(
            list.values
                .iter()
                .map(|v| ComputeRegisteredValue(v, length))
                .collect::<Result<Vec<_>, _>>()?,
            list.separator,
        ),
        _ => value.clone(),
    })
}
/// Keep nonnumeric original tokens (color/ident/string/universal). For values
/// converted by the computed-value owner emit canonical numeric/list tokens.
pub fn RegisteredVariableData(value: &Value, original: &CSSVariableData) -> Rc<CSSVariableData> {
    fn tokens(value: &Value) -> Option<Vec<VariableToken>> {
        match value.Payload() {
            CSSValuePayload::kNumericLiteralClass(n) => {
                let mut token = CSSParserToken::WithNumber(
                    kNumberToken,
                    n.DoubleValue(),
                    if n.DoubleValue().fract() == 0.0 {
                        NumericValueType::kIntegerValueType
                    } else {
                        NumericValueType::kNumberValueType
                    },
                    NumericSign::kNoSign,
                );
                match n.GetType() {
                    UnitType::kNumber | UnitType::kInteger => {}
                    UnitType::kPercentage => token.ConvertToPercentage(),
                    unit => token.ConvertToDimensionWithUnit(StringView::from(&String::from(
                        UnitTypeToString(unit).expect("numeric registered unit"),
                    ))),
                }
                Some(vec![VariableToken {
                    token,
                    text: value.CssText(),
                }])
            }
            CSSValuePayload::kValueListClass(list) => {
                let mut out = Vec::new();
                for (i, v) in list.values.iter().enumerate() {
                    if i > 0 {
                        let (ty, text) = match list.separator {
                            values::ListSeparator::Space => (kWhitespaceToken, " "),
                            values::ListSeparator::Comma => (kCommaToken, ","),
                            values::ListSeparator::Slash => return None,
                        };
                        out.push(VariableToken {
                            token: CSSParserToken::new(ty, BlockType::kNotBlock),
                            text: String::from(text),
                        });
                    }
                    out.extend(tokens(v)?);
                }
                Some(out)
            }
            _ => None,
        }
    }
    Rc::new(tokens(value).map_or_else(
        || original.clone(),
        |tokens| {
            CSSVariableData::FromTokens(
                tokens,
                original.is_animation_tainted,
                original.is_attr_tainted,
            )
        },
    ))
}
impl RegisteredContainerPropertyResolver for PropertyRegistry {
    fn IsRegistered(&self, name: &AtomicString) -> bool {
        self.Registration(&name.Utf8()).is_some()
    }
    fn ComputeValue(
        &self,
        _node: usize,
        name: &AtomicString,
        specified: Option<&Value>,
        custom: &CustomProperties,
        context: SubstitutionContext<'_>,
        media: &DocumentMediaValues<'_>,
    ) -> RegisteredContainerPropertyResult {
        let registration = self.Registration(&name.Utf8()).expect("registered query");
        let has_random = Cell::new(false);
        let unsupported = Cell::new(None);
        let value = match specified {
            None => registration.initial.clone(),
            Some(v) if v.IsInitialValue() => registration.initial.clone(),
            Some(v) => match v.Payload() {
                CSSValuePayload::kUnparsedDeclarationClass(v) => {
                    has_random.set(v.data.features & CSSVariableData::HAS_RANDOM_FUNCTIONS != 0);
                    match custom.SubstituteWithContext(&v.data, context) {
                        Ok(data) => match registration.syntax.ParseTokens(&data) {
                            Ok(value) => Some(value),
                            Err(SyntaxValueError::Invalid) => None,
                            Err(SyntaxValueError::Unsupported(_)) => {
                                unsupported.set(Some(
                                    "CSSSyntaxDefinition registered query value consumer",
                                ));
                                None
                            }
                        },
                        Err(VariableResolutionError::Unsupported(operation)) => {
                            unsupported.set(Some(operation));
                            None
                        }
                        Err(_) => None,
                    }
                }
                _ => None,
            },
        };
        let flags = Cell::new(0);
        let value = value.and_then(|v| {
            ComputeRegisteredValue(&v, &|n, u| {
                flags.set(flags.get() | LengthConversionFlags(u));
                Ok(media.ComputeLength(n, u))
            })
            .map_err(|error| {
                if let VariableResolutionError::Unsupported(operation) = error {
                    unsupported.set(Some(operation));
                }
            })
            .ok()
        });
        RegisteredContainerPropertyResult {
            value,
            conversion_flags: flags.get(),
            has_random: has_random.get(),
            unsupported: unsupported.get(),
        }
    }

    fn ComputedValue(
        &self,
        _node: usize,
        name: &AtomicString,
        custom: &CustomProperties,
    ) -> Option<Rc<Value>> {
        custom.registered_values.get(&name.Utf8()).cloned()
    }
}

// cpp: css_to_length_conversion_data.h:293-342; media_query_evaluator.cc:1745-1779.
fn LengthConversionFlags(unit: UnitType) -> u32 {
    use crate::media_queries::media_query_container::StyleQueryConversionFlags as F;
    use UnitType::*;
    match unit {
        kEms | kQuirkyEms => F::EM,
        kExs | kChs | kIcs | kCaps => F::GLYPH,
        kRems | kRexs | kRchs | kRics | kRcaps => F::ROOT_FONT,
        kLhs => F::LINE_HEIGHT,
        kRlhs => F::ROOT_LINE_HEIGHT,
        kViewportWidth | kViewportHeight | kViewportInlineSize | kViewportBlockSize
        | kViewportMin | kViewportMax => F::VIEWPORT,
        kSmallViewportWidth
        | kSmallViewportHeight
        | kSmallViewportInlineSize
        | kSmallViewportBlockSize
        | kSmallViewportMin
        | kSmallViewportMax
        | kLargeViewportWidth
        | kLargeViewportHeight
        | kLargeViewportInlineSize
        | kLargeViewportBlockSize
        | kLargeViewportMin
        | kLargeViewportMax => F::SMALL_LARGE_VIEWPORT,
        kDynamicViewportWidth
        | kDynamicViewportHeight
        | kDynamicViewportInlineSize
        | kDynamicViewportBlockSize
        | kDynamicViewportMin
        | kDynamicViewportMax => F::DYNAMIC_VIEWPORT,
        kContainerWidth | kContainerHeight | kContainerInlineSize | kContainerBlockSize
        | kContainerMin | kContainerMax => F::CONTAINER,
        _ => 0,
    }
}
