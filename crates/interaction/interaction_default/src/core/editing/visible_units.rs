// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: visible_units.cc:515-1149; editing_utilities.cc:397-421,490-514.
use super::{
    position::{Position, PositionWithAffinity, TextAffinity},
    position_iterator::{PositionIterator, PositionIteratorContext},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditingBoundaryCrossingRule {
    CannotCross,
    CanCross,
    CanSkipOver,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SnapToClient {
    Editing,
    LocalCaretRect,
}

/// Source: visible_units.cc:399-410. A document edge is a visible caret
/// boundary, rather than the first/last DOM text node or a geometry extent.
pub fn IsStartOfDocument<C>(
    ctx: &C,
    position: &super::visible_position::VisiblePosition<C::Node>,
) -> bool
where
    C: super::editing_boundaries::EditableBoundaryContext
        + super::visible_position::DownstreamVisiblePositionContext,
{
    !position.IsNull()
        && super::visible_navigation::PreviousPositionOf(
            ctx,
            position.DeepEquivalent(),
            EditingBoundaryCrossingRule::CanCross,
        )
        .IsNull()
}
pub fn IsEndOfDocument<C>(
    ctx: &C,
    position: &super::visible_position::VisiblePosition<C::Node>,
) -> bool
where
    C: super::editing_boundaries::EditableBoundaryContext
        + super::visible_position::VisiblePositionContext,
{
    !position.IsNull()
        && super::visible_navigation::NextPositionOf(
            ctx,
            position.ToPositionWithAffinity(),
            EditingBoundaryCrossingRule::CanCross,
        )
        .IsNull()
}

/// Facts sampled from the associated resident LayoutObject (including the
/// source first-letter choice). No LayoutObject or style is copied or retained.
pub struct AssociatedCaretFacts<W> {
    pub visible: bool,
    pub blocked_by_display_lock: bool,
    pub writing_mode: W,
    pub text: Option<TextCaretFacts>,
}
pub struct CaretObjectFacts<W> {
    pub visible: bool,
    pub blocked_by_display_lock: bool,
    pub writing_mode: W,
    pub is_text: bool,
}
pub struct TextCaretFacts {
    pub start_offset: usize,
    pub min_offset: usize,
    pub max_offset: usize,
    pub has_non_collapsed_text: bool,
    pub contains_caret_offset: bool,
    pub before_non_collapsed_character: bool,
    pub after_non_collapsed_character: bool,
}
/// Strategy selects DOM or flat-tree traversal through the same canonical
/// nodes. The algorithm, boundary decisions and traversal remain in Editing.
// Source: visible_units.cc:673-681. These are native dynamic-type facts.
pub fn CanHaveCaretPosition(is_svg: bool, is_svg_text: bool, is_svg_foreign_object: bool) -> bool {
    !is_svg || is_svg_text || is_svg_foreign_object
}
pub trait VisibleUnitsContext: PositionIteratorContext {
    type WritingMode: PartialEq;
    fn IsEditable(&self, node: &Self::Node) -> bool;
    fn EndsAreVisuallyDistinct(&self, node: &Self::Node) -> bool;
    fn CanHaveCaretPosition(&self, node: &Self::Node) -> bool;
    fn CaretFacts(
        &self,
        node: &Self::Node,
        offset: usize,
        first_letter: bool,
    ) -> Option<AssociatedCaretFacts<Self::WritingMode>>;
    fn LayoutCaretFacts(&self, node: &Self::Node) -> Option<CaretObjectFacts<Self::WritingMode>>;
    fn CaretMaxOffset(&self, node: &Self::Node) -> usize;
    fn IsBody(&self, node: &Self::Node) -> bool;
    fn IsDocumentElement(&self, node: &Self::Node) -> bool;
    fn IsDocument(&self, node: &Self::Node) -> bool;
    fn IsSelectable(&self, node: &Self::Node) -> bool;
    fn IsBR(&self, node: &Self::Node) -> bool;
    fn IsSVG(&self, node: &Self::Node) -> bool;
    fn IsBlockContainer(&self, node: &Self::Node) -> bool;
    fn IsBlockFlowElement(&self, node: &Self::Node) -> bool;
    fn HasLogicalHeight(&self, node: &Self::Node) -> bool;
    fn HasRenderedNonAnonymousDescendantsWithHeight(&self, node: &Self::Node) -> bool;
    /// Static source dispatch: DOM uses the flat-tree facade plus DOM boundary
    /// adjustment; a flat-tree strategy invokes its own core traversal.
    fn BoundaryCaretPosition(
        &self,
        position: &Position<Self::Node>,
        rule: EditingBoundaryCrossingRule,
        backward: bool,
    ) -> Position<Self::Node>;
}
// Source editing_utilities.cc:959-968; callers use DOM strategy here even
// when canonicalizing a flat-tree position.
pub fn EnclosingBlockFlowElement<C: VisibleUnitsContext>(
    ctx: &C,
    node: &C::Node,
) -> Option<C::Node> {
    if ctx.IsBlockFlowElement(node) {
        return Some(node.clone());
    }
    let mut parent = ctx.Parent(node);
    while let Some(node) = parent {
        if ctx.IsBlockFlowElement(&node) || ctx.IsBody(&node) {
            return Some(node);
        }
        parent = ctx.Parent(&node);
    }
    None
}
fn EnclosingVisualBoundary<C: VisibleUnitsContext>(ctx: &C, node: &C::Node) -> Option<C::Node> {
    let mut node = Some(node.clone());
    while let Some(current) = node {
        if ctx.EndsAreVisuallyDistinct(&current) {
            return Some(current);
        }
        node = ctx.Parent(&current);
    }
    None
}
fn IsStreamer<C: VisibleUnitsContext>(ctx: &C, cursor: &PositionIterator<C::Node>) -> bool {
    cursor
        .GetNode()
        .is_none_or(|node| ctx.ChildCount(node) == 0 || ctx.EditingIgnoresContent(node))
        || cursor.AtStartOfNode(ctx)
}
pub fn MostBackwardCaretPosition<C: VisibleUnitsContext>(
    ctx: &C,
    position: &Position<C::Node>,
    rule: EditingBoundaryCrossingRule,
    client: SnapToClient,
) -> Position<C::Node> {
    let Some(start) = position.AnchorNode() else {
        return Position::default();
    };
    let boundary = EnclosingVisualBoundary(ctx, start);
    let adjusted = if position.AnchorType() == super::position::PositionAnchorType::AfterAnchor {
        if ctx.IsUserSelectContain(start) {
            position.ToOffsetInAnchor(ctx)
        } else {
            Position::EditingPositionOf(ctx, start.clone(), ctx.CaretMaxOffset(start))
        }
    } else {
        position.clone()
    };
    let mut last_visible = PositionIterator::new(ctx, &adjusted);
    let mut last_node = if rule == EditingBoundaryCrossingRule::CannotCross
        && client == SnapToClient::LocalCaretRect
    {
        position
            .ComputeContainerNode(ctx)
            .unwrap_or_else(|| start.clone())
    } else {
        start.clone()
    };
    let start_editable = ctx.IsEditable(&last_node);
    let mut crossed = false;
    let mut writing_mode = None;
    let mut cursor = last_visible.clone();
    while !cursor.AtStart(ctx) {
        let current = cursor.GetNode().expect("nonterminal source cursor").clone();
        if current != last_node {
            if start_editable != ctx.IsEditable(&current) {
                if rule == EditingBoundaryCrossingRule::CannotCross
                    && client != SnapToClient::LocalCaretRect
                {
                    break;
                }
                crossed = true;
            }
            last_node = current.clone();
        }
        if !ctx.CanHaveCaretPosition(&current) {
            if crossed && rule == EditingBoundaryCrossingRule::CannotCross {
                break;
            }
            cursor.Decrement(ctx);
            continue;
        }
        if ctx.EndsAreVisuallyDistinct(&current) && boundary.as_ref() != Some(&current) {
            return last_visible.DeprecatedComputePosition(ctx);
        }
        let facts = ctx.CaretFacts(
            &current,
            if ctx.IsText(&current) {
                cursor.OffsetInTextNode()
            } else {
                0
            },
            true,
        );
        let Some(facts) = facts.filter(|facts| facts.visible && !facts.blocked_by_display_lock)
        else {
            if crossed && rule == EditingBoundaryCrossingRule::CannotCross {
                break;
            }
            cursor.Decrement(ctx);
            continue;
        };
        if let Some(mode) = &writing_mode {
            if mode != &facts.writing_mode {
                return last_visible.ComputePosition(ctx);
            }
        } else {
            writing_mode = Some(facts.writing_mode);
        }
        if crossed {
            if rule == EditingBoundaryCrossingRule::CannotCross {
                return if &current == start {
                    position.clone()
                } else {
                    Position::AfterNode(current)
                };
            }
            if rule == EditingBoundaryCrossingRule::CanCross {
                last_visible = cursor;
                break;
            }
        }
        if IsStreamer(ctx, &cursor) {
            last_visible = cursor.clone();
        }
        if ctx.EndsAreVisuallyDistinct(&current) && cursor.AtStartOfNode(ctx) {
            return last_visible.DeprecatedComputePosition(ctx);
        }
        if ctx.EditingIgnoresContent(&current) || ctx.IsDisplayInsideTable(&current) {
            if cursor.AtEndOfNode(ctx) {
                return Position::AfterNode(current);
            }
            cursor.Decrement(ctx);
            continue;
        }
        if let Some(text) = facts.text.filter(|text| text.has_non_collapsed_text) {
            if &current != start {
                return Position::new(current, text.max_offset + text.start_offset);
            }
            if text.after_non_collapsed_character {
                return cursor.ComputePosition(ctx);
            }
        }
        cursor.Decrement(ctx);
    }
    last_visible.DeprecatedComputePosition(ctx)
}
pub fn MostForwardCaretPosition<C: VisibleUnitsContext>(
    ctx: &C,
    position: &Position<C::Node>,
    rule: EditingBoundaryCrossingRule,
    client: SnapToClient,
) -> Position<C::Node> {
    let Some(start) = position.AnchorNode() else {
        return Position::default();
    };
    let boundary = EnclosingVisualBoundary(ctx, start);
    let adjusted = if position.AnchorType() == super::position::PositionAnchorType::AfterAnchor {
        Position::EditingPositionOf(ctx, start.clone(), ctx.CaretMaxOffset(start))
    } else {
        position.clone()
    };
    let mut last_visible = PositionIterator::new(ctx, &adjusted);
    let mut last_node = if rule == EditingBoundaryCrossingRule::CannotCross
        && client == SnapToClient::LocalCaretRect
    {
        position
            .ComputeContainerNode(ctx)
            .unwrap_or_else(|| start.clone())
    } else {
        start.clone()
    };
    let start_editable = ctx.IsEditable(&last_node);
    let mut crossed = false;
    let mut writing_mode = None;
    let mut cursor = last_visible.clone();
    while !cursor.AtEnd(ctx) {
        let current = cursor.GetNode().expect("nonterminal source cursor").clone();
        if current != last_node {
            if start_editable != ctx.IsEditable(&current) {
                if rule == EditingBoundaryCrossingRule::CannotCross
                    && client != SnapToClient::LocalCaretRect
                {
                    break;
                }
                crossed = true;
            }
            last_node = current.clone();
        }
        if ctx.IsBody(&current) && cursor.AtEndOfNode(ctx) {
            break;
        }
        if !ctx.CanHaveCaretPosition(&current) {
            if crossed && rule == EditingBoundaryCrossingRule::CannotCross {
                break;
            }
            cursor.Increment(ctx);
            continue;
        }
        if (ctx.EndsAreVisuallyDistinct(&current) && boundary.as_ref() != Some(&current))
            || boundary.as_ref().and_then(|node| ctx.Parent(node)).as_ref() == Some(&current)
        {
            return last_visible.DeprecatedComputePosition(ctx);
        }
        let facts = ctx.CaretFacts(
            &current,
            if ctx.IsText(&current) {
                cursor.OffsetInTextNode()
            } else {
                0
            },
            false,
        );
        let Some(facts) = facts.filter(|facts| facts.visible && !facts.blocked_by_display_lock)
        else {
            if crossed && rule == EditingBoundaryCrossingRule::CannotCross {
                break;
            }
            cursor.Increment(ctx);
            continue;
        };
        if let Some(mode) = &writing_mode {
            if mode != &facts.writing_mode {
                return last_visible.ComputePosition(ctx);
            }
        } else {
            writing_mode = Some(facts.writing_mode);
        }
        if crossed {
            if rule == EditingBoundaryCrossingRule::CannotCross {
                return if &current == start {
                    position.clone()
                } else {
                    Position::BeforeNode(current)
                };
            }
            if rule == EditingBoundaryCrossingRule::CanCross {
                return cursor.DeprecatedComputePosition(ctx);
            }
        }
        if IsStreamer(ctx, &cursor) {
            last_visible = cursor.clone();
        }
        if ctx.EditingIgnoresContent(&current) || ctx.IsDisplayInsideTable(&current) {
            if cursor.AtStartOfNode(ctx) {
                return Position::EditingPositionOf(ctx, current, 0);
            }
            cursor.Increment(ctx);
            continue;
        }
        if let Some(text) = facts.text.filter(|text| text.has_non_collapsed_text) {
            if &current != start {
                return Position::new(current, text.min_offset + text.start_offset);
            }
            if text.before_non_collapsed_character {
                return cursor.ComputePosition(ctx);
            }
        }
        cursor.Increment(ctx);
    }
    last_visible.DeprecatedComputePosition(ctx)
}
fn AtEditingBoundary<C: VisibleUnitsContext>(ctx: &C, position: &Position<C::Node>) -> bool {
    let forward = ctx.BoundaryCaretPosition(position, EditingBoundaryCrossingRule::CanCross, false);
    let forward_noneditable = forward
        .AnchorNode()
        .is_some_and(|node| !ctx.IsEditable(node));
    if position.AtFirstEditingPositionForNode(ctx) && forward_noneditable {
        return true;
    }
    let backward = ctx.BoundaryCaretPosition(position, EditingBoundaryCrossingRule::CanCross, true);
    let backward_noneditable = backward
        .AnchorNode()
        .is_some_and(|node| !ctx.IsEditable(node));
    (position.AtLastEditingPositionForNode(ctx) && backward_noneditable)
        || (forward_noneditable && backward_noneditable)
}
pub fn IsVisuallyEquivalentCandidate<C: VisibleUnitsContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> bool {
    let Some(node) = position.AnchorNode() else {
        return false;
    };
    let Some(facts) = ctx
        .LayoutCaretFacts(node)
        .filter(|facts| facts.visible && !facts.blocked_by_display_lock)
    else {
        return false;
    };
    let parent_selectable = || {
        ctx.Parent(node)
            .is_some_and(|parent| ctx.HasLayoutObject(&parent) && ctx.IsSelectable(&parent))
    };
    if ctx.IsBR(node) {
        return position.AnchorType() != super::position::PositionAnchorType::AfterAnchor
            && position.ComputeEditingOffset(ctx) == 0
            && parent_selectable();
    }
    if facts.is_text {
        if !ctx.IsSelectable(node) {
            return false;
        }
        let offset = position.ComputeEditingOffset(ctx);
        let Some(text) = ctx
            .CaretFacts(node, offset, false)
            .and_then(|facts| facts.text)
        else {
            return false;
        };
        let Some(text_offset) = offset.checked_sub(text.start_offset) else {
            return false;
        };
        return text.contains_caret_offset
            && (text_offset == text.min_offset
                || text_offset
                    == ctx.NextGraphemeBoundary(
                        node,
                        ctx.PreviousGraphemeBoundary(node, text_offset),
                    ));
    }
    if ctx.IsSVG(node) {
        return false;
    }
    if ctx.IsDisplayInsideTable(node) || ctx.EditingIgnoresContent(node) {
        return (position.AtFirstEditingPositionForNode(ctx)
            || position.AtLastEditingPositionForNode(ctx))
            && parent_selectable();
    }
    if ctx.IsDocumentElement(node) || ctx.IsDocument(node) || !ctx.IsSelectable(node) {
        return false;
    }
    if ctx.IsBlockContainer(node) {
        if !ctx.HasLogicalHeight(node) && !ctx.IsBody(node) {
            return false;
        }
        if !ctx.HasRenderedNonAnonymousDescendantsWithHeight(node) {
            return position.AtFirstEditingPositionForNode(ctx);
        }
    }
    ctx.IsEditable(node) && AtEditingBoundary(ctx, position)
}
pub fn NextCandidate<C: VisibleUnitsContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    let mut cursor = PositionIterator::new(ctx, position);
    cursor.Increment(ctx);
    while !cursor.AtEnd(ctx) {
        let candidate = cursor.ComputePosition(ctx);
        if IsVisuallyEquivalentCandidate(ctx, &candidate) {
            return candidate;
        }
        cursor.Increment(ctx);
    }
    Position::default()
}
pub fn PreviousCandidate<C: VisibleUnitsContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> Position<C::Node> {
    let mut cursor = PositionIterator::new(ctx, position);
    cursor.Decrement(ctx);
    while !cursor.AtStart(ctx) {
        let candidate = cursor.ComputePosition(ctx);
        if IsVisuallyEquivalentCandidate(ctx, &candidate) {
            return candidate;
        }
        cursor.Decrement(ctx);
    }
    Position::default()
}
// Source editing_utilities.cc:1437-1466; used after native point projection.
pub fn AdjustForEditingBoundary<C: VisibleUnitsContext>(
    ctx: &C,
    value: PositionWithAffinity<C::Node>,
) -> PositionWithAffinity<C::Node> {
    if value.position.IsNull() {
        return value;
    }
    if value
        .position
        .ComputeContainerNode(ctx)
        .is_some_and(|node| ctx.IsEditable(&node))
    {
        return value;
    }
    let adjusted = if value
        .position
        .AnchorNode()
        .is_some_and(|node| ctx.IsEditable(node))
    {
        value.position.ToOffsetInAnchor(ctx)
    } else {
        value.position
    };
    let forward =
        ctx.BoundaryCaretPosition(&adjusted, EditingBoundaryCrossingRule::CanCross, false);
    if forward
        .ComputeContainerNode(ctx)
        .is_some_and(|node| ctx.IsEditable(&node))
    {
        return PositionWithAffinity {
            position: forward,
            affinity: TextAffinity::Downstream,
        };
    }
    let backward =
        ctx.BoundaryCaretPosition(&adjusted, EditingBoundaryCrossingRule::CanCross, true);
    if backward
        .ComputeContainerNode(ctx)
        .is_some_and(|node| ctx.IsEditable(&node))
    {
        return PositionWithAffinity {
            position: backward,
            affinity: TextAffinity::Downstream,
        };
    }
    PositionWithAffinity {
        position: adjusted,
        affinity: value.affinity,
    }
}
