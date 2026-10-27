// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_namespace.h:13-37
// The omitted lines are include guards, GC tracing and downcast boilerplate.

#![allow(non_snake_case)]

use foundation::AtomicString;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleRuleNamespace {
    prefix_: AtomicString,
    uri_: AtomicString,
}
impl StyleRuleNamespace {
    pub fn new(prefix: AtomicString, uri: AtomicString) -> Self {
        Self {
            prefix_: prefix,
            uri_: uri,
        }
    }
    pub fn Copy(&self) -> Self {
        Self::new(self.prefix_.clone(), self.uri_.clone())
    }
    pub fn Prefix(&self) -> AtomicString {
        self.prefix_.clone()
    }
    pub fn Uri(&self) -> AtomicString {
        self.uri_.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn copy_preserves_namespace_values_without_aliasing_storage_contracts() {
        let rule = StyleRuleNamespace::new(
            AtomicString::from_str("svg"),
            AtomicString::from_str("http://www.w3.org/2000/svg"),
        );
        let copy = rule.Copy();
        assert_eq!(copy.Prefix(), AtomicString::from_str("svg"));
        assert_eq!(
            copy.Uri(),
            AtomicString::from_str("http://www.w3.org/2000/svg")
        );
    }
}
