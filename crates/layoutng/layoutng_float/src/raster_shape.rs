#![allow(non_snake_case)]

use std::cell::OnceCell;

use foundation::{gfx, LayoutUnit};
use layoutng_assembly::internal::shapes::shape::{LineSegment, Shape, ShapeBase};
use layoutng_assembly::internal::shapes::shape_interval::IntShapeInterval;
use layoutng_geometry::geometry::logical_rect::LogicalRect;

// cpp: layoutng_float/raster_shape.h:40-74
pub struct RasterShapeIntervals {
    bounds_: gfx::Rect,
    intervals_: Vec<IntShapeInterval>,
    offset_: i32,
}

impl RasterShapeIntervals {
    // cpp: layoutng_float/raster_shape.h:44-45
    pub fn new(size: usize, offset: i32) -> Self {
        Self {
            bounds_: gfx::Rect::default(),
            intervals_: vec![IntShapeInterval::default(); size],
            offset_: offset,
        }
    }

    // cpp: layoutng_float/raster_shape.h:48-49
    pub fn Bounds(&self) -> &gfx::Rect {
        &self.bounds_
    }
    pub fn IsEmpty(&self) -> bool {
        self.bounds_.IsEmpty()
    }

    // cpp: layoutng_float/raster_shape.h:51-60
    pub fn IntervalAt(&self, y: i32) -> &IntShapeInterval {
        let index = y + self.offset_;
        debug_assert!(index >= 0);
        debug_assert!((index as usize) < self.intervals_.len());
        &self.intervals_[index as usize]
    }
    pub fn IntervalAtMut(&mut self, y: i32) -> &mut IntShapeInterval {
        let index = y + self.offset_;
        debug_assert!(index >= 0);
        debug_assert!((index as usize) < self.intervals_.len());
        &mut self.intervals_[index as usize]
    }

    // cpp: layoutng_float/raster_shape.h:66-69
    fn size(&self) -> i32 {
        self.intervals_.len() as i32
    }
    fn Offset(&self) -> i32 {
        self.offset_
    }
    fn MinY(&self) -> i32 {
        -self.offset_
    }
    fn MaxY(&self) -> i32 {
        -self.offset_ + self.size()
    }

    // cpp: layoutng_float/raster_shape.h:47-47
    // cpp: layoutng_float/raster_shape.cc:107-114
    pub fn InitializeBounds(&mut self) {
        self.bounds_ = gfx::Rect::default();
        for y in self.MinY()..self.MaxY() {
            let interval = *self.IntervalAt(y);
            if !interval.IsEmpty() {
                self.bounds_.Union(gfx::Rect::new(
                    gfx::Point::new(interval.X1(), y),
                    gfx::Size::new(interval.Width(), 1),
                ));
            }
        }
    }

    // cpp: layoutng_float/raster_shape.h:62-63
    // cpp: layoutng_float/raster_shape.cc:71-105
    pub fn ComputeShapeMarginIntervals(&self, shape_margin: i32) -> Box<Self> {
        let margin_intervals_size = if self.Offset() > shape_margin {
            self.size()
        } else {
            self.size() - self.Offset() * 2 + shape_margin * 2
        };
        let mut result = Box::new(Self::new(
            margin_intervals_size as usize,
            shape_margin.max(self.Offset()),
        ));
        let mut margin_interval_generator = MarginIntervalGenerator::new(shape_margin as usize);

        for y in self.Bounds().y()..self.Bounds().bottom() {
            let interval_at_y = self.IntervalAt(y);
            if interval_at_y.IsEmpty() {
                continue;
            }
            margin_interval_generator.Set(y, interval_at_y);
            let margin_y0 = self.MinY().max(y - shape_margin);
            let margin_y1 = self.MaxY().min(y + shape_margin + 1);
            for margin_y in (margin_y0..y).rev() {
                if margin_y > self.Bounds().y() && self.IntervalAt(margin_y).Contains(interval_at_y)
                {
                    break;
                }
                result
                    .IntervalAtMut(margin_y)
                    .Unite(&margin_interval_generator.IntervalAt(margin_y));
            }
            result
                .IntervalAtMut(y)
                .Unite(&margin_interval_generator.IntervalAt(y));
            for margin_y in y + 1..margin_y1 {
                if margin_y < self.Bounds().bottom()
                    && self.IntervalAt(margin_y).Contains(interval_at_y)
                {
                    break;
                }
                result
                    .IntervalAtMut(margin_y)
                    .Unite(&margin_interval_generator.IntervalAt(margin_y));
            }
        }
        result.InitializeBounds();
        result
    }
}

// cpp: layoutng_float/raster_shape.cc:36-47
struct MarginIntervalGenerator {
    x_intercepts_: Vec<i32>,
    y_: i32,
    x1_: i32,
    x2_: i32,
}

impl MarginIntervalGenerator {
    // cpp: layoutng_float/raster_shape.cc:49-55
    fn new(radius: usize) -> Self {
        let mut x_intercepts = vec![0; radius + 1];
        let radius_squared = (radius as u32).wrapping_mul(radius as u32);
        for y in 0..=radius {
            let y_squared = (y as u32).wrapping_mul(y as u32);
            x_intercepts[y] = (radius_squared.wrapping_sub(y_squared) as f64).sqrt() as i32;
        }
        Self {
            x_intercepts_: x_intercepts,
            y_: 0,
            x1_: 0,
            x2_: 0,
        }
    }

    // cpp: layoutng_float/raster_shape.cc:57-63
    fn Set(&mut self, y: i32, interval: &IntShapeInterval) {
        debug_assert!(y >= 0);
        debug_assert!(interval.X1() >= 0);
        self.y_ = y;
        self.x1_ = interval.X1();
        self.x2_ = interval.X2();
    }

    // cpp: layoutng_float/raster_shape.cc:65-69
    fn IntervalAt(&self, y: i32) -> IntShapeInterval {
        let index = y.abs_diff(self.y_) as usize;
        let dx = if index >= self.x_intercepts_.len() {
            0
        } else {
            self.x_intercepts_[index]
        };
        IntShapeInterval::new(self.x1_ - dx, self.x2_ + dx)
    }
}

// cpp: layoutng_float/raster_shape.h:76-100
pub struct RasterShape {
    base: ShapeBase,
    intervals_: Box<RasterShapeIntervals>,
    margin_intervals_: OnceCell<Box<RasterShapeIntervals>>,
    margin_rect_size_: gfx::Size,
}

impl RasterShape {
    // cpp: layoutng_float/raster_shape.h:81-85
    pub fn new(mut intervals: Box<RasterShapeIntervals>, margin_rect_size: &gfx::Size) -> Self {
        intervals.InitializeBounds();
        Self {
            base: ShapeBase::default(),
            intervals_: intervals,
            margin_intervals_: OnceCell::new(),
            margin_rect_size_: *margin_rect_size,
        }
    }

    // cpp: layoutng_float/raster_shape.h:95-95
    // cpp: layoutng_float/raster_shape.cc:116-127
    fn MarginIntervals(&self) -> &RasterShapeIntervals {
        debug_assert!(self.ShapeMargin() >= 0.0);
        if self.ShapeMargin() == 0.0 {
            return &self.intervals_;
        }
        let shape_margin = self.ShapeMargin().ceil() as i32;
        let maximum = (self
            .margin_rect_size_
            .width()
            .max(self.margin_rect_size_.height()) as f32
            * 2.0_f32.sqrt()) as i32;
        self.margin_intervals_
            .get_or_init(|| {
                self.intervals_
                    .ComputeShapeMarginIntervals(shape_margin.min(maximum))
            })
            .as_ref()
    }
}

// cpp: layoutng_float/raster_shape.h:79-92
impl Shape for RasterShape {
    fn base(&self) -> &ShapeBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ShapeBase {
        &mut self.base
    }

    // cpp: layoutng_float/raster_shape.h:87-89
    fn ShapeMarginLogicalBoundingBox(&self) -> LogicalRect {
        LogicalRect::from_gfx_rect(self.MarginIntervals().Bounds())
    }

    // cpp: layoutng_float/raster_shape.h:90-90
    fn IsEmpty(&self) -> bool {
        self.intervals_.IsEmpty()
    }

    // cpp: layoutng_float/raster_shape.h:91-92
    // cpp: layoutng_float/raster_shape.cc:129-148
    fn GetExcludedInterval(
        &self,
        logical_top: LayoutUnit,
        logical_height: LayoutUnit,
    ) -> LineSegment {
        let intervals = self.MarginIntervals();
        if intervals.IsEmpty() {
            return LineSegment::default();
        }
        let mut y1 = logical_top.ToInt();
        let mut y2 = (logical_top + logical_height).ToInt();
        debug_assert!(y2 >= y1);
        if y2 < intervals.Bounds().y() || y1 >= intervals.Bounds().bottom() {
            return LineSegment::default();
        }
        y1 = y1.max(intervals.Bounds().y());
        y2 = y2.min(intervals.Bounds().bottom());
        let mut excluded = IntShapeInterval::default();
        if y1 == y2 {
            excluded = *intervals.IntervalAt(y1);
        } else {
            for y in y1..y2 {
                excluded.Unite(intervals.IntervalAt(y));
            }
        }
        LineSegment::new(excluded.X1() as f32, excluded.X2() as f32)
    }
}
