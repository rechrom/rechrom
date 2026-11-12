// Copyright 2017 The Chromium Authors. BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/bidi_adjustment.cc:54-156,346-391,536-740,766-770.
// Layout owns rendered run traversal; Editing owns visible/flat Positions.
use super::*;
use crate::inline_cursor::InlineCursor;
use crate::layout_algo_inline_editing_caret::{
    AtSide, BidiLevel, InlineCaretPosition, ResolveForObject, ResolvedDirection,
    StepIgnoringLineBreak,
};
use foundation::TextDirection;

/// Native DOM positions at the one endpoint selected by the source run owner.
/// No inline object, cursor or retained geometry crosses this boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditingBidiRangeAdjustment {
    Unchanged,
    AdjustedBase(crate::EditingPosition),
    AdjustedExtent(crate::EditingPosition),
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Boundary {
    NotBoundary,
    Left,
    Right,
}
struct RenderedPosition {
    // Exactly the source's line-rooted cursor, borrowing FragmentItems.
    box_: InlineCursor,
    boundary: Boundary,
}
impl PartialEq for RenderedPosition {
    fn eq(&self, other: &Self) -> bool {
        self.box_ == other.box_ && self.boundary == other.boundary
    }
}
impl RenderedPosition {
    fn Create(caret: InlineCaretPosition) -> Self {
        let mut line = caret.cursor.CursorForMovingAcrossFragmentainer();
        line.MoveToContainingLine();
        let mut box_ = line.CursorForDescendants();
        box_.MoveToCursor(&caret.cursor);
        let at_boundary = caret.IsAtFragmentStart() || caret.IsAtFragmentEnd();
        if !at_boundary {
            return Self {
                box_,
                boundary: Boundary::NotBoundary,
            };
        }
        let at_left =
            caret.IsAtFragmentStart() == (ResolvedDirection(&box_) == TextDirection::kLtr);
        let neighbor = StepIgnoringLineBreak(&box_, !at_left);
        // Source: RenderedPosition::Create:709-740. Select the higher-level
        // run at either visual edge, retaining this same line's native items.
        if neighbor.IsNotNull() && BidiLevel(&neighbor) > BidiLevel(&box_) {
            return Self {
                box_: neighbor,
                boundary: if at_left {
                    Boundary::Right
                } else {
                    Boundary::Left
                },
            };
        }
        let boundary = if neighbor.IsNotNull() && BidiLevel(&neighbor) == BidiLevel(&box_) {
            Boundary::NotBoundary
        } else if at_left {
            Boundary::Left
        } else {
            Boundary::Right
        };
        Self { box_, boundary }
    }
    fn AtBidiBoundary(&self) -> bool {
        self.boundary != Boundary::NotBoundary
    }
    fn IsPossiblyOtherBoundaryOf(&self, other: &Self) -> bool {
        debug_assert!(other.AtBidiBoundary());
        self.AtBidiBoundary()
            && self.boundary != other.boundary
            && BidiLevel(&self.box_) >= BidiLevel(&other.box_)
    }
    fn BidiRunContains(&self, other: &Self) -> bool {
        debug_assert!(self.AtBidiBoundary());
        let level = BidiLevel(&self.box_);
        if level > BidiLevel(&other.box_) {
            return false;
        }
        let mut boundary_of_other = other.box_.clone();
        let right = self.boundary == Boundary::Right;
        loop {
            let next = StepIgnoringLineBreak(&boundary_of_other, right);
            if next.IsNull() || BidiLevel(&next) < level {
                break;
            }
            boundary_of_other = next;
        }
        self.box_ == boundary_of_other
    }
    fn GetPosition(&self) -> crate::EditingPosition {
        debug_assert!(self.AtBidiBoundary());
        AtSide(self.box_.clone(), self.boundary == Boundary::Left)
            .ToPositionInDOMTreeWithAffinity()
            .position
    }
}
impl LayoutOutput<'_> {
    /// Inputs have already passed ComputeInlineAdjustedPosition in Editing.
    /// Resolution/traversal remains synchronous over the borrowed native owner.
    pub fn AdjustBidiRange(
        &self,
        base: &crate::EditingPositionWithAffinity,
        extent: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
    ) -> EditingBidiRangeAdjustment {
        let _scope = LayoutPassScope::new(&self.engine.algorithms_, &self.engine.objects_);
        let resolve = |position: &crate::EditingPositionWithAffinity| {
            let anchor = self.Node(position.position.node_id?)?;
            ResolveForObject(anchor.object, position, options).map(RenderedPosition::Create)
        };
        let (Some(base), Some(extent)) = (resolve(base), resolve(extent)) else {
            return EditingBidiRangeAdjustment::Unchanged;
        };
        if base == extent || (!base.AtBidiBoundary() && !extent.AtBidiBoundary()) {
            return EditingBidiRangeAdjustment::Unchanged;
        }
        if base.AtBidiBoundary() {
            if !extent.IsPossiblyOtherBoundaryOf(&base) && base.BidiRunContains(&extent) {
                return EditingBidiRangeAdjustment::AdjustedBase(base.GetPosition());
            }
            return EditingBidiRangeAdjustment::Unchanged;
        }
        if extent.AtBidiBoundary() && extent.BidiRunContains(&base) {
            return EditingBidiRangeAdjustment::AdjustedExtent(extent.GetPosition());
        }
        EditingBidiRangeAdjustment::Unchanged
    }
    /// Source: editing_utilities.cc:1382-1405. Missing layout or identical
    /// native objects is the source true branch, rather than a new identity.
    pub fn EditingHaveSameOutOfFlowAncestor(&self, anchor: u64, target: u64) -> bool {
        let (Some(anchor), Some(target)) = (self.Node(anchor), self.Node(target)) else {
            return true;
        };
        if std::ptr::eq(anchor.object, target.object) {
            return true;
        }
        let _scope = LayoutPassScope::new(&self.engine.algorithms_, &self.engine.objects_);
        let nearest = |mut object: *const LayoutObject| {
            while let Some(current) = unsafe { object.as_ref() } {
                if current.IsOutOfFlowPositioned() {
                    return object;
                }
                object = unsafe { current.ContainingBlock().as_ref() }.map_or(
                    std::ptr::null(),
                    |block| {
                        let base: &LayoutObject = block;
                        base as *const LayoutObject
                    },
                );
            }
            std::ptr::null()
        };
        nearest(anchor.object) == nearest(target.object)
    }
    /// Source: editing_utilities.cc:1420-1435. Both coordinate conversions
    /// explicitly ignore transforms and resolve against the SAME editable box.
    pub fn EditingPositionForPointInContainBoundary(
        &self,
        target: u64,
        editable: u64,
        local_point: &PhysicalOffset,
        options: crate::EditingPositionOptions,
    ) -> Option<crate::EditingPositionWithAffinity> {
        use crate::internal::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
        let target = self.Node(target)?;
        let editable = self.Node(editable)?;
        if !editable.object.VisibleToHitTesting() {
            return None;
        }
        let _scope = LayoutPassScope::new(&self.engine.algorithms_, &self.engine.objects_);
        let flags = MapCoordinatesFlags::from_mode(MapCoordinatesMode::kIgnoreTransforms);
        let absolute = target
            .object
            .LocalToAbsolutePointPhysical(local_point, flags);
        let local = editable
            .object
            .AbsoluteToLocalPointPhysical(&absolute, flags);
        editable.PositionForPoint(&local, options)
    }
}
impl LayoutNode<'_> {
    pub fn EditingVisibleToHitTesting(&self) -> bool {
        self.object.VisibleToHitTesting()
    }
}
