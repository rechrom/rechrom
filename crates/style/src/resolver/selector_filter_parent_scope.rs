// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// cpp: third_party/blink/renderer/core/css/resolver/selector_filter_parent_scope.h:26
// Complete mapping of SelectorFilterParentScope::ScopeType. The enclosing
// scope requires genuine Element, Document, StyleResolver, and SelectorFilter
// lifetimes and is not represented by a substitute type here.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ScopeType {
    kParent,
    kRoot,
}
