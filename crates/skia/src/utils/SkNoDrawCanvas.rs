//! Pixel-free SkCanvas state for Paint analysis. This owns no render target,
//! pixel storage, image cache, resources or compositor state.
#![allow(non_snake_case)]
use crate::{
    include::core::{SkRRect::SkRRectBounds as Rect, SkRegion::SkRegionIRect as IRect},
    src::core::SkM44::SkM44,
};
// Source: src/core/SkDevice.h:612-628; src/core/SkDevice.cpp:510-521.
#[derive(Clone, Copy)]
struct State {
    matrix: SkM44,
    clip: IRect,
    is_aa: bool,
    is_rect: bool,
}
// Source: include/utils/SkNoDrawCanvas.h:16-37,40-83;
// src/core/SkCanvas.cpp:2963-2976. Draw overrides intentionally do no drawing.
pub struct SkNoDrawCanvas {
    bounds: IRect,
    stack: Vec<State>,
}
impl SkNoDrawCanvas {
    pub fn new(width: i32, height: i32) -> Self {
        Self::from_bounds(IRect::MakeLTRB(0, 0, width, height))
    }
    pub fn from_bounds(bounds: IRect) -> Self {
        Self {
            bounds,
            stack: vec![State {
                matrix: SkM44::default(),
                clip: IRect::MakeLTRB(
                    0,
                    0,
                    bounds.fRight.wrapping_sub(bounds.fLeft),
                    bounds.fBottom.wrapping_sub(bounds.fTop),
                ),
                is_aa: false,
                is_rect: true,
            }],
        }
    }
    fn state(&self) -> &State {
        self.stack.last().unwrap()
    }
    fn state_mut(&mut self) -> &mut State {
        self.stack.last_mut().unwrap()
    }
    pub fn getBaseLayerSize(&self) -> (i32, i32) {
        (
            self.bounds.fRight.wrapping_sub(self.bounds.fLeft),
            self.bounds.fBottom.wrapping_sub(self.bounds.fTop),
        )
    }
    // Source: src/core/SkCanvas.cpp:454-464,1250-1289; deferred native saves
    // are materialized as small matrix/clip values, never pixels or scene data.
    pub fn getSaveCount(&self) -> i32 {
        self.stack.len() as i32
    }
    pub fn save(&mut self) {
        self.stack.push(*self.state())
    }
    pub fn restore(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }
    pub fn restoreToCount(&mut self, count: i32) {
        while self.getSaveCount() > count.max(1) {
            self.restore()
        }
    }
    // Source: src/core/SkCanvas.cpp:1289-1373,1660-1662.
    pub fn getLocalToDevice(&self) -> SkM44 {
        self.state().matrix
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        self.state_mut().matrix.preTranslate(x, y, 0.)
    }
    pub fn scale(&mut self, x: f32, y: f32) {
        self.state_mut().matrix.preScale(x, y, 1.)
    }
    pub fn concat(&mut self, matrix: &SkM44) {
        self.state_mut().matrix.preConcat(matrix)
    }
    pub fn setMatrix(&mut self, matrix: SkM44) {
        self.state_mut().matrix = matrix
    }
    // Source: src/core/SkCanvas.cpp:1608-1644.
    pub fn getDeviceClipBounds(&self) -> IRect {
        let c = self.state().clip;
        if c.isEmpty() {
            return IRect::default();
        }
        // Source: SkCanvas.cpp:1626-1644, deviceToGlobal is the native
        // pixel-aligned origin established by SkNoPixelsDevice's constructor.
        let ox = self.bounds.fLeft as f32;
        let oy = self.bounds.fTop as f32;
        round_rect(
            Rect::MakeLTRB(
                c.fLeft as f32 + ox,
                c.fTop as f32 + oy,
                c.fRight as f32 + ox,
                c.fBottom as f32 + oy,
            ),
            Round::Out,
        )
    }
    pub fn getLocalClipBounds(&self) -> Option<Rect> {
        let c = self.getDeviceClipBounds();
        if c.isEmpty() {
            return None;
        }
        let inverse = self.state().matrix.asM33Inverse()?;
        Some(inverse.mapRect(Rect::MakeLTRB(
            c.fLeft.saturating_sub(1) as f32,
            c.fTop.saturating_sub(1) as f32,
            c.fRight.saturating_add(1) as f32,
            c.fBottom.saturating_add(1) as f32,
        )))
    }
    pub fn isClipRect(&self) -> bool {
        !self.state().clip.isEmpty() && self.state().is_rect
    }
    // Source: src/core/SkDevice.cpp:574-588.
    pub fn clipRect(&mut self, bounds: Rect, difference: bool, aa: bool) {
        self.clipShape(bounds, difference, aa, true)
    }
    pub fn clipRRect(&mut self, bounds: Rect, is_rect: bool, difference: bool, aa: bool) {
        self.clipShape(bounds, difference, aa, is_rect)
    }
    pub fn clipPathBounds(&mut self, bounds: Rect, inverse_fill: bool, difference: bool, aa: bool) {
        self.clipShape(bounds, difference ^ inverse_fill, aa, false)
    }
    // Source: src/core/SkDevice.cpp:615-645. NoPixelsDevice deliberately
    // computes conservative clip bounds; no masks or paths are rasterized.
    fn clipShape(&mut self, bounds: Rect, difference: bool, aa: bool, fills_bounds: bool) {
        let ox = self.bounds.fLeft as f32;
        let oy = self.bounds.fTop as f32;
        let s = self.state_mut();
        let mut device_matrix = s.matrix;
        device_matrix.postTranslate(-ox, -oy, 0.);
        let is_rect = fills_bounds && device_matrix.isScaleTranslateAsM33();
        s.is_aa |= aa;
        let dev = if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
            Rect::default()
        } else {
            device_matrix.mapRect(bounds)
        };
        if !difference {
            let r = round_rect(dev, if aa { Round::Out } else { Round::Nearest });
            let old = s.clip;
            let mut clipped = IRect::default();
            if clipped.intersect(&old, &r) {
                s.clip = clipped
            } else {
                s.clip = IRect::default()
            }
            s.is_rect &= is_rect;
        } else if is_rect {
            let r = round_rect(dev, if aa { Round::In } else { Round::Nearest });
            if let Some(diff) = subtract(s.clip, r) {
                s.clip = diff
            } else {
                s.is_rect = false
            }
        } else {
            s.is_rect = false
        }
    }
}
enum Round {
    Nearest,
    Out,
    In,
}
// Source: include/core/SkRect.h:1521-1535; include/private/SkFloatingPoint.h:
// saturating float-to-int conversions retain empty/large native rectangles.
fn round_rect(r: Rect, mode: Round) -> IRect {
    let nearest = |x: f32| (x + 0.5).floor() as i32;
    let v = match mode {
        Round::Nearest => [
            nearest(r.left),
            nearest(r.top),
            nearest(r.right),
            nearest(r.bottom),
        ],
        Round::Out => [
            r.left.floor() as i32,
            r.top.floor() as i32,
            r.right.ceil() as i32,
            r.bottom.ceil() as i32,
        ],
        Round::In => [
            r.left.ceil() as i32,
            r.top.ceil() as i32,
            r.right.floor() as i32,
            r.bottom.floor() as i32,
        ],
    };
    IRect::MakeLTRB(v[0], v[1], v[2], v[3])
}
// Source: src/core/SkRect.cpp:210-295 (SkRectPriv::Subtract). Return None when
// the native difference cannot be represented exactly by one rectangle.
fn subtract(a: IRect, b: IRect) -> Option<IRect> {
    if !IRect::Intersects(&a, &b) {
        return Some(a);
    }
    let ah = a.fBottom.wrapping_sub(a.fTop) as f32;
    let aw = a.fRight.wrapping_sub(a.fLeft) as f32;
    let (mut left, mut right, mut top, mut bottom) = (0., 0., 0., 0.);
    let mut count = 0;
    if b.fLeft > a.fLeft {
        left = b.fLeft.wrapping_sub(a.fLeft) as f32 / aw;
        count += 1
    }
    if a.fRight > b.fRight {
        right = a.fRight.wrapping_sub(b.fRight) as f32 / aw;
        count += 1
    }
    if b.fTop > a.fTop {
        top = b.fTop.wrapping_sub(a.fTop) as f32 / ah;
        count += 1
    }
    if a.fBottom > b.fBottom {
        bottom = a.fBottom.wrapping_sub(b.fBottom) as f32 / ah;
        count += 1
    }
    if count == 0 {
        return Some(IRect::default());
    }
    let mut out = a;
    if left > right && left > top && left > bottom {
        out.fRight = b.fLeft
    } else if right > top && right > bottom {
        out.fLeft = b.fRight
    } else if top > bottom {
        out.fBottom = b.fTop
    } else {
        out.fTop = b.fBottom
    }
    if count == 1 {
        Some(out)
    } else {
        None
    }
}
