/*
 * Copyright (C) 2006, 2008, 2011 Apple Inc. All rights reserved.
 * Copyright (C) 2012 Nokia Corporation and/or its subsidiary(-ies)
 *
 * This library is free software; you can redistribute it and/or modify it
 * under the terms of the GNU Library General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or (at your
 * option) any later version.
 * This library is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU Library General Public License
 * for more details. You should have received a copy along with this library;
 * see COPYING.LIB, or write to the Free Software Foundation, Inc., 51 Franklin
 * Street, Fifth Floor, Boston, MA 02110-1301, USA.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/hit_test_location.h:1-136
// Source: third_party/blink/renderer/core/layout/hit_test_location.cc:1-308
// Rust modules represent includes, include guards, export and namespaces.

use super::geometry::physical_rect::PhysicalRectHitTestExt;
use crate::platform::geometry::contoured_rect::ContouredRectHitTest;
use crate::platform::geometry::path::{PathHitTest, WindRule};
use crate::ui::gfx::geometry::quad_f::QuadFHitTestExt;
use foundation::{gfx, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize};

// Source: hit_test_location.h:38-40,115-131
#[derive(Clone, Copy, Debug)]
pub struct HitTestLocation {
    point_: PhysicalOffset,
    bounding_box_: PhysicalRect,
    transformed_point_: gfx::PointF,
    transformed_rect_: gfx::QuadF,
    fragment_index_: i32,
    is_rect_based_: bool,
    is_rectilinear_: bool,
}

impl HitTestLocation {
    // Source: hit_test_location.h:71-82
    pub fn Point(&self) -> &PhysicalOffset {
        &self.point_
    }
    pub fn RoundedPoint(&self) -> gfx::Point {
        foundation::ToRoundedPoint(&self.point_)
    }
    pub fn FragmentIndex(&self) -> i32 {
        self.fragment_index_
    }
    pub fn IsRectBasedTest(&self) -> bool {
        self.is_rect_based_
    }
    pub fn IsRectilinear(&self) -> bool {
        self.is_rectilinear_
    }
    pub fn BoundingBox(&self) -> &PhysicalRect {
        &self.bounding_box_
    }
    pub fn ToEnclosingRect(&self) -> gfx::Rect {
        self.bounding_box_.ToEnclosingRect()
    }

    // Source: hit_test_location.h:84-87
    pub fn RectForPoint(point: PhysicalOffset) -> PhysicalRect {
        PhysicalRect::new(
            point,
            PhysicalSize::new(LayoutUnit::from_signed(1), LayoutUnit::from_signed(1)),
        )
    }

    // Rust's generic call retains the source overload spelling for native
    // PhysicalRect, RectF and QuadF inputs without projecting their storage.
    // Source: hit_test_location.h:89-96
    pub fn Intersects<T: HitTestLocationIntersects + ?Sized>(&self, value: &T) -> bool {
        value.IntersectsLocation(self)
    }

    // Source: hit_test_location.h:109-110
    pub fn TransformedPoint(&self) -> &gfx::PointF {
        &self.transformed_point_
    }
    pub fn TransformedRect(&self) -> &gfx::QuadF {
        &self.transformed_rect_
    }
}

// Source: hit_test_location.cc:34-52
fn PointInRectangleStroke(point: gfx::PointF, rect: &gfx::RectF, stroke_width: f32) -> bool {
    let half_stroke_width = stroke_width / 2.0;
    let half_width = rect.width() / 2.0;
    let half_height = rect.height() / 2.0;
    let rect_center = gfx::PointF::new(rect.x() + half_width, rect.y() + half_height);
    let abs_delta_x = (point.x() - rect_center.x()).abs();
    let abs_delta_y = (point.y() - rect_center.y()).abs();
    if !(abs_delta_x <= half_width + half_stroke_width
        && abs_delta_y <= half_height + half_stroke_width)
    {
        return false;
    }
    half_width - half_stroke_width <= abs_delta_x || half_height - half_stroke_width <= abs_delta_y
}

// Source: hit_test_location.cc:54-75
fn QuadIntersectsRectangleStroke(quad: &gfx::QuadF, rect: &gfx::RectF, stroke_width: f32) -> bool {
    let half_stroke_width = stroke_width / 2.0;
    let mut outer_edge = *rect;
    // RectF::Outset(float) delegates to Inset(-outset).
    outer_edge.Inset(-half_stroke_width);
    if !quad.IntersectsRect(&outer_edge) {
        return false;
    }
    let mut inner_edge = *rect;
    inner_edge.Inset(half_stroke_width);
    if inner_edge.InclusiveContains(*quad.p1())
        && inner_edge.InclusiveContains(*quad.p2())
        && inner_edge.InclusiveContains(*quad.p3())
        && inner_edge.InclusiveContains(*quad.p4())
    {
        return false;
    }
    true
}

// Source: hit_test_location.cc:77-88
fn PointInEllipse(point: gfx::PointF, center: gfx::PointF, radii: gfx::SizeF) -> bool {
    let point_to_center = gfx::PointF::new(center.x() - point.x(), center.y() - point.y());
    let xr_x = point_to_center.x() / radii.width();
    let yr_y = point_to_center.y() / radii.height();
    xr_x * xr_x + yr_y * yr_y <= 1.0
}

// Source: hit_test_location.cc:90-92
fn DistanceBetween(from: gfx::PointF, to: gfx::PointF) -> f32 {
    (to - from).Length()
}

// Source: hit_test_location.cc:94-100
fn PointInCircleStroke(
    point: gfx::PointF,
    center: gfx::PointF,
    radius: f32,
    stroke_width: f32,
) -> bool {
    let half_stroke_width = stroke_width / 2.0;
    (DistanceBetween(point, center) - radius).abs() <= half_stroke_width
}

// Source: hit_test_location.cc:102-121
fn QuadIntersectsCircleStroke(
    quad: &gfx::QuadF,
    center: gfx::PointF,
    radius: f32,
    stroke_width: f32,
) -> bool {
    let half_stroke_width = stroke_width / 2.0;
    if !quad.IntersectsCircle(center, radius + half_stroke_width) {
        return false;
    }
    let inner_edge_radius = radius - half_stroke_width;
    if DistanceBetween(*quad.p1(), center) < inner_edge_radius
        && DistanceBetween(*quad.p2(), center) < inner_edge_radius
        && DistanceBetween(*quad.p3(), center) < inner_edge_radius
        && DistanceBetween(*quad.p4(), center) < inner_edge_radius
    {
        return false;
    }
    true
}

// Source: hit_test_location.cc:123-132
const kMaxRectHitTestVerbs: i32 = 500;
fn CanUseRectHitTestForPath(path: &(impl PathHitTest + ?Sized)) -> bool {
    path.CountVerbs() <= kMaxRectHitTestVerbs
}

// Source: hit_test_location.cc:136-137
impl Default for HitTestLocation {
    fn default() -> Self {
        Self {
            point_: PhysicalOffset::default(),
            bounding_box_: PhysicalRect::default(),
            transformed_point_: gfx::PointF::default(),
            transformed_rect_: gfx::QuadF::default(),
            fragment_index_: -1,
            is_rect_based_: false,
            is_rectilinear_: true,
        }
    }
}

impl HitTestLocation {
    // Source: hit_test_location.h:49; hit_test_location.cc:136-137
    pub fn new() -> Self {
        Self::default()
    }

    // Source: hit_test_location.h:51; hit_test_location.cc:139-140
    pub fn FromPoint(point: gfx::Point) -> Self {
        Self::FromPhysicalOffset(PhysicalOffset::FromPoint(&point))
    }

    // Source: hit_test_location.h:50; hit_test_location.cc:142-148
    pub fn FromPhysicalOffset(point: PhysicalOffset) -> Self {
        let bounding_box = Self::RectForPoint(point);
        Self {
            point_: point,
            bounding_box_: bounding_box,
            transformed_point_: point.into(),
            transformed_rect_: gfx::QuadF::from(gfx::RectF::from(bounding_box)),
            fragment_index_: -1,
            is_rect_based_: false,
            is_rectilinear_: true,
        }
    }

    // Source: hit_test_location.h:52; hit_test_location.cc:150-156
    pub fn FromPointF(point: gfx::PointF) -> Self {
        let physical_point = PhysicalOffset::FromPointFFloor(&point);
        let bounding_box = Self::RectForPoint(physical_point);
        Self {
            point_: physical_point,
            bounding_box_: bounding_box,
            transformed_point_: point,
            transformed_rect_: gfx::QuadF::from(gfx::RectF::from(bounding_box)),
            fragment_index_: -1,
            is_rect_based_: false,
            is_rectilinear_: true,
        }
    }

    // Source: hit_test_location.h:56-64; hit_test_location.cc:158-165
    pub fn FromPointAndBoundingBox(point: gfx::PointF, bounding_box: PhysicalRect) -> Self {
        Self {
            point_: PhysicalOffset::FromPointFFloor(&point),
            bounding_box_: bounding_box,
            transformed_point_: point,
            transformed_rect_: gfx::QuadF::from(gfx::RectF::from(bounding_box)),
            fragment_index_: -1,
            is_rect_based_: false,
            is_rectilinear_: true,
        }
    }

    // Source: hit_test_location.h:53; hit_test_location.cc:167-173
    pub fn FromPointAndQuad(point: gfx::PointF, quad: gfx::QuadF) -> Self {
        Self {
            point_: PhysicalOffset::FromPointFFloor(&point),
            bounding_box_: PhysicalRect::EnclosingRect(&quad.BoundingBox()),
            transformed_point_: point,
            transformed_rect_: quad,
            fragment_index_: -1,
            is_rect_based_: true,
            is_rectilinear_: quad.IsRectilinear(),
        }
    }

    // Source: hit_test_location.h:54; hit_test_location.cc:175-182
    pub fn FromPhysicalRect(rect: PhysicalRect) -> Self {
        let point = rect.Center();
        Self {
            point_: point,
            bounding_box_: rect,
            transformed_point_: point.into(),
            transformed_rect_: gfx::QuadF::from(gfx::RectF::from(rect)),
            fragment_index_: -1,
            is_rect_based_: true,
            is_rectilinear_: true,
        }
    }

    // Source: hit_test_location.h:66; hit_test_location.cc:184-193
    pub fn WithOffset(other: &Self, offset: PhysicalOffset) -> Self {
        let mut result = Self {
            point_: other.point_,
            bounding_box_: other.bounding_box_,
            transformed_point_: other.transformed_point_,
            transformed_rect_: other.transformed_rect_,
            fragment_index_: -1,
            is_rect_based_: other.is_rect_based_,
            is_rectilinear_: other.is_rectilinear_,
        };
        result.Move(offset);
        result
    }

    // Source: hit_test_location.h:67; hit_test_location.cc:195-203
    pub fn WithFragmentIndex(other: &Self, fragment_index: usize) -> Self {
        Self {
            fragment_index_: fragment_index as i32,
            ..*other
        }
    }

    // Source: hit_test_location.h:68-69; hit_test_location.cc:205-208
    // Derived Copy/Clone and Rust assignment implement the source's defaulted
    // copy/assignment (including fragment_index_). Only WithOffset resets it.

    // Source: hit_test_location.h:113; hit_test_location.cc:210-215
    fn Move(&mut self, offset: PhysicalOffset) {
        self.point_ += offset;
        self.bounding_box_.Move(&offset);
        self.transformed_point_ += gfx::Vector2dF::from(offset);
        self.transformed_rect_.Move(gfx::Vector2dF::from(offset));
    }

    // Source: hit_test_location.h:89; hit_test_location.cc:217-238
    pub fn IntersectsPhysicalRect(&self, rect: &PhysicalRect) -> bool {
        if !rect.Intersects(&self.bounding_box_) {
            return false;
        }
        if self.is_rectilinear_ {
            return true;
        }
        if rect.Contains(&self.bounding_box_) {
            return true;
        }
        self.transformed_rect_
            .IntersectsRectPartial(&gfx::RectF::from(*rect))
    }

    // Source: hit_test_location.h:91-93; hit_test_location.cc:240-244
    pub fn IntersectsRectF(&self, rect: &gfx::RectF) -> bool {
        if self.is_rect_based_ {
            return self.transformed_rect_.IntersectsRect(rect);
        }
        rect.InclusiveContains(self.transformed_point_)
    }

    // Source: hit_test_location.h:94; hit_test_location.cc:246-248
    pub fn IntersectsContouredRect(&self, rect: &(impl ContouredRectHitTest + ?Sized)) -> bool {
        rect.IntersectsQuad(&self.transformed_rect_)
    }

    // Source: hit_test_location.h:95; hit_test_location.cc:250-261
    pub fn IntersectsQuad(&self, quad: &gfx::QuadF) -> bool {
        if self.is_rect_based_ {
            if !self.IntersectsRectF(&quad.BoundingBox()) {
                return false;
            }
            if quad.IsRectilinear() {
                return true;
            }
            return quad.IntersectsQuad(&self.transformed_rect_);
        }
        quad.Contains(self.transformed_point_)
    }

    // Source: hit_test_location.h:96; hit_test_location.cc:263-265
    pub fn ContainsPoint(&self, point: gfx::PointF) -> bool {
        self.transformed_rect_.Contains(point)
    }

    // Source: hit_test_location.h:98; hit_test_location.cc:267-272
    pub fn IntersectsPath(&self, path: &(impl PathHitTest + ?Sized)) -> bool {
        if self.is_rect_based_ && CanUseRectHitTestForPath(path) {
            return path.Intersects(&self.transformed_rect_);
        }
        path.Contains(self.transformed_point_)
    }

    // Source: hit_test_location.h:99; hit_test_location.cc:274-280
    pub fn IntersectsPathWithWindRule(
        &self,
        path: &(impl PathHitTest + ?Sized),
        winding_rule: WindRule,
    ) -> bool {
        if self.is_rect_based_ && CanUseRectHitTestForPath(path) {
            return path.IntersectsWithWindRule(&self.transformed_rect_, winding_rule);
        }
        path.ContainsWithWindRule(self.transformed_point_, winding_rule)
    }

    // Source: hit_test_location.h:101; hit_test_location.cc:282-288
    pub fn IntersectsStroke(&self, rect: &gfx::RectF, stroke_width: f32) -> bool {
        if self.is_rect_based_ {
            return QuadIntersectsRectangleStroke(&self.transformed_rect_, rect, stroke_width);
        }
        PointInRectangleStroke(self.transformed_point_, rect, stroke_width)
    }

    // Source: hit_test_location.h:103-104; hit_test_location.cc:290-296
    pub fn IntersectsEllipse(&self, center: gfx::PointF, radii: gfx::SizeF) -> bool {
        if self.is_rect_based_ {
            return self.transformed_rect_.IntersectsEllipse(center, radii);
        }
        PointInEllipse(self.transformed_point_, center, radii)
    }

    // Source: hit_test_location.h:105-107; hit_test_location.cc:298-306
    pub fn IntersectsCircleStroke(
        &self,
        center: gfx::PointF,
        radius: f32,
        stroke_width: f32,
    ) -> bool {
        if self.is_rect_based_ {
            return QuadIntersectsCircleStroke(
                &self.transformed_rect_,
                center,
                radius,
                stroke_width,
            );
        }
        PointInCircleStroke(self.transformed_point_, center, radius, stroke_width)
    }
}

// Rust overload routing only: the methods above own all source decisions.
pub trait HitTestLocationIntersects {
    fn IntersectsLocation(&self, location: &HitTestLocation) -> bool;
}
impl HitTestLocationIntersects for PhysicalRect {
    fn IntersectsLocation(&self, location: &HitTestLocation) -> bool {
        location.IntersectsPhysicalRect(self)
    }
}
impl HitTestLocationIntersects for gfx::RectF {
    fn IntersectsLocation(&self, location: &HitTestLocation) -> bool {
        location.IntersectsRectF(self)
    }
}
impl HitTestLocationIntersects for gfx::QuadF {
    fn IntersectsLocation(&self, location: &HitTestLocation) -> bool {
        location.IntersectsQuad(self)
    }
}
