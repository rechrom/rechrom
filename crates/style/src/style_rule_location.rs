// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_location.h
// cpp: third_party/blink/renderer/core/css/style_rule_location.cc
// Source physical h61 + cc102; production logic pending: 0.
// Oilpan tracing/allocation, exports and DowncastTraits are Rust ownership and
// StyleRuleBase dispatch; all production route-construction logic is mapped.

#![allow(non_snake_case, non_camel_case_types)]

use crate::css_property_names::CSSPropertyID;
use crate::css_property_value_set::{CSSPropertyValueSetBackend, CSSPropertyValueSetRuleAdapter};
use crate::css_value::{CSSValue, CSSValueDispatch};
use crate::style_rule::RuleType;
use foundation::{AtomicString, String};
use std::rc::Rc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct URLPatternInit {
    pub protocol: Option<String>,
    pub hostname: Option<String>,
    pub port: Option<String>,
    pub pathname: Option<String>,
    pub search: Option<String>,
    pub hash: Option<String>,
    pub base_url: String,
}

// CSSURLPatternValue/CSSStringValue, URLPattern and Document are dependencies
// of this source pair. Required hooks preserve their real typed identities and
// keep URL-pattern parsing in its owner instead of duplicating it here.
pub trait StyleRuleLocationBackend: CSSPropertyValueSetBackend {
    type URLPattern;

    fn ToURLPatternValue(
        value: Option<Rc<CSSValue<Self>>>,
    ) -> Option<Rc<<Self as CSSValueDispatch>::CSSURLPatternValue>>;
    fn ToStringValue(
        value: Option<Rc<CSSValue<Self>>>,
    ) -> Option<Rc<<Self as CSSValueDispatch>::CSSStringValue>>;
    fn URLString(value: &<Self as CSSValueDispatch>::CSSURLPatternValue) -> AtomicString;
    fn StringValue(value: &<Self as CSSValueDispatch>::CSSStringValue) -> String;

    fn DocumentBaseURL(document: &<Self as CSSValueDispatch>::Document) -> String;
    fn CreateURLPatternFromString(
        document: &<Self as CSSValueDispatch>::Document,
        pattern: &AtomicString,
    ) -> Option<Rc<Self::URLPattern>>;
    fn CreateURLPatternFromInit(
        document: &<Self as CSSValueDispatch>::Document,
        init: URLPatternInit,
    ) -> Option<Rc<Self::URLPattern>>;
    fn AddURLPatternFromLocation(
        document: &<Self as CSSValueDispatch>::Document,
        name: &AtomicString,
        pattern: Rc<Self::URLPattern>,
    );
}

// cpp: style_rule_location.h:20-55
pub struct StyleRuleLocation<D: StyleRuleLocationBackend> {
    name_: AtomicString,
    pattern_: Option<Rc<<D as CSSValueDispatch>::CSSURLPatternValue>>,
    protocol_: Option<Rc<<D as CSSValueDispatch>::CSSStringValue>>,
    hostname_: Option<Rc<<D as CSSValueDispatch>::CSSStringValue>>,
    port_: Option<Rc<<D as CSSValueDispatch>::CSSStringValue>>,
    pathname_: Option<Rc<<D as CSSValueDispatch>::CSSStringValue>>,
    search_: Option<Rc<<D as CSSValueDispatch>::CSSStringValue>>,
    hash_: Option<Rc<<D as CSSValueDispatch>::CSSStringValue>>,
    base_url_: Option<Rc<<D as CSSValueDispatch>::CSSStringValue>>,
}

impl<D: StyleRuleLocationBackend> Clone for StyleRuleLocation<D> {
    fn clone(&self) -> Self {
        Self {
            name_: self.name_.clone(),
            pattern_: self.pattern_.clone(),
            protocol_: self.protocol_.clone(),
            hostname_: self.hostname_.clone(),
            port_: self.port_.clone(),
            pathname_: self.pathname_.clone(),
            search_: self.search_.clone(),
            hash_: self.hash_.clone(),
            base_url_: self.base_url_.clone(),
        }
    }
}

impl<D: StyleRuleLocationBackend> StyleRuleLocation<D> {
    // cpp: style_rule_location.cc:21-45
    pub fn new<V>(name: AtomicString, values: &V) -> Self
    where
        V: CSSPropertyValueSetRuleAdapter<CSSValue = CSSValue<D>>,
    {
        debug_assert!(name.length() >= 2 && name.at(0) == b'-' as u16 && name.at(1) == b'-' as u16);
        let get = |id| values.GetPropertyCSSValue(id);
        Self {
            name_: name,
            pattern_: D::ToURLPatternValue(get(CSSPropertyID::kPattern)),
            protocol_: D::ToStringValue(get(CSSPropertyID::kProtocol)),
            hostname_: D::ToStringValue(get(CSSPropertyID::kHostname)),
            port_: D::ToStringValue(get(CSSPropertyID::kPort)),
            pathname_: D::ToStringValue(get(CSSPropertyID::kPathname)),
            search_: D::ToStringValue(get(CSSPropertyID::kSearch)),
            hash_: D::ToStringValue(get(CSSPropertyID::kHash)),
            base_url_: D::ToStringValue(get(CSSPropertyID::kBaseUrl)),
        }
    }

    pub fn GetType(&self) -> RuleType {
        RuleType::kLocation
    }
    pub fn GetName(&self) -> &AtomicString {
        &self.name_
    }
    pub fn GetPattern(&self) -> Option<&<D as CSSValueDispatch>::CSSURLPatternValue> {
        self.pattern_.as_deref()
    }
    pub fn GetProtocol(&self) -> Option<&<D as CSSValueDispatch>::CSSStringValue> {
        self.protocol_.as_deref()
    }
    pub fn GetHostname(&self) -> Option<&<D as CSSValueDispatch>::CSSStringValue> {
        self.hostname_.as_deref()
    }
    pub fn GetPort(&self) -> Option<&<D as CSSValueDispatch>::CSSStringValue> {
        self.port_.as_deref()
    }
    pub fn GetPathname(&self) -> Option<&<D as CSSValueDispatch>::CSSStringValue> {
        self.pathname_.as_deref()
    }
    pub fn GetSearch(&self) -> Option<&<D as CSSValueDispatch>::CSSStringValue> {
        self.search_.as_deref()
    }
    pub fn GetHash(&self) -> Option<&<D as CSSValueDispatch>::CSSStringValue> {
        self.hash_.as_deref()
    }
    pub fn GetBaseUrl(&self) -> Option<&<D as CSSValueDispatch>::CSSStringValue> {
        self.base_url_.as_deref()
    }

    // cpp: style_rule_location.cc:60-98
    pub fn CreateRouteIfNeeded(&self, document: Option<&<D as CSSValueDispatch>::Document>) {
        let Some(document) = document else {
            return;
        };
        let url_pattern = if let Some(pattern) = &self.pattern_ {
            D::CreateURLPatternFromString(document, &D::URLString(pattern))
        } else {
            let value = |source: &Option<Rc<<D as CSSValueDispatch>::CSSStringValue>>| {
                source.as_deref().map(D::StringValue)
            };
            D::CreateURLPatternFromInit(
                document,
                URLPatternInit {
                    protocol: value(&self.protocol_),
                    hostname: value(&self.hostname_),
                    port: value(&self.port_),
                    pathname: value(&self.pathname_),
                    search: value(&self.search_),
                    hash: value(&self.hash_),
                    base_url: value(&self.base_url_)
                        .unwrap_or_else(|| D::DocumentBaseURL(document)),
                },
            )
        };
        if let Some(url_pattern) = url_pattern {
            D::AddURLPatternFromLocation(document, &self.name_, url_pattern);
        }
    }
}
