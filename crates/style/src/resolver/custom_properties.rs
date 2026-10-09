// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Registered/unregistered custom properties and typed var()/env()/attr() substitution.
//! cpp: css_variable_data.h/.cc; css_variable_parser.cc; style_cascade.cc.
#![allow(non_snake_case)]
use crate::{
    css_value::CSSValuePayload,
    parser::{
        css_parser_token::CSSParserTokenType::*,
        production_property_parser::{
            ParsePropertyTokens, PropertyParseError, PropertyParseErrorKind, VariableArguments,
            VariableBlockEnd,
        },
    },
    production_css_value::{self as values, CSSVariableData, PropertyValue, Value, VariableToken},
};
use foundation::{AtomicString, CSSPropertyID, CSSValueID};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[path = "environment_variable_resolver.rs"]
pub mod environment;
pub use environment::EnvironmentVariableResolver;
#[path = "attribute_variable_resolver.rs"]
pub mod attributes;
pub use attributes::AttributeVariableResolver;
/// Per-cascade owners. Borrowing keeps document/style-builder state out of the
/// element's computed custom-property map.
#[derive(Clone, Copy, Default)]
pub struct SubstitutionContext<'a> {
    pub environment: Option<&'a dyn EnvironmentVariableResolver>,
    pub attributes: Option<&'a dyn AttributeVariableResolver>,
}

#[derive(Clone, Debug)]
pub enum ComputedVariableValue {
    Data(Rc<CSSVariableData>),
    Invalid,
    Cyclic,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableResolutionError {
    Invalid,
    Cyclic,
    TooLarge,
    Unsupported(&'static str),
}
/// The inherited computed map is independent of specified declarations. Parent
/// tokens are already resolved: child overrides never re-resolve parent var().
// cpp: style_cascade.cc:1330-1415 ResolveCustomProperty.
#[derive(Clone, Default)]
pub struct CustomProperties {
    pub values: HashMap<std::string::String, ComputedVariableValue>,
    pub registered_values: HashMap<std::string::String, Rc<Value>>,
    pub diagnostics: Vec<(std::string::String, VariableResolutionError)>,
    // cpp: style_cascade.cc:1470-1529 CascadeResolver::shorthand_cache_.
    shorthand_cache: RefCell<Option<(Rc<Value>, Vec<PropertyValue>)>>,
}
impl std::fmt::Debug for CustomProperties {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CustomProperties")
            .field("values", &self.values)
            .field("diagnostics", &self.diagnostics)
            .finish()
    }
}
impl CustomProperties {
    /// `winners` must contain the winning declaration for each custom name.
    /// Cascade priority, origins, layers and revert are handled by the caller.
    pub fn Compute<'a>(
        parent: Option<&Self>,
        winners: impl IntoIterator<Item = &'a PropertyValue>,
    ) -> Self {
        Self::ComputeWithEnvironment(parent, winners, None)
    }
    /// Supplies the document environment owner and its required state updates.
    /// Recompute after an environment invalidation, as for ordinary cascade.
    pub fn ComputeWithEnvironment<'a>(
        parent: Option<&Self>,
        winners: impl IntoIterator<Item = &'a PropertyValue>,
        environment: Option<&dyn EnvironmentVariableResolver>,
    ) -> Self {
        Self::ComputeWithContext(
            parent,
            winners,
            SubstitutionContext {
                environment,
                attributes: None,
            },
        )
    }
    pub fn ComputeWithContext<'a>(
        parent: Option<&Self>,
        winners: impl IntoIterator<Item = &'a PropertyValue>,
        context: SubstitutionContext<'_>,
    ) -> Self {
        Self::ComputeWithRegistry(parent, winners, context, None, &|_, _| {
            Err(VariableResolutionError::Unsupported(
                "registered property length owner",
            ))
        })
    }
    pub fn ComputeWithRegistry<'a>(
        parent: Option<&Self>,
        winners: impl IntoIterator<Item = &'a PropertyValue>,
        context: SubstitutionContext<'_>,
        registry: Option<&crate::property_registry::PropertyRegistry>,
        length: &dyn Fn(
            f64,
            crate::css_primitive_value::UnitType,
        ) -> Result<f64, VariableResolutionError>,
    ) -> Self {
        let parent_values = parent.map_or_else(HashMap::new, |p| p.values.clone());
        let mut inherited = parent_values.clone();
        let mut registered_values =
            parent.map_or_else(HashMap::new, |p| p.registered_values.clone());
        let mut initials = HashMap::new();
        let mut diagnostics = Vec::new();
        if let Some(registry) = registry {
            registered_values.retain(|name, _| registry.Registration(name).is_some());
            for (name, registration) in registry.Registrations() {
                let initial = registration.initial.as_ref().and_then(|value| {
                    match crate::property_registry::ComputeRegisteredValue(value, length) {
                        Ok(value) => Some(value),
                        Err(error) => {
                            diagnostics.push((name.to_owned(), error));
                            None
                        }
                    }
                });
                let data = initial
                    .as_ref()
                    .zip(registration.initial_data.as_ref())
                    .map(|(value, data)| {
                        crate::property_registry::RegisteredVariableData(value, data)
                    });
                let initial_data =
                    data.map_or(ComputedVariableValue::Invalid, ComputedVariableValue::Data);
                initials.insert(name.to_owned(), initial_data.clone());
                if !registration.inherits || !inherited.contains_key(name) {
                    inherited.insert(name.to_owned(), initial_data);
                    registered_values.remove(name);
                    if let Some(initial) = initial {
                        registered_values.insert(name.to_owned(), initial);
                    }
                }
            }
        }
        let mut resolver = VariableResolver {
            registry,
            length,
            parent_values,
            initials,
            registered_values,
            context,
            attribute_stack: Vec::new(),
            inherited,
            specified: HashMap::new(),
            computed: HashMap::new(),
            stack: Vec::new(),
            cyclic: HashSet::new(),
            diagnostics,
        };
        for property in winners {
            if property.PropertyID() == CSSPropertyID::kVariable {
                resolver
                    .specified
                    .insert(property.CustomPropertyName().Utf8(), property.ValueRef());
            }
        }
        let mut names = resolver.specified.keys().cloned().collect::<Vec<_>>();
        names.sort();
        for name in names {
            resolver.ResolveName(&name);
        }
        let mut values = resolver.inherited;
        values.extend(resolver.computed);
        Self {
            values,
            registered_values: resolver.registered_values,
            diagnostics: resolver.diagnostics,
            shorthand_cache: RefCell::default(),
        }
    }
    pub fn Get(&self, name: &str) -> Option<&Rc<CSSVariableData>> {
        match self.values.get(name) {
            Some(ComputedVariableValue::Data(data)) => Some(data),
            _ => None,
        }
    }
    pub fn Substitute(
        &self,
        data: &CSSVariableData,
    ) -> Result<CSSVariableData, VariableResolutionError> {
        self.SubstituteWithEnvironment(data, None)
    }
    /// The environment context belongs to the current cascade invocation and
    /// is never retained in the element's computed custom-property map.
    pub fn SubstituteWithEnvironment(
        &self,
        data: &CSSVariableData,
        environment: Option<&dyn EnvironmentVariableResolver>,
    ) -> Result<CSSVariableData, VariableResolutionError> {
        self.SubstituteWithContext(
            data,
            SubstitutionContext {
                environment,
                attributes: None,
            },
        )
    }
    pub fn SubstituteWithContext(
        &self,
        data: &CSSVariableData,
        context: SubstitutionContext<'_>,
    ) -> Result<CSSVariableData, VariableResolutionError> {
        let mut resolver = VariableResolver {
            registry: None,
            length: &|_, _| Err(VariableResolutionError::Invalid),
            parent_values: self.values.clone(),
            initials: HashMap::new(),
            registered_values: self.registered_values.clone(),
            context,
            attribute_stack: Vec::new(),
            inherited: self.values.clone(),
            specified: HashMap::new(),
            computed: HashMap::new(),
            stack: Vec::new(),
            cyclic: HashSet::new(),
            diagnostics: Vec::new(),
        };
        resolver.ResolveData(data)
    }
}
struct VariableResolver<'env> {
    context: SubstitutionContext<'env>,
    registry: Option<&'env crate::property_registry::PropertyRegistry>,
    length: &'env dyn Fn(
        f64,
        crate::css_primitive_value::UnitType,
    ) -> Result<f64, VariableResolutionError>,
    parent_values: HashMap<std::string::String, ComputedVariableValue>,
    initials: HashMap<std::string::String, ComputedVariableValue>,
    registered_values: HashMap<std::string::String, Rc<Value>>,
    // Attribute cycle nodes record the property stack position on entry.
    attribute_stack: Vec<(AtomicString, usize)>,
    inherited: HashMap<std::string::String, ComputedVariableValue>,
    specified: HashMap<std::string::String, Rc<Value>>,
    computed: HashMap<std::string::String, ComputedVariableValue>,
    stack: Vec<std::string::String>,
    cyclic: HashSet<std::string::String>,
    diagnostics: Vec<(std::string::String, VariableResolutionError)>,
}
impl VariableResolver<'_> {
    // cpp: style_cascade.cc:1775-1824; cascade_resolver.cc DetectCycle/AutoLock.
    fn ResolveName(&mut self, name: &str) -> ComputedVariableValue {
        if let Some(index) = self.stack.iter().position(|entry| entry == name) {
            self.cyclic.extend(self.stack[index..].iter().cloned());
            return ComputedVariableValue::Cyclic;
        }
        if let Some(value) = self.computed.get(name) {
            return value.clone();
        }
        let Some(specified) = self.specified.get(name).cloned() else {
            return self
                .inherited
                .get(name)
                .cloned()
                .unwrap_or(ComputedVariableValue::Invalid);
        };
        self.stack.push(name.to_owned());
        let value = if specified.IsInheritedValue() {
            self.parent_values
                .get(name)
                .cloned()
                .or_else(|| self.initials.get(name).cloned())
                .unwrap_or(ComputedVariableValue::Invalid)
        } else if specified.IsUnsetValue() {
            self.inherited
                .get(name)
                .cloned()
                .unwrap_or(ComputedVariableValue::Invalid)
        } else if specified.IsInitialValue() {
            self.initials
                .get(name)
                .cloned()
                .unwrap_or(ComputedVariableValue::Invalid)
        } else if let CSSValuePayload::kUnparsedDeclarationClass(declaration) = specified.Payload()
        {
            match self.ResolveData(&declaration.data) {
                Ok(data) => {
                    // A substituted CSS-wide keyword determines the custom
                    // property's value as if it had been specified directly.
                    let significant = data
                        .tokens
                        .iter()
                        .filter(|item| {
                            !matches!(item.token.GetType(), kWhitespaceToken | kCommentToken)
                        })
                        .collect::<Vec<_>>();
                    if significant.len() == 1 {
                        match significant[0].token.Id() {
                            CSSValueID::kInitial => self
                                .initials
                                .get(name)
                                .cloned()
                                .unwrap_or(ComputedVariableValue::Invalid),
                            CSSValueID::kInherit => self
                                .parent_values
                                .get(name)
                                .cloned()
                                .or_else(|| self.initials.get(name).cloned())
                                .unwrap_or(ComputedVariableValue::Invalid),
                            CSSValueID::kUnset => self
                                .inherited
                                .get(name)
                                .cloned()
                                .unwrap_or(ComputedVariableValue::Invalid),
                            CSSValueID::kRevert | CSSValueID::kRevertLayer => {
                                self.diagnostics.push((
                                    name.to_owned(),
                                    VariableResolutionError::Unsupported(
                                        "StyleCascade::ResolveRevert custom property",
                                    ),
                                ));
                                ComputedVariableValue::Invalid
                            }
                            _ => ComputedVariableValue::Data(Rc::new(data)),
                        }
                    } else {
                        ComputedVariableValue::Data(Rc::new(data))
                    }
                }
                Err(error) => {
                    if matches!(error, VariableResolutionError::Unsupported(_)) {
                        self.diagnostics.push((name.to_owned(), error));
                    }
                    if self.cyclic.contains(name) {
                        ComputedVariableValue::Cyclic
                    } else {
                        ComputedVariableValue::Invalid
                    }
                }
            }
        } else {
            self.diagnostics.push((
                name.to_owned(),
                VariableResolutionError::Unsupported(
                    "StyleCascade::Resolve custom property cascade keyword",
                ),
            ));
            ComputedVariableValue::Invalid
        };
        self.stack.pop();
        let value = if self.cyclic.contains(name) {
            ComputedVariableValue::Cyclic
        } else {
            value
        };
        // cpp: custom_property.cc:143-225. Typed validation happens while
        // resolving each dependency, before another var() can substitute it.
        let value = if let Some(registration) = self.registry.and_then(|r| r.Registration(name)) {
            match &value {
                ComputedVariableValue::Data(data) => {
                    let typed = registration
                        .syntax
                        .ParseTokens(data)
                        .map_err(|error| match error {
                            crate::css_syntax_definition::SyntaxValueError::Invalid => {
                                VariableResolutionError::Invalid
                            }
                            crate::css_syntax_definition::SyntaxValueError::Unsupported(_) => {
                                VariableResolutionError::Unsupported(
                                    "CSSSyntaxDefinition registered value consumer",
                                )
                            }
                        })
                        .and_then(|value| {
                            crate::property_registry::ComputeRegisteredValue(&value, self.length)
                        });
                    match typed {
                        Ok(typed) => {
                            let data =
                                crate::property_registry::RegisteredVariableData(&typed, data);
                            self.registered_values.insert(name.to_owned(), typed);
                            ComputedVariableValue::Data(data)
                        }
                        Err(error) => {
                            if matches!(error, VariableResolutionError::Unsupported(_)) {
                                self.diagnostics.push((name.to_owned(), error));
                            }
                            self.inherited
                                .get(name)
                                .cloned()
                                .unwrap_or(ComputedVariableValue::Invalid)
                        }
                    }
                }
                _ if !registration.syntax.IsUniversal() => self
                    .inherited
                    .get(name)
                    .cloned()
                    .unwrap_or(ComputedVariableValue::Invalid),
                _ => {
                    self.registered_values.remove(name);
                    value
                }
            }
        } else {
            value
        };
        self.computed.insert(name.to_owned(), value.clone());
        value
    }
    // cpp: style_cascade.cc:1652-1742 ResolveVariableData/ResolveTokensInto;
    // 1744-1828 ResolveVarInto; 2103-2124 AppendDataWithFallback.
    fn ResolveData(
        &mut self,
        data: &CSSVariableData,
    ) -> Result<CSSVariableData, VariableResolutionError> {
        let mut output = Vec::new();
        let mut length = 0;
        let mut animation = data.is_animation_tainted;
        let mut attr = data.is_attr_tainted;
        self.ResolveTokens(
            &data.tokens,
            &mut output,
            &mut length,
            &mut animation,
            &mut attr,
        )?;
        Ok(CSSVariableData::FromTokens(output, animation, attr))
    }
    fn Append(
        tokens: &[VariableToken],
        output: &mut Vec<VariableToken>,
        length: &mut usize,
    ) -> Result<(), VariableResolutionError> {
        for item in tokens {
            // cpp: style_cascade.cc:1082-1120,1139-1156 and
            // css_parser_token.cc:308-350. Comments make serialized computed
            // data preserve the same boundaries as the retained token stream.
            if output.last().is_some_and(|previous| {
                crate::parser::css_parser_token::NeedsInsertedComment(&previous.token, &item.token)
            }) {
                *length += 4;
                output.push(VariableToken {
                    token: crate::parser::css_parser_token::CSSParserToken::new(
                        kCommentToken,
                        crate::parser::css_parser_token::BlockType::kNotBlock,
                    ),
                    text: foundation::String::from("/**/"),
                });
            }
            *length += item.text.length() as usize;
            if *length > CSSVariableData::MAX_VARIABLE_BYTES {
                return Err(VariableResolutionError::TooLarge);
            }
            output.push(item.clone());
        }
        Ok(())
    }
    // cpp: style_cascade.cc:2097-2124 AppendDataWithFallback; 1039-1074 AppendFallback.
    fn AppendDataWithFallback(
        &mut self,
        data: Option<&Rc<CSSVariableData>>,
        fallback: Option<&[VariableToken]>,
        output: &mut Vec<VariableToken>,
        length: &mut usize,
        animation: &mut bool,
        attr: &mut bool,
    ) -> Result<(), VariableResolutionError> {
        if let Some(data) = data {
            *animation |= data.is_animation_tainted;
            *attr |= data.is_attr_tainted;
            return Self::Append(&data.tokens, output, length);
        }
        let fallback = fallback.ok_or(VariableResolutionError::Invalid)?;
        let first = fallback
            .iter()
            .position(|item| !matches!(item.token.GetType(), kWhitespaceToken | kCommentToken))
            .unwrap_or(fallback.len());
        let mut fallback_output = Vec::new();
        let mut fallback_length = 0;
        self.ResolveTokens(
            &fallback[first..],
            &mut fallback_output,
            &mut fallback_length,
            animation,
            attr,
        )?;
        if *length + fallback_length > CSSVariableData::MAX_VARIABLE_BYTES {
            return Err(VariableResolutionError::TooLarge);
        }
        while fallback_output
            .last()
            .is_some_and(|item| matches!(item.token.GetType(), kWhitespaceToken | kCommentToken))
        {
            fallback_output.pop();
        }
        Self::Append(&fallback_output, output, length)
    }
    // cpp: style_cascade.cc:2392-2519 ResolveAttrInto; 1700-1703 SetHasAttrFunction;
    // css_variable_parser.cc:255-293 ConsumeAttributeReference.
    fn ResolveAttr(
        &mut self,
        tokens: &[VariableToken],
        output: &mut Vec<VariableToken>,
        length: &mut usize,
        animation: &mut bool,
        attr: &mut bool,
    ) -> Result<(), VariableResolutionError> {
        let attributes = self
            .context
            .attributes
            .ok_or(VariableResolutionError::Unsupported(
                "StyleCascade::ResolveAttrInto attribute owner/state",
            ))?;
        attributes.SetHasAttrFunction();
        let mut comma = tokens.len();
        let mut index = 0;
        while index < tokens.len() {
            if tokens[index].token.GetType() == kCommaToken {
                comma = index;
                break;
            }
            if tokens[index].token.GetBlockType()
                == crate::parser::css_parser_token::BlockType::kBlockStart
            {
                let end = VariableBlockEnd(tokens, index);
                index = if end == tokens.len() { end } else { end + 1 };
            } else {
                index += 1;
            }
        }
        let first = &tokens[..comma];
        let fallback = if comma < tokens.len() {
            Some(&tokens[comma + 1..])
        } else {
            None
        };
        let first_output;
        let first = if attributes.CSSArgumentGrammarEnabled() {
            first_output =
                self.ResolveData(&CSSVariableData::FromTokens(first.to_vec(), false, false))?;
            first_output.tokens.as_slice()
        } else {
            first
        };
        let (name, kind, missing_type) = attributes::ConsumeAttributeHeader(first)?;
        if let Some((_, property_depth)) = self
            .attribute_stack
            .iter()
            .find(|(active, _)| active == &name)
        {
            self.cyclic
                .extend(self.stack[*property_depth..].iter().cloned());
            return Err(VariableResolutionError::Cyclic);
        }
        self.attribute_stack.push((name.clone(), self.stack.len()));
        let result = (|| {
            let raw = attributes.GetAttribute(&name);
            let data = attributes::ParseAttributeValue(&raw, kind)?;
            // All attr() results, including fallback, are attr-tainted.
            *attr = true;
            if data.is_none() && fallback.is_none() && missing_type {
                let empty = attributes::StringData(&foundation::String::from(""), true);
                self.AppendDataWithFallback(Some(&empty), None, output, length, animation, attr)
            } else {
                self.AppendDataWithFallback(
                    data.as_ref(),
                    fallback,
                    output,
                    length,
                    animation,
                    attr,
                )
            }
        })();
        self.attribute_stack.pop();
        result
    }
    fn ResolveTokens(
        &mut self,
        tokens: &[VariableToken],
        output: &mut Vec<VariableToken>,
        length: &mut usize,
        animation: &mut bool,
        attr: &mut bool,
    ) -> Result<(), VariableResolutionError> {
        let mut index = 0;
        while index < tokens.len() {
            let token = &tokens[index].token;
            if token.FunctionId() == Some(CSSValueID::kVar) {
                let end = VariableBlockEnd(tokens, index);
                let (name, fallback) = VariableArguments(&tokens[index + 1..end])
                    .ok_or(VariableResolutionError::Invalid)?;
                if let Some(registry) = self.registry {
                    registry.MarkReferenced(&name);
                }
                let variable = self.ResolveName(&name);
                // Once a member of the active dependency cycle is detected,
                // fallback inside that cycle cannot make its value valid.
                if self.stack.iter().any(|name| self.cyclic.contains(name)) {
                    return Err(VariableResolutionError::Cyclic);
                }
                let data = match variable {
                    ComputedVariableValue::Data(data) => Some(data),
                    ComputedVariableValue::Invalid | ComputedVariableValue::Cyclic => None,
                };
                self.AppendDataWithFallback(
                    data.as_ref(),
                    fallback,
                    output,
                    length,
                    animation,
                    attr,
                )?;
                index = if end == tokens.len() { end } else { end + 1 };
            } else if token.FunctionId() == Some(CSSValueID::kEnv) {
                let end = VariableBlockEnd(tokens, index);
                let environment =
                    self.context
                        .environment
                        .ok_or(VariableResolutionError::Unsupported(
                            "StyleCascade::ResolveEnvInto environment owner/state",
                        ))?;
                let (name, indices, fallback) = ParseEnvironmentArguments(
                    &tokens[index + 1..end],
                    environment.ViewportSegmentsEnabled(),
                )?;
                let data = environment.ResolveEnvironmentVariable(&name, &indices);
                if data
                    .as_ref()
                    .is_some_and(|data| data.NeedsVariableResolution())
                {
                    return Err(VariableResolutionError::Unsupported(
                        "StyleEnvironmentVariables::ResolveVariable requires computed data",
                    ));
                }
                self.AppendDataWithFallback(
                    data.as_ref(),
                    fallback,
                    output,
                    length,
                    animation,
                    attr,
                )?;
                index = if end == tokens.len() { end } else { end + 1 };
            } else if token.FunctionId() == Some(CSSValueID::kAttr) {
                let end = VariableBlockEnd(tokens, index);
                self.ResolveAttr(&tokens[index + 1..end], output, length, animation, attr)?;
                index = if end == tokens.len() { end } else { end + 1 };
            } else {
                if token.GetType() == kFunctionToken {
                    let operation = match token
                        .Value()
                        .ToString()
                        .Utf8()
                        .to_ascii_lowercase()
                        .as_str()
                    {
                        "inherit" => Some("StyleCascade::ResolveInheritInto"),
                        "if" => Some("StyleCascade::ResolveIfInto"),
                        _ => None,
                    };
                    if let Some(operation) = operation {
                        return Err(VariableResolutionError::Unsupported(operation));
                    }
                }
                Self::Append(&tokens[index..index + 1], output, length)?;
                index += 1;
            }
        }
        Ok(())
    }
}

// cpp: css_variable_parser.cc:202-248 ConsumeEnvVariableReference;
// style_cascade.cc:2351-2390 ResolveEnvInto.
fn ParseEnvironmentArguments(
    tokens: &[VariableToken],
    viewport_segments: bool,
) -> Result<(AtomicString, Vec<u32>, Option<&[VariableToken]>), VariableResolutionError> {
    use crate::parser::css_parser_token::NumericValueType;
    let mut index = 0;
    let skip_trivia = |index: &mut usize| {
        while tokens
            .get(*index)
            .is_some_and(|item| matches!(item.token.GetType(), kWhitespaceToken | kCommentToken))
        {
            *index += 1;
        }
    };
    skip_trivia(&mut index);
    let token = &tokens
        .get(index)
        .ok_or(VariableResolutionError::Invalid)?
        .token;
    if token.GetType() != kIdentToken {
        return Err(VariableResolutionError::Invalid);
    }
    let name = AtomicString::from_utf16(token.Value().ToString().Span16().unwrap_or_default());
    index += 1;
    skip_trivia(&mut index);
    let mut indices = Vec::new();
    if viewport_segments {
        while let Some(item) = tokens
            .get(index)
            .filter(|item| item.token.GetType() == kNumberToken)
        {
            if item.token.GetNumericValueType() != NumericValueType::kIntegerValueType
                || item.token.NumericValue() < 0.0
            {
                return Err(VariableResolutionError::Invalid);
            }
            if item.token.NumericValue() > u32::MAX as f64 {
                return Err(VariableResolutionError::Unsupported(
                    "StyleCascade::ResolveEnvInto unsigned index overflow",
                ));
            }
            indices.push(item.token.NumericValue() as u32);
            index += 1;
            skip_trivia(&mut index);
        }
    }
    match tokens.get(index) {
        None => Ok((name, indices, None)),
        Some(item) if item.token.GetType() == kCommaToken => {
            Ok((name, indices, Some(&tokens[index + 1..])))
        }
        _ => Err(VariableResolutionError::Invalid),
    }
}

/// Resolves both deferred longhands and pending shorthand values. Invalid at
/// computed-value time becomes unset; untranslated consumers remain typed errors.
// cpp: style_cascade.cc:1418-1532 ResolveVariableReference/ResolvePendingSubstitution.
pub fn ResolvePropertyValue(
    id: CSSPropertyID,
    value: &Rc<Value>,
    custom: &CustomProperties,
) -> Result<Rc<Value>, PropertyParseError> {
    ResolvePropertyValueWithEnvironment(id, value, custom, None)
}
/// Receives the current cascade's environment owner/state by reference.
/// Explicit environments bypass the persistent shorthand cache, because their
/// geometry can be invalidated independently of this computed variable map.
pub fn ResolvePropertyValueWithEnvironment(
    id: CSSPropertyID,
    value: &Rc<Value>,
    custom: &CustomProperties,
    environment: Option<&dyn EnvironmentVariableResolver>,
) -> Result<Rc<Value>, PropertyParseError> {
    ResolvePropertyValueWithContext(
        id,
        value,
        custom,
        SubstitutionContext {
            environment,
            attributes: None,
        },
    )
}
pub fn ResolvePropertyValueWithContext(
    id: CSSPropertyID,
    value: &Rc<Value>,
    custom: &CustomProperties,
    context: SubstitutionContext<'_>,
) -> Result<Rc<Value>, PropertyParseError> {
    let (parse_id, declaration) = match value.Payload() {
        CSSValuePayload::kUnparsedDeclarationClass(declaration) => (id, declaration),
        CSSValuePayload::kPendingSubstitutionValueClass(pending) => {
            let CSSValuePayload::kUnparsedDeclarationClass(declaration) = pending.value.Payload()
            else {
                unreachable!("pending substitution contains unparsed declaration")
            };
            (pending.shorthand, declaration)
        }
        _ => return Ok(value.clone()),
    };
    let unset = || values::wide(CSSValueID::kUnset).unwrap();
    if context.environment.is_none()
        && context.attributes.is_none()
        && value.IsPendingSubstitutionValue()
    {
        if let Some((cached, properties)) = custom.shorthand_cache.borrow().as_ref() {
            if Rc::ptr_eq(cached, value) {
                return Ok(properties
                    .iter()
                    .find(|p| p.PropertyID() == id)
                    .map_or_else(unset, |p| p.ValueRef()));
            }
        }
    }
    let data = match custom.SubstituteWithContext(&declaration.data, context) {
        Ok(data) => data,
        Err(VariableResolutionError::Unsupported(operation)) => {
            return Err(PropertyParseError {
                property: id,
                name: crate::css_property_names::GetPropertyName(id).to_owned(),
                offset: 0,
                kind: PropertyParseErrorKind::Unsupported,
                operation,
                source_file: if operation.starts_with("CSSAttrType::Consume") {
                    "third_party/blink/renderer/core/css/css_attr_type.cc"
                } else {
                    "third_party/blink/renderer/core/css/resolver/style_cascade.cc"
                },
                source_line: if operation.starts_with("CSSAttrType::Consume") {
                    66
                } else if operation.starts_with("StyleCascade::ResolveEnvInto") {
                    2351
                } else if operation.starts_with("StyleCascade::ResolveAttrInto") {
                    2392
                } else {
                    1672
                },
            });
        }
        Err(_) => return Ok(unset()),
    };
    match ParsePropertyTokens(parse_id, &data, declaration.mode) {
        Ok(properties) => {
            let resolved = properties
                .iter()
                .find(|p| p.PropertyID() == id)
                .map_or_else(unset, |p| p.ValueRef());
            if context.environment.is_none()
                && context.attributes.is_none()
                && value.IsPendingSubstitutionValue()
            {
                *custom.shorthand_cache.borrow_mut() = Some((value.clone(), properties));
            }
            Ok(resolved)
        }
        Err(error) if error.kind == PropertyParseErrorKind::Unsupported => Err(error),
        Err(_) => Ok(unset()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty},
    };
    use foundation::String;
    fn declarations(css: &str) -> Vec<PropertyValue> {
        let parsed = ParseDeclarationList(&String::from(css), CSSParserMode::kHTMLStandardMode);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        parsed.properties
    }
    fn custom(css: &str) -> CustomProperties {
        CustomProperties::Compute(None, declarations(css).iter())
    }
    fn resolve(id: CSSPropertyID, css: &str, custom: &CustomProperties) -> Rc<Value> {
        let parsed = ParseProperty(
            id,
            &String::from(css),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        ResolvePropertyValue(id, &parsed[0].ValueRef(), custom).unwrap()
    }
    #[test]
    fn preserve_custom_tokens_case_comments_and_importance() {
        let parsed = declarations(
            "--X: 12/**/px !IMPORTANT; --x: [one; two] { color: red!important; }; --empty: /**/;",
        );
        assert!(parsed[0].IsImportant());
        assert_eq!(parsed[0].CustomPropertyName().Utf8(), "--X");
        let custom = CustomProperties::Compute(None, parsed.iter());
        let tokens = &custom.Get("--X").unwrap().tokens;
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token.GetType(), kNumberToken);
        assert_eq!(tokens[1].token.GetType(), kCommentToken);
        assert_eq!(tokens[2].token.GetType(), kIdentToken);
        assert_ne!(
            custom.Get("--X").unwrap().original_text,
            custom.Get("--x").unwrap().original_text
        );
        assert!(custom.Get("--empty").unwrap().tokens.is_empty());
    }
    #[test]
    fn inherits_computed_values_before_child_overrides() {
        let parent = custom("--x: 12px; --y: var(--x);");
        let child = CustomProperties::Compute(
            Some(&parent),
            declarations("--x: 22px; --z:var(--y);").iter(),
        );
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--y)", &child)
                .CssText()
                .Utf8(),
            "12px"
        );
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--z)", &child)
                .CssText()
                .Utf8(),
            "12px"
        );
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--x)", &child)
                .CssText()
                .Utf8(),
            "22px"
        );
    }
    #[test]
    fn nested_fallback_and_function_substitution() {
        let custom = custom("--red: 255; --green: 0; --blue:0; --width: 42px;");
        assert_eq!(
            resolve(
                CSSPropertyID::kWidth,
                "var(--missing, var(--width, 20px))",
                &custom
            )
            .CssText()
            .Utf8(),
            "42px"
        );
        assert_eq!(
            resolve(
                CSSPropertyID::kColor,
                "rgb(var(--red),var(--green),var(--blue))",
                &custom
            )
            .CssText()
            .Utf8(),
            "rgb(255, 0, 0)"
        );
        assert!(resolve(CSSPropertyID::kWidth, "var(--missing,)", &custom).IsUnsetValue());
    }
    #[test]
    fn cycles_invalidate_members_even_with_internal_fallback() {
        let custom = custom(
            "--a:var(--b, 1px); --b:var(--a, 2px); --c:var(--a, 3px); --self:var(--self, 4px);",
        );
        assert!(matches!(
            custom.values.get("--a"),
            Some(ComputedVariableValue::Cyclic)
        ));
        assert!(matches!(
            custom.values.get("--b"),
            Some(ComputedVariableValue::Cyclic)
        ));
        assert!(matches!(
            custom.values.get("--self"),
            Some(ComputedVariableValue::Cyclic)
        ));
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--c)", &custom)
                .CssText()
                .Utf8(),
            "3px"
        );
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--a, 5px)", &custom)
                .CssText()
                .Utf8(),
            "5px"
        );
    }
    #[test]
    fn unused_fallback_does_not_resolve_dependencies() {
        let custom = custom("--a: 5px; --b:var(--a,var(--b));");
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--b)", &custom)
                .CssText()
                .Utf8(),
            "5px"
        );
    }
    #[test]
    fn substituted_tokens_do_not_merge_into_dimension_or_ident() {
        let custom = custom(
            "--number:12; --unit:px; --word:in; --empty:; --joined:var(--number)var(--unit); --trimmed:var(--missing, 5px /**/ );",
        );
        assert_eq!(
            custom.Get("--joined").unwrap().original_text.Utf8(),
            "12/**/px"
        );
        assert_eq!(custom.Get("--trimmed").unwrap().original_text.Utf8(), "5px");
        assert!(resolve(CSSPropertyID::kWidth, "var(--number)px", &custom).IsUnsetValue());
        assert!(resolve(CSSPropertyID::kWidth, "var(--number)var(--unit)", &custom).IsUnsetValue());
        assert!(resolve(CSSPropertyID::kDisplay, "var(--word)line", &custom).IsUnsetValue());
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--empty)12px", &custom)
                .CssText()
                .Utf8(),
            "12px"
        );
    }
    #[test]
    fn pending_shorthand_expands_after_substitution() {
        let custom = custom("--edge: 2px 4px;");
        let properties = ParseProperty(
            CSSPropertyID::kMargin,
            &String::from("var(--edge) !important"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(properties.len(), 4);
        for property in &properties {
            assert!(property.IsImportant());
            assert!(property.Value().IsPendingSubstitutionValue());
            let expected = if matches!(
                property.PropertyID(),
                CSSPropertyID::kMarginTop | CSSPropertyID::kMarginBottom
            ) {
                "2px"
            } else {
                "4px"
            };
            assert_eq!(
                ResolvePropertyValue(property.PropertyID(), &property.ValueRef(), &custom)
                    .unwrap()
                    .CssText()
                    .Utf8(),
                expected
            );
        }
    }
    #[test]
    fn custom_css_wide_initial_inherit_unset() {
        let parent = custom("--a: 7px; --b: 8px; --c:9px;");
        let child = CustomProperties::Compute(
            Some(&parent),
            declarations("--a:initial; --b:inherit; --c:unset; --d:var(--notset,inherit);").iter(),
        );
        assert!(child.Get("--a").is_none());
        assert_eq!(child.Get("--b").unwrap().original_text.Utf8(), "8px");
        assert_eq!(child.Get("--c").unwrap().original_text.Utf8(), "9px");
        assert!(child.Get("--d").is_none());
        assert!(
            resolve(CSSPropertyID::kWidth, "var(--notset, inherit)", &child).IsInheritedValue()
        );
    }
    #[test]
    fn malformed_var_and_positioned_braces_are_rejected() {
        for css in [
            "width:var(color);",
            "width:var(--x extra);",
            "--x:var(--);",
            "--x:var(--y, var(color));",
            "width:foo {} var(--x);",
        ] {
            let parsed = ParseDeclarationList(&String::from(css), CSSParserMode::kHTMLStandardMode);
            assert!(parsed.properties.is_empty(), "{css}");
            assert_eq!(parsed.errors.len(), 1, "{css}");
            assert_eq!(parsed.errors[0].kind, PropertyParseErrorKind::Invalid);
        }
    }
    #[test]
    fn substitution_rechecks_property_grammar() {
        let custom = custom("--invalid:red; --negative:-2px;");
        assert!(resolve(CSSPropertyID::kWidth, "var(--invalid)", &custom).IsUnsetValue());
        assert!(resolve(CSSPropertyID::kPaddingLeft, "var(--negative)", &custom).IsUnsetValue());
        assert!(resolve(CSSPropertyID::kWidth, "var(--missing)", &custom).IsUnsetValue());
    }
    #[test]
    fn substitution_limit_stops_exponential_expansion() {
        let css = format!(
            "--x0:{}; --x1:var(--x0)var(--x0); --x2:var(--x1)var(--x1);",
            "a".repeat(700_000)
        );
        let custom = custom(&css);
        assert!(custom.Get("--x1").is_some());
        assert!(custom.Get("--x2").is_none());
    }
}

#[cfg(test)]
mod token_storage_tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode, production_property_parser::ParseCustomProperty,
    };
    use foundation::String;
    #[test]
    fn source_serialization_handles_eof_escape_and_preserves_utf16() {
        for (css, expected) in [
            ("abc\\", "abc\u{fffd}"),
            ("\"abc\\", "\"abc\""),
            ("url(abc\\", "url(abc\u{fffd})"),
        ] {
            let property = ParseCustomProperty(
                "--x",
                &String::from(css),
                false,
                CSSParserMode::kHTMLStandardMode,
            )
            .unwrap();
            assert_eq!(property.Value().CssText().Utf8(), expected);
        }
        let input = String::from_utf16(&[b'a' as u16, 0xd800]);
        let property =
            ParseCustomProperty("--x", &input, false, CSSParserMode::kHTMLStandardMode).unwrap();
        let CSSValuePayload::kUnparsedDeclarationClass(declaration) = property.Value().Payload()
        else {
            panic!("unparsed");
        };
        assert_eq!(
            declaration.data.original_text.Span16().unwrap(),
            &[b'a' as u16, 0xd800]
        );
    }
}
