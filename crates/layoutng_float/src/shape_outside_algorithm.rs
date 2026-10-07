#![allow(non_snake_case)]

use foundation::{
    gfx, FloatValueForLength, IsHorizontalWritingMode, LayoutUnit, PhysicalOffset, PhysicalRect,
    PhysicalSize, SnapSizeToPixel, TextDirection, WritingDirectionMode, WritingMode,
};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space::ConstraintSpace;
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_input::{
    PaintImage, ShapeCoordinate, ShapeOutsideGeometry, ShapeOutsideKind, ShapeRadius,
    ShapeRadiusKind,
};
use layoutng_assembly::internal::length_utils::ComputePhysicalMarginsForSpace;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};
use layoutng_assembly::internal::shapes::shape::{LineSegment, Shape, ShapeBase};
use layoutng_assembly::internal::shapes::shape_image_services::CurrentShapeImage;
use layoutng_assembly::layout_result::LayoutResult;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize, ToPhysicalSize};
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

use crate::ellipse_shape::EllipseShape;
use crate::inset_shape::InsetShape;
use crate::raster_shape::{RasterShape, RasterShapeIntervals};

// cpp: layoutng_float/shape_outside_algorithm.cc:41-48
fn PixelSnappedLogicalRect(rect: &LogicalRect) -> gfx::Rect {
    gfx::Rect::new(
        gfx::Point::new(
            rect.offset.inline_offset.Round(),
            rect.offset.block_offset.Round(),
        ),
        gfx::Size::new(
            SnapSizeToPixel(rect.size.inline_size, rect.offset.inline_offset),
            SnapSizeToPixel(rect.size.block_size, rect.offset.block_offset),
        ),
    )
}

// cpp: layoutng_float/shape_outside_algorithm.cc:50-106
struct LogicalAlphaScanner<'a> {
    image_: &'a PaintImage,
    target_size_: gfx::Size,
    writing_mode_: WritingMode,
    inline_offset_: u32,
    block_offset_: u32,
}

impl<'a> LogicalAlphaScanner<'a> {
    // cpp: layoutng_float/shape_outside_algorithm.cc:52-55
    fn new(image: &'a PaintImage, target_size: &gfx::Size, writing_mode: WritingMode) -> Self {
        Self {
            image_: image,
            target_size_: *target_size,
            writing_mode_: writing_mode,
            inline_offset_: 0,
            block_offset_: 0,
        }
    }

    // cpp: layoutng_float/shape_outside_algorithm.cc:57-61
    fn Next(&mut self) {
        self.inline_offset_ += 1;
    }
    fn NextLine(&mut self) {
        self.block_offset_ += 1;
        self.inline_offset_ = 0;
    }

    // cpp: layoutng_float/shape_outside_algorithm.cc:63-98
    fn GetAlpha(&self) -> u8 {
        let (x, y) = match self.writing_mode_ {
            WritingMode::kHorizontalTb => (self.inline_offset_, self.block_offset_),
            WritingMode::kVerticalRl | WritingMode::kSidewaysRl => (
                self.target_size_.width() as u32 - self.block_offset_ - 1,
                self.inline_offset_,
            ),
            WritingMode::kVerticalLr => (self.block_offset_, self.inline_offset_),
            WritingMode::kSidewaysLr => (
                self.block_offset_,
                self.target_size_.height() as u32 - self.inline_offset_ - 1,
            ),
        };
        let source_x = (self.image_.width - 1)
            .min(((x as u64 * self.image_.width as u64) / self.target_size_.width() as u64) as u32);
        let source_y = (self.image_.height - 1).min(
            ((y as u64 * self.image_.height as u64) / self.target_size_.height() as u64) as u32,
        );
        self.image_.BitmapPixels().map_or(255, |pixels| {
            pixels[((source_y as usize * self.image_.width as usize + source_x as usize) * 4) + 3]
        })
    }
}

// cpp: layoutng_float/shape_outside_algorithm.cc:108-154
fn ExtractImageIntervals(
    image: &PaintImage,
    threshold: f32,
    content_block_size: i32,
    image_physical_size: &gfx::Size,
    image_logical_rect: &gfx::Rect,
    margin_logical_rect: &gfx::Rect,
    writing_mode: WritingMode,
) -> Box<RasterShapeIntervals> {
    let alpha_threshold = (threshold * 255.0) as u8;
    let image_block_start = image_logical_rect.y();
    let image_block_end = image_logical_rect.bottom();
    let margin_box_block_size = margin_logical_rect.height();
    let margin_block_start = margin_logical_rect.y();
    let margin_block_end = margin_logical_rect.bottom();
    let min_buffer_y = 0.max(margin_block_start).max(image_block_start);
    let max_buffer_y = content_block_size
        .min(image_block_end)
        .min(margin_block_end);
    let mut intervals = Box::new(RasterShapeIntervals::new(
        margin_box_block_size.max(0) as usize,
        -margin_block_start,
    ));
    if image_physical_size.IsEmpty() || min_buffer_y >= max_buffer_y {
        return intervals;
    }

    let mut scanner = LogicalAlphaScanner::new(image, image_physical_size, writing_mode);
    for _ in image_block_start..min_buffer_y {
        scanner.NextLine();
    }
    for y in min_buffer_y..max_buffer_y {
        let mut start_x = i32::MAX;
        let mut end_x = i32::MIN;
        let mut inside = false;
        for x in image_logical_rect.x()..image_logical_rect.right() {
            let above_threshold = scanner.GetAlpha() > alpha_threshold;
            if above_threshold != inside {
                if !inside {
                    start_x = x.min(start_x);
                } else {
                    end_x = x;
                }
            }
            inside = above_threshold;
            scanner.Next();
        }
        if inside {
            end_x = image_logical_rect.right();
        }
        if start_x < end_x {
            *intervals.IntervalAtMut(y) =
                layoutng_assembly::internal::shapes::shape_interval::IntShapeInterval::new(
                    start_x, end_x,
                );
        }
        scanner.NextLine();
    }
    intervals
}

// cpp: layoutng_float/shape_outside_algorithm.cc:156-215
fn CreateImageShape(
    box_: &LayoutBox,
    geometry: &ShapeOutsideGeometry,
    reference_logical_size: LogicalSize,
    writing_mode: WritingMode,
    margin: f32,
    threshold: f32,
) -> Box<dyn Shape> {
    let image = unsafe { &*CurrentShapeImage(geometry.resource_id) };
    let zoom = box_.StyleRef().EffectiveZoom() as f64;
    let physical_width = image.width as f64 / image.resolution_scale * zoom;
    let physical_height = image.height as f64 / image.resolution_scale * zoom;
    const K_MAXIMUM_RASTER_SHAPE_BYTES: f64 = 512.0 * 1024.0 * 1024.0;
    if !physical_width.is_finite()
        || !physical_height.is_finite()
        || physical_width <= 0.0
        || physical_height <= 0.0
        || physical_width > i32::MAX as f64
        || physical_height > i32::MAX as f64
        || physical_width * physical_height * 4.0 >= K_MAXIMUM_RASTER_SHAPE_BYTES
    {
        let mut empty = RasterShape::new(
            Box::new(RasterShapeIntervals::new(0, 0)),
            &gfx::Size::default(),
        );
        empty.SetShapeMargin(margin);
        return Box::new(empty);
    }
    let target_width = 1.max(physical_width.round() as i32);
    let target_height = 1.max(physical_height.round() as i32);
    let image_physical_size = gfx::Size::new(target_width, target_height);
    let reference_physical_size = ToPhysicalSize(reference_logical_size, writing_mode);
    let converter = WritingModeConverter::new(
        WritingDirectionMode::new(writing_mode, TextDirection::kLtr),
        reference_physical_size,
    );
    let image_rect = converter.ToLogicalRect(PhysicalRect::new(
        PhysicalOffset::default(),
        PhysicalSize::new(
            LayoutUnit::from_signed(target_width),
            LayoutUnit::from_signed(target_height),
        ),
    ));
    let margin_border_padding = box_.MarginOutsets() + box_.BorderOutsets() + box_.PaddingOutsets();
    let physical_margin_rect = PhysicalRect::from_units(
        -margin_border_padding.left,
        -margin_border_padding.top,
        margin_border_padding.HorizontalSum() + reference_physical_size.width,
        margin_border_padding.VerticalSum() + reference_physical_size.height,
    );
    let mut margin_rect = converter.ToLogicalRect(physical_margin_rect);
    margin_rect.size.inline_size = margin_rect.size.inline_size.ClampNegativeToZero();
    margin_rect.size.block_size = margin_rect.size.block_size.ClampNegativeToZero();
    let snapped_image = PixelSnappedLogicalRect(&image_rect);
    let snapped_margin = PixelSnappedLogicalRect(&margin_rect);
    let intervals = ExtractImageIntervals(
        image,
        threshold,
        reference_logical_size.block_size.Floor(),
        &image_physical_size,
        &snapped_image,
        &snapped_margin,
        writing_mode,
    );
    let mut shape = RasterShape::new(intervals, &snapped_margin.size());
    shape.SetShapeMargin(margin);
    Box::new(shape)
}

// cpp: layoutng_float/shape_outside_algorithm.cc:217-226
struct FloatInterval {
    left: f32,
    right: f32,
}

impl Default for FloatInterval {
    fn default() -> Self {
        Self {
            left: f32::INFINITY,
            right: f32::NEG_INFINITY,
        }
    }
}

impl FloatInterval {
    fn IsEmpty(&self) -> bool {
        self.right <= self.left
    }
    fn Include(&mut self, first: f32, second: f32) {
        self.left = self.left.min(first.min(second));
        self.right = self.right.max(first.max(second));
    }
}

// cpp: layoutng_float/shape_outside_algorithm.cc:228-232
fn XAtY(first: &gfx::PointF, second: &gfx::PointF, y: f32) -> f32 {
    if first.y() == second.y() {
        return first.x().min(second.x());
    }
    first.x() + (y - first.y()) * (second.x() - first.x()) / (second.y() - first.y())
}

// cpp: layoutng_float/shape_outside_algorithm.cc:234-246
fn IncludeClippedEdge(
    first: &gfx::PointF,
    second: &gfx::PointF,
    top: f32,
    bottom: f32,
    interval: &mut FloatInterval,
) {
    let min_y = first.y().min(second.y());
    let max_y = first.y().max(second.y());
    if min_y == max_y || bottom <= min_y || top >= max_y {
        return;
    }
    let clipped_top = top.max(min_y);
    let clipped_bottom = bottom.min(max_y);
    interval.Include(
        XAtY(first, second, clipped_top),
        XAtY(first, second, clipped_bottom),
    );
}

// cpp: layoutng_float/shape_outside_algorithm.cc:248-264
fn IncludeClippedCircle(
    center: &gfx::PointF,
    radius: f32,
    top: f32,
    bottom: f32,
    interval: &mut FloatInterval,
) {
    if radius <= 0.0 || top >= center.y() + radius || bottom <= center.y() - radius {
        return;
    }
    let mut extent = radius;
    if center.y() < top || center.y() > bottom {
        let y = if center.y() < top {
            top - center.y()
        } else {
            bottom - center.y()
        };
        extent = radius * (1.0_f32 - y * y / (radius * radius)).max(0.0).sqrt();
    }
    interval.Include(center.x() - extent, center.x() + extent);
}

// cpp: layoutng_float/shape_outside_algorithm.cc:266-335
struct ContourShape {
    base: ShapeBase,
    points_: Vec<gfx::PointF>,
    min_x_: f32,
    max_x_: f32,
    min_y_: f32,
    max_y_: f32,
}

impl ContourShape {
    // cpp: layoutng_float/shape_outside_algorithm.cc:268-280
    fn new(points: Vec<gfx::PointF>, margin: f32) -> Self {
        let mut shape = Self {
            base: ShapeBase::default(),
            points_: points,
            min_x_: 0.0,
            max_x_: 0.0,
            min_y_: 0.0,
            max_y_: 0.0,
        };
        shape.SetShapeMargin(margin);
        if shape.points_.is_empty() {
            return shape;
        }
        shape.min_x_ = shape.points_[0].x();
        shape.max_x_ = shape.points_[0].x();
        shape.min_y_ = shape.points_[0].y();
        shape.max_y_ = shape.points_[0].y();
        for point in &shape.points_ {
            shape.min_x_ = shape.min_x_.min(point.x());
            shape.max_x_ = shape.max_x_.max(point.x());
            shape.min_y_ = shape.min_y_.min(point.y());
            shape.max_y_ = shape.max_y_.max(point.y());
        }
        shape
    }
}

// cpp: layoutng_float/shape_outside_algorithm.cc:266-335
impl Shape for ContourShape {
    fn base(&self) -> &ShapeBase {
        &self.base
    }
    fn base_mut(&mut self) -> &mut ShapeBase {
        &mut self.base
    }

    // cpp: layoutng_float/shape_outside_algorithm.cc:282-288
    fn ShapeMarginLogicalBoundingBox(&self) -> LogicalRect {
        if self.IsEmpty() {
            return LogicalRect::default();
        }
        let margin = self.ShapeMargin();
        LogicalRect::from_units(
            LayoutUnit::from_f32(self.min_x_ - margin),
            LayoutUnit::from_f32(self.min_y_ - margin),
            LayoutUnit::from_f32(self.max_x_ - self.min_x_ + 2.0 * margin),
            LayoutUnit::from_f32(self.max_y_ - self.min_y_ + 2.0 * margin),
        )
    }

    // cpp: layoutng_float/shape_outside_algorithm.cc:290-292
    fn IsEmpty(&self) -> bool {
        self.points_.len() < 3 || self.max_x_ <= self.min_x_ || self.max_y_ <= self.min_y_
    }

    // cpp: layoutng_float/shape_outside_algorithm.cc:294-327
    fn GetExcludedInterval(
        &self,
        logical_top: LayoutUnit,
        logical_height: LayoutUnit,
    ) -> LineSegment {
        if self.IsEmpty() {
            return LineSegment::default();
        }
        let top = logical_top.ToFloat();
        let bottom = (logical_top + logical_height).ToFloat();
        if bottom <= self.min_y_ - self.ShapeMargin() || top >= self.max_y_ + self.ShapeMargin() {
            return LineSegment::default();
        }

        let mut interval = FloatInterval::default();
        for index in 0..self.points_.len() {
            let first = self.points_[index];
            let second = self.points_[(index + 1) % self.points_.len()];
            let delta = second - first;
            let length = delta.Length();
            if length == 0.0 {
                continue;
            }
            if self.ShapeMargin() == 0.0 {
                IncludeClippedEdge(&first, &second, top, bottom, &mut interval);
                continue;
            }
            let normal = gfx::Vector2dF::new(-delta.y() / length, delta.x() / length);
            let mut offset = normal;
            offset.ScaleUniform(self.ShapeMargin());
            IncludeClippedEdge(
                &(first + offset),
                &(second + offset),
                top,
                bottom,
                &mut interval,
            );
            IncludeClippedEdge(
                &(first - offset),
                &(second - offset),
                top,
                bottom,
                &mut interval,
            );
            IncludeClippedCircle(&first, self.ShapeMargin(), top, bottom, &mut interval);
            IncludeClippedCircle(&second, self.ShapeMargin(), top, bottom, &mut interval);
        }
        if interval.IsEmpty() {
            LineSegment::default()
        } else {
            LineSegment::new(interval.left, interval.right)
        }
    }
}

// cpp: layoutng_float/shape_outside_algorithm.cc:337-339
fn Resolve(value: &ShapeCoordinate, basis: f32) -> f32 {
    (value.pixels + value.percentage * basis as f64 / 100.0) as f32
}

// cpp: layoutng_float/shape_outside_algorithm.cc:341-350
fn RadiusToCorner(center: &gfx::PointF, width: f32, height: f32, closest: bool) -> f32 {
    let dx = if closest {
        center.x().min(width - center.x())
    } else {
        center.x().max(width - center.x())
    };
    let dy = if closest {
        center.y().min(height - center.y())
    } else {
        center.y().max(height - center.y())
    };
    dx.hypot(dy)
}

// cpp: layoutng_float/shape_outside_algorithm.cc:352-370
fn ResolveEllipseRadius(
    radius: &ShapeRadius,
    physical_center: &gfx::PointF,
    center: f32,
    size: f32,
    width: f32,
    height: f32,
) -> f32 {
    match radius.kind {
        ShapeRadiusKind::kLengthPercentage => Resolve(&radius.value, size).max(0.0),
        ShapeRadiusKind::kClosestSide => center.abs().min((size - center).abs()),
        ShapeRadiusKind::kFarthestSide => center.max((size - center).abs()),
        ShapeRadiusKind::kClosestCorner => RadiusToCorner(physical_center, width, height, true),
        ShapeRadiusKind::kFarthestCorner => RadiusToCorner(physical_center, width, height, false),
    }
}

// cpp: layoutng_float/shape_outside_algorithm.cc:372-393
fn ResolveCircleRadius(radius: &ShapeRadius, center: &gfx::PointF, width: f32, height: f32) -> f32 {
    if radius.kind == ShapeRadiusKind::kLengthPercentage {
        let normalized_diagonal = ((width * width + height * height) / 2.0).sqrt();
        return Resolve(&radius.value, normalized_diagonal).max(0.0);
    }
    if radius.kind == ShapeRadiusKind::kClosestCorner {
        return RadiusToCorner(center, width, height, true);
    }
    if radius.kind == ShapeRadiusKind::kFarthestCorner {
        return RadiusToCorner(center, width, height, false);
    }
    let width_delta = (width - center.x()).abs();
    let height_delta = (height - center.y()).abs();
    if radius.kind == ShapeRadiusKind::kClosestSide {
        return center
            .x()
            .abs()
            .min(width_delta)
            .min(center.y().abs().min(height_delta));
    }
    center
        .x()
        .max(width_delta)
        .max(center.y().max(height_delta))
}

// cpp: layoutng_float/shape_outside_algorithm.cc:395-403
fn LogicalRadii(mut physical: [gfx::SizeF; 4], writing_mode: WritingMode) -> [gfx::SizeF; 4] {
    if writing_mode == WritingMode::kHorizontalTb {
        return physical;
    }
    for radius in &mut physical {
        radius.Transpose();
    }
    if writing_mode == WritingMode::kVerticalLr {
        return [physical[0], physical[3], physical[2], physical[1]];
    }
    [physical[1], physical[2], physical[3], physical[0]]
}

// cpp: layoutng_float/shape_outside_algorithm.cc:405-420
fn InputBorderRadii(box_: &LayoutBox) -> [gfx::SizeF; 4] {
    let node = box_.GetNode();
    assert!(!node.is_null());
    let paint = &unsafe { &*node }.InputStyle().paint;
    let fallback = PaintCornerRadius {
        x: paint.border_radius,
        y: paint.border_radius,
    };
    let radii = paint.border_radii.unwrap_or(PaintCornerRadii {
        top_left: fallback,
        top_right: fallback,
        bottom_right: fallback,
        bottom_left: fallback,
    });
    let size = |radius: PaintCornerRadius| gfx::SizeF::new(radius.x as f32, radius.y as f32);
    [
        size(radii.top_left),
        size(radii.top_right),
        size(radii.bottom_right),
        size(radii.bottom_left),
    ]
}

// cpp: layoutng_float/shape_outside_algorithm.cc:422-429
fn InputGeometry(box_: &LayoutBox) -> &ShapeOutsideGeometry {
    let node = box_.GetNode();
    assert!(!node.is_null());
    let input = unsafe { &*node }.InputStyle();
    let extended = input
        .extended
        .as_ref()
        .expect("shape outside extended style");
    extended
        .shape_outside
        .as_ref()
        .expect("shape outside geometry")
}

// cpp: layoutng_float/shape_outside_algorithm.h:12-14
// cpp: layoutng_float/shape_outside_algorithm.cc:433-520
pub fn CreateShapeOutside(
    box_: &LayoutBox,
    reference_size: LogicalSize,
    percentage_resolution: LayoutUnit,
) -> Option<Box<dyn Shape>> {
    let geometry = InputGeometry(box_);
    let writing_mode = unsafe { &*box_.ContainingBlock() }
        .StyleRef()
        .GetWritingMode();
    let physical_size = ToPhysicalSize(reference_size, writing_mode);
    let width = physical_size.width.ToFloat();
    let height = physical_size.height.ToFloat();
    let converter = WritingModeConverter::new(
        WritingDirectionMode::new(writing_mode, TextDirection::kLtr),
        physical_size,
    );
    let margin = FloatValueForLength(
        box_.StyleRef().ShapeMargin(),
        percentage_resolution.ToFloat(),
    );

    // C++ unique_ptr<Shape> is null until a case allocates the source-owned
    // concrete shape. The Rust option retains the callback's nullable boundary.
    let shape: Box<dyn Shape> = match geometry.kind {
        // cpp: layoutng_float/shape_outside_algorithm.cc:451-461
        ShapeOutsideKind::kPolygon => {
            let mut points = Vec::with_capacity(geometry.contour.len());
            for point in &geometry.contour {
                points.push(gfx::PointF::new(
                    Resolve(&point.x, width),
                    Resolve(&point.y, height),
                ));
            }
            for point in &mut points {
                *point = converter.ToLogicalPointF(*point);
            }
            Box::new(ContourShape::new(points, margin))
        }
        // cpp: layoutng_float/shape_outside_algorithm.cc:462-468
        ShapeOutsideKind::kReferenceBox => {
            let physical_bounds =
                gfx::RectF::new(gfx::PointF::default(), gfx::SizeF::new(width, height));
            let mut shape = InsetShape::new(
                converter.ToLogicalRectF(physical_bounds),
                LogicalRadii(InputBorderRadii(box_), writing_mode),
            );
            shape.SetShapeMargin(margin);
            Box::new(shape)
        }
        // cpp: layoutng_float/shape_outside_algorithm.cc:469-492
        ShapeOutsideKind::kCircle | ShapeOutsideKind::kEllipse => {
            let physical_center = gfx::PointF::new(
                Resolve(&geometry.center.x, width),
                Resolve(&geometry.center.y, height),
            );
            let (mut radius_x, mut radius_y);
            if geometry.kind == ShapeOutsideKind::kCircle {
                radius_x = ResolveCircleRadius(&geometry.radius_x, &physical_center, width, height);
                radius_y = radius_x;
            } else {
                radius_x = ResolveEllipseRadius(
                    &geometry.radius_x,
                    &physical_center,
                    physical_center.x(),
                    width,
                    width,
                    height,
                );
                radius_y = ResolveEllipseRadius(
                    &geometry.radius_y,
                    &physical_center,
                    physical_center.y(),
                    height,
                    width,
                    height,
                );
            }
            if !IsHorizontalWritingMode(writing_mode) {
                std::mem::swap(&mut radius_x, &mut radius_y);
            }
            let mut shape = EllipseShape::new(
                &converter.ToLogicalPointF(physical_center),
                radius_x,
                radius_y,
            );
            shape.SetShapeMargin(margin);
            Box::new(shape)
        }
        // cpp: layoutng_float/shape_outside_algorithm.cc:493-512
        ShapeOutsideKind::kInset => {
            let top = Resolve(&geometry.insets[0], height);
            let right = Resolve(&geometry.insets[1], width);
            let bottom = Resolve(&geometry.insets[2], height);
            let left = Resolve(&geometry.insets[3], width);
            let physical_bounds = gfx::RectF::new(
                gfx::PointF::new(left, top),
                gfx::SizeF::new(
                    (width - left - right).max(0.0),
                    (height - top - bottom).max(0.0),
                ),
            );
            let mut physical_radii = [gfx::SizeF::default(); 4];
            for index in 0..physical_radii.len() {
                physical_radii[index] = gfx::SizeF::new(
                    Resolve(&geometry.corner_radii[index].x, width).max(0.0),
                    Resolve(&geometry.corner_radii[index].y, height).max(0.0),
                );
            }
            let mut shape = InsetShape::new(
                converter.ToLogicalRectF(physical_bounds),
                LogicalRadii(physical_radii, writing_mode),
            );
            shape.SetShapeMargin(margin);
            Box::new(shape)
        }
        // cpp: layoutng_float/shape_outside_algorithm.cc:513-517
        ShapeOutsideKind::kImage => CreateImageShape(
            box_,
            geometry,
            reference_size,
            writing_mode,
            margin,
            box_.StyleRef().ShapeImageThreshold(),
        ),
    };
    Some(shape)
}

// cpp: layoutng_float/shape_outside_algorithm.h:15-18
// cpp: layoutng_float/shape_outside_algorithm.cc:522-541
pub fn UpdateShapeOutsideInfo(
    node: &BlockNode,
    layout_result: &LayoutResult,
    constraint_space: &ConstraintSpace,
) {
    let box_ = node.GetLayoutBox();
    let box_size = layout_result.GetPhysicalFragment().Size();
    let shape_outside = unsafe { &mut *unsafe { &*box_ }.GetShapeOutsideInfo() };
    let writing_mode = unsafe { &*unsafe { &*box_ }.ContainingBlock() }
        .StyleRef()
        .GetWritingMode();
    let margins = ComputePhysicalMarginsForSpace(constraint_space, node.Style())
        .ConvertToLogical(WritingDirectionMode::new(writing_mode, TextDirection::kLtr));
    shape_outside.SetReferenceBoxLogicalSize(
        ToLogicalSize(box_size, writing_mode),
        LogicalSize::new(margins.InlineSum(), margins.BlockSum()),
    );
    shape_outside
        .SetPercentageResolutionInlineSize(constraint_space.PercentageResolutionInlineSize());
}
