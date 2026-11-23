//! Main-thread callback values named by cc::Layer and viz::CopyOutputRequest.
//! Moved from compositor/src/runtime/types.rs, which re-exports them.
use crate::adapter::{
    geometry::PointF, input_types::ElementId, main_layer::LayerDidScrollCallback,
};
use std::sync::Arc;

/// Strong native sequenced-task-runner capability, supplied by the embedding.
/// No worker/thread/clock implementation is created by the compositor family.
// Source: components/viz/common/frame_sinks/copy_output_request.h
//         (result_task_runner_).
pub trait ResultTaskRunner: Send + Sync {
    fn PostTask(&self, task: Box<dyn FnOnce() + Send>);
}
pub type TaskRunner = Arc<dyn ResultTaskRunner>;

/// Native base::RepeatingCallback void(PointF,ElementId). Cloning shares only
/// the callback allocation, not any compositor tree or Layer state.
// Source: cc/layers/layer.h did_scroll_callback_.
#[derive(Clone, Default)]
pub struct DidScrollCallback(pub Option<Arc<dyn Fn(PointF, ElementId) + Send + Sync>>);
impl LayerDidScrollCallback for DidScrollCallback {
    fn is_null(&self) -> bool {
        self.0.is_none()
    }
    fn Run(&self, offset: PointF, element: ElementId) {
        (self.0.as_ref().expect("native null callback invocation"))(offset, element)
    }
}
