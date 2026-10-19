// Translation status: complete
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tiling_coverage_iterator.h:1-35,251-255.
// Rust modules replace includes/namespace/export declarations. The source Tile
// alias is T::Tile, constrained by the translated Tiling concept.
use super::{tile_index::TileIndex, tiling_internal::Tiling};
use crate::adapter::geometry::{
    AxisTransform2d, Rect, RectF, ScaleToEnclosingRect, Size, ToEnclosedRect, ToEnclosingRect,
    Vector2dF,
};

// Source: cc/tiles/tiling_coverage_iterator.h:241-250.
// Native non-owning pointers are lifetime-bound live tiling/tile references.
// The copied transform and geometry values are exactly the source iterator's
// coordinate-conversion/coverage state, not copies of a tiling or tile.
pub struct TilingCoverageIterator<'a, T: Tiling> {
    tiling_: Option<&'a T>,
    coverage_rect_max_bounds_: Rect,
    coverage_rect_: Rect,
    coverage_to_content_: T::RasterTransform,
    top_left_: TileIndex,
    bottom_right_: TileIndex,
    index_: TileIndex,
    geometry_rect_: Rect,
    current_tile_: Option<&'a T::Tile>,
}

// Source: cc/tiles/tiling_coverage_iterator.h:37,75-77.
impl<T: Tiling> Default for TilingCoverageIterator<'_, T> {
    fn default() -> Self {
        Self {
            tiling_: None,
            coverage_rect_max_bounds_: Rect::default(),
            coverage_rect_: Rect::default(),
            coverage_to_content_: T::RasterTransform::default(),
            top_left_: TileIndex::default(),
            bottom_right_: TileIndex::default(),
            index_: TileIndex::default(),
            geometry_rect_: Rect::default(),
            current_tile_: None,
        }
    }
}
// Source: cc/tiles/tiling_coverage_iterator.h:75-77.
// This copies only source pointer identities and value fields; T/Tile need not
// implement Clone. The compiler's default drop releases no tiling/tile owners.
impl<T: Tiling> Clone for TilingCoverageIterator<'_, T> {
    fn clone(&self) -> Self {
        Self {
            tiling_: self.tiling_,
            coverage_rect_max_bounds_: self.coverage_rect_max_bounds_,
            coverage_rect_: self.coverage_rect_,
            coverage_to_content_: self.coverage_to_content_.clone(),
            top_left_: self.top_left_,
            bottom_right_: self.bottom_right_,
            index_: self.index_,
            geometry_rect_: self.geometry_rect_,
            current_tile_: self.current_tile_,
        }
    }
}
impl<'a, T: Tiling> TilingCoverageIterator<'a, T> {
    // Source: cc/tiles/tiling_coverage_iterator.h:39-73.
    pub fn new(tiling: &'a T, coverage_scale: f32, coverage_rect: &Rect) -> Self {
        let coverage_rect_max_bounds =
            Self::ComputeCoverageRectMaxBounds(tiling, &tiling.raster_size(), coverage_scale);
        let mut coverage_rect = *coverage_rect;
        coverage_rect.Intersect(coverage_rect_max_bounds);
        // Source dependency: ui/gfx/geometry/axis_transform2d.h:156-161.
        let mut coverage_to_content = tiling.raster_transform().clone();
        let scale = 1.0 / coverage_scale;
        coverage_to_content.PreScale(Vector2dF::new(scale, scale));
        let mut result = Self {
            tiling_: Some(tiling),
            coverage_rect_max_bounds_: coverage_rect_max_bounds,
            coverage_rect_: coverage_rect,
            coverage_to_content_: coverage_to_content,
            top_left_: TileIndex::default(),
            bottom_right_: TileIndex::default(),
            index_: TileIndex::default(),
            geometry_rect_: Rect::default(),
            current_tile_: None,
        };
        if result.coverage_rect_.IsEmpty() {
            return result;
        }
        let wanted_texels =
            Self::ComputeWantedTexels(&result.coverage_to_content_, &result.coverage_rect_);
        let data = tiling.tiling_data();
        result.top_left_.i = data.LastBorderTileXIndexFromSrcCoord(wanted_texels.x);
        result.top_left_.j = data.LastBorderTileYIndexFromSrcCoord(wanted_texels.y);
        result.bottom_right_.i = 1 + data
            .FirstBorderTileXIndexFromSrcCoord(wanted_texels.right())
            .max(result.top_left_.i);
        result.bottom_right_.j = 1 + data
            .FirstBorderTileYIndexFromSrcCoord(wanted_texels.bottom())
            .max(result.top_left_.j);
        result.index_ = result.top_left_;
        result.AdvanceUntilTileIsRelevant();
        result
    }
    // Source: cc/tiles/tiling_coverage_iterator.h:79-85.
    // IsValid also represents the explicit C++ bool conversion.
    pub fn IsValid(&self) -> bool {
        self.index_.j < self.bottom_right_.j
    }
    // Source: cc/tiles/tiling_coverage_iterator.h:87-95.
    // Rust's named prefix-increment operation returns the same iterator.
    pub fn Advance(&mut self) -> &mut Self {
        if self.IsValid() {
            self.IncrementIndex();
            self.AdvanceUntilTileIsRelevant();
        }
        self
    }
    // Source: cc/tiles/tiling_coverage_iterator.h:97-107.
    pub fn index(&self) -> &TileIndex {
        &self.index_
    }
    pub fn i(&self) -> i32 {
        self.index_.i
    }
    pub fn j(&self) -> i32 {
        self.index_.j
    }
    // Dereference/arrow expose the same nullable current resident tile.
    pub fn current_tile(&self) -> Option<&'a T::Tile> {
        self.current_tile_
    }
    pub fn geometry_rect(&self) -> &Rect {
        &self.geometry_rect_
    }
    // Source: cc/tiles/tiling_coverage_iterator.h:109-121.
    pub fn texture_rect(&self) -> RectF {
        let tex_origin = self
            .tiling_
            .expect("source initialized tiling")
            .tiling_data()
            .TileBoundsWithBorder(self.index_.i, self.index_.j)
            .origin();
        let mut texture_rect = self
            .coverage_to_content_
            .MapRect(RectF::from(self.geometry_rect_));
        texture_rect.Offset(-(tex_origin.x() as f32), -(tex_origin.y() as f32));
        texture_rect
    }
    // Source: cc/tiles/tiling_coverage_iterator.h:124-133.
    fn ComputeCoverageRectMaxBounds(tiling: &T, layer_bounds: &Size, coverage_scale: f32) -> Rect {
        let mut tiling_rect_in_layer_space = ToEnclosingRect(
            tiling
                .raster_transform()
                .InverseMapRect(RectF::from(*tiling.tiling_data().tiling_rect())),
        );
        tiling_rect_in_layer_space.Intersect(Rect::new(
            0,
            0,
            layer_bounds.width(),
            layer_bounds.height(),
        ));
        ScaleToEnclosingRect(tiling_rect_in_layer_space, coverage_scale)
    }
    // Source: cc/tiles/tiling_coverage_iterator.h:135-142.
    fn ComputeWantedTexels(coverage_to_content: &T::RasterTransform, coverage_rect: &Rect) -> Rect {
        let mut content_rect = coverage_to_content.MapRect(RectF::from(*coverage_rect));
        content_rect.Offset(-0.5, -0.5);
        ToEnclosingRect(content_rect)
    }
    // Source: cc/tiles/tiling_coverage_iterator.h:144-150.
    fn IncrementIndex(&mut self) {
        self.index_.i += 1;
        if self.index_.i >= self.bottom_right_.i {
            self.index_.i = self.top_left_.i;
            self.index_.j += 1;
        }
    }
    // Source: cc/tiles/tiling_coverage_iterator.h:152-201.
    fn AdvanceUntilTileIsRelevant(&mut self) {
        let tiling = self.tiling_.expect("source initialized tiling");
        let data = tiling.tiling_data();
        let mut last_geometry_rect = Rect::default();
        let mut next_tile = None;
        while self.IsValid() {
            let mut texel_extent = data.TexelExtent(self.index_.i, self.index_.j);
            // Bilinear-filter clamping overreach remains below a color rounding
            // threshold: 255*(1-(1-1/1024)^2) ~= 0.498 (source lines161-166).
            const kEpsilon: f32 = 1.0 / 1024.0;
            texel_extent.Inset(-kEpsilon, -kEpsilon, -kEpsilon, -kEpsilon);
            let mut geometry_rect =
                ToEnclosedRect(self.coverage_to_content_.InverseMapRect(texel_extent));
            geometry_rect = Rect::FromBounds(
                if self.index_.i == 0 {
                    self.coverage_rect_max_bounds_.x
                } else {
                    geometry_rect.x
                },
                if self.index_.j == 0 {
                    self.coverage_rect_max_bounds_.y
                } else {
                    geometry_rect.y
                },
                if self.index_.i == data.num_tiles_x() - 1 {
                    self.coverage_rect_max_bounds_.right()
                } else {
                    geometry_rect.right()
                },
                if self.index_.j == data.num_tiles_y() - 1 {
                    self.coverage_rect_max_bounds_.bottom()
                } else {
                    geometry_rect.bottom()
                },
            );
            geometry_rect.Intersect(self.coverage_rect_);
            if !geometry_rect.IsEmpty() {
                next_tile = tiling.TileAt(&self.index_);
                last_geometry_rect = std::mem::replace(&mut self.geometry_rect_, geometry_rect);
                break;
            }
            self.IncrementIndex();
        }
        // Source: cc/tiles/tiling_coverage_iterator.h:203-223.
        self.current_tile_ = next_tile;
        if last_geometry_rect.IsEmpty() {
            return;
        }
        let (min_left, min_top);
        let new_row = self.index_.i == self.top_left_.i;
        if new_row {
            min_left = self.coverage_rect_.x;
            min_top = last_geometry_rect.bottom();
        } else {
            min_left = last_geometry_rect.right();
            min_top = last_geometry_rect.y;
        }
        let inset_left = 0.max(min_left - self.geometry_rect_.x);
        let inset_top = 0.max(min_top - self.geometry_rect_.y);
        self.geometry_rect_.Inset(inset_top, inset_left, 0, 0);
        // Source: cc/tiles/tiling_coverage_iterator.h:225-239.
        #[cfg(debug_assertions)]
        {
            const kReasonableOffsetForDcheck: i32 = 100_000_000;
            if !new_row
                && self.geometry_rect_.x <= kReasonableOffsetForDcheck
                && self.geometry_rect_.y <= kReasonableOffsetForDcheck
            {
                debug_assert_eq!(last_geometry_rect.right(), self.geometry_rect_.x);
                debug_assert_eq!(last_geometry_rect.bottom(), self.geometry_rect_.bottom());
                debug_assert_eq!(last_geometry_rect.y, self.geometry_rect_.y);
            }
        }
    }
}
