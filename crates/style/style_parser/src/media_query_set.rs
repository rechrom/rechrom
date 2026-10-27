/*
 * (C) 1999-2003 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2004, 2006, 2010, 2012 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Ledger uses C++ physical / effective / mapped / omitted / pending lines.
// Effective excludes comments, blanks, preprocessor/includes, namespaces and
// lines consisting only of braces/parentheses/semicolons.
// MediaQuerySet class scope only; MediaList DOM wrapper is outside this batch.
// media_list.h scope 45-68: physical file=112; scoped effective=16,
// mapped=13, omitted=3 (46,55,66 access/GC), pending=0.
// media_list.cc scope 60-161: physical file=258; scoped effective=55,
// mapped=53, omitted=2 (76-78 GC), pending=0.
// Source ledger (third_party/blink/renderer/core/css/):
// MediaQuerySet lives in media_list.h/.cc in the current Chromium checkout.
// - media_list.h:45-68; media_list.cc:60-74,80-161: complete set runtime.
// Omitted: media_list.cc:76-78 GC Trace; MediaList DOM wrapper .cc:163-end.
// Required parsing dependency: MediaQuerySetParser supplied by media query parser.
#![allow(non_snake_case)]
use super::media_query::MediaQuery;
use super::media_query_exp::MediaQueryExpSerialization;
use foundation::String;
use std::rc::Rc;

pub trait MediaQuerySetParser<V, U = V> {
    fn ParseMediaQuerySet(&mut self, text: &String) -> MediaQuerySet<V, U>;
}
pub struct MediaQuerySet<V, U = V> {
    queries_: Vec<Rc<MediaQuery<V, U>>>,
}
impl<V, U> Default for MediaQuerySet<V, U> {
    fn default() -> Self {
        Self::FromQueries(Vec::new())
    }
}
impl<V, U> Clone for MediaQuerySet<V, U> {
    fn clone(&self) -> Self {
        Self::FromQueries(self.queries_.clone())
    }
}
impl<V, U> MediaQuerySet<V, U> {
    // cpp: media_list.cc:60-74; media_list.h:50-52,65-67
    pub fn Create() -> Self {
        Self::default()
    }
    pub fn FromQueries(queries: Vec<Rc<MediaQuery<V, U>>>) -> Self {
        Self { queries_: queries }
    }
    pub fn CreateFromString(text: &String, parser: &mut impl MediaQuerySetParser<V, U>) -> Self {
        if text.empty() {
            Self::Create()
        } else {
            parser.ParseMediaQuerySet(text)
        }
    }
    pub fn QueryVector(&self) -> &[Rc<MediaQuery<V, U>>] {
        &self.queries_
    }
}
impl<V: MediaQueryExpSerialization, U: MediaQueryExpSerialization> MediaQuerySet<V, U> {
    // cpp: media_list.cc:80-109
    pub fn CopyAndAdd(
        &self,
        text: &String,
        parser: &mut impl MediaQuerySetParser<V, U>,
    ) -> Option<Self> {
        let result = Self::CreateFromString(text, parser);
        if result.queries_.len() != 1 {
            return None;
        }
        let new_query = &result.queries_[0];
        if self.queries_.iter().any(|query| **query == **new_query) {
            return None;
        }
        let mut queries = self.queries_.clone();
        queries.push(new_query.clone());
        Some(Self::FromQueries(queries))
    }
    // cpp: media_list.cc:111-146. Rc preserves the source's return-this identity.
    pub fn CopyAndRemove(
        self: &Rc<Self>,
        text: &String,
        parser: &mut impl MediaQuerySetParser<V, U>,
    ) -> Option<Rc<Self>> {
        let result = Self::CreateFromString(text, parser);
        if result.queries_.len() != 1 {
            return Some(self.clone());
        }
        let new_query = &result.queries_[0];
        let queries: Vec<_> = self
            .queries_
            .iter()
            .filter(|query| ***query != **new_query)
            .cloned()
            .collect();
        if queries.len() == self.queries_.len() {
            return None;
        }
        Some(Rc::new(Self::FromQueries(queries)))
    }
    // cpp: media_list.cc:148-161
    pub fn MediaText(&self) -> String {
        let mut text = String::from("");
        for (i, query) in self.queries_.iter().enumerate() {
            if i != 0 {
                text.push_str(", ");
            }
            text.push_string(&query.CssText());
        }
        text
    }
}
