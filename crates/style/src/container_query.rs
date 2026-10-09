// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: container_query.h:20-37/.cc:15-40; container_query_set.h:20-45/.cc:14-35.
#![allow(non_snake_case)]
use crate::media_queries::{
    conditional_exp_node::ConditionalExpNode, media_query_exp::MediaQueryExpSerialization,
};
use foundation::{AtomicString, String};
use std::rc::Rc;
/// The parsed selector name and immutable conditional tree retained by a rule.
/// Container selection feature requirements are read from this same tree.
pub struct ContainerQuery<V, U> {
    name: AtomicString,
    query: Option<Rc<ConditionalExpNode<V, U>>>,
}
impl<V, U> Clone for ContainerQuery<V, U> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            query: self.query.clone(),
        }
    }
}
impl<V, U> ContainerQuery<V, U> {
    pub fn new(name: AtomicString, query: Option<Rc<ConditionalExpNode<V, U>>>) -> Self {
        Self { name, query }
    }
    pub fn SelectorName(&self) -> AtomicString {
        self.name.clone()
    }
    pub fn Query(&self) -> Option<&ConditionalExpNode<V, U>> {
        self.query.as_deref()
    }
}
impl<V: MediaQueryExpSerialization, U: MediaQueryExpSerialization> ContainerQuery<V, U> {
    pub fn ToString(&self) -> String {
        let mut result = String::from("");
        if !self.name.IsNull() && !self.name.Utf8().is_empty() {
            result.push_string(&crate::css_markup::SerializeIdentifier(
                &String::from_utf16(self.name.utf16_units().unwrap_or_default()),
                false,
            ));
            if self.query.is_some() {
                result.push_str(" ");
            }
        }
        if let Some(query) = &self.query {
            result.push_string(&query.Serialize());
        }
        result
    }
}
pub struct ContainerQuerySet<V, U> {
    queries: Vec<Rc<ContainerQuery<V, U>>>,
    parent: Option<Rc<Self>>,
}
impl<V, U> Clone for ContainerQuerySet<V, U> {
    fn clone(&self) -> Self {
        Self {
            queries: self.queries.clone(),
            parent: self.parent.clone(),
        }
    }
}
impl<V, U> ContainerQuerySet<V, U> {
    pub fn new(queries: Vec<Rc<ContainerQuery<V, U>>>) -> Self {
        Self {
            queries,
            parent: None,
        }
    }
    pub fn Queries(&self) -> &[Rc<ContainerQuery<V, U>>] {
        &self.queries
    }
    pub fn SingleQuery(&self) -> Option<&ContainerQuery<V, U>> {
        if self.queries.len() == 1 {
            self.queries.first().map(Rc::as_ref)
        } else {
            None
        }
    }
    pub fn Parent(&self) -> Option<&Self> {
        self.parent.as_deref()
    }
    pub fn CopyWithParent(&self, parent: Option<Rc<Self>>) -> Rc<Self> {
        let mut copy = self.clone();
        copy.parent = parent;
        Rc::new(copy)
    }
}
impl<V: MediaQueryExpSerialization, U: MediaQueryExpSerialization> ContainerQuerySet<V, U> {
    pub fn ToString(&self) -> String {
        let mut result = String::from("");
        for (index, query) in self.queries.iter().enumerate() {
            if index > 0 {
                result.push_str(", ");
            }
            result.push_string(&query.ToString());
        }
        result
    }
}
