// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/parser/css_nesting_type.h:12-33

#![allow(non_camel_case_types)]

// Note that order matters: kNesting effectively also means kScope,
// and therefore it's convenient to compute the max CSSNestingType
// in some cases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CSSNestingType {
    // We are not in a nesting context, and '&' resolves like :scope instead.
    kNone,
    // We are in a nesting context as defined by @scope.
    //
    // https://drafts.csswg.org/css-cascade-6/#scope-atrule
    // https://drafts.csswg.org/selectors-4/#scope-pseudo
    kScope,
    // We are in a css-nesting nesting context, and '&' resolves according to:
    // https://drafts.csswg.org/css-nesting-1/#nest-selector
    kNesting,
    // We are inside @function. The parsing behavior is generally the same as
    // kNesting, except we don't allow qualified rules, and we emit
    // CSSFunctionDeclarations instead of CSSNestedDeclarations.
    kFunction,
    // We are inside @mixin. The parsing behavior is generally the same as
    // kFunction; we only use it to track the RuleType through conditional rules.
    kMixin,
}
