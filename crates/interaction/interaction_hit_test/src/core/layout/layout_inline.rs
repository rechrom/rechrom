/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 *           (C) 1999 Antti Koivisto (koivisto@kde.org)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008, 2009 Apple Inc.
 *               All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB. If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/layout_inline.h:197-200,298-301,318
// Source: third_party/blink/renderer/core/layout/layout_inline.cc:597-697,834-844
// Only the hit-test methods of LayoutInline are translated in this module.

use foundation::{gfx, PhysicalOffset, PhysicalRect};

use super::geometry::physical_rect::PhysicalRectHitTestExt;
use super::hit_test_location::HitTestLocation;
use super::hit_test_phase::HitTestPhase;
use super::hit_test_request::ListBasedHitTestBehavior;
use super::hit_test_result::{HitTestRegion, HitTestResult, HitTestResultRequest, NodeRef};

// Source: layout_inline.cc:614-647,671-688
// The iterator borrows the existing InlineCursor/FragmentItems owner. Its
// items are call-scoped adapters borrowing native entries, rather than a
// copied fragment list. Each adapter is produced lazily as the cursor advances.
pub trait InlineHitTestItem<'a, L: ?Sized> {
    fn ContainerFragmentIndex(&self) -> usize;
    fn HasBoxFragment(&self) -> bool;
    fn IsOpaqueBoxFragment(&self) -> bool;
    fn OffsetInContainerFragment(&self) -> PhysicalOffset;
    fn RectInContainerFragment(&self) -> PhysicalRect;
    // Calls the corresponding BoxFragmentPainter::NodeAtPoint overload with
    // this very cursor/item/physical-fragment identity and a local context.
    fn BoxFragmentNodeAtPoint(
        &self,
        result: &mut HitTestResult<'a, L>,
        location: &HitTestLocation,
        physical_offset: PhysicalOffset,
        inline_root_offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
}

pub trait InlineHitTestParentCursor {
    fn Identity(&self) -> usize;
}

// The existing cc::Region implementation supplies union/containment; Layout
// traverses element by element and does not retain a second geometric model.
pub trait InlineHitTestRegion: HitTestRegion {
    fn Union(&mut self, rect: gfx::Rect);
    fn AsRegion(&self) -> &dyn HitTestRegion;
}

pub trait LayoutInlineHitTestSource<'a, L: ?Sized> {
    fn IsInLayoutNGInlineFormattingContext(&self) -> bool;
    fn NeedsLayout(&self) -> bool;
    fn HasSelfPaintingLayer(&self) -> bool;
    fn CanTraversePhysicalFragments(&self) -> bool;
    fn IsFragmented(&self) -> bool;
    fn VisibleToHitTestRequest(&self, request: &HitTestResultRequest<'a, L>) -> bool;
    fn IsDescendantOfParentBlock(&self, parent: &dyn InlineHitTestParentCursor) -> bool;
    fn SameLayoutObjectItems(
        &self,
    ) -> Box<dyn Iterator<Item = Box<dyn InlineHitTestItem<'a, L> + '_>> + '_>;
    fn CulledInlineItems<'s>(
        &'s self,
        parent: &'s dyn InlineHitTestParentCursor,
    ) -> Box<dyn Iterator<Item = Box<dyn InlineHitTestItem<'a, L> + 's>> + 's>;
    fn NewRegion(&self) -> Box<dyn InlineHitTestRegion>;
    fn GetNode(&self) -> Option<NodeRef>;
}

// Source: layout_inline.h:298-301; layout_inline.cc:597-652
pub fn NodeAtPoint<'a, L: ?Sized>(
    inline: &dyn LayoutInlineHitTestSource<'a, L>,
    result: &mut HitTestResult<'a, L>,
    hit_test_location: &HitTestLocation,
    accumulated_offset: PhysicalOffset,
    phase: HitTestPhase,
) -> bool {
    if inline.IsInLayoutNGInlineFormattingContext() {
        if inline.NeedsLayout() {
            // Source: layout_inline.cc:603-607, the release diagnostic does
            // not change the source early-out on a stale layout object.
            return false;
        }
        debug_assert!(inline.HasSelfPaintingLayer());
        let mut cursor = inline.SameLayoutObjectItems().peekable();
        let Some(first) = cursor.peek() else {
            return false;
        };
        let mut target_fragment_idx = hit_test_location.FragmentIndex();
        debug_assert!(
            !inline.CanTraversePhysicalFragments()
                || target_fragment_idx >= 0
                || !inline.IsFragmented()
        );
        if target_fragment_idx != -1 {
            target_fragment_idx += first.ContainerFragmentIndex() as i32;
        }
        for item in cursor {
            if target_fragment_idx != -1
                && target_fragment_idx as usize != item.ContainerFragmentIndex()
            {
                continue;
            }
            debug_assert!(item.HasBoxFragment());
            let child_offset = accumulated_offset + item.OffsetInContainerFragment();
            if item.BoxFragmentNodeAtPoint(
                result,
                hit_test_location,
                child_offset,
                accumulated_offset,
                phase,
            ) {
                return true;
            }
        }
        return false;
    }
    unreachable!("LayoutInline hit testing requires LayoutNG inline formatting");
}

// Source: layout_inline.h:197-200; layout_inline.cc:654-697
pub fn HitTestCulledInline<'a, L: ?Sized>(
    inline: &dyn LayoutInlineHitTestSource<'a, L>,
    result: &mut HitTestResult<'a, L>,
    hit_test_location: &HitTestLocation,
    accumulated_offset: PhysicalOffset,
    parent_cursor: &dyn InlineHitTestParentCursor,
) -> bool {
    if !inline.VisibleToHitTestRequest(result.GetHitTestRequest()) {
        return false;
    }
    let adjusted_location = HitTestLocation::WithOffset(hit_test_location, -accumulated_offset);
    let mut region_result = inline.NewRegion();
    let mut intersected = false;
    debug_assert!(inline.IsDescendantOfParentBlock(parent_cursor));
    for item in inline.CulledInlineItems(parent_cursor) {
        if item.HasBoxFragment() && item.IsOpaqueBoxFragment() {
            continue;
        }
        let rect = item.RectInContainerFragment();
        if adjusted_location.Intersects(&rect) {
            intersected = true;
            region_result.Union(rect.ToEnclosingRect());
        }
    }
    if intersected {
        UpdateHitTestResult(inline, result, *adjusted_location.Point());
        if result.AddNodeToListBasedTestResultRegion(
            inline.GetNode(),
            &adjusted_location,
            region_result.AsRegion(),
        ) == ListBasedHitTestBehavior::kStopHitTesting
        {
            return true;
        }
    }
    false
}

// Source: layout_inline.h:318; layout_inline.cc:834-844
pub fn UpdateHitTestResult<'a, L: ?Sized>(
    inline: &dyn LayoutInlineHitTestSource<'a, L>,
    result: &mut HitTestResult<'a, L>,
    point: PhysicalOffset,
) {
    if result.InnerNode().is_some() {
        return;
    }
    if let Some(node) = inline.GetNode() {
        result.SetNodeAndPosition(Some(node), point);
    }
}
