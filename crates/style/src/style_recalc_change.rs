// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/style_recalc_change.h
// cpp: third_party/blink/renderer/core/css/style_recalc_change.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   style_recalc_change.h: 255 / 173 / 152 / 21 / 0.
//   style_recalc_change.cc: 200 / 162 / 101 / 61 / 0.
// Effective lines exclude comments and blanks; scaffolding is explicitly
// omitted below. Every production declaration/statement is mapped.
// h mapped: 25-26,29,32,35,38,41,44,47,50,53,55,58,63-65,67-68,70-71,73-74,76-77,79-81,84,86,88,92,94,96-97,102,104,106-177,179-180,182-188,193-199,208,217-219,224-225,227-245,248,250-251.
// h omitted: 5-6,8-11,13,15-18,23-24,83,99-101,221,223,253,255.
// cc mapped: 14-17,19-22,24-29,31,33-40,44-54,56-67,69-95,153-157,159,163-164,169-173,179-183,185-186,188-189,193-198.
// cc omitted: 5,7-10,12,97-121,123-151,200.
// Omissions: preprocessor/namespace/forward/access/class scaffolding,
// redundant default/copy operators and debug-only ToString.

use layoutng_style::style::computed_style::ComputedStyle;

/// Only foreign DOM ownership and subtype views cross this boundary. Query
/// dependency flags are read directly from the canonical ComputedStyle.
pub trait StyleRecalcChangeDOMBackend {
    type Node;
    type Element;
    type PseudoElement;
    fn NodeElement(node: &Self::Node) -> Option<&Self::Element>;
    fn ElementNode(element: &Self::Element) -> &Self::Node;
    fn PseudoElementNode(element: &Self::PseudoElement) -> &Self::Node;
    fn ElementComputedStyle(element: &Self::Element) -> Option<&ComputedStyle>;
    fn PseudoElementComputedStyle(element: &Self::PseudoElement) -> &ComputedStyle;
    fn NodeNeedsStyleRecalc(node: &Self::Node) -> bool;
    fn NodeChildNeedsStyleRecalc(node: &Self::Node) -> bool;
    fn NodeGetForceReattachLayoutTree(node: &Self::Node) -> bool;
    fn NodeNeedsLayoutSubtreeUpdate(node: &Self::Node) -> bool;
    // ComputedStyle's existing DOM ABI uses its foreign Element type. The DOM
    // owner adapts its element to that ABI without substituting a style model.
    fn CanMatchSizeContainerQueries(style: &ComputedStyle, element: &Self::Element) -> bool;
}

// cpp: style_recalc_change.h:25-81
const SIZE: u16 = 1 << 0;
const DESCENDANT_SIZE: u16 = 1 << 1;
const STYLE_CHILDREN: u16 = 1 << 2;
const STYLE_DESCENDANTS: u16 = 1 << 3;
const SCROLL: u16 = 1 << 4;
const DESCENDANT_SCROLL: u16 = 1 << 5;
const ANCHORED: u16 = 1 << 6;
const DESCENDANT_ANCHORED: u16 = 1 << 7;
const CONTENT_VISIBILITY: u16 = 1 << 8;
const REATTACH: u16 = 1 << 9;
const SUPPRESS: u16 = 1 << 10;
const MARK_REATTACH: u16 = 1 << 11;
const SIZE_FLAGS: u16 = SIZE | DESCENDANT_SIZE;
const STYLE_FLAGS: u16 = STYLE_CHILDREN | STYLE_DESCENDANTS;
const SCROLL_FLAGS: u16 = SCROLL | DESCENDANT_SCROLL;
const ANCHORED_FLAGS: u16 = ANCHORED | DESCENDANT_ANCHORED;
const CONTAINER_FLAGS: u16 = SIZE_FLAGS | STYLE_FLAGS | SCROLL_FLAGS | ANCHORED_FLAGS;

// cpp: style_recalc_change.h:84-97
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Propagate {
    #[default]
    kNo,
    kUpdatePseudoElements,
    kIndependentInherit,
    kRecalcChildren,
    kRecalcDescendants,
}

// cpp: style_recalc_change.h:99-102,248-250
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleRecalcChange {
    propagate_: Propagate,
    flags_: u16,
}

impl StyleRecalcChange {
    pub const fn new(propagate: Propagate) -> Self {
        Self {
            propagate_: propagate,
            flags_: 0,
        }
    }
    // cpp: style_recalc_change.h:104-127
    pub fn IsEmpty(&self) -> bool {
        self.propagate_ == Propagate::kNo && self.flags_ == 0
    }
    pub fn ForChildren<B: StyleRecalcChangeDOMBackend>(&self, element: &B::Element) -> Self {
        Self {
            propagate_: if self.RecalcDescendants() {
                Propagate::kRecalcDescendants
            } else {
                Propagate::kNo
            },
            flags_: self.FlagsForChildren::<B>(element),
        }
    }
    pub fn ForPseudoElement(&self) -> Self {
        if self.propagate_ == Propagate::kUpdatePseudoElements {
            Self {
                propagate_: Propagate::kRecalcChildren,
                flags_: self.flags_,
            }
        } else {
            *self
        }
    }
    pub fn EnsureAtLeast(&self, propagate: Propagate) -> Self {
        Self {
            propagate_: self.propagate_.max(propagate),
            flags_: self.flags_,
        }
    }
    pub fn ForceRecalcDescendants(&self) -> Self {
        Self {
            propagate_: Propagate::kRecalcDescendants,
            flags_: self.flags_,
        }
    }
    pub fn ForceRecalcChildren(&self) -> Self {
        Self {
            propagate_: Propagate::kRecalcChildren,
            flags_: self.flags_,
        }
    }
    // cpp: style_recalc_change.h:128-177,224-225
    fn WithFlags(&self, flags: u16) -> Self {
        Self {
            propagate_: self.propagate_,
            flags_: self.flags_ | flags,
        }
    }
    pub fn ForceReattachLayoutTree(&self) -> Self {
        self.WithFlags(REATTACH)
    }
    pub fn ForceMarkReattachLayoutTree(&self) -> Self {
        self.WithFlags(MARK_REATTACH)
    }
    pub fn ForceRecalcSizeContainer(&self) -> Self {
        self.WithFlags(SIZE)
    }
    pub fn ForceRecalcDescendantSizeContainers(&self) -> Self {
        self.WithFlags(DESCENDANT_SIZE)
    }
    pub fn ForceRecalcStyleContainerChildren(&self) -> Self {
        self.WithFlags(STYLE_CHILDREN)
    }
    pub fn ForceRecalcStyleContainerDescendants(&self) -> Self {
        self.WithFlags(STYLE_DESCENDANTS)
    }
    pub fn ForceRecalcScrollStateContainer(&self) -> Self {
        self.WithFlags(SCROLL)
    }
    pub fn ForceRecalcDescendantScrollStateContainers(&self) -> Self {
        self.WithFlags(DESCENDANT_SCROLL)
    }
    pub fn ForceRecalcAnchoredContainer(&self) -> Self {
        self.WithFlags(ANCHORED)
    }
    pub fn ForceRecalcDescendantAnchoredContainers(&self) -> Self {
        self.WithFlags(DESCENDANT_ANCHORED)
    }
    pub fn ForceRecalcDescendantContainers(&self) -> Self {
        self.WithFlags(CONTAINER_FLAGS)
    }
    pub fn ForceRecalcDescendantContentVisibility(&self) -> Self {
        self.WithFlags(CONTENT_VISIBILITY)
    }
    pub fn SuppressRecalc(&self) -> Self {
        self.WithFlags(SUPPRESS)
    }
    pub fn Combine(&self, other: &Self) -> Self {
        Self {
            propagate_: self.propagate_.max(other.propagate_),
            flags_: self.flags_ | other.flags_,
        }
    }
    // cpp: style_recalc_change.h:179-199,208,217-219
    pub fn ReattachLayoutTree(&self) -> bool {
        self.flags_ & REATTACH != 0
    }
    pub fn MarkReattachLayoutTree(&self) -> bool {
        self.flags_ & (MARK_REATTACH | REATTACH | SUPPRESS) == MARK_REATTACH | REATTACH
    }
    pub fn RecalcChildren(&self) -> bool {
        self.propagate_ > Propagate::kUpdatePseudoElements
    }
    pub fn RecalcDescendants(&self) -> bool {
        self.propagate_ == Propagate::kRecalcDescendants
    }
    pub fn UpdatePseudoElements(&self) -> bool {
        self.propagate_ != Propagate::kNo
    }
    pub fn IsSuppressed(&self) -> bool {
        self.flags_ & SUPPRESS != 0
    }
    pub fn RootRelativeUnitsMaybeChanged(&self) -> bool {
        self.RecalcDescendants()
    }
    pub fn ContainerRelativeUnitsMaybeChanged(&self) -> bool {
        self.flags_ & DESCENDANT_SIZE != 0
    }
    // cpp: style_recalc_change.h:227-245 (Rust cannot overload by arity).
    fn RecalcSizeContainerQueryDependent(&self) -> bool {
        self.flags_ & SIZE_FLAGS != 0
    }
    fn RecalcStyleContainerQueryDependent(&self) -> bool {
        self.flags_ & STYLE_FLAGS != 0
    }
    fn RecalcScrollStateContainerQueryDependent(&self) -> bool {
        self.flags_ & SCROLL_FLAGS != 0
    }
    fn RecalcAnchoredContainerQueryDependent(&self) -> bool {
        self.flags_ & ANCHORED_FLAGS != 0
    }
    fn HasContainerQueryDependentRecalc(&self) -> bool {
        self.flags_ & CONTAINER_FLAGS != 0
    }
    fn RecalcDescendantContentVisibility(&self) -> bool {
        self.flags_ & CONTENT_VISIBILITY != 0
    }

    // cpp: style_recalc_change.cc:14-17
    pub fn TraverseChildren<B: StyleRecalcChangeDOMBackend>(&self, element: &B::Element) -> bool {
        self.RecalcChildren()
            || self.HasContainerQueryDependentRecalc()
            || B::NodeChildNeedsStyleRecalc(B::ElementNode(element))
            || self.RecalcDescendantContentVisibility()
    }
    // cpp: style_recalc_change.cc:19-22
    pub fn TraversePseudoElements<B: StyleRecalcChangeDOMBackend>(
        &self,
        element: &B::Element,
    ) -> bool {
        self.UpdatePseudoElements()
            || self.HasContainerQueryDependentRecalc()
            || B::NodeChildNeedsStyleRecalc(B::ElementNode(element))
            || self.RecalcDescendantContentVisibility()
    }
    // cpp: style_recalc_change.cc:24-29
    pub fn TraverseChild<B: StyleRecalcChangeDOMBackend>(&self, node: &B::Node) -> bool {
        self.ShouldRecalcStyleFor::<B>(node)
            || self.MarkReattachLayoutTree()
            || B::NodeChildNeedsStyleRecalc(node)
            || B::NodeGetForceReattachLayoutTree(node)
            || self.HasContainerQueryDependentRecalc()
            || B::NodeNeedsLayoutSubtreeUpdate(node)
            || self.RecalcDescendantContentVisibility()
    }
    // cpp: style_recalc_change.cc:31-54
    pub fn RecalcContainerQueryDependent<B: StyleRecalcChangeDOMBackend>(
        &self,
        node: &B::Node,
    ) -> bool {
        if !self.HasContainerQueryDependentRecalc() {
            return false;
        }
        let Some(element) = B::NodeElement(node) else {
            return false;
        };
        let Some(style) = B::ElementComputedStyle(element) else {
            return true;
        };
        (self.RecalcSizeContainerQueryDependent()
            && (style.DependsOnSizeContainerQueries()
                || style.HighlightPseudoElementStylesDependOnContainerUnits()))
            || (self.RecalcStyleContainerQueryDependent() && style.DependsOnStyleContainerQueries())
            || (self.RecalcScrollStateContainerQueryDependent()
                && style.DependsOnScrollStateContainerQueries())
            || (self.RecalcAnchoredContainerQueryDependent()
                && style.DependsOnAnchoredContainerQueries())
    }
    // cpp: style_recalc_change.cc:56-67
    pub fn ShouldRecalcStyleFor<B: StyleRecalcChangeDOMBackend>(&self, node: &B::Node) -> bool {
        if self.IsSuppressed() {
            return false;
        }
        if self.RecalcChildren() {
            return true;
        }
        if B::NodeNeedsStyleRecalc(node) {
            return true;
        }
        self.RecalcContainerQueryDependent::<B>(node)
    }
    // cpp: style_recalc_change.cc:69-95
    pub fn ShouldUpdatePseudoElement<B: StyleRecalcChangeDOMBackend>(
        &self,
        pseudo: &B::PseudoElement,
    ) -> bool {
        if self.UpdatePseudoElements() {
            return true;
        }
        let node = B::PseudoElementNode(pseudo);
        if B::NodeNeedsStyleRecalc(node) {
            return true;
        }
        if B::NodeChildNeedsStyleRecalc(node) {
            return true;
        }
        if B::NodeNeedsLayoutSubtreeUpdate(node) {
            return true;
        }
        if !self.HasContainerQueryDependentRecalc() {
            return false;
        }
        let style = B::PseudoElementComputedStyle(pseudo);
        (self.RecalcSizeContainerQueryDependent() && style.DependsOnSizeContainerQueries())
            || (self.RecalcStyleContainerQueryDependent() && style.DependsOnStyleContainerQueries())
            || (self.RecalcScrollStateContainerQueryDependent()
                && style.DependsOnScrollStateContainerQueries())
            || (self.RecalcAnchoredContainerQueryDependent()
                && style.DependsOnAnchoredContainerQueries())
    }
    // cpp: style_recalc_change.cc:153-186
    fn FlagsForChildren<B: StyleRecalcChangeDOMBackend>(&self, element: &B::Element) -> u16 {
        if self.flags_ == 0 {
            return 0;
        }
        let mut result = self.flags_ & !STYLE_CHILDREN;
        if result & (SIZE_FLAGS | SUPPRESS) == SIZE {
            if let Some(style) = B::ElementComputedStyle(element) {
                if B::CanMatchSizeContainerQueries(style, element) {
                    result &= !SIZE;
                }
            }
        }
        if result & SUPPRESS != 0 {
            result &= !SUPPRESS;
        } else {
            result &= !MARK_REATTACH;
        }
        result
    }
    // cpp: style_recalc_change.cc:188-198
    pub fn IndependentInherit(&self, old_style: &ComputedStyle) -> bool {
        self.propagate_ == Propagate::kIndependentInherit
            && (!self.RecalcSizeContainerQueryDependent()
                || !old_style.DependsOnSizeContainerQueries())
            && (!self.RecalcStyleContainerQueryDependent()
                || !old_style.DependsOnStyleContainerQueries())
    }
}
