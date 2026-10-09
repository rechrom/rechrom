// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_nested_declarations.h
// Source physical h76; production logic pending: 0.
// Trace/Oilpan/downcast boilerplate is represented by Rust ownership and
// StyleRuleBase dispatch.

#![allow(non_snake_case)]

use crate::parser::css_nesting_type::CSSNestingType;
use crate::style_rule::{RuleType, StyleRule, StyleRuleDependencies};
use std::rc::Rc;

// cpp: style_rule_nested_declarations.h:27-74
// This wrapper intentionally owns its non-observable inner StyleRule. The
// latter is what RuleSet receives; the wrapper only preserves whether it was
// produced by regular nesting or @scope for later re-nesting.
pub struct StyleRuleNestedDeclarations<D: StyleRuleDependencies> {
    nesting_type_: CSSNestingType,
    style_rule_: StyleRule<D>,
}

impl<D: StyleRuleDependencies> StyleRuleNestedDeclarations<D> {
    // cpp: style_rule_nested_declarations.h:30-35
    pub fn new(nesting_type: CSSNestingType, style_rule: StyleRule<D>) -> Self {
        Self {
            nesting_type_: nesting_type,
            style_rule_: style_rule,
        }
    }

    pub fn GetType(&self) -> RuleType {
        RuleType::kNestedDeclarations
    }
    pub fn NestingType(&self) -> CSSNestingType {
        self.nesting_type_
    }
    pub fn InnerStyleRule(&self) -> &StyleRule<D> {
        &self.style_rule_
    }

    // cpp: style_rule_nested_declarations.h:41-48
    pub fn Properties(&self) -> Rc<D::CSSPropertyValueSet> {
        self.style_rule_.Properties()
    }
    pub fn MutableProperties(&mut self) -> Rc<D::CSSPropertyValueSet> {
        self.style_rule_.MutableProperties()
    }

    /// Consumes the wrapper when the caller needs the same allocation-like
    /// ownership transfer used by StyleRuleBase::Clone.
    pub fn IntoInnerStyleRule(self) -> StyleRule<D> {
        self.style_rule_
    }
}
