// Translation status: complete
// Source: cc/input/browser_controls_offset_manager_client.h:1-42
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/input/browser_controls_offset_manager_client.h:1-17,35-42.
// Imports/trait objects replace includes, forwards, namespace, guards and the
// protected virtual destructor. Mutation callbacks use an explicit mutable
// live facade; the manager never retains a self-reference to its owner.
use crate::adapter::browser_controls::{
    BrowserControlsEnvironment, BrowserControlsReadEnvironment,
};
use crate::adapter::input_geometry::PointF;
use crate::adapter::time::TimeDelta;
/// Source const client queries, split so a getter can borrow live fields shared.
pub trait BrowserControlsOffsetManagerQueries: BrowserControlsReadEnvironment {
    // Source: cc/input/browser_controls_offset_manager_client.h:18-21.
    fn TopControlsHeight(&self) -> f32;
    fn TopControlsMinHeight(&self) -> f32;
    fn BottomControlsHeight(&self) -> f32;
    fn BottomControlsMinHeight(&self) -> f32;
    // Source: cc/input/browser_controls_offset_manager_client.h:24-27.
    fn CurrentTopControlsShownRatio(&self) -> f32;
    fn CurrentBottomControlsShownRatio(&self) -> f32;
    fn ViewportScrollOffset(&self) -> PointF;
    fn MaxViewportScrollOffsetY(&self) -> f32;
    // Source: cc/input/browser_controls_offset_manager_client.h:29-30,33-34.
    fn OnlyExpandTopControlsAtPageTop(&self) -> bool;
    fn HaveRootScrollNode(&self) -> bool;
    fn CurrentFrameInterval(&self) -> TimeDelta;
    fn DeviceScaleFactor(&self) -> f32;
}
pub trait BrowserControlsOffsetManagerClient:
    BrowserControlsOffsetManagerQueries + BrowserControlsEnvironment
{
    // Source: cc/input/browser_controls_offset_manager_client.h:22-23,28.
    // Rust explicit receiver reborrow replaces the source client->owner->same
    // manager raw_ptr traversal; only the callback invocation owns this borrow.
    fn SetCurrentBrowserControlsShownRatio(
        &mut self,
        top_ratio: f32,
        bottom_ratio: f32,
        manager:&mut crate::cc::input::browser_controls_offset_manager::BrowserControlsOffsetManager,
    );
    fn DidChangeBrowserControlsPosition(
        &mut self,
        manager:&mut crate::cc::input::browser_controls_offset_manager::BrowserControlsOffsetManager,
    );
    // Source: cc/input/browser_controls_offset_manager_client.h:31-32.
    fn SetNeedsCommit(&mut self, reason: Self::BeginMainFrameReason);
    fn SetNeedsCommitDefault(&mut self) {
        self.SetNeedsCommit(self.OtherBeginMainFrameReason());
    }
}
