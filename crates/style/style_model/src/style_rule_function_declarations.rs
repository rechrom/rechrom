// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_function_declarations.h
// Chromium commit 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// style_rule_function_declarations.h: 55 / 23 / 15 / 8 / 0.
// The eight omitted effective lines are the C++ forward declaration plus
// Trace/GC and DowncastTraits machinery represented by Rust ownership and
// StyleRuleBase enum dispatch.

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
