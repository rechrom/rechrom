// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Required owners for StyleCascade's environment-variable lookup and state.
#![allow(non_snake_case)]
use crate::production_css_value::CSSVariableData;
use foundation::AtomicString;
use std::rc::Rc;

/// Uses the existing environment-variable owner, including its parent lookup
/// and document observer. The resolver does not synthesize platform geometry.
// cpp: style_environment_variables.cc:233-264; style_cascade.cc:2984-2996.
pub trait EnvironmentVariableSource {
    fn ResolveVariable(&self, name: &AtomicString, indices: &[u32]) -> Option<Rc<CSSVariableData>>;
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
    fn HasEnvironmentVariableReference(&self) -> bool {
        false
    }
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
