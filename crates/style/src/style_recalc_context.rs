// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/style_recalc_context.h
// cpp: third_party/blink/renderer/core/css/style_recalc_context.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   style_recalc_context.h: 131 / 42 / 22 / 20 / 0.
//   style_recalc_context.cc: 124 / 90 / 82 / 8 / 0.
// Effective excludes comments/blanks; braces retained. Every production
// declaration/body is mapped, including the fatal pseudo-id precondition.
// h mapped: 27,36,44-46,48-50,59,67,74,81,87,89,94,107,111,115,119,123,126-127.
// h omitted: 5-6,8-11,13,15-19,28,30,52,61-62,64,129,131.
// cc mapped: 15-26,36-47,49-57,59-64,66-71,73-78,80-83,92-94,96-103,105-115,117-119,121-122.
// cc omitted: 5,7-11,13,124.
// Omissions: preprocessing/includes/namespace/forward/access/stack-allocation
// scaffolding and friend-test declarations. No production statement omitted.
// This source version has four ancestor/parent constructors. It has no
// ForSlottedRules/ForPartRules or LayoutObject-based context derivation.

use crate::css_property_value_set::{CSSPropertyValueSet, CSSPropertyValueSetBackend};
use foundation::Persistent;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::PseudoId;
use std::rc::Rc;

/// Existing DOM/pseudo/display-lock owners. Returned Rc values retain the
/// original Element identity. No context algorithm or ComputedStyle flag
/// computation is delegated to the backend, and no defaults are supplied.
pub trait StyleRecalcContextBackend {
    type Element;
    type AnchorEvaluator;
    type StyleScopeFrame;
    type PropertyBackend: CSSPropertyValueSetBackend;
    type DisplayLockContext;
    type ElementAnimations;

    fn FlatTreeParentElement(element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ElementComputedStyle(element: &Self::Element) -> Option<&ComputedStyle>;
    fn ElementPseudoId(element: &Self::Element) -> PseudoId;
    fn IsLayoutSiblingOfOriginatingElement(element: &Self::Element, pseudo: PseudoId) -> bool;
    fn IsDocumentElement(element: &Self::Element) -> bool;
    fn ElementDisplayLockContext(element: &Self::Element) -> Option<&Self::DisplayLockContext>;
    fn DisplayLockIsAuto(context: &Self::DisplayLockContext) -> bool;
    fn DisplayLockIsLocked(context: &Self::DisplayLockContext) -> bool;
    fn ElementAnimations(element: &Self::Element) -> Option<&Self::ElementAnimations>;
}

// cpp: style_recalc_context.h:67-126
// Rc and Persistent root the same owner objects while a context is carried
// across a recalc; Clone preserves owner identity and never copies a style.
pub struct StyleRecalcContext<B: StyleRecalcContextBackend> {
    pub size_container: Option<Rc<B::Element>>,
    pub anchor_evaluator: Option<Rc<B::AnchorEvaluator>>,
    pub try_set: Option<Rc<CSSPropertyValueSet<B::PropertyBackend>>>,
    pub try_tactics_set: Option<Rc<CSSPropertyValueSet<B::PropertyBackend>>>,
    pub style_scope_frame: Option<Rc<B::StyleScopeFrame>>,
    pub old_style: Option<Persistent<ComputedStyle>>,
    pub can_use_incremental_style: bool,
    pub is_ensuring_style: bool,
    pub has_content_visibility_auto_locked_ancestor: bool,
    pub has_animating_ancestor: bool,
    pub has_scroller_ancestor_with_scroll_marker_group_property: bool,
    pub has_anchored_container: bool,
}

impl<B: StyleRecalcContextBackend> Default for StyleRecalcContext<B> {
    fn default() -> Self {
        Self {
            size_container: None,
            anchor_evaluator: None,
            try_set: None,
            try_tactics_set: None,
            style_scope_frame: None,
            old_style: None,
            can_use_incremental_style: false,
            is_ensuring_style: false,
            has_content_visibility_auto_locked_ancestor: false,
            has_animating_ancestor: false,
            has_scroller_ancestor_with_scroll_marker_group_property: false,
            has_anchored_container: false,
        }
    }
}

impl<B: StyleRecalcContextBackend> Clone for StyleRecalcContext<B> {
    fn clone(&self) -> Self {
        Self {
            size_container: self.size_container.clone(),
            anchor_evaluator: self.anchor_evaluator.clone(),
            try_set: self.try_set.clone(),
            try_tactics_set: self.try_tactics_set.clone(),
            style_scope_frame: self.style_scope_frame.clone(),
            old_style: self.old_style.clone(),
            can_use_incremental_style: self.can_use_incremental_style,
            is_ensuring_style: self.is_ensuring_style,
            has_content_visibility_auto_locked_ancestor: self
                .has_content_visibility_auto_locked_ancestor,
            has_animating_ancestor: self.has_animating_ancestor,
            has_scroller_ancestor_with_scroll_marker_group_property: self
                .has_scroller_ancestor_with_scroll_marker_group_property,
            has_anchored_container: self.has_anchored_container,
        }
    }
}

impl<B: StyleRecalcContextBackend> StyleRecalcContext<B> {
    // cpp: style_recalc_context.cc:15-64
    fn FromInclusiveAncestors(start: Rc<B::Element>, pseudo_id: PseudoId) -> Self {
        let mut result = Self::default();
        let mut current = Some(start.clone());
        while let Some(element) = current {
            if let Some(style) = B::ElementComputedStyle(&element) {
                if result.size_container.is_none()
                    && style.IsContainerForSizeContainerQueries()
                    && (!Rc::ptr_eq(&element, &start)
                        || !B::IsLayoutSiblingOfOriginatingElement(&start, pseudo_id))
                {
                    result.size_container = Some(element.clone());
                }
                if !result.has_scroller_ancestor_with_scroll_marker_group_property
                    && !style.ScrollMarkerGroupNone()
                    && (style.IsScrollContainer() || B::IsDocumentElement(&element))
                {
                    result.has_scroller_ancestor_with_scroll_marker_group_property = true;
                }
                if !result.has_anchored_container {
                    result.has_anchored_container = style.IsContainerForAnchoredContainerQueries();
                }
            }
            if !result.has_content_visibility_auto_locked_ancestor {
                if let Some(lock) = B::ElementDisplayLockContext(&element) {
                    if B::DisplayLockIsAuto(lock) && B::DisplayLockIsLocked(lock) {
                        result.has_content_visibility_auto_locked_ancestor = true;
                    }
                }
            }
            if !result.has_animating_ancestor && B::ElementAnimations(&element).is_some() {
                result.has_animating_ancestor = true;
            }
            current = B::FlatTreeParentElement(&element);
        }
        result
    }
    // cpp: style_recalc_context.cc:66-71
    pub fn FromAncestors(element: &Rc<B::Element>) -> Self {
        if let Some(parent) = B::FlatTreeParentElement(element) {
            Self::FromInclusiveAncestors(parent, B::ElementPseudoId(element))
        } else {
            Self::default()
        }
    }
    // cpp: style_recalc_context.cc:73-78
    pub fn FromPseudoElementAncestors(
        originating_element: Rc<B::Element>,
        pseudo_id: PseudoId,
    ) -> Self {
        assert!(
            pseudo_id != PseudoId::kPseudoIdNone,
            "pseudo element ancestors require a pseudo id"
        );
        Self::FromInclusiveAncestors(originating_element, pseudo_id)
    }
    // cpp: style_recalc_context.cc:80-122
    pub fn FromParentContext(parent_context: &Self, element: Rc<B::Element>) -> Self {
        let mut result = parent_context.clone();
        result.anchor_evaluator = None;
        result.try_set = None;
        result.try_tactics_set = None;
        if !result.has_content_visibility_auto_locked_ancestor {
            if let Some(lock) = B::ElementDisplayLockContext(&element) {
                if B::DisplayLockIsAuto(lock) && B::DisplayLockIsLocked(lock) {
                    result.has_content_visibility_auto_locked_ancestor = true;
                }
            }
        }
        if let Some(style) = B::ElementComputedStyle(&element) {
            result.has_scroller_ancestor_with_scroll_marker_group_property |=
                (style.IsScrollContainer() || B::IsDocumentElement(&element))
                    && !style.ScrollMarkerGroupNone();
            if style.IsContainerForSizeContainerQueries() {
                result.size_container = Some(element.clone());
            }
            if style.IsContainerForAnchoredContainerQueries() {
                result.has_anchored_container = true;
            }
        }
        if !result.has_animating_ancestor && B::ElementAnimations(&element).is_some() {
            result.has_animating_ancestor = true;
        }
        result
    }
}
