//! Typed boundaries for commit payload members owned outside the translated
//! compositor-core source set.

use std::sync::Arc;

/// `cc::SwapPromise` remains an owned promise whose implementation is supplied
/// by the embedder that consumes compositor commits.
pub trait SwapPromise {}

/// Shared immutable animated-image frame indices are produced by paint/image
/// code and only carried through the compositor commit.
pub trait AnimatedImageFrameIndexMap {}

pub type SharedAnimatedImageFrameIndexMap = Arc<dyn AnimatedImageFrameIndexMap>;

// Source vocabulary: cc/trees/layer_tree_host_delegate.h:62-68.
pub type ManipulationInfo = u32;
pub const kManipulationInfoNone: ManipulationInfo = 0;
pub const kManipulationInfoWheel: ManipulationInfo = 1 << 0;
pub const kManipulationInfoTouch: ManipulationInfo = 1 << 1;
pub const kManipulationInfoPrecisionTouchPad: ManipulationInfo = 1 << 2;
pub const kManipulationInfoPinchZoom: ManipulationInfo = 1 << 3;
pub const kManipulationInfoScrollbar: ManipulationInfo = 1 << 4;

/// Mutator output is owned by the animation subsystem and moved with the
/// BeginMainFrame state.
pub trait MutatorEvents {}

pub type ActiveFrameSequenceTrackers = u16;
pub type BeginMainFrameReasons = u16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BeginMainFrameTraceId(pub u64);

/// Supplies the global trace identity that Chromium obtains from base tracing.
/// The core stays deterministic and does not reach a process-global singleton.
pub trait BeginMainFrameTraceIdProvider {
    fn NextTraceId(&mut self) -> BeginMainFrameTraceId;
}
