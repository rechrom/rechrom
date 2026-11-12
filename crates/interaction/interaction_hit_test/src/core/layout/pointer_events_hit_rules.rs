/*
    Copyright (C) 2007 Rob Buis <buis@kde.org>

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Library General Public
    License as published by the Free Software Foundation; either
    version 2 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
    Library General Public License for more details.

    You should have received a copy of the GNU Library General Public License
    aint with this library; see the file COPYING.LIB. If not, write to
    the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
    Boston, MA 02110-1301, USA.
*/
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/pointer_events_hit_rules.h:1-52
// Source: third_party/blink/renderer/core/layout/pointer_events_hit_rules.cc:1-127

use super::hit_test_request::HitTestRequest;
use foundation::EPointerEvents;

// Source: pointer_events_hit_rules.h:34-38
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EHitTesting {
    kSvgImageHitTesting,
    kSvgGeometryHitTesting,
    kSvgTextHitTesting,
}

// Source: pointer_events_hit_rules.h:30-48
pub struct PointerEventsHitRules {
    pub require_visible: bool,
    pub require_fill: bool,
    pub require_stroke: bool,
    pub can_hit_stroke: bool,
    pub can_hit_fill: bool,
    pub can_hit_bounding_box: bool,
}

impl PointerEventsHitRules {
    // Source: pointer_events_hit_rules.cc:33-125
    pub fn new<L: ?Sized, N: ?Sized>(
        hit_testing: EHitTesting,
        request: &HitTestRequest<'_, L, N>,
        mut pointer_events: EPointerEvents,
    ) -> Self {
        let mut rules = Self {
            require_visible: false,
            require_fill: false,
            require_stroke: false,
            can_hit_stroke: false,
            can_hit_fill: false,
            can_hit_bounding_box: false,
        };
        if request.SvgClipContent() {
            pointer_events = EPointerEvents::kFill;
        }
        if hit_testing == EHitTesting::kSvgGeometryHitTesting {
            match pointer_events {
                EPointerEvents::kBoundingBox => rules.can_hit_bounding_box = true,
                EPointerEvents::kVisiblepainted | EPointerEvents::kAuto => {
                    rules.require_fill = true;
                    rules.require_stroke = true;
                    rules.require_visible = true;
                    rules.can_hit_fill = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kVisible => {
                    rules.require_visible = true;
                    rules.can_hit_fill = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kVisiblefill => {
                    rules.require_visible = true;
                    rules.can_hit_fill = true;
                }
                EPointerEvents::kVisiblestroke => {
                    rules.require_visible = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kPainted => {
                    rules.require_fill = true;
                    rules.require_stroke = true;
                    rules.can_hit_fill = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kAll => {
                    rules.can_hit_fill = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kFill => rules.can_hit_fill = true,
                EPointerEvents::kStroke => rules.can_hit_stroke = true,
                EPointerEvents::kNone => {}
            }
        } else {
            match pointer_events {
                EPointerEvents::kBoundingBox => rules.can_hit_bounding_box = true,
                EPointerEvents::kVisiblepainted | EPointerEvents::kAuto => {
                    rules.require_visible = true;
                    rules.require_fill = true;
                    rules.require_stroke = true;
                    rules.can_hit_fill = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kVisiblefill
                | EPointerEvents::kVisiblestroke
                | EPointerEvents::kVisible => {
                    rules.require_visible = true;
                    rules.can_hit_fill = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kPainted => {
                    rules.require_fill = true;
                    rules.require_stroke = true;
                    rules.can_hit_fill = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kFill | EPointerEvents::kStroke | EPointerEvents::kAll => {
                    rules.can_hit_fill = true;
                    rules.can_hit_stroke = true;
                }
                EPointerEvents::kNone => {}
            }
        }
        rules
    }
}
