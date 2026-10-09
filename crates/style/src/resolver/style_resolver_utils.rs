//! Translation of Chromium resolver/style_resolver_utils.h.
//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! style_resolver_utils.h: 55 / 15 / 14 / 1 / 0.
//! style_resolver_utils.cc does not exist in this source checkout (not counted).
//! Effective excludes copyright/comments, blank/preprocessor/include/namespace
//! and lines containing only brackets/punctuation. Only forward declaration 17
//! is omitted. The FunctionRuleMap alias and all template behavior at 19-49 map:
//! first insertion, layer comparison and later-entry replacement on equal order.
//! Actual StyleRuleFunction, CascadeLayered and CascadeLayerMap are reused; the
//! name getter trait exposes actual rule owners without a second rule model.
#![allow(non_snake_case)]
use crate::cascade_layer_map::CascadeLayerMap;
use crate::cascade_layered::CascadeLayered;
use crate::style_rule::{StyleRuleDependencies, StyleRuleFunction};
use foundation::AtomicString;
use std::cmp::Ordering;
use std::collections::{hash_map::Entry, HashMap};
use std::rc::Rc;

pub type FunctionRuleMap<D> = HashMap<AtomicString, CascadeLayered<Rc<StyleRuleFunction<D>>>>;

/// The source template only requires a name getter from the actual rule owner.
/// No rule data or matching model is defined here.
pub trait NameDefiningRule {
    fn Name(&self) -> &AtomicString;
}
impl<D: StyleRuleDependencies> NameDefiningRule for StyleRuleFunction<D> {
    fn Name(&self) -> &AtomicString {
        StyleRuleFunction::Name(self)
    }
}

pub fn AddNameDefiningRules<T: NameDefiningRule>(
    input_rules: &[CascadeLayered<Rc<T>>],
    cascade_layer_map: Option<&CascadeLayerMap>,
    out: &mut HashMap<AtomicString, CascadeLayered<Rc<T>>>,
) {
    for rule in input_rules {
        match out.entry(rule.value.Name().clone()) {
            Entry::Vacant(entry) => {
                entry.insert(CascadeLayered::new(rule.value.clone(), rule.layer.clone()));
            }
            Entry::Occupied(mut entry) => {
                let stored_rule = entry.get_mut();
                // CascadeLayerMap's static overload first tests equal identity;
                // differing layers require a map (a production CHECK).
                let order = if stored_rule.layer == rule.layer {
                    Ordering::Equal
                } else {
                    cascade_layer_map
                        .expect("different layers require CascadeLayerMap")
                        .CompareLayerOrder(stored_rule.layer.as_ref(), rule.layer.as_ref())
                };
                if order != Ordering::Greater {
                    *stored_rule = CascadeLayered::new(rule.value.clone(), rule.layer.clone());
                }
            }
        }
    }
}
