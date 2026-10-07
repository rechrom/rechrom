// Migrated unchanged in behavior from tiny-skia-0.12.0/src/scan/mod.rs.

// Copyright 2011 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub use crate::src::core::SkScan_AntiPath as path_aa;
pub use crate::src::core::SkScan_Antihair as hairline_aa;
pub use crate::src::core::SkScan_Hairline as hairline;
pub use crate::src::core::SkScan_Path as path;

use crate::raster::{IntRect, Rect};

use crate::raster::blitter::Blitter;
use crate::raster::geom::{IntRectExt, ScreenIntRect};

pub fn fill_rect(rect: &Rect, clip: &ScreenIntRect, blitter: &mut dyn Blitter) {
    if let Some(rect) = rect.round() {
        fill_int_rect(&rect, clip, blitter);
    }
}

fn fill_int_rect(rect: &IntRect, clip: &ScreenIntRect, blitter: &mut dyn Blitter) {
    let rect = match rect.intersect(&clip.to_int_rect()) {
        Some(v) => v,
        None => return, // everything was clipped out
    };

    let rect = match rect.to_screen_int_rect() {
        Some(v) => v,
        None => return,
    };

    blitter.blit_rect(&rect);
}

pub fn fill_rect_aa(rect: &Rect, clip: &ScreenIntRect, blitter: &mut dyn Blitter) {
    hairline_aa::fill_rect(rect, clip, blitter);
}

// CPU implementation moved from renderer/rect_stroke.rs.

// SkDraw rectangular miter strokes: SkScan::AntiFrameRect/FrameRect.
// Copyright 2011 The Android Open Source Project; BSD-3-Clause.
// See glyphs/SKIA_LICENSE. Preserve fractional bias and ordered blits.
use crate::compat::commands::PaintRect;
use crate::raster::{Point, Transform};

pub(crate) struct RectCoverageBlitter {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) blits: Vec<(usize, u8)>,
}
impl RectCoverageBlitter {
    pub(crate) fn h(&mut self, x: i32, y: i32, count: i32, alpha: i32) {
        if y < 0 || y >= self.height || count <= 0 || alpha <= 0 {
            return;
        }
        assert!(alpha <= 255);
        for x in x.max(0)..(x + count).min(self.width) {
            self.blits
                .push(((y * self.width + x) as usize, alpha as u8));
        }
    }
    pub(crate) fn v(&mut self, x: i32, y: i32, count: i32, alpha: i32) {
        if x < 0 || x >= self.width || count <= 0 || alpha <= 0 {
            return;
        }
        assert!(alpha <= 255);
        for y in y.max(0)..(y + count).min(self.height) {
            self.blits
                .push(((y * self.width + x) as usize, alpha as u8));
        }
    }
    pub(crate) fn rect(&mut self, l: i32, t: i32, r: i32, b: i32) {
        for y in t.max(0)..b.min(self.height) {
            self.h(l, y, r - l, 255);
        }
    }
    pub(crate) fn fill_rect(&mut self, r: [f32; 4]) {
        let [l, t, r, b] = r.map(|x| (x + 0.5).floor() as i32);
        self.rect(l, t, r, b);
    }

    // SkScan_Antihair.cpp::antifilldot8(..., fillInner=false).

    // SkScan_Antihair.cpp::AntiFrameRect.

    // SkScan_Hairline.cpp::FrameRect.
}

// SkDraw::ComputeRectType/easy_rect_join for a solid miter rect stroke.
pub(crate) fn coverage(
    bounds: PaintRect,
    transform: Transform,
    stroke_width: f32,
    antialias: bool,
    width: u32,
    height: u32,
) -> Option<Vec<(usize, u8)>> {
    if stroke_width <= 0.0
        || !stroke_width.is_finite()
        || bounds.width <= 0.0
        || bounds.height <= 0.0
    {
        return None;
    }
    if ![bounds.x, bounds.y, bounds.width, bounds.height]
        .into_iter()
        .all(f64::is_finite)
    {
        return None;
    }
    if !(transform.kx == 0.0 && transform.ky == 0.0 || transform.sx == 0.0 && transform.sy == 0.0) {
        return None;
    }
    let mut points = [
        Point::from_xy(bounds.x as f32, bounds.y as f32),
        Point::from_xy(
            bounds.x as f32 + bounds.width as f32,
            bounds.y as f32 + bounds.height as f32,
        ),
    ];
    transform.map_points(&mut points);
    if !points
        .into_iter()
        .all(|p| p.x.is_finite() && p.y.is_finite())
    {
        return None;
    }
    let r = [
        points[0].x.min(points[1].x),
        points[0].y.min(points[1].y),
        points[0].x.max(points[1].x),
        points[0].y.max(points[1].y),
    ];
    let stroke = [
        (transform.sx * stroke_width + transform.kx * stroke_width).abs(),
        (transform.ky * stroke_width + transform.sy * stroke_width).abs(),
    ];
    if !r.into_iter().chain(stroke).all(f32::is_finite)
        || r[0] - stroke[0] / 2.0 < -32767.0
        || r[1] - stroke[1] / 2.0 < -32767.0
        || r[2] + stroke[0] / 2.0 > 32767.0
        || r[3] + stroke[1] / 2.0 > 32767.0
    {
        return None;
    }
    let mut blitter = RectCoverageBlitter {
        width: width as i32,
        height: height as i32,
        blits: Vec::new(),
    };
    if antialias {
        crate::src::core::SkScan_Antihair::AntiFrameRect(&mut blitter, r, stroke);
    } else {
        crate::src::core::SkScan_Hairline::FrameRect(&mut blitter, r, stroke);
    }
    Some(blitter.blits)
}
