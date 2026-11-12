// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/compare_positions.cc:39-427,438-466.
use super::{
    position::{Position, PositionTree},
    position_conversion::PositionTreeScope,
};
use std::cmp::Ordering;

enum Offset<N> {
    Int(usize),
    ChildBefore(Option<N>),
}
fn CompareNodes<C: PositionTree>(
    ctx: &C,
    first: Option<C::Node>,
    second: Option<C::Node>,
) -> Ordering {
    if first == second {
        return Ordering::Equal;
    }
    let Some(first) = first else {
        return Ordering::Less;
    };
    let Some(second) = second else {
        return Ordering::Greater;
    };
    let (mut back_a, mut forward_a, mut back_b, mut forward_b) =
        (first.clone(), first, second.clone(), second);
    loop {
        let Some(next) = ctx.PreviousSibling(&back_a) else {
            return Ordering::Less;
        };
        back_a = next;
        if back_a == forward_b {
            return Ordering::Greater;
        }
        let Some(next) = ctx.NextSibling(&forward_a) else {
            return Ordering::Greater;
        };
        forward_a = next;
        if forward_a == back_b {
            return Ordering::Less;
        }
        let Some(next) = ctx.PreviousSibling(&back_b) else {
            return Ordering::Greater;
        };
        back_b = next;
        if forward_a == back_b {
            return Ordering::Less;
        }
        let Some(next) = ctx.NextSibling(&forward_b) else {
            return Ordering::Less;
        };
        forward_b = next;
        if back_a == forward_b {
            return Ordering::Greater;
        }
    }
}
fn CompareIntToNode<C: PositionTree>(
    ctx: &C,
    offset: usize,
    before: &Option<C::Node>,
    equal: Ordering,
) -> Ordering {
    let Some(before) = before else {
        return if offset == 0 {
            equal
        } else {
            Ordering::Greater
        };
    };
    if offset == 0 {
        return Ordering::Less;
    }
    let parent = ctx.Parent(before).expect("child offset has parent");
    for index in 0..ctx.ChildCount(&parent) {
        let child = ctx.ChildAt(&parent, index).expect("source child");
        if offset == index + 1 {
            return if &child == before {
                equal
            } else {
                Ordering::Less
            };
        }
        if &child == before {
            return Ordering::Greater;
        }
    }
    unreachable!("child offset belongs to its canonical parent")
}
fn CompareOffsets<C: PositionTree>(
    ctx: &C,
    a: &Offset<C::Node>,
    b: &Offset<C::Node>,
    equal: Ordering,
) -> Ordering {
    match (a, b) {
        (Offset::Int(a), Offset::Int(b)) => {
            if a == b {
                equal
            } else {
                a.cmp(b)
            }
        }
        (Offset::Int(a), Offset::ChildBefore(b)) => CompareIntToNode(ctx, *a, b, equal),
        (Offset::ChildBefore(a), Offset::Int(b)) => {
            CompareIntToNode(ctx, *b, a, equal.reverse()).reverse()
        }
        (Offset::ChildBefore(a), Offset::ChildBefore(b)) => {
            let order = CompareNodes(ctx, a.clone(), b.clone());
            if order == Ordering::Equal {
                equal
            } else {
                order
            }
        }
    }
}
fn ChildInAncestors<C: PositionTree>(ctx: &C, node: &C::Node, parent: &C::Node) -> Option<C::Node> {
    let mut child = node.clone();
    while let Some(ancestor) = ctx.Parent(&child) {
        if &ancestor == parent {
            return Some(child);
        }
        child = ancestor;
    }
    None
}
pub fn CommonAncestor<C: PositionTree>(
    ctx: &C,
    first: &C::Node,
    second: &C::Node,
) -> Option<C::Node> {
    let depth = |node: &C::Node| {
        let mut depth = 0;
        let mut node = Some(node.clone());
        while let Some(current) = node {
            depth += 1;
            node = ctx.Parent(&current);
        }
        depth
    };
    let (mut a, mut b) = (Some(first.clone()), Some(second.clone()));
    let (mut da, mut db) = (depth(first), depth(second));
    while da > db {
        a = a.as_ref().and_then(|node| ctx.Parent(node));
        da -= 1;
    }
    while db > da {
        b = b.as_ref().and_then(|node| ctx.Parent(node));
        db -= 1;
    }
    while a != b {
        a = a.as_ref().and_then(|node| ctx.Parent(node));
        b = b.as_ref().and_then(|node| ctx.Parent(node));
    }
    a
}
fn Compare<C: PositionTree>(
    ctx: &C,
    a: &C::Node,
    oa: Offset<C::Node>,
    b: &C::Node,
    ob: Offset<C::Node>,
) -> Option<Ordering> {
    if a == b {
        return Some(CompareOffsets(ctx, &oa, &ob, Ordering::Equal));
    }
    if let Some(child) = ChildInAncestors(ctx, b, a) {
        return Some(CompareOffsets(
            ctx,
            &oa,
            &Offset::ChildBefore(ctx.PreviousSibling(&child)),
            Ordering::Less,
        ));
    }
    if let Some(child) = ChildInAncestors(ctx, a, b) {
        return Some(
            CompareOffsets(
                ctx,
                &ob,
                &Offset::ChildBefore(ctx.PreviousSibling(&child)),
                Ordering::Less,
            )
            .reverse(),
        );
    }
    let ancestor = CommonAncestor(ctx, a, b)?;
    let last = || {
        ctx.ChildCount(&ancestor)
            .checked_sub(1)
            .and_then(|index| ctx.ChildAt(&ancestor, index))
    };
    Some(CompareNodes(
        ctx,
        ChildInAncestors(ctx, a, &ancestor).or_else(last),
        ChildInAncestors(ctx, b, &ancestor).or_else(last),
    ))
}
fn PositionOffset<C: PositionTree>(ctx: &C, position: &Position<C::Node>) -> Offset<C::Node> {
    if position.IsOffsetInAnchor() {
        Offset::Int(position.ComputeOffsetInContainerNode(ctx))
    } else {
        Offset::ChildBefore(position.ComputeNodeBeforePosition(ctx))
    }
}
pub fn CompareFlatPositions<C: PositionTree>(
    ctx: &C,
    a: &Position<C::Node>,
    b: &Position<C::Node>,
) -> Option<Ordering> {
    Compare(
        ctx,
        &a.ComputeContainerNode(ctx)?,
        PositionOffset(ctx, a),
        &b.ComputeContainerNode(ctx)?,
        PositionOffset(ctx, b),
    )
}
fn ScopeParent<C: PositionTreeScope>(ctx: &C, scope: &C::Node) -> Option<C::Node> {
    ctx.ShadowHost(scope).map(|host| ctx.TreeScopeRoot(&host))
}
pub fn AncestorInScope<C: PositionTreeScope>(
    ctx: &C,
    node: &C::Node,
    scope: &C::Node,
) -> Option<C::Node> {
    let mut node = node.clone();
    while &ctx.TreeScopeRoot(&node) != scope {
        node = ctx.ShadowHost(&ctx.TreeScopeRoot(&node))?;
    }
    Some(node)
}
pub fn CompareDOMPositions<C: PositionTreeScope>(
    ctx: &C,
    a: &Position<C::Node>,
    b: &Position<C::Node>,
) -> Option<Ordering> {
    let ca = a.ComputeContainerNode(ctx)?;
    let cb = b.ComputeContainerNode(ctx)?;
    let mut common_scope = Some(ctx.TreeScopeRoot(&ca));
    let common_scope = loop {
        let scope = common_scope?;
        if AncestorInScope(ctx, &cb, &scope).is_some() {
            break scope;
        }
        common_scope = ScopeParent(ctx, &scope);
    };
    let na = AncestorInScope(ctx, &ca, &common_scope)?;
    let nb = AncestorInScope(ctx, &cb, &common_scope)?;
    let da = na != ca;
    let db = nb != cb;
    let offset = |position: &Position<C::Node>, has_descendant: bool| {
        if position.IsOffsetInAnchor() && !has_descendant {
            Offset::Int(position.ComputeOffsetInContainerNode(ctx))
        } else if position.IsOffsetInAnchor() || has_descendant {
            Offset::ChildBefore(None)
        } else {
            Offset::ChildBefore(position.ComputeNodeBeforePosition(ctx))
        }
    };
    let result = Compare(ctx, &na, offset(a, da), &nb, offset(b, db))?;
    if result != Ordering::Equal {
        return Some(result);
    }
    Some(if na != nb {
        Ordering::Equal
    } else if da {
        Ordering::Greater
    } else if db {
        Ordering::Less
    } else {
        Ordering::Equal
    })
}
