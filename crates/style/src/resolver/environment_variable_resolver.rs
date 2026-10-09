// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Required owners for StyleCascade's environment-variable lookup and state.
#![allow(non_snake_case)]
use crate::production_css_value::CSSVariableData;
use crate::{
    style_environment_variables::{
        DocumentStyleEnvironmentVariables, DocumentStyleEnvironmentVariablesBackend,
        StyleEnvironmentVariables, StyleEnvironmentVariablesBackend,
    },
    style_rule::StyleRuleDependencies,
};
use foundation::AtomicString;
use std::rc::Rc;

/// Uses the existing environment-variable owner, including its parent lookup
/// and document observer. The resolver does not synthesize platform geometry.
// cpp: style_environment_variables.cc:233-264; style_cascade.cc:2984-2996.
pub trait EnvironmentVariableSource {
    fn ResolveVariable(&self, name: &AtomicString, indices: &[u32]) -> Option<Rc<CSSVariableData>>;
}
impl<B> EnvironmentVariableSource for StyleEnvironmentVariables<B>
where
    B: StyleEnvironmentVariablesBackend,
    B::RuleDependencies: StyleRuleDependencies<CSSVariableData = CSSVariableData>,
{
    fn ResolveVariable(&self, name: &AtomicString, indices: &[u32]) -> Option<Rc<CSSVariableData>> {
        StyleEnvironmentVariables::ResolveVariable(self, name, indices)
    }
}
impl<B> EnvironmentVariableSource for DocumentStyleEnvironmentVariables<B>
where
    B: DocumentStyleEnvironmentVariablesBackend,
    B::RuleDependencies: StyleRuleDependencies<CSSVariableData = CSSVariableData>,
{
    fn ResolveVariable(&self, name: &AtomicString, indices: &[u32]) -> Option<Rc<CSSVariableData>> {
        StyleEnvironmentVariables::ResolveVariable(std::ops::Deref::deref(self), name, indices)
    }
}
/// These state writes are part of substitution, even when lookup uses fallback.
/// A document integration must provide the actual style builder/engine owner.
// cpp: style_cascade.cc:2356-2364.
pub trait EnvironmentResolutionState {
    fn SetHasEnv(&self);
    fn SetHasEnvSafeAreaInsetBottom(&self);
    fn SetNeedsToUpdateComplexSafeAreaConstraints(&self);
}
/// Runtime feature selection and complete owner operations required by env().
pub trait EnvironmentVariableResolver {
    fn ViewportSegmentsEnabled(&self) -> bool;
    fn ResolveEnvironmentVariable(
        &self,
        name: &AtomicString,
        indices: &[u32],
    ) -> Option<Rc<CSSVariableData>>;
}
/// Adapts the existing neutral environment-variable store, preserving its
/// inherited lookup and observer/invalidation path without text conversion.
pub struct EnvironmentResolver<S, R> {
    source: Rc<S>,
    state: Rc<R>,
    viewport_segments_enabled: bool,
}
impl<S, R> EnvironmentResolver<S, R> {
    pub fn new(source: Rc<S>, state: Rc<R>, viewport_segments_enabled: bool) -> Self {
        Self {
            source,
            state,
            viewport_segments_enabled,
        }
    }
}
impl<S: EnvironmentVariableSource, R: EnvironmentResolutionState> EnvironmentVariableResolver
    for EnvironmentResolver<S, R>
{
    fn ViewportSegmentsEnabled(&self) -> bool {
        self.viewport_segments_enabled
    }
    fn ResolveEnvironmentVariable(
        &self,
        name: &AtomicString,
        indices: &[u32],
    ) -> Option<Rc<CSSVariableData>> {
        self.state.SetHasEnv();
        if name == &AtomicString::from_str("safe-area-inset-bottom") {
            self.state.SetHasEnvSafeAreaInsetBottom();
            self.state.SetNeedsToUpdateComplexSafeAreaConstraints();
        }
        self.source.ResolveVariable(name, indices)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        CustomProperties, ResolvePropertyValue, ResolvePropertyValueWithEnvironment,
        VariableResolutionError,
    };
    use super::*;
    use crate::{
        css_value::CSSValuePayload,
        parser::{
            css_parser_mode::CSSParserMode,
            production_property_parser::{
                ParseCustomProperty, ParseDeclarationList, ParseProperty, PropertyParseErrorKind,
            },
        },
        production_css_value::{PropertyValue, Value},
    };
    use foundation::{CSSPropertyID, String};
    use std::{
        cell::{Cell, RefCell},
        collections::HashMap,
    };

    // Fixture collaborators only supply typed lookup/state. Production env()
    // grammar, token append, fallback and normal consumer logic stay unchanged.
    #[derive(Default)]
    struct Source {
        values: RefCell<HashMap<(AtomicString, Vec<u32>), Rc<CSSVariableData>>>,
        lookups: RefCell<Vec<(AtomicString, Vec<u32>)>>,
    }
    impl Source {
        fn set(&self, name: &str, indices: &[u32], value: &str) {
            self.values.borrow_mut().insert(
                (AtomicString::from_str(name), indices.to_vec()),
                data(value),
            );
        }
    }
    impl EnvironmentVariableSource for Source {
        fn ResolveVariable(
            &self,
            name: &AtomicString,
            indices: &[u32],
        ) -> Option<Rc<CSSVariableData>> {
            self.lookups
                .borrow_mut()
                .push((name.clone(), indices.to_vec()));
            self.values
                .borrow()
                .get(&(name.clone(), indices.to_vec()))
                .cloned()
        }
    }
    #[derive(Default)]
    struct State {
        env: Cell<u32>,
        safe_area_bottom: Cell<u32>,
        complex_constraints: Cell<u32>,
    }
    impl EnvironmentResolutionState for State {
        fn SetHasEnv(&self) {
            self.env.set(self.env.get() + 1);
        }
        fn SetHasEnvSafeAreaInsetBottom(&self) {
            self.safe_area_bottom.set(self.safe_area_bottom.get() + 1);
        }
        fn SetNeedsToUpdateComplexSafeAreaConstraints(&self) {
            self.complex_constraints
                .set(self.complex_constraints.get() + 1);
        }
    }
    fn data(value: &str) -> Rc<CSSVariableData> {
        let property = ParseCustomProperty(
            "--fixture",
            &String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        let CSSValuePayload::kUnparsedDeclarationClass(declaration) = property.Value().Payload()
        else {
            panic!("fixture token stream");
        };
        declaration.data.clone()
    }
    fn declarations(value: &str) -> Vec<PropertyValue> {
        let result = ParseDeclarationList(&String::from(value), CSSParserMode::kHTMLStandardMode);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        result.properties
    }
    struct Context {
        custom: CustomProperties,
        environment: EnvironmentResolver<Source, State>,
    }
    impl std::ops::Deref for Context {
        type Target = CustomProperties;
        fn deref(&self) -> &Self::Target {
            &self.custom
        }
    }
    impl Context {
        fn Substitute(
            &self,
            data: &CSSVariableData,
        ) -> Result<CSSVariableData, VariableResolutionError> {
            self.custom
                .SubstituteWithEnvironment(data, Some(&self.environment))
        }
    }
    fn context(
        source: Rc<Source>,
        state: Rc<State>,
        indices: bool,
        declarations: &[PropertyValue],
    ) -> Context {
        let environment = EnvironmentResolver::new(source, state, indices);
        let custom =
            CustomProperties::ComputeWithEnvironment(None, declarations.iter(), Some(&environment));
        Context {
            custom,
            environment,
        }
    }
    fn resolve(id: CSSPropertyID, value: &str, context: &Context) -> Rc<Value> {
        let declarations = ParseProperty(
            id,
            &String::from(value),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        ResolvePropertyValueWithEnvironment(
            id,
            &declarations[0].ValueRef(),
            &context.custom,
            Some(&context.environment),
        )
        .unwrap()
    }
    #[test]
    fn env_lookup_uses_typed_data_and_native_property_consumer() {
        let source = Rc::new(Source::default());
        source.set("safe-area-inset-top", &[], "17px");
        let state = Rc::new(State::default());
        let custom = context(source.clone(), state.clone(), false, &[]);
        assert_eq!(
            resolve(
                CSSPropertyID::kPaddingTop,
                "env(safe-area-inset-top)",
                &custom
            )
            .CssText()
            .Utf8(),
            "17px"
        );
        assert_eq!(
            *source.lookups.borrow(),
            vec![(AtomicString::from_str("safe-area-inset-top"), vec![])]
        );
        assert_eq!(state.env.get(), 1);
        assert_eq!(state.safe_area_bottom.get(), 0);
    }
    #[test]
    fn env_fallback_trims_trivia_and_can_resolve_var_or_nested_env() {
        let source = Rc::new(Source::default());
        source.set("known", &[], "19px");
        let custom = context(
            source,
            Rc::new(State::default()),
            false,
            &declarations("--width:23px; --trim:env(absent, /**/ var(--width) /**/ );"),
        );
        assert_eq!(custom.Get("--trim").unwrap().original_text.Utf8(), "23px");
        assert_eq!(
            resolve(
                CSSPropertyID::kWidth,
                "env(absent, env(known, 4px))",
                &custom
            )
            .CssText()
            .Utf8(),
            "19px"
        );
        assert!(resolve(CSSPropertyID::kWidth, "env(absent)", &custom).IsUnsetValue());
        assert!(resolve(CSSPropertyID::kWidth, "env(absent,)", &custom).IsUnsetValue());
    }
    #[test]
    fn env_safe_area_state_updates_include_fallback_and_case_sensitive_names() {
        let source = Rc::new(Source::default());
        source.set("Safe-Area-Inset-Bottom", &[], "8px");
        let state = Rc::new(State::default());
        let custom = context(source, state.clone(), false, &[]);
        assert_eq!(
            resolve(
                CSSPropertyID::kPaddingBottom,
                "env(safe-area-inset-bottom, 12px)",
                &custom
            )
            .CssText()
            .Utf8(),
            "12px"
        );
        assert_eq!(
            resolve(
                CSSPropertyID::kPaddingBottom,
                "env(Safe-Area-Inset-Bottom)",
                &custom
            )
            .CssText()
            .Utf8(),
            "8px"
        );
        assert_eq!(state.env.get(), 2);
        assert_eq!(state.safe_area_bottom.get(), 1);
        assert_eq!(state.complex_constraints.get(), 1);
    }
    #[test]
    fn env_viewport_segment_indices_follow_nonnegative_integer_grammar() {
        let source = Rc::new(Source::default());
        source.set("viewport-segment-width", &[1, 0], "200px");
        let custom = context(source.clone(), Rc::new(State::default()), true, &[]);
        assert_eq!(
            resolve(
                CSSPropertyID::kWidth,
                "env(viewport-segment-width 1 0)",
                &custom
            )
            .CssText()
            .Utf8(),
            "200px"
        );
        assert_eq!(
            resolve(
                CSSPropertyID::kWidth,
                "env(viewport-segment-width 0 1, 4px)",
                &custom
            )
            .CssText()
            .Utf8(),
            "4px"
        );
        for value in [
            "env(viewport-segment-width -1 0, 4px)",
            "env(viewport-segment-width 1.0 0, 4px)",
            "env(viewport-segment-width 1px 0, 4px)",
        ] {
            // css_variable_parser.cc:202-249 rejects illegal indices during
            // declaration parsing, before the environment lookup collaborator.
            assert!(
                ParseProperty(
                    CSSPropertyID::kWidth,
                    &String::from(value),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .is_err(),
                "{value}"
            );
        }
        let disabled = context(source, Rc::new(State::default()), false, &[]);
        assert!(resolve(
            CSSPropertyID::kWidth,
            "env(viewport-segment-width 1 0, 4px)",
            &disabled
        )
        .IsUnsetValue());
    }
    #[test]
    fn env_present_value_skips_unused_fallback_and_preserves_token_boundaries() {
        let source = Rc::new(Source::default());
        source.set("number", &[], "12");
        source.set("length", &[], "12px");
        let custom = context(
            source.clone(),
            Rc::new(State::default()),
            false,
            &declarations("--joined:env(number)px;"),
        );
        assert_eq!(
            custom.Get("--joined").unwrap().original_text.Utf8(),
            "12/**/px"
        );
        assert!(resolve(CSSPropertyID::kWidth, "env(number)px", &custom).IsUnsetValue());
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "env(length, env(unused))", &custom)
                .CssText()
                .Utf8(),
            "12px"
        );
        assert!(!source
            .lookups
            .borrow()
            .iter()
            .any(|(name, _)| name == &AtomicString::from_str("unused")));
    }
    #[test]
    fn env_in_custom_values_resolves_before_child_inheritance() {
        let source = Rc::new(Source::default());
        source.set("length", &[], "13px");
        let state = Rc::new(State::default());
        let parent = context(
            source.clone(),
            state.clone(),
            false,
            &declarations("--size:env(length);"),
        );
        source.set("length", &[], "27px");
        let environment = EnvironmentResolver::new(source, state, false);
        let child = Context {
            custom: CustomProperties::ComputeWithEnvironment(
                Some(&parent.custom),
                declarations("--fresh:env(length);").iter(),
                Some(&environment),
            ),
            environment,
        };
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--size)", &child)
                .CssText()
                .Utf8(),
            "13px"
        );
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--fresh)", &child)
                .CssText()
                .Utf8(),
            "27px"
        );
    }
    #[test]
    fn env_missing_collaborators_and_uncomputed_provider_values_stay_unsupported() {
        let absent = CustomProperties::default();
        assert!(matches!(
            absent.Substitute(&data("env(missing, 5px)")),
            Err(VariableResolutionError::Unsupported(
                "StyleCascade::ResolveEnvInto environment owner/state"
            ))
        ));
        let source = Rc::new(Source::default());
        source.set("uncomputed", &[], "var(--x)");
        let custom = context(source, Rc::new(State::default()), false, &[]);
        assert!(matches!(
            custom.Substitute(&data("env(uncomputed, 5px)")),
            Err(VariableResolutionError::Unsupported(
                "StyleEnvironmentVariables::ResolveVariable requires computed data"
            ))
        ));
        let property = ParseProperty(
            CSSPropertyID::kWidth,
            &String::from("env(missing, 5px)"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert_eq!(
            ResolvePropertyValue(CSSPropertyID::kWidth, &property[0].ValueRef(), &absent)
                .err()
                .expect("missing environment collaborator is unsupported")
                .kind,
            PropertyParseErrorKind::Unsupported
        );
    }
    #[test]
    fn env_fallback_obeys_active_custom_property_cycles_and_size_limit() {
        let source = Rc::new(Source::default());
        source.set("large", &[], &"a".repeat(1_050_000));
        let custom = context(
            source,
            Rc::new(State::default()),
            false,
            &declarations("--a:env(absent,var(--a)); --b:var(--a,2px);"),
        );
        assert!(custom.Get("--a").is_none());
        assert_eq!(
            resolve(CSSPropertyID::kWidth, "var(--b)", &custom)
                .CssText()
                .Utf8(),
            "2px"
        );
        assert!(matches!(
            custom.Substitute(&data("env(large)env(large)")),
            Err(VariableResolutionError::TooLarge)
        ));
    }
}
