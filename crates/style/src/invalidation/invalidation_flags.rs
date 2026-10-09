// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/invalidation/invalidation_flags.h:10-59
// cpp: third_party/blink/renderer/core/css/invalidation/invalidation_flags.cc:9-27

/// Flags passed through multiple layers of the style invalidation process.
///
/// C++ bit-field storage becomes Rust booleans; defaults, field visibility,
/// equality, accessors and merge behavior follow the source.
// cpp: invalidation_flags.cc:19-27 — derived equality compares all seven fields.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InvalidationFlags {
    // cpp: invalidation_flags.h:41-58
    // All descendants which are custom pseudo-elements must be invalidated.
    invalidate_custom_pseudo_: bool,
    // All descendants might be invalidated; a full subtree recalc is required.
    whole_subtree_invalid_: bool,
    // The invalidation must traverse into ShadowRoots with this set.
    tree_boundary_crossing_: bool,
    // Insertion point descendants must be invalidated.
    insertion_point_crossing_: bool,
    // Distributed nodes of <slot> elements need to be invalidated.
    invalidates_slotted_: bool,
    // Parts inside this node's shadow tree need to be invalidated.
    invalidates_parts_: bool,
    // Elements whose style depends on tree-counting functions are invalidated.
    invalidates_tree_counting_: bool,
}

#[allow(non_snake_case)]
impl InvalidationFlags {
    /// Merges two flag sets by ORing every field.
    // cpp: invalidation_flags.cc:9-17
    pub fn Merge(&mut self, other: &Self) {
        self.invalidate_custom_pseudo_ |= other.invalidate_custom_pseudo_;
        self.tree_boundary_crossing_ |= other.tree_boundary_crossing_;
        self.insertion_point_crossing_ |= other.insertion_point_crossing_;
        self.whole_subtree_invalid_ |= other.whole_subtree_invalid_;
        self.invalidates_slotted_ |= other.invalidates_slotted_;
        self.invalidates_parts_ |= other.invalidates_parts_;
        self.invalidates_tree_counting_ |= other.invalidates_tree_counting_;
    }

    // cpp: invalidation_flags.h:19-20
    pub fn WholeSubtreeInvalid(&self) -> bool {
        self.whole_subtree_invalid_
    }
    pub fn SetWholeSubtreeInvalid(&mut self, value: bool) {
        self.whole_subtree_invalid_ = value;
    }

    // cpp: invalidation_flags.h:22-23
    pub fn TreeBoundaryCrossing(&self) -> bool {
        self.tree_boundary_crossing_
    }
    pub fn SetTreeBoundaryCrossing(&mut self, value: bool) {
        self.tree_boundary_crossing_ = value;
    }

    // cpp: invalidation_flags.h:25-28
    pub fn InsertionPointCrossing(&self) -> bool {
        self.insertion_point_crossing_
    }
    pub fn SetInsertionPointCrossing(&mut self, value: bool) {
        self.insertion_point_crossing_ = value;
    }

    // cpp: invalidation_flags.h:30-31
    pub fn InvalidatesSlotted(&self) -> bool {
        self.invalidates_slotted_
    }
    pub fn SetInvalidatesSlotted(&mut self, value: bool) {
        self.invalidates_slotted_ = value;
    }

    // cpp: invalidation_flags.h:33-34
    pub fn InvalidatesParts(&self) -> bool {
        self.invalidates_parts_
    }
    pub fn SetInvalidatesParts(&mut self, value: bool) {
        self.invalidates_parts_ = value;
    }

    // cpp: invalidation_flags.h:36-39
    pub fn InvalidatesTreeCounting(&self) -> bool {
        self.invalidates_tree_counting_
    }
    pub fn SetInvalidatesTreeCounting(&mut self, value: bool) {
        self.invalidates_tree_counting_ = value;
    }
}
