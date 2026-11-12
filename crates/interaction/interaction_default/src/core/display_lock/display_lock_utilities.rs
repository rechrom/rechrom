// Copyright 2026 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source: core/display_lock/display_lock_utilities.cc:730-780.
use std::{collections::HashSet, hash::Hash};

pub trait SelectionChangedContext {
    type Node: Copy + Eq + Hash;
    type Context: Copy + Eq + Hash;
    fn InclusiveAncestorContexts(&self, node: Self::Node) -> Vec<Self::Context>;
}

/// Source temporary sets, returned to the caller solely to release its live
/// DOM read before notifying native contexts. No range/tree is retained.
pub struct SelectionChangedContexts<C> {
    pub lost: HashSet<C>,
    pub gained: HashSet<C>,
}
pub fn SelectionChanged<C: SelectionChangedContext>(
    context: &C,
    old_nodes: impl Iterator<Item = C::Node>,
    new_nodes: impl Iterator<Item = C::Node>,
) -> SelectionChangedContexts<C::Context> {
    let mut new_nodes: HashSet<_> = new_nodes.collect();
    let mut lost = HashSet::new();
    let mut gained = HashSet::new();
    for node in old_nodes {
        if new_nodes.remove(&node) {
            continue;
        }
        lost.extend(context.InclusiveAncestorContexts(node));
    }
    for node in new_nodes {
        gained.extend(context.InclusiveAncestorContexts(node));
    }
    lost.retain(|context| !gained.remove(context));
    SelectionChangedContexts { lost, gained }
}
