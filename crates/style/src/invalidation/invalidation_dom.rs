// Copyright 2026 The Chromium Authors and other contributors.
// Use of this source code is governed by a BSD-style license.
// Required operations from Blink Node/Element/Document/ShadowRoot/HTMLSlotElement.
// The translated invalidation algorithms own every scheduling/traversal decision.
// This adapter contains no default operations and does not implement policy.

use super::invalidation_set::InvalidationElement;
use std::hash::Hash;

/// Blink StyleChangeType ordering used by the invalidation scheduler.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum StyleChangeType {
    #[default]
    NoStyleChange,
    InlineIndependentStyleChange,
    LocalStyleChange,
    SubtreeStyleChange,
}

/// Assembly supplies actual DOM state. Node handles must retain stable identity
/// for the lifetime of a pending queue; `element` exposes actual matching data.
pub trait InvalidationDom {
    type Node: Clone + Eq + Hash;
    type Element: InvalidationElement;

    fn element(&self, node: &Self::Node) -> Option<&Self::Element>;
    fn is_document(&self, node: &Self::Node) -> bool;
    fn shadow_host(&self, node: &Self::Node) -> Option<Self::Node>;
    fn parent_node(&self, node: &Self::Node) -> Option<Self::Node>;
    fn parent_or_shadow_host_node(&self, node: &Self::Node) -> Option<Self::Node>;
    fn next_sibling(&self, node: &Self::Node) -> Option<Self::Node>;
    fn next_element_sibling(&self, element: &Self::Node) -> Option<Self::Node>;
    fn element_children(&self, node: &Self::Node) -> Vec<Self::Node>;
    fn shadow_root(&self, element: &Self::Node) -> Option<Self::Node>;
    fn document_element(&self, document: &Self::Node) -> Option<Self::Node>;
    fn style_change_type(&self, node: &Self::Node) -> StyleChangeType;
    fn set_needs_style_recalc(&mut self, node: &Self::Node, change: StyleChangeType);
    fn needs_style_recalc(&self, node: &Self::Node) -> bool;
    fn needs_style_invalidation(&self, node: &Self::Node) -> bool;
    fn child_needs_style_invalidation(&self, node: &Self::Node) -> bool;
    fn set_needs_style_invalidation(&mut self, node: &Self::Node);
    fn clear_needs_style_invalidation(&mut self, node: &Self::Node);
    fn clear_child_needs_style_invalidation(&mut self, node: &Self::Node);
    fn is_html_slot(&self, element: &Self::Node) -> bool;
    fn flattened_assigned_nodes(&self, slot: &Self::Node) -> Vec<Self::Node>;
    // Calls the mapped StyleEngine helper. Its owner collects and schedules
    // nth sets; PendingInvalidations does not duplicate that owner's algorithm.
    fn possibly_schedule_nth_pseudo_invalidations(&mut self, node: &Self::Node);
}
