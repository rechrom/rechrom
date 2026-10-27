// Translation status: complete
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tiling_set_coverage_iterator.h:1-241.
// Rust modules replace templates, concepts, includes, guards and namespaces.

use crate::{
    tile_priority::TileResolution,
    tiling_coverage_iterator::TilingCoverageIterator,
    tiling_internal::{TileLike, Tiling},
};
use compositor_tree::{
    geometry::{Rect, RectF},
    region::Region,
    region_backend::{SkIRect, SkRegionBackend},
};

// Source: cc/tiles/tiling_set_coverage_iterator.h:24-36.
pub trait TilingWithCover: Tiling {
    fn resolution(&self) -> TileResolution;
    fn Cover<'a>(&'a self, rect: &Rect, scale: f32) -> TilingCoverageIterator<'a, Self>
    where
        Self: Sized,
    {
        TilingCoverageIterator::new(self, scale, rect)
    }
}

// Source: cc/tiles/tiling_set_coverage_iterator.h:38-67,225-237.
// Indices replace native vector iterators. Regions and tilings stay resident;
// each region rectangle is read from Region rather than a copied region list.
pub struct TilingSetCoverageIterator<'a, T: TilingWithCover, B: SkRegionBackend> {
    tilings_: &'a [Box<T>],
    coverage_scale_: f32,
    ideal_tiling_: usize,
    current_tiling_: Option<usize>,
    missing_region_: Region<B>,
    current_region_: Region<B>,
    region_index_: usize,
    tiling_iter_: TilingCoverageIterator<'a, T>,
}

impl<'a, T: TilingWithCover, B: SkRegionBackend> TilingSetCoverageIterator<'a, T, B> {
    // Source: cc/tiles/tiling_set_coverage_iterator.h:43-65.
    pub fn new(
        tilings: &'a [Box<T>],
        coverage_rect: Rect,
        coverage_scale: f32,
        ideal_contents_scale: f32,
    ) -> Self {
        let mut result = Self {
            tilings_: tilings,
            coverage_scale_: coverage_scale,
            ideal_tiling_: Self::FindIdealTiling(tilings, ideal_contents_scale),
            current_tiling_: None,
            missing_region_: Region::new(coverage_rect),
            current_region_: Region::default(),
            region_index_: 0,
            tiling_iter_: TilingCoverageIterator::default(),
        };
        result.AdvanceUntilTileIsRelevant();
        result
    }

    // Source: cc/tiles/tiling_set_coverage_iterator.h:67-77.
    pub fn IsValid(&self) -> bool {
        self.current_tiling_
            .is_some_and(|index| index < self.tilings_.len())
            || self.current_region_rect().is_some()
    }

    // Source: cc/tiles/tiling_set_coverage_iterator.h:79-93.
    pub fn CurrentTiling(&self) -> Option<&'a T> {
        self.tilings_.get(self.current_tiling_?).map(Box::as_ref)
    }
    pub fn current_tile(&self) -> Option<&'a T::Tile> {
        self.tiling_iter_
            .IsValid()
            .then(|| self.tiling_iter_.current_tile())
            .flatten()
    }

    // Source: cc/tiles/tiling_set_coverage_iterator.h:95-123.
    pub fn geometry_rect(&self) -> Rect {
        if self.tiling_iter_.IsValid() {
            *self.tiling_iter_.geometry_rect()
        } else {
            self.current_region_rect().unwrap_or_default()
        }
    }
    pub fn texture_rect(&self) -> RectF {
        if self.tiling_iter_.IsValid() {
            self.tiling_iter_.texture_rect()
        } else {
            RectF::default()
        }
    }
    pub fn resolution(&self) -> TileResolution {
        self.CurrentTiling()
            .expect("valid current tiling")
            .resolution()
    }
    pub fn Advance(&mut self) -> &mut Self {
        assert!(self.IsValid());
        self.AdvanceUntilTileIsRelevant();
        self
    }

    // Source: cc/tiles/tiling_set_coverage_iterator.h:131-150.
    fn FindIdealTiling(tilings: &[Box<T>], ideal_contents_scale: f32) -> usize {
        if tilings.is_empty() {
            return 0;
        }
        for (index, tiling) in tilings.iter().enumerate() {
            if tiling.contents_scale_key() < ideal_contents_scale {
                return index.saturating_sub(1);
            }
        }
        tilings.len() - 1
    }

    // Source: cc/tiles/tiling_set_coverage_iterator.h:152-168.
    fn AdvanceTiling(&mut self) {
        debug_assert_ne!(self.current_tiling_, Some(self.tilings_.len()));
        self.current_tiling_ = Some(match self.current_tiling_ {
            None => self.ideal_tiling_,
            Some(current) if current > self.ideal_tiling_ => current + 1,
            Some(current) if current > 0 => current - 1,
            Some(_) => self.ideal_tiling_ + 1,
        });
    }

    // Source: cc/tiles/tiling_set_coverage_iterator.h:170-223.
    fn AdvanceUntilTileIsRelevant(&mut self) {
        if !self.IsValid() && self.current_tiling_.is_some() {
            return;
        }
        if self.tiling_iter_.IsValid() {
            self.tiling_iter_.Advance();
        }
        loop {
            let mut missing_batch = Vec::<SkIRect>::new();
            while self.tiling_iter_.IsValid() {
                let tile = self
                    .CurrentTiling()
                    .expect("coverage tiling")
                    .TileAt(self.tiling_iter_.index());
                if tile.is_some_and(TileLike::IsReadyToDraw) {
                    self.missing_region_.Union(missing_batch.as_slice());
                    return;
                }
                missing_batch.push(SkIRect::FromRect(*self.tiling_iter_.geometry_rect()));
                self.tiling_iter_.Advance();
            }
            self.missing_region_.Union(missing_batch.as_slice());

            if self.current_region_rect().is_none() {
                self.AdvanceTiling();
                self.current_region_.Swap(&mut self.missing_region_);
                self.missing_region_.Clear();
                self.region_index_ = 0;
                if self.current_region_rect().is_none() {
                    self.current_tiling_ = Some(self.tilings_.len());
                    return;
                }
                if self.current_tiling_ == Some(self.tilings_.len()) {
                    return;
                }
            }

            let last_rect = self.current_region_rect().expect("current region");
            self.region_index_ += 1;
            if self.current_tiling_ == Some(self.tilings_.len()) {
                return;
            }
            self.tiling_iter_ = self
                .CurrentTiling()
                .expect("coverage tiling")
                .Cover(&last_rect, self.coverage_scale_);
        }
    }

    fn current_region_rect(&self) -> Option<Rect> {
        (&self.current_region_).into_iter().nth(self.region_index_)
    }
}

// Source: cc/tiles/tiling_set_coverage_iterator.h:239-241.
// Namespace and include-guard closure map to the Rust module boundary.
