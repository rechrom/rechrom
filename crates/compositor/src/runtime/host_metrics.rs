//! Actual external metrics owners. The canonical host retains only these native
//! allocations; all cc management decisions remain in the translated owner.
//! No implementation or substitute metric state is supplied by this module.
use crate::adapter::layer_tree_host_impl::*;

pub trait NativeReportingController<B: HostImplTypes>:
    crate::adapter::scheduler_host::CompositorFrameReportingController
{
    fn ReportingSetScrollJankOsReporter(&self, owner: HostImplWeak);
    fn ReportingSetFrameSorter(&self, sorter: &mut B::FrameSorter);
    fn ReportingSetFrameSequenceTrackerCollection(
        &self,
        trackers: &mut B::FrameSequenceTrackerCollection,
    );
    fn ReportingSetEventLatencyTracker(
        &self,
        owner: Option<crate::adapter::layer_tree_impl::LayerTreeHostId>,
    );
    fn ReportingClearFrameSequenceTrackerCollection(&self);
    fn ReportingSetNeedsRasterPropertiesAnimated(&self, animated: bool);
    fn ReportingSetVisible(&self, visible: bool);
}
pub struct ReportingController<B: HostImplTypes>(
    pub std::sync::Arc<dyn NativeReportingController<B>>,
);

pub trait NativeFrameSequenceTrackers<B: HostImplTypes> {
    fn GetScrollingThread(&self) -> super::input::SmoothEffectDrivingThread;
    fn StartMainSequence(
        &self,
        sequence: crate::adapter::client_layer_tree_host_impl::ClientSequence,
    );
    fn StartCustomSequence(&self, id: usize);
    fn StopCustomSequence(&self, id: usize);

    fn TrackersUpdateSmoothMainThreadHistory(&self, modifier: i32);
    fn TrackersSetCustomResultsAddedCallback(&self, callback: HostImplWeak);
    fn FrameSequenceTrackerActiveTypes(&self) -> B::ActiveFrameSequenceTrackers;
    fn TrackersStartPinchZoom(&self);
    fn TrackersStopPinchZoom(&self);
    fn TrackersStartSequence(&self, kind: HostAnimationSequence);
    fn TrackersStopSequence(&self, kind: HostAnimationSequence);
    fn TrackersNotifyBeginImplFrame(&self, args: &crate::adapter::begin_frame::BeginFrameArgs);
    fn TrackersNotifyFrameEnd(
        &self,
        current: &crate::adapter::begin_frame::BeginFrameArgs,
        args: &crate::adapter::begin_frame::BeginFrameArgs,
    );
    fn FrameTrackersNotifyPauseFrameProduction(&self);
    fn FrameTrackersClearAll(&self);
    fn FrameTrackersStartScrollSequence(
        &self,
        sequence: B::FrameSequenceTrackerType,
        thread: B::SmoothEffectDrivingThread,
    );
    fn FrameTrackersStopSequence(&self, sequence: B::FrameSequenceTrackerType);
    fn StopDrawSequence(&self, kind: HostDrawSequence);
}
pub struct FrameSequenceTrackers<B: HostImplTypes>(
    pub std::sync::Arc<dyn NativeFrameSequenceTrackers<B>>,
);
impl<B: HostImplTypes> FrameSequenceTrackers<B> {
    pub(crate) fn StopDrawSequence(&self, kind: HostDrawSequence) {
        self.0.StopDrawSequence(kind)
    }
}

// Source dependency: cc/metrics/frame_sequence_metrics.h:92-111;
// cc/metrics/frame_sequence_tracker_collection.h:29-31. These are native owned
// result values, not a second tracker or a copy of compositor owner state.
#[derive(Clone, Debug)]
pub struct Jank {
    pub start_time: crate::adapter::time::TimeTicks,
    pub duration: crate::adapter::time::TimeDelta,
}
#[derive(Clone, Debug, Default)]
pub struct CustomReportData {
    pub frames_expected_v3: u32,
    pub frames_dropped_v3: u32,
    pub jank_count_v3: u32,
    pub janks: Vec<Jank>,
}
pub type CustomTrackerResults = std::collections::BTreeMap<i32, CustomReportData>;
// Source dependency: cc/metrics/event_metrics.h:39-74,272-278,725-729.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EventType {
    kMousePressed,
    kMouseReleased,
    kMouseWheel,
    kKeyPressed,
    kKeyReleased,
    kTouchPressed,
    kTouchReleased,
    kTouchMoved,
    kGestureScrollBegin,
    kGestureScrollUpdate,
    kGestureScrollEnd,
    kGestureDoubleTap,
    kGestureLongPress,
    kGestureLongTap,
    kGestureShowPress,
    kGestureTap,
    kGestureTapCancel,
    kGestureTapDown,
    kGestureTapUnconfirmed,
    kGestureTwoFingerTap,
    kFirstGestureScrollUpdate,
    kMouseDragged,
    kGesturePinchBegin,
    kGesturePinchEnd,
    kGesturePinchUpdate,
    kInertialGestureScrollUpdate,
    kMouseMoved,
    kInertialGestureScrollEnd,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ScrollType {
    kAutoscroll,
    kScrollbar,
    kTouchscreen,
    kWheel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PinchType {
    kTouchpad,
    kTouchscreen,
}
// Source dependency: cc/metrics/event_latency_tracker.h:25-46. Preserve native
// variant alternatives rather than conflating scroll and pinch numeric values.
pub enum EventInputType {
    None,
    Scroll(ScrollType),
    Pinch(PinchType),
}
pub struct EventLatencyData {
    pub event_type: EventType,
    pub total_latency: crate::adapter::time::TimeDelta,
    pub input_type: EventInputType,
}
