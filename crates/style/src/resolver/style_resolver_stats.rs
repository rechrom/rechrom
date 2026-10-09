//! Translation of Chromium resolver/style_resolver_stats.{h,cc}.
//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! style_resolver_stats.h: 70 / 20 / 18 / 2 / 0.
//! style_resolver_stats.cc: 77 / 36 / 36 / 0 / 0.
//! Effective excludes copyright/comments, blanks, preprocessing/includes,
//! namespaces and pure bracket/punctuation lines. Header omissions: access
//! label 43 and counter instrumentation macro continuation 66 (its #define
//! line 65 is excluded as preprocessing). All fields, constructor and cc:38-75
//! production map: fourteen u32 counters, Reset and exact ToTracedValue key/order.
//! TracedValue construction/SetInteger belongs solely to its required typed
//! instrumentation owner, without default implementations or a local trace model.
//! This source version has no Aggregate or ToString method.
#![allow(non_snake_case)]

/// The external instrumentation class alone owns its serialization format.
/// Counter selection and serialization order remain in this module.
pub trait StyleResolverStatsTraceBackend {
    type TracedValue;
    fn NewTracedValue(&self) -> Self::TracedValue;
    fn SetInteger(&self, value: &mut Self::TracedValue, name: &'static str, number: i32);
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleResolverStats {
    pub matched_property_apply: u32,
    pub matched_property_cache_hit: u32,
    pub matched_property_cache_added: u32,
    pub rules_fast_rejected: u32,
    pub rules_rejected: u32,
    pub rules_matched: u32,
    pub styles_changed: u32,
    pub styles_unchanged: u32,
    pub styles_animated: u32,
    pub elements_styled: u32,
    pub pseudo_elements_styled: u32,
    pub base_styles_used: u32,
    pub independent_inherited_styles_propagated: u32,
    pub custom_properties_applied: u32,
}
impl StyleResolverStats {
    pub fn new() -> Self {
        let mut result = Self::default();
        result.Reset();
        result
    }
    pub fn Reset(&mut self) {
        self.matched_property_apply = 0;
        self.matched_property_cache_hit = 0;
        self.matched_property_cache_added = 0;
        self.rules_fast_rejected = 0;
        self.rules_rejected = 0;
        self.rules_matched = 0;
        self.styles_changed = 0;
        self.styles_unchanged = 0;
        self.styles_animated = 0;
        self.elements_styled = 0;
        self.pseudo_elements_styled = 0;
        self.base_styles_used = 0;
        self.independent_inherited_styles_propagated = 0;
        self.custom_properties_applied = 0;
    }
    pub fn ToTracedValue<B: StyleResolverStatsTraceBackend>(&self, backend: &B) -> B::TracedValue {
        let mut value = backend.NewTracedValue();
        backend.SetInteger(
            &mut value,
            "matchedPropertyApply",
            self.matched_property_apply as i32,
        );
        backend.SetInteger(
            &mut value,
            "matchedPropertyCacheHit",
            self.matched_property_cache_hit as i32,
        );
        backend.SetInteger(
            &mut value,
            "matchedPropertyCacheAdded",
            self.matched_property_cache_added as i32,
        );
        backend.SetInteger(&mut value, "rulesRejected", self.rules_rejected as i32);
        backend.SetInteger(
            &mut value,
            "rulesFastRejected",
            self.rules_fast_rejected as i32,
        );
        backend.SetInteger(&mut value, "rulesMatched", self.rules_matched as i32);
        backend.SetInteger(&mut value, "stylesChanged", self.styles_changed as i32);
        backend.SetInteger(&mut value, "stylesUnchanged", self.styles_unchanged as i32);
        backend.SetInteger(&mut value, "stylesAnimated", self.styles_animated as i32);
        backend.SetInteger(&mut value, "elementsStyled", self.elements_styled as i32);
        backend.SetInteger(
            &mut value,
            "pseudoElementsStyled",
            self.pseudo_elements_styled as i32,
        );
        backend.SetInteger(&mut value, "baseStylesUsed", self.base_styles_used as i32);
        backend.SetInteger(
            &mut value,
            "independentInheritedStylesPropagated",
            self.independent_inherited_styles_propagated as i32,
        );
        backend.SetInteger(
            &mut value,
            "customPropertiesApplied",
            self.custom_properties_applied as i32,
        );
        value
    }
}
