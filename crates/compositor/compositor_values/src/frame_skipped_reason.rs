// Source: cc/scheduler/scheduler.h:48-53. The reason a BeginFrame produced
// no frame; the impl Scheduler decides it and the main thread receives it
// with LayerTreeFrameSink::DidNotProduceFrame's BeginFrameAck.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameSkippedReason {
    kRecoverLatency,
    kNoDamage,
    kWaitingOnMain,
    kDrawThrottled,
}
