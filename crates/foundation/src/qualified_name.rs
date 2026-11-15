// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source: platform/wtf/text/atomic_string.h; core/dom/qualified_name.h.
use crate::AtomicString;

// QualifiedName's three actual atom components. Selector storage never reduces
// a name to a local-name string: prefix/null/namespace are retained separately.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    prefix_: AtomicString,
    local_name_: AtomicString,
    namespace_uri_: AtomicString,
}
impl QualifiedName {
    pub fn new(
        prefix: AtomicString,
        local_name: AtomicString,
        namespace_uri: AtomicString,
    ) -> Self {
        Self {
            prefix_: prefix,
            local_name_: local_name,
            namespace_uri_: namespace_uri,
        }
    }
    pub fn Prefix(&self) -> &AtomicString {
        &self.prefix_
    }
    pub fn LocalName(&self) -> &AtomicString {
        &self.local_name_
    }
    pub fn NamespaceURI(&self) -> &AtomicString {
        &self.namespace_uri_
    }
    pub fn AnyQName() -> Self {
        Self::new(
            AtomicString::default(),
            AtomicString::default(),
            AtomicString::from_str("*"),
        )
    }
}
