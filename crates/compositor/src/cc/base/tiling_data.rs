// Translation status: complete
// Copyright 2010 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/tiling_data.h:1-25,185-188; cc/base/tiling_data.cc:1-20,599.
// Rust modules replace include guards, includes, forward declarations and namespaces.
use super::{
    index_rect::IndexRect, reverse_spiral_iterator::ReverseSpiralIterator,
    spiral_iterator::SpiralIterator,
};
use crate::adapter::geometry::{Rect, RectF, Size, Vector2d};
use std::ops::{Deref, DerefMut};
// Source: cc/base/tiling_data.cc:17-20.
const kNonPositiveQuadrantIndexRect: IndexRect = IndexRect::new(-1, -1, -1, -1);
// Source: cc/base/tiling_data.cc:22-32.
fn ComputeNumTiles(max_texture_size: i32, total_size: i32, border_texels: i32) -> i32 {
    if max_texture_size - 2 * border_texels <= 0 {
        return if total_size > 0 && max_texture_size >= total_size {
            1
        } else {
            0
        };
    }
    let num_tiles =
        1.max(1 + (total_size - 1 - 2 * border_texels) / (max_texture_size - 2 * border_texels));
    if total_size > 0 { num_tiles } else { 0 }
}
// Source: cc/base/tiling_data.h:25-33,166-183.
// The deleted bool constructor is enforced by the i32 border_texels parameter;
// Rust has no implicit bool-to-int conversion. Source implicit value copy is derived.
#[derive(Clone, Copy, Debug)]
pub struct TilingData {
    max_texture_size_: Size,
    tiling_rect_: Rect,
    border_texels_: i32,
    num_tiles_x_: i32,
    num_tiles_y_: i32,
}
impl Default for TilingData {
    // Source: cc/base/tiling_data.cc:34-36.
    fn default() -> Self {
        let mut result = Self {
            max_texture_size_: Size::default(),
            tiling_rect_: Rect::default(),
            border_texels_: 0,
            num_tiles_x_: 0,
            num_tiles_y_: 0,
        };
        result.RecomputeNumTiles();
        result
    }
}
impl TilingData {
    // Source: cc/base/tiling_data.cc:38-43.
    pub fn new(max_texture_size: &Size, tiling_rect: &Rect, border_texels: i32) -> Self {
        let mut result = Self {
            max_texture_size_: *max_texture_size,
            tiling_rect_: Rect::default(),
            border_texels_: border_texels,
            num_tiles_x_: 0,
            num_tiles_y_: 0,
        };
        result.SetTilingRect(tiling_rect);
        result
    }
    // Source: cc/base/tiling_data.h:35-45.
    pub fn tiling_rect(&self) -> &Rect {
        &self.tiling_rect_
    }
    pub fn max_texture_size(&self) -> Size {
        self.max_texture_size_
    }
    pub fn border_texels(&self) -> i32 {
        self.border_texels_
    }
    pub fn has_empty_bounds(&self) -> bool {
        self.num_tiles_x_ == 0 || self.num_tiles_y_ == 0
    }
    pub fn num_tiles_x(&self) -> i32 {
        self.num_tiles_x_
    }
    pub fn num_tiles_y(&self) -> i32 {
        self.num_tiles_y_
    }
    // Source: cc/base/tiling_data.cc:45-50.
    pub fn SetTilingRect(&mut self, tiling_rect: &Rect) {
        debug_assert!(tiling_rect.x >= 0);
        debug_assert!(tiling_rect.y >= 0);
        self.tiling_rect_ = *tiling_rect;
        self.RecomputeNumTiles();
    }
    // Source: cc/base/tiling_data.cc:52-55.
    pub fn SetMaxTextureSize(&mut self, max_texture_size: &Size) {
        self.max_texture_size_ = *max_texture_size;
        self.RecomputeNumTiles();
    }
    // Source: cc/base/tiling_data.cc:57-65.
    pub fn TileXIndexFromSrcCoord(&self, src_position: i32) -> i32 {
        if self.num_tiles_x_ <= 1 {
            return 0;
        }
        debug_assert!(self.max_texture_size_.width() - 2 * self.border_texels_ > 0);
        let x = (src_position - self.tiling_rect_.x - self.border_texels_)
            / (self.max_texture_size_.width() - 2 * self.border_texels_);
        x.clamp(0, self.num_tiles_x_ - 1)
    }
    // Source: cc/base/tiling_data.cc:67-75.
    pub fn TileYIndexFromSrcCoord(&self, src_position: i32) -> i32 {
        if self.num_tiles_y_ <= 1 {
            return 0;
        }
        debug_assert!(self.max_texture_size_.height() - 2 * self.border_texels_ > 0);
        let y = (src_position - self.tiling_rect_.y - self.border_texels_)
            / (self.max_texture_size_.height() - 2 * self.border_texels_);
        y.clamp(0, self.num_tiles_y_ - 1)
    }
    // Source: cc/base/tiling_data.cc:77-86.
    pub fn FirstBorderTileXIndexFromSrcCoord(&self, src_position: i32) -> i32 {
        if self.num_tiles_x_ <= 1 {
            return 0;
        }
        debug_assert!(self.max_texture_size_.width() - 2 * self.border_texels_ > 0);
        let inner_tile_size = self.max_texture_size_.width() - 2 * self.border_texels_;
        let x = (src_position - self.tiling_rect_.x - 2 * self.border_texels_) / inner_tile_size;
        x.clamp(0, self.num_tiles_x_ - 1)
    }
    // Source: cc/base/tiling_data.cc:88-97.
    pub fn FirstBorderTileYIndexFromSrcCoord(&self, src_position: i32) -> i32 {
        if self.num_tiles_y_ <= 1 {
            return 0;
        }
        debug_assert!(self.max_texture_size_.height() - 2 * self.border_texels_ > 0);
        let inner_tile_size = self.max_texture_size_.height() - 2 * self.border_texels_;
        let y = (src_position - self.tiling_rect_.y - 2 * self.border_texels_) / inner_tile_size;
        y.clamp(0, self.num_tiles_y_ - 1)
    }
    // Source: cc/base/tiling_data.cc:99-107.
    pub fn LastBorderTileXIndexFromSrcCoord(&self, src_position: i32) -> i32 {
        if self.num_tiles_x_ <= 1 {
            return 0;
        }
        debug_assert!(self.max_texture_size_.width() - 2 * self.border_texels_ > 0);
        let inner_tile_size = self.max_texture_size_.width() - 2 * self.border_texels_;
        let x = (src_position - self.tiling_rect_.x) / inner_tile_size;
        x.clamp(0, self.num_tiles_x_ - 1)
    }
    // Source: cc/base/tiling_data.cc:109-117.
    pub fn LastBorderTileYIndexFromSrcCoord(&self, src_position: i32) -> i32 {
        if self.num_tiles_y_ <= 1 {
            return 0;
        }
        debug_assert!(self.max_texture_size_.height() - 2 * self.border_texels_ > 0);
        let inner_tile_size = self.max_texture_size_.height() - 2 * self.border_texels_;
        let y = (src_position - self.tiling_rect_.y) / inner_tile_size;
        y.clamp(0, self.num_tiles_y_ - 1)
    }
    // Source: cc/base/tiling_data.cc:119-163.
    pub fn TileAroundIndexRect(&self, center_rect: &Rect) -> IndexRect {
        let around_left;
        if center_rect.x < self.tiling_rect_.x || center_rect.IsEmpty() {
            around_left = -1;
        } else if center_rect.x >= self.tiling_rect_.right() {
            around_left = self.num_tiles_x();
        } else {
            around_left = self.TileXIndexFromSrcCoord(center_rect.x);
        }
        let around_top;
        if center_rect.y < self.tiling_rect_.y || center_rect.IsEmpty() {
            around_top = -1;
        } else if center_rect.y >= self.tiling_rect_.bottom() {
            around_top = self.num_tiles_y();
        } else {
            around_top = self.TileYIndexFromSrcCoord(center_rect.y);
        }
        let around_right;
        let right_src_coord = center_rect.right() - 1;
        if right_src_coord < self.tiling_rect_.x || center_rect.IsEmpty() {
            around_right = -1;
        } else if right_src_coord >= self.tiling_rect_.right() {
            around_right = self.num_tiles_x();
        } else {
            around_right = self.TileXIndexFromSrcCoord(right_src_coord);
        }
        let around_bottom;
        let bottom_src_coord = center_rect.bottom() - 1;
        if bottom_src_coord < self.tiling_rect_.y || center_rect.IsEmpty() {
            around_bottom = -1;
        } else if bottom_src_coord >= self.tiling_rect_.bottom() {
            around_bottom = self.num_tiles_y();
        } else {
            around_bottom = self.TileYIndexFromSrcCoord(bottom_src_coord);
        }
        IndexRect::new(around_left, around_right, around_top, around_bottom)
    }
    // Source: cc/base/tiling_data.cc:165-181.
    pub fn ExpandRectToTileBounds(&self, rect: &Rect) -> Rect {
        if rect.IsEmpty() || self.has_empty_bounds() {
            return Rect::default();
        }
        if rect.x > self.tiling_rect_.right() || rect.y > self.tiling_rect_.bottom() {
            return Rect::default();
        }
        let index_x = self.FirstBorderTileXIndexFromSrcCoord(rect.x);
        let index_y = self.FirstBorderTileYIndexFromSrcCoord(rect.y);
        let index_right = self.LastBorderTileXIndexFromSrcCoord(rect.right() - 1);
        let index_bottom = self.LastBorderTileYIndexFromSrcCoord(rect.bottom() - 1);
        let mut rect_top_left = self.TileBounds(index_x, index_y);
        let rect_bottom_right = self.TileBounds(index_right, index_bottom);
        rect_top_left.Union(rect_bottom_right);
        rect_top_left
    }
    // Source: cc/base/tiling_data.cc:183-218.
    pub fn TileBounds(&self, i: i32, j: i32) -> Rect {
        self.AssertTile(i, j);
        let max_texture_size_x = self.max_texture_size_.width() - 2 * self.border_texels_;
        let max_texture_size_y = self.max_texture_size_.height() - 2 * self.border_texels_;
        let mut lo_x = max_texture_size_x * i;
        if i != 0 {
            lo_x += self.border_texels_;
        }
        let mut lo_y = max_texture_size_y * j;
        if j != 0 {
            lo_y += self.border_texels_;
        }
        let mut hi_x = max_texture_size_x * (i + 1) + self.border_texels_;
        if i + 1 == self.num_tiles_x_ {
            hi_x += self.border_texels_;
        }
        let mut hi_y = max_texture_size_y * (j + 1) + self.border_texels_;
        if j + 1 == self.num_tiles_y_ {
            hi_y += self.border_texels_;
        }
        hi_x = hi_x.min(self.tiling_rect_.width);
        hi_y = hi_y.min(self.tiling_rect_.height);
        let x = lo_x;
        let y = lo_y;
        let width = hi_x - lo_x;
        let height = hi_y - lo_y;
        debug_assert!(x >= 0);
        debug_assert!(y >= 0);
        debug_assert!(width >= 0);
        debug_assert!(height >= 0);
        debug_assert!(x <= self.tiling_rect_.width);
        debug_assert!(y <= self.tiling_rect_.height);
        Rect::new(
            x + self.tiling_rect_.x,
            y + self.tiling_rect_.y,
            width,
            height,
        )
    }
    // Source: cc/base/tiling_data.cc:220-245.
    pub fn TileBoundsWithBorder(&self, i: i32, j: i32) -> Rect {
        self.AssertTile(i, j);
        let max_texture_size_x = self.max_texture_size_.width() - 2 * self.border_texels_;
        let max_texture_size_y = self.max_texture_size_.height() - 2 * self.border_texels_;
        let lo_x = max_texture_size_x * i;
        let lo_y = max_texture_size_y * j;
        let mut hi_x = lo_x + max_texture_size_x + 2 * self.border_texels_;
        let mut hi_y = lo_y + max_texture_size_y + 2 * self.border_texels_;
        hi_x = hi_x.min(self.tiling_rect_.width);
        hi_y = hi_y.min(self.tiling_rect_.height);
        let x = lo_x;
        let y = lo_y;
        let width = hi_x - lo_x;
        let height = hi_y - lo_y;
        debug_assert!(x >= 0);
        debug_assert!(y >= 0);
        debug_assert!(width >= 0);
        debug_assert!(height >= 0);
        debug_assert!(x <= self.tiling_rect_.width);
        debug_assert!(y <= self.tiling_rect_.height);
        Rect::new(
            x + self.tiling_rect_.x,
            y + self.tiling_rect_.y,
            width,
            height,
        )
    }
    // Source: cc/base/tiling_data.cc:247-256.
    pub fn TilePositionX(&self, x_index: i32) -> i32 {
        debug_assert!(x_index >= 0);
        debug_assert!(x_index < self.num_tiles_x_);
        let mut pos = (self.max_texture_size_.width() - 2 * self.border_texels_) * x_index;
        if x_index != 0 {
            pos += self.border_texels_;
        }
        pos + self.tiling_rect_.x
    }
    // Source: cc/base/tiling_data.cc:258-267.
    pub fn TilePositionY(&self, y_index: i32) -> i32 {
        debug_assert!(y_index >= 0);
        debug_assert!(y_index < self.num_tiles_y_);
        let mut pos = (self.max_texture_size_.height() - 2 * self.border_texels_) * y_index;
        if y_index != 0 {
            pos += self.border_texels_;
        }
        pos + self.tiling_rect_.y
    }
    // Source: cc/base/tiling_data.cc:269-287.
    pub fn TileSizeX(&self, x_index: i32) -> i32 {
        debug_assert!(x_index >= 0);
        debug_assert!(x_index < self.num_tiles_x_);
        if x_index == 0 && self.num_tiles_x_ == 1 {
            return self.tiling_rect_.width;
        }
        if x_index == 0 && self.num_tiles_x_ > 1 {
            return self.max_texture_size_.width() - self.border_texels_;
        }
        if x_index < self.num_tiles_x_ - 1 {
            return self.max_texture_size_.width() - 2 * self.border_texels_;
        }
        if x_index == self.num_tiles_x_ - 1 {
            return self.tiling_rect_.right() - self.TilePositionX(x_index);
        }
        unreachable!("NOTREACHED: tile index outside TilingData");
    }
    // Source: cc/base/tiling_data.cc:289-307.
    pub fn TileSizeY(&self, y_index: i32) -> i32 {
        debug_assert!(y_index >= 0);
        debug_assert!(y_index < self.num_tiles_y_);
        if y_index == 0 && self.num_tiles_y_ == 1 {
            return self.tiling_rect_.height;
        }
        if y_index == 0 && self.num_tiles_y_ > 1 {
            return self.max_texture_size_.height() - self.border_texels_;
        }
        if y_index < self.num_tiles_y_ - 1 {
            return self.max_texture_size_.height() - 2 * self.border_texels_;
        }
        if y_index == self.num_tiles_y_ - 1 {
            return self.tiling_rect_.bottom() - self.TilePositionY(y_index);
        }
        unreachable!("NOTREACHED: tile index outside TilingData");
    }
    // Source: cc/base/tiling_data.cc:309-313.
    pub fn TexelExtent(&self, i: i32, j: i32) -> RectF {
        let tile = self.TileBoundsWithBorder(i, j);
        let mut result = RectF::new(
            tile.x as f32,
            tile.y as f32,
            tile.width as f32,
            tile.height as f32,
        );
        result.Inset(0.5, 0.5, 0.5, 0.5);
        result
    }
    // Source: cc/base/tiling_data.cc:315-320.
    pub fn TextureOffset(&self, x_index: i32, y_index: i32) -> Vector2d {
        let left = if x_index == 0 || self.num_tiles_x_ == 1 {
            0
        } else {
            self.border_texels_
        };
        let top = if y_index == 0 || self.num_tiles_y_ == 1 {
            0
        } else {
            self.border_texels_
        };
        Vector2d::new(left, top)
    }
    // Source: cc/base/tiling_data.h:168-173.
    fn AssertTile(&self, i: i32, j: i32) {
        debug_assert!(i >= 0);
        debug_assert!(i < self.num_tiles_x_);
        debug_assert!(j >= 0);
        debug_assert!(j < self.num_tiles_y_);
    }
    // Source: cc/base/tiling_data.cc:322-327.
    fn RecomputeNumTiles(&mut self) {
        self.num_tiles_x_ = ComputeNumTiles(
            self.max_texture_size_.width(),
            self.tiling_rect_.width,
            self.border_texels_,
        );
        self.num_tiles_y_ = ComputeNumTiles(
            self.max_texture_size_.height(),
            self.tiling_rect_.height,
            self.border_texels_,
        );
    }
}
// Source: cc/base/tiling_data.h:78-97.
#[derive(Clone, Copy, Debug)]
pub struct BaseIterator {
    index_x_: i32,
    index_y_: i32,
}
impl Default for BaseIterator {
    // Source: cc/base/tiling_data.cc:329-330.
    fn default() -> Self {
        Self {
            index_x_: -1,
            index_y_: -1,
        }
    }
}
impl BaseIterator {
    // Source: cc/base/tiling_data.h:80-86.
    pub fn IsValid(&self) -> bool {
        self.index_x_ != -1 && self.index_y_ != -1
    }
    pub fn index_x(&self) -> i32 {
        self.index_x_
    }
    pub fn index_y(&self) -> i32 {
        self.index_y_
    }
    pub fn index(&self) -> (i32, i32) {
        (self.index_x_, self.index_y_)
    }
    // Source: cc/base/tiling_data.h:90-93.
    fn done(&mut self) {
        self.index_x_ = -1;
        self.index_y_ = -1;
    }
}
// Source: cc/base/tiling_data.h:99-110.
// Composition/Deref maps source base-class fields and methods, without a second
// index state. Advance is the source preincrement operator returning *this.
#[derive(Clone, Copy, Debug)]
pub struct Iterator {
    base: BaseIterator,
    index_rect_: IndexRect,
}
impl Deref for Iterator {
    type Target = BaseIterator;
    fn deref(&self) -> &BaseIterator {
        &self.base
    }
}
impl DerefMut for Iterator {
    fn deref_mut(&mut self) -> &mut BaseIterator {
        &mut self.base
    }
}
impl Default for Iterator {
    // Source: cc/base/tiling_data.cc:332-334.
    fn default() -> Self {
        let mut result = Self {
            base: BaseIterator::default(),
            index_rect_: kNonPositiveQuadrantIndexRect,
        };
        result.done();
        result
    }
}
impl Iterator {
    // Source: cc/base/tiling_data.cc:336-377.
    pub fn new(tiling_data: &TilingData, consider_rect: &Rect, include_borders: bool) -> Self {
        let mut result = Self {
            base: BaseIterator::default(),
            index_rect_: kNonPositiveQuadrantIndexRect,
        };
        if tiling_data.num_tiles_x() <= 0 || tiling_data.num_tiles_y() <= 0 {
            result.done();
            return result;
        }
        let mut rect = *consider_rect;
        rect.Intersect(*tiling_data.tiling_rect());
        if rect.IsEmpty() {
            result.done();
            return result;
        }
        let top_left_tile;
        if include_borders {
            result.index_x_ = tiling_data.FirstBorderTileXIndexFromSrcCoord(rect.x);
            result.index_y_ = tiling_data.FirstBorderTileYIndexFromSrcCoord(rect.y);
            result.index_rect_ = IndexRect::new(
                result.index_x_,
                tiling_data.LastBorderTileXIndexFromSrcCoord(rect.right() - 1),
                result.index_y_,
                tiling_data.LastBorderTileYIndexFromSrcCoord(rect.bottom() - 1),
            );
            debug_assert!(result.index_rect_.is_valid());
            top_left_tile = tiling_data.TileBoundsWithBorder(result.index_x_, result.index_y_);
        } else {
            result.index_x_ = tiling_data.TileXIndexFromSrcCoord(rect.x);
            result.index_y_ = tiling_data.TileYIndexFromSrcCoord(rect.y);
            result.index_rect_ = IndexRect::new(
                result.index_x_,
                tiling_data.TileXIndexFromSrcCoord(rect.right() - 1),
                result.index_y_,
                tiling_data.TileYIndexFromSrcCoord(rect.bottom() - 1),
            );
            debug_assert!(result.index_rect_.is_valid());
            top_left_tile = tiling_data.TileBounds(result.index_x_, result.index_y_);
        }
        // Source index functions return valid indices even when rects do not intersect.
        if !top_left_tile.Intersects(rect) {
            result.done();
        }
        result
    }
    // Source: cc/base/tiling_data.cc:379-392.
    pub fn Advance(&mut self) -> &mut Self {
        if !self.IsValid() {
            return self;
        }
        self.index_x_ += 1;
        if self.index_x_ > self.index_rect_.right() {
            self.index_x_ = self.index_rect_.left();
            self.index_y_ += 1;
            if self.index_y_ > self.index_rect_.bottom() {
                self.done();
            }
        }
        self
    }
}
// Source: cc/base/tiling_data.h:112-123.
#[derive(Clone, Copy, Debug)]
pub struct BaseDifferenceIterator {
    base: BaseIterator,
    consider_index_rect_: IndexRect,
    ignore_index_rect_: IndexRect,
}
impl Deref for BaseDifferenceIterator {
    type Target = BaseIterator;
    fn deref(&self) -> &BaseIterator {
        &self.base
    }
}
impl DerefMut for BaseDifferenceIterator {
    fn deref_mut(&mut self) -> &mut BaseIterator {
        &mut self.base
    }
}
impl Default for BaseDifferenceIterator {
    // Source: cc/base/tiling_data.cc:394-398.
    fn default() -> Self {
        let mut result = Self {
            base: BaseIterator::default(),
            consider_index_rect_: kNonPositiveQuadrantIndexRect,
            ignore_index_rect_: kNonPositiveQuadrantIndexRect,
        };
        result.done();
        result
    }
}
impl BaseDifferenceIterator {
    // Source: cc/base/tiling_data.cc:400-450.
    fn new(tiling_data: &TilingData, consider_rect: &Rect, ignore_rect: &Rect) -> Self {
        let mut result = Self {
            base: BaseIterator::default(),
            consider_index_rect_: kNonPositiveQuadrantIndexRect,
            ignore_index_rect_: kNonPositiveQuadrantIndexRect,
        };
        if tiling_data.num_tiles_x() <= 0 || tiling_data.num_tiles_y() <= 0 {
            result.done();
            return result;
        }
        let mut consider = *consider_rect;
        consider.Intersect(*tiling_data.tiling_rect());
        if consider.IsEmpty() {
            result.done();
            return result;
        }
        result.consider_index_rect_ = IndexRect::new(
            tiling_data.TileXIndexFromSrcCoord(consider.x),
            tiling_data.TileXIndexFromSrcCoord(consider.right() - 1),
            tiling_data.TileYIndexFromSrcCoord(consider.y),
            tiling_data.TileYIndexFromSrcCoord(consider.bottom() - 1),
        );
        debug_assert!(result.consider_index_rect_.is_valid());
        let mut ignore = *ignore_rect;
        ignore.Intersect(*tiling_data.tiling_rect());
        if !ignore.IsEmpty() {
            result.ignore_index_rect_ = IndexRect::new(
                tiling_data.TileXIndexFromSrcCoord(ignore.x),
                tiling_data.TileXIndexFromSrcCoord(ignore.right() - 1),
                tiling_data.TileYIndexFromSrcCoord(ignore.y),
                tiling_data.TileYIndexFromSrcCoord(ignore.bottom() - 1),
            );
            debug_assert!(result.ignore_index_rect_.is_valid());
            result
                .ignore_index_rect_
                .ClampTo(&result.consider_index_rect_);
            if !result.ignore_index_rect_.is_valid() {
                result.ignore_index_rect_ = kNonPositiveQuadrantIndexRect;
            }
            if result.ignore_index_rect_ == result.consider_index_rect_ {
                result.consider_index_rect_ = kNonPositiveQuadrantIndexRect;
                result.done();
                return result;
            }
        }
        result
    }
    // Source: cc/base/tiling_data.cc:452-457.
    fn HasConsiderRect(&self) -> bool {
        debug_assert!(
            self.consider_index_rect_.is_in_positive_quadrant()
                || self.consider_index_rect_ == kNonPositiveQuadrantIndexRect
        );
        self.consider_index_rect_.left() != -1
    }
}
// Source: cc/base/tiling_data.h:125-134.
#[derive(Clone, Copy, Debug, Default)]
pub struct DifferenceIterator {
    base: BaseDifferenceIterator,
}
impl Deref for DifferenceIterator {
    type Target = BaseDifferenceIterator;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for DifferenceIterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
// Source: cc/base/tiling_data.cc:459. Derived Default delegates to the base constructor.
impl DifferenceIterator {
    // Source: cc/base/tiling_data.cc:461-476.
    pub fn new(tiling_data: &TilingData, consider_rect: &Rect, ignore_rect: &Rect) -> Self {
        let mut result = Self {
            base: BaseDifferenceIterator::new(tiling_data, consider_rect, ignore_rect),
        };
        if !result.HasConsiderRect() {
            result.done();
            return result;
        }
        result.index_x_ = result.consider_index_rect_.left();
        result.index_y_ = result.consider_index_rect_.top();
        if result
            .ignore_index_rect_
            .Contains(result.index_x_, result.index_y_)
        {
            result.Advance();
        }
        result
    }
    // Source: cc/base/tiling_data.cc:478-506.
    pub fn Advance(&mut self) -> &mut Self {
        if !self.IsValid() {
            return self;
        }
        self.index_x_ += 1;
        if self
            .ignore_index_rect_
            .Contains(self.index_x_, self.index_y_)
        {
            self.index_x_ = self.ignore_index_rect_.right() + 1;
        }
        if self.index_x_ > self.consider_index_rect_.right() {
            self.index_x_ = self.consider_index_rect_.left();
            self.index_y_ += 1;
            if self
                .ignore_index_rect_
                .Contains(self.index_x_, self.index_y_)
            {
                self.index_x_ = self.ignore_index_rect_.right() + 1;
                // Source ignore rectangle may span the entire consider row.
                if self
                    .ignore_index_rect_
                    .Contains(self.index_x_, self.index_y_)
                    || self.index_x_ > self.consider_index_rect_.right()
                {
                    self.index_y_ = self.ignore_index_rect_.bottom() + 1;
                    self.index_x_ = self.consider_index_rect_.left();
                }
            }
            if self.index_y_ > self.consider_index_rect_.bottom() {
                self.done();
            }
        }
        self
    }
}
// Source: cc/base/tiling_data.h:136-151.
pub struct SpiralDifferenceIterator {
    base: BaseDifferenceIterator,
    spiral_iterator_: SpiralIterator,
}
impl Deref for SpiralDifferenceIterator {
    type Target = BaseDifferenceIterator;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for SpiralDifferenceIterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl Default for SpiralDifferenceIterator {
    // Source: cc/base/tiling_data.cc:508-510.
    fn default() -> Self {
        let mut result = Self {
            base: BaseDifferenceIterator::default(),
            spiral_iterator_: SpiralIterator::default(),
        };
        result.done();
        result
    }
}
impl SpiralDifferenceIterator {
    // Source: cc/base/tiling_data.cc:512-536.
    pub fn new(
        tiling_data: &TilingData,
        consider_rect: &Rect,
        ignore_rect: &Rect,
        center_rect: &Rect,
    ) -> Self {
        let mut result = Self {
            base: BaseDifferenceIterator::new(tiling_data, consider_rect, ignore_rect),
            spiral_iterator_: SpiralIterator::default(),
        };
        if !result.HasConsiderRect() {
            result.done();
            return result;
        }
        let around_index_rect = tiling_data.TileAroundIndexRect(center_rect);
        debug_assert!(around_index_rect.is_valid());
        result.spiral_iterator_ = SpiralIterator::new(
            around_index_rect,
            result.consider_index_rect_,
            result.ignore_index_rect_,
        );
        if !result.spiral_iterator_.IsValid() {
            result.done();
            return result;
        }
        result.index_x_ = result.spiral_iterator_.index_x();
        result.index_y_ = result.spiral_iterator_.index_y();
        result
    }
    // Source: cc/base/tiling_data.cc:538-551.
    pub fn Advance(&mut self) -> &mut Self {
        self.spiral_iterator_.Advance();
        if !self.spiral_iterator_.IsValid() {
            self.done();
            return self;
        }
        self.index_x_ = self.spiral_iterator_.index_x();
        self.index_y_ = self.spiral_iterator_.index_y();
        self
    }
}
// Source: cc/base/tiling_data.h:153-165.
pub struct ReverseSpiralDifferenceIterator {
    base: BaseDifferenceIterator,
    reverse_spiral_iterator_: ReverseSpiralIterator,
}
impl Deref for ReverseSpiralDifferenceIterator {
    type Target = BaseDifferenceIterator;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for ReverseSpiralDifferenceIterator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl Default for ReverseSpiralDifferenceIterator {
    // Source: cc/base/tiling_data.cc:553-555.
    fn default() -> Self {
        let mut result = Self {
            base: BaseDifferenceIterator::default(),
            reverse_spiral_iterator_: ReverseSpiralIterator::default(),
        };
        result.done();
        result
    }
}
impl ReverseSpiralDifferenceIterator {
    // Source: cc/base/tiling_data.cc:557-581.
    pub fn new(
        tiling_data: &TilingData,
        consider_rect: &Rect,
        ignore_rect: &Rect,
        center_rect: &Rect,
    ) -> Self {
        let mut result = Self {
            base: BaseDifferenceIterator::new(tiling_data, consider_rect, ignore_rect),
            reverse_spiral_iterator_: ReverseSpiralIterator::default(),
        };
        if !result.HasConsiderRect() {
            result.done();
            return result;
        }
        let around_index_rect = tiling_data.TileAroundIndexRect(center_rect);
        debug_assert!(around_index_rect.is_valid());
        result.reverse_spiral_iterator_ = ReverseSpiralIterator::new(
            around_index_rect,
            result.consider_index_rect_,
            result.ignore_index_rect_,
        );
        if !result.reverse_spiral_iterator_.IsValid() {
            result.done();
            return result;
        }
        result.index_x_ = result.reverse_spiral_iterator_.index_x();
        result.index_y_ = result.reverse_spiral_iterator_.index_y();
        result
    }
    // Source: cc/base/tiling_data.cc:583-597.
    pub fn Advance(&mut self) -> &mut Self {
        self.reverse_spiral_iterator_.Advance();
        if !self.reverse_spiral_iterator_.IsValid() {
            self.done();
            return self;
        }
        self.index_x_ = self.reverse_spiral_iterator_.index_x();
        self.index_y_ = self.reverse_spiral_iterator_.index_y();
        self
    }
}
// Source: cc/base/tiling_data.h:47-78,175-184.
// Public function declarations map to the same-name bodies above. The nested
// C++ iterator classes map to this same Rust module; preincrement uses Advance.
