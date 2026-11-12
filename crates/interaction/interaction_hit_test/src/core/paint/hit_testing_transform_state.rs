/*
 * Copyright (C) 2011 Apple Inc.  All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/paint/hit_testing_transform_state.h:1-88
// Source: third_party/blink/renderer/core/paint/hit_testing_transform_state.cc:1-83
// Rust modules replace include guards, includes, namespaces and allocation macros.

use foundation::{gfx, InfiniteIntRect, PhysicalRect};

// Source: hit_testing_transform_state.h:41,65; .cc:36-39
// Borrow the canonical property node only for its origin-adjusted matrix.
pub trait TransformPaintPropertyNode {
    fn MatrixWithOriginApplied(&self) -> gfx::Transform;
}

// Source: hit_testing_transform_state.h:43-44,60-62,77-83
#[derive(Clone, Debug)]
pub struct HitTestingTransformState {
    last_planar_point_: gfx::PointF,
    last_planar_quad_: gfx::QuadF,
    last_planar_area_: Option<gfx::QuadF>,
    accumulated_transform_: gfx::Transform,
}

impl HitTestingTransformState {
    // Source: hit_testing_transform_state.h:47-58
    pub fn new(p: gfx::PointF, quad: gfx::QuadF, area: PhysicalRect) -> Self {
        Self {
            last_planar_point_: p,
            last_planar_quad_: quad,
            last_planar_area_: if area != PhysicalRect::from(InfiniteIntRect()) {
                Some(gfx::QuadF::from(gfx::RectF::from(area)))
            } else {
                None
            },
            accumulated_transform_: gfx::Transform::default(),
        }
    }

    // Source: hit_testing_transform_state.h:64; .cc:32-34
    pub fn Translate(&mut self, offset: gfx::Vector2dF) {
        self.accumulated_transform_
            .Translate(offset.x(), offset.y());
    }

    // Source: hit_testing_transform_state.h:65; .cc:36-39
    pub fn ApplyPropertyTransform(&mut self, transform: &dyn TransformPaintPropertyNode) {
        self.accumulated_transform_
            .PreConcat(&transform.MatrixWithOriginApplied());
    }

    // Source: hit_testing_transform_state.h:66; .cc:41-43
    pub fn ApplyTransform(&mut self, transform: &gfx::Transform) {
        self.accumulated_transform_.PreConcat(transform);
    }

    // Source: hit_testing_transform_state.h:72; .cc:45-56
    pub fn Flatten(&mut self) {
        let mut inverse_transform = gfx::Transform::default();
        if self
            .accumulated_transform_
            .GetInverse(&mut inverse_transform)
        {
            self.last_planar_point_ = inverse_transform.ProjectPoint(self.last_planar_point_);
            self.last_planar_quad_ = inverse_transform.ProjectQuad(self.last_planar_quad_);
            if let Some(area) = &mut self.last_planar_area_ {
                *area = inverse_transform.ProjectQuad(*area);
            }
        }
        self.accumulated_transform_.MakeIdentity();
    }

    // Source: hit_testing_transform_state.h:68; .cc:58-61
    pub fn MappedPoint(&self) -> gfx::PointF {
        self.accumulated_transform_
            .InverseOrIdentity()
            .ProjectPoint(self.last_planar_point_)
    }

    // Source: hit_testing_transform_state.h:69; .cc:63-66
    pub fn MappedQuad(&self) -> gfx::QuadF {
        self.accumulated_transform_
            .InverseOrIdentity()
            .ProjectQuad(self.last_planar_quad_)
    }

    // Source: hit_testing_transform_state.h:70; .cc:68-70
    pub fn BoundsOfMappedQuad(&self) -> PhysicalRect {
        self.BoundsOfMappedQuadInternal(&self.last_planar_quad_)
    }

    // Source: hit_testing_transform_state.h:71; .cc:72-75
    pub fn BoundsOfMappedArea(&self) -> PhysicalRect {
        self.last_planar_area_.as_ref().map_or_else(
            || PhysicalRect::from(InfiniteIntRect()),
            |area| self.BoundsOfMappedQuadInternal(area),
        )
    }

    // Source: hit_testing_transform_state.h:73-75
    pub fn AccumulatedTransform(&self) -> gfx::Transform {
        self.accumulated_transform_.clone()
    }

    // Source: hit_testing_transform_state.h:78; .cc:77-81
    fn BoundsOfMappedQuadInternal(&self, q: &gfx::QuadF) -> PhysicalRect {
        PhysicalRect::EnclosingRect(
            &self
                .accumulated_transform_
                .InverseOrIdentity()
                .ProjectQuad(*q)
                .BoundingBox(),
        )
    }
}
