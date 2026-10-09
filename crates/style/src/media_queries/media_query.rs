/*
 * CSS Media Query
 *
 * Copyright (C) 2005, 2006 Kimmo Kinnunen <kimmo.t.kinnunen@nokia.com>.
 * Copyright (C) 2010 Nokia Corporation and/or its subsidiary(-ies).
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Ledger uses C++ physical / effective / mapped / omitted / pending lines.
// Effective excludes comments, blanks, preprocessor/includes, namespaces and
// lines consisting only of braces/parentheses/semicolons.
// media_query.h: 86 / 27 / 20 / 7 / 0.
// media_query.cc: 147 / 65 / 60 / 5 / 0.
// Header omitted: 45-46,49,56-58,74 (forward/access/deleted/dtor/GC).
// Implementation omitted: 94-98,103,107 (default dtor/GC/access).
// Source ledger (third_party/blink/renderer/core/css/):
// - media_query.h:48-82; media_query.cc:42-94,100-145: complete query runtime.
// Omitted: .cc:115-117 (GC Trace); static-string interning uses String storage.
// Dependencies: translated ConditionalExpNode and CSS identifier serialization.
#![allow(non_snake_case, non_camel_case_types)]
use super::conditional_exp_node::{ConditionalExpNode, ConditionalExpNodeVisitor};
use super::media_query_exp::{MediaQueryExp, MediaQueryExpSerialization};
use crate::kleene_value::KleeneValue;
use foundation::String;
use std::cell::RefCell;
use std::rc::Rc;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestrictorType {
    kOnly,
    kNot,
    kNone,
}

pub struct MediaQuery<V, U = V> {
    media_type_: String,
    serialization_cache_: RefCell<Option<String>>,
    exp_node_: Option<Rc<ConditionalExpNode<V, U>>>,
    restrictor_: RestrictorType,
}
impl<V, U> Clone for MediaQuery<V, U> {
    // cpp: media_query.cc:88-92
    fn clone(&self) -> Self {
        Self {
            media_type_: self.media_type_.clone(),
            serialization_cache_: RefCell::new(self.serialization_cache_.borrow().clone()),
            exp_node_: self.exp_node_.clone(),
            restrictor_: self.restrictor_,
        }
    }
}
impl<V, U> MediaQuery<V, U> {
    // cpp: media_query.cc:76-86
    pub fn CreateNotAll() -> Self {
        Self::new(RestrictorType::kNot, String::from("all"), None)
    }
    pub fn new(
        restrictor: RestrictorType,
        media_type: String,
        exp_node: Option<Rc<ConditionalExpNode<V, U>>>,
    ) -> Self {
        // ToAsciiLower operates on ASCII code units only, including lone surrogates.
        let units: Vec<u16> = (0..media_type.length())
            .map(|i| {
                let c = media_type.CodeUnitAt(i);
                if (65..=90).contains(&c) {
                    c + 32
                } else {
                    c
                }
            })
            .collect();
        Self {
            media_type_: if media_type.IsNull() {
                String::default()
            } else {
                String::from_utf16(&units)
            },
            serialization_cache_: RefCell::new(None),
            exp_node_: exp_node,
            restrictor_: restrictor,
        }
    }
    // cpp: media_query.cc:121-131
    pub fn Restrictor(&self) -> RestrictorType {
        self.restrictor_
    }
    pub fn ExpNode(&self) -> Option<&ConditionalExpNode<V, U>> {
        self.exp_node_.as_deref()
    }
    pub fn MediaType(&self) -> &String {
        &self.media_type_
    }
    // cpp: media_query.cc:100-119; media_query.h:60-66
    pub fn CollectExpressionsFrom(
        root: &ConditionalExpNode<V, U>,
        expressions: &mut Vec<MediaQueryExp<V, U>>,
    ) {
        struct Collector<'a, V, U>(&'a mut Vec<MediaQueryExp<V, U>>);
        impl<V, U> ConditionalExpNodeVisitor<V, U> for Collector<'_, V, U> {
            fn EvaluateMediaQueryFeatureExpNode(
                &mut self,
                exp: &MediaQueryExp<V, U>,
            ) -> KleeneValue {
                self.0.push(exp.clone());
                KleeneValue::kUnknown
            }
        }
        root.Evaluate(&mut Collector(expressions));
    }
    pub fn CollectExpressions(&self, expressions: &mut Vec<MediaQueryExp<V, U>>) {
        if let Some(node) = self.ExpNode() {
            Self::CollectExpressionsFrom(node, expressions);
        }
    }
}
impl<V: MediaQueryExpSerialization, U: MediaQueryExpSerialization> MediaQuery<V, U> {
    // cpp: media_query.cc:42-74
    fn Serialize(&self) -> String {
        let mut result = String::from(match self.Restrictor() {
            RestrictorType::kOnly => "only ",
            RestrictorType::kNot => "not ",
            RestrictorType::kNone => "",
        });
        let Some(node) = self.ExpNode() else {
            result.push_string(&crate::css_markup::SerializeIdentifier(
                self.MediaType(),
                false,
            ));
            return result;
        };
        if self.MediaType() != &String::from("all") || self.Restrictor() != RestrictorType::kNone {
            result.push_string(&crate::css_markup::SerializeIdentifier(
                self.MediaType(),
                false,
            ));
            result.push_str(" and ");
        }
        result.push_string(&node.Serialize());
        result
    }
    // cpp: media_query.cc:139-145
    pub fn CssText(&self) -> String {
        if self.serialization_cache_.borrow().is_none() {
            *self.serialization_cache_.borrow_mut() = Some(self.Serialize());
        }
        self.serialization_cache_.borrow().as_ref().unwrap().clone()
    }
}
impl<V: MediaQueryExpSerialization, U: MediaQueryExpSerialization> PartialEq for MediaQuery<V, U> {
    // cpp: media_query.cc:134-136
    fn eq(&self, other: &Self) -> bool {
        self.CssText() == other.CssText()
    }
}
