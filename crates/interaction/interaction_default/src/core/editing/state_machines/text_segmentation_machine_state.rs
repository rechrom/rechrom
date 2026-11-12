// Copyright 2016 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/state_machines/text_segmentation_machine_state.h:12-26.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSegmentationMachineState {
    Invalid,
    NeedFollowingCodeUnit,
    NeedMoreCodeUnit,
    Finished,
}
