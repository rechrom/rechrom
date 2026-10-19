// Translation status: in_progress
// Copyright 2006 The Android Open Source Project
// Use of this source code is governed by a BSD-style license that can be found in LICENSE.
//! Public canonical SkRegion RLE storage and algorithms. Empty/rectangle head
//! sentinels become enum variants; Arc supplies RunHead's atomic COW ownership.
#![allow(non_camel_case_types, non_upper_case_globals, dead_code)]
use std::sync::Arc;

// Source: include/core/SkRect.h:37-44,207-218; src/core/SkRect.cpp:17-45.
// The existing tiny-skia SkIRect excludes empty/large rectangles. This value
// carries official SkIRect's complete four-i32 LTRB domain without clamping.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SkRegionIRect {
    pub fLeft: i32,
    pub fTop: i32,
    pub fRight: i32,
    pub fBottom: i32,
}
impl SkRegionIRect {
    pub const fn MakeLTRB(l: i32, t: i32, r: i32, b: i32) -> Self {
        Self {
            fLeft: l,
            fTop: t,
            fRight: r,
            fBottom: b,
        }
    }
    pub fn isEmpty(&self) -> bool {
        self.fLeft >= self.fRight || self.fTop >= self.fBottom
    }
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.fLeft <= x && x < self.fRight && self.fTop <= y && y < self.fBottom
    }
    pub fn containsNoEmptyCheck(&self, r: &Self) -> bool {
        self.fLeft <= r.fLeft
            && self.fTop <= r.fTop
            && self.fRight >= r.fRight
            && self.fBottom >= r.fBottom
    }
    pub fn contains(&self, r: &Self) -> bool {
        !r.isEmpty() && self.containsNoEmptyCheck(r)
    }
    pub fn intersect(&mut self, a: &Self, b: &Self) -> bool {
        let r = Self::MakeLTRB(
            a.fLeft.max(b.fLeft),
            a.fTop.max(b.fTop),
            a.fRight.min(b.fRight),
            a.fBottom.min(b.fBottom),
        );
        if r.isEmpty() {
            false
        } else {
            *self = r;
            true
        }
    }
    pub fn Intersects(a: &Self, b: &Self) -> bool {
        !a.isEmpty()
            && !b.isEmpty()
            && a.fLeft < b.fRight
            && b.fLeft < a.fRight
            && a.fTop < b.fBottom
            && b.fTop < a.fBottom
    }
    fn offset(&mut self, x: i32, y: i32) {
        self.fLeft = self.fLeft.wrapping_add(x);
        self.fRight = self.fRight.wrapping_add(x);
        self.fTop = self.fTop.wrapping_add(y);
        self.fBottom = self.fBottom.wrapping_add(y);
    }
}

// Source: include/core/SkRegion.h:378-391; src/core/SkRegionPriv.h:20-39.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Op {
    kDifference_Op = 0,
    kIntersect_Op = 1,
    kUnion_Op = 2,
    kXOR_Op = 3,
    kReverseDifference_Op = 4,
    kReplace_Op = 5,
}
const S: i32 = i32::MAX;
const RECT_RUNS: usize = 7;

// Source: src/core/SkRegionPriv.h:59-148,187-255.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RunHead {
    runs: Vec<i32>,
    y_span_count: i32,
    interval_count: i32,
}
impl RunHead {
    fn new(count: usize) -> Self {
        assert!(
            count >= RECT_RUNS
                && count
                    .checked_mul(4)
                    .and_then(|v| v.checked_add(16))
                    .is_some_and(|v| v <= i32::MAX as usize)
        );
        Self {
            runs: vec![0; count],
            y_span_count: 0,
            interval_count: 0,
        }
    }
    // Source: src/core/SkRegionPriv.h:154-185.
    fn findScanline(&self, y: i32) -> usize {
        debug_assert!(y >= self.runs[0]);
        let mut i = 1;
        loop {
            debug_assert!(self.runs[i] < S);
            if y < self.runs[i] {
                return i;
            }
            i = skip_scanline(&self.runs, i);
        }
    }
    fn computeRunBounds(&mut self, bounds: &mut SkRegionIRect) {
        let runs = &self.runs;
        bounds.fTop = runs[0];
        let mut i = 1;
        let mut ys = 0;
        let mut intervals = 0;
        let mut l = i32::MAX;
        let mut r = i32::MIN;
        let mut bot;
        loop {
            bot = runs[i];
            i += 1;
            debug_assert!(bot < S);
            ys += 1;
            let n = runs[i];
            i += 1;
            debug_assert!(n >= 0 && n < S);
            if n > 0 {
                l = l.min(runs[i]);
                i += n as usize * 2;
                r = r.max(runs[i - 1]);
                intervals += n;
            }
            debug_assert_eq!(runs[i], S);
            i += 1;
            if runs[i] == S {
                break;
            }
        }
        debug_assert_eq!(i + 1, runs.len());
        self.y_span_count = ys;
        self.interval_count = intervals;
        bounds.fLeft = l;
        bounds.fRight = r;
        bounds.fBottom = bot;
    }
}
// Source: src/core/SkRegion.cpp:55-57; include/core/SkRegion.h:640-665.
#[derive(Clone, Debug, Eq, PartialEq)]
enum RunStorage {
    Empty,
    Rect,
    Complex(Arc<RunHead>),
}
// Source: include/core/SkRegion.h:656-665; src/core/SkRegion.cpp:145-236.
#[derive(Clone, Debug)]
pub struct SkRegion {
    fBounds: SkRegionIRect,
    fRunHead: RunStorage,
}
impl Default for SkRegion {
    fn default() -> Self {
        Self {
            fBounds: SkRegionIRect::default(),
            fRunHead: RunStorage::Empty,
        }
    }
}

// Source: src/core/SkRegion.cpp:489-506.
enum RunView<'a> {
    Empty([i32; 1]),
    Rect([i32; 7]),
    Complex(&'a [i32]),
}
impl AsRef<[i32]> for RunView<'_> {
    fn as_ref(&self) -> &[i32] {
        match self {
            Self::Empty(v) => v,
            Self::Rect(v) => v,
            Self::Complex(v) => v,
        }
    }
}
// Source: src/core/SkRegion.cpp:102-119,420-442.
fn skip_scanline(runs: &[i32], i: usize) -> usize {
    debug_assert!(runs[i] < S);
    i + 2 + runs[i + 1] as usize * 2 + 1
}
fn scanline_contains(runs: &[i32], mut i: usize, l: i32, r: i32) -> bool {
    i += 2;
    loop {
        if l < runs[i] {
            return false;
        }
        if r <= runs[i + 1] {
            return true;
        }
        i += 2;
    }
}
// Source: src/core/SkRegion.cpp:509-522.
fn scanline_intersects(runs: &[i32], mut i: usize, l: i32, r: i32) -> bool {
    i += 2;
    loop {
        if r <= runs[i] {
            return false;
        }
        if l < runs[i + 1] {
            return true;
        }
        i += 2;
    }
}

impl SkRegion {
    // Source: src/core/SkRegion.cpp:145-245; include/core/SkRegion.h:114-167.
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_rect(rect: SkRegionIRect) -> Self {
        let mut r = Self::new();
        r.setRect(rect);
        r
    }
    pub fn isEmpty(&self) -> bool {
        matches!(self.fRunHead, RunStorage::Empty)
    }
    pub fn isRect(&self) -> bool {
        matches!(self.fRunHead, RunStorage::Rect)
    }
    pub fn isComplex(&self) -> bool {
        matches!(self.fRunHead, RunStorage::Complex(_))
    }
    pub fn getBounds(&self) -> &SkRegionIRect {
        &self.fBounds
    }
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    pub fn computeRegionComplexity(&self) -> i32 {
        match &self.fRunHead {
            RunStorage::Empty => 0,
            RunStorage::Rect => 1,
            RunStorage::Complex(h) => h.interval_count,
        }
    }
    pub fn setEmpty(&mut self) -> bool {
        self.fBounds = SkRegionIRect::default();
        self.fRunHead = RunStorage::Empty;
        false
    }
    pub fn setRect(&mut self, r: SkRegionIRect) -> bool {
        if r.isEmpty() || r.fRight == S || r.fBottom == S {
            return self.setEmpty();
        }
        self.fBounds = r;
        self.fRunHead = RunStorage::Rect;
        true
    }
    pub fn setRegion(&mut self, src: &Self) -> bool {
        self.clone_from(src);
        !self.isEmpty()
    }
    pub fn set(&mut self, src: &Self) -> bool {
        self.setRegion(src)
    }
    // Source: src/core/SkRegion.cpp:121-141,375-383.
    fn BuildRectRuns(b: SkRegionIRect) -> [i32; 7] {
        [b.fTop, b.fBottom, 1, b.fLeft, b.fRight, S, S]
    }
    fn getRuns(&self) -> RunView<'_> {
        match &self.fRunHead {
            RunStorage::Empty => RunView::Empty([S]),
            RunStorage::Rect => RunView::Rect(Self::BuildRectRuns(self.fBounds)),
            RunStorage::Complex(h) => RunView::Complex(&h.runs),
        }
    }
    // Source: src/core/SkRegion.cpp:301-372.
    fn setRuns(&mut self, mut runs: Vec<i32>) -> bool {
        debug_assert!(self.isValid());
        let mut count = runs.len();
        debug_assert!(count > 0);
        if count <= 2 {
            return self.setEmpty();
        }
        let mut start = 0;
        if count > RECT_RUNS {
            if runs[3] == S {
                start = 3;
                runs[start] = runs[start - 2];
            }
            if runs[count - 5] == S {
                runs[count - 4] = S;
                count -= 3;
            }
        }
        count -= start;
        debug_assert!(count >= RECT_RUNS);
        if count == RECT_RUNS {
            return self.setRect(SkRegionIRect::MakeLTRB(
                runs[start + 3],
                runs[start],
                runs[start + 4],
                runs[start + 1],
            ));
        }
        if !matches!(&self.fRunHead,RunStorage::Complex(h) if h.runs.len()==count) {
            self.fRunHead = RunStorage::Complex(Arc::new(RunHead::new(count)));
        }
        let RunStorage::Complex(head) = &mut self.fRunHead else {
            unreachable!()
        };
        let head = Arc::make_mut(head);
        head.runs.copy_from_slice(&runs[start..start + count]);
        head.computeRunBounds(&mut self.fBounds);
        if self.fBounds.isEmpty() {
            return self.setEmpty();
        }
        debug_assert!(self.isValid());
        true
    }
    // Source: src/core/SkRegion.cpp:386-418.
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        debug_assert!(self.isValid());
        if !self.fBounds.contains_point(x, y) {
            return false;
        }
        if self.isRect() {
            return true;
        }
        let RunStorage::Complex(h) = &self.fRunHead else {
            unreachable!()
        };
        let mut i = h.findScanline(y) + 2;
        loop {
            if x < h.runs[i] {
                return false;
            }
            if x < h.runs[i + 1] {
                return true;
            }
            i += 2;
        }
    }
    // Source: src/core/SkRegion.cpp:444-487.
    pub fn contains_rect(&self, r: SkRegionIRect) -> bool {
        debug_assert!(self.isValid());
        if !self.fBounds.contains(&r) {
            return false;
        }
        if self.isRect() {
            return true;
        }
        let RunStorage::Complex(h) = &self.fRunHead else {
            unreachable!()
        };
        let mut i = h.findScanline(r.fTop);
        loop {
            if !scanline_contains(&h.runs, i, r.fLeft, r.fRight) {
                return false;
            }
            if r.fBottom <= h.runs[i] {
                break;
            }
            i = skip_scanline(&h.runs, i);
        }
        true
    }
    pub fn contains(&self, rgn: &Self) -> bool {
        debug_assert!(self.isValid());
        if self.isEmpty() || rgn.isEmpty() || !self.fBounds.contains(&rgn.fBounds) {
            return false;
        }
        if self.isRect() {
            return true;
        }
        if rgn.isRect() {
            return self.contains_rect(rgn.fBounds);
        }
        !Self::Oper(rgn, self, Op::kDifference_Op, None)
    }
    // Source: src/core/SkRegion.cpp:524-577.
    pub fn intersects_rect(&self, r: SkRegionIRect) -> bool {
        debug_assert!(self.isValid());
        if self.isEmpty() || r.isEmpty() {
            return false;
        }
        let mut sect = SkRegionIRect::default();
        if !sect.intersect(&self.fBounds, &r) {
            return false;
        }
        if self.isRect() {
            return true;
        }
        let RunStorage::Complex(h) = &self.fRunHead else {
            unreachable!()
        };
        let mut i = h.findScanline(sect.fTop);
        loop {
            if scanline_intersects(&h.runs, i, sect.fLeft, sect.fRight) {
                return true;
            }
            if sect.fBottom <= h.runs[i] {
                break;
            }
            i = skip_scanline(&h.runs, i);
        }
        false
    }
    pub fn intersects(&self, r: &Self) -> bool {
        if self.isEmpty() || r.isEmpty() || !SkRegionIRect::Intersects(&self.fBounds, &r.fBounds) {
            return false;
        }
        if self.isRect() && r.isRect() {
            return true;
        }
        if self.isRect() {
            return r.intersects_rect(self.fBounds);
        }
        if r.isRect() {
            return self.intersects_rect(r.fBounds);
        }
        Self::Oper(self, r, Op::kIntersect_Op, None)
    }
    // Source: src/core/SkRegion.cpp:609-675.
    pub fn translate(&mut self, dx: i32, dy: i32) {
        debug_assert!(self.isValid());
        if self.isEmpty() {
            self.setEmpty();
            return;
        }
        let dx = pin_offset_s32(self.fBounds.fLeft, self.fBounds.fRight, dx);
        let dy = pin_offset_s32(self.fBounds.fTop, self.fBounds.fBottom, dy);
        if self.isRect() {
            let mut b = self.fBounds;
            b.offset(dx, dy);
            self.setRect(b);
            return;
        }
        self.fBounds.offset(dx, dy);
        let RunStorage::Complex(h) = &mut self.fRunHead else {
            unreachable!()
        };
        let runs = &mut Arc::make_mut(h).runs;
        runs[0] += dy;
        let mut i = 1;
        loop {
            if runs[i] == S {
                break;
            }
            runs[i] += dy;
            i += 2;
            loop {
                if runs[i] == S {
                    break;
                }
                runs[i] += dx;
                runs[i + 1] += dx;
                i += 2;
            }
            i += 1;
        }
    }
    pub fn translate_to(&self, dx: i32, dy: i32, dst: &mut Self) {
        *dst = self.clone();
        dst.translate(dx, dy);
    }
    // Source: src/core/SkRegion.cpp:679-692.
    pub fn setRects<R: Copy + Into<SkRegionIRect>>(&mut self, rects: &[R]) -> bool {
        if rects.is_empty() {
            return self.setEmpty();
        }
        if rects.len() == 1 {
            return self.setRect(rects[0].into());
        }
        let mid = rects.len() / 2;
        let mut a = Self::new();
        let mut b = Self::new();
        a.setRects(&rects[..mid]);
        b.setRects(&rects[mid..]);
        self.op_regions(&a, &b, Op::kUnion_Op)
    }
}
// Source: src/core/SkRegion.cpp:581-606.
impl PartialEq for SkRegion {
    fn eq(&self, b: &Self) -> bool {
        debug_assert!(self.isValid());
        debug_assert!(b.isValid());
        if self.fBounds != b.fBounds {
            return false;
        }
        match (&self.fRunHead, &b.fRunHead) {
            (RunStorage::Empty, RunStorage::Empty) | (RunStorage::Rect, RunStorage::Rect) => true,
            (RunStorage::Complex(a), RunStorage::Complex(b)) => {
                Arc::ptr_eq(a, b) || a.runs == b.runs
            }
            _ => false,
        }
    }
}
impl Eq for SkRegion {}
// Source: src/core/SkRegion.cpp:609-616.
fn pin_offset_s32(min: i32, max: i32, mut offset: i32) -> i32 {
    debug_assert!(min <= max);
    if (min as i64) + (offset as i64) < i32::MIN as i64 {
        offset = i32::MIN - min;
    }
    if (max as i64) + (offset as i64) > i32::MAX as i64 {
        offset = i32::MAX - max;
    }
    offset
}

// Source: src/core/SkRegion.cpp:60-100.
struct RunArray(Vec<i32>);
impl RunArray {
    fn new() -> Self {
        Self(vec![0; 256])
    }
    fn resizeToAtLeast(&mut self, count: usize) {
        if count > self.0.len() {
            let count = count
                .checked_add(count >> 1)
                .filter(|n| *n <= i32::MAX as usize)
                .unwrap_or(i32::MAX as usize);
            self.0.resize(count, 0);
        }
    }
}
// Source: src/core/SkRegion.cpp:711-799.
struct SpanRec<'a> {
    a: &'a [i32],
    b: &'a [i32],
    ai: usize,
    bi: usize,
    al: i32,
    ar: i32,
    bl: i32,
    br: i32,
    left: i32,
    right: i32,
    inside: i32,
}
impl<'a> SpanRec<'a> {
    fn new(a: &'a [i32], b: &'a [i32]) -> Self {
        // A sentinel has no active right edge. C++ reads a don't-care slot;
        // Rust supplies zero only for that unreachable/don't-care operand.
        Self {
            a,
            b,
            ai: 2,
            bi: 2,
            al: a[0],
            ar: *a.get(1).unwrap_or(&0),
            bl: b[0],
            br: *b.get(1).unwrap_or(&0),
            left: 0,
            right: 0,
            inside: 0,
        }
    }
    fn done(&self) -> bool {
        self.al == S && self.bl == S
    }
    fn next(&mut self) {
        debug_assert!(self.al == S || self.al < self.ar);
        debug_assert!(self.bl == S || self.bl < self.br);
        let (mut al, mut ar, mut bl, mut br) = (self.al, self.ar, self.bl, self.br);
        let mut af = false;
        let mut bf = false;
        let inside;
        let left;
        let mut right = 0;
        if al < bl {
            inside = 1;
            left = al;
            if ar <= bl {
                right = ar;
                af = true;
            } else {
                al = bl;
                right = al;
            }
        } else if bl < al {
            inside = 2;
            left = bl;
            if br <= al {
                right = br;
                bf = true;
            } else {
                bl = al;
                right = bl;
            }
        } else {
            inside = 3;
            left = al;
            if ar <= br {
                bl = ar;
                right = bl;
                af = true;
            }
            if br <= ar {
                al = br;
                right = al;
                bf = true;
            }
        }
        if af {
            al = self.a[self.ai];
            ar = *self.a.get(self.ai + 1).unwrap_or(&0);
            self.ai += 2;
        }
        if bf {
            bl = self.b[self.bi];
            br = *self.b.get(self.bi + 1).unwrap_or(&0);
            self.bi += 2;
        }
        debug_assert!(left <= right);
        self.al = al;
        self.ar = ar;
        self.bl = bl;
        self.br = br;
        self.left = left;
        self.right = right;
        self.inside = inside;
    }
}
// Source: src/core/SkRegion.cpp:801-857.
fn operate_on_span(
    a: &[i32],
    b: &[i32],
    array: &mut RunArray,
    offset: usize,
    min: i32,
    max: i32,
) -> usize {
    // Counted run slices contain exactly the intervals followed by X-sentinel.
    let ac = a.len() - 1;
    let bc = b.len() - 1;
    debug_assert_eq!(a[ac], S);
    debug_assert_eq!(b[bc], S);
    array.resizeToAtLeast(offset + ac + bc + 2);
    let mut dst = offset;
    let mut rec = SpanRec::new(a, b);
    let mut first = true;
    while !rec.done() {
        rec.next();
        let l = rec.left;
        let r = rec.right;
        if (rec.inside.wrapping_sub(min) as u32) <= max.wrapping_sub(min) as u32 && l < r {
            if first || array.0[dst - 1] < l {
                array.0[dst] = l;
                array.0[dst + 1] = r;
                dst += 2;
                first = false;
            } else {
                array.0[dst - 1] = r;
            }
        }
    }
    debug_assert!(dst < array.0.len() - 1);
    array.0[dst] = S;
    dst + 1
}
// Source: src/core/SkRegion.cpp:863-931.
struct RgnOper<'a> {
    array: &'a mut RunArray,
    min: i32,
    max: i32,
    prev_dst: usize,
    prev_len: usize,
    top: i32,
}
impl<'a> RgnOper<'a> {
    fn new(top: i32, array: &'a mut RunArray, op: Op) -> Self {
        let (min, max) = [(1, 1), (3, 3), (1, 3), (1, 2)][op as usize];
        Self {
            array,
            min,
            max,
            prev_dst: 1,
            prev_len: 0,
            top,
        }
    }
    fn addSpan(&mut self, bottom: i32, a: &[i32], b: &[i32]) {
        let start = self.prev_dst + self.prev_len + 2;
        let stop = operate_on_span(a, b, self.array, start, self.min, self.max);
        let len = stop - start;
        debug_assert!(len >= 1 && len & 1 == 1);
        debug_assert_eq!(self.array.0[stop - 1], S);
        if self.prev_len == len
            && (len == 1
                || self.array.0[self.prev_dst..self.prev_dst + len - 1]
                    == self.array.0[start..start + len - 1])
        {
            self.array.0[self.prev_dst - 2] = bottom;
        } else if len == 1 && self.prev_len == 0 {
            self.top = bottom;
        } else {
            self.array.0[start - 2] = bottom;
            self.array.0[start - 1] = (len >> 1) as i32;
            self.prev_dst = start;
            self.prev_len = len;
        }
    }
    fn flush(&mut self) -> usize {
        self.array.0[0] = self.top;
        self.array.0[self.prev_dst + self.prev_len] = S;
        self.prev_dst + self.prev_len + 1
    }
    fn isEmpty(&self) -> bool {
        self.prev_len == 0
    }
}
// Source: src/core/SkRegion.cpp:936-1043.
fn operate(a: &[i32], b: &[i32], dst: &mut RunArray, op: Op, quick_exit: bool) -> isize {
    let (mut at, mut ab, mut bt, mut bb) = (a[0], a[1], b[0], b[1]);
    let (mut ai, mut bi) = (3, 3);
    let mut out = RgnOper::new(at.min(bt), dst, op);
    let mut prev = S;
    while ab < S || bb < S {
        let top;
        let mut bot = 0;
        let (mut ar, mut br): (&[i32], &[i32]) = (&[S], &[S]);
        let mut af = false;
        let mut bf = false;
        // Slice ends are obtained from each resident scanline's native count.
        // This replaces pointer advancement, not its event/strip ordering.
        if at < bt {
            top = at;
            ar = &a[ai..ai + a[ai - 1] as usize * 2 + 1];
            if ab <= bt {
                bot = ab;
                af = true;
            } else {
                at = bt;
                bot = at;
            }
        } else if bt < at {
            top = bt;
            br = &b[bi..bi + b[bi - 1] as usize * 2 + 1];
            if bb <= at {
                bot = bb;
                bf = true;
            } else {
                bt = at;
                bot = bt;
            }
        } else {
            top = at;
            ar = &a[ai..ai + a[ai - 1] as usize * 2 + 1];
            br = &b[bi..bi + b[bi - 1] as usize * 2 + 1];
            if ab <= bb {
                bt = ab;
                bot = bt;
                af = true;
            }
            if bb <= ab {
                at = bb;
                bot = at;
                bf = true;
            }
        }
        if top > prev {
            out.addSpan(top, &[S], &[S]);
        }
        out.addSpan(bot, ar, br);
        if quick_exit && !out.isEmpty() {
            return -1;
        }
        if af {
            ai += a[ai - 1] as usize * 2 + 1;
            at = ab;
            ab = a[ai];
            ai += 2;
            if ab == S {
                at = ab;
            }
        }
        if bf {
            bi += b[bi - 1] as usize * 2 + 1;
            bt = bb;
            bb = b[bi];
            bi += 2;
            if bb == S {
                bt = bb;
            }
        }
        prev = bot;
    }
    out.flush() as isize
}
// Source: src/core/SkRegion.cpp:1063-1180.
impl SkRegion {
    fn setEmptyCheck(result: Option<&mut Self>) -> bool {
        result.is_some_and(Self::setEmpty)
    }
    fn setRectCheck(result: Option<&mut Self>, r: SkRegionIRect) -> bool {
        match result {
            Some(dst) => dst.setRect(r),
            None => !r.isEmpty(),
        }
    }
    fn setRegionCheck(result: Option<&mut Self>, r: &Self) -> bool {
        match result {
            Some(dst) => dst.setRegion(r),
            None => !r.isEmpty(),
        }
    }
    fn Oper<'a>(mut a: &'a Self, mut b: &'a Self, mut op: Op, result: Option<&mut Self>) -> bool {
        if op == Op::kReplace_Op {
            return Self::setRegionCheck(result, b);
        }
        if op == Op::kReverseDifference_Op {
            std::mem::swap(&mut a, &mut b);
            op = Op::kDifference_Op;
        }
        let ae = a.isEmpty();
        let be = b.isEmpty();
        let ar = a.isRect();
        let br = b.isRect();
        let mut bounds = SkRegionIRect::default();
        match op {
            Op::kDifference_Op => {
                if ae {
                    return Self::setEmptyCheck(result);
                }
                if be || !SkRegionIRect::Intersects(&a.fBounds, &b.fBounds) {
                    return Self::setRegionCheck(result, a);
                }
                if br && b.fBounds.containsNoEmptyCheck(&a.fBounds) {
                    return Self::setEmptyCheck(result);
                }
            }
            Op::kIntersect_Op => {
                if (ae | be) || !bounds.intersect(&a.fBounds, &b.fBounds) {
                    return Self::setEmptyCheck(result);
                }
                if ar & br {
                    return Self::setRectCheck(result, bounds);
                }
                if ar && a.fBounds.contains(&b.fBounds) {
                    return Self::setRegionCheck(result, b);
                }
                if br && b.fBounds.contains(&a.fBounds) {
                    return Self::setRegionCheck(result, a);
                }
            }
            Op::kUnion_Op => {
                if ae {
                    return Self::setRegionCheck(result, b);
                }
                if be {
                    return Self::setRegionCheck(result, a);
                }
                if ar && a.fBounds.contains(&b.fBounds) {
                    return Self::setRegionCheck(result, a);
                }
                if br && b.fBounds.contains(&a.fBounds) {
                    return Self::setRegionCheck(result, b);
                }
            }
            Op::kXOR_Op => {
                if ae {
                    return Self::setRegionCheck(result, b);
                }
                if be {
                    return Self::setRegionCheck(result, a);
                }
            }
            _ => return false,
        }
        let av = a.getRuns();
        let bv = b.getRuns();
        let mut array = RunArray::new();
        let count = operate(av.as_ref(), bv.as_ref(), &mut array, op, result.is_none());
        if let Some(dst) = result {
            array.0.truncate(count as usize);
            dst.setRuns(array.0)
        } else {
            count == -1 || count > 2
        }
    }
    pub fn op_regions(&mut self, a: &Self, b: &Self, op: Op) -> bool {
        Self::Oper(a, b, op, Some(self))
    }
    pub fn op(&mut self, b: &Self, op: Op) -> bool {
        // Source permits result==a. Arc clone retains only the native COW head
        // so Rust's nonaliasing reference rules don't copy the run buffer.
        let a = self.clone();
        Self::Oper(&a, b, op, Some(self))
    }
    // Source: include/core/SkRegion.h:393-410.
    pub fn op_rect(&mut self, r: SkRegionIRect, op: Op) -> bool {
        if self.isRect() && op == Op::kIntersect_Op {
            let old = self.fBounds;
            if !self.fBounds.intersect(&old, &r) {
                return self.setEmpty();
            }
            return true;
        }
        self.op(&Self::from_rect(r), op)
    }
    // Source: src/core/SkRegion.cpp:239-249.
    pub fn op_rect_region(&mut self, a: SkRegionIRect, b: &Self, op: Op) -> bool {
        self.op_regions(&Self::from_rect(a), b, op)
    }
    pub fn op_region_rect(&mut self, a: &Self, b: SkRegionIRect, op: Op) -> bool {
        self.op_regions(a, &Self::from_rect(b), op)
    }
    // Source: include/core/SkRegion.h:322-331.
    pub fn quickContains(&self, r: SkRegionIRect) -> bool {
        debug_assert_eq!(self.isEmpty(), self.fBounds.isEmpty());
        self.isRect() && self.fBounds.contains(&r)
    }
    pub fn quickReject_rect(&self, r: SkRegionIRect) -> bool {
        self.isEmpty() || r.isEmpty() || !SkRegionIRect::Intersects(&self.fBounds, &r)
    }
    pub fn quickReject(&self, r: &Self) -> bool {
        self.isEmpty() || r.isEmpty() || !SkRegionIRect::Intersects(&self.fBounds, &r.fBounds)
    }
}

// Source: include/core/SkRegion.h:448-500; src/core/SkRegion.cpp:1402-1470.
pub struct Iterator<'a> {
    rgn: Option<&'a SkRegion>,
    rect: SkRegionIRect,
    runs: Option<usize>,
    done: bool,
}
impl<'a> Default for Iterator<'a> {
    fn default() -> Self {
        Self {
            rgn: None,
            rect: Default::default(),
            runs: None,
            done: true,
        }
    }
}
impl<'a> Iterator<'a> {
    pub fn new(rgn: &'a SkRegion) -> Self {
        let mut i = Self::default();
        i.reset(rgn);
        i
    }
    pub fn rewind(&mut self) -> bool {
        if let Some(r) = self.rgn {
            self.reset(r);
            true
        } else {
            false
        }
    }
    pub fn reset(&mut self, rgn: &'a SkRegion) {
        self.rgn = Some(rgn);
        if rgn.isEmpty() {
            self.done = true;
        } else {
            self.done = false;
            if rgn.isRect() {
                self.rect = rgn.fBounds;
                self.runs = None;
            } else {
                let RunStorage::Complex(h) = &rgn.fRunHead else {
                    unreachable!()
                };
                let r = &h.runs;
                self.rect = SkRegionIRect::MakeLTRB(r[3], r[0], r[4], r[1]);
                self.runs = Some(5);
            }
        }
    }
    pub fn done(&self) -> bool {
        self.done
    }
    pub fn rect(&self) -> &SkRegionIRect {
        &self.rect
    }
    pub fn rgn(&self) -> Option<&'a SkRegion> {
        self.rgn
    }
    pub fn next(&mut self) {
        if self.done {
            return;
        }
        let Some(mut i) = self.runs else {
            self.done = true;
            return;
        };
        let RunStorage::Complex(h) = &self.rgn.unwrap().fRunHead else {
            unreachable!()
        };
        let r = &h.runs;
        if r[i] < S {
            self.rect.fLeft = r[i];
            self.rect.fRight = r[i + 1];
            i += 2;
        } else {
            i += 1;
            if r[i] < S {
                if r[i + 1] == 0 {
                    self.rect.fTop = r[i];
                    i += 3;
                } else {
                    self.rect.fTop = self.rect.fBottom;
                }
                self.rect.fBottom = r[i];
                self.rect.fLeft = r[i + 2];
                self.rect.fRight = r[i + 3];
                i += 4;
            } else {
                self.done = true;
            }
        }
        self.runs = Some(i);
    }
}
// Source: include/core/SkRegion.h:514-563; src/core/SkRegion.cpp:1472-1507.
pub struct Cliperator<'a> {
    iter: Iterator<'a>,
    clip: SkRegionIRect,
    rect: SkRegionIRect,
    done: bool,
}
impl<'a> Cliperator<'a> {
    pub fn new(rgn: &'a SkRegion, clip: SkRegionIRect) -> Self {
        let mut s = Self {
            iter: Iterator::new(rgn),
            clip,
            rect: Default::default(),
            done: true,
        };
        while !s.iter.done() {
            if s.iter.rect.fTop >= clip.fBottom {
                break;
            }
            if s.rect.intersect(&clip, s.iter.rect()) {
                s.done = false;
                break;
            }
            s.iter.next();
        }
        s
    }
    pub fn done(&self) -> bool {
        self.done
    }
    pub fn rect(&self) -> &SkRegionIRect {
        &self.rect
    }
    pub fn next(&mut self) {
        if self.done {
            return;
        }
        self.done = true;
        self.iter.next();
        while !self.iter.done() {
            if self.iter.rect.fTop >= self.clip.fBottom {
                break;
            }
            if self.rect.intersect(&self.clip, self.iter.rect()) {
                self.done = false;
                break;
            }
            self.iter.next();
        }
    }
}
// Source: include/core/SkRegion.h:574-611; src/core/SkRegion.cpp:1511-1588.
pub struct Spanerator<'a> {
    runs: Option<&'a [i32]>,
    left: i32,
    right: i32,
    done: bool,
}
impl<'a> Spanerator<'a> {
    pub fn new(rgn: &'a SkRegion, y: i32, mut left: i32, mut right: i32) -> Self {
        let mut s = Self {
            runs: None,
            left: 0,
            right: 0,
            done: true,
        };
        let r = rgn.fBounds;
        if !rgn.isEmpty() && y >= r.fTop && y < r.fBottom && right > r.fLeft && left < r.fRight {
            if rgn.isRect() {
                left = left.max(r.fLeft);
                right = right.min(r.fRight);
                s.left = left;
                s.right = right;
                s.done = false;
            } else {
                let RunStorage::Complex(h) = &rgn.fRunHead else {
                    unreachable!()
                };
                let mut i = h.findScanline(y) + 2;
                loop {
                    if h.runs[i] >= right {
                        break;
                    }
                    if h.runs[i + 1] <= left {
                        i += 2;
                        continue;
                    }
                    s.runs = Some(&h.runs[i..]);
                    s.left = left;
                    s.right = right;
                    s.done = false;
                    break;
                }
            }
        }
        s
    }
    pub fn next(&mut self, left: Option<&mut i32>, right: Option<&mut i32>) -> bool {
        if self.done {
            return false;
        }
        if let Some(r) = self.runs {
            if r[0] >= self.right {
                self.done = true;
                return false;
            }
            debug_assert!(r[1] > self.left);
            if let Some(l) = left {
                *l = self.left.max(r[0]);
            }
            if let Some(v) = right {
                *v = self.right.min(r[1]);
            }
            self.runs = Some(&r[2..]);
        } else {
            self.done = true;
            if let Some(l) = left {
                *l = self.left;
            }
            if let Some(r) = right {
                *r = self.right;
            }
        }
        true
    }
}
// Source: src/core/SkRegion.cpp:1592-1630.
impl SkRegion {
    pub fn VisitSpans(&self, mut visitor: impl FnMut(SkRegionIRect)) {
        if self.isEmpty() {
            return;
        }
        if self.isRect() {
            visitor(self.fBounds);
            return;
        }
        let RunStorage::Complex(h) = &self.fRunHead else {
            unreachable!()
        };
        let r = &h.runs;
        let mut top = r[0];
        let mut bot = r[1];
        let mut p = 2;
        loop {
            let n = r[p] as usize;
            p += 1;
            if n == 1 {
                visitor(SkRegionIRect::MakeLTRB(r[p], top, r[p + 1], bot));
                p += 2;
            } else if n > 1 {
                for y in top..bot {
                    for i in 0..n {
                        visitor(SkRegionIRect::MakeLTRB(
                            r[p + 2 * i],
                            y,
                            r[p + 2 * i + 1],
                            y + 1,
                        ));
                    }
                }
                p += n * 2;
            }
            debug_assert_eq!(r[p], S);
            p += 1;
            top = bot;
            bot = r[p];
            p += 1;
            if bot == S {
                break;
            }
        }
    }
}
// Source: src/core/SkRegion.cpp:1215-1319.
fn validate_run_count(ys: i32, intervals: i32, count: usize) -> bool {
    if ys < 1 || intervals < 2 {
        return false;
    }
    2i32.checked_add(ys)
        .and_then(|v| v.checked_add(ys))
        .and_then(|v| v.checked_add(ys))
        .and_then(|v| v.checked_add(intervals))
        .and_then(|v| v.checked_add(intervals))
        .is_some_and(|n| n as usize == count)
}
fn validate_run_data(
    count: usize,
    at: impl Fn(usize) -> i32,
    given: SkRegionIRect,
    mut ys: i32,
    mut intervals: i32,
) -> bool {
    if !validate_run_count(ys, intervals, count) {
        return false;
    }
    if at(count - 1) != S || at(count - 2) != S {
        return false;
    }
    let mut bounds = SkRegionIRect::default();
    let mut rect = SkRegionIRect::default();
    let mut prev_empty = true;
    rect.fTop = at(0);
    let mut p = 1;
    if rect.fTop == S || rect.fTop != given.fTop {
        return false;
    }
    loop {
        ys -= 1;
        if ys < 0 || p + 2 > count {
            return false;
        }
        rect.fBottom = at(p);
        p += 1;
        if rect.fBottom == S || rect.fBottom > given.fBottom || rect.fBottom <= rect.fTop {
            return false;
        }
        let mut n = at(p);
        p += 1;
        if n < 0 || n > intervals || p + 1 + n as usize * 2 > count {
            return false;
        }
        if n == 0 {
            if prev_empty {
                return false;
            }
            prev_empty = true;
        } else {
            prev_empty = false;
        }
        intervals -= n;
        let mut first = true;
        let mut last_right = 0;
        while n > 0 {
            n -= 1;
            rect.fLeft = at(p);
            rect.fRight = at(p + 1);
            p += 2;
            if rect.fLeft == S
                || rect.fRight == S
                || rect.fLeft >= rect.fRight
                || (!first && rect.fLeft <= last_right)
            {
                return false;
            }
            last_right = rect.fRight;
            first = false;
            if bounds.isEmpty() {
                bounds = rect;
            } else {
                bounds.fLeft = bounds.fLeft.min(rect.fLeft);
                bounds.fTop = bounds.fTop.min(rect.fTop);
                bounds.fRight = bounds.fRight.max(rect.fRight);
                bounds.fBottom = bounds.fBottom.max(rect.fBottom);
            }
        }
        if at(p) != S {
            return false;
        }
        p += 1;
        rect.fTop = rect.fBottom;
        if p >= count {
            return false;
        }
        if at(p) == S {
            break;
        }
    }
    p += 1;
    ys == 0 && intervals == 0 && given == bounds && p == count
}
fn validate_run(runs: &[i32], bounds: SkRegionIRect, ys: i32, n: i32) -> bool {
    validate_run_data(runs.len(), |i| runs[i], bounds, ys, n)
}
// Source: src/core/SkRegion.cpp:1182-1213,1321-1400.
impl SkRegion {
    pub fn writeToMemory(&self, storage: Option<&mut [u8]>) -> usize {
        let size = if self.isEmpty() {
            4
        } else {
            20 + match &self.fRunHead {
                RunStorage::Complex(h) => 8 + 4 * h.runs.len(),
                _ => 0,
            }
        };
        let Some(storage) = storage else {
            return size;
        };
        assert!(storage.len() >= size);
        let mut p = 0;
        let mut write = |v: i32| {
            storage[p..p + 4].copy_from_slice(&v.to_ne_bytes());
            p += 4;
        };
        if self.isEmpty() {
            write(-1);
        } else {
            let n = match &self.fRunHead {
                RunStorage::Complex(h) => h.runs.len() as i32,
                _ => 0,
            };
            write(n);
            write(self.fBounds.fLeft);
            write(self.fBounds.fTop);
            write(self.fBounds.fRight);
            write(self.fBounds.fBottom);
            if let RunStorage::Complex(h) = &self.fRunHead {
                write(h.y_span_count);
                write(h.interval_count);
                for v in &h.runs {
                    write(*v);
                }
            }
        }
        p
    }
    pub fn readFromMemory(&mut self, storage: &[u8]) -> usize {
        let mut p = 0;
        let mut read = || {
            let bytes: [u8; 4] = storage.get(p..p + 4)?.try_into().ok()?;
            p += 4;
            Some(i32::from_ne_bytes(bytes))
        };
        let Some(count) = read() else {
            return 0;
        };
        if count < -1 {
            return 0;
        }
        let mut tmp = Self::new();
        if count >= 0 {
            let (Some(l), Some(t), Some(r), Some(b)) = (read(), read(), read(), read()) else {
                return 0;
            };
            tmp.fBounds = SkRegionIRect::MakeLTRB(l, t, r, b);
            if tmp.fBounds.isEmpty() {
                return 0;
            }
            if count == 0 {
                tmp.fRunHead = RunStorage::Rect;
            } else {
                let (Some(ys), Some(n)) = (read(), read()) else {
                    return 0;
                };
                let start = p;
                let Some(size) = (count as usize).checked_mul(4) else {
                    return 0;
                };
                let Some(bytes) = storage.get(start..).and_then(|b| b.get(..size)) else {
                    return 0;
                };
                // Validation precedes installing/allocating the resident RunHead.
                if !validate_run_data(
                    count as usize,
                    |i| i32::from_ne_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap()),
                    tmp.fBounds,
                    ys,
                    n,
                ) {
                    return 0;
                }
                let runs: Vec<i32> = bytes
                    .chunks_exact(4)
                    .map(|b| i32::from_ne_bytes(b.try_into().unwrap()))
                    .collect();
                tmp.fRunHead = RunStorage::Complex(Arc::new(RunHead {
                    runs,
                    y_span_count: ys,
                    interval_count: n,
                }));
                p += size;
            }
        }
        debug_assert!(tmp.isValid());
        self.swap(&mut tmp);
        p
    }
    pub fn isValid(&self) -> bool {
        match &self.fRunHead {
            RunStorage::Empty => self.fBounds == SkRegionIRect::default(),
            RunStorage::Rect => !self.fBounds.isEmpty(),
            RunStorage::Complex(h) => {
                !self.fBounds.isEmpty()
                    && validate_run(&h.runs, self.fBounds, h.y_span_count, h.interval_count)
            }
        }
    }
}

// Source: src/core/SkRegion_path.cpp:467-485.
#[derive(Clone)]
struct Edge {
    x: i32,
    y0: i32,
    y1: i32,
    flags: u8,
    next: Option<usize>,
}
impl Edge {
    fn new(x: i32, y0: i32, y1: i32) -> Self {
        debug_assert_ne!(y0, y1);
        Self {
            x,
            y0,
            y1,
            flags: 0,
            next: None,
        }
    }
    fn top(&self) -> i32 {
        self.y0.min(self.y1)
    }
}
// Source: src/core/SkRegion_path.cpp:487-534.
fn find_link(edges: &mut [Edge], base: usize) {
    if edges[base].flags == 3 {
        debug_assert!(edges[base].next.is_some());
        return;
    }
    debug_assert!(base + 1 < edges.len());
    let y0 = edges[base].y0;
    let y1 = edges[base].y1;
    if edges[base].flags & 1 == 0 {
        let mut e = base;
        loop {
            e += 1;
            if edges[e].flags & 2 == 0 && y0 == edges[e].y1 {
                debug_assert!(edges[e].next.is_none());
                edges[e].next = Some(base);
                edges[e].flags |= 2;
                break;
            }
        }
    }
    if edges[base].flags & 2 == 0 {
        let mut e = base;
        loop {
            e += 1;
            if edges[e].flags & 1 == 0 && y1 == edges[e].y0 {
                debug_assert!(edges[base].next.is_none());
                edges[base].next = Some(e);
                edges[e].flags |= 1;
                break;
            }
        }
    }
    edges[base].flags = 3;
}
// Source: src/core/SkRegion_path.cpp:536-565.
fn extract_path(edges: &mut [Edge], builder: &mut impl BoundaryPathBuilder) -> usize {
    let mut edge = 0;
    while edges[edge].flags == 0 {
        edge += 1;
    }
    let base = edge;
    let mut prev = edge;
    edge = edges[edge].next.unwrap();
    debug_assert_ne!(edge, base);
    let mut count = 1;
    builder.moveTo(edges[prev].x as f32, edges[prev].y0 as f32);
    edges[prev].flags = 0;
    loop {
        if edges[prev].x != edges[edge].x || edges[prev].y1 != edges[edge].y0 {
            builder.lineTo(edges[prev].x as f32, edges[prev].y1 as f32);
            builder.lineTo(edges[edge].x as f32, edges[edge].y0 as f32);
        }
        prev = edge;
        edge = edges[edge].next.unwrap();
        count += 1;
        edges[prev].flags = 0;
        if edge == base {
            break;
        }
    }
    builder.lineTo(edges[prev].x as f32, edges[prev].y1 as f32);
    builder.close();
    count
}
/// Exact linear boundary-path output. Unlike tiny-skia Path this retains empty
/// and scalar-degenerate contours, as official SkPath/SkPathBuilder do.
#[derive(Clone, Debug, PartialEq)]
pub enum BoundaryPathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    Close,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoundaryPath {
    pub commands: Vec<BoundaryPathCommand>,
}
pub trait BoundaryPathBuilder {
    fn moveTo(&mut self, x: f32, y: f32);
    fn lineTo(&mut self, x: f32, y: f32);
    fn close(&mut self);
    fn incReserve(&mut self, points: usize);
}
impl BoundaryPathBuilder for BoundaryPath {
    fn moveTo(&mut self, x: f32, y: f32) {
        self.commands.push(BoundaryPathCommand::MoveTo(x, y));
    }
    fn lineTo(&mut self, x: f32, y: f32) {
        self.commands.push(BoundaryPathCommand::LineTo(x, y));
    }
    fn close(&mut self) {
        self.commands.push(BoundaryPathCommand::Close);
    }
    fn incReserve(&mut self, points: usize) {
        self.commands.reserve(points);
    }
}
// Source: src/core/SkRegion_path.cpp:567-633.
impl SkRegion {
    pub fn addBoundaryPath(&self, builder: &mut impl BoundaryPathBuilder) -> bool {
        if self.isEmpty() {
            return false;
        }
        let b = self.fBounds;
        if self.isRect() {
            // Source SkPathBuilder::addRect uses clockwise top-left start.
            builder.moveTo(b.fLeft as f32, b.fTop as f32);
            builder.lineTo(b.fRight as f32, b.fTop as f32);
            builder.lineTo(b.fRight as f32, b.fBottom as f32);
            builder.lineTo(b.fLeft as f32, b.fBottom as f32);
            builder.close();
            return true;
        }
        let mut iter = Iterator::new(self);
        let mut edges = Vec::new();
        while !iter.done() {
            let r = iter.rect();
            edges.push(Edge::new(r.fLeft, r.fBottom, r.fTop));
            edges.push(Edge::new(r.fRight, r.fTop, r.fBottom));
            iter.next();
        }
        let mut count = edges.len();
        crate::src::core::SkTSort::sort(&mut edges, |a, b| {
            if a.x == b.x {
                a.top() < b.top()
            } else {
                a.x < b.x
            }
        });
        for e in 0..edges.len() {
            find_link(&mut edges, e);
        }
        debug_assert!(edges.iter().all(|e| e.next.is_some() && e.flags == 3));
        builder.incReserve(count << 1);
        loop {
            debug_assert!(count > 1);
            count -= extract_path(&mut edges, builder);
            if count == 0 {
                break;
            }
        }
        true
    }
    pub fn getBoundaryPath(&self) -> BoundaryPath {
        let mut builder = BoundaryPath::default();
        self.addBoundaryPath(&mut builder);
        builder
    }
}

// Source: include/core/SkRegion.h:385-388.
impl Op {
    pub const kLastOp: Self = Self::kReplace_Op;
}
impl SkRegion {
    pub const kOpCnt: usize = Op::kLastOp as usize + 1;
}
// Source: src/core/SkRegion.cpp:283-295.
impl SkRegion {
    fn count_runtype_values(&self) -> (i32, i32, i32) {
        let count = if self.isRect() {
            2
        } else {
            let RunStorage::Complex(head) = &self.fRunHead else {
                panic!("nonempty region required")
            };
            head.interval_count * 2
        };
        (count, self.fBounds.fTop, self.fBounds.fBottom)
    }
    // Source: src/core/SkRegion.cpp:255-277. Android-only diagnostic API.
    #[cfg(target_os = "android")]
    pub fn toString(&self) -> String {
        let mut result = String::from("SkRegion(");
        let mut it = Iterator::new(self);
        while !it.done() {
            let r = it.rect();
            result.push_str(&format!(
                "({},{},{},{})",
                r.fLeft, r.fTop, r.fRight, r.fBottom
            ));
            it.next();
        }
        result.push(')');
        result
    }
}
