// Copyright 2026 The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: menu_safe_triangle.h:38-137; .cc:25-391.
// Geometry and deferred actions remain on the document's canonical rare owner;
// the source algorithm lends each individual fact across the package boundary.
use foundation::{
    gfx::{PointF, QuadF},
    popup_types::*,
};

#[derive(Clone, Copy)]
pub struct TriangleFacts {
    pub invoker: u64,
    pub submenu: u64,
    pub triangle: QuadF,
    pub submenu_quad: QuadF,
}

pub trait MenuSafeTriangleEnvironment {
    type Error;
    fn OwningMenuIsList(&self, invoker: u64) -> bool;
    fn LastMousePosition(&self) -> Option<PointF>;
    fn UpdateStyleAndLayoutTreeForElement(&self, node: u64);
    fn AbsoluteQuads(&self, node: u64) -> Result<Option<Vec<QuadF>>, Self::Error>;
    fn Create(
        &self,
        invoker: u64,
        submenu: u64,
        triangle: QuadF,
        submenu_quad: QuadF,
    ) -> Result<MenuTriangleId, Self::Error>;
    fn SetActive(&self, triangle: Option<MenuTriangleId>);
    fn Active(&self) -> Option<MenuTriangleId>;
    fn Facts(&self, triangle: MenuTriangleId) -> Option<TriangleFacts>;
    fn MenuStackLen(&self) -> usize;
    fn MenuStackAt(&self, index: usize) -> u64;
    fn IsMenuList(&self, node: u64) -> bool;
    fn InclusiveFlatAncestor(&self, ancestor: u64, node: u64) -> bool;
    fn RemoveLosses(&self, triangle: MenuTriangleId, invoker: u64) -> bool;
    fn RemoveGains(&self, triangle: MenuTriangleId, invoker: u64) -> bool;
    fn UpdateGain(&self, triangle: MenuTriangleId, invoker: u64, state: InterestState) -> bool;
    fn UpdateLoss(
        &self,
        triangle: MenuTriangleId,
        invoker: u64,
        cancelable: InterestLostCancelable,
        behavior: InterestLostPopoverBehavior,
    ) -> bool;
    fn PushGain(&self, triangle: MenuTriangleId, gain: DeferredInterestGain);
    fn PushLoss(&self, triangle: MenuTriangleId, loss: DeferredInterestLoss);
    fn LossAt(&self, triangle: MenuTriangleId, index: usize) -> Option<DeferredInterestLoss>;
    fn GainAt(&self, triangle: MenuTriangleId, index: usize) -> Option<DeferredInterestGain>;
    fn InterestLost(&self, loss: DeferredInterestLoss) -> Result<(), Self::Error>;
    fn InterestGained(&self, gain: DeferredInterestGain) -> Result<(), Self::Error>;
    fn Clear(&self, triangle: MenuTriangleId, from_timer: bool);
}

// Source: .cc:43-167. The center/padding and closest finite edge calculation
// preserve gfx::Vector2dF's single precision rather than using a new polygon.
pub fn MaybeCreate<E: MenuSafeTriangleEnvironment + ?Sized>(
    e: &E,
    invoker: u64,
    submenu: u64,
) -> Result<(), E::Error> {
    if !e.OwningMenuIsList(invoker) {
        return Ok(());
    }
    let Some(mouse) = e.LastMousePosition() else {
        return Ok(());
    };
    e.UpdateStyleAndLayoutTreeForElement(submenu);
    let Some(item_quads) = e.AbsoluteQuads(invoker)? else {
        return Ok(());
    };
    let Some(submenu_quads) = e.AbsoluteQuads(submenu)? else {
        return Ok(());
    };
    if !item_quads.iter().any(|quad| quad.Contains(mouse)) || submenu_quads.len() != 1 {
        return Ok(());
    }
    let submenu_quad = submenu_quads[0];
    let edges = [
        (*submenu_quad.p1(), *submenu_quad.p2()),
        (*submenu_quad.p2(), *submenu_quad.p3()),
        (*submenu_quad.p3(), *submenu_quad.p4()),
        (*submenu_quad.p4(), *submenu_quad.p1()),
    ];
    let distance = |(p1, p2): (PointF, PointF)| {
        let a = (p1.x() - mouse.x(), p1.y() - mouse.y());
        let b = (p2.x() - mouse.x(), p2.y() - mouse.y());
        let mut direction = (p2.x() - p1.x(), p2.y() - p1.y());
        let length = (direction.0 * direction.0 + direction.1 * direction.1).sqrt();
        if length != 0.0 {
            direction.0 /= length;
            direction.1 /= length;
        }
        let dot_a = a.0 * direction.0 + a.1 * direction.1;
        let dot_b = b.0 * direction.0 + b.1 * direction.1;
        if dot_a.is_sign_negative() == dot_b.is_sign_negative() {
            (a.0 * a.0 + a.1 * a.1)
                .sqrt()
                .min((b.0 * b.0 + b.1 * b.1).sqrt())
        } else {
            let perpendicular = (a.0 - direction.0 * dot_a, a.1 - direction.1 * dot_a);
            (perpendicular.0 * perpendicular.0 + perpendicular.1 * perpendicular.1).sqrt()
        }
    };
    let mut closest = 0;
    let mut minimum = distance(edges[0]);
    for (index, edge) in edges.iter().enumerate().skip(1) {
        let value = distance(*edge);
        if value < minimum {
            minimum = value;
            closest = index;
        }
    }
    let mut triangle = QuadF::new(mouse, mouse, edges[closest].0, edges[closest].1);
    let center = triangle.CenterPoint();
    let pad = |p: PointF| {
        let mut offset = (p.x() - center.x(), p.y() - center.y());
        let length = (offset.0 * offset.0 + offset.1 * offset.1).sqrt();
        if length != 0.0 {
            let scale = 10.0 / length;
            offset.0 *= scale;
            offset.1 *= scale;
        }
        PointF::new(p.x() + offset.0, p.y() + offset.1)
    };
    triangle.set_p1(pad(*triangle.p1()));
    triangle.set_p2(pad(*triangle.p2()));
    triangle.set_p3(pad(*triangle.p3()));
    triangle.set_p4(pad(*triangle.p4()));
    let owner = e.Create(invoker, submenu, triangle, submenu_quad)?;
    e.SetActive(Some(owner));
    Ok(())
}

// Source .cc:169-227.
pub fn Recheck<E: MenuSafeTriangleEnvironment + ?Sized>(
    e: &E,
    id: MenuTriangleId,
) -> Result<(), E::Error> {
    assert_eq!(e.Active(), Some(id));
    let facts = e
        .Facts(id)
        .expect("active menu triangle retains its original nodes");
    let index = (0..e.MenuStackLen()).find(|&index| e.MenuStackAt(index) == facts.submenu);
    let Some(index) = index else {
        return Finish(e, id, false);
    };
    if ((index + 1)..e.MenuStackLen()).any(|index| e.IsMenuList(e.MenuStackAt(index))) {
        return Finish(e, id, false);
    }
    if let Some(mouse) = e.LastMousePosition() {
        if facts.triangle.Contains(mouse) || facts.submenu_quad.Contains(mouse) {
            return Ok(());
        }
    }
    Finish(e, id, false)
}

// Source .cc:272-295. The current triangle is detached before replaying the
// source deferred losses then gains. Those events may create a new owner.
pub fn Finish<E: MenuSafeTriangleEnvironment + ?Sized>(
    e: &E,
    id: MenuTriangleId,
    from_timer: bool,
) -> Result<(), E::Error> {
    assert_eq!(e.Active(), Some(id));
    e.SetActive(None);
    let mut index = 0;
    while let Some(loss) = e.LossAt(id, index) {
        e.InterestLost(loss)?;
        index += 1;
    }
    let mut index = 0;
    while let Some(gain) = e.GainAt(id, index) {
        e.InterestGained(gain)?;
        index += 1;
    }
    e.Clear(id, from_timer);
    Ok(())
}

// Source .cc:301-331.
pub fn ShouldDeferInterestGained<E: MenuSafeTriangleEnvironment + ?Sized>(
    e: &E,
    id: MenuTriangleId,
    invoker: u64,
    target: u64,
    state: InterestState,
) -> Result<bool, E::Error> {
    if e.RemoveLosses(id, invoker) {
        return Ok(true);
    }
    let facts = e.Facts(id).expect("active triangle retains nodes");
    if e.InclusiveFlatAncestor(facts.submenu, invoker) {
        Finish(e, id, false)?;
        return Ok(false);
    }
    if e.InclusiveFlatAncestor(facts.invoker, invoker) {
        return Ok(false);
    }
    if e.UpdateGain(id, invoker, state) {
        return Ok(true);
    }
    e.PushGain(
        id,
        DeferredInterestGain {
            invoker,
            target,
            state,
        },
    );
    Ok(true)
}

// Source .cc:333-389.
pub fn ShouldDeferInterestLost<E: MenuSafeTriangleEnvironment + ?Sized>(
    e: &E,
    id: MenuTriangleId,
    invoker: u64,
    target: u64,
    cancelable: InterestLostCancelable,
    behavior: InterestLostPopoverBehavior,
) -> Result<bool, E::Error> {
    if e.RemoveGains(id, invoker) {
        return Ok(true);
    }
    let facts = e.Facts(id).expect("active triangle retains nodes");
    let index = (0..e.MenuStackLen()).find(|&index| e.MenuStackAt(index) == facts.submenu);
    let can_defer = index.is_some_and(|index| {
        (0..=index)
            .rev()
            .any(|index| e.MenuStackAt(index) == target)
    });
    if !can_defer {
        return Ok(false);
    }
    if e.UpdateLoss(id, invoker, cancelable, behavior) {
        return Ok(true);
    }
    e.PushLoss(
        id,
        DeferredInterestLoss {
            invoker,
            target,
            cancelable,
            behavior,
        },
    );
    Ok(true)
}
