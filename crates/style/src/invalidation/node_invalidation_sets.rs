// Copyright 2015 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Source ledger: comments and blank lines stripped; braces retained.
// Effective = mapped + omitted + production pending. All non-omitted
// effective source lines are mapped; omissions are listed exactly below.
// Omitted: includes/guards/access/forward/cast/allocation scaffolding,
// deleted copy/move/default destructors, GC Trace and inspector tracing.
// node_invalidation_sets.h: physical 33, effective 22, mapped 9,
// omitted 13, production pending 0.
// node_invalidation_sets.h omitted effective lines:5-6,8-9,11,14,16-19,26,31,33
// cpp: third_party/blink/renderer/core/css/invalidation/node_invalidation_sets.h
// Rust move ownership replaces deleted copying and default move operations.

use super::invalidation_set::{InvalidationSetRef, InvalidationSetVector};
use std::rc::Rc;

#[derive(Default)]
pub struct NodeInvalidationSets {
    descendants_: InvalidationSetVector,
    siblings_: InvalidationSetVector,
}

impl NodeInvalidationSets {
    pub fn Descendants(&self) -> &InvalidationSetVector {
        &self.descendants_
    }
    pub fn DescendantsMut(&mut self) -> &mut InvalidationSetVector {
        &mut self.descendants_
    }
    pub fn Siblings(&self) -> &InvalidationSetVector {
        &self.siblings_
    }
    pub fn SiblingsMut(&mut self) -> &mut InvalidationSetVector {
        &mut self.siblings_
    }
}

// C++ Vector<scoped_refptr<T>>::Contains compares pointer identity, not the
// InvalidationSet::operator== value comparison.
pub(crate) fn Contains(sets: &InvalidationSetVector, set: &InvalidationSetRef) -> bool {
    sets.iter().any(|existing| Rc::ptr_eq(existing, set))
}
