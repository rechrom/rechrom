/*
 * Copyright (C) 2006 Apple Computer, Inc.
 * Copyright (C) 2009 Torch Mobile Inc. http://www.torchmobile.com/
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
// Copyright 2021 The Chromium Authors
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/hit_test_request.h:1-154
// Source: third_party/blink/renderer/core/layout/hit_test_request.cc:1-23

use std::cell::Cell;
use std::rc::Rc;

// Source: hit_test_request.h:36-38,119-120; hit_test_request.cc:11-16
// Borrow the resident LayoutObject identity for stop comparisons and its
// PaintingLayer identity for the source request's lazy layer cache.
pub trait HitTestStopNode<L: ?Sized> {
    /// Identity of the canonical resident LayoutObject, matching the source
    /// stop-node pointer comparison independently of its painting layer.
    fn Identity(&self) -> usize;
    fn NodeForHitTest(&self) -> Option<super::hit_test_result::NodeRef>;
    fn PaintingLayer(&self) -> Option<&L>;
}

// Source: hit_test_request.h:40-44
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListBasedHitTestBehavior {
    kContinueHitTesting,
    kStopHitTesting,
}

// Source: hit_test_request.h:50-70
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RequestType {
    kReadOnly = 1 << 1,
    kActive = 1 << 2,
    kMove = 1 << 3,
    kRelease = 1 << 4,
    kIgnoreClipping = 1 << 5,
    kSVGClipContent = 1 << 6,
    kTouchEvent = 1 << 7,
    kAllowChildFrameContent = 1 << 8,
    kChildFrameHitTest = 1 << 9,
    kIgnorePointerEventsNone = 1 << 10,
    kListBased = 1 << 11,
    kPenetratingList = 1 << 12,
    kAvoidCache = 1 << 13,
    kIgnoreZeroOpacityObjects = 1 << 14,
    kHitTestVisualOverflow = 1 << 15,
}

// Source: hit_test_request.h:72-74
pub type HitTestRequestType = u32;
pub type HitNodeCb<'a, N> = Rc<dyn Fn(&N) -> ListBasedHitTestBehavior + 'a>;

// Source: hit_test_request.h:46-49,141-149
// Rust borrows retain the source stop-node/layer identity without copying or
// exposing either owning engine. N is the existing DOM node passed to the cb.
pub struct HitTestRequest<'a, L: ?Sized, N: ?Sized> {
    request_type_: HitTestRequestType,
    stop_node_: Option<&'a dyn HitTestStopNode<L>>,
    stop_layer_: Cell<Option<&'a L>>,
    hit_node_cb_: Option<HitNodeCb<'a, N>>,
}

// Source: hit_test_request.h:46-150 (implicit memberwise copy).
// Copy the borrowed identities and repeating callback owner, not L or N.
impl<L: ?Sized, N: ?Sized> Clone for HitTestRequest<'_, L, N> {
    fn clone(&self) -> Self {
        Self {
            request_type_: self.request_type_,
            stop_node_: self.stop_node_,
            stop_layer_: Cell::new(self.stop_layer_.get()),
            hit_node_cb_: self.hit_node_cb_.clone(),
        }
    }
}

impl<'a, L: ?Sized, N: ?Sized> HitTestRequest<'a, L, N> {
    // Source: hit_test_request.h:50-70
    pub const kReadOnly: u32 = RequestType::kReadOnly as u32;
    pub const kActive: u32 = RequestType::kActive as u32;
    pub const kMove: u32 = RequestType::kMove as u32;
    pub const kRelease: u32 = RequestType::kRelease as u32;
    pub const kIgnoreClipping: u32 = RequestType::kIgnoreClipping as u32;
    pub const kSVGClipContent: u32 = RequestType::kSVGClipContent as u32;
    pub const kTouchEvent: u32 = RequestType::kTouchEvent as u32;
    pub const kAllowChildFrameContent: u32 = RequestType::kAllowChildFrameContent as u32;
    pub const kChildFrameHitTest: u32 = RequestType::kChildFrameHitTest as u32;
    pub const kIgnorePointerEventsNone: u32 = RequestType::kIgnorePointerEventsNone as u32;
    pub const kListBased: u32 = RequestType::kListBased as u32;
    pub const kPenetratingList: u32 = RequestType::kPenetratingList as u32;
    pub const kAvoidCache: u32 = RequestType::kAvoidCache as u32;
    pub const kIgnoreZeroOpacityObjects: u32 = RequestType::kIgnoreZeroOpacityObjects as u32;
    pub const kHitTestVisualOverflow: u32 = RequestType::kHitTestVisualOverflow as u32;

    // Source: hit_test_request.h:76-87
    pub fn new(
        request_type: HitTestRequestType,
        stop_node: Option<&'a dyn HitTestStopNode<L>>,
        hit_node_cb: Option<HitNodeCb<'a, N>>,
    ) -> Self {
        debug_assert!(
            request_type & Self::kPenetratingList == 0 || request_type & Self::kListBased != 0
        );
        debug_assert!(
            hit_node_cb.is_none()
                || (request_type & Self::kPenetratingList != 0
                    && request_type & Self::kAvoidCache != 0)
        );
        Self {
            request_type_: request_type,
            stop_node_: stop_node,
            stop_layer_: Cell::new(None),
            hit_node_cb_: hit_node_cb,
        }
    }

    // Source: hit_test_request.h:89-113
    pub fn ReadOnly(&self) -> bool {
        self.request_type_ & Self::kReadOnly != 0
    }
    pub fn Active(&self) -> bool {
        self.request_type_ & Self::kActive != 0
    }
    pub fn Move(&self) -> bool {
        self.request_type_ & Self::kMove != 0
    }
    pub fn Release(&self) -> bool {
        self.request_type_ & Self::kRelease != 0
    }
    pub fn IgnoreClipping(&self) -> bool {
        self.request_type_ & Self::kIgnoreClipping != 0
    }
    pub fn SvgClipContent(&self) -> bool {
        self.request_type_ & Self::kSVGClipContent != 0
    }
    pub fn TouchEvent(&self) -> bool {
        self.request_type_ & Self::kTouchEvent != 0
    }
    pub fn AllowsChildFrameContent(&self) -> bool {
        self.request_type_ & Self::kAllowChildFrameContent != 0
    }
    pub fn IsChildFrameHitTest(&self) -> bool {
        self.request_type_ & Self::kChildFrameHitTest != 0
    }
    pub fn IsHitTestVisualOverflow(&self) -> bool {
        self.request_type_ & Self::kHitTestVisualOverflow != 0
    }
    pub fn IgnorePointerEventsNone(&self) -> bool {
        self.request_type_ & Self::kIgnorePointerEventsNone != 0
    }
    pub fn ListBased(&self) -> bool {
        self.request_type_ & Self::kListBased != 0
    }
    pub fn PenetratingList(&self) -> bool {
        self.request_type_ & Self::kPenetratingList != 0
    }
    pub fn AvoidCache(&self) -> bool {
        self.request_type_ & Self::kAvoidCache != 0
    }
    pub fn UseHitNodeCb(&self) -> bool {
        self.hit_node_cb_.is_some()
    }

    // Source: hit_test_request.h:115-116
    pub fn TouchMove(&self) -> bool {
        self.Move() && self.TouchEvent()
    }
    // Source: hit_test_request.h:118-120
    pub fn GetType(&self) -> HitTestRequestType {
        self.request_type_
    }
    pub fn GetStopNode(&self) -> Option<&'a dyn HitTestStopNode<L>> {
        self.stop_node_
    }

    // Source: hit_test_request.h:122-125
    pub fn RunHitNodeCb(&self, node: &N) -> ListBasedHitTestBehavior {
        self.hit_node_cb_
            .as_ref()
            .expect("a hit-node callback must be installed")(node)
    }

    // Source: hit_test_request.h:127-137
    pub const kCacheabilityBits: u32 =
        Self::kReadOnly | Self::kActive | Self::kMove | Self::kRelease | Self::kTouchEvent;
    pub fn EqualForCacheability(&self, value: &HitTestRequest<'_, L, N>) -> bool {
        (self.request_type_ | Self::kCacheabilityBits)
            == (value.request_type_ | Self::kCacheabilityBits)
            && match (self.stop_node_, value.stop_node_) {
                (Some(a), Some(b)) => std::ptr::eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }

    // Source: hit_test_request.cc:11-16
    pub fn GetStopLayer(&self) -> Option<&'a L> {
        if let Some(stop_node) = self.stop_node_ {
            if self.stop_layer_.get().is_none() {
                self.stop_layer_.set(stop_node.PaintingLayer());
            }
        }
        self.stop_layer_.get()
    }
    // Source: hit_test_request.h:139; hit_test_request.cc:18-21
    // The GC Trace edges are represented by the live borrowed stop node/layer.
    // The embedding keeps their original owners alive for the request lifetime.
}
