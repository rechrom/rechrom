// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkEdge::SkEdge;
// Source: src/core/SkScanPriv.h:16-16
pub const SK_SUPERSAMPLE_SHIFT: i32 = 2;
// Source: src/core/SkScanPriv.h:18-34
// Adapted representation: a tagged owner avoids self-referential wrapper pointers.
// Constructor selection and wrapper behavior follow the native release branches.
pub(crate) enum ScanBlitter<'a> {
    None,
    Direct(core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>),
    #[cfg(sk_debug)]
    Check(crate::src::core::SkBlitter::SkRectClipCheckBlitter<'a>),
    Rect(crate::src::core::SkBlitter::SkRectClipBlitter<'a>),
    Region(crate::src::core::SkBlitter::SkRgnClipBlitter<'a>),
}
pub struct SkScanClipper<'a> {
    pub(crate) blitter: ScanBlitter<'a>,
    pub(crate) clip_rect: Option<crate::src::core::SkRect::SkIRect>,
}
impl<'a> SkScanClipper<'a> {
    pub fn getBlitter(
        self: core::pin::Pin<&mut Self>,
    ) -> Option<core::pin::Pin<&mut (dyn crate::src::core::SkBlitter::SkBlitter + '_)>> {
        // Tagged payload cannot change while pinned; each arm projects its
        // original inline owner, or reborrows the original pinned destination.
        let this = unsafe { self.get_unchecked_mut() };
        match &mut this.blitter {
            ScanBlitter::None => None,
            ScanBlitter::Direct(b) => Some(b.as_mut()),
            #[cfg(sk_debug)]
            ScanBlitter::Check(b) => Some(unsafe { core::pin::Pin::new_unchecked(b) }),
            ScanBlitter::Rect(b) => Some(unsafe { core::pin::Pin::new_unchecked(b) }),
            ScanBlitter::Region(b) => Some(unsafe { core::pin::Pin::new_unchecked(b) }),
        }
    }
    pub fn getClipRect(&self) -> Option<&crate::src::core::SkRect::SkIRect> {
        self.clip_rect.as_ref()
    }
}

// Rust field access abstraction for the upstream EdgeType template. Each
// implementation keeps the concrete owner's intrusive links and x coordinate.
pub unsafe trait LinkedEdge: Sized {
    unsafe fn next(edge: *mut Self) -> *mut Self;
    unsafe fn prev(edge: *mut Self) -> *mut Self;
    unsafe fn set_next(edge: *mut Self, value: *mut Self);
    unsafe fn set_prev(edge: *mut Self, value: *mut Self);
    unsafe fn x(edge: *mut Self) -> i32;
}
unsafe impl LinkedEdge for SkEdge {
    unsafe fn next(edge: *mut Self) -> *mut Self {
        (*edge).fNext
    }
    unsafe fn prev(edge: *mut Self) -> *mut Self {
        (*edge).fPrev
    }
    unsafe fn set_next(edge: *mut Self, value: *mut Self) {
        (*edge).fNext = value;
    }
    unsafe fn set_prev(edge: *mut Self, value: *mut Self) {
        (*edge).fPrev = value;
    }
    unsafe fn x(edge: *mut Self) -> i32 {
        (*edge).fX
    }
}
// Source: src/core/SkScanPriv.h:40-44
/// Safety: edge and its previous/next links must be live nodes in the same list.
pub unsafe fn remove_edge<E: LinkedEdge>(edge: *mut E) {
    E::set_next(E::prev(edge), E::next(edge));
    E::set_prev(E::next(edge), E::prev(edge));
}
// Source: src/core/SkScanPriv.h:46-52
/// Safety: nodes must be live and after_me must have a live successor.
pub unsafe fn insert_edge_after<E: LinkedEdge>(edge: *mut E, after_me: *mut E) {
    E::set_prev(edge, after_me);
    E::set_next(edge, E::next(after_me));
    E::set_prev(E::next(after_me), edge);
    E::set_next(after_me, edge);
}
// Source: src/core/SkScanPriv.h:54-65
/// Safety: the list must be well formed and terminate at a live head sentinel.
pub unsafe fn backward_insert_edge_based_on_x<E: LinkedEdge>(edge: *mut E) {
    let x = E::x(edge);
    let mut prev = E::prev(edge);
    while !E::prev(prev).is_null() && E::x(prev) > x {
        prev = E::prev(prev);
    }
    if E::next(prev) != edge {
        remove_edge(edge);
        insert_edge_after(edge, prev);
    }
}
// Source: src/core/SkScanPriv.h:71-77
/// Safety: prev must be live in a list terminating at a head sentinel.
pub unsafe fn backward_insert_start<E: LinkedEdge>(mut prev: *mut E, x: i32) -> *mut E {
    while !E::prev(prev).is_null() && E::x(prev) > x {
        prev = E::prev(prev);
    }
    prev
}

unsafe impl LinkedEdge for crate::src::core::SkAnalyticEdge::SkAnalyticEdge {
    unsafe fn next(e: *mut Self) -> *mut Self {
        (*e).fNext
    }
    unsafe fn prev(e: *mut Self) -> *mut Self {
        (*e).fPrev
    }
    unsafe fn set_next(e: *mut Self, v: *mut Self) {
        (*e).fNext = v;
    }
    unsafe fn set_prev(e: *mut Self, v: *mut Self) {
        (*e).fPrev = v;
    }
    unsafe fn x(e: *mut Self) -> i32 {
        (*e).fX
    }
}
