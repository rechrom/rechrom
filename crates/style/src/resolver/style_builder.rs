//! Translation of Chromium resolver/style_builder.{h,cc}.
//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! style_builder.h: 85 / 22 / 18 / 4 / 0.
//! style_builder.cc: 136 / 57 / 42 / 15 / 0.
//! Effective excludes copyright/comments, blank/preprocessor/include/namespace
//! and lines containing only brackets/punctuation. Header omissions are forward
//! declarations 41-42, STATIC_ONLY 45 and access label 47. cc omissions are pure
//! debug assertions/diagnostics 60,73,82-85,90-92,98,100-102,112,119. All remaining
//! declarations and cc:55-134 production map. CSSPropertyName overload is named
//! ApplyPropertyByName; optional value_mode None supplies the source kNormal
//! default. Initial/inherit/unset selection, parent-child explicit-inheritance
//! flags and surrogate selection are retained; generated dispatch/ref ownership
//! are required typed operations. This source version has no visited/link branch;
//! the actual StyleResolverState already owns those rules.
#![allow(non_snake_case)]
use super::style_resolver_state::{ResolverValue, StyleResolverState, StyleResolverStateBackend};
use crate::css_property_name::CSSPropertyName;
use crate::properties::css_property::CSSProperty;
pub use crate::properties::css_property::{ValueMode, ValueModeFlags};
use foundation::WritingDirectionMode;
use std::marker::PhantomData;

/// CSSPropertyRef owns its source temporary CustomProperty, when applicable.
/// GetProperty returns that exact instance, whose identity remains stable until
/// application ends. Only generated Longhand dispatch and property-ref/surrogate
/// owners are external; all application control lives in StyleBuilder below.
pub trait StyleBuilderBackend: StyleResolverStateBackend {
    type CSSPropertyRef;
    fn NewCSSPropertyRef(
        &self,
        name: &CSSPropertyName,
        document: &Self::Document,
    ) -> Self::CSSPropertyRef;
    fn PropertyRefGetProperty<'a>(&self, property_ref: &'a Self::CSSPropertyRef)
        -> &'a CSSProperty;
    fn PropertySurrogateFor<'a>(
        &self,
        property: &'a CSSProperty,
        direction: WritingDirectionMode,
    ) -> &'a CSSProperty;
    fn LonghandApplyInitial(&self, property: &CSSProperty, state: &StyleResolverState<'_, Self>);
    fn LonghandApplyInherit(&self, property: &CSSProperty, state: &StyleResolverState<'_, Self>);
    fn LonghandApplyValue(
        &self,
        property: &CSSProperty,
        state: &StyleResolverState<'_, Self>,
        value: &ResolverValue<Self>,
        value_mode: ValueModeFlags,
    );
}

pub struct StyleBuilder<B: StyleBuilderBackend>(PhantomData<fn() -> B>);
impl<B: StyleBuilderBackend> StyleBuilder<B> {
    /// CSSPropertyName overload. None supplies the source kNormal default.
    pub fn ApplyPropertyByName(
        backend: &B,
        name: &CSSPropertyName,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
        value_mode: Option<ValueModeFlags>,
    ) {
        let property_ref = backend.NewCSSPropertyRef(name, state.GetDocument());
        let property = backend.PropertyRefGetProperty(&property_ref);
        Self::ApplyProperty(backend, property, state, value, value_mode);
    }
    /// CSSProperty overload. Surrogates are resolved using the builder's current
    /// writing direction before any value handling or generated dispatch.
    pub fn ApplyProperty(
        backend: &B,
        property: &CSSProperty,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
        value_mode: Option<ValueModeFlags>,
    ) {
        let physical = if property.IsSurrogate() {
            let direction = state.StyleBuilder().GetWritingDirection();
            backend.PropertySurrogateFor(property, direction)
        } else {
            property
        };
        Self::ApplyPhysicalProperty(backend, physical, state, value, value_mode);
    }
    pub fn ApplyPhysicalProperty(
        backend: &B,
        property: &CSSProperty,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
        value_mode: Option<ValueModeFlags>,
    ) {
        let _id = property.PropertyID();
        let mut is_inherit = value.IsInheritedValue();
        let mut is_initial = value.IsInitialValue();
        let mut is_unset = value.IsUnsetValue();
        if (is_inherit || is_unset) && state.ParentStyle().is_none() {
            is_inherit = false;
            is_unset = false;
            is_initial = true;
        }
        let is_inherited_for_unset = state.IsInheritedForUnset(property);
        if is_inherit && !is_inherited_for_unset {
            state.StyleBuilderMut().SetHasExplicitInheritance();
            let parent = state.ParentStyle().expect("inherit has a parent style");
            // The Persistent handle roots the actual native ComputedStyle for
            // this access. This flag mutates the source's mutable cache bits.
            unsafe { &*parent.Get() }.SetChildHasExplicitInheritance();
        } else if is_unset {
            if is_inherited_for_unset {
                is_inherit = true;
            } else {
                is_initial = true;
            }
        }
        if is_initial {
            backend.LonghandApplyInitial(property, state);
        } else if is_inherit {
            backend.LonghandApplyInherit(property, state);
        } else {
            backend.LonghandApplyValue(
                property,
                state,
                value,
                value_mode.unwrap_or(ValueMode::kNormal as ValueModeFlags),
            );
        }
    }
}
