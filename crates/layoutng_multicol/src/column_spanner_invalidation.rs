// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in ../../../third_party/LICENSE.chromium.
//
// Source: Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
// third_party/blink/renderer/core/layout/layout_box.cc:2897-2929.

use foundation::DynamicTo;
use layoutng_assembly::internal::{layout_box::LayoutBox, layout_object::LayoutObject};

// LayoutBox's lifecycle calls this through its existing multicol-owner slot
// when a style change alters whether the box permits spanner descendants.
pub fn MarkColumnSpannerCandidatesForLayoutIfNeeded(container: &mut LayoutBox) {
    container.CheckIsNotDestroyed();
    debug_assert!(container.IsInsideMulticol());

    // Examine descendants and ancestors, not the changed box itself.
    if container.IsMulticolContainer() || container.IsSelfValidColumnSpannerCurrentStyle() {
        return;
    }
    if !container.DoesAncestryAllowColumnSpannerCurrentStyle() {
        return;
    }

    let root = container as *mut LayoutBox as *const LayoutObject;
    let mut descendant = container.NextInPreOrder(root);
    while !descendant.is_null() {
        let box_ = DynamicTo::<LayoutBox>(descendant);
        if !box_.is_null() {
            if unsafe { &*box_ }.IsSelfValidColumnSpannerCurrentStyle() {
                unsafe { &mut *box_ }.MarkParentForSpannerOrOutOfFlowPositionedChange();
                descendant = unsafe { &*descendant }.NextInPreOrderAfterChildren(root);
                continue;
            }
            if unsafe { &*box_ }.ShouldPreventColumnSpannerDescendants() {
                descendant = unsafe { &*descendant }.NextInPreOrderAfterChildren(root);
                continue;
            }
        }
        descendant = unsafe { &*descendant }.NextInPreOrder(root);
    }
}
