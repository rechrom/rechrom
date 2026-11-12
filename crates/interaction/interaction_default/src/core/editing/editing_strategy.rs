// Copyright 2015 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: editing_strategy.cc:18-32,84-104.
use super::position::PositionTree;
use foundation::EUserSelect;
pub trait SelectionStyleContext: PositionTree {
    fn HasLayoutObject(&self, node: &Self::Node) -> bool;
    fn IsTextControlForUserSelect(&self, node: &Self::Node) -> bool;
    fn LayoutUsedValueOfUserSelect(&self, node: &Self::Node) -> Option<EUserSelect>;
}
pub fn RootUserSelectAllForNode<C: SelectionStyleContext>(
    ctx: &C,
    node: &C::Node,
) -> Option<C::Node> {
    let used = |node: &C::Node| {
        if ctx.IsTextControlForUserSelect(node) {
            EUserSelect::kText
        } else {
            ctx.LayoutUsedValueOfUserSelect(node)
                .unwrap_or(EUserSelect::kNone)
        }
    };
    if used(node) != EUserSelect::kAll {
        return None;
    }
    let mut root = node.clone();
    let mut parent = ctx.Parent(node);
    while let Some(node) = parent {
        if !ctx.HasLayoutObject(&node) {
            parent = ctx.Parent(&node);
            continue;
        }
        if used(&node) != EUserSelect::kAll {
            break;
        }
        parent = ctx.Parent(&node);
        root = node;
    }
    Some(root)
}
