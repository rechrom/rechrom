// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_function_declarations.h
// Source physical h55; production logic pending: 0.
// Trace, GC allocation and DowncastTraits are represented by Rust ownership
// and StyleRuleBase dispatch.

#![allow(non_snake_case)]

use crate::css_property_value_set::{
    CSSPropertyValueSetBackend, CSSPropertyValueSetRuleAdapter, CSSPropertyValueSetRuleHandle,
    MutableCSSPropertyValueSet,
};
use crate::style_rule::RuleType;
use std::cell::RefCell;
use std::rc::Rc;

// cpp: style_rule_function_declarations.h:15-45
pub struct StyleRuleFunctionDeclarations<D: CSSPropertyValueSetBackend> {
    properties_: RefCell<Rc<CSSPropertyValueSetRuleHandle<D>>>,
}

impl<D: CSSPropertyValueSetBackend> StyleRuleFunctionDeclarations<D> {
    // cpp: style_rule_function_declarations.h:18-19
    pub fn new(properties: Rc<CSSPropertyValueSetRuleHandle<D>>) -> Self {
        Self {
            properties_: RefCell::new(properties),
        }
    }

    pub fn GetType(&self) -> RuleType {
        RuleType::kFunctionDeclarations
    }

    // cpp: style_rule_function_declarations.h:24
    pub fn Properties(&self) -> Rc<CSSPropertyValueSetRuleHandle<D>> {
        self.properties_.borrow().clone()
    }

    // cpp: style_rule_function_declarations.h:26-31
    pub fn MutableProperties(&self) -> Rc<RefCell<MutableCSSPropertyValueSet<D>>> {
        let current = self.properties_.borrow().clone();
        if !current.IsMutable() {
            *self.properties_.borrow_mut() =
                CSSPropertyValueSetRuleHandle::FromMutable(current.MutableCopy());
        }
        self.properties_
            .borrow()
            .MutablePropertySet()
            .expect("MutableProperties converted the property store")
    }

    // cpp: style_rule_function_declarations.h:33-35
    pub fn Copy(&self) -> Self {
        self.clone()
    }
}

impl<D: CSSPropertyValueSetBackend> Clone for StyleRuleFunctionDeclarations<D> {
    // cpp: style_rule_function_declarations.h:21-22. Unlike most rule copies,
    // this copy explicitly freezes mutable declarations.
    fn clone(&self) -> Self {
        Self::new(self.properties_.borrow().ImmutableCopyIfNeeded())
    }
}
