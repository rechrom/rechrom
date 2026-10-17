//! Window presentation executor policy. Page execution belongs to page_runtime;
//! window_runtime only schedules Display from already produced Page frames.
#[cfg(test)]
use crate::{
    display::{self, Display},
    engine::{Output, UserEvent},
    window_surface::DisplayTarget,
};
#[cfg(test)]
use foundation::begin_frame::BeginFrameSource;
#[cfg(test)]
use std::{
    collections::VecDeque,
    io,
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

#[cfg(test)]
pub(crate) use crate::page_presentation::{
    Command as CompositorCommand, Snapshot as ArtifactSnapshot,
};
#[cfg(test)]
use crate::page_presentation::{Effect as CompositorEffect, WindowCompositor as CompositorEngine};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DisplayExecutor {
    #[default]
    DedicatedThread,
    CompositorThread,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Config {
    pub display_executor: DisplayExecutor,
    pub capture_frames: bool,
}

#[cfg(test)]
#[path = "presentation_legacy.rs"]
mod legacy;
#[cfg(test)]
pub(crate) use legacy::{MainThreadClient, ReceiveSwapAcks, Spawn};
