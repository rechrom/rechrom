// Migrated unchanged in behavior from tiny-skia-0.12.0/src/path64/point64.rs.

// Copyright 2012 Google Inc.
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::raster::Point;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SearchAxis {
    X,
    Y,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct SkDPoint {
    pub x: f64,
    pub y: f64,
}

impl SkDPoint {
    pub fn from_xy(x: f64, y: f64) -> Self {
        SkDPoint { x, y }
    }

    pub fn from_point(p: Point) -> Self {
        SkDPoint {
            x: f64::from(p.x),
            y: f64::from(p.y),
        }
    }

    pub fn zero() -> Self {
        SkDPoint { x: 0.0, y: 0.0 }
    }

    pub fn to_point(&self) -> Point {
        Point::from_xy(self.x as f32, self.y as f32)
    }

    pub fn axis_coord(&self, axis: SearchAxis) -> f64 {
        match axis {
            SearchAxis::X => self.x,
            SearchAxis::Y => self.y,
        }
    }
}

/// Legacy Rust API alias; canonical implementation name follows Skia.
pub use SkDPoint as Point64;
