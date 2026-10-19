// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::adapter::begin_frame::BeginFrameArgs;
use std::{cell::RefCell, rc::Rc};

// Source: cc/scheduler/video_frame_controller.h:15-22
pub trait VideoFrameControllerClient {
    fn AddVideoFrameController(&mut self, controller: Rc<RefCell<dyn VideoFrameController>>);
    fn RemoveVideoFrameController(&mut self, controller: &Rc<RefCell<dyn VideoFrameController>>);
}
// Source: cc/scheduler/video_frame_controller.h:24-36
pub trait VideoFrameController {
    fn OnBeginFrame(&mut self, args: &BeginFrameArgs);
    // Called after DidDrawAllLayers, including aborted draws.
    fn DidDrawFrame(&mut self);
}
