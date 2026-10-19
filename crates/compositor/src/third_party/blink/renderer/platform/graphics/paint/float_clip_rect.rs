// Translation status: complete
// Copyright 2017 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:1-15.
// Includes, allocator and namespace declarations map to Rust modules.
use crate::adapter::{
    geometry::{RRectF, RectF, Vector2dF},
    paint_geometry::FloatClipRectTransform,
};
// Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:16-32,115-120.
#[derive(Clone, Copy, Debug)]
pub struct FloatClipRect {
    rect_: RectF,
    has_radius_: bool,
    is_tight_: bool,
    is_infinite_: bool,
}
impl Default for FloatClipRect {
    fn default() -> Self {
        let rect = foundation::infinite_int_rect::InfiniteIntRect();
        Self {
            rect_: RectF::new(
                rect.x() as f32,
                rect.y() as f32,
                rect.width() as f32,
                rect.height() as f32,
            ),
            has_radius_: false,
            is_tight_: true,
            is_infinite_: true,
        }
    }
}
impl FloatClipRect {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn FromRect(rect: RectF) -> Self {
        let mut value = Self::default();
        value.SetRect(rect);
        value
    }
    pub fn FromRoundedRect(rect: &RRectF) -> Self {
        let rounded = rect.corner_radii.iter().any(|r| r.x() != 0. || r.y() != 0.);
        Self {
            rect_: rect.rect,
            has_radius_: rounded,
            is_tight_: !rounded,
            is_infinite_: false,
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:34-42.
    pub fn Rect(&self) -> &RectF {
        &self.rect_
    }
    pub fn RectMut(&mut self) -> &mut RectF {
        &mut self.rect_
    }
    pub fn SetRect(&mut self, rect: RectF) {
        self.rect_ = rect;
        self.has_radius_ = false;
        self.is_tight_ = true;
        self.is_infinite_ = false;
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:44-58.
    pub fn Intersect(&mut self, other: &Self) {
        if other.is_infinite_ {
            return;
        }
        if self.is_infinite_ {
            self.is_infinite_ = other.is_infinite_;
            self.rect_ = other.rect_;
        } else {
            self.rect_.Intersect(other.rect_);
        }
        if other.HasRadius() {
            self.SetHasRadius();
        } else if !other.IsTight() {
            self.ClearIsTight();
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:60-78.
    pub fn InclusiveIntersect(&mut self, other: &Self) -> bool {
        if other.is_infinite_ {
            return true;
        }
        let mut retval = true;
        if self.is_infinite_ {
            self.is_infinite_ = other.is_infinite_;
            self.rect_ = other.rect_;
        } else {
            retval = crate::adapter::layer_tree_impl::InclusiveIntersectRectF(
                &mut self.rect_,
                other.rect_,
            );
        }
        if other.HasRadius() {
            self.SetHasRadius();
        } else if !other.IsTight() {
            self.ClearIsTight();
        }
        retval
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:80-93.
    pub fn HasRadius(&self) -> bool {
        self.has_radius_
    }
    pub fn SetHasRadius(&mut self) {
        self.has_radius_ = true;
        self.is_tight_ = false;
        self.is_infinite_ = false;
    }
    pub fn IsTight(&self) -> bool {
        debug_assert!(!self.is_tight_ || !self.has_radius_);
        self.is_tight_
    }
    pub fn ClearIsTight(&mut self) {
        self.is_tight_ = false;
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:95-113.
    pub fn Move(&mut self, offset: Vector2dF) {
        if self.is_infinite_ {
            return;
        }
        self.rect_.Offset(offset.x(), offset.y());
    }
    pub fn Map(&mut self, matrix: &impl FloatClipRectTransform) {
        if matrix.IsIdentityOr2dTranslation() {
            self.Move(matrix.To2dTranslation());
            return;
        }
        self.is_tight_ = false;
        if self.is_infinite_ {
            return;
        }
        self.rect_ = matrix.MapRect(self.rect_);
    }
    pub fn IsInfinite(&self) -> bool {
        self.is_infinite_
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:122-129.
impl PartialEq for FloatClipRect {
    fn eq(&self, other: &Self) -> bool {
        if self.IsTight() != other.IsTight() {
            return false;
        }
        if self.IsInfinite() && other.IsInfinite() {
            return true;
        }
        !self.IsInfinite()
            && !other.IsInfinite()
            && self.HasRadius() == other.HasRadius()
            && self.Rect() == other.Rect()
    }
}
// Source: third_party/blink/renderer/platform/graphics/paint/float_clip_rect.h:131-143.
pub fn InfiniteLooseFloatClipRect() -> FloatClipRect {
    let mut rect = FloatClipRect::default();
    rect.ClearIsTight();
    rect
}
