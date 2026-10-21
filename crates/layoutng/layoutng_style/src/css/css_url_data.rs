// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Immutable URL identity shared by parsed and computed CSS values.
//! URL resolution and resource loading remain host inputs, never operations here.
#![allow(non_snake_case)]
use foundation::String;

/// KURL-shaped parser input owned by the production stylesheet adapter.
/// `Null` is distinct from an explicitly empty URL, matching Blink's KURL
/// contract. A concrete value is canonicalized by the host URL parser when
/// possible and remains a string only at this package boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionURL {
    Null,
    Value(String),
}

impl ProductionURL {
    pub fn FromString(value: &str) -> Self {
        Self::Value(String::FromUtf8(value.as_bytes()))
    }

    pub fn AsString(&self) -> Option<&String> {
        match self {
            Self::Null => None,
            Self::Value(value) => Some(value),
        }
    }
}

// css_url_data.h:45-110; css_url_data.cc:56-83. Request modifiers and
// dangling-markup state remain pending, but relative and absolute identity are
// retained separately so serialization and resource lookup use Blink's owner.
#[derive(Clone, Debug)]
pub struct CSSUrlData {
    pub unresolved_url: String,
    pub resolved_url: String,
}

impl CSSUrlData {
    pub fn new(unresolved_url: String, resolved_url: ProductionURL) -> Self {
        let resolved_url = resolved_url
            .AsString()
            .cloned()
            .unwrap_or_else(|| unresolved_url.clone());
        Self {
            unresolved_url,
            resolved_url,
        }
    }

    pub fn unresolved(url: String) -> Self {
        Self {
            resolved_url: url.clone(),
            unresolved_url: url,
        }
    }

    // cpp: css_url_data.cc:172-190. Computed values preserve local, empty,
    // and unresolved URLs; otherwise the resolved spelling becomes relative.
    // Request modifiers remain parser-rejected. Dangling-markup/referrer
    // metadata is not represented by the existing typed input contract.
    pub fn MakeComputed(&self) -> Self {
        if self.unresolved_url.empty()
            || self.unresolved_url.starts_with('#')
            || self.resolved_url.empty()
        {
            return self.clone();
        }
        Self {
            unresolved_url: self.resolved_url.clone(),
            resolved_url: self.resolved_url.clone(),
        }
    }

    pub fn ValueForSerialization(&self) -> &String {
        if self.unresolved_url.Utf8().starts_with('#') || self.resolved_url.empty() {
            &self.unresolved_url
        } else {
            &self.resolved_url
        }
    }
}

// cpp: css_url_data.cc:242-258. Authored aliases share non-local resolved
// identity, while fragment-only references keep their local identity.
impl PartialEq for CSSUrlData {
    fn eq(&self, other: &Self) -> bool {
        let local = self.unresolved_url.starts_with('#');
        if local != other.unresolved_url.starts_with('#') {
            return false;
        }
        if local || (self.resolved_url.empty() && other.resolved_url.empty()) {
            self.unresolved_url == other.unresolved_url
        } else {
            self.resolved_url == other.resolved_url
        }
    }
}
impl Eq for CSSUrlData {}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(unresolved: &str, resolved: &str) -> CSSUrlData {
        CSSUrlData::new(
            String::from(unresolved),
            ProductionURL::FromString(resolved),
        )
    }

    #[test]
    fn computed_url_preserves_source_url_semantics() {
        for (unresolved, resolved, computed) in [
            (
                "../a.svg",
                "https://example.test/a.svg",
                "https://example.test/a.svg",
            ),
            ("#icon", "https://example.test/page#icon", "#icon"),
            ("", "https://example.test/page", ""),
            ("invalid-relative", "", "invalid-relative"),
            (
                "data:image/svg+xml,%3Csvg/%3E",
                "data:image/svg+xml,%3Csvg/%3E",
                "data:image/svg+xml,%3Csvg/%3E",
            ),
        ] {
            let source = data(unresolved, resolved);
            let value = source.MakeComputed();
            assert_eq!(value.unresolved_url.as_ref(), computed);
            assert_eq!(value.resolved_url.as_ref(), resolved);
            assert_eq!(
                source.unresolved_url.as_ref(),
                unresolved,
                "source remains immutable"
            );
            assert_eq!(source, value, "computed conversion preserves URL identity");
        }
    }

    #[test]
    fn url_equality_distinguishes_local_references_from_resolved_aliases() {
        assert_eq!(
            data("a.svg", "https://example.test/a.svg"),
            data("./a.svg", "https://example.test/a.svg")
        );
        assert_eq!(
            data("#icon", "https://one.test/page#icon"),
            data("#icon", "https://two.test/page#icon")
        );
        assert_ne!(
            data("#icon", "https://example.test/page#icon"),
            data(
                "https://example.test/page#icon",
                "https://example.test/page#icon"
            )
        );
        assert_ne!(data("first", ""), data("second", ""));
        assert_eq!(data("first", ""), data("first", ""));
    }
}
