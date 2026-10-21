use crate::src::core::SkBlitter::SkBlitter;
// Pin adaptation: concrete blitters inherit !Unpin from their actual embedded
// SkBlitterStorage. Scan entrypoints receive the original pinned destination;
// local polymorphic owners are pinned before dispatch and never moved afterward.
// get_unchecked_mut only projects existing fields; it never extracts/replaces
// pinned storage or transfers an installed arena/inline allocation.
use crate::src::core::SkRect::SkIRect;
use crate::src::core::SkRegion::SkRegion;
use crate::src::core::SkRegionPriv::RUN_TYPE_SENTINEL;
use std::mem::MaybeUninit;

// Source: src/core/SkRegion_path.cpp:35-44
fn memeq32(a: &[i32], b: &[i32], count: usize) -> bool {
    for i in 0..count {
        if a[i] != b[i] {
            return false;
        }
    }
    true
}

// Source: src/core/SkRegion_path.cpp:54-62
impl SkRgnBuilder {
    pub fn done(&mut self) {
        if let Some(curr) = self.curr {
            self.set(curr + 1, (self.x_ptr - (curr + 2)) as i32);
            if !self.collapse_with_prev() {
                self.curr = Some(self.next_scanline(curr));
            }
        }
    }
}

// Source: src/core/SkRegion_path.cpp:68-70
// Safety: each blitter owns SkBlitterStorage for its full lifetime; allocation
// pointers remain valid through the next reset or the owner's drop.
unsafe impl crate::src::core::SkBlitter::SkBlitter for SkRgnBuilder {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.storage_base.allocBlitMemory(sz)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe { this.blit_h(x, y, width) }
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        _x: i32,
        _y: i32,
        _aa: &mut [u8],
        _runs: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        unsafe {
            // Upstream deliberately rejects AA callbacks: this builder is used
            // only by non-AA FillPath, whose contract calls blitH.
            crate::SkDEBUGFAIL!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 69; b"blitAntiH not implemented\0".as_ptr());
        }
    }
}

// Source: src/core/SkRegion_path.cpp:73-85
// Rust indices retain the original packed scanline walk and native diagnostic calls.
#[cfg(sk_debug)]
impl SkRgnBuilder {
    pub fn dump(&self) {
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 74; b"SkRgnBuilder: Top = %d\n\0".as_ptr(),self.top as core::ffi::c_int);
        }
        let mut line = 0;
        while line < self.curr.unwrap_or(0) {
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 77; b"SkRgnBuilder::Scanline: LastY=%d, fXCount=%d\0".as_ptr(),
                    self.get(line) as core::ffi::c_int,self.get(line+1) as core::ffi::c_int);
            }
            for i in 0..self.get(line + 1) {
                unsafe {
                    crate::SkDebugf!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 79; b" %d\0".as_ptr(),self.get(line+2+i as usize) as core::ffi::c_int);
                }
            }
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 81; b"\n\0".as_ptr());
            }
            line = self.next_scanline(line);
        }
    }
}

// Source: src/core/SkRegion_path.cpp:87-130
// The native packed [LastY XCount L R ... scratch] scanline layout is retained.
// Vec<MaybeUninit<i32>> owns allocation; indices replace interior pointers.
// Only fields initialized by the same source transition are ever read.
pub struct SkRgnBuilder {
    storage_base: crate::src::core::SkBlitter::SkBlitterStorage,
    storage: Vec<MaybeUninit<i32>>,
    curr: Option<usize>,
    prev: Option<usize>,
    x_ptr: usize,
    top: i32,
}
impl SkRgnBuilder {
    fn get(&self, i: usize) -> i32 {
        unsafe { self.storage[i].assume_init() }
    }
    fn set(&mut self, i: usize, value: i32) {
        self.storage[i].write(value);
    }
    fn next_scanline(&self, i: usize) -> usize {
        i + 2 + self.get(i + 1) as usize + 1
    }
    fn collapse_with_prev(&mut self) -> bool {
        if let (Some(prev), Some(curr)) = (self.prev, self.curr) {
            if self.get(prev) + 1 == self.get(curr) && self.get(prev + 1) == self.get(curr + 1) {
                let count = self.get(curr + 1) as usize;
                let mut same = true;
                for i in 0..count {
                    if self.get(prev + 2 + i) != self.get(curr + 2 + i) {
                        same = false;
                        break;
                    }
                }
                if same {
                    self.set(prev, self.get(curr));
                    return true;
                }
            }
        }
        false
    }
}

// Source: src/core/SkRegion_path.cpp:133-139
// C++ destroys derived owned storage before its SkBlitter base allocation.
impl Drop for SkRgnBuilder {
    fn drop(&mut self) {
        drop(core::mem::take(&mut self.storage));
    }
}
impl SkRgnBuilder {
    pub fn new() -> Self {
        Self {
            storage_base: crate::src::core::SkBlitter::SkBlitterStorage::new(),
            storage: Vec::new(),
            curr: None,
            prev: None,
            x_ptr: 0,
            top: 0,
        }
    }
}

// Source: src/core/SkRegion_path.cpp:141-177
impl SkRgnBuilder {
    pub fn init(&mut self, max_height: i32, mut max_transitions: i32, inverse: bool) -> bool {
        if max_height | max_transitions < 0 {
            return false;
        }
        if inverse {
            let Some(n) = max_transitions.checked_add(2) else {
                return false;
            };
            max_transitions = n;
        }
        let (Some(h), Some(t)) = (max_height.checked_add(1), max_transitions.checked_add(3)) else {
            return false;
        };
        let Some(mut count) = (h as usize).checked_mul(t as usize) else {
            return false;
        };
        if inverse {
            let Some(n) = count.checked_add(10) else {
                return false;
            };
            count = n;
        }
        if count > i32::MAX as usize {
            return false;
        }
        let mut storage = Vec::new();
        if storage.try_reserve_exact(count).is_err() {
            return false;
        }
        storage.resize_with(count, MaybeUninit::uninit);
        self.storage = storage;
        self.curr = None;
        self.prev = None;
        true
    }
}

// Source: src/core/SkRegion_path.cpp:179-217
impl SkRgnBuilder {
    pub fn blit_h(&mut self, x: i32, y: i32, width: i32) {
        if self.curr.is_none() {
            self.top = y;
            self.curr = Some(0);
            self.set(0, y);
            self.x_ptr = 2;
        } else {
            let curr = self.curr.unwrap();
            crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 186; @condition b"y >= fCurrScanline->fLastY\0".as_ptr(); y >= self.get(curr));
            if y > self.get(curr) {
                self.set(curr + 1, (self.x_ptr - (curr + 2)) as i32);
                let prev_y = self.get(curr);
                if !self.collapse_with_prev() {
                    self.prev = self.curr;
                    self.curr = Some(self.next_scanline(curr));
                }
                if y - 1 > prev_y {
                    let curr = self.curr.unwrap();
                    self.set(curr, y - 1);
                    self.set(curr + 1, 0);
                    self.curr = Some(self.next_scanline(curr));
                }
                let curr = self.curr.unwrap();
                self.set(curr, y);
                self.x_ptr = curr + 2;
            }
        }
        let curr = self.curr.unwrap();
        if self.x_ptr > curr + 2 && self.get(self.x_ptr - 1) == x {
            self.set(self.x_ptr - 1, x + width);
        } else {
            self.set(self.x_ptr, x);
            self.set(self.x_ptr + 1, x + width);
            self.x_ptr += 2;
        }
        crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 216; @condition b"fCurrXPtr - fStorage < fStorageCount\0".as_ptr(); self.x_ptr < self.storage.len());
    }
}

// Source: src/core/SkRegion_path.cpp:219-228
impl SkRgnBuilder {
    pub fn compute_run_count(&self) -> usize {
        self.curr.map_or(0, |curr| 2 + curr)
    }
}

// Source: src/core/SkRegion_path.cpp:230-239
impl SkRgnBuilder {
    pub fn copy_to_rect(&self, r: &mut SkIRect) {
        crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 233; @condition b"(const SkRegion::RunType*)fCurrScanline - fStorage == 5\0".as_ptr(); (self.curr) == ( Some(5)));
        crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 236; @condition b"line->fXCount == 2\0".as_ptr(); (self.get(1)) == ( 2));
        r.set_ltrb(self.get(2), self.top, self.get(3), self.get(0) + 1);
    }
}

// Source: src/core/SkRegion_path.cpp:241-262
impl SkRgnBuilder {
    fn copy_runs(&self, mut write: impl FnMut(usize, i32)) {
        let stop = self.curr.unwrap();
        crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 243; @condition b"(const SkRegion::RunType*)fCurrScanline - fStorage > 4\0".as_ptr(); stop > 4);
        let (mut line, mut p) = (0, 1);
        write(0, self.top);
        loop {
            write(p, self.get(line) + 1);
            let count = self.get(line + 1) as usize;
            write(p + 1, (count >> 1) as i32);
            p += 2;
            for i in 0..count {
                write(p + i, self.get(line + 2 + i));
            }
            p += count;
            write(p, RUN_TYPE_SENTINEL);
            p += 1;
            line = self.next_scanline(line);
            if line >= stop {
                break;
            }
        }
        crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 260; @condition b"line == stop\0".as_ptr(); (line) == (stop));
        write(p, RUN_TYPE_SENTINEL);
    }
    pub fn copy_to_region(&self, runs: &mut [i32]) {
        self.copy_runs(|index, value| runs[index] = value);
    }
    fn copy_to_native(&self, head: &mut crate::src::core::SkRegionPriv::RunHeadHandle) {
        self.copy_runs(|index, value| head.write_run(index, value));
        unsafe {
            head.assume_initialized();
        }
    }
}
// Source: src/core/SkRegion_path.cpp:264-276
fn verb_to_initial_last_index(verb: crate::include::core::SkPathTypes::SkPathVerb) -> usize {
    const INITIAL_LAST_INDEX: [u8; 6] = [0, 1, 2, 2, 3, 0];
    let index = verb as usize;
    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 274; @condition b"index < std::size(gPathVerbToInitialLastIndex)\0".as_ptr(); index < INITIAL_LAST_INDEX.len());
    INITIAL_LAST_INDEX[index] as usize
}
// Source: src/core/SkRegion_path.cpp:278-290
fn verb_to_max_edges(verb: crate::include::core::SkPathTypes::SkPathVerb) -> i32 {
    const MAX_EDGES: [u8; 6] = [0, 1, 2, 2, 3, 0];
    let index = verb as usize;
    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 288; @condition b"index < std::size(gPathVerbToMaxEdges)\0".as_ptr(); index < MAX_EDGES.len());
    i32::from(MAX_EDGES[index])
}
// Source: src/core/SkRegion_path.cpp:293-330
fn count_path_runtype_values(
    path: &crate::src::core::SkPath::SkPath,
    top_out: &mut i32,
    bottom_out: &mut i32,
) -> i32 {
    use crate::include::core::SkPathTypes::SkPathVerb;
    let mut max_edges: i32 = 0;
    let mut top = i16::MAX as f32;
    let mut bottom = i16::MIN as f32;
    let mut iter = crate::src::core::SkPath::Iter::new(path, true);
    while let Some(record) = iter.next() {
        let verb = record.fVerb;
        let points = record.fPoints;
        max_edges = max_edges.wrapping_add(verb_to_max_edges(verb));
        let last_index = verb_to_initial_last_index(verb);
        if last_index > 0 {
            for i in 1..=last_index {
                if top > points[i].fY {
                    top = points[i].fY;
                } else if bottom < points[i].fY {
                    bottom = points[i].fY;
                }
            }
        } else if verb == SkPathVerb::kMove {
            if top > points[0].fY {
                top = points[0].fY;
            } else if bottom < points[0].fY {
                bottom = points[0].fY;
            }
        }
    }
    if max_edges == 0 {
        return 0;
    }
    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 326; @condition b"top <= bot\0".as_ptr(); top <= bottom);
    *top_out = crate::include::core::SkScalar::SkScalarRoundToInt(top);
    *bottom_out = crate::include::core::SkScalar::SkScalarRoundToInt(bottom);
    max_edges
}
// Source: src/core/SkRegion_path.cpp:332-338
fn check_inverse_on_empty_return(
    destination: &mut SkRegion,
    path: &crate::src::core::SkPath::SkPath,
    clip: &SkRegion,
) -> bool {
    if path.isInverseFillType() {
        destination.set_region(clip)
    } else {
        destination.set_empty()
    }
}
// Source: src/core/SkRegion_path.cpp:340-465
impl SkRegion {
    pub fn set_path(&mut self, path: &crate::src::core::SkPath::SkPath, clip: &SkRegion) -> bool {
        use crate::src::core::SkPathEnums::SkResolveConvexity;
        use crate::src::core::SkPathPriv::SkPathPriv;
        use crate::src::core::SkRegion::RegionRuns;
        use crate::src::core::SkRegionPriv::RunHead;
        use crate::src::core::SkScan::SkScan;
        #[cfg(sk_debug)]
        self.validate();
        let raw = SkPathPriv::Raw(path, SkResolveConvexity::kYes);
        if clip.is_empty() || raw.is_none() || path.isEmpty() {
            return check_inverse_on_empty_return(self, path, clip);
        }
        let clip_bounds = *clip.get_bounds();
        if clip.is_complex() {
            if !self.set_path(path, &SkRegion::from_rect(clip_bounds)) {
                return false;
            }
            return self.op_region(clip, crate::src::core::SkRegion::Op::Intersect);
        }
        if SkScan::PathRequiresTiling(&clip_bounds) {
            const TILE_SIZE: i32 = 32767 >> 1;
            const TILE_LIMIT: i32 = 1000;
            let path_bounds = path.getBounds().roundOut();
            self.set_empty();
            let mut intersection = SkIRect::make_empty();
            if !intersection.intersect_ab(path_bounds, clip_bounds) {
                return false;
            }
            // Expand the source SkSafeMath integer operations without allocating
            // a substitute helper object: each preserves its low32 return and
            // accumulates the same sticky overflow flag.
            let mut arithmetic_ok = true;
            let mut checked_i32 = |value: i64| {
                arithmetic_ok &= i32::try_from(value).is_ok();
                value as i32
            };
            let width = checked_i32(i64::from(intersection.fRight) - i64::from(intersection.fLeft));
            let height =
                checked_i32(i64::from(intersection.fBottom) - i64::from(intersection.fTop));
            let tiles_x = checked_i32(i64::from(width) + i64::from(TILE_SIZE - 1)) / TILE_SIZE;
            let tiles_y = checked_i32(i64::from(height) + i64::from(TILE_SIZE - 1)) / TILE_SIZE;
            let total_tiles = checked_i32(i64::from(tiles_x) * i64::from(tiles_y));
            if !arithmetic_ok || total_tiles > TILE_LIMIT {
                return false;
            }
            let mut top = i64::from(intersection.fTop);
            while top < i64::from(intersection.fBottom) {
                let bottom = (top + i64::from(TILE_SIZE)).min(i64::from(intersection.fBottom));
                let mut left = i64::from(intersection.fLeft);
                while left < i64::from(intersection.fRight) {
                    let right = (left + i64::from(TILE_SIZE)).min(i64::from(intersection.fRight));
                    let mut tile_clip_bounds =
                        SkIRect::make_ltrb(left as i32, top as i32, right as i32, bottom as i32);
                    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 400; @condition b"SkIRect::Intersects(pathBounds, tileClipBounds)\0".as_ptr(); SkIRect::intersects(&path_bounds, &tile_clip_bounds));
                    tile_clip_bounds.offset((-left) as i32, (-top) as i32);
                    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 405; @condition b"!SkScan::PathRequiresTiling(tileClipBounds)\0".as_ptr(); !SkScan::PathRequiresTiling(&tile_clip_bounds));
                    let mut tile = SkRegion::new();
                    if let Some(new_path) = path.tryMakeOffset((-left) as f32, (-top) as f32) {
                        tile.set_path(&new_path, &SkRegion::from_rect(tile_clip_bounds));
                        tile.translate(left as i32, top as i32);
                        self.op_region(&tile, crate::src::core::SkRegion::Op::Union);
                    } else {
                        return false;
                    }
                    left += i64::from(TILE_SIZE);
                }
                top += i64::from(TILE_SIZE);
            }
            return self.op_region(clip, crate::src::core::SkRegion::Op::Intersect);
        }
        let (mut path_top, mut path_bottom) = (0, 0);
        let path_transitions = count_path_runtype_values(path, &mut path_top, &mut path_bottom);
        if path_transitions == 0 {
            return check_inverse_on_empty_return(self, path, clip);
        }
        let (mut clip_top, mut clip_bottom) = (0, 0);
        let clip_transitions = clip.count_runtype_values(&mut clip_top, &mut clip_bottom);
        let top = path_top.max(clip_top);
        let bottom = path_bottom.min(clip_bottom);
        if top >= bottom {
            return check_inverse_on_empty_return(self, path, clip);
        }
        let mut builder = Box::pin(SkRgnBuilder::new());
        if !unsafe { builder.as_mut().get_unchecked_mut() }.init(
            bottom.wrapping_sub(top),
            path_transitions.max(clip_transitions),
            path.isInverseFillType(),
        ) {
            return self.set_empty();
        }
        // The exclusive region builder owns output storage and the clip bounds the spans.
        unsafe {
            SkScan::FillPath(&raw.unwrap(), clip, builder.as_mut());
        }
        unsafe { builder.as_mut().get_unchecked_mut() }.done();
        let count = builder.compute_run_count();
        if count == 0 {
            return self.set_empty();
        } else if count == 7 {
            builder.copy_to_rect(&mut self.bounds);
            self.set_rect(self.bounds);
        } else {
            let mut temporary = SkRegion::new();
            let mut head = RunHead::alloc(count as i32).expect("complex region run count");
            builder.copy_to_native(&mut head);
            head.compute_run_bounds(&mut temporary.bounds);
            temporary.runs = RegionRuns::Complex(head);
            self.swap(&mut temporary);
        }
        #[cfg(sk_debug)]
        self.validate();
        true
    }
}
// Source: src/core/SkRegion_path.cpp:470-496
// Indices replace pointers into the eventual sorted edge allocation. Links are
// established only after sorting, matching the native pointer lifetime.
#[derive(Clone, Copy)]
struct Edge {
    x: i32,
    y0: i32,
    y1: i32,
    flags: u8,
    next: MaybeUninit<Option<usize>>,
}
impl Edge {
    const Y0_LINK: u8 = 1;
    const Y1_LINK: u8 = 2;
    const COMPLETE_LINK: u8 = Self::Y0_LINK | Self::Y1_LINK;
    fn set(&mut self, x: i32, y0: i32, y1: i32) {
        crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 484; @condition b"y0 != y1\0".as_ptr(); (y0) != (y1));
        self.x = x;
        self.y0 = y0;
        self.y1 = y1;
        self.flags = 0;
        // Source: src/core/SkRegion_path.cpp:490
        #[cfg(sk_debug)]
        self.next.write(None);
    }
    // Only the native debug null probes or an established Y1 link read this
    // field. In release, set() intentionally leaves its bytes untouched.
    unsafe fn next(&self) -> Option<usize> {
        self.next.assume_init_read()
    }
    fn top(&self) -> i32 {
        self.y0.min(self.y1)
    }
}
// Source: src/core/SkRegion_path.cpp:498-538
fn find_link(edges: &mut [Edge], base: usize) {
    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 499; @condition b"base < stop\0".as_ptr(); base < edges.len());
    if edges[base].flags == Edge::COMPLETE_LINK {
        crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 502; @condition b"base->fNext\0".as_ptr(); unsafe { edges[base].next() }.is_some());
        return;
    }
    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 506; @condition b"base + 1 < stop\0".as_ptr(); base + 1 < edges.len());
    let (y0, y1) = (edges[base].y0, edges[base].y1);
    let mut e = base;
    if edges[base].flags & Edge::Y0_LINK == 0 {
        loop {
            e += 1;
            if edges[e].flags & Edge::Y1_LINK == 0 && y0 == edges[e].y1 {
                crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 516; @condition b"nullptr == e->fNext\0".as_ptr(); unsafe { edges[e].next() }.is_none());
                edges[e].next.write(Some(base));
                edges[e].flags |= Edge::Y1_LINK;
                break;
            }
        }
    }
    e = base;
    if edges[base].flags & Edge::Y1_LINK == 0 {
        loop {
            e += 1;
            if edges[e].flags & Edge::Y0_LINK == 0 && y1 == edges[e].y0 {
                crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 529; @condition b"nullptr == base->fNext\0".as_ptr(); unsafe { edges[base].next() }.is_none());
                edges[base].next.write(Some(e));
                edges[e].flags |= Edge::Y0_LINK;
                break;
            }
        }
    }
    edges[base].flags = Edge::COMPLETE_LINK;
}
// Source: src/core/SkRegion_path.cpp:540-568
fn extract_path(
    edges: &mut [Edge],
    mut edge: usize,
    builder: &mut crate::src::core::SkPathBuilder::SkPathBuilder,
) -> i32 {
    use crate::src::core::SkPoint::SkPoint;
    while edges[edge].flags == 0 {
        edge += 1;
    }
    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 545; @condition b"edge < stop\0".as_ptr(); edge < edges.len());
    let base = edge;
    let mut previous = edge;
    edge = unsafe { edges[edge].next() }.expect("complete edge link");
    crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 550; @condition b"edge != base\0".as_ptr(); (edge) != (base));
    let mut count = 1;
    builder.moveTo(SkPoint::make(
        edges[previous].x as f32,
        edges[previous].y0 as f32,
    ));
    edges[previous].flags = 0;
    loop {
        if edges[previous].x != edges[edge].x || edges[previous].y1 != edges[edge].y0 {
            builder.lineTo(SkPoint::make(
                edges[previous].x as f32,
                edges[previous].y1 as f32,
            ));
            builder.lineTo(SkPoint::make(edges[edge].x as f32, edges[edge].y0 as f32));
        }
        previous = edge;
        edge = unsafe { edges[edge].next() }.expect("complete edge link");
        count += 1;
        edges[previous].flags = 0;
        if edge == base {
            break;
        }
    }
    builder.lineTo(SkPoint::make(
        edges[previous].x as f32,
        edges[previous].y1 as f32,
    ));
    builder.close();
    count
}
// Source: src/core/SkRegion_path.cpp:570-574
fn edge_less(a: &Edge, b: &Edge) -> bool {
    if a.x == b.x {
        a.top() < b.top()
    } else {
        a.x < b.x
    }
}

// Source: src/core/SkRegion_path.cpp:576-627
impl SkRegion {
    pub fn add_boundary_path(
        &self,
        builder: &mut crate::src::core::SkPathBuilder::SkPathBuilder,
    ) -> bool {
        use crate::include::core::SkPathTypes::SkPathDirection;
        use crate::src::core::SkRect::SkRect;
        use crate::src::core::SkRegion::RegionIterator;
        use crate::src::core::SkTDArray::SkTDArray;
        use crate::src::core::SkTSort::SkTQSort;
        if self.is_empty() {
            return false;
        }
        if self.is_rect() {
            builder.addRect(&SkRect::Make(self.get_bounds()), SkPathDirection::kCW, 0);
            return true;
        }
        let mut iter = RegionIterator::new(self);
        let mut edges = SkTDArray::<Edge>::new();
        while !iter.done() {
            let r = iter.rect();
            // Native append creates uninitialized POD slots. Initialize both
            // before forming any typed slice of the expanded array.
            let edge = unsafe { edges.append_uninit(2) };
            let mut left = Edge {
                x: 0,
                y0: 0,
                y1: 0,
                flags: 0,
                next: MaybeUninit::uninit(),
            };
            let mut right = left;
            left.set(r.fLeft, r.fBottom, r.fTop);
            right.set(r.fRight, r.fTop, r.fBottom);
            unsafe {
                edge.write(left);
                edge.add(1).write(right);
            }
            iter.next();
        }
        let mut count = edges.size();
        SkTQSort(edges.as_mut_slice(), &edge_less);
        for e in 0..edges.size() as usize {
            find_link(edges.as_mut_slice(), e);
        }
        // Source: src/core/SkRegion_path.cpp:613-618
        #[cfg(sk_debug)]
        for e in edges.as_slice() {
            crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 615; @condition b"e->fNext != nullptr\0".as_ptr(); unsafe { e.next() }.is_some());
            crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 616; @condition b"e->fFlags == Edge::kCompleteLink\0".as_ptr(); e.flags == Edge::COMPLETE_LINK);
        }
        builder.incReserve((count << 1) as usize, (count << 1) as usize, 0);
        loop {
            crate::SkASSERT!(@source b"src/core/SkRegion_path.cpp\0".as_ptr(), 622; @condition b"count > 1\0".as_ptr(); count > 1);
            count -= extract_path(edges.as_mut_slice(), 0, builder);
            if count <= 0 {
                break;
            }
        }
        true
    }
}
// Source: src/core/SkRegion_path.cpp:629-633
impl SkRegion {
    pub fn get_boundary_path(&self) -> crate::src::core::SkPath::SkPath {
        let mut builder = crate::src::core::SkPathBuilder::SkPathBuilder::new();
        let _ = self.add_boundary_path(&mut builder);
        builder.detach(None)
    }
}
