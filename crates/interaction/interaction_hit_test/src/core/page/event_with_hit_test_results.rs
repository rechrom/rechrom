/*
   Copyright (C) 2000 Simon Hausmann <hausmann@kde.org>
   Copyright (C) 2006 Apple Computer, Inc.

   This library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Library General Public
   License as published by the Free Software Foundation; either
   version 2 of the License, or (at your option) any later version.

   This library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
   Library General Public License for more details.

   You should have received a copy of the GNU Library General Public License
   along with this library; see the file COPYING.LIB. If not, write to
   the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
   Boston, MA 02110-1301, USA.
*/
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/page/event_with_hit_test_results.h:1-78
// Source: event_with_hit_test_results.h:21-35,76-78
// Rust modules and the scoped value replace guards/includes/namespaces and
// STACK_ALLOCATED. The result retains its existing request borrows and shared
// DOM/scrollbar owners when the source value copy is constructed.

use foundation::PhysicalOffset;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;

use crate::core::layout::hit_test_location::HitTestLocation;
use crate::core::layout::hit_test_result::{HitTestResult, NodeRef, ScrollbarRef};

// Source: event_with_hit_test_results.h:33-35,66-70
pub struct EventWithHitTestResults<'a, E, L: ?Sized> {
    event_: E,
    hit_test_result_: HitTestResult<'a, L>,
    hit_test_location_: HitTestLocation,
}

impl<'a, E: Clone, L: ?Sized> EventWithHitTestResults<'a, E, L> {
    // Source: event_with_hit_test_results.h:38-43
    pub fn new(
        event: &E,
        location: &HitTestLocation,
        hit_test_result: &HitTestResult<'a, L>,
    ) -> Self {
        Self {
            event_: event.clone(),
            hit_test_result_: hit_test_result.clone(),
            hit_test_location_: *location,
        }
    }
}

impl<'a, E, L: ?Sized> EventWithHitTestResults<'a, E, L> {
    // Source: event_with_hit_test_results.h:45
    pub fn Event(&self) -> &E {
        &self.event_
    }

    // Source: event_with_hit_test_results.h:46
    pub fn GetHitTestResult(&self) -> &HitTestResult<'a, L> {
        &self.hit_test_result_
    }

    // Source: event_with_hit_test_results.h:47
    pub fn LocalPoint(&self) -> PhysicalOffset {
        *self.hit_test_result_.LocalPoint()
    }

    // Source: event_with_hit_test_results.h:48
    pub fn GetScrollbar(&self) -> Option<&ScrollbarRef> {
        self.hit_test_result_.GetScrollbar()
    }

    // Source: event_with_hit_test_results.h:49
    pub fn IsOverLink(&self) -> bool {
        self.hit_test_result_.IsOverLink()
    }

    // Source: event_with_hit_test_results.h:50-52
    pub fn IsOverEmbeddedContentView(&self) -> bool {
        self.hit_test_result_.IsOverEmbeddedContentView()
    }

    // Source: event_with_hit_test_results.h:53
    pub fn InnerNode(&self) -> Option<&NodeRef> {
        self.hit_test_result_.InnerNode()
    }

    // Source: event_with_hit_test_results.h:54
    pub fn InnerElement(&self) -> Option<&NodeRef> {
        self.hit_test_result_.InnerElement()
    }

    // Source: event_with_hit_test_results.h:55-57
    pub fn InnerPossiblyPseudoElement(&self) -> Option<&NodeRef> {
        self.hit_test_result_.InnerPossiblyPseudoElement()
    }

    // Source: event_with_hit_test_results.h:59-61
    pub fn GetHitTestLocation(&self) -> &HitTestLocation {
        &self.hit_test_location_
    }

    // Source: event_with_hit_test_results.h:62-64
    pub fn SetHitTestLocation(&mut self, new_location: &HitTestLocation) {
        self.hit_test_location_ = *new_location;
    }
}

// Source: event_with_hit_test_results.h:72
pub type MouseEventWithHitTestResults<'a, L> = EventWithHitTestResults<'a, WebMouseEvent, L>;

// Source: event_with_hit_test_results.h:74
// The source template only copies its event and exposes a const reference;
// no gesture fields or algorithms cross this boundary. G retains the concrete
// WebGestureEvent value once that type is admitted by the input package.
pub type GestureEventWithHitTestResults<'a, G, L> = EventWithHitTestResults<'a, G, L>;
