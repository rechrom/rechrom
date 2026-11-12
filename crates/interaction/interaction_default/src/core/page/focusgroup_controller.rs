// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/page/focusgroup_controller.h:25-44
// Source: third_party/blink/renderer/core/page/focusgroup_controller.cc:30-385
use super::{
    focus_navigation::FocusNavigationNode as Node,
    focusgroup_controller_utils::{
        self as utils, DirectionalFocusgroupTree, FocusgroupDirection as D, FocusgroupItemPosition,
        FocusgroupType,
    },
    grid_focusgroup_structure_info::{GridQuery as Q, GridResult},
};
use foundation::focusgroup_flags::FocusgroupFlags as F;
use interaction_dispatch::core::events::keyboard_event::KeyboardEventRef;

pub trait FocusgroupContext: DirectionalFocusgroupTree {
    fn CaretBrowsingEnabled(&self) -> bool;
    fn HasEditContext(&self, node: Node) -> bool;
    fn IsEditable(&self, node: Node) -> bool;
    fn UpdateStyleAndLayoutTreeForElement(&self, node: Node);
    fn UpdateAllLifecyclePhasesExceptPaint(&self, node: Node);
    fn Grid(&self, root: Node, query: Q) -> GridResult;
    fn Focus(&self, node: Node, direction: D);
}

pub fn HandleKeyboardEvent(context: &dyn FocusgroupContext, event: &KeyboardEventRef) -> bool {
    if !event.borrow().isTrusted() || !context.Enabled() {
        return false;
    }
    HandleArrowKeyboardEvent(context, event) || HandleHomeEndKeyboardEvent(context, event)
}
pub fn HandleArrowKeyboardEvent(context: &dyn FocusgroupContext, event: &KeyboardEventRef) -> bool {
    assert!(context.Enabled());
    let Some(focused) = CurrentTarget(context, event) else {
        return false;
    };
    let direction = {
        let event = event.borrow();
        utils::FocusgroupDirectionForEvent(
            context,
            focused,
            event.key(),
            event.ctrlKey() || event.metaKey() || event.shiftKey() || event.altKey(),
        )
    };
    if direction == D::kNone || utils::IsInDirectionalKeyHandler(context, focused, direction) {
        return false;
    }
    Advance(context, focused, direction)
}
pub fn HandleHomeEndKeyboardEvent(
    context: &dyn FocusgroupContext,
    event: &KeyboardEventRef,
) -> bool {
    assert!(context.Enabled());
    let Some(focused) = CurrentTarget(context, event) else {
        return false;
    };
    let home = {
        let event = event.borrow();
        if !matches!(event.key(), "Home" | "End")
            || context.CaretBrowsingEnabled()
            || event.ctrlKey()
            || event.metaKey()
            || event.shiftKey()
            || event.altKey()
        {
            return false;
        }
        event.key() == "Home"
    };
    if context.HasEditContext(focused) {
        return false;
    }
    let Some(owner) =
        utils::FindNearestFocusgroupAncestor(context, focused, FocusgroupType::kLinear)
    else {
        return false;
    };
    if utils::GetFocusgroupOwnerOfItem(context, focused) != Some(owner) {
        return false;
    }
    context.UpdateStyleAndLayoutTreeForElement(focused);
    let Some(focused) = CurrentTarget(context, event) else {
        return false;
    };
    if utils::GetFocusgroupOwnerOfItem(context, focused) != Some(owner)
        || context.IsEditable(focused)
        || utils::IsInDirectionalKeyHandlerForAnyAxis(context, focused, owner)
    {
        return false;
    }
    let Some(target) = utils::FocusgroupItemWithin(
        context,
        owner,
        if home {
            FocusgroupItemPosition::kFirst
        } else {
            FocusgroupItemPosition::kLast
        },
    )
    .filter(|&target| target != focused) else {
        return false;
    };
    context.Focus(
        target,
        if home {
            D::kBackwardInline
        } else {
            D::kForwardInline
        },
    );
    true
}
fn CurrentTarget(context: &dyn FocusgroupContext, event: &KeyboardEventRef) -> Option<Node> {
    let focused = context.FocusedElement()?;
    (event.borrow().RawTarget()?.NodeId() == Some(focused.0)).then_some(focused)
}
pub fn Advance(context: &dyn FocusgroupContext, initial: Node, direction: D) -> bool {
    if context.V2Enabled() {
        if let Some(grid) =
            utils::FindNearestFocusgroupAncestor(context, initial, FocusgroupType::kGrid)
        {
            if utils::IsGridFocusgroupItem(context, initial) {
                return AdvanceInGrid(context, initial, grid, direction);
            }
        }
    }
    let Some(owner) =
        utils::FindNearestFocusgroupAncestor(context, initial, FocusgroupType::kLinear)
    else {
        return false;
    };
    if utils::GetFocusgroupOwnerOfItem(context, initial) != Some(owner) {
        return false;
    }
    let flags = context.Data(owner).flags;
    if !utils::IsAxisSupported(flags, direction) {
        return false;
    }
    if let Some(candidate) =
        utils::NextFocusgroupItemInDirection(context, owner, initial, direction)
    {
        context.Focus(candidate, direction);
        return true;
    }
    if !utils::WrapsInDirection(flags, direction) {
        return false;
    }
    let Some(candidate) = utils::WrappedFocusgroupCandidate(context, owner, initial, direction)
    else {
        return false;
    };
    context.Focus(candidate, direction);
    true
}
fn GridNode(context: &dyn FocusgroupContext, root: Node, query: Q) -> Option<Node> {
    let GridResult::Node(node) = context.Grid(root, query) else {
        panic!("grid node query response");
    };
    node
}
fn GridIndex(context: &dyn FocusgroupContext, root: Node, query: Q) -> u32 {
    let GridResult::Index(index) = context.Grid(root, query) else {
        panic!("grid index query response");
    };
    index
}
pub fn AdvanceInGrid(
    context: &dyn FocusgroupContext,
    initial: Node,
    root: Node,
    direction: D,
) -> bool {
    context.UpdateAllLifecyclePhasesExceptPaint(root);
    let mut current = initial;
    loop {
        let query = match direction {
            D::kBackwardInline => Q::PreviousCellInRow(current),
            D::kForwardInline => Q::NextCellInRow(current),
            D::kBackwardBlock => Q::PreviousCellInColumn(current),
            D::kForwardBlock => Q::NextCellInColumn(current),
            D::kNone => unreachable!(),
        };
        let Some(next) = GridNode(context, root, query)
            .or_else(|| WrapOrFlowInGrid(context, current, root, direction))
        else {
            break;
        };
        current = next;
        if current == initial {
            break;
        }
        if utils::IsGridFocusgroupItem(context, current) && context.IsKeyboardFocusable(current) {
            context.Focus(current, direction);
            return true;
        }
    }
    false
}
pub fn WrapOrFlowInGrid(
    context: &dyn FocusgroupContext,
    element: Node,
    root: Node,
    direction: D,
) -> Option<Node> {
    let flags = context.Data(root).flags;
    let node = |query| GridNode(context, root, query);
    match direction {
        D::kBackwardInline => {
            if flags.Intersects(F::kWrapInline) {
                return node(Q::LastCellInRow(node(Q::RowForCell(element))?));
            }
            if flags.Intersects(F::kRowFlow) {
                let row = node(Q::RowForCell(element))?;
                let row = node(Q::PreviousRow(row)).or_else(|| node(Q::LastRow))?;
                return node(Q::LastCellInRow(row));
            }
        }
        D::kForwardInline => {
            if flags.Intersects(F::kWrapInline) {
                return node(Q::FirstCellInRow(node(Q::RowForCell(element))?));
            }
            if flags.Intersects(F::kRowFlow) {
                let row = node(Q::RowForCell(element))?;
                let row = node(Q::NextRow(row)).or_else(|| node(Q::FirstRow))?;
                return node(Q::FirstCellInRow(row));
            }
        }
        D::kBackwardBlock => {
            if flags.Intersects(F::kWrapBlock) {
                return node(Q::LastCellInColumn(GridIndex(
                    context,
                    root,
                    Q::ColumnIndexForCell(element),
                )));
            }
            if flags.Intersects(F::kColFlow) {
                let mut column = GridIndex(context, root, Q::ColumnIndexForCell(element));
                if column == 0 {
                    column = GridIndex(context, root, Q::ColumnCount);
                }
                return node(Q::LastCellInColumn(column.wrapping_sub(1)));
            }
        }
        D::kForwardBlock => {
            if flags.Intersects(F::kWrapBlock) {
                return node(Q::FirstCellInColumn(GridIndex(
                    context,
                    root,
                    Q::ColumnIndexForCell(element),
                )));
            }
            if flags.Intersects(F::kColFlow) {
                let mut column =
                    GridIndex(context, root, Q::ColumnIndexForCell(element)).wrapping_add(1);
                if column >= GridIndex(context, root, Q::ColumnCount) {
                    column = 0;
                }
                return node(Q::FirstCellInColumn(column));
            }
        }
        D::kNone => unreachable!(),
    }
    None
}
